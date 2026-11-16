//! What the command line and the MCP server both do for a request: the same runner requests and
//! the same JSON, defined once (`EP-07`, `NFR-US-03`). The command line prints the value; the MCP
//! server returns it as the structured content of the command's twin tool.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::checks;
use crate::client::{self, Conn};
use crate::config;
use crate::error::CliError;
use crate::runner::run;
use crate::wire::{Event, Request, SetupSpec};

/// The project: the folder given, or the nearest one upward from the current folder.
pub fn project(given: Option<&Path>) -> Result<config::Project, CliError> {
    let cwd =
        std::env::current_dir().map_err(|e| CliError::framework(format!("Cannot read the current folder: {e}.")))?;
    let root = config::find_root(given, &cwd)?;
    let root = std::fs::canonicalize(&root).map(strip_verbatim).unwrap_or(root);
    config::load(&root)
}

/// `canonicalize` on Windows gives `\\?\C:\...`, which the game's options do not take.
fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => p,
    }
}

pub fn spec(project: &config::Project, name: &str) -> Result<SetupSpec, CliError> {
    let setup = project.setup(name)?.clone();
    let mods = setup.mods.iter().map(|m| project.mods[m].clone()).collect();
    Ok(SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
        base_port: project.base_port,
        project: project.root.clone(),
        lib: project.lib.clone(),
    })
}

/// A `no-session` refusal names the command that starts a session and the setups the project
/// declares (`US-047`).
pub fn with_setups(e: CliError, project: Option<&config::Project>) -> CliError {
    if e.tag.as_deref() != Some("no-session") {
        return e;
    }
    let setups: Vec<String> = project.map(|p| p.setups.keys().cloned().collect()).unwrap_or_default();
    CliError {
        message: format!(
            "No session runs. Start one with `shakedown session start --setup <name>`; the project declares the setups: {}.",
            if setups.is_empty() {
                "none".to_string()
            } else {
                setups.join(", ")
            }
        ),
        ..e
    }
}

