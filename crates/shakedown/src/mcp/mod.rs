//! `shakedown mcp`: the MCP server over stdio (`AGT-01`, `INT-MCP-01`). Every tool asks the runner
//! the same requests as its command line twin and returns the same JSON (`NFR-US-03`). No call
//! lasts longer than 60 s (`AGT-03`); a cancelled call closes its runner connection, which stops
//! the work it started in the game (`AGT-04`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData, Implementation, InitializeResult,
    ListToolsResult, PaginatedRequestParams, ProgressNotificationParam, ProgressToken, ServerCapabilities, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::checks;
use crate::client::{self, Closer};
use crate::error::CliError;
use crate::ops;
use crate::wire::{Event, Request};

/// The longest a tool waits before it answers (`AGT-03`).
const WAIT_MAX_S: f64 = 50.0;
/// How often a waiting tool sends a progress notification (`AGT-03`: at least every 5 s).
const PROGRESS_EVERY: Duration = Duration::from_secs(4);
/// The most characters a result holds; a longer one goes to a file (`AGT-07`).
const RESULT_LIMIT: usize = 20_000;
/// How long `run_start` waits for the run to begin before it answers.
const RUN_BEGIN_CEILING: Duration = Duration::from_secs(4);

const TOOLS: &str = include_str!("tools.json");
const RESULT_SCHEMA: &str = include_str!("../../../../spec/contract/result.schema.json");

/// A run started by `run_start`, followed by a thread that holds its runner connection.
struct Track {
    state: String,
    ids: Vec<String>,
    outcomes: Vec<Value>,
    ended: bool,
    /// A refusal before the run began, returned by `run_start`.
    refused: Option<CliError>,
}

pub struct Server {
    project: Option<PathBuf>,
    tools: Vec<Tool>,
    runs: Arc<Mutex<HashMap<String, Track>>>,
}

/// The tools of `contract/mcp-tools.md`, each output schema with the shapes it names from
/// `result.schema.json` and this file's own, and the alternative of a result written to a file.
pub fn tools() -> Vec<Tool> {
    let spec: Value = serde_json::from_str(TOOLS).expect("tools.json is JSON");
    let result: Value = serde_json::from_str(RESULT_SCHEMA).expect("result.schema.json is JSON");
    let mut defs = result["$defs"].as_object().cloned().unwrap_or_default();
    defs.extend(spec["defs"].as_object().cloned().unwrap_or_default());
    spec["tools"]
        .as_array()
        .expect("tools.json has tools")
        .iter()
        .map(|t| {
            let mut t = t.clone();
            for key in ["inputSchema", "outputSchema"] {
                let schema = t[key].clone();
                let mut used = Map::new();
                collect_refs(&schema, &defs, &mut used);
                let mut out = if key == "outputSchema" {
                    json!({
                        "type": "object",
                        "anyOf": [schema, {
                            "type": "object",
                            "description": "A result longer than 20,000 characters, written to a file (AGT-07).",
                            "required": ["result_path"],
                            "properties": {"result_path": {"type": "string"}}
                        }]
                    })
                } else {
                    schema
                };
                out["$schema"] = json!("https://json-schema.org/draft/2020-12/schema");
                if !used.is_empty() {
                    out["$defs"] = Value::Object(used);
                }
                t[key] = out;
            }
            serde_json::from_value(t).expect("a tool definition")
        })
        .collect()
}

/// Every `$defs` entry `value` refers to, directly or through another.
fn collect_refs(value: &Value, defs: &Map<String, Value>, used: &mut Map<String, Value>) {
    match value {
        Value::Object(o) => {
            if let Some(name) = o
                .get("$ref")
                .and_then(|r| r.as_str())
                .and_then(|r| r.strip_prefix("#/$defs/"))
                && !used.contains_key(name)
                && let Some(def) = defs.get(name)
            {
                used.insert(name.to_string(), def.clone());
                collect_refs(def, defs, used);
            }
            o.values().for_each(|v| collect_refs(v, defs, used));
        }
        Value::Array(a) => a.iter().for_each(|v| collect_refs(v, defs, used)),
        _ => {}
    }
}

fn bad(message: String) -> CliError {
    CliError::wrong(message).tag("bad-arguments")
}

fn args<T: for<'de> Deserialize<'de>>(tool: &str, arguments: Value) -> Result<T, CliError> {
    serde_json::from_value(arguments).map_err(|e| {
        bad(format!(
            "The arguments of {tool} do not match its input schema: {e}. Give the arguments its inputSchema names"
        ))
    })
}

