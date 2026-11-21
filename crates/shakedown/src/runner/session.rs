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

pub mod join;

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
    /// How and when the machine was lost (`P-07`, `P-09`), once it is.
    pub lost: Option<LostRecord>,
    /// The heartbeats of the last few seconds, the newest last (`HLT-02`).
    beats: Vec<Beat>,
    /// When the last heartbeat came, or the machine became ready.
    last_beat: Option<Instant>,
    /// The measure the watch flagged, while it is flagged (`HLT-02`, `HLT-01`).
    flag: Option<FlagInterval>,
    /// The ping the runner sent with the last heartbeat and has no answer to yet, with its id and
    /// when it was sent; one at a time (`HLT-01`).
    ping: Option<(String, Instant)>,
    /// How long the last answered ping took, in seconds (`HLT-01`).
    answered_s: Option<f64>,
    /// When its agent said its mission display closed, until it is ready again: a client between
    /// two missions sends no heartbeat, and the watch gives it `LEAVE_CEILING` to start its agent
    /// again, so a client sent to another server is ended for that (`SEC-05`), not as `no-answer`.
    left_mission: Option<Instant>,
}

/// How a machine was lost: its process `exited`, it gave `no-answer` (`P-07`, `P-09`), or the
/// runner ended it because it joined a `foreign-server` (`SEC-05`).
#[derive(Debug, Clone)]
pub struct LostRecord {
    pub reason: &'static str,
    /// The wall-clock time it was lost (`RES-05`).
    pub at: String,
    pub instant: Instant,
    pub exit_code: Option<u32>,
    /// What the check it ended and the session's reason say.
    pub message: String,
    /// The crash dumps in its profile folder (`P-09` step 2).
    pub dumps: Vec<PathBuf>,
    /// While the runner still looks for dumps being written.
    pub searching: bool,
}

/// One heartbeat's clocks (`OBS-05`), and the machine's answer time when it came (`HLT-01`).
#[derive(Debug, Clone)]
struct Beat {
    utc: String,
    tick: f64,
    time: f64,
    frame: f64,
    /// The answer time of `answer_time` when the heartbeat came, in seconds.
    answer: f64,
}

/// An interval in which the health watch flagged a machine (`HLT-02`, `HLT-01`): which measure
/// crossed its threshold (`frame`, `frame-rate`, `mission-time` or `answer-time`), the
/// wall-clock time it crossed, the time the flag was raised and the time it cleared.
#[derive(Debug, Clone)]
pub struct FlagInterval {
    pub machine: String,
    pub clock: &'static str,
    pub from: String,
    pub since: String,
    pub cleared: Option<String>,
    /// How much mission time and real time passed over the window that raised it.
    pub advanced: (f64, f64),
    /// The frame rate over that window, and the smallest answer time in it, in seconds.
    pub measured: (f64, f64),
}

/// The real time over which the clock watch judges a machine's clocks (`HLT-02`).
pub const WATCH_WINDOW_S: f64 = 5.0;
/// Below this share of real time, mission time is said to have stopped (`HLT-02`).
pub const WATCH_RATIO: f64 = 0.5;
/// Below this many frames a second over the window, the frame rate has collapsed (`HLT-01`,
/// `SPK-22`).
pub const FRAME_RATE_MIN: f64 = 10.0;
/// Answers slower than this, in seconds, over the whole window lag (`HLT-01`, `SPK-22`).
pub const ANSWER_MAX_S: f64 = 2.0;
/// A machine whose heartbeat has not come for this long has stopped answering (`HLT-02`, `P-07`).
pub const HEARTBEAT_GAP: Duration = Duration::from_secs(10);
/// How long a client whose mission display closed may go without a heartbeat before it is
/// `no-answer`: the time a client takes to join a server and start its mission (`SEC-05`).
pub const LEAVE_CEILING: Duration = Duration::from_secs(90);
/// How often the runner looks at every machine's process and heartbeat (`P-07`, `P-09`).
const WATCH_EVERY: Duration = Duration::from_millis(250);
/// A watch round this late shows that the runner itself did not run (`SES-06`, `US-060`).
const RUNNER_PAUSE: Duration = Duration::from_secs(2);
/// How long the runner waits for a crash dump still being written (`P-09` step 2).
const DUMP_WAIT: Duration = Duration::from_secs(5);

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
            lost: None,
            beats: Vec::new(),
            last_beat: None,
            flag: None,
            ping: None,
            answered_s: None,
            left_mission: None,
        }
    }
}

/// What an order asked, in a few words, for the message of a machine lost while it waited
/// (`P-07` step 2).
fn describe(order: &Value) -> String {
    let short = |s: &str| {
        let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
        if one.chars().count() > 80 {
            format!("{}...", one.chars().take(80).collect::<String>())
        } else {
            one
        }
    };
    let source = order["source"].as_str().unwrap_or_default();
    match order["kind"].as_str().unwrap_or_default() {
        "check" => format!("to run check {}", order["check_id"].as_str().unwrap_or("?")),
        "poll" => format!("to poll `{}`", short(source)),
        "exec" => format!("to run `{}`", short(source)),
        other => format!("an order of kind {other}"),
    }
}

