//! A session: starting it (`P-02`), its machines and their connections, orders to them (`P-03`)
//! and stopping it (`P-10`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::result::ModVersion;
use shakedown_protocol::{ENV_MACHINE, ENV_RUNNER, ENV_SESSION, ENV_TOKEN};
use tokio::io::{AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{Notify, mpsc, oneshot};

use super::Runner;
use crate::config::Mode;
use crate::error::CliError;
use crate::mission;
use crate::paths;
use crate::win::{self, Child, Job};
use crate::wire::SetupSpec;

/// How often a wait polls a process or a state.
const POLL: Duration = Duration::from_millis(200);
/// How long a stop waits for each process to end (`P-10` step 2).
const STOP_CEILING: Duration = Duration::from_secs(10);
/// The in-game ceiling of the runner's own short orders.
const ORDER_CEILING_S: f64 = 10.0;
/// The grace the runner adds to every in-game ceiling (`OBS-03`).
pub const GRACE: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Ready,
    Running,
    Failed,
    Stopping,
    Stopped,
}

impl SessionState {
    fn as_str(self) -> &'static str {
        match self {
            SessionState::Starting => "starting",
            SessionState::Ready => "ready",
            SessionState::Running => "running",
            SessionState::Failed => "failed",
            SessionState::Stopping => "stopping",
            SessionState::Stopped => "stopped",
        }
    }
}

pub struct Machine {
    pub name: String,
    pub kind: &'static str,
    pub start: u32,
    pub state: &'static str,
    pub profile: PathBuf,
    pub pid: Option<u32>,
    pub process_started_at: Option<String>,
    pub last_answer_at: Option<String>,
    pub game_version: Option<String>,
    pub mods: Vec<ModVersion>,
    process: Option<Child>,
    tx: Option<mpsc::UnboundedSender<String>>,
}

struct Inner {
    state: SessionState,
    machines: Vec<Machine>,
    reason: Option<Value>,
    job: Option<Job>,
    pending: HashMap<String, (String, oneshot::Sender<Value>)>,
}

pub struct Session {
    pub id: String,
    pub token: String,
    pub spec: SetupSpec,
    pub folder: PathBuf,
    pub started_at: String,
    inner: Mutex<Inner>,
    changed: Notify,
    orders: AtomicU64,
}

/// Why an order got no answer.
#[derive(Debug)]
pub enum OrderError {
    /// The machine has no connection to send it on.
    NotConnected,
    /// The connection closed before the answer came.
    Lost,
    /// The runner ceiling passed with no answer.
    Ceiling,
}

impl Session {
    pub fn has_machine(&self, name: &str) -> bool {
        self.inner.lock().unwrap().machines.iter().any(|m| m.name == name)
    }

    pub fn state(&self) -> SessionState {
        self.inner.lock().unwrap().state
    }

    pub(crate) fn set_state(&self, state: SessionState) {
        self.inner.lock().unwrap().state = state;
        self.changed.notify_waiters();
    }

