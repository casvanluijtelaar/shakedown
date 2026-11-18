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
    /// Each selected check's summary over its repeats, when the run repeated them (`OUT-02`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeats: Option<Vec<RepeatSummary>>,
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
    /// How many times each selected check runs in a row (`OUT-02`).
    #[serde(default = "one")]
    pub repeat: u32,
    /// The project's `slow_step` the run flagged steps against (`TIM-01`).
    #[serde(default = "ten")]
    pub slow_step_s: f64,
    pub stopped_reason: Option<String>,
}

fn one() -> u32 {
    1
}

fn ten() -> f64 {
    10.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    pub ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// `None` when the run's session start was refused before a session existed (`RUN-07`).
    pub id: Option<String>,
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
    /// Longer than the run's `slow_step_s` (`TIM-01`).
    #[serde(default)]
    pub slow: bool,
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
    /// Its duration was more than half its timeout (`TIM-01`).
    #[serde(default)]
    pub slow: bool,
    pub steps: Vec<Step>,
    pub errors: Vec<Value>,
    /// Every event the check recorded, on every machine, in wall-clock order (`EVT-01`).
    #[serde(default)]
    pub events: Vec<RecordedEvent>,
    pub evidence: Evidence,
    /// Only on the execution that handed the session over to a manual tester, which is `pass`
    /// (`MAN-01`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handed_over: Option<HandedOver>,
}

/// A hand over to a manual tester (`MAN-01`): the game client whose window to use and the things
/// to look for, in the order the check gave them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandedOver {
    pub machine: String,
    pub items: Vec<String>,
}

/// An event a check recorded with `skd_fnc_record` on one machine (`EVT-01`, `OBS-05`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub machine: String,
    pub name: String,
    /// `cba` or `mission`.
    pub kind: String,
    pub args: Value,
    pub at: Clocks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotRun {
    pub check_id: String,
    /// The repeat the execution would have had.
    #[serde(default = "one")]
    pub repeat: u32,
    /// `setup`, `run-stopped`, `session-failed`, `session-broken` or `handed-over`.
    pub reason: String,
    /// For `session-failed`: the start's reason (`RUN-07`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// One check's summary over its repeats (`DM-09`, `OUT-02`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepeatSummary {
    pub check_id: String,
    /// `pass`, `fail`, `flaky`, `invalid` or `not-run`.
    pub outcome: String,
    pub counts: Counts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Counts {
    pub pass: u32,
    pub fail: u32,
    pub invalid: u32,
    pub not_run: u32,
}

impl Counts {
    /// The summary outcome of `DM-09`, read from the repeats that ran.
    pub fn outcome(&self) -> &'static str {
        match (self.pass, self.fail, self.invalid) {
            (0, 0, 0) => "not-run",
            (0, 0, _) => "invalid",
            (_, 0, _) => "pass",
            (0, _, _) => "fail",
            _ => "flaky",
        }
    }
}

/// The summaries of `DM-09` of every check in `order` (each id once, in the order first
/// selected), from the run's executions and the executions it did not run.
pub fn repeat_summaries(order: &[String], checks: &[CheckExecution], not_run: &[NotRun]) -> Vec<RepeatSummary> {
    let mut ids: Vec<&String> = Vec::new();
    for id in order {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids.into_iter()
        .map(|id| {
            let mut counts = Counts::default();
            for c in checks.iter().filter(|c| &c.check_id == id) {
                match c.outcome.as_str() {
                    "pass" => counts.pass += 1,
                    "fail" => counts.fail += 1,
                    _ => counts.invalid += 1,
                }
            }
            counts.not_run = not_run.iter().filter(|n| &n.check_id == id).count() as u32;
            RepeatSummary {
                check_id: id.clone(),
                outcome: counts.outcome().into(),
                counts,
            }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pass: u32, fail: u32, invalid: u32, not_run: u32) -> Counts {
        Counts {
            pass,
            fail,
            invalid,
            not_run,
        }
    }

    #[test]
    fn us_076_us_077_dm_09_the_summary_outcome_is_read_from_the_repeats_that_ran() {
        assert_eq!(counts(10, 0, 0, 0).outcome(), "pass");
        assert_eq!(counts(0, 10, 0, 0).outcome(), "fail");
        assert_eq!(counts(6, 4, 0, 0).outcome(), "flaky");
        assert_eq!(counts(2, 0, 1, 2).outcome(), "pass", "US-077 1");
        assert_eq!(counts(0, 3, 1, 0).outcome(), "fail");
        assert_eq!(counts(0, 0, 1, 4).outcome(), "invalid", "US-077 2");
        assert_eq!(counts(0, 0, 0, 5).outcome(), "not-run");
        assert_eq!(counts(1, 1, 3, 0).outcome(), "flaky");
    }
}
