//! The runner (`P-01`): a long-lived process that owns the session's game processes, holds the
//! tokens, routes orders to machines and writes the result file. Clients and extensions reach it
//! on `127.0.0.1` with a token (`SEC-01`).

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
        if let Some(file) = self.log.lock().unwrap().as_mut() {
            let _ = writeln!(file, "{} {line}", shakedown_protocol::time::now());
        }
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

    async fn connection(self: Arc<Self>, stream: TcpStream) {
        let _ = stream.set_nodelay(true);
        let (read, write) = stream.into_split();
        let mut lines = BufReader::new(read).lines();
        let first = tokio::time::timeout(HELLO_CEILING, lines.next_line()).await;
        let Ok(Ok(Some(first))) = first else { return };
        match serde_json::from_str::<Hello>(&first) {
            Ok(Hello::Client { version, token }) => {
                if token != self.token {
                    self.log("refused client connection: bad-token");
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
                    self.log(&format!(
                        "refused extension connection for {machine} of {session}: bad-token"
                    ));
                    return;
                };
                s.extension(&self, machine, lines, write).await;
            }
            _ => self.log("refused connection: not-hello"),
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
        let writer = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let line = serde_json::to_string(&event).unwrap();
                if write.write_all(format!("{line}\n").as_bytes()).await.is_err() {
                    break;
                }
            }
        });
        while let Ok(Some(line)) = lines.next_line().await {
            let event = match serde_json::from_str::<Request>(&line) {
                Ok(request) => self.request(request, &tx).await,
                Err(e) => Event::Error {
                    error: CliError::framework(format!("The runner could not read a request: {e}.")),
                },
            };
            let _ = tx.send(event);
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
                session::start(self, project, spec, &progress)
                    .await
                    .map(|s| Event::Session {
                        session: Some(s.view()),
                    })
            }
            Request::Status => Ok(Event::Value {
                value: json!({
                    "runner": "running",
                    "runner_pid": std::process::id(),
                    "version": shakedown_protocol::VERSION,
                    "session": self.session().map(|s| s.view()),
                    "run": self.progress.lock().unwrap().clone(),
                    "last_reload": null,
                }),
            }),
            Request::Reset => self.reset().await.map(|value| Event::Value { value }),
            Request::SessionStop => match self.session() {
                None => Err(CliError::not_now("No session runs. Nothing to stop.")),
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
            } => {
                let emit = |line: serde_json::Value, text: String| {
                    let _ = tx.send(Event::Check { line, text });
                };
                run::run(self, project, setups, checks, ids, tags, &emit)
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
            ));
        };
        match session.state() {
            session::SessionState::Ready => {}
            session::SessionState::Broken => {
                return Err(CliError::not_now(format!(
                    "Session {} is broken: {}. It cannot be reset. Bring every machine back with a game-restart reload, or stop it with `shakedown session stop`.",
                    session.id,
                    session.broken_reason()
                )));
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
                        "busy with a reset"
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

/// `shakedown runner`: runs until it has had no session and no client for `IDLE_EXIT`.
pub fn main() -> u8 {
    let dir = paths::runner_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return 5;
    }
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
        let file = json!({
            "port": runner.port,
            "token": runner.token,
            "pid": std::process::id(),
            "started_at": shakedown_protocol::time::now(),
        });
        // runner.json lives in the user's own local application data folder, which only that
        // user can read (SEC-02).
        if std::fs::write(paths::runner_file(), file.to_string()).is_err() {
            return 5;
        }
        runner.log(&format!(
            "runner {} started on port {}",
            std::process::id(),
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
