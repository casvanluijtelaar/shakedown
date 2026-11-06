//! Asking a running session from outside a check: `exec` (`CTL-01`, `CTL-05`), `wait`
//! (`OBS-01`), `expect` (`OBS-06`), `inspect` (`INS-01`) and `errors` (`ERR-05`). Each sends an
//! order of kind `exec` and, when its client goes away before the answer, tells the machine to
//! stop the code (`AGT-04`).

use std::sync::Arc;
use std::time::Instant;

use serde_json::{Value, json};
use tokio::task::JoinSet;

use super::Runner;
use super::errors;
use super::session::{Session, SessionState};
use crate::config::Mode;
use crate::error::CliError;

/// The longest ceiling the command line takes (`contract/cli.md`, `exec`).
pub const CEILING_MAX_S: f64 = 600.0;
/// The ceiling of `inspect`'s order (`INS-01`).
const INSPECT_CEILING_S: f64 = 10.0;
/// The most script errors `inspect` lists (`INS-01`).
const INSPECT_ERRORS: usize = 10;
/// The operators of `skd_fnc_expectOn` (`contract/check-api.md`).
pub const OPS: [&str; 9] = ["==", "!=", "<", "<=", ">", ">=", "~=", "in", "contains"];

/// The running session, ready to be asked; a refusal with the next step otherwise. The client adds
/// the setups the project declares to a `no-session` refusal (`US-047`).
fn session(runner: &Runner) -> Result<Arc<Session>, CliError> {
    let Some(session) = runner.session() else {
        return Err(
            CliError::not_now("No session runs. Start one with `shakedown session start --setup <name>`.")
                .tag("no-session"),
        );
    };
    match session.state() {
        SessionState::Starting | SessionState::Stopping => Err(CliError::not_now(format!(
            "Session {} is {}, so its machines cannot be asked now. Wait for it to be ready, and follow it with `shakedown status`.",
            session.id,
            session.state().as_str()
        ))
        .tag("session-busy")),
        _ => Ok(session),
    }
}

/// The machine names a `machine` argument names (`DM-10`): one name, or with `many` also a group,
/// `all`, `clients` or `headless`. In singleplayer `server` names `p1`.
fn targets(session: &Session, machine: &str, many: bool) -> Result<Vec<String>, CliError> {
    let names = session.machine_names();
    let single = session.spec.setup.mode == Mode::Singleplayer;
    let group: Option<Vec<String>> = match machine {
        "all" => Some(names.clone()),
        "clients" => Some(names.iter().filter(|n| n.starts_with('p')).cloned().collect()),
        "headless" => Some(names.iter().filter(|n| n.starts_with("hc")).cloned().collect()),
        _ => None,
    };
    if let Some(group) = group {
        if !many {
            return Err(CliError::wrong(format!(
                "`{machine}` is a group; this command takes one machine. Name one of {}.",
                names.join(", ")
            ))
            .tag("bad-arguments"));
        }
        if group.is_empty() {
            return Err(CliError::wrong(format!(
                "Session {} has no machine in the group {machine}; its machines are {}. Name one of those.",
                session.id,
                names.join(", ")
            ))
            .tag("unknown-machine"));
        }
        return Ok(group);
    }
    let name = if single && machine == "server" { "p1" } else { machine };
    if !names.iter().any(|n| n == name) {
        return Err(CliError::wrong(format!(
            "Session {} has no machine `{machine}`; its machines are {}{}. Name one of those.",
            session.id,
            names.join(", "),
            if many {
                ", and the groups all, clients and headless"
            } else {
                ""
            }
        ))
        .tag("unknown-machine"));
    }
    Ok(vec![name.to_string()])
}

fn check_ceiling(ceiling: f64) -> Result<(), CliError> {
    if ceiling.is_finite() && ceiling > 0.0 && ceiling <= CEILING_MAX_S {
        return Ok(());
    }
    Err(CliError::wrong(format!(
        "The ceiling {ceiling} is not a number of seconds above 0 and at most {CEILING_MAX_S}. Give a ceiling in that range."
    ))
    .tag("bad-arguments"))
}

