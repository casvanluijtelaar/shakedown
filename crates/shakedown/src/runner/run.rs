//! A run (`P-04`): every selected check executed in order against the session (`P-05`), and the
//! result file written at its end (`RES-01`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::result::{
    Attached, CheckExecution, Clocks, Evidence, NotRun, Reason, ResultFile, Run, SCHEMA, Selection,
    Session as RSession, Setup as RSetup, Step, Summary,
};
use shakedown_protocol::time;
use tokio::sync::watch;

use super::session::{GRACE, OrderError, Session, SessionState};
use super::{Runner, errors};
use crate::checks::{self, CheckRef, Header};
use crate::error::CliError;
use crate::paths;

/// A new run id, `r-<UTC time>-<n>`, with its stop switch, so the run can be stopped from the
/// moment its id is handed out (`RUN-05`).
pub fn new_run_id(runner: &Runner) -> String {
    let id = format!("r-{}-{}", time::stamp(SystemTime::now()), runner.next_run_number());
    control(runner, &id);
    id
}

/// The `stopped_reason` of a run stopped with `run stop`, `run_stop`, Ctrl+C or Ctrl+Break
/// (`RUN-05`).
pub const BY_HAND: &str = "by-hand";
/// The `stopped_reason` of a run stopped at its runner ceiling (`RUN-06`).
pub const RUN_CEILING: &str = "run-ceiling";
/// The `stopped_reason` of a run whose session was stopped under it (`P-10` step 1).
pub const SESSION_STOPPED: &str = "session-stopped";

/// What the run's own margin adds to the sum of its checks' timeouts (`RUN-06`).
const CEILING_MARGIN: Duration = Duration::from_secs(60);
/// How long a stop waits for the run to write its result file before it answers; under the 60 s
/// of a tool call (`AGT-03`).
const STOP_WAIT: Duration = Duration::from_secs(50);

/// A run's stop switch, set once by the first reason that comes, and its result file once it is
/// written.
pub struct Control {
    stop: watch::Sender<Option<&'static str>>,
    result: Mutex<Option<PathBuf>>,
}

impl Control {
    fn new() -> Control {
        Control {
            stop: watch::Sender::new(None),
            result: Mutex::new(None),
        }
    }

    /// Asks the run to stop; a reason given earlier stays.
    pub fn request(&self, reason: &'static str) {
        self.stop.send_if_modified(|r| {
            if r.is_none() {
                *r = Some(reason);
                true
            } else {
                false
            }
        });
    }

    pub fn reason(&self) -> Option<&'static str> {
        *self.stop.borrow()
    }

    /// Returns once the run is asked to stop, with the reason.
    async fn stopped(&self) -> &'static str {
        let mut rx = self.stop.subscribe();
        let reason = rx.wait_for(|r| r.is_some()).await.map(|r| r.unwrap_or(BY_HAND));
        match reason {
            Ok(r) => r,
            // The sender lives as long as `self`, so this is never reached.
            Err(_) => std::future::pending().await,
        }
    }

    pub fn result(&self) -> Option<PathBuf> {
        self.result.lock().unwrap().clone()
    }
}

/// The stop switch of the run `id`, made when it is first asked for.
pub fn control(runner: &Runner, id: &str) -> Arc<Control> {
    Arc::clone(
        runner
            .controls
            .lock()
            .unwrap()
            .entry(id.to_string())
            .or_insert_with(|| Arc::new(Control::new())),
    )
}

/// The id of the active run, if one is active.
fn active_run(runner: &Runner) -> Option<String> {
    runner.progress.lock().unwrap().as_ref()?["id"]
        .as_str()
        .map(String::from)
}

