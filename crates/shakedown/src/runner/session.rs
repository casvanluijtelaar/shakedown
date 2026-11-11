//! A session: starting it (`P-02`), its machines and their connections, orders to them and the
//! orders one machine routes to another through the runner (`P-03`), and stopping it (`P-10`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
/// The client state at which a joining client is ready to play, `BRIEFING READ` (`INT-AR-09`).
const CLIENT_READY_STATE: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Ready,
    Running,
    Reloading,
    Broken,
    Failed,
    Stopping,
    Stopped,
}

impl SessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionState::Starting => "starting",
            SessionState::Ready => "ready",
            SessionState::Running => "running",
            SessionState::Reloading => "reloading",
            SessionState::Broken => "broken",
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
    /// The mission start count the agent last reported with `ready`.
    pub mission_start: u64,
    /// The baseline the agent captured at its mission start and sent with `ready` (`RST-02`).
    pub baseline: Value,
    /// The agent sent `ready`; a dedicated client is `ready` once the server also sees it at
    /// client state 10 (`P-02` step 7).
    agent_ready: bool,
    /// While its mission restarts or its process is replaced, an order to the machine waits until
    /// it is ready again instead of being sent into the gap (`NFR-RL-04`).
    held: bool,
    process: Option<Child>,
    tx: Option<mpsc::UnboundedSender<String>>,
}

impl Machine {
    fn new(name: &str, kind: &'static str, start: u32, profile: PathBuf) -> Machine {
        Machine {
            name: name.into(),
            kind,
            start,
            state: "launched",
            profile,
            pid: None,
            process_started_at: None,
            last_answer_at: None,
            game_version: None,
            mods: vec![],
            mission_start: 0,
            baseline: Value::Null,
            agent_ready: false,
            held: false,
            process: None,
            tx: None,
        }
    }
}

/// A file of a watched `source` folder as the snapshot saw it (`RLD-01`, `P-08` step 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub hash: u64,
}

struct Inner {
    state: SessionState,
    machines: Vec<Machine>,
    /// The machine starts a mission restart or a game restart replaced, each with the time it
    /// was replaced; the result file lists those that took part in a run (`RES-02`).
    retired: Vec<(Machine, SystemTime)>,
    reason: Option<Value>,
    /// While the session is `reloading`, what it is busy with: the reload with its id and level,
    /// or a reset, which a refusal names (`RLD-01`).
    busy: String,
    job: Option<Job>,
    pending: HashMap<String, (String, oneshot::Sender<Value>)>,
    /// A poll a machine runs for another (`P-03`): its order id to the machine that asked for it
    /// and that machine's reference.
    streams: HashMap<String, (String, String)>,
    /// The reverse: a reference of the asking machine to the polling machine and the order id.
    refs: HashMap<String, (String, String)>,
    /// The script errors the machines' handlers sent (`ERR-01`), until a check execution takes
    /// those stamped with its key.
    errors: Vec<Value>,
    /// Every script error the handlers sent in this session, inside a check or not, the newest
    /// last, for `errors` and `inspect` (`ERR-05`, `INS-01`).
    history: Vec<Value>,
}

/// The most script errors a session keeps for `errors`.
const HISTORY_LIMIT: usize = 1000;

pub struct Session {
    pub id: String,
    pub token: String,
    pub spec: SetupSpec,
    pub folder: PathBuf,
    pub started_at: String,
    /// The copy of the session mission in the game's `MPMissions` folder, for a dedicated session.
    mission_copy: Option<PathBuf>,
    /// The server's `password` and `serverCommandPassword`, random per session (`INT-AR-05`).
    password: String,
    command_password: String,
    inner: Mutex<Inner>,
    changed: Notify,
    orders: AtomicU64,
    /// `audit.jsonl` in the session folder (`SEC-04`).
    audit: Mutex<Option<std::fs::File>>,
    /// What replaces the Steam identity in the audit, as in everything else the runner stores.
    redactor: crate::logs::Redactor,
    /// The id of the check executing now, the source of the orders its code routes.
    pub(crate) check: Mutex<Option<String>>,
    /// The singleplayer `-autotest` list, which every start of `p1` loads (`INT-AR-04`).
    autotest: Option<PathBuf>,
    /// Every file under the watched `source` folders at the last ready state (`RLD-01`).
    pub(crate) snapshot: Mutex<std::collections::BTreeMap<PathBuf, Stamp>>,
}

/// Tells a machine to stop an order's code when the request that sent it is dropped before the
/// answer came (`AGT-04`).
struct CancelOnDrop<'a> {
    session: &'a Session,
    machine: String,
    order: String,
    armed: bool,
}

impl Drop for CancelOnDrop<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.session.inner.lock().unwrap().pending.remove(&self.order);
            self.session
                .push(&self.machine, json!({"kind": "cancel", "order": self.order}));
        }
    }
}