    /// The lead machine: `server` on a dedicated session, `p1` in singleplayer.
    pub fn lead(&self) -> &'static str {
        match self.spec.setup.mode {
            Mode::Singleplayer => "p1",
            Mode::Dedicated => "server",
        }
    }

    pub fn machine_names(&self) -> Vec<String> {
        self.inner
            .lock()
            .unwrap()
            .machines
            .iter()
            .map(|m| m.name.clone())
            .collect()
    }

    /// The machines as the result file records them (`RES-02`).
    pub fn result_machines(&self) -> Vec<shakedown_protocol::result::Machine> {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .map(|m| shakedown_protocol::result::Machine {
                name: m.name.clone(),
                kind: m.kind.to_string(),
                start: m.start,
                game_version: m.game_version.clone().unwrap_or_default(),
                mods: m.mods.clone(),
                end_state: if m.state == "lost" {
                    "lost".into()
                } else if m.state == "stopped" {
                    "stopped".into()
                } else {
                    "ready".into()
                },
            })
            .collect()
    }

    /// The Session shape of `contract/mcp-tools.md`.
    pub fn view(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        json!({
            "id": self.id,
            "setup": self.spec.setup.name,
            "mode": self.spec.setup.mode.as_str(),
            "state": inner.state.as_str(),
            "started_at": self.started_at,
            "folder": self.folder,
            "machines": inner.machines.iter().map(|m| json!({
                "name": m.name,
                "kind": m.kind,
                "start": m.start,
                "state": m.state,
                "pid": m.pid,
                "process_started_at": m.process_started_at,
                "profile": m.profile,
                "last_answer_at": m.last_answer_at,
                "health": {"flag": null, "since": null, "heartbeat": null},
            })).collect::<Vec<_>>(),
            "reason": inner.reason,
        })
    }

    /// Serves one machine's extension connection until it closes.
    pub(crate) async fn extension(
        self: Arc<Self>,
        runner: &Runner,
        machine: String,
        mut lines: Lines<BufReader<OwnedReadHalf>>,
        mut write: OwnedWriteHalf,
    ) {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        {
            let mut inner = self.inner.lock().unwrap();
            let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) else {
                return;
            };
            m.tx = Some(tx);
            if m.state == "launched" {
                m.state = "connected";
            }
        }
        self.changed.notify_waiters();
        runner.log(&format!("{} {machine}: extension connected", self.id));
        let writer = tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                if write.write_all(format!("{line}\n").as_bytes()).await.is_err() {
                    break;
                }
            }
        });
        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<Value>(&line) {
                Ok(message) => self.message(&machine, message),
                Err(e) => runner.log(&format!("{} {machine}: unreadable message: {e}", self.id)),
            }
        }
        writer.abort();
        let mut inner = self.inner.lock().unwrap();
        let stopping = matches!(
            inner.state,
            SessionState::Stopping | SessionState::Stopped | SessionState::Failed
        );
        if let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) {
            m.tx = None;
            if !stopping {
                m.state = "lost";
            }
        }
        // Every order still waiting on this machine is answered by the closed channel.
        inner.pending.retain(|_, (target, _)| *target != machine);
        drop(inner);
        self.changed.notify_waiters();
        runner.log(&format!("{} {machine}: extension disconnected", self.id));
    }

    fn message(&self, machine: &str, message: Value) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) {
            m.last_answer_at = Some(shakedown_protocol::time::now());
            if message["kind"] == "ready" {
                m.state = "ready";
                m.game_version = message["game_version"].as_str().map(String::from);
            }
        }
        if let Some(order) = message["order"].as_str()
            && let Some((_, reply)) = inner.pending.remove(order)
        {
            let _ = reply.send(message);
        }
        drop(inner);
        self.changed.notify_waiters();
    }

    /// Sends an order to a machine (`P-03`) and waits for its answer up to `ceiling`.
    pub async fn order(&self, machine: &str, mut order: Value, ceiling: Duration) -> Result<Value, OrderError> {
        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
        order["id"] = Value::String(id.clone());
        let (reply, answer) = oneshot::channel();
        {
            let mut inner = self.inner.lock().unwrap();
            let tx = inner
                .machines
                .iter()
                .find(|m| m.name == machine)
                .and_then(|m| m.tx.clone())
                .ok_or(OrderError::NotConnected)?;
            inner.pending.insert(id.clone(), (machine.to_string(), reply));
            tx.send(order.to_string()).map_err(|_| OrderError::NotConnected)?;
        }
        match tokio::time::timeout(ceiling, answer).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(_)) => Err(OrderError::Lost),
            Err(_) => {
                self.inner.lock().unwrap().pending.remove(&id);
                Err(OrderError::Ceiling)
            }
        }
    }

    /// Sends `exec` with SQF source and arguments, waiting the in-game ceiling plus the grace.
    pub async fn exec(&self, machine: &str, source: &str, args: Value, ceiling_s: f64) -> Result<Value, OrderError> {
        let order = json!({"kind": "exec", "source": source, "args": args, "ceiling": ceiling_s});
        self.order(machine, order, Duration::from_secs_f64(ceiling_s) + GRACE)
            .await
    }

    /// Stops every process of the session (`P-10`) and removes it from the runner.
    pub async fn stop(&self, runner: &Runner) {
        if self.state() != SessionState::Failed {
            self.set_state(SessionState::Stopping);
        }
        let children: Vec<(String, u32)> = {
            let inner = self.inner.lock().unwrap();
            inner
                .machines
                .iter()
                .filter_map(|m| m.process.as_ref().map(|p| (m.name.clone(), p.pid)))
                .collect()
        };
        {
            let inner = self.inner.lock().unwrap();
            for m in &inner.machines {
                if let Some(p) = &m.process {
                    p.terminate();
                }
            }
        }
        let deadline = Instant::now() + STOP_CEILING;
        loop {
            let left: Vec<u32> = {
                let inner = self.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .filter_map(|m| m.process.as_ref())
                    .filter(|p| win::exited(p).is_none())
                    .map(|p| p.pid)
                    .collect()
            };
            if left.is_empty() {
                break;
            }
            if Instant::now() >= deadline {
                runner.log(&format!(
                    "{}: processes {left:?} outlived the stop; closing the job",
                    self.id
                ));
                break;
            }
            tokio::time::sleep(POLL).await;
        }
        {
            let mut inner = self.inner.lock().unwrap();
            // Closing the job ends anything still in it (P-10 step 4).
            inner.job = None;
            for m in inner.machines.iter_mut() {
                m.tx = None;
                if m.state != "lost" {
                    m.state = "stopped";
                }
            }
            inner.pending.clear();
            if inner.state != SessionState::Failed {
                inner.state = SessionState::Stopped;
            }
        }
        for (name, pid) in children {
            runner.log(&format!("{} {name}: process {pid} stopped", self.id));
        }
        runner.set_session(None);
        self.changed.notify_waiters();
    }
}

