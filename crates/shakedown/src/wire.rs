//! What a client and the runner say to each other after the hello: one JSON object per line.
//! A client sends one request at a time; the runner answers with events, the last of which is
//! `session`, `run_done` or `error`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::checks::CheckRef;
use crate::config::{Mod, Setup};
use crate::error::CliError;

/// A setup with everything the runner needs to start it, resolved by the client from the
/// project file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetupSpec {
    pub setup: Setup,
    pub game: PathBuf,
    pub mods: Vec<Mod>,
    /// The server's game port; it and the next four are the session's (`SES-03`).
    pub base_port: u16,
}

/// The `rebuild` commands of the project's mods, read fresh from the project file, and the folder
/// they run in, the project's (`RLD-06`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Rebuild {
    pub folder: PathBuf,
    /// Each mod with a `rebuild`, by name, and its command.
    pub commands: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    /// The running session, or none.
    SessionInfo,
    SessionStart {
        project: PathBuf,
        spec: SetupSpec,
    },
    SessionStop,
    /// The runner's own state for `status` (`INS-02`).
    Status,
    /// A reset outside a run (`RST-03`).
    Reset,
    /// A reload of the files given, or of what changed (`RLD-01`); the answer comes when it is
    /// done, or after `wait_s` with state `applying`.
    Reload {
        files: Vec<PathBuf>,
        /// The tests folder, the project file's `tests`.
        tests: PathBuf,
        #[serde(default)]
        rebuild: Rebuild,
        wait_s: Option<f64>,
    },
    /// A machine's log lines, redacted (`LOG-03`).
    Logs {
        project: PathBuf,
        machine: String,
        run: Option<String>,
        check: Option<String>,
        grep: Option<String>,
        last: usize,
    },
    Run {
        project: PathBuf,
        /// The setups the project file declares, so a check naming another is a check-error (`DEF-06`).
        setups: Vec<String>,
        checks: Vec<CheckRef>,
        ids: Vec<String>,
        tags: Vec<String>,
        /// The id `ReserveRun` gave, for a run whose id the client handed out before it began.
        #[serde(default)]
        run_id: Option<String>,
    },
    /// A run id handed out before the run begins, so `run_start` returns it at once (`AGT-03`).
    ReserveRun,
    /// Stops a run, the one with this id or the active one, and answers once its result file is
    /// written (`RUN-05`).
    RunStop {
        #[serde(default)]
        run_id: Option<String>,
    },
    /// SQF on a machine or a group (`CTL-01`, `CTL-05`).
    Exec {
        machine: String,
        code: String,
        args: Value,
        ceiling: f64,
        /// Who sent it, for the audit (`SEC-04`): `cli` or `mcp`.
        #[serde(default)]
        source: String,
    },
    /// A wait for a condition on a machine (`OBS-01`).
    Wait {
        machine: String,
        condition: String,
        description: Option<String>,
        args: Value,
        ceiling: f64,
    },
    /// An assertion tried live (`OBS-06`).
    Expect {
        machine: String,
        value: String,
        op: String,
        expected: Value,
        args: Value,
        ceiling: f64,
    },
    /// A machine's state summary (`INS-01`).
    Inspect {
        machine: String,
        near: Option<String>,
        radius: f64,
    },
    /// The script errors the session caught (`ERR-05`).
    Errors {
        machine: Option<String>,
        run: Option<String>,
        check: Option<String>,
    },
}

impl Request {
    /// A request whose work in the game stops when its client goes away before the answer
    /// (`AGT-04`). The others run to their end, so no session or run is left half done.
    pub fn cancellable(&self) -> bool {
        matches!(
            self,
            Request::Exec { .. } | Request::Wait { .. } | Request::Expect { .. } | Request::Inspect { .. }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// A line for a person while a request is in progress.
    Progress {
        message: String,
    },
    /// A run began: its id and how many check executions it has.
    RunStarted {
        run_id: String,
        total: usize,
    },
    /// One check execution ended.
    Check {
        line: Value,
        text: String,
    },
    /// The answer to a session request: the session, or none.
    Session {
        session: Option<Value>,
    },
    /// An answer whose shape is that of the command's MCP twin.
    Value {
        value: Value,
    },
    /// The answer to a run: the `run_result` shape of `contract/mcp-tools.md`.
    RunDone {
        result: Value,
    },
    Error {
        error: CliError,
    },
}