fn machine_view(m: &Machine) -> Value {
    json!({
        "name": m.name,
        "kind": m.kind,
        "start": m.start,
        "state": m.state,
        "pid": m.pid,
        "process_started_at": m.process_started_at,
        "profile": m.profile,
        "last_answer_at": m.last_answer_at,
        "health": {"flag": null, "since": null, "heartbeat": null},
    })
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

/// The machine names of a setup, in the order `server`, `p1`..`pN`, `hc1`..`hcN` (`DM-10`), with
/// their kinds. In singleplayer `p1` alone.
pub fn machine_plan(spec: &SetupSpec) -> Vec<(String, &'static str)> {
    match spec.setup.mode {
        Mode::Singleplayer => vec![("p1".into(), "client")],
        Mode::Dedicated => std::iter::once(("server".to_string(), "server"))
            .chain((1..=spec.setup.clients).map(|n| (format!("p{n}"), "client")))
            .chain((1..=spec.setup.headless).map(|n| (format!("hc{n}"), "headless")))
            .collect(),
    }
}

impl Session {
    /// Appends one `exec` order to `audit.jsonl`: its time, source (`cli`, `mcp` or a check id),
    /// target machine and code (`SEC-04`).
    pub fn audit(&self, source: &str, target: &str, code: &str) {
        use std::io::Write as _;
        let line = json!({
            "time": shakedown_protocol::time::now(),
            "source": source,
            "target": target,
            "code": self.redactor.redact(code),
        });
        if let Some(file) = self.audit.lock().unwrap().as_mut() {
            let _ = writeln!(file, "{line}");
        }
    }

    /// The process id the runner started for a machine, while it runs (`SEC-03`).
    pub fn machine_pid(&self, name: &str) -> Option<u32> {
        self.inner
            .lock()
            .unwrap()
            .machines
            .iter()
            .find(|m| m.name == name)
            .and_then(|m| m.pid)
    }

    pub fn has_machine(&self, name: &str) -> bool {
        self.inner.lock().unwrap().machines.iter().any(|m| m.name == name)
    }

    /// The mission start count a machine last reported, 0 before its first `ready`.
    pub fn mission_start(&self, machine: &str) -> u64 {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .find(|m| m.name == machine)
            .map_or(0, |m| m.mission_start)
    }

    pub fn state(&self) -> SessionState {
        self.inner.lock().unwrap().state
    }

    /// Marks the session `broken` with what broke it; no check runs in it after this (`P-07`).
    pub(crate) fn set_broken(&self, machine: &str, message: String) {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state = SessionState::Broken;
            inner.reason = Some(json!({"message": message, "machine": machine, "rpt_tail": []}));
        }
        self.changed.notify_waiters();
    }

    /// Why the session broke, for the checks that follow (`DM-05`).
    pub fn broken_reason(&self) -> String {
        let inner = self.inner.lock().unwrap();
        inner
            .reason
            .as_ref()
            .and_then(|r| r["message"].as_str().map(String::from))
            .unwrap_or_else(|| "the session broke".into())
    }

    /// Marks the session `ready` and clears what broke it: a reload brought every machine back.
    pub(crate) fn set_ready(&self) {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state = SessionState::Ready;
            inner.reason = None;
        }
        self.changed.notify_waiters();
    }

    pub(crate) fn set_state(&self, state: SessionState) {
        self.inner.lock().unwrap().state = state;
        self.changed.notify_waiters();
    }

    /// Moves the session from `from` to `to` only if it is in `from`; false otherwise.
    pub(crate) fn transition(&self, from: SessionState, to: SessionState) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.state != from {
            return false;
        }
        inner.state = to;
        drop(inner);
        self.changed.notify_waiters();
        true
    }

    /// Moves the session from `from` to `reloading` only if it is in `from`, recording `what` it
    /// is busy with for the refusals of `RLD-01`; false otherwise.
    pub(crate) fn begin_busy(&self, from: SessionState, what: String) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.state != from {
            return false;
        }
        inner.state = SessionState::Reloading;
        inner.busy = what;
        drop(inner);
        self.changed.notify_waiters();
        true
    }

    /// What the `reloading` session is busy with, such as `reload rl-... (level game-restart)` or
    /// `a reset` (`RLD-01`).
    pub(crate) fn busy_with(&self) -> String {
        self.inner.lock().unwrap().busy.clone()
    }

    /// The lead machine: `server` on a dedicated session, `p1` in singleplayer.
    /// The ports of a dedicated session, its base port and the next four, that a program holds
    /// (`SES-03`, `STP-04`); none for singleplayer, which opens no game port.
    fn held_ports(&self) -> Vec<u16> {
        if self.spec.setup.mode != Mode::Dedicated {
            return vec![];
        }
        let base = self.spec.base_port;
        (base..base.saturating_add(5))
            .filter(|p| std::net::UdpSocket::bind(("0.0.0.0", *p)).is_err())
            .collect()
    }

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

    /// Takes the script errors the handlers sent that are stamped with the key of a check
    /// execution (`<run id>#<n>`).
    pub fn take_errors(&self, key: &str) -> Vec<Value> {
        let mut inner = self.inner.lock().unwrap();
        let (taken, kept) = std::mem::take(&mut inner.errors)
            .into_iter()
            .partition(|e| e["key"] == key);
        inner.errors = kept;
        taken
    }

    /// A machine's profile folder, which holds its RPT (`LOG-01`).
    pub fn profile(&self, machine: &str) -> Option<PathBuf> {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .find(|m| m.name == machine)
            .map(|m| m.profile.clone())
    }

    /// The baseline a machine sent with `ready` (`RST-02`).
    pub fn baseline(&self, machine: &str) -> Value {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .find(|m| m.name == machine)
            .map_or(Value::Null, |m| m.baseline.clone())
    }

    /// The machine starts that took part in a run that began at `since`, as the result file
    /// records them (`RES-02`): the starts replaced during the run, then the current ones.
    pub fn result_machines(&self, since: SystemTime) -> Vec<shakedown_protocol::result::Machine> {
        let inner = self.inner.lock().unwrap();
        inner
            .retired
            .iter()
            .filter(|(_, at)| *at >= since)
            .map(|(m, _)| m)
            .chain(inner.machines.iter())
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

    /// The Session shape of `contract/mcp-tools.md`. A machine is listed from the moment its
    /// process is launched (`INS-04`), so a client the start has not launched yet, while the
    /// server loads its mission, is not in it.
    pub fn view(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        json!({
            "id": self.id,
            "setup": self.spec.setup.name,
            "mode": self.spec.setup.mode.as_str(),
            "state": inner.state.as_str(),
            "started_at": self.started_at,
            "folder": self.folder,
            "machines": inner.machines.iter().filter(|m| m.pid.is_some()).map(machine_view).collect::<Vec<_>>(),
            "reason": inner.reason,
        })
    }

    /// One machine in the Machine shape of `contract/mcp-tools.md`.
    pub fn machine(&self, name: &str) -> Option<Value> {
        let inner = self.inner.lock().unwrap();
        inner.machines.iter().find(|m| m.name == name).map(machine_view)
    }

    /// Every script error the session's handlers sent, the oldest first (`ERR-05`).
    pub fn history(&self) -> Vec<Value> {
        self.inner.lock().unwrap().history.clone()
    }

    /// Sends `exec` like [`Session::exec`] and also returns the order's id, which stamps the
    /// script errors the code raises. Dropped before the answer, it tells the machine to stop the
    /// code (`AGT-04`).
    pub async fn exec_tracked(
        &self,
        machine: &str,
        source: &str,
        args: Value,
        ceiling_s: f64,
    ) -> (String, Result<Value, OrderError>) {
        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
        let order = json!({"kind": "exec", "source": source, "args": args, "ceiling": ceiling_s});
        let mut guard = CancelOnDrop {
            session: self,
            machine: machine.to_string(),
            order: id.clone(),
            armed: true,
        };
        let answer = self
            .order_as(machine, order, id.clone(), Duration::from_secs_f64(ceiling_s) + GRACE)
            .await;
        guard.armed = false;
        (id, answer)
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
        // The start this connection belongs to: a machine replaced by a restart keeps its name,
        // and the old start's connection must not touch the new one.
        let start;
        {
            let mut inner = self.inner.lock().unwrap();
            let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) else {
                return;
            };
            start = m.start;
            m.tx = Some(tx);
            if m.state == "launched" {
                m.state = "connected";
            }
            // A game client's window is titled by its machine name once its extension connects
            // (INT-AR-02); the window may come a little later, so it is tried for up to 30 s.
            if let (Some(pid), "client") = (m.pid, m.kind) {
                let title = format!("Arma 3 - {machine} ({})", self.id);
                tokio::spawn(async move {
                    for _ in 0..60 {
                        if win::set_window_title(pid, &title) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                });
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
                Ok(message) => self.message(&machine, start, message),
                Err(e) => runner.log(&format!("{} {machine}: unreadable message: {e}", self.id)),
            }
        }
        writer.abort();
        let mut inner = self.inner.lock().unwrap();
        let stopping = matches!(
            inner.state,
            SessionState::Stopping | SessionState::Stopped | SessionState::Failed
        );
        let current = inner.machines.iter().any(|m| m.name == machine && m.start == start);
        if let Some(m) = inner
            .machines
            .iter_mut()
            .find(|m| m.name == machine && m.start == start)
        {
            m.tx = None;
            if !stopping {
                m.state = "lost";
            }
        }
        // Every order still waiting on this machine is answered by the closed channel.
        if current {
            inner.pending.retain(|_, (target, _)| *target != machine);
        }
        drop(inner);
        self.changed.notify_waiters();
        runner.log(&format!("{} {machine}: extension disconnected", self.id));
    }

    fn message(self: &Arc<Self>, machine: &str, start: u32, message: Value) {
        let kind = message["kind"].as_str().unwrap_or_default().to_string();
        let mut inner = self.inner.lock().unwrap();
        if !inner.machines.iter().any(|m| m.name == machine && m.start == start) {
            return;
        }
        if let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) {
            m.last_answer_at = Some(shakedown_protocol::time::now());
            if kind == "ready" {
                m.agent_ready = true;
                // A dedicated client is ready once the server sees it at client state 10, which
                // the start asks the server (P-02 step 7).
                if !(m.kind == "client" && self.spec.setup.mode == Mode::Dedicated) {
                    m.state = "ready";
                    m.held = false;
                }
                m.game_version = message["game_version"].as_str().map(String::from);
                m.mission_start = message["mission_start"].as_u64().unwrap_or(0);
                m.baseline = message["baseline"].clone();
            }
        }
        match kind.as_str() {
            // A script error the machine's handler caught (ERR-01).
            "error" => {
                inner.history.push(message.clone());
                if inner.history.len() > HISTORY_LIMIT {
                    inner.history.remove(0);
                }
                inner.errors.push(message);
            }
            // A machine asks the runner to send an order to another (P-03 step 1).
            "route" => {
                drop(inner);
                let me = Arc::clone(self);
                let from = machine.to_string();
                tokio::spawn(async move { me.route(from, message).await });
                return;
            }
            // The machine that asked for a poll stops it.
            "route-stop" => {
                let reference = message["ref"].as_str().unwrap_or_default();
                if let Some((target, order)) = inner.refs.remove(reference) {
                    let tx = inner
                        .machines
                        .iter()
                        .find(|m| m.name == target)
                        .and_then(|m| m.tx.clone());
                    if let Some(tx) = tx {
                        let _ =
                            tx.send(json!({"id": format!("{order}-stop"), "kind": "stop", "poll": order}).to_string());
                    }
                }
            }
            // A value a poll read, forwarded to the machine that asked for the poll.
            "poll" => {
                let order = message["order"].as_str().unwrap_or_default();
                if let Some((from, reference)) = inner.streams.get(order).cloned() {
                    let tx = inner
                        .machines
                        .iter()
                        .find(|m| m.name == from)
                        .and_then(|m| m.tx.clone());
                    if let Some(tx) = tx {
                        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
                        let _ = tx.send(
                            json!({
                                "id": id, "kind": "routed-poll", "ref": reference,
                                "value": message["value"], "reads": message["reads"], "error": message["error"],
                            })
                            .to_string(),
                        );
                    }
                }
            }
            _ => {
                if let Some(order) = message["order"].as_str()
                    && let Some((_, reply)) = inner.pending.remove(order)
                {
                    let _ = reply.send(message);
                }
            }
        }
        drop(inner);
        self.changed.notify_waiters();
    }

    /// Sends the order a machine routed to its target and gives that machine the answer (`P-03`).
    /// A target that does not answer within the order's ceiling plus the grace is `lost`, the
    /// answer says so, and the session goes `broken`.
    async fn route(self: Arc<Self>, from: String, message: Value) {
        let target = message["target"].as_str().unwrap_or_default().to_string();
        let reference = message["ref"].as_str().unwrap_or_default().to_string();
        let mut order = message["order"].clone();
        if matches!(order["kind"].as_str(), Some("exec" | "poll")) {
            let check = self.check.lock().unwrap().clone().unwrap_or_else(|| "check".into());
            self.audit(&check, &target, order["source"].as_str().unwrap_or_default());
        }
        let ceiling = order["ceiling"].as_f64().unwrap_or(ORDER_CEILING_S).max(0.0);
        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
        order["id"] = Value::String(id.clone());
        if order["kind"] == "poll" {
            let mut inner = self.inner.lock().unwrap();
            inner.streams.insert(id.clone(), (from.clone(), reference.clone()));
            inner.refs.insert(reference.clone(), (target.clone(), id.clone()));
        }
        let answer = self
            .order_as(&target, order, id.clone(), Duration::from_secs_f64(ceiling) + GRACE)
            .await;
        {
            let mut inner = self.inner.lock().unwrap();
            inner.streams.remove(&id);
            inner.refs.remove(&reference);
        }
        let answer = match answer {
            Ok(a) => a,
            Err(e) => {
                let why = match e {
                    OrderError::NotConnected | OrderError::Lost => format!("{target} has no connection to the runner"),
                    OrderError::Ceiling => format!(
                        "{target} did not answer within the ceiling of {ceiling} s plus the grace of {} s",
                        GRACE.as_secs()
                    ),
                };
                {
                    let mut inner = self.inner.lock().unwrap();
                    if let Some(m) = inner.machines.iter_mut().find(|m| m.name == target) {
                        m.state = "lost";
                    }
                }
                if self.state() != SessionState::Stopping {
                    self.set_broken(&target, why.clone());
                }
                json!({"state": "lost", "message": why})
            }
        };
        self.push(&from, json!({"kind": "routed", "ref": reference, "answer": answer}));
    }

    /// Sends an order that expects no answer.
    fn push(&self, machine: &str, mut order: Value) -> bool {
        order["id"] = Value::String(format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1));
        let inner = self.inner.lock().unwrap();
        let tx = inner
            .machines
            .iter()
            .find(|m| m.name == machine)
            .and_then(|m| m.tx.clone());
        tx.is_some_and(|tx| tx.send(order.to_string()).is_ok())
    }

    /// Sends an order to a machine (`P-03`) and waits for its answer up to `ceiling`.
    pub async fn order(&self, machine: &str, order: Value, ceiling: Duration) -> Result<Value, OrderError> {
        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
        self.order_as(machine, order, id, ceiling).await
    }

    async fn order_as(
        &self,
        machine: &str,
        mut order: Value,
        id: String,
        ceiling: Duration,
    ) -> Result<Value, OrderError> {
        order["id"] = Value::String(id.clone());
        // An order to a machine whose mission restarts waits until the machine is ready again,
        // within the start ceiling the restart itself has, and then has its whole ceiling, so no
        // order is sent into the gap and lost (NFR-RL-04).
        let deadline = Instant::now() + Duration::from_secs(self.spec.setup.start_timeout);
        loop {
            let notified = self.changed.notified();
            let held = {
                let inner = self.inner.lock().unwrap();
                inner.machines.iter().any(|m| m.name == machine && m.held)
            };
            if !held {
                break;
            }
            if Instant::now() >= deadline {
                return Err(OrderError::Ceiling);
            }
            tokio::select! {
                _ = notified => {}
                _ = tokio::time::sleep(POLL) => {}
            }
        }
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

    /// Sends an order like [`Session::order`]; dropped before the answer, it tells the machine to
    /// stop the order's code (`AGT-04`, `RUN-05`).
    pub async fn order_tracked(&self, machine: &str, order: Value, ceiling: Duration) -> Result<Value, OrderError> {
        let id = format!("o{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
        let mut guard = CancelOnDrop {
            session: self,
            machine: machine.to_string(),
            order: id.clone(),
            armed: true,
        };
        let answer = self.order_as(machine, order, id, ceiling).await;
        guard.armed = false;
        answer
    }

    /// Sends `exec` with SQF source and arguments, waiting the in-game ceiling plus the grace.
    pub async fn exec(&self, machine: &str, source: &str, args: Value, ceiling_s: f64) -> Result<Value, OrderError> {
        let order = json!({"kind": "exec", "source": source, "args": args, "ceiling": ceiling_s});
        self.order(machine, order, Duration::from_secs_f64(ceiling_s) + GRACE)
            .await
    }

    /// Stops every process of the session (`P-10`) and removes it from the runner: the game
    /// clients and headless clients first, then the server.
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
        for wave in [false, true] {
            {
                let inner = self.inner.lock().unwrap();
                for m in inner.machines.iter().filter(|m| (m.kind == "server") == wave) {
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
                        .filter(|m| (m.kind == "server") == wave)
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
            inner.streams.clear();
            inner.refs.clear();
            if inner.state != SessionState::Failed {
                inner.state = SessionState::Stopped;
            }
        }
        // The stop returns only once no process of the session runs and, for a dedicated session,
        // no port of it is held (STP-04), at most STOP_CEILING after the job closed.
        let deadline = Instant::now() + STOP_CEILING;
        loop {
            let alive: Vec<u32> = {
                let inner = self.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .filter_map(|m| m.process.as_ref())
                    .filter(|p| win::exited(p).is_none())
                    .map(|p| p.pid)
                    .collect()
            };
            let held = self.held_ports();
            if alive.is_empty() && held.is_empty() {
                break;
            }
            if Instant::now() >= deadline {
                runner.log(&format!(
                    "{}: after the stop, processes {alive:?} still run and ports {held:?} are held",
                    self.id
                ));
                break;
            }
            tokio::time::sleep(POLL).await;
        }
        if let Some(copy) = &self.mission_copy {
            let _ = std::fs::remove_dir_all(copy);
        }
        let end = self.state();
        super::record::write_end(&self.folder, &self.id, end.as_str(), None);
        super::record::remove_session(&self.id);
        // Every RPT of the session's machines, redacted once nothing writes it any more (LOG-02).
        let redactor = runner.redactor();
        let profiles: Vec<PathBuf> = self
            .inner
            .lock()
            .unwrap()
            .machines
            .iter()
            .map(|m| m.profile.clone())
            .collect();
        for profile in profiles {
            for entry in std::fs::read_dir(&profile).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("rpt"))
                    && let Err(e) = redactor.redact_file(&path)
                {
                    runner.log(&format!("{}: could not redact {}: {e}", self.id, path.display()));
                }
            }
        }
        for (name, pid) in children {
            runner.log(&format!("{} {name}: process {pid} stopped", self.id));
        }
        runner.set_session(None);
        self.changed.notify_waiters();
    }

    fn fail(&self, runner: &Runner, machine: &str, cause: String) {
        let tail = {
            let inner = self.inner.lock().unwrap();
            inner
                .machines
                .iter()
                .find(|m| m.name == machine)
                .map(|m| rpt_tail(&m.profile, 20))
                .unwrap_or_default()
        };
        let mut inner = self.inner.lock().unwrap();
        inner.state = SessionState::Failed;
        inner.reason = Some(json!({"message": cause, "machine": machine, "rpt_tail": tail}));
        drop(inner);
        runner.log(&format!("{}: failed: {cause}", self.id));
    }

    /// Launches the named machines' processes in the session's job (`P-02` step 5).
    fn launch(&self, runner: &Runner, names: &[String], progress: &(dyn Fn(String) + Sync)) -> Result<(), String> {
        let dedicated = self.spec.setup.mode == Mode::Dedicated;
        // A dedicated server and a headless client load extensions only from inside the game
        // folder, so a dedicated session loads the framework's addon through its junction there
        // (INT-AR-06).
        let addon = if dedicated {
            self.spec.game.join("@shakedown")
        } else {
            runner.addon.clone()
        };
        let mods: Vec<String> = self
            .spec
            .mods
            .iter()
            .map(|m| m.path.display().to_string())
            .chain(std::iter::once(addon.display().to_string()))
            .collect();
        let port = self.spec.base_port.to_string();
        for name in names {
            let (kind, profile, password) = {
                let inner = self.inner.lock().unwrap();
                let m = inner
                    .machines
                    .iter()
                    .find(|m| &m.name == name)
                    .expect("a planned machine");
                (m.kind, m.profile.clone(), self.password())
            };
            std::fs::create_dir_all(&profile).map_err(|e| format!("cannot create {}: {e}", profile.display()))?;
            let common = [
                format!("-profiles={}", profile.display()),
                format!("-name={name}"),
                format!("-mod={}", mods.join(";")),
                "-filePatching".into(),
                "-debug".into(),
            ];
            let window = [
                "-window",
                "-noPause",
                "-noPauseAudio",
                "-noSound",
                "-skipIntro",
                "-noSplash",
                "-world=empty",
            ]
            .map(String::from);
            let (exe, args): (PathBuf, Vec<String>) = match (dedicated, kind) {
                (false, _) => (
                    win::game_exe(&self.spec.game),
                    std::iter::once(format!(
                        "-autotest={}",
                        self.autotest
                            .clone()
                            .unwrap_or_else(|| profile.join("autotest.cfg"))
                            .display()
                    ))
                    .chain(common.iter().cloned())
                    .chain(window.iter().cloned())
                    .collect(),
                ),
                (true, "server") => (
                    win::server_exe(&self.spec.game),
                    [
                        format!("-port={port}"),
                        format!("-config={}", self.folder.join("server.cfg").display()),
                    ]
                    .into_iter()
                    .chain(common.iter().cloned())
                    .chain([
                        "-autoInit".to_string(),
                        format!("-pid={}", profile.join("server.pid").display()),
                    ])
                    .collect(),
                ),
                (true, "headless") => (
                    win::server_exe(&self.spec.game),
                    [
                        "-client".to_string(),
                        "-connect=127.0.0.1".into(),
                        format!("-port={port}"),
                        format!("-password={password}"),
                    ]
                    .into_iter()
                    .chain(common.iter().cloned())
                    .collect(),
                ),
                (true, _) => (
                    win::game_exe(&self.spec.game),
                    [
                        "-connect=127.0.0.1".to_string(),
                        format!("-port={port}"),
                        format!("-password={password}"),
                    ]
                    .into_iter()
                    .chain(common.iter().cloned())
                    .chain(window.iter().cloned())
                    .collect(),
                ),
            };
            let env = vec![
                (ENV_RUNNER.to_string(), format!("127.0.0.1:{}", runner.port)),
                (ENV_SESSION.to_string(), self.id.clone()),
                (ENV_MACHINE.to_string(), name.clone()),
                (ENV_TOKEN.to_string(), self.token.clone()),
            ];
            let mut inner = self.inner.lock().unwrap();
            if inner.job.is_none() {
                inner.job = Some(Job::new().map_err(|e| format!("cannot create the job object: {e}"))?);
            }
            let child = inner
                .job
                .as_ref()
                .unwrap()
                .spawn(&exe, &args, &self.spec.game, &env)
                .map_err(|e| format!("{name} could not be started: {e}"))?;
            progress(format!("session {}: started {name} (process {})", self.id, child.pid));
            runner.log(&format!("{} {name}: started process {}", self.id, child.pid));
            super::record::add_process(&self.id, child.pid, &child.started_at);
            let m = inner.machines.iter_mut().find(|m| &m.name == name).unwrap();
            m.pid = Some(child.pid);
            m.process_started_at = Some(child.started_at.clone());
            m.process = Some(child);
        }
        Ok(())
    }

    fn password(&self) -> String {
        self.password.clone()
    }

    fn command_password(&self) -> String {
        self.command_password.clone()
    }
}