/// Starts a session of a setup and returns it once it is `ready` (`P-02`, `SES-01`).
pub async fn start(
    runner: &Arc<Runner>,
    project: PathBuf,
    spec: SetupSpec,
    progress: &(dyn Fn(String) + Sync),
) -> Result<Arc<Session>, CliError> {
    if let Some(s) = runner.session() {
        return Err(CliError::not_now(format!(
            "A session runs already: {} of setup {}. Stop it with `shakedown session stop`, or run against it without --setup.",
            s.id, s.spec.setup.name
        )));
    }
    if spec.setup.mode == Mode::Dedicated {
        return Err(CliError::not_now(format!(
            "Setup {} is dedicated, and this version of Shakedown starts singleplayer sessions only. Use a singleplayer setup.",
            spec.setup.name
        )));
    }
    preconditions(runner, &spec)?;

    let now = SystemTime::now();
    let id = format!("s-{}", shakedown_protocol::time::stamp(now));
    let state_dir = paths::project_state_dir(&project)
        .map_err(|e| CliError::framework(format!("Cannot create .shakedown in {}: {e}.", project.display())))?;
    let folder = state_dir.join("sessions").join(&id);
    let profile = folder.join("machines").join("p1-1");
    let mission_dir = profile
        .join("missions")
        .join(mission::folder_name(&id, &spec.setup.world));
    let io =
        |e: std::io::Error| CliError::framework(format!("Cannot write the session folder {}: {e}.", folder.display()));
    std::fs::create_dir_all(&mission_dir).map_err(io)?;
    std::fs::write(
        mission_dir.join("mission.sqm"),
        mission::mission_sqm(1, spec.setup.spawn),
    )
    .map_err(io)?;
    std::fs::write(mission_dir.join("description.ext"), mission::description_ext(1)).map_err(io)?;
    let autotest = profile.join("autotest.cfg");
    std::fs::write(&autotest, mission::autotest_cfg(&mission_dir)).map_err(io)?;

    let session = Arc::new(Session {
        id: id.clone(),
        token: paths::token("skd_s_"),
        spec: spec.clone(),
        folder: folder.clone(),
        started_at: shakedown_protocol::time::format_system_time(now),
        inner: Mutex::new(Inner {
            state: SessionState::Starting,
            machines: vec![Machine {
                name: "p1".into(),
                kind: "client",
                start: 1,
                state: "launched",
                profile: profile.clone(),
                pid: None,
                process_started_at: None,
                last_answer_at: None,
                game_version: None,
                mods: vec![],
                process: None,
                tx: None,
            }],
            reason: None,
            job: None,
            pending: HashMap::new(),
        }),
        changed: Notify::new(),
        orders: AtomicU64::new(0),
    });
    runner.set_session(Some(Arc::clone(&session)));

    let mods: Vec<String> = spec
        .mods
        .iter()
        .map(|m| m.path.display().to_string())
        .chain(std::iter::once(runner.addon.display().to_string()))
        .collect();
    let args: Vec<String> = vec![
        format!("-autotest={}", autotest.display()),
        format!("-profiles={}", profile.display()),
        "-name=p1".into(),
        format!("-mod={}", mods.join(";")),
        "-filePatching".into(),
        "-debug".into(),
        "-window".into(),
        "-noPause".into(),
        "-noPauseAudio".into(),
        "-noSound".into(),
        "-skipIntro".into(),
        "-noSplash".into(),
        "-world=empty".into(),
    ];
    let env = vec![
        (ENV_RUNNER.to_string(), format!("127.0.0.1:{}", runner.port)),
        (ENV_SESSION.to_string(), id.clone()),
        (ENV_MACHINE.to_string(), "p1".to_string()),
        (ENV_TOKEN.to_string(), session.token.clone()),
    ];
    let launched = Job::new().and_then(|job| {
        let child = job.spawn(&win::game_exe(&spec.game), &args, &spec.game, &env)?;
        Ok((job, child))
    });
    let (job, child) = match launched {
        Ok(v) => v,
        Err(e) => {
            fail(&session, runner, format!("The game could not be started: {e}."), None);
            return Err(CliError::not_now(format!(
                "Session {id} failed: the game could not be started: {e}."
            )));
        }
    };
    progress(format!(
        "session {id}: started p1 (process {}), waiting for it to be ready",
        child.pid
    ));
    runner.log(&format!("{id} p1: started process {}", child.pid));
    {
        let mut inner = session.inner.lock().unwrap();
        inner.job = Some(job);
        let m = &mut inner.machines[0];
        m.pid = Some(child.pid);
        m.process_started_at = Some(child.started_at.clone());
        m.process = Some(child);
    }

    let deadline = Instant::now() + Duration::from_secs(spec.setup.start_timeout);
    loop {
        let (state, exit) = {
            let inner = session.inner.lock().unwrap();
            let m = &inner.machines[0];
            (m.state, m.process.as_ref().and_then(win::exited))
        };
        if state == "ready" {
            break;
        }
        if let Some(code) = exit {
            let cause = if code == 53 {
                "Steam is not running or not logged in (the game exited with code 53)".to_string()
            } else {
                format!("p1 exited with code {code} before it was ready")
            };
            fail(&session, runner, cause.clone(), Some(code));
            session.stop(runner).await;
            return Err(start_failed(&session, &cause));
        }
        if Instant::now() >= deadline {
            let cause = format!(
                "p1 was not ready within the start ceiling of {} s; it was {state}",
                spec.setup.start_timeout
            );
            fail(&session, runner, cause.clone(), None);
            session.stop(runner).await;
            return Err(start_failed(&session, &cause));
        }
        tokio::select! {
            _ = session.changed.notified() => {}
            _ = tokio::time::sleep(POLL) => {}
        }
    }

    // The version of each mod with a patch key, as the machine loaded it (RES-02).
    let patched: Vec<(String, String)> = spec
        .mods
        .iter()
        .filter_map(|m| m.patch.clone().map(|p| (m.name.clone(), p)))
        .collect();
    if !patched.is_empty() {
        let patches: Vec<&str> = patched.iter().map(|(_, p)| p.as_str()).collect();
        let source = r#"_this apply { private _c = configFile >> "CfgPatches" >> _x; if (isClass _c) then { getText (_c >> "versionStr") } else { nil } }"#;
        let answer = session.exec("p1", source, json!(patches), ORDER_CEILING_S).await;
        let versions = answer.ok().map(|a| a["value"].clone()).unwrap_or(Value::Null);
        let mut inner = session.inner.lock().unwrap();
        inner.machines[0].mods = patched
            .iter()
            .enumerate()
            .map(|(i, (name, patch))| ModVersion {
                mod_name: name.clone(),
                patch: patch.clone(),
                version: versions[i].as_str().filter(|v| !v.is_empty()).map(String::from),
            })
            .collect();
    }
    session.set_state(SessionState::Ready);
    progress(format!("session {id}: ready"));
    runner.log(&format!("{id}: ready"));
    Ok(session)
}

