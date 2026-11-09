//! The runner (`P-01`): a long-lived process that owns the session's game processes, holds the
//! tokens, routes orders to machines and writes the result file. Clients and extensions reach it
//! on `127.0.0.1` with a token (`SEC-01`).

pub mod control;
pub mod errors;
pub mod record;
pub mod reload;
pub mod run;
pub mod session;

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shakedown_protocol::Hello;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::error::CliError;
use crate::logs::Redactor;
use crate::paths;
use crate::wire::{Event, Request};
use session::Session;

/// How long the runner stays up with no session and no client before it exits by itself.
pub const IDLE_EXIT: Duration = Duration::from_secs(60);
/// How long a connection has to send its hello.
const HELLO_CEILING: Duration = Duration::from_secs(5);

pub struct Runner {
    pub token: String,
    pub port: u16,
    /// The installed `@shakedown` folder, loaded as the last mod of every game process.
    pub addon: PathBuf,
    state: Mutex<State>,
    runs: AtomicU32,
    log: Mutex<Option<std::fs::File>>,
    /// The active run's progress for `status` (`INS-02`): id, done, total, current check.
    pub(crate) progress: Mutex<Option<Value>>,
    /// What replaces the Steam identity in everything the runner returns or stores (`LOG-02`).
    redactor: Mutex<Redactor>,
    /// The project and key of the check execution running now, for `logs --check` (`LOG-03`).
    pub(crate) executing: Mutex<Option<(String, String)>>,
    /// The last start that failed, with its reason, which `status` shows until the next start or
    /// `stop` (`INS-02`).
    failed: Mutex<Option<Value>>,
    /// The last reload, in the Reload shape, for `status` (`INS-02`).
    pub(crate) last_reload: Mutex<Option<Value>>,
}

struct State {
    session: Option<Arc<Session>>,
    clients: usize,
    idle_since: Instant,
    /// Set once the runner has decided to exit; a client that connects then gets no answer.
    closing: bool,
}