pub fn session_of(event: Event) -> Value {
    match event {
        Event::Session { session } => session.unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

/// Sends a request whose answer is a value.
pub fn value(conn: &mut Conn, request: &Request, on_event: impl FnMut(&Event)) -> Result<Value, CliError> {
    match conn.request(request, on_event)? {
        Event::Value { value } => Ok(value),
        _ => Err(CliError::framework("The runner answered with something else.")),
    }
}

/// `status` (`INS-02`): with no runner, and with a runner that does not answer, without starting
/// one (`AGT-05`, `P-01`).
pub fn status() -> Result<Value, CliError> {
    match client::existing() {
        Ok(None) => Ok(json!({
            "runner": "not-running", "runner_pid": null, "version": shakedown_protocol::VERSION,
            "session": null, "run": null, "last_reload": null,
        })),
        Ok(Some(mut conn)) => value(&mut conn, &Request::Status, |_| {}),
        Err(_) => Ok(json!({
            "runner": "unreachable", "runner_pid": client::runner_pid(), "version": shakedown_protocol::VERSION,
            "session": null, "run": null, "last_reload": null,
        })),
    }
}

/// The runner request of `reload` (`RLD-01`): each file made absolute against `base` (the current
/// folder for the command line, the project for the MCP tool).
pub fn reload_request(project: &config::Project, files: &[PathBuf], wait_s: Option<f64>) -> Result<Request, CliError> {
    let base = if wait_s.is_some() {
        project.root.clone()
    } else {
        std::env::current_dir().map_err(|e| CliError::framework(format!("Cannot read the current folder: {e}.")))?
    };
    let files = files
        .iter()
        .map(|f| if f.is_absolute() { f.clone() } else { base.join(f) })
        .collect();
    let rebuild = crate::wire::Rebuild {
        folder: project.root.clone(),
        commands: project
            .mods
            .values()
            .filter_map(|m| Some((m.name.clone(), m.rebuild.clone()?)))
            .collect(),
    };
    Ok(Request::Reload {
        files,
        tests: project.tests.clone(),
        lib: project.lib.clone(),
        rebuild,
        wait_s,
    })
}

/// `list` (`RUN-02`): each selected check once, from the files, with no runner.
pub fn list(project: &config::Project, ids: &[String], tags: &[String]) -> Result<Value, CliError> {
    let all = checks::discover(&project.tests, &project.lib)?;
    let mut selected = checks::select(&all, ids, tags)?;
    let mut seen = Vec::new();
    selected.retain(|c| {
        let first = !seen.contains(&c.id);
        seen.push(c.id.clone());
        first
    });
    let declared: Vec<String> = project.setups.keys().cloned().collect();
    let listed: Vec<checks::Listed> = selected
        .iter()
        .map(|c| checks::read_listed(c, Some(&declared)))
        .collect();
    Ok(json!({"checks": listed}))
}

/// The `run_result` shape of a run that ended, from its result file (`AGT-02`).
pub fn run_result(project: &Path, run_id: &str) -> Option<Value> {
    let file = project.join(".shakedown").join("runs").join(run_id).join("result.json");
    let result: Value = serde_json::from_str(&std::fs::read_to_string(&file).ok()?).ok()?;
    Some(run::run_result_of(&result, &file))
}

/// The `run_status` shape of a run that ended, from its result file.
pub fn run_status_of_file(project: &Path, run_id: &str) -> Option<Value> {
    run_status_of_result(&project.join(".shakedown").join("runs").join(run_id).join("result.json"))
}

/// The `run_status` shape of a run that ended, from the result file at `file`; `total` counts
/// the checks not run too.
pub fn run_status_of_result(file: &Path) -> Option<Value> {
    let content: Value = serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()?;
    let result = run::run_result_of(&content, file);
    let run_id = result["run_id"].clone();
    let outcomes: Vec<Value> = result["outcomes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|o| {
            json!({"check_id": o["check_id"], "repeat": o["repeat"], "setup": o["setup"], "outcome": o["outcome"],
                   "reason": o["reason"], "duration_s": o["duration_s"], "slow": o["slow"]})
        })
        .collect();
    Some(json!({
        "run_id": run_id, "state": result["state"], "done": outcomes.len(), "total": result["summary"]["total"],
        "current_check": null, "outcomes": outcomes, "setup": content["session"]["setup"]["name"],
    }))
}

/// The guide of `AGT-06`.
pub const GUIDE: &str = include_str!("mcp/guide.txt");

/// The exit code of one run from its `run_result` (`contract/cli.md`): 1 when a check failed, 2
/// when none failed and a check was invalid, the run was stopped, or a check was not run because
/// its session failed to start or broke (`RUN-05`, `RUN-07`, `OUT-02`), otherwise 0.
pub fn exit_of(result: &Value) -> u8 {
    let count = |k: &str| result["summary"][k].as_u64().unwrap_or(0);
    let lost = result["not_run"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|n| n["reason"] == "session-failed" || n["reason"] == "session-broken");
    if count("fail") > 0 {
        1
    } else if count("invalid") > 0 || result["state"] == "stopped" || lost {
        2
    } else {
        0
    }
}

/// The setups a run over every setup needs (`RUN-07`), in the order the project file declares
/// them, each with the selected checks that name it. A check with no `@setups`, or whose header
/// cannot be read, names every setup.
pub fn setups_needed(project: &config::Project, selected: &[checks::CheckRef]) -> Vec<(String, Vec<checks::CheckRef>)> {
    let named: Vec<Option<Vec<String>>> = selected
        .iter()
        .map(|c| {
            let text = std::fs::read_to_string(&c.path).ok()?;
            checks::parse_header_in(&text, Some(&project.setup_order)).ok()?.setups
        })
        .collect();
    project
        .setup_order
        .iter()
        .filter_map(|setup| {
            let mine: Vec<checks::CheckRef> = selected
                .iter()
                .zip(&named)
                .filter(|(_, n)| n.as_ref().is_none_or(|n| n.contains(setup)))
                .map(|(c, _)| c.clone())
                .collect();
            (!mine.is_empty()).then(|| (setup.clone(), mine))
        })
        .collect()
}

/// A run over every setup is refused while a session runs (`RUN-07`); a failed start that
/// `status` still shows is no session.
pub fn refuse_running_session(running: &Value) -> Result<(), CliError> {
    if running.is_null() || running["state"] == "failed" {
        return Ok(());
    }
    Err(CliError::not_now(format!(
        "Session {} of setup {} runs, and a run over every setup starts a session of its own for each. Stop it with `shakedown session stop`, or run against it without --each-setup.",
        running["id"].as_str().unwrap_or("?"),
        running["setup"].as_str().unwrap_or("?"),
    ))
    .tag("session-exists"))
}

/// What the command line and the MCP server each do as a run over every setup goes.
pub trait EachSetupHooks {
    /// A setup's turn has come; `run_id` is its run's id.
    fn began(&self, setup: &str, run_id: &str);
    /// An event of the session start or the run, such as a check's line.
    fn event(&self, event: &Event);
    /// A setup's run ended, or its session did not start: its `run_result` with `setup`.
    fn ended(&self, setup: &str, result: &Value);
    /// A stop was asked: no further setup starts.
    fn stopped(&self) -> bool;
}

/// `run --each-setup` and `run_start` with `each_setup` (`RUN-07`): each setup of `plan` in turn
/// gets a session of its own, a run of its checks and a result file, and its session is stopped
/// before the next starts. A setup whose start is refused or fails gets a result file with every
/// check `not-run` / `session-failed`, and the next setup still runs. The answer is each setup's
/// `run_result` with `setup`, and `session_reason` for a setup that did not start.
pub fn run_each_setup(
    project: &config::Project,
    plan: &[(String, Vec<checks::CheckRef>)],
    (ids, tags): (&[String], &[String]),
    repeat: u32,
    hooks: &dyn EachSetupHooks,
) -> Result<Vec<Value>, CliError> {
    let mut results = Vec::new();
    for (setup, mine) in plan {
        if hooks.stopped() {
            break;
        }
        let spec = spec(project, setup)?;
        let mut conn = client::connect()?;
        let run_id = value(&mut conn, &Request::ReserveRun, |_| {})?["run_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        hooks.began(setup, &run_id);
        let started = conn.request(
            &Request::SessionStart {
                project: project.root.clone(),
                spec: spec.clone(),
            },
            |e| hooks.event(e),
        );
        // The start's reason when it was refused or failed, and the session's id when one existed.
        let failed: Option<(Option<String>, String)> = match started {
            Ok(event) => {
                let session = session_of(event);
                (session["state"] == "failed").then(|| {
                    let reason = &session["reason"];
                    let machine = reason["machine"]
                        .as_str()
                        .map(|m| format!(" ({m})"))
                        .unwrap_or_default();
                    (
                        session["id"].as_str().map(String::from),
                        format!(
                            "session {} failed to start{machine}: {}",
                            session["id"].as_str().unwrap_or("?"),
                            reason["message"].as_str().unwrap_or("the start failed")
                        ),
                    )
                })
            }
            Err(e) => Some((None, format!("the session start was refused: {}", e.message))),
        };
        let mut result = match failed {
            None => {
                let request = Request::Run {
                    project: project.root.clone(),
                    setups: project.setup_order.clone(),
                    checks: mine.clone(),
                    ids: ids.to_vec(),
                    tags: tags.to_vec(),
                    run_id: Some(run_id.clone()),
                    repeat,
                    slow_step: project.slow_step,
                };
                let answer = conn.request(&request, |e| hooks.event(e));
                conn.request(&Request::SessionStop, |e| hooks.event(e))?;
                match answer? {
                    Event::RunDone { result } => result,
                    _ => return Err(CliError::framework("The runner ended the run without a result.")),
                }
            }
            Some((session_id, message)) => {
                // A failed start that status still shows is cleared, so nothing of it is left.
                let _ = conn.request(&Request::SessionStop, |_| {});
                let request = Request::RunUnstarted {
                    project: project.root.clone(),
                    run_id: run_id.clone(),
                    spec: spec.clone(),
                    checks: mine.clone(),
                    ids: ids.to_vec(),
                    tags: tags.to_vec(),
                    repeat,
                    slow_step: project.slow_step,
                    session_id,
                    message: message.clone(),
                };
                let mut result = match conn.request(&request, |_| {})? {
                    Event::RunDone { result } => result,
                    _ => return Err(CliError::framework("The runner wrote no result for the setup.")),
                };
                result["session_reason"] = json!(message);
                result
            }
        };
        result["setup"] = json!(setup);
        hooks.ended(setup, &result);
        results.push(result);
    }
    Ok(results)
}

/// The `run_result` of a whole run over every setup (`RUN-07`): the setups' summaries added up,
/// their outcomes, checks not run and repeat summaries in order, and each setup's own answer in
/// `runs`.
pub fn each_setup_result(handle: &str, results: &[Value], stopped: bool) -> Value {
    let mut summary = json!({"total": 0, "pass": 0, "fail": 0, "invalid": 0, "not_run": 0});
    let mut outcomes = Vec::new();
    let mut not_run = Vec::new();
    let mut repeats = Vec::new();
    for r in results {
        for k in ["total", "pass", "fail", "invalid", "not_run"] {
            summary[k] = json!(summary[k].as_u64().unwrap_or(0) + r["summary"][k].as_u64().unwrap_or(0));
        }
        outcomes.extend(r["outcomes"].as_array().cloned().unwrap_or_default());
        not_run.extend(r["not_run"].as_array().cloned().unwrap_or_default());
        repeats.extend(r["repeats"].as_array().cloned().unwrap_or_default());
    }
    let stopped = stopped || results.iter().any(|r| r["state"] == "stopped");
    let mut answer = json!({
        "run_id": handle,
        "state": if stopped { "stopped" } else { "finished" },
        "summary": summary,
        "outcomes": outcomes,
        "not_run": not_run,
        "result_file": null,
        "runs": results,
    });
    if results.iter().any(|r| r["repeats"].is_array()) {
        answer["repeats"] = json!(repeats);
    }
    answer
}

/// A handle for a run over every setup, `e-<UTC time>-<n>` (`RUN-07`).
pub fn each_setup_handle() -> String {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    format!(
        "e-{}-{}",
        shakedown_protocol::time::stamp(std::time::SystemTime::now()),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A project with the setups `sp`, `sp-ace` and `dedicated-2`, declared in that order, and
    /// checks that name one, two or all three of them.
    fn project(dir: &Path) -> config::Project {
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        let toml = "game = 'C:/Games/Arma 3'\n\n[setups.sp]\nmode = \"singleplayer\"\nmods = []\n\n[setups.sp-ace]\nmode = \"singleplayer\"\nmods = []\n\n[setups.dedicated-2]\nmode = \"dedicated\"\nclients = 2\nmods = []\n";
        std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
        let checks = [
            ("all", "// @check US-074 every setup\n// @tags m00\n"),
            (
                "two",
                "// @check US-074 two setups\n// @setups dedicated-2 sp-ace\n// @tags m00\n",
            ),
            (
                "ded",
                "// @check US-074 dedicated only\n// @setups dedicated-2\n// @tags m01\n",
            ),
            ("broken", "// no header\n"),
        ];
        for (id, text) in checks {
            std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
        }
        config::load(dir).unwrap()
    }

    fn names(plan: &[(String, Vec<checks::CheckRef>)]) -> Vec<(String, Vec<String>)> {
        plan.iter()
            .map(|(s, c)| (s.clone(), c.iter().map(|c| c.id.clone()).collect()))
            .collect()
    }

    #[test]
    fn us_074_run_07_each_setup_runs_in_the_file_order_with_the_checks_that_name_it() {
        let dir = std::env::temp_dir().join(format!("skd-m13-plan-{}", std::process::id()));
        let p = project(&dir);
        let all = checks::discover(&p.tests, &p.lib).unwrap();
        let ids = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let selected = checks::select(&all, &ids(&["all", "two", "ded"]), &[]).unwrap();
        assert_eq!(
            names(&setups_needed(&p, &selected)),
            vec![
                ("sp".to_string(), ids(&["all"])),
                ("sp-ace".to_string(), ids(&["all", "two"])),
                ("dedicated-2".to_string(), ids(&["all", "two", "ded"])),
            ]
        );
        // US-074 4: a selection starts only the setups its checks name.
        let selected = checks::select(&all, &[], &ids(&["m01"])).unwrap();
        assert_eq!(
            names(&setups_needed(&p, &selected)),
            vec![("dedicated-2".to_string(), ids(&["ded"]))]
        );
        // A check whose header cannot be read names every setup, so its check-error is reported.
        let selected = checks::select(&all, &ids(&["broken"]), &[]).unwrap();
        assert_eq!(setups_needed(&p, &selected).len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn us_074_run_07_a_run_over_every_setup_is_refused_while_a_session_runs() {
        assert!(refuse_running_session(&Value::Null).is_ok());
        assert!(refuse_running_session(&json!({"id": "s-1", "setup": "sp", "state": "failed"})).is_ok());
        let e = refuse_running_session(&json!({"id": "s-1", "setup": "sp", "state": "ready"})).unwrap_err();
        assert_eq!((e.code, e.tag.as_deref()), (4, Some("session-exists")));
        assert!(e.message.contains("session stop"), "{}", e.message);
    }

    #[test]
    fn us_074_us_075_run_07_the_whole_result_adds_up_the_setups_and_exits_with_the_highest_code() {
        let pass = json!({"run_id": "r-1", "state": "finished", "setup": "sp",
            "summary": {"total": 2, "pass": 2, "fail": 0, "invalid": 0, "not_run": 0},
            "outcomes": [{"check_id": "a"}, {"check_id": "b"}], "not_run": []});
        let failed = json!({"run_id": "r-2", "state": "finished", "setup": "sp-ace", "session_reason": "refused",
            "summary": {"total": 1, "pass": 0, "fail": 0, "invalid": 0, "not_run": 1},
            "outcomes": [], "not_run": [{"check_id": "a", "repeat": 1, "reason": "session-failed", "message": "refused"}]});
        let whole = each_setup_result("e-1", &[pass.clone(), failed.clone()], false);
        assert_eq!(whole["summary"]["total"], 3);
        assert_eq!(whole["summary"]["not_run"], 1);
        assert_eq!(whole["runs"].as_array().unwrap().len(), 2);
        assert_eq!(whole["result_file"], Value::Null);
        assert_eq!([exit_of(&pass), exit_of(&failed)], [0, 2]);
    }
}