/// The preconditions of `SES-03` a singleplayer start needs.
fn preconditions(runner: &Runner, spec: &SetupSpec) -> Result<(), CliError> {
    let mut missing = Vec::new();
    let exe = win::game_exe(&spec.game);
    if !exe.is_file() {
        missing.push(format!(
            "the game executable {} (check `game` in shakedown.toml)",
            exe.display()
        ));
    }
    if !win::steam_running() {
        missing.push("a running Steam client of this user (start Steam and log in)".to_string());
    }
    for m in &spec.mods {
        if !m.path.is_dir() {
            missing.push(format!("the folder {} of mod {}", m.path.display(), m.name));
        }
    }
    for file in ["addons/shakedown_main.pbo", "shakedown_x64.dll"] {
        if !runner.addon.join(file).is_file() {
            missing.push(format!(
                "the framework's addon file {} (install Shakedown again)",
                runner.addon.join(file).display()
            ));
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(CliError::not_now(format!(
            "The session cannot start; missing: {}.",
            missing.join("; ")
        )))
    }
}

fn fail(session: &Session, runner: &Runner, cause: String, _code: Option<u32>) {
    let tail = rpt_tail(&session.inner.lock().unwrap().machines[0].profile, 20);
    let mut inner = session.inner.lock().unwrap();
    inner.state = SessionState::Failed;
    inner.reason = Some(json!({"message": cause, "machine": "p1", "rpt_tail": tail}));
    drop(inner);
    runner.log(&format!("{}: failed: {cause}", session.id));
}

fn start_failed(session: &Session, cause: &str) -> CliError {
    let tail = session
        .inner
        .lock()
        .unwrap()
        .reason
        .as_ref()
        .map(|r| r["rpt_tail"].clone())
        .unwrap_or(Value::Null);
    let lines: Vec<String> = tail
        .as_array()
        .map(|a| a.iter().filter_map(|l| l.as_str().map(String::from)).collect())
        .unwrap_or_default();
    CliError::not_now(format!(
        "Session {} failed: {cause}. Every process it started was stopped.{}",
        session.id,
        if lines.is_empty() {
            String::new()
        } else {
            format!(" The last lines of p1's RPT:\n{}", lines.join("\n"))
        }
    ))
}

/// The last `n` lines of the newest RPT in a profile folder.
pub fn rpt_tail(profile: &std::path::Path, n: usize) -> Vec<String> {
    let newest = std::fs::read_dir(profile)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("rpt")))
        .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let Some(path) = newest else { return vec![] };
    let text = std::fs::read(&path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let lines: Vec<String> = text.lines().map(String::from).collect();
    lines[lines.len().saturating_sub(n)..].to_vec()
}