impl Runner {
    /// Binds the runner's port on loopback and starts accepting connections.
    pub async fn bind(addon: PathBuf, log: Option<std::fs::File>) -> std::io::Result<Arc<Runner>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let runner = Arc::new(Runner {
            token: paths::token("skd_r_"),
            port,
            addon,
            state: Mutex::new(State {
                session: None,
                clients: 0,
                idle_since: Instant::now(),
                closing: false,
            }),
            runs: AtomicU32::new(0),
            log: Mutex::new(log),
            progress: Mutex::new(None),
            redactor: Mutex::new(Redactor::default()),
            executing: Mutex::new(None),
            failed: Mutex::new(None),
            last_reload: Mutex::new(None),
        });
        let me = Arc::clone(&runner);
        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                let me = Arc::clone(&me);
                tokio::spawn(async move { me.connection(stream).await });
            }
        });
        Ok(runner)
    }

    pub fn log(&self, line: &str) {
        let line = self.redactor().redact(line);
        if let Some(file) = self.log.lock().unwrap().as_mut() {
            let _ = writeln!(file, "{} {line}", shakedown_protocol::time::now());
        }
    }

    pub fn redactor(&self) -> Redactor {
        self.redactor.lock().unwrap().clone()
    }

    pub(crate) fn set_redactor(&self, redactor: Redactor) {
        *self.redactor.lock().unwrap() = redactor;
    }

    pub fn session(&self) -> Option<Arc<Session>> {
        self.state.lock().unwrap().session.clone()
    }

    pub(crate) fn set_session(&self, session: Option<Arc<Session>>) {
        let mut state = self.state.lock().unwrap();
        state.session = session;
        state.idle_since = Instant::now();
    }

    pub fn next_run_number(&self) -> u32 {
        self.runs.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Once the runner has had no session and no client for `IDLE_EXIT`, removes `runner.json`
    /// and returns true. Both happen under the state lock, so no client is accepted after it.
    fn close_if_idle(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        if state.session.is_none() && state.clients == 0 && state.idle_since.elapsed() >= IDLE_EXIT {
            state.closing = true;
            let _ = std::fs::remove_file(paths::runner_file());
            return true;
        }
        false
    }

    /// The process id at the other end of a connection, from the system's TCP table (`SEC-03`).
    fn peer_pid(&self, stream: &TcpStream) -> Option<u32> {
        let peer = stream.peer_addr().ok()?.port();
        crate::win::tcp_peer_pid(peer, self.port)
    }

    /// Logs a refused connection in the form of `contract/cli.md` and closes it.
    fn refuse(&self, who: &str, pid: Option<u32>, why: &str) {
        let pid = pid.map_or_else(|| "unknown".to_string(), |p| p.to_string());
        self.log(&format!("refused {who} connection from pid {pid}: {why}"));
    }

    async fn connection(self: Arc<Self>, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let pid = self.peer_pid(&stream);
        let (read, write) = stream.into_split();
        let mut lines = BufReader::new(read).lines();
        let first = tokio::time::timeout(HELLO_CEILING, lines.next_line()).await;
        let Ok(Ok(Some(first))) = first else {
            self.refuse("client", pid, "not-hello");
            return;
        };
        match serde_json::from_str::<Hello>(&first) {
            Ok(Hello::Client { version, token }) => {
                if token != self.token {
                    self.refuse("client", pid, "bad-token");
                    return;
                }
                self.client(version, lines, write).await;
            }
            Ok(Hello::Extension {
                session,
                machine,
                token,
            }) => {
                let current = self.session();
                let Some(s) = current.filter(|s| s.id == session && s.token == token && s.has_machine(&machine)) else {
                    self.refuse("extension", pid, "bad-token");
                    return;
                };
                // Only the process the runner started for that machine may speak for it (SEC-03).
                if pid.is_none() || pid != s.machine_pid(&machine) {
                    self.refuse("extension", pid, "unknown-process");
                    return;
                }
                s.extension(&self, machine, lines, write).await;
            }
            _ => self.refuse("client", pid, "not-hello"),
        }
    }

    async fn client(
        self: &Arc<Self>,
        _version: String,
        mut lines: tokio::io::Lines<BufReader<tokio::net::tcp::OwnedReadHalf>>,
        mut write: tokio::net::tcp::OwnedWriteHalf,
    ) {
        {
            let mut state = self.state.lock().unwrap();
            if state.closing {
                return;
            }
            state.clients += 1;
        }
        let hello = serde_json::to_string(&Hello::Runner {
            version: shakedown_protocol::VERSION.into(),
        })
        .unwrap();
        if write
            .write_all(
                format!(
                    "{hello}
"
                )
                .as_bytes(),
            )
            .await
            .is_err()
        {
            self.client_left();
            return;
        }
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        let me = Arc::clone(self);
        let writer = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Everything a client prints comes through here, redacted (LOG-02).
                let mut value = serde_json::to_value(&event).unwrap();
                me.redactor().redact_value(&mut value);
                let line = value.to_string();
                if write.write_all(format!("{line}\n").as_bytes()).await.is_err() {
                    break;
                }
            }
        });
        let mut queued: Option<String> = None;
        loop {
            let line = match queued.take() {
                Some(line) => line,
                None => match lines.next_line().await {
                    Ok(Some(line)) => line,
                    _ => break,
                },
            };
            let request = match serde_json::from_str::<Request>(&line) {
                Ok(request) => request,
                Err(e) => {
                    let _ = tx.send(Event::Error {
                        error: CliError::framework(format!("The runner could not read a request: {e}.")),
                    });
                    continue;
                }
            };
            if !request.cancellable() {
                let _ = tx.send(self.request(request, &tx).await);
                continue;
            }
            // A client that goes away before the answer cancels the request: dropping it stops
            // the work it started in the game (AGT-04).
            let answer = self.request(request, &tx);
            tokio::pin!(answer);
            tokio::select! {
                event = &mut answer => {
                    let _ = tx.send(event);
                }
                next = lines.next_line() => match next {
                    Ok(Some(line)) => {
                        queued = Some(line);
                        let _ = tx.send(answer.await);
                    }
                    _ => {
                        self.log("a client went away during its request; the request is cancelled");
                        break;
                    }
                },
            }
        }
        drop(tx);
        let _ = writer.await;
        self.client_left();
    }

    fn client_left(&self) {
        let mut state = self.state.lock().unwrap();
        state.clients -= 1;
        state.idle_since = Instant::now();
    }

    async fn request(self: &Arc<Self>, request: Request, tx: &mpsc::UnboundedSender<Event>) -> Event {
        let progress = |message: String| {
            let _ = tx.send(Event::Progress { message });
        };
        let answer = match request {
            Request::SessionInfo => Ok(Event::Session {
                session: self.session().map(|s| s.view()),
            }),
            Request::SessionStart { project, spec } => {
                if self.session().is_none() {
                    *self.failed.lock().unwrap() = None;
                }
                session::start(self, project, spec, &progress).await.map(|s| {
                    let view = s.view();
                    if s.state() == session::SessionState::Failed {
                        *self.failed.lock().unwrap() = Some(view.clone());
                    }
                    Event::Session { session: Some(view) }
                })
            }
            Request::Status => Ok(Event::Value {
                value: json!({
                    "runner": "running",
                    "runner_pid": std::process::id(),
                    "version": shakedown_protocol::VERSION,
                    "session": self.session().map(|s| s.view()).or_else(|| self.failed.lock().unwrap().clone()),
                    "run": self.progress.lock().unwrap().clone(),
                    "last_reload": self.last_reload.lock().unwrap().clone(),
                }),
            }),
            Request::Reload {
                files,
                checks,
                rebuild,
                wait_s,
            } => reload::reload(self, files, checks, rebuild, wait_s)
                .await
                .map(|value| Event::Value { value }),
            Request::ReserveRun => Ok(Event::Value {
                value: json!({"run_id": run::new_run_id(self)}),
            }),
            Request::Exec {
                machine,
                code,
                args,
                ceiling,
                source,
            } => control::exec(self, &machine, &code, args, ceiling, &source)
                .await
                .map(|value| Event::Value { value }),
            Request::Wait {
                machine,
                condition,
                description,
                args,
                ceiling,
            } => control::wait(self, &machine, &condition, description.as_deref(), args, ceiling)
                .await
                .map(|value| Event::Value { value }),
            Request::Expect {
                machine,
                value,
                op,
                expected,
                args,
                ceiling,
            } => control::expect(self, &machine, &value, &op, expected, args, ceiling)
                .await
                .map(|value| Event::Value { value }),
            Request::Inspect { machine, near, radius } => control::inspect(self, &machine, near.as_deref(), radius)
                .await
                .map(|value| Event::Value { value }),
            Request::Errors { machine, run, check } => {
                control::errors(self, machine.as_deref(), run.as_deref(), check.as_deref())
                    .map(|value| Event::Value { value })
            }
            Request::Reset => self.reset().await.map(|value| Event::Value { value }),
            Request::Logs {
                project,
                machine,
                run,
                check,
                grep,
                last,
            } => run::logs(
                self,
                &project,
                &machine,
                run.as_deref(),
                check.as_deref(),
                grep.as_deref(),
                last,
            )
            .map(|value| Event::Value { value }),
            Request::SessionStop => match self.session() {
                // A kept failed start is cleared; nothing of it runs (INS-02).
                None => match self.failed.lock().unwrap().take() {
                    Some(view) => Ok(Event::Session { session: Some(view) }),
                    None => Err(CliError::not_now("No session runs. Nothing to stop.").tag("no-session")),
                },
                Some(s) => {
                    s.stop(self).await;
                    Ok(Event::Session {
                        session: Some(s.view()),
                    })
                }
            },
            Request::Run {
                project,
                setups,
                checks,
                ids,
                tags,
                run_id,
            } => {
                let emit = |line: serde_json::Value, text: String| {
                    let _ = tx.send(Event::Check { line, text });
                };
                let started = |run_id: String, total: usize| {
                    let _ = tx.send(Event::RunStarted { run_id, total });
                };
                run::run(self, project, setups, checks, ids, tags, run_id, &started, &emit)
                    .await
                    .map(|result| Event::RunDone { result })
            }
        };
        answer.unwrap_or_else(|error| Event::Error { error })
    }
}

