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
    Run {
        project: PathBuf,
        checks: Vec<CheckRef>,
        ids: Vec<String>,
        tags: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// A line for a person while a request is in progress.
    Progress {
        message: String,
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
    /// The answer to a run: the `run_result` shape of `contract/mcp-tools.md`.
    RunDone {
        result: Value,
    },
    Error {
        error: CliError,
    },
}