/// What the health watch flagged over a machine's heartbeats.
type Crossed = (&'static str, String, (f64, f64), (f64, f64));

/// The health watch over a machine's heartbeats (`HLT-02`, `HLT-01`): the measure that crossed its
/// threshold over the last `WATCH_WINDOW_S` of real time, with the time it crossed, how much
/// mission time and real time passed over the window, and the window's frame rate and smallest
/// answer time; or `None` while every measure holds. A history shorter than the window is judged
/// once it is long enough. A frame number that stands still is named before a frame rate under
/// `FRAME_RATE_MIN`, that before mission time under `WATCH_RATIO` of real time, and that before
/// answers slower than `ANSWER_MAX_S` over the whole window.
fn stopped_clock(beats: &[Beat]) -> Option<Crossed> {
    let newest = beats.last()?;
    let i = beats.iter().rposition(|b| newest.tick - b.tick >= WATCH_WINDOW_S)?;
    let start = &beats[i];
    let real = newest.tick - start.tick;
    let mission = newest.time - start.time;
    let rate = (newest.frame - start.frame) / real;
    let answer = beats[i..].iter().map(|b| b.answer).fold(f64::INFINITY, f64::min);
    let clock = if newest.frame <= start.frame {
        "frame"
    } else if rate < FRAME_RATE_MIN {
        "frame-rate"
    } else if mission < WATCH_RATIO * real {
        "mission-time"
    } else if answer > ANSWER_MAX_S {
        "answer-time"
    } else {
        return None;
    };
    // It crossed by the first beat that shows it: the end of the first bad step of the run of bad
    // steps that ends at the newest beat. A cause in the check before that beat came first
    // (DM-06), so a failure is never hidden by a crossing the beats only show later.
    let mut from = newest.utc.clone();
    for pair in beats.windows(2).rev() {
        if healthy(&pair[0], &pair[1]) {
            break;
        }
        from = pair[1].utc.clone();
    }
    Some((clock, from, (mission, real), (rate, answer)))
}

/// Whether every measure held between two heartbeats: both clocks moved, the frame rate stayed at
/// or above its floor and the later answer time at or under its ceiling.
fn healthy(a: &Beat, b: &Beat) -> bool {
    let real = b.tick - a.tick;
    b.frame > a.frame
        && b.time - a.time >= WATCH_RATIO * real
        && b.frame - a.frame >= FRAME_RATE_MIN * real
        && b.answer <= ANSWER_MAX_S
}

/// A machine's frame rate over the last `WATCH_WINDOW_S` of its heartbeats, or over all of them
/// while they span less, from 1 s on (`HLT-01`, `INS-04`).
fn frame_rate(beats: &[Beat]) -> Option<f64> {
    let newest = beats.last()?;
    let start = beats
        .iter()
        .rev()
        .find(|b| newest.tick - b.tick >= WATCH_WINDOW_S)
        .or_else(|| beats.first())?;
    let real = newest.tick - start.tick;
    (real >= 1.0).then(|| (newest.frame - start.frame) / real)
}

/// A machine's answer time now (`HLT-01`): how long its last answered ping took, or how long the
/// ping it has not answered has waited, whichever is longer; `None` before its first ping.
fn answer_time(m: &Machine) -> Option<f64> {
    let waiting = m.ping.as_ref().map(|(_, sent)| sent.elapsed().as_secs_f64());
    match (m.answered_s, waiting) {
        (None, None) => None,
        (a, w) => Some(a.unwrap_or(0.0).max(w.unwrap_or(0.0))),
    }
}

/// An event message of an agent as the result file records it (`EVT-01`); the clocks travel as
/// text with fixed decimals.
fn recorded_event(m: &Value) -> Option<shakedown_protocol::result::RecordedEvent> {
    let n = |v: &Value| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()));
    let at = &m["at"];
    Some(shakedown_protocol::result::RecordedEvent {
        machine: m["machine"].as_str()?.to_string(),
        name: m["name"].as_str()?.to_string(),
        kind: m["type"].as_str().unwrap_or("cba").to_string(),
        args: m["args"].clone(),
        at: shakedown_protocol::result::Clocks {
            utc: at["utc"].as_str()?.to_string(),
            tick: n(&at["tick"])?,
            time: n(&at["time"])?,
            frame: n(&at["frame"])?.max(0.0) as u64,
        },
    })
}

/// Whether every measure held over the whole of the last `WATCH_WINDOW_S`, which clears a flag.
fn moving_again(beats: &[Beat]) -> bool {
    let Some(newest) = beats.last() else { return false };
    let Some(i) = beats.iter().rposition(|b| newest.tick - b.tick >= WATCH_WINDOW_S) else {
        return false;
    };
    beats[i..].windows(2).all(|p| healthy(&p[0], &p[1]))
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
    /// Each order waiting for its answer: the machine it went to, where the answer goes, and what
    /// it asked, which a machine lost while it waits is named with (`P-07`).
    pending: HashMap<String, (String, oneshot::Sender<Value>, String)>,
    /// Every interval in which the clock watch flagged a machine (`HLT-02`), the newest last.
    flags: Vec<FlagInterval>,
    /// A poll a machine runs for another (`P-03`): its order id to the machine that asked for it
    /// and that machine's reference.
    streams: HashMap<String, (String, String)>,
    /// The reverse: a reference of the asking machine to the polling machine and the order id.
    refs: HashMap<String, (String, String)>,
    /// The script errors the machines' handlers sent (`ERR-01`), until a check execution takes
    /// those stamped with its key.
    errors: Vec<Value>,
    /// The events a check's recording sent (`EVT-01`), until a check execution takes those
    /// stamped with its key.
    events: Vec<Value>,
    /// Every script error the handlers sent in this session, inside a check or not, the newest
    /// last, for `errors` and `inspect` (`ERR-05`, `INS-01`).
    history: Vec<Value>,
    /// Each `PlayerDisconnected` the server's agent reported: the user's profile name and when
    /// it came (`MCH-01`).
    disconnects: Vec<(String, Instant)>,
}

/// What every machine runs to read the release files: each path with `loadFile`, the bytes read
/// summed (`RLD-07`).
const READ_RELEASE: &str = r#"params ["_paths"]; private _n = 0; { _n = _n + count loadFile _x } forEach _paths; _n"#;