impl Runner {
    /// `reset` outside a run (`RST-03`): refused with no session, on a `broken` session with the
    /// reason and the ways out, and while a run or another reset is in progress.
    async fn reset(self: &Arc<Self>) -> Result<Value, CliError> {
        let Some(session) = self.session() else {
            return Err(CliError::not_now(
                "No session runs, so there is nothing to reset. Start one with `shakedown session start --setup <name>`.",
            )
            .tag("no-session"));
        };
        match session.state() {
            session::SessionState::Ready => {}
            session::SessionState::Broken => {
                return Err(CliError::not_now(format!(
                    "Session {} is broken: {}. It cannot be reset. Bring every machine back with `shakedown reload`, which restarts the game of a broken session, or stop it with `shakedown session stop`.",
                    session.id,
                    session.broken_reason()
                ))
                .tag("session-broken"));
            }
            session::SessionState::Running => {
                let progress = self.progress.lock().unwrap().clone().unwrap_or(Value::Null);
                return Err(CliError::not_now(format!(
                    "Session {} is running run {} ({} of {} checks done). Wait for it to end, or stop it with `shakedown run stop`.",
                    session.id, progress["id"], progress["done"], progress["total"]
                )));
            }
            other => {
                return Err(CliError::not_now(format!(
                    "Session {} is {}, so it cannot be reset now. Wait for it to be ready.",
                    session.id,
                    if other == session::SessionState::Reloading {
                        "busy with a reload or a reset"
                    } else {
                        other.as_str()
                    }
                )));
            }
        }
        if !session.transition(session::SessionState::Ready, session::SessionState::Reloading) {
            return Err(CliError::not_now(format!(
                "Session {} became busy; try the reset again when it is ready.",
                session.id
            )));
        }
        let clock = Instant::now();
        let outcome = run::reset(self, &session).await;
        let duration = (clock.elapsed().as_secs_f64() * 1000.0).round() / 1000.0;
        let clock_done = match &outcome {
            Ok(_) => json!({"state": "done", "duration_s": duration, "mismatches": []}),
            Err(m) => {
                json!({"state": "failed", "duration_s": duration, "mismatches": m.split("; ").collect::<Vec<_>>()})
            }
        };
        match outcome {
            Ok(_) => {
                session.transition(session::SessionState::Reloading, session::SessionState::Ready);
            }
            Err(m) => session.set_broken(session.lead(), format!("the reset failed: {m}")),
        }
        Ok(clock_done)
    }
}

