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
        rebuild,
        wait_s,
    })
}

/// `list` (`RUN-02`): each selected check once, from the files, with no runner.
pub fn list(project: &config::Project, ids: &[String], tags: &[String]) -> Result<Value, CliError> {
    let all = checks::discover(&project.tests)?;
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
            json!({"check_id": o["check_id"], "repeat": o["repeat"], "outcome": o["outcome"],
                   "reason": o["reason"], "duration_s": o["duration_s"]})
        })
        .collect();
    Some(json!({
        "run_id": run_id, "state": result["state"], "done": outcomes.len(), "total": result["summary"]["total"],
        "current_check": null, "outcomes": outcomes,
    }))
}

/// The guide of `AGT-06`.
pub const GUIDE: &str = include_str!("mcp/guide.txt");