/// Starts a session of a setup and returns it once it is `ready` or `failed` (`P-02`, `SES-01`).
/// A refusal before any process starts is an error; a start that failed returns the session in
/// state `failed`, every process stopped.
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
        ))
        .tag("session-exists"));
    }
    preconditions(runner, &spec)?;
    // The names of the logged-in Steam account, kept in memory for redaction (INT-STM-04).
    runner.set_redactor(crate::logs::Redactor::from_steam());
    let dedicated = spec.setup.mode == Mode::Dedicated;

    let now = SystemTime::now();
    let id = format!("s-{}", shakedown_protocol::time::stamp(now));
    let state_dir = paths::project_state_dir(&project)
        .map_err(|e| CliError::framework(format!("Cannot create .shakedown in {}: {e}.", project.display())))?;
    let folder = state_dir.join("sessions").join(&id);
    let plan = machine_plan(&spec);
    let profile = |name: &str| folder.join("machines").join(format!("{name}-1"));
    let io =
        |e: std::io::Error| CliError::framework(format!("Cannot write the session folder {}: {e}.", folder.display()));
    let mission_name = mission::folder_name(&id, &spec.setup.world);
    let sqm = mission::mission_sqm(spec.setup.clients, spec.setup.headless, spec.setup.spawn);
    let ext = mission::description_ext(spec.setup.clients + spec.setup.headless);
    let token = paths::token("skd_s_");
    std::fs::create_dir_all(&folder).map_err(io)?;
    super::record::add_session(&id, &folder);
    let mut mission_copy = None;
    if dedicated {
        let dir = folder.join("mission").join(&mission_name);
        std::fs::create_dir_all(&dir).map_err(io)?;
        std::fs::write(dir.join("mission.sqm"), &sqm).map_err(io)?;
        std::fs::write(dir.join("description.ext"), &ext).map_err(io)?;
        let copy = spec.game.join("MPMissions").join(&mission_name);
        super::record::add_path(&id, &copy);
        std::fs::create_dir_all(&copy)
            .and_then(|_| std::fs::write(copy.join("mission.sqm"), &sqm))
            .and_then(|_| std::fs::write(copy.join("description.ext"), &ext))
            .map_err(|e| CliError::framework(format!("Cannot copy the session mission to {}: {e}.", copy.display())))?;
        mission_copy = Some(copy);
    } else {
        let p1 = profile("p1");
        let dir = p1.join("missions").join(&mission_name);
        std::fs::create_dir_all(&dir).map_err(io)?;
        std::fs::write(dir.join("mission.sqm"), &sqm).map_err(io)?;
        std::fs::write(dir.join("description.ext"), &ext).map_err(io)?;
        std::fs::write(p1.join("autotest.cfg"), mission::autotest_cfg(&dir)).map_err(io)?;
    }

    let session = Arc::new(Session {
        id: id.clone(),
        token,
        spec: spec.clone(),
        folder: folder.clone(),
        started_at: shakedown_protocol::time::format_system_time(now),
        mission_copy,
        // 32 hexadecimal digits of a random value each; neither is the session token.
        password: paths::token("")[..32].to_string(),
        command_password: paths::token("")[..32].to_string(),
        inner: Mutex::new(Inner {
            state: SessionState::Starting,
            machines: plan.iter().map(|(n, k)| Machine::new(n, k, 1, profile(n))).collect(),
            retired: Vec::new(),
            reason: None,
            busy: String::new(),
            job: None,
            pending: HashMap::new(),
            streams: HashMap::new(),
            refs: HashMap::new(),
            errors: Vec::new(),
            history: Vec::new(),
        }),
        changed: Notify::new(),
        orders: AtomicU64::new(0),
        audit: Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(folder.join("audit.jsonl"))
                .ok(),
        ),
        redactor: runner.redactor(),
        check: Mutex::new(None),
        autotest: (!dedicated).then(|| profile("p1").join("autotest.cfg")),
        snapshot: Mutex::new(Default::default()),
    });
    if dedicated {
        std::fs::write(
            folder.join("server.cfg"),
            mission::server_cfg(
                &id,
                &spec.setup.world,
                &session.password(),
                &session.command_password(),
                spec.setup.clients + spec.setup.headless,
            ),
        )
        .map_err(io)?;
    }
    runner.set_session(Some(Arc::clone(&session)));

    if let Err((machine, cause)) = session.bring_up(runner, true, progress).await {
        session.fail(runner, &machine, cause);
        session.stop(runner).await;
        return Ok(session);
    }
    session.versions().await;
    session.take_snapshot();
    session.set_state(SessionState::Ready);
    progress(format!("session {id}: ready"));
    runner.log(&format!("{id}: ready"));
    Ok(session)
}