/// Appends one line to `runner.log` from outside the runner's own logger: a panic, a console
/// event, the relay.
fn append_log(line: &str) {
    use std::io::Write as _;
    if let Ok(mut log) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths::runner_log())
    {
        let _ = writeln!(log, "{} {line}", shakedown_protocol::time::now());
    }
}

/// `shakedown runner`: runs until it has had no session and no client for `IDLE_EXIT`.
pub fn main() -> u8 {
    // The relay of win::start_runner: starts the runner itself, out of the client's process tree,
    // and ends (SES-08, INT-WIN-02).
    if std::env::var(crate::win::RELAY_VAR).is_ok_and(|v| v == "1") {
        let Ok(exe) = std::env::current_exe() else { return 5 };
        let args = ["runner".to_string()];
        let env = [(crate::win::RELAY_VAR.into(), String::new())];
        // The relay has left the client's job; a job around that one, which forbids breakaway,
        // may still hold it, and then WMI starts the runner outside it.
        let started = crate::win::spawn_breakaway(&exe, &args, &env, true)
            .map(|_| "breakaway")
            .or_else(|_| crate::win::start_by_wmi(&exe).map(|_| "wmi"));
        let what = match &started {
            Ok(way) => format!("started the runner by {way}"),
            Err(e) => format!("cannot start the runner: {e}"),
        };
        append_log(&format!("relay {}: {what}", std::process::id()));
        return if started.is_ok() { 0 } else { 5 };
    }
    let dir = paths::runner_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return 5;
    }
    // A runner has no console: a panic goes to runner.log, or nobody would see why it ended.
    std::panic::set_hook(Box::new(|info| {
        append_log(&format!("runner {} panicked: {info}", std::process::id()))
    }));
    // So does a console control event, which ends a process that has a console.
    crate::win::on_console_event(|event| {
        append_log(&format!("runner {} received console event {event}", std::process::id()))
    });
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths::runner_log())
        .ok();
    let addon = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("@shakedown")))
        .unwrap_or_else(|| PathBuf::from("@shakedown"));
    let rt = tokio::runtime::Runtime::new().expect("a tokio runtime");
    rt.block_on(async move {
        let Ok(runner) = Runner::bind(addon, log).await else {
            return 5;
        };
        // What a runner that died left behind is cleared before any client is served (P-12).
        for line in record::recover() {
            runner.log(&line);
        }
        let pid = std::process::id();
        // The process's own start time, which names it together with its id (P-01).
        let started_at = crate::win::process_started(pid).unwrap_or_else(shakedown_protocol::time::now);
        let file = json!({
            "port": runner.port,
            "token": runner.token,
            "pid": pid,
            "started_at": started_at,
        });
        // runner.json holds the runner token, so only the current user can read it (SEC-02).
        if crate::win::write_private(&paths::runner_file(), file.to_string().as_bytes()).is_err() {
            return 5;
        }
        runner.log(&format!(
            "runner {pid} started on port {}; its process was created at {started_at}",
            runner.port
        ));
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if runner.close_if_idle() {
                runner.log("no session and no client for 60 s; exiting");
                return 0;
            }
        }
    })
}