fn check_args(args: Value) -> Result<Value, CliError> {
    match args {
        Value::Null => Ok(json!([])),
        Value::Array(_) => Ok(args),
        other => Err(CliError::wrong(format!(
            "The arguments must be a JSON array, which the code gets as _this; got {other}. Wrap them in [ ]."
        ))
        .tag("bad-arguments")),
    }
}

fn number(v: &Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// A handler's error in the `ScriptError` shape of the result file; `check_id` is `null` for an
/// error raised outside any check.
pub fn script_error(e: &Value) -> Value {
    let check = e["check_id"].as_str().unwrap_or_default();
    let mut out = errors::from_handler(e, check);
    if check.is_empty() {
        out["check_id"] = Value::Null;
    }
    if let Some(o) = out.as_object_mut() {
        o.remove("_compile");
    }
    out
}

fn compile_error(machine: &str, message: &Value) -> CliError {
    CliError::wrong(format!(
        "The code does not compile on {machine}: {}. Fix the SQF and send it again.",
        message.as_str().unwrap_or("the engine gave no message")
    ))
    .tag("compile-error")
}

/// One `exec` order to each machine at once; each machine's name, the order id, the answer and
/// how long it took. Dropped before every answer came, the orders still running are stopped.
async fn send(
    session: &Arc<Session>,
    names: &[String],
    source: &str,
    args: &Value,
    ceiling: f64,
) -> Vec<(String, String, Option<Value>, f64)> {
    let mut set = JoinSet::new();
    for (i, name) in names.iter().enumerate() {
        let session = Arc::clone(session);
        let (name, source, args) = (name.clone(), source.to_string(), args.clone());
        set.spawn(async move {
            let clock = Instant::now();
            let (id, answer) = session.exec_tracked(&name, &source, args, ceiling).await;
            (i, name, id, answer.ok(), clock.elapsed().as_secs_f64())
        });
    }
    let mut out = Vec::new();
    while let Some(done) = set.join_next().await {
        if let Ok(x) = done {
            out.push(x);
        }
    }
    out.sort_by_key(|x| x.0);
    out.into_iter().map(|(_, n, id, a, d)| (n, id, a, d)).collect()
}

/// `exec` (`CTL-01`, `CTL-05`): the code on a machine or a group; each machine's state, value and
/// duration, and the script errors the code raised, which do not make the call fail.
pub async fn exec(runner: &Runner, machine: &str, code: &str, args: Value, ceiling: f64) -> Result<Value, CliError> {
    check_ceiling(ceiling)?;
    let args = check_args(args)?;
    let session = session(runner)?;
    let names = targets(&session, machine, true)?;
    let answers = send(&session, &names, code, &args, ceiling).await;
    let history = session.history();
    let mut results = serde_json::Map::new();
    let mut errors = Vec::new();
    for (name, id, answer, took) in answers {
        let result = match answer {
            Some(a) if a["state"] == "compile-error" => return Err(compile_error(&name, &a["error"])),
            Some(a) if a["state"] == "done" => {
                json!({"state": "done", "value": a["value"], "duration_s": round3(number(&a["duration_s"]))})
            }
            Some(a) => json!({"state": "ceiling", "value": null, "duration_s": round3(number(&a["duration_s"]))}),
            None => json!({"state": "lost", "value": null, "duration_s": round3(took)}),
        };
        results.insert(name, result);
        errors.extend(
            history
                .iter()
                .filter(|e| e["order"] == id.as_str() && e["compile"] != true)
                .map(script_error),
        );
    }
    Ok(json!({"results": results, "errors": errors}))
}

/// `wait` (`OBS-01`): polls the condition on one machine every frame until it holds or the
/// ceiling passes. A condition that raises a script error or returns no Boolean ends the wait at
/// that evaluation with `met` `false` and the cause in `error`.
pub async fn wait(
    runner: &Runner,
    machine: &str,
    condition: &str,
    _description: Option<&str>,
    args: Value,
    ceiling: f64,
) -> Result<Value, CliError> {
    check_ceiling(ceiling)?;
    let args = check_args(args)?;
    let session = session(runner)?;
    let name = targets(&session, machine, false)?.remove(0);
    let order_args = json!([condition, args, ceiling]);
    let answers = send(
        &session,
        std::slice::from_ref(&name),
        "_this call skd_fnc_cliWait",
        &order_args,
        ceiling + 2.0,
    )
    .await;
    let (_, id, answer, took) = answers.into_iter().next().expect("one machine");
    let v = match answer {
        Some(a) if a["state"] == "done" && a["value"].is_object() => a["value"].clone(),
        // A script error that stopped the wait's own script leaves no answer of the wait; the
        // error the order raised is the cause.
        Some(a) if a["state"] == "done" => {
            let raised = session
                .history()
                .into_iter()
                .find(|e| e["order"] == id.as_str())
                .map(|e| {
                    format!(
                        "the condition raised a script error: {}",
                        e["text"].as_str().unwrap_or_default()
                    )
                });
            return Ok(json!({"met": false, "waited_s": round3(number(&a["duration_s"])), "error": raised}));
        }
        _ => {
            return Ok(json!({
                "met": false,
                "waited_s": round3(took),
                "error": format!("{name} did not answer the wait within its ceiling of {ceiling} s plus the grace"),
            }));
        }
    };
    if !v["compile"].is_null() {
        return Err(compile_error(&name, &v["compile"]));
    }
    Ok(json!({"met": v["met"] == true, "waited_s": round3(number(&v["waited_s"])), "error": v["error"]}))
}

/// `expect` (`OBS-06`): reads the value on one machine every frame until the comparison holds or
/// the ceiling passes, as `skd_fnc_expectOn` would, without a check.
pub async fn expect(
    runner: &Runner,
    machine: &str,
    value: &str,
    op: &str,
    expected: Value,
    args: Value,
    ceiling: f64,
) -> Result<Value, CliError> {
    check_ceiling(ceiling)?;
    let args = check_args(args)?;
    if !OPS.contains(&op) {
        return Err(CliError::wrong(format!(
            "`{op}` is not an operator; the operators are {}. Use one of those.",
            OPS.join(" ")
        ))
        .tag("bad-arguments"));
    }
    let session = session(runner)?;
    let name = targets(&session, machine, false)?.remove(0);
    let order_args = json!([value, op, expected, args, ceiling]);
    let answers = send(
        &session,
        std::slice::from_ref(&name),
        "_this call skd_fnc_cliExpect",
        &order_args,
        ceiling + 2.0,
    )
    .await;
    let (_, _, answer, took) = answers.into_iter().next().expect("one machine");
    let v = match answer {
        Some(a) if a["state"] == "done" => a["value"].clone(),
        _ => return Ok(json!({"passed": false, "last_value": null, "waited_s": round3(took)})),
    };
    if !v["compile"].is_null() {
        return Err(compile_error(&name, &v["compile"]));
    }
    if let Some(needs) = v["type_error"].as_str() {
        return Err(CliError::wrong(format!(
            "Operator {op} needs {needs}; the value read {} and expected is {expected}. Change the operator or the expected value.",
            v["last_value"]
        ))
        .tag("bad-arguments"));
    }
    Ok(
        json!({"passed": v["passed"] == true, "last_value": v["last_value"], "waited_s": round3(number(&v["waited_s"]))}),
    )
}

/// `inspect` (`INS-01`): a compact summary of a machine, without stopping anything. A machine that
/// does not answer within the ceiling plus the grace comes back with state `lost` and every other
/// field `null`.
pub async fn inspect(runner: &Runner, machine: &str, near: Option<&str>, radius: f64) -> Result<Value, CliError> {
    if !(radius.is_finite() && radius > 0.0) {
        return Err(CliError::wrong(format!(
            "The radius {radius} is not a number of metres above 0. Give one."
        ))
        .tag("bad-arguments"));
    }
    let session = session(runner)?;
    let name = targets(&session, machine, false)?.remove(0);
    let answers = send(
        &session,
        std::slice::from_ref(&name),
        "_this call skd_fnc_inspect",
        &json!([near.unwrap_or_default(), radius]),
        INSPECT_CEILING_S,
    )
    .await;
    let (_, _, answer, _) = answers.into_iter().next().expect("one machine");
    let mut view = session.machine(&name).unwrap_or(Value::Null);
    let v = match answer {
        Some(a) if a["state"] == "done" => a["value"].clone(),
        _ => {
            view["state"] = json!("lost");
            return Ok(json!({
                "machine": view, "mission": null, "mission_start": null, "time": null, "tick": null, "fps": null,
                "frame": null, "run_id": null, "check_id": null, "step_path": null, "player": null, "nearby": null,
                "scripts": null, "errors": null,
            }));
        }
    };
    if let Some(e) = v["error"].as_str() {
        return Err(
            CliError::wrong(format!("{e}. Give the net id of an object that exists on {name}.")).tag("bad-arguments"),
        );
    }
    let executing = runner.executing.lock().unwrap().clone();
    let progress = runner.progress.lock().unwrap().clone();
    let run_id = executing
        .as_ref()
        .and_then(|(key, _)| key.split('#').next().map(String::from))
        .or_else(|| progress.as_ref().and_then(|p| p["id"].as_str().map(String::from)));
    let mine: Vec<Value> = session
        .history()
        .into_iter()
        .filter(|e| e["machine"] == name.as_str() && e["compile"] != true)
        .collect();
    let errors: Vec<Value> = mine[mine.len().saturating_sub(INSPECT_ERRORS)..]
        .iter()
        .map(script_error)
        .collect();
    // A player on foot has no vehicle, which the machine leaves out.
    let mut player = v["player"].clone();
    if let Some(p) = player.as_object_mut() {
        p.entry("vehicle").or_insert(Value::Null);
    }
    Ok(json!({
        "machine": view,
        "mission": v["mission"],
        "mission_start": session.mission_start(&name),
        "time": number(&v["time"]),
        "tick": number(&v["tick"]),
        "fps": number(&v["fps"]),
        "frame": number(&v["frame"]) as u64,
        "run_id": run_id,
        "check_id": executing.map(|(_, id)| id),
        "step_path": v["step_path"].as_array().cloned().unwrap_or_default(),
        "player": player,
        "nearby": v["nearby"].as_array().cloned().unwrap_or_default(),
        "scripts": v["scripts"].as_array().cloned().unwrap_or_default(),
        "errors": errors,
    }))
}

/// `errors` (`ERR-05`): the script errors the session caught, inside checks and outside, the
/// oldest first, narrowed to a machine, a run or a check.
pub fn errors(
    runner: &Runner,
    machine: Option<&str>,
    run: Option<&str>,
    check: Option<&str>,
) -> Result<Value, CliError> {
    let session = runner.session().ok_or_else(|| {
        CliError::not_now("No session runs. Start one with `shakedown session start --setup <name>`.").tag("no-session")
    })?;
    let name = match machine {
        Some(m) => Some(targets(&session, m, false)?.remove(0)),
        None => None,
    };
    let errors: Vec<Value> = session
        .history()
        .iter()
        .filter(|e| name.as_deref().is_none_or(|n| e["machine"] == n))
        .filter(|e| {
            run.is_none_or(|r| {
                let key = e["key"].as_str().unwrap_or_default();
                key == r || key.starts_with(&format!("{r}#"))
            })
        })
        .filter(|e| check.is_none_or(|c| e["check_id"] == c))
        .filter(|e| e["compile"] != true)
        .map(script_error)
        .collect();
    Ok(json!({"errors": errors}))
}