impl Session {
    /// Brings every machine of the session to `ready` (`P-02` steps 5 to 7): with `launch_server`
    /// the lead's process is launched first; every machine not yet launched is launched at once
    /// when the lead is ready, so no client tries to join a server that is not up yet. A machine
    /// whose process exits, or the start ceiling of `SES-04`, ends it with that machine and the
    /// cause. Used by a start, and by a mission restart and a game restart (`P-08`).
    async fn bring_up(
        &self,
        runner: &Runner,
        launch_server: bool,
        progress: &(dyn Fn(String) + Sync),
    ) -> Result<(), (String, String)> {
        let spec = &self.spec;
        let dedicated = spec.setup.mode == Mode::Dedicated;
        let plan = machine_plan(spec);
        if launch_server {
            let first = vec![plan[0].0.clone()];
            if let Err(cause) = self.launch(runner, &first, progress) {
                return Err((first[0].clone(), format!("The game could not be started: {cause}")));
            }
        }
        let rest: Vec<String> = {
            let inner = self.inner.lock().unwrap();
            inner
                .machines
                .iter()
                .skip(1)
                .filter(|m| m.pid.is_none())
                .map(|m| m.name.clone())
                .collect()
        };
        let session = self;
        let result = self.wait_ready(runner, rest, dedicated, progress).await;
        // Orders held while the machines came back go out now, or fail at once (NFR-RL-04).
        {
            let mut inner = session.inner.lock().unwrap();
            for m in inner.machines.iter_mut() {
                m.held = false;
            }
        }
        session.changed.notify_waiters();
        result
    }