/// Waits while the run `id` is active, at most `ceiling`.
async fn until_ended(runner: &Runner, id: &str, ceiling: Duration) {
    let clock = Instant::now();
    while active_run(runner).as_deref() == Some(id) && clock.elapsed() < ceiling {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// `run stop` and `run_stop` (`RUN-05`): asks the run `id`, or the active run when `id` is
/// `None`, to stop, and returns once its result file is written, at most `STOP_WAIT` later:
/// `{"run_id", "result_file"}`, the file `null` for a run that has not begun. A run whose id was
/// handed out before it began stops as soon as it begins, with every check not run.
pub async fn stop(runner: &Runner, id: Option<String>) -> Result<Value, CliError> {
    let id = match id {
        Some(id) => id,
        None => active_run(runner).ok_or_else(|| {
            CliError::not_now(
                "No run is active, so there is nothing to stop. `shakedown status` shows the session and any active run.",
            )
            .tag("unknown-run")
        })?,
    };
    let control = runner.controls.lock().unwrap().get(&id).cloned().ok_or_else(|| {
        CliError::wrong(format!(
            "No run has the id {id}. Give the run_id that run_start returned, or leave it out to stop the active run."
        ))
        .tag("unknown-run")
    })?;
    if control.result().is_none() {
        control.request(BY_HAND);
        runner.log(&format!("run {id}: stop asked by hand"));
        until_ended(runner, &id, STOP_WAIT).await;
    }
    Ok(json!({"run_id": id, "result_file": control.result()}))
}

/// Stops the active run, if any, for `reason`, and waits for its result file (`P-10` step 1).
pub async fn stop_active(runner: &Runner, reason: &'static str) {
    let Some(id) = active_run(runner) else { return };
    control(runner, &id).request(reason);
    runner.log(&format!("run {id}: stop asked, {reason}"));
    until_ended(runner, &id, STOP_WAIT).await;
}

/// The runner ceiling of a run (`RUN-06`): the sum of the timeouts of the checks it will execute
/// in the session's setup, read from their headers, plus 60 s. A check whose header cannot be read
/// ends at once and adds nothing.
fn run_ceiling(checks: &[CheckRef], declared: &[String], setup: &str) -> Duration {
    let sum: u64 = checks
        .iter()
        .filter_map(|c| {
            let text = std::fs::read_to_string(&c.path).ok()?;
            let h = checks::parse_header_in(&text, Some(declared)).ok()?;
            h.setups
                .as_ref()
                .is_none_or(|s| s.iter().any(|x| x == setup))
                .then_some(h.timeout_s)
        })
        .sum();
    Duration::from_secs(sum) + CEILING_MARGIN
}

/// The reason of the check a stop ended before it reached a verdict (`DM-05`).
fn run_stopped(reason: &str, ceiling: Duration) -> Reason {
    let message = match reason {
        RUN_CEILING => format!(
            "The run reached its runner ceiling of {} s, the sum of its checks' timeouts plus 60 s, before this check reached a verdict. Raise the @timeout of a check that needs longer, or find what runs long, such as a cleanup.",
            ceiling.as_secs()
        ),
        SESSION_STOPPED => "The session was stopped before this check reached a verdict.".to_string(),
        _ => "The run was stopped by hand (`run stop`, `run_stop`, Ctrl+C or Ctrl+Break) before this check reached a verdict.".to_string(),
    };
    Reason {
        kind: "run-stopped".into(),
        message,
        machine: None,
        at: time::now(),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    runner: &Arc<Runner>,
    project: PathBuf,
    setups: Vec<String>,
    checks: Vec<CheckRef>,
    ids: Vec<String>,
    tags: Vec<String>,
    run_id: Option<String>,
    started: &(dyn Fn(String, usize) + Sync),
    emit: &(dyn Fn(Value, String) + Sync),
) -> Result<Value, CliError> {
    let Some(session) = runner.session() else {
        return Err(CliError::not_now(
            "No session runs. Start one with `shakedown session start --setup <name>`, or give `run` a --setup.",
        )
        .tag("no-session"));
    };
    if session.state() == SessionState::Broken {
        return Err(CliError::not_now(format!(
            "Session {} is broken ({}), so it cannot run checks. Bring every machine back with `shakedown reload`, which restarts its game, or stop it with `shakedown session stop`.",
            session.id,
            session.broken_reason()
        ))
        .tag("session-broken"));
    }
    let run_id = run_id.unwrap_or_else(|| new_run_id(runner));
    {
        // The progress is set under the same lock as the state, so a reload or a reset asked at
        // any moment of the run sees the run and its progress (`RLD-01`).
        let mut progress = runner.progress.lock().unwrap();
        if !session.transition(SessionState::Ready, SessionState::Running) {
            return Err(CliError::not_now(format!(
                "Session {} is {}, not ready, so it cannot run checks now. Wait for it, or stop it with `shakedown session stop`.",
                session.id,
                session_state_word(&session)
            ))
            .tag("session-busy"));
        }
        *progress = Some(progress_value(&run_id, 0, checks.len(), None));
    }
    started(run_id.clone(), checks.len());
    let control = control(runner, &run_id);
    let result = execute(
        runner, &session, project, &setups, checks, ids, tags, run_id, &control, emit,
    )
    .await;
    let mut progress = runner.progress.lock().unwrap();
    if session.state() == SessionState::Running {
        session.set_state(SessionState::Ready);
    }
    *progress = None;
    result
}

/// The active run's progress as `status` shows it (`INS-02`).
fn progress_value(run_id: &str, done: usize, total: usize, current: Option<&str>) -> Value {
    json!({"id": run_id, "state": "running", "done": done, "total": total, "current_check": current})
}

/// The refusal of a `reload` or a `reset` asked while a run is active, naming the run, its
/// progress and `run stop` (`RLD-01`, `US-058`); `None` when no run is active. A run is active
/// from its start to its result file, also while a check of it breaks the session.
pub fn refuse_during_run(runner: &Runner, session_id: &str, what: &str) -> Option<CliError> {
    let progress = runner.progress.lock().unwrap().clone()?;
    Some(busy_with_run(session_id, what, &progress))
}

fn busy_with_run(session_id: &str, what: &str, progress: &Value) -> CliError {
    let current = match progress["current_check"].as_str() {
        Some(c) => format!(", now at {c}"),
        None => String::new(),
    };
    CliError::not_now(format!(
        "Session {session_id} is running run {} ({} of {} checks done{current}), so it cannot {what} now. Wait for the run to end, or stop it with `shakedown run stop`.",
        progress["id"].as_str().unwrap_or("?"),
        progress["done"].as_u64().unwrap_or(0),
        progress["total"].as_u64().unwrap_or(0),
    ))
    .tag("session-busy")
}

fn session_state_word(session: &Session) -> String {
    match session.state() {
        SessionState::Starting => "starting".into(),
        SessionState::Ready => "ready".into(),
        SessionState::Running => "running a run".into(),
        SessionState::Reloading => format!("busy with {}", session.busy_with()),
        SessionState::Broken => "broken".into(),
        SessionState::Failed => "failed".into(),
        SessionState::Stopping => "stopping".into(),
        SessionState::Stopped => "stopped".into(),
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute(
    runner: &Arc<Runner>,
    session: &Arc<Session>,
    project: PathBuf,
    setups: &[String],
    checks: Vec<CheckRef>,
    ids: Vec<String>,
    tags: Vec<String>,
    run_id: String,
    control: &Control,
    emit: &(dyn Fn(Value, String) + Sync),
) -> Result<Value, CliError> {
    let started = SystemTime::now();
    let clock = Instant::now();
    let state_dir = paths::project_state_dir(&project)
        .map_err(|e| CliError::framework(format!("Cannot create .shakedown in {}: {e}.", project.display())))?;
    let folder = state_dir.join("runs").join(&run_id);
    std::fs::create_dir_all(&folder)
        .map_err(|e| CliError::framework(format!("Cannot create the run folder {}: {e}.", folder.display())))?;
    runner.log(&format!("{} run {run_id}: {} checks", session.id, checks.len()));
    // Every machine marks the run's start in its RPT, so the run's lines can be cut out (LOG-04).
    mark(session, &run_id, "start", "").await;

    let mut executions: Vec<CheckExecution> = Vec::new();
    let mut not_run: Vec<NotRun> = Vec::new();
    // How often each check has come up in this run: its executions' `repeat` (`RUN-03`).
    let mut times: BTreeMap<String, u32> = BTreeMap::new();
    let progress = |done: usize, current: Option<&str>| {
        *runner.progress.lock().unwrap() = Some(progress_value(&run_id, done, checks.len(), current));
    };
    let ceiling = run_ceiling(&checks, setups, &session.spec.setup.name);
    // Set at the start of the first check (RUN-06).
    let mut deadline: Option<tokio::time::Instant> = None;
    // Why the run stopped, once it has: a check ended by the stop, or checks left not run.
    let mut stopped: Option<&'static str> = None;
    for (n, check) in checks.iter().enumerate() {
        if deadline.is_some_and(|d| tokio::time::Instant::now() >= d) {
            control.request(RUN_CEILING);
        }
        if let Some(reason) = control.reason() {
            stopped = Some(reason);
            not_run.extend(checks[n..].iter().map(|c| NotRun {
                check_id: c.id.clone(),
                reason: "run-stopped".into(),
            }));
            break;
        }
        let deadline = *deadline.get_or_insert_with(|| tokio::time::Instant::now() + ceiling);
        progress(n, Some(&check.id));
        let repeat = {
            let n = times.entry(check.id.clone()).or_insert(0);
            *n += 1;
            *n
        };
        let execution = execute_check(
            runner,
            session,
            &run_id,
            &folder,
            executions.len() + 1,
            check,
            setups,
            control,
            (deadline, ceiling),
        )
        .await
        .map(|mut e| {
            e.repeat = repeat;
            e
        });
        let Some(execution) = execution else {
            not_run.push(NotRun {
                check_id: check.id.clone(),
                reason: "setup".into(),
            });
            continue;
        };
        if execution.reason.as_ref().is_some_and(|r| r.kind == "run-stopped") {
            stopped = control.reason();
        }
        let line = outcome_line(&execution, executions.len());
        emit(line, outcome_text(&execution));
        executions.push(execution);
    }
    if let Some(reason) = stopped {
        runner.log(&format!("{} run {run_id}: stopped, {reason}", session.id));
    }

    progress(checks.len(), None);
    mark(session, &run_id, "end", "").await;
    let mut run_logs = BTreeMap::new();
    for machine in session.machine_names() {
        let rel = format!("logs/{machine}.log");
        if let Some((_, rel)) = excerpt(runner, session, &machine, &run_id, &folder, &rel).await {
            run_logs.insert(machine, rel);
        }
    }
    let summary = Summary::of(&executions, not_run.len());
    for machine in session.machine_names() {
        if session.is_lost(&machine) {
            session.settled(&machine).await;
        }
    }
    let mut machines = session.result_machines(started);
    for m in machines.iter_mut() {
        if let Some(lost) = m.lost.as_mut() {
            let found: Vec<PathBuf> = lost.dumps.iter().map(PathBuf::from).collect();
            lost.dumps = copy_dumps(&folder, &m.name, &found);
        }
    }
    let result = ResultFile {
        schema: SCHEMA.into(),
        framework_version: shakedown_protocol::VERSION.into(),
        run: Run {
            id: run_id.clone(),
            state: if stopped.is_some() { "stopped" } else { "finished" }.into(),
            started_at: time::format_system_time(started),
            ended_at: time::now(),
            duration_s: round3(clock.elapsed().as_secs_f64()),
            selection: Selection { ids, tags },
            stopped_reason: stopped.map(String::from),
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
            machines,
        },
        summary: summary.clone(),
        checks: executions.clone(),
        not_run,
        reloads: vec![],
        logs: run_logs,
    };
    let file = folder.join("result.json");
    // Nothing in the result file names the Steam account (LOG-02).
    let mut value = serde_json::to_value(&result).expect("the result serialises");
    runner.redactor().redact_value(&mut value);
    let text = serde_json::to_string_pretty(&value).expect("the result serialises");
    std::fs::write(&file, text)
        .map_err(|e| CliError::framework(format!("Cannot write the result file {}: {e}.", file.display())))?;
    runner.log(&format!("{} run {run_id}: done, {}", session.id, file.display()));
    *control.result.lock().unwrap() = Some(file.clone());

    Ok(run_result_of(&value, &file))
}

/// The `run_result` shape of `contract/mcp-tools.md` from a result file's content, so the command
/// line's `run --json` and the `run_result` tool give the same JSON (`NFR-US-03`).
pub fn run_result_of(result: &Value, file: &Path) -> Value {
    let outcomes: Vec<Value> = result["checks"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, c)| {
            json!({
                "check_id": c["check_id"],
                "repeat": c["repeat"],
                "outcome": c["outcome"],
                "reason": if c["reason"].is_null() { Value::Null } else { json!({"kind": c["reason"]["kind"], "message": c["reason"]["message"]}) },
                "duration_s": c["duration_s"],
                "failed_step": c["evidence"]["failed_step"],
                "evidence_path": format!("result.json#/checks/{i}/evidence"),
            })
        })
        .collect();
    json!({
        "run_id": result["run"]["id"],
        "state": result["run"]["state"],
        "summary": result["summary"],
        "outcomes": outcomes,
        "result_file": file,
    })
}

/// Executes one check (`P-05`) as the run's `n`th execution; `None` when its `@setups` excludes
/// the session's setup. Every machine marks the execution's start before its reset and its end
/// after the body and the wait for errors in flight; the lines between, each machine's script
/// errors and the `expectError` blocks decide the outcome with the lead's report (`ERR-01` to
/// `ERR-04`, `LOG-04`). A stop of the run, or its runner ceiling (`deadline`, `ceiling`), that
/// comes during the reset or the body ends the check `invalid` / `run-stopped` and stops its body
/// in the game (`RUN-05`, `RUN-06`); its logs are cut out as for any other check.
#[allow(clippy::too_many_arguments)]
async fn execute_check(
    runner: &Runner,
    session: &Arc<Session>,
    run_id: &str,
    folder: &Path,
    n: usize,
    check: &CheckRef,
    setups: &[String],
    control: &Control,
    (deadline, ceiling): (tokio::time::Instant, Duration),
) -> Option<CheckExecution> {
    let started = SystemTime::now();
    let clock = Instant::now();
    // The check is read fresh from disk for every execution (RLD-04).
    let text = std::fs::read_to_string(&check.path);
    let header = text
        .as_ref()
        .map_err(|e| format!("cannot read the check file: {e}"))
        .and_then(|t| checks::parse_header_in(t, Some(setups)));
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
    // P-05 step 2: no check runs in a broken session.
    if session.state() == SessionState::Broken {
        let reason = Reason {
            kind: "session-broken".into(),
            message: format!(
                "The session was broken before this check started: {}. Bring every machine back with `shakedown reload`, which restarts the game of a broken session, or stop it with `shakedown session stop`.",
                session.broken_reason()
            ),
            machine: None,
            at: time::now(),
        };
        return Some(ended(
            check,
            Some(&header),
            started,
            clock,
            "invalid",
            Some(reason),
            vec![],
            Evidence::default(),
        ));
    }
    // P-05 step 3 for reset mode `mission` (RST-06): the mission restart comes before the start
    // marker and the context, which a mission restart clears.
    if header.reset == "mission"
        && let Err((machine, message)) = session.mission_restart(runner, &|_| {}).await
    {
        session.release();
        session.set_broken(
            &machine,
            format!(
                "the mission restart before check {} did not bring {machine} back: {message}",
                check.id
            ),
        );
        let reason = Reason {
            kind: "reset-failed".into(),
            message: format!(
                "The mission restart before the check did not bring {machine} back: {message}. The session is broken; restart the game with `shakedown reload`, or stop it with `shakedown session stop`."
            ),
            machine: Some(machine),
            at: time::now(),
        };
        return Some(ended(
            check,
            Some(&header),
            started,
            clock,
            "invalid",
            Some(reason),
            vec![],
            Evidence::default(),
        ));
    }
    let key = format!("{run_id}#{n}");
    mark(session, &key, "start", &check.id).await;
    *runner.executing.lock().unwrap() = Some((key.clone(), check.id.clone()));
    // A machine lost during the check ends it at once (P-07 step 2, P-09 step 3); the lead's
    // report still counts for a cause that came first if it comes within a second (DM-06). The
    // body's order is dropped with `work`, which stops the body in the game.
    let raced = {
        let work = prepare_and_execute(runner, session, run_id, check, &header, &text, started, clock);
        tokio::pin!(work);
        tokio::select! {
            done = &mut work => Ok(done),
            reason = stop_or_ceiling(control, deadline) => Err(Some(reason)),
            _ = session.wait_lost(clock) => {
                match tokio::time::timeout(Duration::from_secs(1), &mut work).await {
                    Ok(done) => Ok(done),
                    Err(_) => Err(None),
                }
            }
        }
    };
    *session.check.lock().unwrap() = None;
    let (mut execution, report, executing_at) = match raced {
        Ok(done) => done,
        Err(reason) => {
            let reason = match reason {
                Some(reason) => run_stopped(reason, ceiling),
                // Replaced by the lost machine's own reason below.
                None => Reason {
                    kind: "machine-lost".into(),
                    message: "A machine was lost during the check.".into(),
                    machine: None,
                    at: time::now(),
                },
            };
            let e = ended(
                check,
                Some(&header),
                started,
                clock,
                "invalid",
                Some(reason),
                vec![],
                Evidence::default(),
            );
            (e, Value::Null, None)
        }
    };
    // P-05 step 9: script errors still in flight.
    tokio::time::sleep(ERRORS_IN_FLIGHT).await;
    mark(session, &key, "end", &check.id).await;
    *runner.executing.lock().unwrap() = None;
    let mut errors: Vec<Value> = session
        .take_errors(&key)
        .iter()
        .map(|e| errors::from_handler(e, &check.id))
        .collect();
    for machine in session.machine_names() {
        let rel = format!("checks/{n}/{machine}.log");
        let Some((lines, rel)) = excerpt(runner, session, &machine, &key, folder, &rel).await else {
            continue;
        };
        execution.evidence.logs.insert(machine.clone(), rel);
        errors::from_rpt(&mut errors, &machine, &check.id, &lines);
    }
    errors::judge(&mut execution, errors, &report["expects"], executing_at.as_deref());
    let mut causes = Vec::new();
    if let Some((machine, _)) = session.lost_since(clock)
        && let Some(lost) = session.settled(&machine).await
    {
        causes.push(lost_reason(folder, &machine, &lost));
        let tail = session
            .profile(&machine)
            .map(|p| super::session::rpt_tail(&p, 20))
            .unwrap_or_default();
        execution.evidence.attached.push(Attached {
            label: format!("the last 20 lines of {machine}'s RPT"),
            value: json!(tail),
        });
    }
    if let Some(flag) = session.flag_since(&execution.started_at) {
        causes.push(unhealthy_reason(&flag));
    }
    decide(&mut execution, causes);
    execution.ended_at = time::now();
    execution.duration_s = round3(clock.elapsed().as_secs_f64());
    Some(execution)
}

/// The reason of a check a lost machine ended (`DM-05`, `P-07`, `P-09`): the machine, how it was
/// lost, its exit code and its crash dumps, copied into the run folder.
fn lost_reason(folder: &Path, machine: &str, lost: &super::session::LostRecord) -> Reason {
    let dumps = copy_dumps(folder, machine, &lost.dumps);
    let dumps = match (lost.reason, dumps.is_empty()) {
        ("exited", true) => "; it left no crash dump in its profile folder".to_string(),
        ("exited", false) => format!("; its crash dumps: {}", dumps.join(", ")),
        _ => String::new(),
    };
    let next = if lost.reason == "exited" {
        "The environment failed, not the mod; the session is broken: restart its game with `shakedown reload`, or stop it with `shakedown session stop`."
    } else {
        "The environment failed, not the mod; its process still runs for inspection, and the session is broken: restart its game with `shakedown reload`, or stop it with `shakedown session stop`."
    };
    Reason {
        kind: "machine-lost".into(),
        message: format!("{}{dumps}. {next}", lost.message),
        machine: Some(machine.into()),
        at: lost.at.clone(),
    }
}

/// The reason of a check that overlapped an interval the clock watch flagged (`HLT-02`).
fn unhealthy_reason(flag: &super::session::FlagInterval) -> Reason {
    let what = if flag.clock == "frame" {
        "frame number"
    } else {
        "mission time"
    };
    Reason {
        kind: "unhealthy".into(),
        message: format!(
            "{}'s {what} stopped at {} while it still answered: over {:.1} s of real time its mission time advanced {:.2} s. The environment failed, not the mod; the check runs again once the machine's clocks move.",
            flag.machine, flag.from, flag.advanced.1, flag.advanced.0
        ),
        machine: Some(flag.machine.clone()),
        at: flag.from.clone(),
    }
}

/// `DM-06`: of the causes of `invalid` the runner saw and the cause the check ended with, the
/// first in wall-clock time decides. A check that passed, or ended for a cause that came later,
/// takes the first of the runner's causes; a `machine-lost` the check saw itself gives way to
/// the runner's own record of that loss, which names the exit code and the dumps.
fn decide(execution: &mut CheckExecution, causes: Vec<Reason>) {
    let Some(first) = causes.into_iter().min_by(|a, b| a.at.cmp(&b.at)) else {
        return;
    };
    let keep = execution.outcome != "pass"
        && execution
            .reason
            .as_ref()
            .is_some_and(|r| r.at < first.at && r.kind != "machine-lost");
    if !keep {
        execution.outcome = "invalid".into();
        execution.reason = Some(first);
    }
}

/// Copies a lost machine's crash dumps into the run folder as `dumps/<machine>/<file>`, once,
/// and returns their paths relative to it (`P-09` step 2, `RES-04`).
fn copy_dumps(folder: &Path, machine: &str, dumps: &[PathBuf]) -> Vec<String> {
    dumps
        .iter()
        .filter_map(|d| {
            let rel = format!("dumps/{machine}/{}", d.file_name()?.to_string_lossy());
            let to = folder.join(&rel);
            if !to.is_file() {
                std::fs::create_dir_all(to.parent()?).ok()?;
                std::fs::copy(d, &to).ok()?;
            }
            Some(rel)
        })
        .collect()
}

/// Returns the reason once the run is asked to stop, or `run-ceiling` once `deadline` passes.
async fn stop_or_ceiling(control: &Control, deadline: tokio::time::Instant) -> &'static str {
    tokio::select! {
        reason = control.stopped() => reason,
        _ = tokio::time::sleep_until(deadline) => {
            control.request(RUN_CEILING);
            control.reason().unwrap_or(RUN_CEILING)
        }
    }
}

/// How long the runner waits after a check's end report for script errors still in flight
/// (`P-05` step 9).
const ERRORS_IN_FLIGHT: Duration = Duration::from_secs(1);
/// How long the runner waits for a marker to reach a machine's RPT, which the engine writes about
/// every 0.5 s (`SPK-13`).
const MARKER_CEILING: Duration = Duration::from_secs(3);

/// Has every machine write the marker of `key` and `edge` into its RPT (`LOG-04`), and, for a
/// check execution, set or clear the context that stamps its script errors (`ERR-01`).
async fn mark(session: &Arc<Session>, key: &str, edge: &str, check_id: &str) {
    let mut handles = Vec::new();
    for machine in session.machine_names() {
        if session.is_lost(&machine) {
            continue;
        }
        let session = Arc::clone(session);
        let args = json!([key, edge, check_id]);
        handles.push(tokio::spawn(async move {
            let _ = session.exec(&machine, "_this call skd_fnc_context", args, 5.0).await;
        }));
    }
    for h in handles {
        let _ = h.await;
    }
}

/// Cuts a machine's lines between the markers of `key` out of its RPT, once the end marker has
/// reached the file, writes them redacted to `rel` in the run folder, and returns them with `rel`.
async fn excerpt(
    runner: &Runner,
    session: &Session,
    machine: &str,
    key: &str,
    folder: &Path,
    rel: &str,
) -> Option<(String, String)> {
    let rpt = crate::logs::newest_rpt(&session.profile(machine)?)?;
    let end = crate::logs::marker(key, "end");
    let mut text = crate::logs::read_text(&rpt);
    for _ in 0..(MARKER_CEILING.as_millis() / 100) {
        // A lost machine writes no end marker; its lines up to its last are kept (US-043).
        if text.contains(&end) || session.is_lost(machine) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        text = crate::logs::read_text(&rpt);
    }
    let lines = runner.redactor().redact(&crate::logs::cut(&text, key)?);
    let path = folder.join(rel);
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, &lines).ok()?;
    Some((lines, rel.to_string()))
}

/// The reset before a check and the check's body on the lead (`P-05` steps 3 to 8): the
/// execution, the lead's end report (`null` when there is none) and the wall-clock time the body
/// was sent.
#[allow(clippy::too_many_arguments)]
async fn prepare_and_execute(
    runner: &Runner,
    session: &Arc<Session>,
    run_id: &str,
    check: &CheckRef,
    header: &Header,
    text: &Result<String, std::io::Error>,
    started: SystemTime,
    clock: Instant,
) -> (CheckExecution, Value, Option<String>) {
    let header = header.clone();
    let lead = session.lead();
    // P-05 step 3: the reset in place before the check (P-06, RST-01); a check with reset mode
    // `mission` had its mission restart before its start marker.
    if header.reset != "mission"
        && let Err(message) = reset(runner, session).await
    {
        session.set_broken(lead, format!("the reset before check {} failed: {message}", check.id));
        let reason = Reason {
            kind: "reset-failed".into(),
            message: format!("The reset before the check failed: {message}."),
            machine: Some(lead.into()),
            at: time::now(),
        };
        let e = ended(
            check,
            Some(&header),
            started,
            clock,
            "invalid",
            Some(reason),
            vec![],
            Evidence::default(),
        );
        return (e, Value::Null, None);
    }
    let setup = &session.spec.setup;
    let order = json!({
        "kind": "check",
        "run_id": run_id,
        "check_id": check.id,
        "source": checks::body_source(text.as_ref().map(String::as_str).unwrap_or_default()),
        "timeout": header.timeout_s,
        "machines": session.machine_names(),
        "mode": setup.mode.as_str(),
        "setup": {
            "name": setup.name,
            "mode": setup.mode.as_str(),
            "clients": setup.clients,
            "headless": setup.headless,
            "world": setup.world,
        },
    });
    let ceiling = Duration::from_secs(header.timeout_s) + GRACE;
    let executing_at = Some(time::now());
    // The check's code, and every order its code routes, go into the audit under its id (SEC-04).
    session.audit(&check.id, lead, order["source"].as_str().unwrap_or_default());
    *session.check.lock().unwrap() = Some(check.id.clone());
    // Dropped by a stop of the run before the end report, the order stops the body in the game.
    let answer = session.order_tracked(lead, order, ceiling).await;
    *session.check.lock().unwrap() = None;
    match answer {
        Ok(report) => {
            let e = from_report(check, &header, started, clock, &report, lead);
            (e, report, executing_at)
        }
        Err(e) => {
            let why = match &e {
                OrderError::NotConnected | OrderError::Lost => {
                    format!("{lead} lost its connection to the runner during the check")
                }
                OrderError::Ceiling => format!(
                    "{lead} sent no end report within the check's timeout of {} s plus the grace of {} s",
                    header.timeout_s,
                    GRACE.as_secs()
                ),
            };
            if matches!(e, OrderError::Ceiling) {
                session.lose(
                    lead,
                    "no-answer",
                    None,
                    format!("{why}; it was asked to run check {}", check.id),
                );
            } else if !session.is_lost(lead) {
                session.set_broken(lead, why.clone());
            }
            let reason = Reason {
                kind: "machine-lost".into(),
                message: format!("{why}."),
                machine: Some(lead.into()),
                at: time::now(),
            };
            let e = ended(
                check,
                Some(&header),
                started,
                clock,
                "invalid",
                Some(reason),
                vec![],
                Evidence::default(),
            );
            (e, Value::Null, executing_at)
        }
    }
}

/// The ceiling of a reset (`RST-04`).
pub const RESET_CEILING_S: f64 = 15.0;

/// Resets the session in place (`P-06`): the cleanups on the lead, then the phases `free`,
/// `restore` and `verify` on every machine at once, each phase waiting for every machine. Returns
/// how long it took, or what did not match when it failed or a machine did not answer.
pub async fn reset(runner: &Runner, session: &Arc<Session>) -> Result<f64, String> {
    let clock = Instant::now();
    let lead = session.lead();
    let left = || (RESET_CEILING_S - clock.elapsed().as_secs_f64()).max(0.1);
    let world = session.baseline(lead);
    let names = session.machine_names();
    let players: Vec<Value> = names
        .iter()
        .filter(|n| n.starts_with('p'))
        .filter_map(|n| {
            let b = session.baseline(n);
            (!b["position"].is_null()).then(|| json!([n, b["position"], b["direction"]]))
        })
        .collect();
    let phase = |name: &'static str, on: Vec<String>, args: Value| {
        let session = Arc::clone(session);
        async move {
            let mut handles = Vec::new();
            for machine in on {
                let session = Arc::clone(&session);
                let args = args.clone();
                // The order's own ceiling is a second longer, so the phase reports what did not match.
                let ceiling = args[0].as_f64().unwrap_or(RESET_CEILING_S) + 1.0;
                handles.push(tokio::spawn(async move {
                    let answer = session
                        .exec(&machine, "_this call skd_fnc_reset", json!([name, args]), ceiling)
                        .await;
                    match answer {
                        Ok(a) if a["state"] == "done" => a["value"]
                            .as_array()
                            .map(|m| m.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                        Ok(a) => vec![format!(
                            "{machine} did not finish the reset's {name} within its ceiling ({})",
                            a["state"]
                        )],
                        Err(e) => vec![format!("{machine} did not answer the reset's {name} ({e:?})")],
                    }
                }));
            }
            let mut out: Vec<String> = Vec::new();
            for h in handles {
                out.extend(
                    h.await
                        .unwrap_or_else(|e| vec![format!("the reset's {name} failed: {e}")]),
                );
            }
            out
        }
    };
    let mut mismatches = phase("cleanups", vec![lead.to_string()], json!([left()])).await;
    if mismatches.is_empty() {
        mismatches = phase("free", names.clone(), json!([left()])).await;
    }
    if mismatches.is_empty() {
        mismatches = phase("restore", names.clone(), json!([left(), world.clone()])).await;
    }
    if mismatches.is_empty() {
        mismatches = phase("verify", names.clone(), json!([left(), world, players])).await;
    }
    let duration = clock.elapsed().as_secs_f64();
    if mismatches.is_empty() {
        runner.log(&format!("{} reset done in {duration:.3} s", session.id));
        Ok(duration)
    } else {
        runner.log(&format!("{} reset failed: {}", session.id, mismatches.join("; ")));
        Err(mismatches.join("; "))
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
    let repeat = if e.repeat > 1 {
        format!(" #{}", e.repeat)
    } else {
        String::new()
    };
    let mut text = format!("{:<7} {}{repeat}  ({:.1} s)", e.outcome, e.check_id, e.duration_s);
    if let Some(r) = &e.reason {
        text.push_str(&format!("\n        {}: {}", r.kind, r.message));
    }
    text
}

/// The most characters of lines `logs` returns itself, so its JSON stays under the 20,000 of a
/// tool result; more go to a file (`AGT-07`).
const LOGS_LIMIT: usize = 18_000;

/// `logs` (`LOG-03`): a machine's log lines, from its running game's RPT, from a run's folder
/// with `run`, or for one check execution with `check` (the one running now, or else its last
/// execution in the newest run that has one), filtered by a regular expression and cut to the
/// last `last` lines, redacted. The shape is that of the `logs` tool: `lines`, `truncated` when
/// more lines matched than are returned, and `path`, the file holding them all when they are
/// longer than a tool result may be.
pub fn logs(
    runner: &Runner,
    project: &Path,
    machine: &str,
    run: Option<&str>,
    check: Option<&str>,
    grep: Option<&str>,
    last: usize,
) -> Result<Value, CliError> {
    let filter = grep
        .map(regex::Regex::new)
        .transpose()
        .map_err(|e| CliError::wrong(format!("--grep is not a regular expression: {e}. Fix the pattern.")))?;
    let runs = project.join(".shakedown").join("runs");
    let session = runner.session();
    let live = |key: Option<&str>| -> Result<String, CliError> {
        let Some(session) = &session else {
            return Err(CliError::not_now(
                "No session runs, so there is no live log to read. Give --run <id> to read a run's logs, or start a session.",
            )
            .tag("no-session"));
        };
        let Some(profile) = session.profile(machine) else {
            return Err(CliError::wrong(format!(
                "Session {} has no machine {machine}; its machines are {}. Use one of those.",
                session.id,
                session.machine_names().join(", ")
            ))
            .tag("unknown-machine"));
        };
        let text = crate::logs::newest_rpt(&profile)
            .map(|p| crate::logs::read_text(&p))
            .unwrap_or_default();
        Ok(match key {
            Some(key) => crate::logs::cut(&text, key).unwrap_or_default(),
            None => text,
        })
    };
    // The excerpt a run's result file names for the machine: the whole run's, or that of the
    // check's last execution in it.
    let from_run = |dir: &Path, check: Option<&str>| -> Option<String> {
        let result: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("result.json")).ok()?).ok()?;
        let rel = match check {
            None => result["logs"][machine].as_str()?.to_string(),
            Some(id) => result["checks"]
                .as_array()?
                .iter()
                .rev()
                .find(|c| c["check_id"] == id)?["evidence"]["logs"][machine]
                .as_str()?
                .to_string(),
        };
        std::fs::read_to_string(dir.join(rel)).ok()
    };
    let text = match (run, check) {
        (Some(run), check) => {
            let dir = runs.join(run);
            if !dir.join("result.json").is_file() {
                return Err(CliError::wrong(format!(
                    "No run {run} with a result file is in {}. Give the id of a run that ended.",
                    runs.display()
                ))
                .tag("unknown-run"));
            }
            from_run(&dir, check).ok_or_else(|| {
                CliError::wrong(format!(
                    "Run {run} keeps no log of {machine}{}. Check the machine name and the check id.",
                    check.map(|c| format!(" for check {c}")).unwrap_or_default()
                ))
            })?
        }
        (None, Some(check)) => {
            let now = runner.executing.lock().unwrap().clone();
            match now {
                Some((key, id)) if id == check => live(Some(&key))?,
                _ => {
                    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&runs)
                        .into_iter()
                        .flatten()
                        .flatten()
                        .map(|e| e.path())
                        .collect();
                    dirs.sort_by_key(|d| std::fs::metadata(d.join("result.json")).and_then(|m| m.modified()).ok());
                    dirs.iter()
                        .rev()
                        .find_map(|d| from_run(d, Some(check)))
                        .ok_or_else(|| {
                            CliError::wrong(format!(
                                "No run in {} executed check {check} with a log of {machine}. Run the check first.",
                                runs.display()
                            ))
                        })?
                }
            }
        }
        (None, None) => live(None)?,
    };
    let text = runner.redactor().redact(&text);
    let matched: Vec<&str> = text
        .lines()
        .filter(|l| filter.as_ref().is_none_or(|f| f.is_match(l)))
        .collect();
    let mut lines: Vec<String> = matched[matched.len().saturating_sub(last)..]
        .iter()
        .map(|l| l.to_string())
        .collect();
    let mut truncated = matched.len() > lines.len();
    let mut path = Value::Null;
    // Each line's length as JSON, which escapes the backslashes of paths.
    let json_len = |l: &String| serde_json::to_string(l).map_or(l.len(), |j| j.len()) + 1;
    if lines.iter().map(json_len).sum::<usize>() > LOGS_LIMIT {
        let dir = session.as_ref().map_or_else(|| runs.clone(), |s| s.folder.clone());
        let file = dir.join(format!("logs-{machine}-{}.log", time::stamp(SystemTime::now())));
        if std::fs::write(&file, lines.join("\n") + "\n").is_ok() {
            path = json!(file);
        }
        let mut size = 0;
        let keep = lines
            .iter()
            .rev()
            .take_while(|l| {
                size += json_len(l);
                size <= LOGS_LIMIT
            })
            .count();
        lines.drain(..lines.len() - keep);
        truncated = true;
    }
    Ok(json!({"lines": lines, "truncated": truncated, "path": path}))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `US-058` 1, `RLD-01`: a reload asked during a run is refused with exit code 4 and the MCP
    /// code `session-busy`, naming the run, its progress and `run stop`.
    #[test]
    fn us_058_rld_01_a_reload_during_a_run_names_the_run_its_progress_and_run_stop() {
        let progress = progress_value("r-20261111-120000-3", 2, 5, Some("slow"));
        let e = busy_with_run("s-1", "reload", &progress);
        assert_eq!((e.code, e.mcp_code()), (4, Some("session-busy")));
        for part in [
            "s-1",
            "r-20261111-120000-3",
            "2 of 5 checks done",
            "now at slow",
            "cannot reload",
            "`shakedown run stop`",
        ] {
            assert!(e.message.contains(part), "{part}: {}", e.message);
        }
        let e = busy_with_run("s-1", "be reset", &progress_value("r-1", 5, 5, None));
        assert!(
            e.message.contains("5 of 5 checks done)") && e.message.contains("cannot be reset"),
            "{}",
            e.message
        );
    }

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
                    lost: None,
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

    /// `RUN-06`, `US-063` 1: eight checks of `@timeout 1` have a runner ceiling of 68 s; a check
    /// whose `@setups` leaves out the session's setup adds nothing.
    #[test]
    fn us_063_run_06_the_ceiling_is_the_sum_of_the_timeouts_plus_60_s() {
        let dir = std::env::temp_dir().join(format!("skd-run06-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut refs = Vec::new();
        for i in 0..8 {
            let path = dir.join(format!("c{i}.sqf"));
            std::fs::write(
                &path,
                "// @check US-063 one second
// @timeout 1
",
            )
            .unwrap();
            refs.push(CheckRef {
                id: format!("c{i}"),
                path,
            });
        }
        let other = dir.join("other.sqf");
        std::fs::write(
            &other,
            "// @check US-063 elsewhere
// @setups ded
// @timeout 500
",
        )
        .unwrap();
        refs.push(CheckRef {
            id: "other".into(),
            path: other,
        });
        let declared = vec!["sp".to_string(), "ded".to_string()];
        assert_eq!(run_ceiling(&refs, &declared, "sp"), Duration::from_secs(68));
        assert_eq!(run_ceiling(&refs, &declared, "ded"), Duration::from_secs(568));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `RUN-05`, `RUN-06`: the first reason a run is asked to stop for is the one it keeps.
    #[tokio::test]
    async fn run_05_the_first_stop_reason_wins() {
        let c = Control::new();
        assert_eq!(c.reason(), None);
        c.request(RUN_CEILING);
        c.request(BY_HAND);
        assert_eq!(c.stopped().await, RUN_CEILING);
        let c = Control::new();
        let deadline = tokio::time::Instant::now() + Duration::from_millis(50);
        assert_eq!(stop_or_ceiling(&c, deadline).await, RUN_CEILING);
        assert_eq!(c.reason(), Some(RUN_CEILING));
    }

    /// `US-034` 1, `US-063` 1, `RES-01`: a stopped run's result, its check `invalid` /
    /// `run-stopped` and the rest not run, is valid against the schema.
    #[test]
    fn us_034_us_063_a_stopped_result_is_valid_against_the_schema() {
        let check = CheckRef {
            id: "second".into(),
            path: PathBuf::from("second.sqf"),
        };
        for reason in [BY_HAND, RUN_CEILING, SESSION_STOPPED] {
            let e = ended(
                &check,
                None,
                SystemTime::now(),
                Instant::now(),
                "invalid",
                Some(run_stopped(reason, Duration::from_secs(68))),
                vec![],
                Evidence::default(),
            );
            let mut r = sample("pass");
            r.checks.push(e);
            r.not_run = vec![NotRun {
                check_id: "third".into(),
                reason: "run-stopped".into(),
            }];
            r.summary = Summary::of(&r.checks, 1);
            r.run.state = "stopped".into();
            r.run.stopped_reason = Some(reason.into());
            assert_eq!((r.summary.total, r.summary.invalid, r.summary.not_run), (3, 1, 1));
            validate(&r);
        }
        assert!(
            run_stopped(RUN_CEILING, Duration::from_secs(68))
                .message
                .contains("68 s")
        );
    }

    fn reason(kind: &str, at: &str) -> Reason {
        Reason {
            kind: kind.into(),
            message: format!("{kind} at {at}"),
            machine: Some("p2".into()),
            at: at.into(),
        }
    }

    /// `DM-06`, `US-041` 3, `US-059` 3: the first cause in wall-clock time decides. A failed
    /// assertion before the loss or the stopped clock stays `fail`; one after it does not hide
    /// it; a check that passed takes the runner's cause; a `machine-lost` the check saw itself
    /// gives way to the runner's record of the loss.
    #[test]
    fn us_041_us_059_dm_06_the_first_cause_decides() {
        let mut e = sample("fail").checks.remove(0);
        e.reason = Some(reason("assertion", "2026-11-12T12:00:01.000Z"));
        decide(&mut e, vec![reason("machine-lost", "2026-11-12T12:00:01.500Z")]);
        assert_eq!(
            (e.outcome.as_str(), e.reason.as_ref().unwrap().kind.as_str()),
            ("fail", "assertion")
        );

        decide(&mut e, vec![reason("unhealthy", "2026-11-12T12:00:00.500Z")]);
        assert_eq!(
            (e.outcome.as_str(), e.reason.as_ref().unwrap().kind.as_str()),
            ("invalid", "unhealthy")
        );

        let mut e = sample("pass").checks.remove(0);
        decide(
            &mut e,
            vec![
                reason("unhealthy", "2026-11-12T12:00:03.000Z"),
                reason("machine-lost", "2026-11-12T12:00:02.000Z"),
            ],
        );
        assert_eq!(
            (e.outcome.as_str(), e.reason.as_ref().unwrap().kind.as_str()),
            ("invalid", "machine-lost")
        );

        let mut e = sample("fail").checks.remove(0);
        e.outcome = "invalid".into();
        e.reason = Some(reason("machine-lost", "2026-11-12T12:00:01.000Z"));
        let mut runner_cause = reason("machine-lost", "2026-11-12T12:00:02.000Z");
        runner_cause.message = "p2's process exited unasked with exit code 1".into();
        decide(&mut e, vec![runner_cause]);
        assert!(e.reason.as_ref().unwrap().message.contains("exit code 1"));

        let mut e = sample("pass").checks.remove(0);
        decide(&mut e, vec![]);
        assert_eq!(e.outcome, "pass");
    }

    /// `US-016` 2, `US-043` 2, `RES-05`: a lost machine's dumps are copied into the run folder as
    /// `dumps/<machine>/<file>`, its reason names them and its exit code, and a result with the
    /// machine `lost` is valid against the schema.
    #[test]
    fn us_016_us_043_res_05_a_lost_machine_is_in_the_result_with_its_dumps() {
        let dir = std::env::temp_dir().join(format!("skd-m09-dumps-{}", std::process::id()));
        let profile = dir.join("profile");
        std::fs::create_dir_all(&profile).unwrap();
        let dump = profile.join("arma3_x64_2026-11-12_10-54-04.mdmp");
        std::fs::write(&dump, b"MDMP").unwrap();
        let run = dir.join("run");
        let lost = super::super::session::LostRecord {
            reason: "exited",
            at: "2026-11-12T12:00:02.000Z".into(),
            instant: Instant::now(),
            exit_code: Some(3221225477),
            message: "p2's process 4242 exited unasked with exit code 3221225477 (0xC0000005)".into(),
            dumps: vec![dump.clone()],
            searching: false,
        };
        let r = lost_reason(&run, "p2", &lost);
        assert_eq!(r.kind, "machine-lost");
        assert_eq!(r.machine.as_deref(), Some("p2"));
        assert!(
            r.message.contains("0xC0000005") && r.message.contains("dumps/p2/arma3_x64_2026-11-12_10-54-04.mdmp"),
            "{}",
            r.message
        );
        assert!(run.join("dumps/p2/arma3_x64_2026-11-12_10-54-04.mdmp").is_file());
        let mut result = sample("pass");
        result.session.machines[0].end_state = "lost".into();
        result.session.machines[0].lost = Some(shakedown_protocol::result::Lost {
            reason: "exited".into(),
            at: lost.at.clone(),
            exit_code: lost.exit_code,
            dumps: copy_dumps(&run, "p2", &lost.dumps),
        });
        validate(&result);
        let _ = std::fs::remove_dir_all(&dir);
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