/// A ceiling of a tool that waits: at most 50 s (`AGT-03`).
fn ceiling(given: Option<f64>, default: f64) -> Result<f64, CliError> {
    let c = given.unwrap_or(default);
    if !(c.is_finite() && c > 0.0 && c <= WAIT_MAX_S) {
        return Err(bad(format!(
            "The ceiling {c} is not above 0 and at most {WAIT_MAX_S} s, the most a tool call waits. Give a ceiling of at most {WAIT_MAX_S}"
        )));
    }
    Ok(c)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupArgs {
    setup: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    setup: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReloadArgs {
    #[serde(default)]
    files: Vec<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunArgs {
    run_id: String,
    #[serde(default)]
    wait_s: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecArgs {
    machine: String,
    code: String,
    #[serde(default)]
    args: Value,
    #[serde(default)]
    ceiling: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WaitArgs {
    machine: String,
    condition: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    args: Value,
    #[serde(default)]
    ceiling: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectArgs {
    machine: String,
    value: String,
    op: String,
    expected: Value,
    #[serde(default)]
    args: Value,
    #[serde(default)]
    ceiling: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectArgs {
    machine: String,
    #[serde(default)]
    near: Option<String>,
    #[serde(default)]
    radius: Option<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogsArgs {
    machine: String,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    check_id: Option<String>,
    #[serde(default)]
    grep: Option<String>,
    #[serde(default)]
    last: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorsArgs {
    #[serde(default)]
    machine: Option<String>,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    check_id: Option<String>,
}

/// Sends progress notifications while a call waits, when the client gave a progress token.
struct Progress {
    ctx: RequestContext<RoleServer>,
    token: Option<ProgressToken>,
    sent: f64,
}

impl Progress {
    fn new(ctx: &RequestContext<RoleServer>) -> Progress {
        Progress {
            ctx: ctx.clone(),
            token: ctx.meta.get_progress_token(),
            sent: 0.0,
        }
    }

    async fn tick(&mut self, message: String) {
        if let Some(token) = &self.token {
            self.sent += 1.0;
            let _ = self
                .ctx
                .peer
                .notify_progress(ProgressNotificationParam::new(token.clone(), self.sent).with_message(message))
                .await;
        }
    }
}

impl Server {
    pub fn new(project: Option<PathBuf>) -> Server {
        Server {
            project,
            tools: tools(),
            runs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn project(&self) -> Result<crate::config::Project, CliError> {
        ops::project(self.project.as_deref())
    }

    /// Runs a runner request on a thread of its own, with progress notifications while it waits;
    /// a cancelled call closes the request's connection (`AGT-04`).
    async fn ask(&self, request: Request, ctx: &RequestContext<RoleServer>) -> Result<Value, CliError> {
        let project = self.project().ok();
        let closer: Arc<Mutex<Option<Closer>>> = Arc::new(Mutex::new(None));
        let held = Arc::clone(&closer);
        let mut work = tokio::task::spawn_blocking(move || {
            let mut conn = client::connect()?;
            *held.lock().unwrap() = conn.closer();
            ops::value(&mut conn, &request, |_| {})
        });
        let mut progress = Progress::new(ctx);
        let clock = Instant::now();
        loop {
            tokio::select! {
                done = &mut work => {
                    let answer = done.map_err(|e| CliError::framework(format!("The request failed: {e}.")))?;
                    return answer.map_err(|e| ops::with_setups(e, project.as_ref()));
                }
                _ = ctx.ct.cancelled() => {
                    if let Some(c) = closer.lock().unwrap().take() {
                        c.close();
                    }
                    return Err(CliError::framework("The call was cancelled."));
                }
                _ = tokio::time::sleep(PROGRESS_EVERY) => {
                    progress.tick(format!("waiting, {} s", clock.elapsed().as_secs())).await;
                }
            }
        }
    }

    async fn call(&self, name: &str, arguments: Value, ctx: &RequestContext<RoleServer>) -> Result<Value, CliError> {
        match name {
            "status" => {
                args::<Empty>(name, arguments)?;
                blocking(ops::status).await
            }
            "guide" => {
                args::<Empty>(name, arguments)?;
                Ok(json!({"text": ops::GUIDE}))
            }
            "list_checks" => {
                let a: Selection = args(name, arguments)?;
                if a.setup.is_some() {
                    return Err(bad("list_checks takes ids and tags, not setup. Leave setup out".into()));
                }
                ops::list(&self.project()?, &a.ids, &a.tags)
            }
            "session_start" => {
                let a: SetupArgs = args(name, arguments)?;
                self.session_start(&a.setup, ctx).await
            }
            "session_stop" => {
                args::<Empty>(name, arguments)?;
                blocking(|| {
                    let mut conn = client::connect()?;
                    Ok(ops::session_of(conn.request(&Request::SessionStop, |_| {})?))
                })
                .await
            }
            "reset" => {
                args::<Empty>(name, arguments)?;
                self.ask(Request::Reset, ctx).await
            }
            "reload" => {
                let a: ReloadArgs = args(name, arguments)?;
                let project = self.project()?;
                // Files are relative to the project; the reload goes on in the runner after 50 s
                // (AGT-03), and `status` shows it as `last_reload`.
                let request = ops::reload_request(&project, &a.files, Some(WAIT_MAX_S))?;
                self.ask(request, ctx).await
            }
            "exec" => {
                let a: ExecArgs = args(name, arguments)?;
                let ceiling = ceiling(a.ceiling, 10.0)?;
                self.ask(
                    Request::Exec {
                        machine: a.machine,
                        code: a.code,
                        args: a.args,
                        ceiling,
                        source: "mcp".into(),
                    },
                    ctx,
                )
                .await
            }
            "wait_until" => {
                let a: WaitArgs = args(name, arguments)?;
                let ceiling = ceiling(a.ceiling, 10.0)?;
                self.ask(
                    Request::Wait {
                        machine: a.machine,
                        condition: a.condition,
                        description: a.description,
                        args: a.args,
                        ceiling,
                    },
                    ctx,
                )
                .await
            }
            "expect" => {
                let a: ExpectArgs = args(name, arguments)?;
                let ceiling = ceiling(a.ceiling, 10.0)?;
                self.ask(
                    Request::Expect {
                        machine: a.machine,
                        value: a.value,
                        op: a.op,
                        expected: a.expected,
                        args: a.args,
                        ceiling,
                    },
                    ctx,
                )
                .await
            }
            "inspect" => {
                let a: InspectArgs = args(name, arguments)?;
                self.ask(
                    Request::Inspect {
                        machine: a.machine,
                        near: a.near,
                        radius: a.radius.unwrap_or(50.0),
                    },
                    ctx,
                )
                .await
            }
            "logs" => {
                let a: LogsArgs = args(name, arguments)?;
                let project = self.project()?;
                self.ask(
                    Request::Logs {
                        project: project.root,
                        machine: a.machine,
                        run: a.run_id,
                        check: a.check_id,
                        grep: a.grep,
                        last: a.last.unwrap_or(200),
                    },
                    ctx,
                )
                .await
            }
            "errors" => {
                let a: ErrorsArgs = args(name, arguments)?;
                self.ask(
                    Request::Errors {
                        machine: a.machine,
                        run: a.run_id,
                        check: a.check_id,
                    },
                    ctx,
                )
                .await
            }
            "run_start" => {
                let a: Selection = args(name, arguments)?;
                self.run_start(a).await
            }
            "run_status" => {
                let a: RunArgs = args(name, arguments)?;
                self.run_status(&a.run_id, a.wait_s, ctx).await
            }
            "run_result" => {
                let a: RunArgs = args(name, arguments)?;
                if a.wait_s.is_some() {
                    return Err(bad("run_result takes only run_id. Leave wait_s out".into()));
                }
                self.run_result(&a.run_id)
            }
            other => Err(bad(format!(
                "There is no tool {other}. Call one of the tools tools/list names"
            ))),
        }
    }

    /// `session_start`: the session once it is ready or failed, or after 50 s the session as it
    /// stands, `starting`; the start goes on in the runner (`AGT-03`).
    async fn session_start(&self, setup: &str, ctx: &RequestContext<RoleServer>) -> Result<Value, CliError> {
        let project = self.project()?;
        let spec = ops::spec(&project, setup)?;
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let answer = client::connect().and_then(|mut conn| {
                conn.request(
                    &Request::SessionStart {
                        project: project.root.clone(),
                        spec,
                    },
                    |_| {},
                )
                .map(ops::session_of)
            });
            let _ = tx.send(answer);
        });
        let mut progress = Progress::new(ctx);
        let clock = Instant::now();
        let deadline = tokio::time::Instant::now() + Duration::from_secs_f64(start_wait_s());
        loop {
            tokio::select! {
                answer = &mut rx => {
                    return answer.map_err(|_| CliError::framework("The session start ended without an answer."))?;
                }
                _ = tokio::time::sleep_until(deadline) => {
                    let status = blocking(ops::status).await?;
                    return Ok(status["session"].clone());
                }
                _ = tokio::time::sleep(PROGRESS_EVERY) => {
                    progress.tick(format!("starting, {} s", clock.elapsed().as_secs())).await;
                }
            }
        }
    }

    /// `run_start`: the run's id at once; a thread holds the run's connection and records each
    /// outcome for `run_status` (`AGT-03`). With a setup and no session, the run starts one and
    /// stops it at its end (`RUN-04`).
    async fn run_start(&self, a: Selection) -> Result<Value, CliError> {
        let project = self.project()?;
        let all = checks::discover(&project.checks)?;
        let selected = checks::select(&all, &a.ids, &a.tags)?;
        let spec = a.setup.as_deref().map(|s| ops::spec(&project, s)).transpose()?;
        let (running, run_id) = blocking(|| {
            let mut conn = client::connect()?;
            let running = ops::session_of(conn.request(&Request::SessionInfo, |_| {})?);
            let id = ops::value(&mut conn, &Request::ReserveRun, |_| {})?;
            Ok(json!([running, id["run_id"]]))
        })
        .await
        .map(|v| (v[0].clone(), v[1].as_str().unwrap_or_default().to_string()))?;
        let start = match (&spec, running.is_null()) {
            (Some(spec), true) => Some(spec.clone()),
            (Some(spec), false) if running["setup"] != spec.setup.name.as_str() => {
                return Err(crate::cli::other_setup(&running, &spec.setup.name));
            }
            (None, true) => {
                return Err(ops::with_setups(
                    CliError::not_now("No session runs.").tag("no-session"),
                    Some(&project),
                ));
            }
            _ => None,
        };
        let total = selected.len();
        let starting = start.is_some();
        self.runs.lock().unwrap().insert(
            run_id.clone(),
            Track {
                state: if starting { "starting-session" } else { "pending" }.into(),
                ids: selected.iter().map(|c| c.id.clone()).collect(),
                outcomes: vec![],
                ended: false,
                refused: None,
            },
        );
        let runs = Arc::clone(&self.runs);
        let id = run_id.clone();
        let request = Request::Run {
            project: project.root.clone(),
            setups: project.setups.keys().cloned().collect(),
            checks: selected,
            ids: a.ids,
            tags: a.tags,
            run_id: Some(run_id.clone()),
        };
        let root = project.root.clone();
        std::thread::spawn(move || follow(runs, id, root, start, request));
        if starting {
            return Ok(json!({"run_id": run_id, "state": "starting-session", "total": total}));
        }
        let clock = Instant::now();
        while clock.elapsed() < RUN_BEGIN_CEILING {
            {
                let mut runs = self.runs.lock().unwrap();
                let track = runs.get_mut(&run_id).expect("the run's track");
                if let Some(e) = track.refused.take() {
                    runs.remove(&run_id);
                    return Err(ops::with_setups(e, Some(&project)));
                }
                if track.state != "pending" {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(json!({"run_id": run_id, "state": "running", "total": total}))
    }

    fn status_of(&self, run_id: &str) -> Option<(Value, usize, bool)> {
        let runs = self.runs.lock().unwrap();
        let t = runs.get(run_id)?;
        let state = match t.state.as_str() {
            "pending" => "running",
            s => s,
        };
        let done = t.outcomes.len();
        Some((
            json!({
                "run_id": run_id,
                "state": state,
                "done": done,
                "total": t.ids.len(),
                "current_check": if t.ended { None } else { t.ids.get(done).cloned() },
                "outcomes": t.outcomes,
            }),
            done,
            t.ended,
        ))
    }

    /// `run_status`: with `wait_s`, returns as soon as a check or the run ends, or at `wait_s`.
    async fn run_status(
        &self,
        run_id: &str,
        wait_s: Option<f64>,
        ctx: &RequestContext<RoleServer>,
    ) -> Result<Value, CliError> {
        let wait = match wait_s {
            None => 0.0,
            Some(w) if w.is_finite() && w > 0.0 && w <= WAIT_MAX_S => w,
            Some(w) => {
                return Err(bad(format!(
                    "wait_s {w} is not above 0 and at most {WAIT_MAX_S}, the most a tool call waits. Give a wait_s of at most {WAIT_MAX_S}"
                )));
            }
        };
        let Some((first, done, ended)) = self.status_of(run_id) else {
            let project = self.project()?;
            return ops::run_status_of_file(&project.root, run_id).ok_or_else(|| unknown_run(run_id));
        };
        if ended || wait == 0.0 {
            return Ok(first);
        }
        let mut progress = Progress::new(ctx);
        let clock = Instant::now();
        let mut last_tick = Instant::now();
        while clock.elapsed().as_secs_f64() < wait {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let (now, now_done, now_ended) = self.status_of(run_id).expect("a run's track stays");
            if now_done != done || now_ended {
                return Ok(now);
            }
            if last_tick.elapsed() >= PROGRESS_EVERY {
                last_tick = Instant::now();
                progress.tick(format!("{now_done} of {} done", now["total"])).await;
            }
            if ctx.ct.is_cancelled() {
                break;
            }
        }
        Ok(self.status_of(run_id).expect("a run's track stays").0)
    }

    /// `run_result`: the result of a run that ended, from its result file.
    fn run_result(&self, run_id: &str) -> Result<Value, CliError> {
        if let Some((_, _, false)) = self.status_of(run_id) {
            return Err(CliError::wrong(format!(
                "Run {run_id} has not ended, so it has no result yet. Call run_status with wait_s until its state is finished, then run_result"
            ))
            .tag("unknown-run"));
        }
        let project = self.project()?;
        ops::run_result(&project.root, run_id).ok_or_else(|| unknown_run(run_id))
    }
}

/// How long `session_start` waits for the session, 50 s (`AGT-03`). The framework's own tests set
/// `SHAKEDOWN_MCP_START_WAIT_S` to a shorter wait, to see a start outlast it without a slower
/// game.
fn start_wait_s() -> f64 {
    std::env::var("SHAKEDOWN_MCP_START_WAIT_S")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|w| *w > 0.0 && *w <= WAIT_MAX_S)
        .unwrap_or(WAIT_MAX_S)
}

fn unknown_run(run_id: &str) -> CliError {
    CliError::wrong(format!(
        "No run has the id {run_id}. Give the run_id that run_start returned"
    ))
    .tag("unknown-run")
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, CliError> + Send + 'static) -> Result<T, CliError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| CliError::framework(format!("The request failed: {e}.")))?
}

/// Follows a run on its own connection and records it in its track.
fn follow(
    runs: Arc<Mutex<HashMap<String, Track>>>,
    id: String,
    project: PathBuf,
    start: Option<crate::wire::SetupSpec>,
    request: Request,
) {
    let set = |f: &dyn Fn(&mut Track)| {
        if let Some(t) = runs.lock().unwrap().get_mut(&id) {
            f(t);
        }
    };
    let mut conn = match client::connect() {
        Ok(c) => c,
        Err(e) => {
            set(&|t| {
                t.refused = Some(e.clone());
                t.state = "stopped".into();
                t.ended = true;
            });
            return;
        }
    };
    let started_here = start.is_some();
    if let Some(spec) = start {
        let started = conn.request(&Request::SessionStart { project, spec }, |_| {});
        let ok = matches!(&started, Ok(Event::Session { session: Some(s) }) if s["state"] != "failed");
        if !ok {
            set(&|t| {
                t.state = "stopped".into();
                t.ended = true;
            });
            return;
        }
    }
    let answer = conn.request(&request, |event| match event {
        Event::RunStarted { .. } => set(&|t| t.state = "running".into()),
        Event::Check { line, .. } => set(&|t| t.outcomes.push(line.clone())),
        _ => {}
    });
    if started_here {
        let _ = conn.request(&Request::SessionStop, |_| {});
    }
    match answer {
        Ok(_) => set(&|t| {
            t.state = "finished".into();
            t.ended = true;
        }),
        Err(e) => set(&|t| {
            if t.state == "pending" {
                t.refused = Some(e.clone());
            }
            t.state = "stopped".into();
            t.ended = true;
        }),
    }
}

/// A result longer than a tool result may be goes to a file, and the result names it (`AGT-07`).
fn fit(value: Value, project: Option<&Path>, tool: &str) -> Value {
    let text = value.to_string();
    if text.len() <= RESULT_LIMIT {
        return value;
    }
    let dir = project
        .map(|p| p.join(".shakedown").join("results"))
        .unwrap_or_else(|| std::env::temp_dir().join("shakedown-results"));
    let file = dir.join(format!(
        "{tool}-{}-{}.json",
        shakedown_protocol::time::stamp(std::time::SystemTime::now()),
        std::process::id()
    ));
    let written = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&file, &text));
    match written {
        Ok(()) => json!({"result_path": file}),
        Err(_) => value,
    }
}

impl ServerHandler for Server {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("shakedown", shakedown_protocol::VERSION))
            .with_instructions("Shakedown runs checks of an Arma 3 mod in the running game. Call the guide tool first.")
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools.clone()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.iter().find(|t| t.name == name).cloned()
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let name = request.name.to_string();
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        let answer = self.call(&name, arguments, &context).await;
        let result = match answer {
            Ok(value) => {
                let root = self.project().ok().map(|p| p.root);
                CallToolResult::structured(fit(value, root.as_deref(), &name))
            }
            Err(e) => match e.mcp_code() {
                // A mistake the agent can fix: its code, what is wrong and what to do next (AGT-02).
                Some(code) => {
                    let message = e.message.trim_end_matches('.');
                    CallToolResult::error(vec![ContentBlock::text(format!("{code}: {message}."))])
                }
                None => return Err(ErrorData::internal_error(e.message, None)),
            },
        };
        Ok(CallToolResponse::Complete(result))
    }
}

/// `shakedown mcp`: serves the tools on stdio until the client closes it.
pub fn main(project: Option<PathBuf>) -> u8 {
    let rt = tokio::runtime::Runtime::new().expect("a tokio runtime");
    rt.block_on(async move {
        let server = Server::new(project);
        match server.serve(rmcp::transport::stdio()).await {
            Ok(service) => {
                let _ = service.waiting().await;
                0
            }
            Err(e) => {
                eprintln!("The MCP server could not start: {e}.");
                5
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agt_02_every_tool_of_the_contract_has_an_input_and_an_output_schema() {
        let tools = tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        for want in [
            "status",
            "session_start",
            "session_stop",
            "list_checks",
            "run_start",
            "run_status",
            "run_result",
            "reset",
            "reload",
            "exec",
            "wait_until",
            "expect",
            "inspect",
            "logs",
            "errors",
            "guide",
        ] {
            assert!(names.contains(&want), "{want} missing");
        }
        for t in &tools {
            let out = t.output_schema.as_ref().expect("an output schema");
            assert_eq!(out["type"], "object", "{}", t.name);
            // Every reference resolves inside the schema.
            let text = serde_json::to_string(&**out).unwrap();
            for r in text.split("\"$ref\":\"#/$defs/").skip(1) {
                let name = &r[..r.find('"').unwrap()];
                assert!(out["$defs"].get(name).is_some(), "{}: {name}", t.name);
            }
        }
    }

    #[test]
    fn agt_06_the_guide_is_under_8000_characters() {
        assert!(ops::GUIDE.chars().count() < 8000);
        for topic in [
            "session_start",
            "run_start",
            "run_result",
            "@check",
            "skd_fnc_waitUntil",
        ] {
            assert!(ops::GUIDE.contains(topic), "{topic}");
        }
    }

    #[test]
    fn agt_06_the_docs_hold_the_text_of_the_guide() {
        let doc = include_str!("../../../../docs/agent-tools.md");
        assert!(doc.contains(ops::GUIDE), "docs/agent-tools.md does not hold guide.txt");
    }

    #[test]
    fn agt_07_a_long_result_goes_to_a_file() {
        let long = json!({"lines": vec!["x".repeat(100); 300]});
        let out = fit(long.clone(), None, "logs");
        let path = out["result_path"].as_str().expect("a path");
        assert_eq!(std::fs::read_to_string(path).unwrap(), long.to_string());
        std::fs::remove_file(path).unwrap();
        assert_eq!(fit(json!({"a": 1}), None, "logs"), json!({"a": 1}));
    }

    #[test]
    fn agt_03_a_ceiling_above_50_is_refused_with_the_most() {
        let e = ceiling(Some(55.0), 10.0).unwrap_err();
        assert_eq!(e.mcp_code(), Some("bad-arguments"));
        assert!(e.message.contains("at most 50"));
        assert_eq!(ceiling(None, 10.0).unwrap(), 10.0);
    }
}