    async fn wait_ready(
        &self,
        runner: &Runner,
        rest: Vec<String>,
        dedicated: bool,
        progress: &(dyn Fn(String) + Sync),
    ) -> Result<(), (String, String)> {
        let session = self;
        let spec = &self.spec;
        let plan = machine_plan(spec);
        let deadline = Instant::now() + Duration::from_secs(spec.setup.start_timeout);
        let mut rest_launched = rest.is_empty();
        let mut asked = Instant::now() - Duration::from_secs(1);
        loop {
            struct Seen {
                name: String,
                state: &'static str,
                kind: &'static str,
                agent_ready: bool,
                exit: Option<u32>,
            }
            let seen: Vec<Seen> = {
                let inner = session.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .map(|m| Seen {
                        name: m.name.clone(),
                        state: m.state,
                        kind: m.kind,
                        agent_ready: m.agent_ready,
                        exit: m.process.as_ref().and_then(win::exited),
                    })
                    .collect()
            };
            if let Some(gone) = seen.iter().find(|m| m.exit.is_some()) {
                let code = gone.exit.unwrap();
                let cause = if code == 53 {
                    format!(
                        "Steam is not running or not logged in ({} exited with code 53)",
                        gone.name
                    )
                } else {
                    format!(
                        "{} exited with code {code} before it was ready; it was {}",
                        gone.name, gone.state
                    )
                };
                return Err((gone.name.clone(), cause));
            }
            if seen.iter().all(|m| m.state == "ready") && rest_launched {
                return Ok(());
            }
            if !rest_launched && seen[0].state == "ready" {
                if let Err(cause) = session.launch(runner, &rest, progress) {
                    return Err((rest[0].clone(), format!("The game could not be started: {cause}")));
                }
                rest_launched = true;
                continue;
            }
            // A dedicated client whose agent is ready is ready once the server sees it at client
            // state 10 (P-02 step 7, INT-AR-09).
            if dedicated && asked.elapsed() >= Duration::from_millis(500) {
                asked = Instant::now();
                for m in seen
                    .iter()
                    .filter(|m| m.kind == "client" && m.agent_ready && m.state != "ready")
                {
                    let answer = session
                        .exec("server", "_this call skd_fnc_clientState", json!([m.name]), 5.0)
                        .await;
                    let state = answer.as_ref().ok().and_then(|a| a["value"].as_i64()).unwrap_or(-1);
                    if state >= CLIENT_READY_STATE {
                        let mut inner = session.inner.lock().unwrap();
                        if let Some(x) = inner.machines.iter_mut().find(|x| x.name == m.name) {
                            x.state = "ready";
                            x.held = false;
                        }
                    }
                }
            }
            if Instant::now() >= deadline {
                let waiting: Vec<String> = seen
                    .iter()
                    .filter(|m| m.state != "ready")
                    .map(|m| {
                        if rest_launched || m.kind == "server" {
                            format!("{} was {}", m.name, m.state)
                        } else {
                            format!("{} was not launched, since the server was not ready", m.name)
                        }
                    })
                    .collect();
                let first_waiting = seen
                    .iter()
                    .find(|m| m.state != "ready")
                    .map_or(plan[0].0.clone(), |m| m.name.clone());
                let cause = format!(
                    "the session was not ready within the start ceiling of {} s: {}",
                    spec.setup.start_timeout,
                    waiting.join("; ")
                );
                return Err((first_waiting, cause));
            }
            tokio::select! {
                _ = session.changed.notified() => {}
                _ = tokio::time::sleep(POLL) => {}
            }
        }
    }

