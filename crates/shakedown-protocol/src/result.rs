//! The result file, `result.json`, as `spec/contract/result-file.md` and
//! `spec/contract/result.schema.json` define it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCHEMA: &str = "shakedown/result/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultFile {
    pub schema: String,
    pub framework_version: String,
    pub run: Run,
    pub session: Session,
    pub summary: Summary,
    pub checks: Vec<CheckExecution>,
    pub not_run: Vec<NotRun>,
    pub reloads: Vec<Value>,
    pub logs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub state: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_s: f64,
    pub selection: Selection,
    pub stopped_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub setup: Setup,
    pub machines: Vec<Machine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub name: String,
    pub mode: String,
    pub clients: u32,
    pub headless: u32,
    pub world: String,
    pub mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub name: String,
    pub kind: String,
    pub start: u32,
    pub game_version: String,
    pub mods: Vec<ModVersion>,
    pub end_state: String,
    /// How and when a machine that was `lost` was lost (`RES-05`, `P-07`, `P-09`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lost: Option<Lost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lost {
    /// `exited` or `no-answer`.
    pub reason: String,
    pub at: String,
    pub exit_code: Option<u32>,
    /// The crash dumps copied into the run folder, under `dumps/<machine>/`.
    pub dumps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    #[serde(rename = "mod")]
    pub mod_name: String,
    pub patch: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub total: u32,
    pub pass: u32,
    pub fail: u32,
    pub invalid: u32,
    pub not_run: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub machine: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clocks {
    pub utc: String,
    pub tick: f64,
    pub time: f64,
    pub frame: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub path: Vec<String>,
    pub machine: String,
    pub state: String,
    pub start: Clocks,
    pub end: Option<Clocks>,
    pub duration_s: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attached {
    pub label: String,
    pub value: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Evidence {
    pub failed_step: Option<Vec<String>>,
    pub observed: Option<BTreeMap<String, Value>>,
    pub attached: Vec<Attached>,
    pub logs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckExecution {
    pub check_id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub repeat: u32,
    pub outcome: String,
    pub reason: Option<Reason>,
    pub started_at: String,
    pub ended_at: String,
    pub duration_s: f64,
    pub reload: String,
    pub steps: Vec<Step>,
    pub errors: Vec<Value>,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotRun {
    pub check_id: String,
    pub reason: String,
}

impl Summary {
    /// Counts the outcomes of `checks` and the checks not run.
    pub fn of(checks: &[CheckExecution], not_run: usize) -> Self {
        let count = |o: &str| checks.iter().filter(|c| c.outcome == o).count() as u32;
        let (pass, fail, invalid) = (count("pass"), count("fail"), count("invalid"));
        Summary {
            total: pass + fail + invalid + not_run as u32,
            pass,
            fail,
            invalid,
            not_run: not_run as u32,
        }
    }
}