/// What every machine runs after a mission restart to recompile the `CfgFunctions` functions whose
/// file is one of the given game paths, compared without case or a leading backslash; it returns
/// the names of the functions it recompiled (`RLD-02`).
const RECOMPILE: &str = r#"params ["_paths"];
private _want = _paths apply { private _p = toLower _x; if (_p select [0, 1] == "\") then { _p = _p select [1] }; _p };
private _found = [];
{
    private _tagCfg = _x;
    private _tag = getText (_tagCfg >> "tag");
    if (_tag == "") then { _tag = configName _tagCfg };
    {
        private _cat = _x;
        private _dir = getText (_cat >> "file");
        {
            private _file = getText (_x >> "file");
            if (_file == "") then {
                private _ext = getText (_x >> "ext");
                if (_ext == "") then { _ext = ".sqf" };
                _file = format ["%1\fn_%2%3", _dir, configName _x, _ext];
            };
            _file = toLower _file;
            if (_file select [0, 1] == "\") then { _file = _file select [1] };
            if (_file in _want) then { _found pushBack format ["%1_fnc_%2", _tag, configName _x] };
        } forEach ("true" configClasses _cat);
    } forEach ("true" configClasses _tagCfg);
} forEach ("true" configClasses (configFile >> "CfgFunctions"));
{ [_x] call BIS_fnc_recompile } forEach _found;
_found"#;

/// The most script errors a session keeps for `errors`.
const HISTORY_LIMIT: usize = 1000;
/// The most recorded events a session holds until check executions take them (`EVT-01`).
const EVENTS_LIMIT: usize = 10_000;

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
    /// The library folder as the runner read it last, and what keeps it from loading (`API-06`).
    pub(crate) library: Mutex<crate::library::Library>,
    /// The game paths of the files reloads of level `mission-restart` changed since the last game
    /// start, whose `CfgFunctions` functions every machine recompiles after each mission restart,
    /// and the functions that recompile found (`RLD-02`, `RLD-05`).
    pub(crate) recompile: Mutex<(Vec<String>, std::collections::BTreeSet<String>)>,
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

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
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
        "health": {
            "flag": m.flag.as_ref().map(|f| f.clock),
            "since": m.flag.as_ref().map(|f| f.since.clone()),
            "frame_rate": frame_rate(&m.beats).map(round1),
            "answer_s": answer_time(m).map(round3),
            "heartbeat": m.beats.last().map(|b| json!({"utc": b.utc, "tick": b.tick, "time": b.time, "frame": b.frame as u64})),
        },
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

    /// The exit code of a machine's current process once it has ended, `None` while it runs.
    pub fn exit_code(&self, name: &str) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .find(|m| m.name == name)
            .and_then(|m| m.process.as_ref())
            .and_then(win::exited)
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
        crate::win::udp_ports_held(&(base..base.saturating_add(5)).collect::<Vec<_>>())
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

    /// Takes the events a check's recording sent that are stamped with the key of a check
    /// execution, in the order of their wall-clock time (`EVT-01`).
    pub fn take_events(&self, key: &str) -> Vec<shakedown_protocol::result::RecordedEvent> {
        let mut inner = self.inner.lock().unwrap();
        let (taken, kept): (Vec<Value>, Vec<Value>) = std::mem::take(&mut inner.events)
            .into_iter()
            .partition(|e| e["key"] == key);
        inner.events = kept;
        drop(inner);
        let mut out: Vec<_> = taken.iter().filter_map(recorded_event).collect();
        out.sort_by(|a, b| a.at.utc.cmp(&b.at.utc));
        out
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
                lost: m.lost.as_ref().map(|l| shakedown_protocol::result::Lost {
                    reason: l.reason.to_string(),
                    at: l.at.clone(),
                    exit_code: l.exit_code,
                    dumps: l.dumps.iter().map(|d| d.display().to_string()).collect(),
                }),
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
        runner: &Arc<Runner>,
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
                Ok(message) => self.message(runner, &machine, start, message),
                Err(e) => runner.log(&format!("{} {machine}: unreadable message: {e}", self.id)),
            }
        }
        writer.abort();
        // A process that ended is lost through the watch of its process handle (P-09), and one
        // that stops answering through its heartbeat (P-07); the connection alone decides nothing.
        let mut inner = self.inner.lock().unwrap();
        let current = inner.machines.iter().any(|m| m.name == machine && m.start == start);
        if let Some(m) = inner
            .machines
            .iter_mut()
            .find(|m| m.name == machine && m.start == start)
        {
            m.tx = None;
        }
        // Every order still waiting on this machine is answered by the closed channel.
        if current {
            inner.pending.retain(|_, (target, _, _)| *target != machine);
        }
        drop(inner);
        self.changed.notify_waiters();
        runner.log(&format!("{} {machine}: extension disconnected", self.id));
    }

    fn message(self: &Arc<Self>, runner: &Arc<Runner>, machine: &str, start: u32, message: Value) {
        let kind = message["kind"].as_str().unwrap_or_default().to_string();
        let mut inner = self.inner.lock().unwrap();
        if !inner.machines.iter().any(|m| m.name == machine && m.start == start) {
            return;
        }
        if let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) {
            m.last_answer_at = Some(shakedown_protocol::time::now());
            // The answer to the runner's ping: the machine's answer time (HLT-01).
            if kind == "pong" && m.ping.as_ref().is_some_and(|(id, _)| message["ping"] == id.as_str()) {
                m.answered_s = m.ping.take().map(|(_, sent)| sent.elapsed().as_secs_f64());
            }
            if kind == "heartbeat" {
                m.last_beat = Some(Instant::now());
                let c = &message["clocks"];
                let n = |v: &Value| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()));
                if let (Some(utc), Some(tick), Some(time), Some(frame)) =
                    (c["utc"].as_str(), n(&c["tick"]), n(&c["time"]), n(&c["frame"]))
                {
                    m.beats.push(Beat {
                        utc: utc.to_string(),
                        tick,
                        time,
                        frame,
                        answer: answer_time(m).unwrap_or(0.0),
                    });
                    // The window that is judged, and the steps before it to find when a clock
                    // stopped.
                    let keep = tick - 3.0 * WATCH_WINDOW_S;
                    m.beats.retain(|b| b.tick >= keep);
                }
            }
            if kind == "mission-left" {
                m.left_mission = Some(Instant::now());
            }
            if kind == "ready" {
                m.left_mission = None;
                m.beats.clear();
                m.flag = None;
                m.ping = None;
                m.answered_s = None;
                m.last_beat = Some(Instant::now());
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
            "pong" => return,
            "heartbeat" => {
                // Each heartbeat of a watched machine sends it a ping, one at a time, which its
                // agent answers through the path of every order: its answer time (HLT-01).
                let id = format!("ping{}", self.orders.fetch_add(1, Ordering::SeqCst) + 1);
                if let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine)
                    && m.ping.is_none()
                    && m.state == "ready"
                    && !m.held
                    && let Some(tx) = &m.tx
                    && tx.send(json!({"id": id, "kind": "ping"}).to_string()).is_ok()
                {
                    m.ping = Some((id, Instant::now()));
                }
                let changed = watch_clocks(&mut inner, machine);
                drop(inner);
                if changed {
                    self.changed.notify_waiters();
                }
                return;
            }
            // A script error the machine's handler caught (ERR-01).
            "error" => {
                inner.history.push(message.clone());
                if inner.history.len() > HISTORY_LIMIT {
                    inner.history.remove(0);
                }
                inner.errors.push(message);
            }
            // An event a check records (EVT-01); one sent outside a check execution is dropped.
            "event" => {
                if message["key"].as_str().is_some_and(|k| !k.is_empty()) && inner.events.len() < EVENTS_LIMIT {
                    inner.events.push(message);
                }
            }
            // The machine's agent started in a mission whose server is not the session's own
            // (SEC-05): its process is ended and it is lost for that.
            "foreign-server" => {
                let server = message["server"].as_str().unwrap_or_default().to_string();
                if let Some(p) = inner
                    .machines
                    .iter()
                    .find(|m| m.name == machine)
                    .and_then(|m| m.process.as_ref())
                {
                    p.terminate();
                }
                drop(inner);
                let message = format!(
                    "{machine} joined the server \"{server}\", which is not the session's own (\"shakedown {}\"), so the runner ended its process: a client may join only the session's own server",
                    self.id
                );
                if self.lose(machine, "foreign-server", None, message.clone()) {
                    runner.log(&format!("{} {machine}: lost: {message}", self.id));
                }
                return;
            }
            // The server saw a user leave (MCH-01).
            "disconnected" => {
                Session::disconnected(&mut inner, message["name"].as_str().unwrap_or_default());
            }
            // The lead asks for a join or a leave in a check (MCH-01).
            "join" | "leave" => {
                drop(inner);
                let me = Arc::clone(self);
                let runner = Arc::clone(runner);
                let from = machine.to_string();
                tokio::spawn(async move { me.machine_request(runner, from, message).await });
                return;
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
                    && let Some((_, reply, _)) = inner.pending.remove(order)
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
                if matches!(e, OrderError::Ceiling) {
                    let asked = describe(&message["order"]);
                    self.lose(&target, "no-answer", None, format!("{why}; it was asked {asked}"));
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
        if self.is_lost(machine) {
            return Err(OrderError::NotConnected);
        }
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
            inner
                .pending
                .insert(id.clone(), (machine.to_string(), reply, describe(&order)));
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
        // The reason of a start that failed, or of a session that broke, stays in the session
        // folder once the runner forgets it (SES-10).
        let mut reason = self.inner.lock().unwrap().reason.clone().unwrap_or(Value::Null);
        runner.redactor().redact_value(&mut reason);
        super::record::write_end(&self.folder, &self.id, end.as_str(), reason);
        super::record::remove_session(&self.id);
        // Every RPT of the session's machines, redacted once nothing writes it any more (LOG-02).
        let redactor = runner.redactor();
        let profiles: Vec<PathBuf> = {
            let inner = self.inner.lock().unwrap();
            inner
                .machines
                .iter()
                .chain(inner.retired.iter().map(|(m, _)| m))
                .map(|m| m.profile.clone())
                .collect()
        };
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
    // A late client needs a slot of its own in the mission and on the server (MCH-01).
    let players = spec.setup.clients + spec.setup.late;
    let sqm = mission::mission_sqm(players, spec.setup.headless, spec.setup.spawn);
    let ext = mission::description_ext(players + spec.setup.headless);
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
            flags: Vec::new(),
            streams: HashMap::new(),
            refs: HashMap::new(),
            errors: Vec::new(),
            events: Vec::new(),
            history: Vec::new(),
            disconnects: Vec::new(),
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
        library: Mutex::new(crate::library::read(&spec.lib, &spec.project)),
        recompile: Mutex::new(Default::default()),
    });
    if dedicated {
        std::fs::write(
            folder.join("server.cfg"),
            mission::server_cfg(
                &id,
                &spec.setup.world,
                &session.password(),
                &session.command_password(),
                spec.setup.clients + spec.setup.late + spec.setup.headless,
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
    if let Some((machine, cause)) = session.wrong_versions() {
        session.fail(runner, &machine, cause);
        session.stop(runner).await;
        return Ok(session);
    }
    session.load_library(&session.machine_names()).await;
    session.take_snapshot();
    session.set_state(SessionState::Ready);
    tokio::spawn(Arc::clone(&session).watch(Arc::clone(runner)));
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
                // The machine named is the one that got least far: a launched machine whose
                // extension never connected before one that connected (SES-04).
                let first_waiting = seen
                    .iter()
                    .filter(|m| m.state != "ready" && (rest_launched || m.kind == "server"))
                    .min_by_key(|m| match m.state {
                        "launched" => 0,
                        "connected" => 1,
                        _ => 2,
                    })
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
        self.versions_of(&self.machine_names()).await;
    }

    /// Asks the named machines for the version of each mod with a patch key (RES-02).
    async fn versions_of(&self, names: &[String]) {
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
        for name in names {
            let answer = self.exec(name, source, json!(patches), ORDER_CEILING_S).await;
            let versions = answer.ok().map(|a| a["value"].clone()).unwrap_or(Value::Null);
            let mut inner = self.inner.lock().unwrap();
            let Some(m) = inner.machines.iter_mut().find(|m| &m.name == name) else {
                continue;
            };
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

    /// The first machine that loaded a mod in another version than its `version` key names, and
    /// a cause naming every such mod and machine with the version found and the version wanted
    /// (`ENV-01`, `INT-STM-03`).
    fn wrong_versions(&self) -> Option<(String, String)> {
        let inner = self.inner.lock().unwrap();
        let reported: Vec<(String, Vec<ModVersion>)> = inner
            .machines
            .iter()
            .map(|m| (m.name.clone(), m.mods.clone()))
            .collect();
        drop(inner);
        version_gaps(&self.spec.mods, &reported)
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
                inner.pending.values().any(|(target, _, _)| names.contains(target))
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
        inner.pending.retain(|_, (target, _, _)| !names.contains(target));
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
            self.recompile_functions().await;
            self.versions().await;
            self.load_library(&self.machine_names()).await;
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
        let failed = self.rebuild_all(runner, rebuilds, folder, progress).await;
        progress(format!("session {}: game restart", self.id));
        // New processes compile every function from its file.
        *self.recompile.lock().unwrap() = Default::default();
        let result = self.bring_up(runner, true, progress).await;
        if result.is_ok() {
            self.versions().await;
            self.load_library(&self.machine_names()).await;
            self.take_snapshot();
        }
        match (result, failed) {
            (Err(e), _) => Err(e),
            (Ok(()), Some(why)) => Err((lead, why)),
            (Ok(()), None) => Ok(()),
        }
    }

    /// Has every machine recompile, with `BIS_fnc_recompile`, each `CfgFunctions` function whose
    /// file is one of the game paths a `mission-restart` reload changed since the last game start,
    /// since the Functions Library copies an addon's function compiled at the game start into each
    /// new mission (`RLD-02`, research 27). The functions found are kept for `RLD-05`.
    pub async fn recompile_functions(&self) {
        let paths = self.recompile.lock().unwrap().0.clone();
        if paths.is_empty() {
            return;
        }
        for name in self.machine_names() {
            let Ok(answer) = self.exec(&name, RECOMPILE, json!([paths.clone()]), 30.0).await else {
                continue;
            };
            let found: Vec<String> = answer["value"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            self.recompile.lock().unwrap().1.extend(found);
        }
    }

    /// Whether a check executing now runs after a recompile of a `CfgFunctions` function, so it
    /// records `reload: partial` (`RLD-05`).
    pub fn partial(&self) -> bool {
        !self.recompile.lock().unwrap().1.is_empty()
    }

    /// Every machine reads the release files at once, within the ceiling of `RLD-07`; on failure
    /// the machine that did not answer and why.
    pub async fn read_release(self: &Arc<Self>, runner: &Runner, paths: &[String]) -> Result<(), (String, String)> {
        let mut reads = tokio::task::JoinSet::new();
        for name in self.machine_names() {
            let (session, paths) = (Arc::clone(self), paths.to_vec());
            reads.spawn(async move {
                let answer = session.exec(&name, READ_RELEASE, json!([paths]), 60.0).await;
                (name, answer)
            });
        }
        let mut failed = None;
        while let Some(done) = reads.join_next().await {
            let Ok((name, answer)) = done else { continue };
            match answer.as_ref().ok().and_then(|a| a["value"].as_f64()) {
                // File patching found none of the release files; the hold check that follows
                // decides whether the changed files are free anyway.
                Some(n) if n <= 0.0 => runner.log(&format!("{} {name} read none of the release files", self.id)),
                Some(_) => {}
                None => {
                    failed = failed.or(Some((
                        name,
                        format!("the read of the release files did not complete ({answer:?})"),
                    )))
                }
            }
        }
        failed.map_or(Ok(()), Err)
    }

    /// Runs each `rebuild` in `folder`, in the order given, while no game process holds the files
    /// it writes (`RLD-06`); the first failure, in the words of the reload's reason.
    pub async fn rebuild_all(
        &self,
        runner: &Runner,
        rebuilds: &[(String, String)],
        folder: &Path,
        progress: &(dyn Fn(String) + Sync),
    ) -> Option<String> {
        for (name, command) in rebuilds {
            progress(format!("session {}: rebuild of {name}", self.id));
            let log = self.folder.join(format!("rebuild-{name}.log"));
            if let Err(why) = rebuild(command, folder, &log).await {
                runner.log(&format!("{} rebuild of {name} failed: {why}", self.id));
                return Some(format!(
                    "the rebuild of mod {name} failed: {why} (its output is in {}); every machine came back on the files as they are",
                    log.display()
                ));
            }
            runner.log(&format!("{} rebuild of {name} done", self.id));
        }
        None
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

    /// Reads the library folder again (`API-06`), as every reload does.
    pub fn read_library(&self, dir: &Path, project: &Path) {
        *self.library.lock().unwrap() = crate::library::read(dir, project);
    }

    /// What keeps the library from loading, if anything: every check then ends `fail` /
    /// `check-error` with it (`API-06`).
    pub fn library_problem(&self) -> Option<String> {
        self.library.lock().unwrap().problem.clone()
    }

    /// Has each of the named machines compile the library into `skd_lib_<file name>`
    /// (`API-06`), after a mission start or a reload. The first compile error a machine reports
    /// becomes the library's problem. A machine that does not answer is left to the watch, which
    /// sees it lost.
    pub async fn load_library(&self, names: &[String]) {
        let args = self.library.lock().unwrap().order_args();
        // One machine after another: each compiles in a few milliseconds.
        for name in names {
            let Ok(answer) = self.exec(name, "_this call skd_fnc_library", args.clone(), 30.0).await else {
                continue;
            };
            let first = answer["value"].as_array().and_then(|e| e.first()).cloned();
            if let Some(e) = first {
                let problem = crate::library::Library::compile_problem(
                    e[0].as_str().unwrap_or_default(),
                    &e[1],
                    e[2].as_str().unwrap_or_default(),
                );
                let mut library = self.library.lock().unwrap();
                if library.problem.is_none() {
                    library.problem = Some(problem);
                }
            }
        }
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

/// The health watch of `HLT-02` and `HLT-01` on one heartbeat of `machine`: raises its flag when a
/// measure crossed its threshold over the last window, and clears it once every measure held over
/// a whole window again. True when the flag changed. Only a `ready` machine that is not held is watched.
fn watch_clocks(inner: &mut Inner, machine: &str) -> bool {
    let Some(i) = inner.machines.iter().position(|m| m.name == machine) else {
        return false;
    };
    let m = &inner.machines[i];
    if m.state != "ready" || m.held || m.lost.is_some() {
        return false;
    }
    if m.flag.is_none() {
        let Some((clock, from, advanced, measured)) = stopped_clock(&m.beats) else {
            return false;
        };
        let flag = FlagInterval {
            machine: machine.to_string(),
            clock,
            from,
            since: shakedown_protocol::time::now(),
            cleared: None,
            advanced,
            measured,
        };
        inner.machines[i].flag = Some(flag.clone());
        inner.flags.push(flag);
        return true;
    }
    if !moving_again(&m.beats) {
        return false;
    }
    inner.machines[i].flag = None;
    let now = shakedown_protocol::time::now();
    if let Some(f) = inner
        .flags
        .iter_mut()
        .rev()
        .find(|f| f.machine == machine && f.cleared.is_none())
    {
        f.cleared = Some(now);
    }
    true
}

/// The crash dumps in a profile folder, each with its size.
fn dump_files(profile: &Path) -> Vec<(PathBuf, u64)> {
    let mut found: Vec<(PathBuf, u64)> = std::fs::read_dir(profile)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("mdmp") || e.eq_ignore_ascii_case("bidmp"))
        })
        .map(|p| {
            let size = std::fs::metadata(&p).map_or(0, |m| m.len());
            (p, size)
        })
        .collect();
    found.sort();
    found
}

impl Session {
    /// Whether a machine of the session is `lost`.
    pub fn is_lost(&self, machine: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.machines.iter().any(|m| m.name == machine && m.lost.is_some())
    }

    /// Marks a machine `lost` with `reason` (`exited` or `no-answer`), the wall-clock time and
    /// `message`, answers every order still waiting on it as lost, and marks the session `broken`
    /// with that message and the machine's last 20 RPT lines (`P-07` step 2, `P-09` step 2). The
    /// game processes keep running. False when the machine was lost already, or the session is
    /// starting or stopping, whose own steps handle a machine that goes away.
    pub(crate) fn lose(&self, machine: &str, reason: &'static str, exit_code: Option<u32>, message: String) -> bool {
        {
            let mut inner = self.inner.lock().unwrap();
            if matches!(
                inner.state,
                SessionState::Starting | SessionState::Stopping | SessionState::Stopped | SessionState::Failed
            ) {
                return false;
            }
            let Some(m) = inner.machines.iter_mut().find(|m| m.name == machine) else {
                return false;
            };
            if m.lost.is_some() {
                return false;
            }
            m.state = "lost";
            m.lost = Some(LostRecord {
                reason,
                at: shakedown_protocol::time::now(),
                instant: Instant::now(),
                exit_code,
                message: message.clone(),
                dumps: vec![],
                searching: reason == "exited",
            });
            let tail = rpt_tail(&m.profile, 20);
            inner.pending.retain(|_, (target, _, _)| target != machine);
            inner.state = SessionState::Broken;
            inner.reason = Some(json!({"message": message, "machine": machine, "rpt_tail": tail}));
        }
        self.changed.notify_waiters();
        true
    }

    /// The first machine lost at or after `since`, with how it was lost.
    pub fn lost_since(&self, since: Instant) -> Option<(String, LostRecord)> {
        let inner = self.inner.lock().unwrap();
        inner
            .machines
            .iter()
            .filter_map(|m| m.lost.as_ref().map(|l| (m.name.clone(), l.clone())))
            .filter(|(_, l)| l.instant >= since)
            .min_by_key(|(_, l)| l.instant)
    }

    /// Returns once a machine is lost at or after `since`.
    pub async fn wait_lost(&self, since: Instant) -> (String, LostRecord) {
        loop {
            let notified = self.changed.notified();
            if let Some(lost) = self.lost_since(since) {
                return lost;
            }
            tokio::select! {
                _ = notified => {}
                _ = tokio::time::sleep(POLL) => {}
            }
        }
    }

    /// A lost machine's record once the search for its crash dumps has ended, at most
    /// `DUMP_WAIT` and a second later.
    pub async fn settled(&self, machine: &str) -> Option<LostRecord> {
        let deadline = Instant::now() + DUMP_WAIT + Duration::from_secs(1);
        loop {
            let lost = {
                let inner = self.inner.lock().unwrap();
                inner.machines.iter().find(|m| m.name == machine)?.lost.clone()
            }?;
            if !lost.searching || Instant::now() >= deadline {
                return Some(lost);
            }
            tokio::time::sleep(POLL).await;
        }
    }

    /// The first interval the clock watch flagged that is still open or cleared at or after
    /// `from`, a wall-clock time: one that overlaps everything since `from` (`HLT-02`).
    pub fn flag_since(&self, from: &str) -> Option<FlagInterval> {
        let inner = self.inner.lock().unwrap();
        inner
            .flags
            .iter()
            .filter(|f| f.cleared.as_deref().is_none_or(|c| c >= from))
            .min_by(|a, b| a.from.cmp(&b.from))
            .cloned()
    }

    /// The watch of every `ready` machine that is not held, every `WATCH_EVERY`, until the session
    /// stops: a process that ended unasked is lost as `exited` (`P-09`), and a machine whose
    /// heartbeat has not come for `HEARTBEAT_GAP` is lost as `no-answer` (`P-07`, `HLT-02`).
    ///
    /// The gap is the time the runner itself ran without a heartbeat: a round that comes more than
    /// `RUNNER_PAUSE` late shows that the runner was paused, suspended or starved, and the
    /// heartbeats sent meanwhile are still in its connections, so that round judges nothing and
    /// the gap of every machine counts from it (`SES-06`, `US-060`).
    async fn watch(self: Arc<Self>, runner: Arc<Runner>) {
        let mut round = Instant::now();
        let mut resumed: Option<Instant> = None;
        loop {
            tokio::time::sleep(WATCH_EVERY).await;
            let late = round.elapsed();
            round = Instant::now();
            if late >= RUNNER_PAUSE {
                runner.log(&format!(
                    "{} the runner did not run for {:.1} s; the heartbeat gap counts from now",
                    self.id,
                    late.as_secs_f64()
                ));
                resumed = Some(round);
                continue;
            }
            match self.state() {
                SessionState::Stopping | SessionState::Stopped | SessionState::Failed => return,
                SessionState::Starting => continue,
                _ => {}
            }
            struct Seen {
                name: String,
                pid: Option<u32>,
                exit: Option<u32>,
                gap: Duration,
                last: Option<String>,
                asked: Vec<String>,
            }
            let seen: Vec<Seen> = {
                let inner = self.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .filter(|m| m.state == "ready" && !m.held && m.lost.is_none())
                    .map(|m| Seen {
                        name: m.name.clone(),
                        pid: m.pid,
                        exit: m.process.as_ref().and_then(win::exited),
                        gap: if m.left_mission.is_some_and(|t| t.elapsed() < LEAVE_CEILING) {
                            Duration::ZERO
                        } else {
                            m.last_beat
                                .map_or(Duration::ZERO, |t| t.elapsed())
                                .min(resumed.map_or(Duration::MAX, |t| t.elapsed()))
                        },
                        last: m.last_answer_at.clone(),
                        asked: inner
                            .pending
                            .values()
                            .filter(|(target, _, _)| *target == m.name)
                            .map(|(_, _, what)| what.clone())
                            .collect(),
                    })
                    .collect()
            };
            for m in seen {
                if let Some(code) = m.exit {
                    let message = format!(
                        "{}'s process {} exited unasked with exit code {code} ({code:#010X})",
                        m.name,
                        m.pid.map_or("?".into(), |p| p.to_string())
                    );
                    if self.lose(&m.name, "exited", Some(code), message.clone()) {
                        runner.log(&format!("{} {}: lost: {message}", self.id, m.name));
                        tokio::spawn(Arc::clone(&self).find_dumps(m.name.clone()));
                    }
                } else if m.gap >= HEARTBEAT_GAP {
                    let asked = if m.asked.is_empty() {
                        "nothing at the time".to_string()
                    } else {
                        m.asked.join(", ")
                    };
                    let message = format!(
                        "{} stopped answering: no heartbeat came for {} s, its last answer at {}; it was asked {asked}",
                        m.name,
                        m.gap.as_secs(),
                        m.last.as_deref().unwrap_or("none")
                    );
                    if self.lose(&m.name, "no-answer", None, message.clone()) {
                        runner.log(&format!("{} {}: lost: {message}", self.id, m.name));
                    }
                }
            }
        }
    }

    /// Looks in a lost machine's profile folder for its crash dumps, waiting up to `DUMP_WAIT`
    /// for one still being written, and records them (`P-09` step 2).
    async fn find_dumps(self: Arc<Self>, machine: String) {
        let Some(profile) = self.profile(&machine) else { return };
        let deadline = Instant::now() + DUMP_WAIT;
        let mut last = dump_files(&profile);
        loop {
            tokio::time::sleep(POLL).await;
            let now = dump_files(&profile);
            let steady = !now.is_empty() && now == last;
            last = now;
            if steady || Instant::now() >= deadline {
                break;
            }
        }
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(l) = inner
                .machines
                .iter_mut()
                .find(|m| m.name == machine)
                .and_then(|m| m.lost.as_mut())
            {
                l.dumps = last.into_iter().map(|(p, _)| p).collect();
                l.searching = false;
            }
        }
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
    let state = paths::project_state_dir(&spec.project).unwrap_or_else(|_| spec.project.clone());
    missing.extend(steam_and_disk_gaps(
        win::steam_running(),
        crate::logs::steam_account_named() && win::steam_active_user() != Some(false),
        win::free_bytes(&state),
        &state,
    ));
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
        let held = crate::win::udp_ports_held(&(base..base.saturating_add(5)).collect::<Vec<_>>());
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

/// `ENV-01`: of the versions each machine reported, those that differ from the mod's `version`
/// key; the first such machine and a cause naming each mod, machine, version found and version
/// wanted. A mod whose patch a machine did not load counts as found in no version.
fn version_gaps(mods: &[crate::config::Mod], reported: &[(String, Vec<ModVersion>)]) -> Option<(String, String)> {
    let mut first = None;
    let mut gaps = Vec::new();
    for m in mods {
        let Some(wanted) = &m.version else { continue };
        for (machine, versions) in reported {
            let found = versions
                .iter()
                .find(|v| v.mod_name == m.name)
                .and_then(|v| v.version.clone());
            if found.as_deref() == Some(wanted.as_str()) {
                continue;
            }
            first.get_or_insert_with(|| machine.clone());
            gaps.push(format!(
                "mod {} on {machine} is {}, and shakedown.toml wants version {wanted}",
                m.name,
                match found {
                    Some(v) => format!("version {v}"),
                    None => format!(
                        "in no version (its CfgPatches class {} is not loaded)",
                        m.patch.as_deref().unwrap_or("?")
                    ),
                }
            ));
        }
    }
    first.map(|machine| {
        (
            machine,
            format!(
                "a mod was loaded in another version than the project names: {}. Install the version wanted, or set mods.<name>.version in shakedown.toml to the version found",
                gaps.join("; ")
            ),
        )
    })
}

/// The free space a start needs on the drive of the project's `.shakedown` folder (`ENV-01`).
pub const MIN_FREE: u64 = 2 * 1024 * 1024 * 1024;

/// The gaps of `ENV-01` beside the folders and ports of `SES-03`: Steam running, Steam logged in,
/// and 2 GB free on the drive of `state`, the project's `.shakedown` folder. A free space that
/// cannot be read is no gap.
fn steam_and_disk_gaps(running: bool, logged_in: bool, free: Option<u64>, state: &Path) -> Vec<String> {
    let mut gaps = Vec::new();
    if !running {
        gaps.push("a running Steam client of this user (start Steam and log in)".to_string());
    } else if !logged_in {
        gaps.push(
            "an account logged in to Steam: Steam runs, but no account is logged in (log in to Steam)".to_string(),
        );
    }
    if let Some(free) = free.filter(|f| *f < MIN_FREE) {
        let shown = state.display().to_string();
        let drive: String = shown.trim_start_matches(r"\\?\").chars().take(2).collect();
        gaps.push(format!(
            "2 GB free on drive {drive}, which holds {shown}; it has {:.2} GB free (free space on it)",
            free as f64 / (1024.0 * 1024.0 * 1024.0)
        ));
    }
    gaps
}

/// Makes `link` a directory junction to `target`: created when missing, pointed at `target`
/// when it is a junction to another folder, and left alone when it already points there
/// (`INT-AR-06`). A real folder of that name is never touched.
pub(crate) fn junction(link: &Path, target: &Path) -> Result<(), String> {
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
            format!(" {} wrote no RPT line.", reason["machine"].as_str().unwrap_or("?"))
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

    /// Heartbeats one second apart from `tick` 100, each advancing mission time by `rate` of real
    /// time and the frame by 60.
    fn beats(rates: &[f64]) -> Vec<Beat> {
        let mut out = vec![Beat {
            utc: "2026-11-12T12:00:00.000Z".into(),
            tick: 100.0,
            time: 50.0,
            frame: 1000.0,
            answer: 0.01,
        }];
        for (i, rate) in rates.iter().enumerate() {
            let last = out.last().unwrap().clone();
            out.push(Beat {
                utc: format!("2026-11-12T12:00:{:02}.000Z", i + 1),
                tick: last.tick + 1.0,
                time: last.time + rate,
                frame: last.frame + 60.0,
                answer: 0.01,
            });
        }
        out
    }

    /// `HLT-02`, `US-059` 1: mission time that advances less than half as much as real time over
    /// 5 s flags the machine `mission-time`, from the first beat that shows the stop; a clock that
    /// moves is not flagged, nor a history shorter than the window.
    #[test]
    fn us_059_hlt_02_a_stopped_mission_time_is_flagged_from_when_it_stopped() {
        assert!(stopped_clock(&beats(&[1.0; 8])).is_none());
        assert!(stopped_clock(&beats(&[0.0; 4])).is_none());
        let (clock, from, (mission, real), _) =
            stopped_clock(&beats(&[1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0])).unwrap();
        assert_eq!(clock, "mission-time");
        assert_eq!(from, "2026-11-12T12:00:04.000Z");
        assert_eq!(real, 5.0);
        assert!(mission < 2.5, "{mission}");
        // A slow clock, above half of real time, is not stopped.
        assert!(stopped_clock(&beats(&[0.6; 8])).is_none());
        let mut frozen = beats(&[1.0; 6]);
        for b in frozen.iter_mut() {
            b.frame = 7.0;
        }
        assert_eq!(stopped_clock(&frozen).unwrap().0, "frame");
    }

    /// `HLT-02`, `US-059` 2: the flag clears only once both clocks moved over a whole window.
    #[test]
    fn us_059_hlt_02_the_flag_clears_after_5_s_of_moving_clocks() {
        assert!(!moving_again(&beats(&[0.0, 0.0, 1.0, 1.0, 1.0, 1.0])));
        assert!(!moving_again(&beats(&[0.0, 1.0, 1.0, 1.0])));
        assert!(moving_again(&beats(&[0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0])));
    }

    /// `HLT-02`, `INS-04`: the watch over a machine's heartbeats raises its flag into the session's
    /// intervals and the machine's health, and clears both; a machine not `ready` is not watched.
    #[test]
    fn hlt_02_ins_04_the_flag_is_raised_and_cleared_on_the_machine() {
        let mut inner = Inner {
            state: SessionState::Running,
            machines: vec![Machine::new("p1", "client", 1, PathBuf::from("p1-1"))],
            retired: vec![],
            reason: None,
            busy: String::new(),
            job: None,
            pending: HashMap::new(),
            flags: vec![],
            streams: HashMap::new(),
            refs: HashMap::new(),
            errors: vec![],
            events: vec![],
            history: vec![],
            disconnects: vec![],
        };
        inner.machines[0].beats = beats(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!(
            !watch_clocks(&mut inner, "p1"),
            "a machine that is not ready is not watched"
        );
        inner.machines[0].state = "ready";
        assert!(watch_clocks(&mut inner, "p1"));
        let view = machine_view(&inner.machines[0]);
        assert_eq!(view["health"]["flag"], "mission-time");
        assert_eq!(view["health"]["heartbeat"]["frame"], 1360);
        assert_eq!(inner.flags.len(), 1);
        assert!(!watch_clocks(&mut inner, "p1"), "the flag is raised once");
        inner.machines[0].beats = beats(&[0.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        assert!(watch_clocks(&mut inner, "p1"));
        assert!(machine_view(&inner.machines[0])["health"]["flag"].is_null());
        assert!(inner.flags[0].cleared.is_some());
    }

    /// `P-07` step 2: an order is named in the words of a lost machine's message.
    #[test]
    fn us_072_evt_01_obs_05_an_event_message_becomes_a_recorded_event() {
        let e = recorded_event(&json!({
            "kind": "event", "machine": "p2", "key": "r#1", "name": "skdfx_signalled", "type": "cba",
            "args": [{"$type": "object", "netId": "2:5", "class": "B_Soldier_F", "alive": true}],
            "at": {"utc": "2026-11-15T10:00:00.000Z", "tick": "12.500", "time": "3.250", "frame": "120"}
        }))
        .unwrap();
        assert_eq!(
            (e.machine.as_str(), e.name.as_str(), e.kind.as_str()),
            ("p2", "skdfx_signalled", "cba")
        );
        assert_eq!((e.at.tick, e.at.time, e.at.frame), (12.5, 3.25, 120));
        assert_eq!(e.args[0]["netId"], "2:5");
    }

    fn fixture_mod(version: Option<&str>) -> crate::config::Mod {
        crate::config::Mod {
            name: "fixture".into(),
            path: PathBuf::from("f"),
            patch: Some("skdfx_main".into()),
            version: version.map(String::from),
            source: None,
            prefix: None,
            rebuild: None,
        }
    }

    fn reported(machine: &str, version: Option<&str>) -> (String, Vec<ModVersion>) {
        (
            machine.into(),
            vec![ModVersion {
                mod_name: "fixture".into(),
                patch: "skdfx_main".into(),
                version: version.map(String::from),
            }],
        )
    }

    #[test]
    fn us_086_env_01_a_mod_in_another_version_names_the_mod_the_version_found_and_wanted() {
        let gaps = version_gaps(
            &[fixture_mod(Some("9.9.9"))],
            &[
                reported("server", Some("0.0.6")),
                reported("p1", Some("9.9.9")),
                reported("p2", None),
            ],
        )
        .unwrap();
        assert_eq!(gaps.0, "server");
        assert!(
            gaps.1
                .contains("mod fixture on server is version 0.0.6, and shakedown.toml wants version 9.9.9"),
            "{}",
            gaps.1
        );
        assert!(
            gaps.1
                .contains("mod fixture on p2 is in no version (its CfgPatches class skdfx_main is not loaded)"),
            "{}",
            gaps.1
        );
        assert!(!gaps.1.contains("on p1"), "{}", gaps.1);
    }

    #[test]
    fn us_086_env_01_the_right_version_or_no_version_key_passes() {
        assert!(version_gaps(&[fixture_mod(Some("0.0.6"))], &[reported("p1", Some("0.0.6"))]).is_none());
        assert!(version_gaps(&[fixture_mod(None)], &[reported("p1", Some("0.0.6"))]).is_none());
    }

    #[test]
    fn us_085_env_01_steam_logged_out_and_a_full_drive_are_each_a_gap() {
        let state = Path::new(r"C:\p\.shakedown");
        assert!(steam_and_disk_gaps(true, true, Some(MIN_FREE), state).is_empty());
        assert!(steam_and_disk_gaps(true, true, None, state).is_empty());
        let gaps = steam_and_disk_gaps(true, false, Some(MIN_FREE / 4), state);
        assert_eq!(gaps.len(), 2, "{gaps:?}");
        assert!(gaps[0].contains("no account is logged in"), "{gaps:?}");
        assert!(
            gaps[1].contains("drive C:") && gaps[1].contains("0.50 GB free"),
            "{gaps:?}"
        );
        let gaps = steam_and_disk_gaps(false, false, Some(MIN_FREE), state);
        assert_eq!(
            gaps,
            vec!["a running Steam client of this user (start Steam and log in)".to_string()]
        );
    }

    #[cfg(windows)]
    #[test]
    fn int_stm_01_active_user_zero_is_no_account() {
        let reg = "HKEY_CURRENT_USER\\Software\\Valve\\Steam\\ActiveProcess\n    ActiveUser    REG_DWORD    0x0\n";
        assert_eq!(win::active_user_set(reg), Some(false));
        assert_eq!(win::active_user_set(&reg.replace("0x0", "0x1a2b")), Some(true));
        assert_eq!(win::active_user_set("ERROR: not found"), None);
    }

    #[test]
    fn p_07_an_order_is_described_for_a_lost_machine() {
        assert_eq!(describe(&json!({"kind": "check", "check_id": "x"})), "to run check x");
        let long = "a".repeat(200);
        let d = describe(&json!({"kind": "poll", "source": long}));
        assert!(d.starts_with("to poll `aaa") && d.ends_with("...`"), "{d}");
    }

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