    /// Asks every machine for the version of each mod with a patch key, as it loaded it (RES-02).
    async fn versions(&self) {
        let plan = machine_plan(&self.spec);
        let patched: Vec<(String, String)> = self
            .spec
            .mods
            .iter()
            .filter_map(|m| m.patch.clone().map(|p| (m.name.clone(), p)))
            .collect();
        if patched.is_empty() {
            return;
        }
        let patches: Vec<&str> = patched.iter().map(|(_, p)| p.as_str()).collect();
        let source = r#"_this apply { private _c = configFile >> "CfgPatches" >> _x; if (isClass _c) then { getText (_c >> "versionStr") } else { nil } }"#;
        for (name, _) in &plan {
            let answer = self.exec(name, source, json!(patches), ORDER_CEILING_S).await;
            let versions = answer.ok().map(|a| a["value"].clone()).unwrap_or(Value::Null);
            let mut inner = self.inner.lock().unwrap();
            let m = inner.machines.iter_mut().find(|m| &m.name == name).unwrap();
            m.mods = patched
                .iter()
                .enumerate()
                .map(|(i, (mod_name, patch))| ModVersion {
                    mod_name: mod_name.clone(),
                    patch: patch.clone(),
                    version: versions[i].as_str().filter(|v| !v.is_empty()).map(String::from),
                })
                .collect();
        }
    }

    /// Holds new orders to `names`, then waits up to 10 s for the orders already sent to them to
    /// be answered, so a restart cuts no order off (NFR-RL-04).
    async fn hold(&self, names: &[String]) {
        {
            let mut inner = self.inner.lock().unwrap();
            for m in inner.machines.iter_mut().filter(|m| names.contains(&m.name)) {
                m.held = true;
            }
        }
        for _ in 0..(STOP_CEILING.as_millis() / POLL.as_millis()) {
            let busy = {
                let inner = self.inner.lock().unwrap();
                inner.pending.values().any(|(target, _)| names.contains(target))
            };
            if !busy {
                break;
            }
            tokio::time::sleep(POLL).await;
        }
    }

