//! A run (`P-04`): every selected check executed in order against the session (`P-05`), and the
//! result file written at its end (`RES-01`).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::result::{
    Attached, CheckExecution, Clocks, Evidence, NotRun, Reason, ResultFile, Run, SCHEMA, Selection,
    Session as RSession, Setup as RSetup, Step, Summary,
};
use shakedown_protocol::time;

use super::Runner;
use super::session::{GRACE, OrderError, Session, SessionState};
use crate::checks::{self, CheckRef, Header};
use crate::error::CliError;
use crate::paths;

pub async fn run(
    runner: &Arc<Runner>,
    project: PathBuf,
    checks: Vec<CheckRef>,
    ids: Vec<String>,
    tags: Vec<String>,
    emit: &(dyn Fn(Value, String) + Sync),
) -> Result<Value, CliError> {
    let Some(session) = runner.session() else {
        return Err(CliError::not_now(
            "No session runs. Start one with `shakedown session start --setup <name>`, or give `run` a --setup.",
        ));
    };
    if session.state() != SessionState::Ready {
        return Err(CliError::not_now(format!(
            "Session {} is {}, not ready, so it cannot run checks now. Wait for it, or stop it with `shakedown session stop`.",
            session.id,
            session_state_word(session.state())
        )));
    }
    session.set_state(SessionState::Running);
    let result = execute(runner, &session, project, checks, ids, tags, emit).await;
    if session.state() == SessionState::Running {
        session.set_state(SessionState::Ready);
    }
    result
}

fn session_state_word(state: SessionState) -> &'static str {
    match state {
        SessionState::Starting => "starting",
        SessionState::Ready => "ready",
        SessionState::Running => "running a run",
        SessionState::Failed => "failed",
        SessionState::Stopping => "stopping",
        SessionState::Stopped => "stopped",
    }
}

async fn execute(
    runner: &Arc<Runner>,
    session: &Arc<Session>,
    project: PathBuf,
    checks: Vec<CheckRef>,
    ids: Vec<String>,
    tags: Vec<String>,
    emit: &(dyn Fn(Value, String) + Sync),
) -> Result<Value, CliError> {
    let started = SystemTime::now();
    let clock = Instant::now();
    let run_id = format!("r-{}-{}", time::stamp(started), runner.next_run_number());
    let state_dir = paths::project_state_dir(&project)
        .map_err(|e| CliError::framework(format!("Cannot create .shakedown in {}: {e}.", project.display())))?;
    let folder = state_dir.join("runs").join(&run_id);
    std::fs::create_dir_all(&folder)
        .map_err(|e| CliError::framework(format!("Cannot create the run folder {}: {e}.", folder.display())))?;
    runner.log(&format!("{} run {run_id}: {} checks", session.id, checks.len()));

    let mut executions: Vec<CheckExecution> = Vec::new();
    let mut not_run: Vec<NotRun> = Vec::new();
    for check in &checks {
        let execution = execute_check(session, &run_id, check).await;
        let Some(execution) = execution else {
            not_run.push(NotRun {
                check_id: check.id.clone(),
                reason: "setup".into(),
            });
            continue;
        };
        let line = outcome_line(&execution, executions.len());
        emit(line, outcome_text(&execution));
        executions.push(execution);
    }

    let summary = Summary::of(&executions, not_run.len());
    let result = ResultFile {
        schema: SCHEMA.into(),
        framework_version: shakedown_protocol::VERSION.into(),
        run: Run {
            id: run_id.clone(),
            state: "finished".into(),
            started_at: time::format_system_time(started),
            ended_at: time::now(),
            duration_s: round3(clock.elapsed().as_secs_f64()),
            selection: Selection { ids, tags },
            stopped_reason: None,
        },
        session: RSession {
            id: session.id.clone(),
            setup: RSetup {
                name: session.spec.setup.name.clone(),
                mode: session.spec.setup.mode.as_str().into(),
                clients: session.spec.setup.clients,
                headless: session.spec.setup.headless,
                world: session.spec.setup.world.clone(),
                mods: session.spec.setup.mods.clone(),
            },
            machines: session.result_machines(),
        },
        summary: summary.clone(),
        checks: executions.clone(),
        not_run,
        reloads: vec![],
        logs: BTreeMap::new(),
    };
    let file = folder.join("result.json");
    let text = serde_json::to_string_pretty(&result).expect("the result serialises");
    std::fs::write(&file, text)
        .map_err(|e| CliError::framework(format!("Cannot write the result file {}: {e}.", file.display())))?;
    runner.log(&format!("{} run {run_id}: done, {}", session.id, file.display()));

    Ok(json!({
        "run_id": run_id,
        "state": "finished",
        "summary": summary,
        "outcomes": executions.iter().enumerate().map(|(i, e)| {
            let mut line = outcome_line(e, i);
            line["failed_step"] = json!(e.evidence.failed_step);
            line["evidence_path"] = json!(format!("result.json#/checks/{i}/evidence"));
            line
        }).collect::<Vec<_>>(),
        "result_file": file,
    }))
}