    /// Ends the processes of `names` and puts a new start of each role in its place, with start
    /// number plus one and a profile folder of its own, not yet launched (`P-08` steps 4 and 5).
    /// The old starts end `stopped`.
    async fn replace(&self, runner: &Runner, names: &[String]) {
        {
            let inner = self.inner.lock().unwrap();
            for m in inner.machines.iter().filter(|m| names.contains(&m.name)) {
                if let Some(p) = &m.process {
                    p.terminate();
                }
            }
        }
        for _ in 0..(STOP_CEILING.as_millis() / POLL.as_millis()) {
            let left = {
                let inner = self.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .filter(|m| names.contains(&m.name))
                    .filter_map(|m| m.process.as_ref())
                    .any(|p| win::exited(p).is_none())
            };
            if !left {
                break;
            }
            tokio::time::sleep(POLL).await;
        }
        let now = SystemTime::now();
        let mut inner = self.inner.lock().unwrap();
        for i in 0..inner.machines.len() {
            if !names.contains(&inner.machines[i].name) {
                continue;
            }
            let (name, kind, start) = {
                let m = &inner.machines[i];
                (m.name.clone(), m.kind, m.start + 1)
            };
            let profile = self.folder.join("machines").join(format!("{name}-{start}"));
            let mut new = Machine::new(&name, kind, start, profile);
            new.held = true;
            let mut old = std::mem::replace(&mut inner.machines[i], new);
            if let Some(p) = old.process.take() {
                runner.log(&format!(
                    "{} {name}: process {} stopped for start {start}",
                    self.id, p.pid
                ));
            }
            old.tx = None;
            old.state = "stopped";
            inner.retired.push((old, now));
        }
        inner.pending.retain(|_, (target, _)| !names.contains(target));
    }

    /// A mission restart, level B of `P-08`: in singleplayer the lead ends the mission and the
    /// game loads the next entry of the `-autotest` list; on a dedicated session the clients and
    /// headless clients are replaced by fresh processes around the server's `#restart`. Every
    /// agent captures a new baseline at the mission start (`RST-02`). On failure, the machine that
    /// did not come back and why.
    pub async fn mission_restart(
        &self,
        runner: &Runner,
        progress: &(dyn Fn(String) + Sync),
    ) -> Result<(), (String, String)> {
        let lead = self.lead().to_string();
        let names = self.machine_names();
        let others: Vec<String> = names.iter().filter(|n| **n != lead).cloned().collect();
        if !others.is_empty() {
            self.hold(&others).await;
            self.replace(runner, &others).await;
            // Users the server still lists after their processes ended are kicked, so the
            // mission restarts with nobody on it (P-08 step 4).
            let kick = r##"params ["_pw"];
                private _t = diag_tickTime;
                waitUntil {
                    { _pw serverCommand format ["#kick %1", _x] } forEach allUsers;
                    sleep 0.5;
                    count allUsers == 0 || { diag_tickTime - _t > 20 }
                };
                count allUsers"##;
            let left = self.exec(&lead, kick, json!([self.command_password()]), 22.0).await;
            match left.as_ref().ok().and_then(|a| a["value"].as_u64()) {
                Some(0) => {}
                Some(n) => {
                    return Err((
                        lead,
                        format!("the server still listed {n} users 20 s after their processes ended"),
                    ));
                }
                None => {
                    return Err((
                        lead,
                        format!("the server did not answer the kick of its users ({left:?})"),
                    ));
                }
            }
        }
        let before = self.mission_start(&lead);
        self.hold(std::slice::from_ref(&lead)).await;
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(m) = inner.machines.iter_mut().find(|m| m.name == lead) {
                m.state = "connected";
                m.agent_ready = false;
            }
        }
        let (end, args) = if others.is_empty() {
            (r#"endMission "END1"; true"#, json!([]))
        } else {
            (
                r##"params ["_pw"]; [_pw] spawn { params ["_pw"]; sleep 0.1; _pw serverCommand "#restart" }; true"##,
                json!([self.command_password()]),
            )
        };
        if !self.push(
            &lead,
            json!({"kind": "exec", "source": end, "args": args, "ceiling": 5.0}),
        ) {
            return Err((lead, "the lead has no connection to the runner".into()));
        }
        progress(format!("session {}: mission restart", self.id));
        let result = self.bring_up(runner, false, progress).await;
        if result.is_ok() && self.mission_start(&lead) <= before {
            return Err((lead, "the lead reported ready without a new mission start".into()));
        }
        if result.is_ok() {
            self.versions().await;
            self.take_snapshot();
        }
        result
    }

    /// A game restart, level C of `P-08`: every process of the session is replaced by a new start
    /// of its role, in the session's folder, as a start brings them up (`P-02` steps 5 to 8).
    /// Between the end of the old processes and the start of the new ones, while no game process
    /// holds a mod file, each `rebuild` runs in `folder`, in the order given (`RLD-06`). A rebuild
    /// that fails still brings every machine back, on the files as they are, and fails the restart.
    pub async fn game_restart(
        &self,
        runner: &Runner,
        rebuilds: &[(String, String)],
        folder: &Path,
        progress: &(dyn Fn(String) + Sync),
    ) -> Result<(), (String, String)> {
        let lead = self.lead().to_string();
        let others: Vec<String> = self.machine_names().into_iter().filter(|n| *n != lead).collect();
        self.hold(&others).await;
        self.replace(runner, &others).await;
        self.hold(std::slice::from_ref(&lead)).await;
        self.replace(runner, std::slice::from_ref(&lead)).await;
        let mut failed = None;
        for (name, command) in rebuilds {
            progress(format!("session {}: rebuild of {name}", self.id));
            let log = self.folder.join(format!("rebuild-{name}.log"));
            if let Err(why) = rebuild(command, folder, &log).await {
                runner.log(&format!("{} rebuild of {name} failed: {why}", self.id));
                failed = Some(format!(
                    "the rebuild of mod {name} failed: {why} (its output is in {}); every machine came back on the files as they are",
                    log.display()
                ));
                break;
            }
            runner.log(&format!("{} rebuild of {name} done", self.id));
        }
        progress(format!("session {}: game restart", self.id));
        let result = self.bring_up(runner, true, progress).await;
        if result.is_ok() {
            self.versions().await;
            self.take_snapshot();
        }
        match (result, failed) {
            (Err(e), _) => Err(e),
            (Ok(()), Some(why)) => Err((lead, why)),
            (Ok(()), None) => Ok(()),
        }
    }

    /// The names of the setup's mods, in the setup's order.
    pub fn mod_names(&self) -> Vec<String> {
        self.spec.mods.iter().map(|m| m.name.clone()).collect()
    }

    /// The watched `source` folders: those of the setup's mods that have one (`CFG-01`).
    pub fn sources(&self) -> Vec<(String, PathBuf, String)> {
        self.spec
            .mods
            .iter()
            .filter_map(|m| Some((m.name.clone(), m.source.clone()?, m.prefix.clone().unwrap_or_default())))
            .collect()
    }

    /// Takes the snapshot of every file under the watched `source` folders (`RLD-01`).
    pub fn take_snapshot(&self) {
        let mut snap = std::collections::BTreeMap::new();
        for (_, dir, _) in self.sources() {
            walk(&dir, &mut snap);
        }
        *self.snapshot.lock().unwrap() = snap;
    }

    /// Takes the machines out of their hold; for a restart that failed, so their orders fail at
    /// once instead of waiting.
    pub fn release(&self) {
        let mut inner = self.inner.lock().unwrap();
        for m in inner.machines.iter_mut() {
            m.held = false;
        }
        drop(inner);
        self.changed.notify_waiters();
    }
}

/// The stamp of a file: its size, modification time and a hash of its content.
pub fn stamp(path: &Path) -> Option<Stamp> {
    use std::hash::{Hash, Hasher};
    let meta = std::fs::metadata(path).ok()?;
    let bytes = std::fs::read(path).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    Some(Stamp {
        size: meta.len(),
        modified: meta.modified().ok(),
        hash: h.finish(),
    })
}

fn walk(dir: &Path, out: &mut std::collections::BTreeMap<PathBuf, Stamp>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if let Some(s) = stamp(&path) {
            out.insert(path, s);
        }
    }
}

/// The preconditions of `SES-03`, and for a dedicated setup the framework's junction in the game
/// folder (`INT-AR-06`).
fn preconditions(runner: &Runner, spec: &SetupSpec) -> Result<(), CliError> {
    let mut missing = Vec::new();
    let dedicated = spec.setup.mode == Mode::Dedicated;
    let mut exes = vec![win::game_exe(&spec.game)];
    if dedicated {
        exes.push(win::server_exe(&spec.game));
    }
    for exe in exes {
        if !exe.is_file() {
            missing.push(format!(
                "the game executable {} (check `game` in shakedown.toml)",
                exe.display()
            ));
        }
    }
    if !win::steam_running() {
        missing.push("a running Steam client of this user (start Steam and log in)".to_string());
    }
    for m in &spec.mods {
        if !m.path.is_dir() {
            missing.push(format!("the folder {} of mod {}", m.path.display(), m.name));
        }
    }
    let mut addon_ok = true;
    for file in ["addons/shakedown_main.pbo", "shakedown_x64.dll"] {
        if !runner.addon.join(file).is_file() {
            addon_ok = false;
            missing.push(format!(
                "the framework's addon file {} (install Shakedown again)",
                runner.addon.join(file).display()
            ));
        }
    }
    if dedicated {
        let base = spec.base_port;
        let held: Vec<u16> = (base..base.saturating_add(5))
            .filter(|p| std::net::UdpSocket::bind(("0.0.0.0", *p)).is_err())
            .collect();
        if !held.is_empty() {
            missing.push(format!(
                "free ports {base} to {} for the server; {} held by another program (stop it, or set base_port in shakedown.toml)",
                base + 4,
                held.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")
            ));
        }
        if addon_ok
            && spec.game.is_dir()
            && let Err(e) = junction(&spec.game.join("@shakedown"), &runner.addon)
        {
            missing.push(e);
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(CliError::not_now(format!(
            "The session cannot start; missing: {}. Provide what is missing and start the session again.",
            missing.join("; ")
        ))
        .tag("precondition-failed"))
    }
}

/// Makes `link` a directory junction to `target`: created when missing, pointed at `target`
/// when it is a junction to another folder, and left alone when it already points there
/// (`INT-AR-06`). A real folder of that name is never touched.
fn junction(link: &Path, target: &Path) -> Result<(), String> {
    let plain = |p: PathBuf| PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\").to_string());
    let want = std::fs::canonicalize(target)
        .map(plain)
        .unwrap_or_else(|_| target.to_path_buf());
    if let Ok(meta) = std::fs::symlink_metadata(link) {
        let is_link = std::fs::read_link(link).is_ok();
        if !is_link {
            return Err(format!(
                "the framework's link {} in the game folder: a folder of that name that is not a link is there (move it away)",
                link.display()
            ));
        }
        let now = std::fs::canonicalize(link).map(plain).ok();
        if now.as_ref().is_some_and(|n| n.eq(&want)) && meta.is_dir() {
            return Ok(());
        }
        std::fs::remove_dir(link)
            .map_err(|e| format!("the framework's link {} could not be replaced: {e}", link.display()))?;
    }
    let out = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(link)
        .arg(&want)
        .output()
        .map_err(|e| format!("the framework's link {}: {e}", link.display()))?;
    if !out.status.success() {
        return Err(format!(
            "the framework's link {} could not be made: {}",
            link.display(),
            String::from_utf8_lossy(&out.stdout).trim()
        ));
    }
    Ok(())
}

/// The error a client reports for a session that ended `failed`.
pub fn start_failed(session: &Value) -> CliError {
    let reason = &session["reason"];
    let lines: Vec<String> = reason["rpt_tail"]
        .as_array()
        .map(|a| a.iter().filter_map(|l| l.as_str().map(String::from)).collect())
        .unwrap_or_default();
    CliError::not_now(format!(
        "Session {} failed: {}. Every process it started was stopped.{}",
        session["id"].as_str().unwrap_or("?"),
        reason["message"].as_str().unwrap_or("the start failed"),
        if lines.is_empty() {
            String::new()
        } else {
            format!(
                " The last lines of {}'s RPT:\n{}",
                reason["machine"].as_str().unwrap_or("?"),
                lines.join("\n")
            )
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

/// The ceiling of one mod's `rebuild` (`RLD-06`).
const REBUILD_CEILING: Duration = Duration::from_secs(120);

/// Runs a mod's `rebuild` through the command interpreter in `folder`, with no window, its output
/// in `log`; an error says why it failed: its exit code, or the ceiling.
async fn rebuild(command: &str, folder: &Path, log: &Path) -> Result<(), String> {
    let out = std::fs::File::create(log).map_err(|e| format!("cannot write {}: {e}", log.display()))?;
    let err = out.try_clone().map_err(|e| e.to_string())?;
    let mut cmd = shell(command);
    cmd.current_dir(folder)
        .stdin(std::process::Stdio::null())
        .stdout(out)
        .stderr(err);
    let mut child = cmd.spawn().map_err(|e| format!("it did not start ({e})"))?;
    for _ in 0..(REBUILD_CEILING.as_millis() / POLL.as_millis()) {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => return Err(format!("it exited with {status}")),
            Ok(None) => tokio::time::sleep(POLL).await,
            Err(e) => return Err(e.to_string()),
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Err(format!("it did not end within {} s", REBUILD_CEILING.as_secs()))
}

#[cfg(windows)]
fn shell(command: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/C").raw_arg(command).creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
fn shell(command: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rld_06_a_rebuild_runs_in_the_project_folder_and_a_failing_one_names_its_exit() {
        let dir = std::env::temp_dir().join(format!("skd-rebuild-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let log = dir.join("rebuild.log");
        rebuild("echo built> out.txt", &dir, &log).await.unwrap();
        assert!(std::fs::read_to_string(dir.join("out.txt")).unwrap().contains("built"));
        let why = rebuild("exit 3", &dir, &log).await.unwrap_err();
        assert!(why.contains('3'), "{why}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