/// Executes one check (`P-05`); `None` when its `@setups` excludes the session's setup.
async fn execute_check(session: &Arc<Session>, run_id: &str, check: &CheckRef) -> Option<CheckExecution> {
    let started = SystemTime::now();
    let clock = Instant::now();
    // The check is read fresh from disk for every execution (RLD-04).
    let text = std::fs::read_to_string(&check.path);
    let header = text
        .as_ref()
        .map_err(|e| format!("cannot read the check file: {e}"))
        .and_then(|t| checks::parse_header(t));
    let header = match header {
        Ok(h) => h,
        Err(message) => {
            let reason = Reason {
                kind: "check-error".into(),
                message: format!("The check header is wrong, {message} Fix the header and run the check again."),
                machine: None,
                at: time::now(),
            };
            return Some(ended(
                check,
                None,
                started,
                clock,
                "fail",
                Some(reason),
                vec![],
                Evidence::default(),
            ));
        }
    };
    if let Some(setups) = &header.setups
        && !setups.contains(&session.spec.setup.name)
    {
        return None;
    }
    let lead = session.lead();
    let order = json!({
        "kind": "check",
        "run_id": run_id,
        "check_id": check.id,
        "source": checks::body_source(text.as_deref().unwrap_or_default()),
        "timeout": header.timeout_s,
        "machines": session.machine_names(),
        "mode": session.spec.setup.mode.as_str(),
    });
    let ceiling = Duration::from_secs(header.timeout_s) + GRACE;
    match session.order(lead, order, ceiling).await {
        Ok(report) => Some(from_report(check, &header, started, clock, &report, lead)),
        Err(e) => {
            let why = match e {
                OrderError::NotConnected | OrderError::Lost => {
                    format!("{lead} lost its connection to the runner during the check")
                }
                OrderError::Ceiling => format!(
                    "{lead} sent no end report within the check's timeout of {} s plus the grace of {} s",
                    header.timeout_s,
                    GRACE.as_secs()
                ),
            };
            let reason = Reason {
                kind: "machine-lost".into(),
                message: format!("{why}."),
                machine: Some(lead.into()),
                at: time::now(),
            };
            Some(ended(
                check,
                Some(&header),
                started,
                clock,
                "invalid",
                Some(reason),
                vec![],
                Evidence::default(),
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn ended(
    check: &CheckRef,
    header: Option<&Header>,
    started: SystemTime,
    clock: Instant,
    outcome: &str,
    reason: Option<Reason>,
    steps: Vec<Step>,
    evidence: Evidence,
) -> CheckExecution {
    CheckExecution {
        check_id: check.id.clone(),
        name: header.map(|h| h.name.clone()).unwrap_or_default(),
        tags: header.map(|h| h.tags.clone()).unwrap_or_default(),
        repeat: 1,
        outcome: outcome.into(),
        reason,
        started_at: time::format_system_time(started),
        ended_at: time::now(),
        duration_s: round3(clock.elapsed().as_secs_f64()),
        reload: "full".into(),
        steps,
        errors: vec![],
        evidence,
    }
}

/// Turns the lead's end report into a check execution.
fn from_report(
    check: &CheckRef,
    header: &Header,
    started: SystemTime,
    clock: Instant,
    report: &Value,
    lead: &str,
) -> CheckExecution {
    let outcome = report["outcome"]
        .as_str()
        .filter(|o| matches!(*o, "pass" | "fail" | "invalid"))
        .unwrap_or("invalid");
    let reason = if outcome == "pass" {
        None
    } else {
        let r = &report["reason"];
        Some(Reason {
            kind: r["kind"].as_str().unwrap_or("machine-lost").into(),
            message: r["message"]
                .as_str()
                .filter(|m| !m.is_empty())
                .unwrap_or("The lead's report gave no reason.")
                .into(),
            machine: r["machine"].as_str().map(String::from).or_else(|| Some(lead.into())),
            at: r["at"].as_str().map(String::from).unwrap_or_else(time::now),
        })
    };
    let steps = report["steps"]
        .as_array()
        .map(|steps| steps.iter().filter_map(step).collect())
        .unwrap_or_default();
    let ev = &report["evidence"];
    let evidence = Evidence {
        failed_step: ev["failed_step"]
            .as_array()
            .map(|p| p.iter().filter_map(|s| s.as_str().map(String::from)).collect()),
        observed: ev["observed"]
            .as_object()
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        attached: ev["attached"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|x| Attached {
                        label: x["label"].as_str().unwrap_or_default().into(),
                        value: x["value"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        logs: BTreeMap::new(),
    };
    ended(check, Some(header), started, clock, outcome, reason, steps, evidence)
}

fn number(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn clocks(v: &Value) -> Option<Clocks> {
    Some(Clocks {
        utc: v["utc"].as_str()?.to_string(),
        tick: number(&v["tick"])?,
        time: number(&v["time"])?,
        frame: number(&v["frame"])?.max(0.0) as u64,
    })
}

fn step(v: &Value) -> Option<Step> {
    Some(Step {
        path: v["path"]
            .as_array()?
            .iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect(),
        machine: v["machine"].as_str()?.to_string(),
        state: v["state"].as_str()?.to_string(),
        start: clocks(&v["start"])?,
        end: clocks(&v["end"]),
        duration_s: number(&v["duration_s"]).unwrap_or(0.0).max(0.0),
        error: v["error"].as_str().map(String::from),
    })
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The Outcome line of `contract/mcp-tools.md`.
fn outcome_line(e: &CheckExecution, _index: usize) -> Value {
    json!({
        "check_id": e.check_id,
        "repeat": e.repeat,
        "outcome": e.outcome,
        "reason": e.reason.as_ref().map(|r| json!({"kind": r.kind, "message": r.message})),
        "duration_s": e.duration_s,
    })
}

/// One line for a person.
fn outcome_text(e: &CheckExecution) -> String {
    let mut text = format!("{:<7} {}  ({:.1} s)", e.outcome, e.check_id, e.duration_s);
    if let Some(r) = &e.reason {
        text.push_str(&format!("\n        {}: {}", r.kind, r.message));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(result: &ResultFile) {
        let schema: Value = serde_json::from_str(include_str!("../../../../spec/contract/result.schema.json")).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let instance = serde_json::to_value(result).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(&instance)
            .map(|e| format!("{e} at {}", e.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{errors:#?}");
    }

    fn sample(outcome: &str) -> ResultFile {
        let check = CheckRef {
            id: "smoke".into(),
            path: PathBuf::from("smoke.sqf"),
        };
        let header = checks::parse_header("// @check US-037 smoke\n// @tags smoke\n").unwrap();
        let report = json!({
            "kind": "report", "order": "o1", "outcome": outcome,
            "reason": if outcome == "pass" { Value::Null } else { json!({"kind": "assertion", "message": "x: expected == 2, got 3.", "at": "2026-11-05T12:00:01.000Z", "machine": "p1"}) },
            "steps": [
                {"path": ["outer"], "machine": "p1", "state": "passed",
                 "start": {"utc": "2026-11-05T12:00:00.000Z", "tick": "10.500", "time": "1.250", "frame": "120"},
                 "end": {"utc": "2026-11-05T12:00:01.000Z", "tick": "11.500", "time": "2.250", "frame": "180"},
                 "duration_s": "1.000"},
                {"path": ["outer", "inner"], "machine": "p1", "state": "running",
                 "start": {"utc": "2026-11-05T12:00:00.500Z", "tick": "11.000", "time": "1.750", "frame": "150"},
                 "duration_s": "0.500"}
            ],
            "evidence": {"attached": [{"label": "answer", "value": 42}], "failed_step": ["outer", "inner"], "observed": {"p1": 3}}
        });
        let e = from_report(&check, &header, SystemTime::now(), Instant::now(), &report, "p1");
        let summary = Summary::of(std::slice::from_ref(&e), 0);
        ResultFile {
            schema: SCHEMA.into(),
            framework_version: "0.0.1".into(),
            run: Run {
                id: "r-20261105-120000-1".into(),
                state: "finished".into(),
                started_at: "2026-11-05T12:00:00.000Z".into(),
                ended_at: "2026-11-05T12:00:02.000Z".into(),
                duration_s: 2.0,
                selection: Selection {
                    ids: vec!["smoke".into()],
                    tags: vec![],
                },
                stopped_reason: None,
            },
            session: RSession {
                id: "s-20261105-115900".into(),
                setup: RSetup {
                    name: "sp".into(),
                    mode: "singleplayer".into(),
                    clients: 1,
                    headless: 0,
                    world: "VR".into(),
                    mods: vec![],
                },
                machines: vec![shakedown_protocol::result::Machine {
                    name: "p1".into(),
                    kind: "client".into(),
                    start: 1,
                    game_version: "2.22.154089 stable".into(),
                    mods: vec![],
                    end_state: "ready".into(),
                }],
            },
            summary,
            checks: vec![e],
            not_run: vec![],
            reloads: vec![],
            logs: BTreeMap::new(),
        }
    }

    #[test]
    fn res_01_a_passing_result_is_valid_against_the_schema() {
        let r = sample("pass");
        assert!(r.checks[0].reason.is_none());
        assert_eq!(r.checks[0].steps.len(), 2);
        assert_eq!(r.checks[0].steps[0].start.frame, 120);
        assert_eq!(r.checks[0].evidence.attached[0].value, json!(42));
        validate(&r);
    }

    #[test]
    fn res_01_a_failing_result_is_valid_against_the_schema() {
        let r = sample("fail");
        assert_eq!(r.checks[0].reason.as_ref().unwrap().kind, "assertion");
        assert_eq!(r.summary.fail, 1);
        validate(&r);
    }

    #[test]
    fn us_038_a_header_error_ends_fail_check_error_without_the_game() {
        let check = CheckRef {
            id: "bad".into(),
            path: PathBuf::from("bad.sqf"),
        };
        let e = ended(
            &check,
            None,
            SystemTime::now(),
            Instant::now(),
            "fail",
            Some(Reason {
                kind: "check-error".into(),
                message: "The check header is wrong at line 2".into(),
                machine: None,
                at: time::now(),
            }),
            vec![],
            Evidence::default(),
        );
        let mut r = sample("pass");
        r.checks = vec![e];
        r.summary = Summary::of(&r.checks, 0);
        validate(&r);
    }
}
