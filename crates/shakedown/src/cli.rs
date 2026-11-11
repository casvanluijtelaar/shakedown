//! The `shakedown` command line (`contract/cli.md`).

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::checks;
use crate::client;
use crate::error::CliError;
use crate::ops;
use crate::wire::{Event, Request};

#[derive(Parser)]
#[command(name = "shakedown", version, about = "Test automation for Arma 3 mods.")]
struct Cli {
    /// Use this project folder.
    #[arg(long, global = true)]
    project: Option<PathBuf>,
    /// Print one JSON document on stdout and nothing else.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lists checks from the files, with no runner (RUN-02).
    List {
        ids: Vec<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    /// Runs the selection against the session; with --setup and no session, starts one and stops it after (RUN-04). `run stop` stops the active run (RUN-05).
    Run {
        ids: Vec<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        setup: Option<String>,
    },
    /// Starts or stops the session.
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// The runner, the session and its machines, and the active run (INS-02, INS-04).
    Status,
    /// Resets the session in place (RST-03).
    Reset,
    /// Reloads changed mod code: the files given, or what changed under the watched sources (RLD-01).
    Reload { files: Vec<PathBuf> },
    /// Runs SQF on a machine or a group and prints the value and any script errors (CTL-01, CTL-05).
    Exec {
        #[arg(long)]
        machine: String,
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        code: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
        /// The arguments, a JSON array the code gets as _this.
        #[arg(long)]
        args: Option<String>,
        #[arg(long, default_value_t = 10.0)]
        ceiling: f64,
    },
    /// Waits for a condition on a machine (OBS-01).
    Wait {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        condition: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        args: Option<String>,
        #[arg(long, default_value_t = 10.0)]
        ceiling: f64,
    },
    /// Tries an assertion live, as skd_fnc_expectOn would, without a check (OBS-06).
    Expect {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        op: String,
        /// The expected value, as JSON.
        #[arg(long)]
        expected: String,
        #[arg(long)]
        args: Option<String>,
        #[arg(long, default_value_t = 10.0)]
        ceiling: f64,
    },
    /// A compact state summary of a machine (INS-01).
    Inspect {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        near: Option<String>,
        #[arg(long, default_value_t = 50.0)]
        radius: f64,
    },
    /// A machine's log lines, redacted (LOG-03).
    Logs {
        #[arg(long)]
        machine: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        check: Option<String>,
        #[arg(long)]
        grep: Option<String>,
        #[arg(long, default_value_t = 200)]
        last: usize,
    },
    /// Script errors caught in the session (ERR-05).
    Errors {
        #[arg(long)]
        machine: Option<String>,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        check: Option<String>,
    },
    /// How to use Shakedown and write a check, in short (AGT-06).
    Guide,
    /// The framework version.
    Version,
    /// Runs the MCP server on stdio (AGT-01).
    Mcp,
    /// Runs the runner. Started by the other commands; not for direct use.
    #[command(hide = true)]
    Runner,
}

#[derive(Subcommand)]
enum SessionAction {
    /// Starts a session and returns when it is ready or failed (SES-01).
    Start {
        #[arg(long)]
        setup: String,
    },
    /// Stops the session and returns when nothing of it runs (SES-07).
    Stop,
}

pub fn main() -> u8 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let code = if e.use_stderr() { 3 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    match cli.command {
        Command::Runner => return crate::runner::main(),
        Command::Mcp => return crate::mcp::main(cli.project.clone()),
        _ => {}
    }
    match dispatch(&cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e.message);
            e.code
        }
    }
}

/// A JSON argument of the command line.
fn json_arg(name: &str, text: Option<&str>) -> Result<Value, CliError> {
    match text {
        None => Ok(Value::Null),
        Some(t) => serde_json::from_str(t).map_err(|e| {
            CliError::wrong(format!(
                "--{name} is not JSON: {e}. Give it as JSON, such as [1, \"a\"]."
            ))
            .tag("bad-arguments")
        }),
    }
}

fn dispatch(cli: &Cli) -> Result<u8, CliError> {
    let out = Out { json: cli.json };
    let given = cli.project.as_deref();
    // A command that asks the session names the project's setups when none runs (US-047).
    let asked = |request: Request| -> Result<Value, CliError> {
        let project = ops::project(given).ok();
        let mut conn = client::connect()?;
        ops::value(&mut conn, &request, |e| out.progress(e)).map_err(|e| ops::with_setups(e, project.as_ref()))
    };
    match &cli.command {
        Command::Version => {
            if cli.json {
                out.json(&json!({"version": shakedown_protocol::VERSION}));
            } else {
                println!("shakedown {}", shakedown_protocol::VERSION);
            }
            Ok(0)
        }
        Command::Guide => {
            if cli.json {
                out.json(&json!({"text": ops::GUIDE}));
            } else {
                print!("{}", ops::GUIDE);
            }
            Ok(0)
        }
        Command::List { ids, tags } => {
            let project = ops::project(given)?;
            let listed = ops::list(&project, ids, tags)?;
            if cli.json {
                out.json(&listed);
            } else {
                for c in listed["checks"].as_array().into_iter().flatten() {
                    let id = c["id"].as_str().unwrap_or("?");
                    match (c["name"].as_str(), c["error"].as_str()) {
                        (Some(name), _) => {
                            let tags: Vec<&str> = c["tags"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|t| t.as_str())
                                .collect();
                            let tags = if tags.is_empty() {
                                String::new()
                            } else {
                                format!("  [{}]", tags.join(" "))
                            };
                            println!("{id}  {name}{tags}");
                        }
                        (None, Some(e)) => println!("{id}  (header error: {e})"),
                        (None, None) => println!("{id}"),
                    }
                }
            }
            Ok(0)
        }
        Command::Session {
            action: SessionAction::Start { setup },
        } => {
            let project = ops::project(given)?;
            let spec = ops::spec(&project, setup)?;
            let mut conn = client::connect()?;
            let last = conn.request(
                &Request::SessionStart {
                    project: project.root.clone(),
                    spec,
                },
                |e| out.progress(e),
            )?;
            let session = ops::session_of(last);
            if cli.json {
                out.json(&session);
            }
            if session["state"] == "failed" {
                // A start that failed is the session in state `failed`, with exit code 4 (`SES-04`).
                return Err(crate::runner::session::start_failed(&session));
            }
            if !cli.json {
                println!(
                    "session {} ready (setup {})",
                    session["id"].as_str().unwrap_or("?"),
                    setup
                );
            }
            Ok(0)
        }
        Command::Session {
            action: SessionAction::Stop,
        } => {
            let mut conn = client::connect()?;
            let last = conn.request(&Request::SessionStop, |e| out.progress(e))?;
            let session = ops::session_of(last);
            if cli.json {
                out.json(&session);
            } else if session["state"] == "failed" {
                println!(
                    "the failed start of session {} is cleared",
                    session["id"].as_str().unwrap_or("?")
                );
            } else {
                println!("session {} stopped", session["id"].as_str().unwrap_or("?"));
            }
            Ok(0)
        }
        // `run stop` stops the active run (RUN-05); `stop` alone is never read as a check id.
        Command::Run { ids, tags, setup } if ids == &["stop"] && tags.is_empty() && setup.is_none() => {
            run_stop(cli, &out)
        }
        Command::Run { ids, tags, setup } => run(cli, &out, ids, tags, setup.as_deref()),
        Command::Status => {
            let status = ops::status()?;
            if cli.json {
                out.json(&status);
            } else {
                print_status(&status);
            }
            Ok(0)
        }
        Command::Reset => {
            let value = asked(Request::Reset)?;
            if cli.json {
                out.json(&value);
            } else {
                println!(
                    "reset {} in {} s",
                    value["state"].as_str().unwrap_or("?"),
                    value["duration_s"]
                );
                if let Some(m) = value["mismatches"].as_array().filter(|m| !m.is_empty()) {
                    for x in m {
                        println!("  {}", x.as_str().unwrap_or_default());
                    }
                }
            }
            if value["state"] == "done" {
                Ok(0)
            } else {
                if !cli.json {
                    eprintln!("The reset failed and left the session broken. Stop it with `shakedown session stop`.");
                }
                Ok(4)
            }
        }
        Command::Reload { files } => {
            let project = ops::project(given)?;
            let value = asked(ops::reload_request(&project, files, None)?)?;
            if cli.json {
                out.json(&value);
            } else {
                print_reload(&value);
            }
            if value["state"] == "failed" {
                return Err(CliError::not_now(format!(
                    "The reload failed at its {} on {}: {}. The session is broken; `shakedown reload` restarts its game, or stop it with `shakedown session stop`.",
                    value["reason"]["step"].as_str().unwrap_or("?"),
                    value["reason"]["machine"].as_str().unwrap_or("?"),
                    value["reason"]["message"].as_str().unwrap_or("?"),
                )));
            }
            Ok(0)
        }
        Command::Exec {
            machine,
            code,
            file,
            args,
            ceiling,
        } => {
            let code = match (code, file) {
                (Some(c), _) => c.clone(),
                (None, Some(f)) => std::fs::read_to_string(f).map_err(|e| {
                    CliError::wrong(format!("Cannot read {}: {e}. Give a file that exists.", f.display()))
                        .tag("bad-arguments")
                })?,
                (None, None) => unreachable!("clap requires --code or --file"),
            };
            let value = asked(Request::Exec {
                machine: machine.clone(),
                code,
                args: json_arg("args", args.as_deref())?,
                ceiling: *ceiling,
                source: "cli".into(),
            })?;
            if cli.json {
                out.json(&value);
            } else {
                for (name, r) in value["results"].as_object().into_iter().flatten() {
                    println!(
                        "{name}  {}  {}  ({} s)",
                        r["state"].as_str().unwrap_or("?"),
                        r["value"],
                        r["duration_s"]
                    );
                }
                print_errors(&value["errors"]);
            }
            Ok(0)
        }
        Command::Wait {
            machine,
            condition,
            description,
            args,
            ceiling,
        } => {
            let value = asked(Request::Wait {
                machine: machine.clone(),
                condition: condition.clone(),
                description: description.clone(),
                args: json_arg("args", args.as_deref())?,
                ceiling: *ceiling,
            })?;
            if cli.json {
                out.json(&value);
            } else {
                match (value["met"] == true, value["error"].as_str()) {
                    (true, _) => println!("met after {} s", value["waited_s"]),
                    (false, Some(e)) => println!("not met: {e} (after {} s)", value["waited_s"]),
                    (false, None) => println!("not met within its ceiling of {ceiling} s"),
                }
            }
            Ok(0)
        }
        Command::Expect {
            machine,
            value,
            op,
            expected,
            args,
            ceiling,
        } => {
            let answer = asked(Request::Expect {
                machine: machine.clone(),
                value: value.clone(),
                op: op.clone(),
                expected: json_arg("expected", Some(expected))?,
                args: json_arg("args", args.as_deref())?,
                ceiling: *ceiling,
            })?;
            if cli.json {
                out.json(&answer);
            } else {
                println!(
                    "{}: last read {} after {} s",
                    if answer["passed"] == true {
                        "held"
                    } else {
                        "did not hold"
                    },
                    answer["last_value"],
                    answer["waited_s"]
                );
            }
            Ok(0)
        }
        Command::Inspect { machine, near, radius } => {
            let value = asked(Request::Inspect {
                machine: machine.clone(),
                near: near.clone(),
                radius: *radius,
            })?;
            if cli.json {
                out.json(&value);
            } else {
                print_inspect(&value);
            }
            Ok(0)
        }
        Command::Errors { machine, run, check } => {
            let value = asked(Request::Errors {
                machine: machine.clone(),
                run: run.clone(),
                check: check.clone(),
            })?;
            if cli.json {
                out.json(&value);
            } else {
                print_errors(&value["errors"]);
                if value["errors"].as_array().is_none_or(|e| e.is_empty()) {
                    println!("no script errors");
                }
            }
            Ok(0)
        }
        Command::Logs {
            machine,
            run,
            check,
            grep,
            last,
        } => {
            let project = ops::project(given)?;
            let mut conn = client::connect()?;
            let request = Request::Logs {
                project: project.root.clone(),
                machine: machine.clone(),
                run: run.clone(),
                check: check.clone(),
                grep: grep.clone(),
                last: *last,
            };
            let value = ops::value(&mut conn, &request, |e| out.progress(e))?;
            if cli.json {
                out.json(&value);
            } else {
                for line in value["lines"].as_array().into_iter().flatten() {
                    println!("{}", line.as_str().unwrap_or_default());
                }
                if let Some(path) = value["path"].as_str() {
                    eprintln!("Every line is in {path}.");
                }
            }
            Ok(0)
        }
        Command::Runner | Command::Mcp => unreachable!("handled before dispatch"),
    }
}

fn run(cli: &Cli, out: &Out, ids: &[String], tags: &[String], setup: Option<&str>) -> Result<u8, CliError> {
    let project = ops::project(cli.project.as_deref())?;
    let all = checks::discover(&project.checks)?;
    let selected = checks::select(&all, ids, tags)?;
    let spec = setup.map(|s| ops::spec(&project, s)).transpose()?;
    let mut conn = client::connect()?;
    // The run's id is handed out before any session start, so Ctrl+C or Ctrl+Break stops the run
    // from the moment the command starts (RUN-05).
    let run_id = ops::value(&mut conn, &Request::ReserveRun, |_| {})?["run_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    stop_on_console_event(&run_id);
    let running = ops::session_of(conn.request(&Request::SessionInfo, |_| {})?);
    let mut started_here = false;
    match (&spec, running.is_null()) {
        (Some(spec), true) => {
            let started = ops::session_of(conn.request(
                &Request::SessionStart {
                    project: project.root.clone(),
                    spec: spec.clone(),
                },
                |e| out.progress(e),
            )?);
            if started["state"] == "failed" {
                return Err(crate::runner::session::start_failed(&started));
            }
            started_here = true;
        }
        (Some(spec), false) if running["setup"] != spec.setup.name.as_str() => {
            return Err(other_setup(&running, &spec.setup.name));
        }
        (None, true) => {
            return Err(ops::with_setups(
                CliError::not_now("No session runs.").tag("no-session"),
                Some(&project),
            ));
        }
        _ => {}
    }
    let request = Request::Run {
        project: project.root.clone(),
        setups: project.setups.keys().cloned().collect(),
        checks: selected,
        ids: ids.to_vec(),
        tags: tags.to_vec(),
        run_id: Some(run_id),
    };
    let result = conn.request(&request, |e| out.progress(e));
    // A run that started its own session stops it, whatever the outcomes (RUN-04).
    if started_here {
        conn.request(&Request::SessionStop, |e| out.progress(e))?;
    }
    let Event::RunDone { result } = result? else {
        return Err(CliError::framework("The runner ended the run without a result."));
    };
    let summary = &result["summary"];
    let stopped = result["state"] == "stopped";
    if cli.json {
        out.json(&result);
    } else {
        if stopped {
            println!("Run stopped: {}.", stopped_reason(&result));
        }
        println!(
            "{} passed, {} failed, {} invalid, {} not run. Result: {}",
            summary["pass"],
            summary["fail"],
            summary["invalid"],
            summary["not_run"],
            result["result_file"].as_str().unwrap_or("?")
        );
    }
    let count = |k: &str| summary[k].as_u64().unwrap_or(0);
    // A stopped run did not reach a verdict on every check: 2 unless a check failed (RUN-05).
    Ok(if count("fail") > 0 {
        1
    } else if count("invalid") > 0 || stopped {
        2
    } else {
        0
    })
}

/// The `stopped_reason` of a run, read from its result file.
fn stopped_reason(result: &Value) -> String {
    result["result_file"]
        .as_str()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|r| r["run"]["stopped_reason"].as_str().map(String::from))
        .unwrap_or_else(|| "?".into())
}

/// The run a Ctrl+C or Ctrl+Break to this command stops.
static STOPPING: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Has a Ctrl+C or a Ctrl+Break console event stop the run `run_id` as `run stop` does, on a
/// connection of its own, while the command goes on waiting for the run's result (`RUN-05`).
fn stop_on_console_event(run_id: &str) {
    *STOPPING.lock().unwrap() = Some(run_id.to_string());
    crate::win::on_stop_keys(|event| {
        if event > 1 {
            return;
        }
        let Some(id) = STOPPING.lock().unwrap().clone() else {
            return;
        };
        eprintln!("Stopping run {id}; its result file comes next.");
        std::thread::spawn(move || {
            if let Ok(Some(mut conn)) = client::existing() {
                let _ = conn.request(&Request::RunStop { run_id: Some(id) }, |_| {});
            }
        });
    });
}

/// `run stop` (`RUN-05`): stops the active run and prints its outcomes once its result file is
/// written, in the shape of `run_stop`.
fn run_stop(cli: &Cli, out: &Out) -> Result<u8, CliError> {
    let no_run = || CliError::not_now("No run is active, so there is nothing to stop.").tag("unknown-run");
    let mut conn = client::existing()?.ok_or_else(no_run)?;
    let answer = ops::value(&mut conn, &Request::RunStop { run_id: None }, |e| out.progress(e))?;
    let run_id = answer["run_id"].as_str().unwrap_or("?");
    let status = answer["result_file"]
        .as_str()
        .and_then(|f| ops::run_status_of_result(std::path::Path::new(f)))
        .unwrap_or_else(|| {
            json!({"run_id": run_id, "state": "stopped", "done": 0, "total": 0, "current_check": null, "outcomes": []})
        });
    if cli.json {
        out.json(&status);
    } else {
        println!(
            "run {run_id} {}: {} of {} checks executed. Result: {}",
            status["state"].as_str().unwrap_or("?"),
            status["done"],
            status["total"],
            answer["result_file"].as_str().unwrap_or("none, the run had not begun")
        );
    }
    Ok(0)
}

/// The refusal of a run with `--setup` naming another setup than the running session's.
pub fn other_setup(running: &Value, setup: &str) -> CliError {
    CliError::not_now(format!(
        "Session {} of setup {} runs, not one of setup {setup}. Stop it with `shakedown session stop`, or run without --setup against it.",
        running["id"].as_str().unwrap_or("?"),
        running["setup"].as_str().unwrap_or("?"),
    ))
    .tag("session-exists")
}

/// A reload for a person: its level, state and time, each file with its level and reason, and the
/// machines it reached (`RLD-03`).
pub fn print_reload(v: &Value) {
    println!(
        "reload {} {}: level {} in {} s",
        v["id"].as_str().unwrap_or("?"),
        v["state"].as_str().unwrap_or("?"),
        v["level"].as_str().unwrap_or("?"),
        v["duration_s"]
    );
    for f in v["files"].as_array().into_iter().flatten() {
        println!(
            "  {}  {}: {}",
            f["level"].as_str().unwrap_or("?"),
            f["path"].as_str().unwrap_or("?"),
            f["reason"].as_str().unwrap_or("")
        );
    }
    if let Some(m) = v["machines"].as_array().filter(|m| !m.is_empty()) {
        let names: Vec<&str> = m.iter().filter_map(|x| x.as_str()).collect();
        println!("  reached {}", names.join(", "));
    }
}

fn print_errors(errors: &Value) {
    for e in errors.as_array().into_iter().flatten() {
        println!(
            "error on {} at {}: {} ({}, line {}){}",
            e["machine"].as_str().unwrap_or("?"),
            e["at"]["utc"].as_str().unwrap_or("?"),
            e["text"].as_str().unwrap_or("?"),
            e["file"].as_str().filter(|f| !f.is_empty()).unwrap_or("no file"),
            e["line"],
            e["check_id"]
                .as_str()
                .map(|c| format!(" in check {c}"))
                .unwrap_or_else(|| " outside any check".into())
        );
    }
}

fn print_inspect(v: &Value) {
    let m = &v["machine"];
    println!(
        "{} ({}) {}, start {}",
        m["name"].as_str().unwrap_or("?"),
        m["kind"].as_str().unwrap_or("?"),
        m["state"].as_str().unwrap_or("?"),
        m["start"]
    );
    if v["mission"].is_null() {
        return;
    }
    println!(
        "mission {} (start {}), time {} s, tick {} s, {} fps, frame {}",
        v["mission"].as_str().unwrap_or("?"),
        v["mission_start"],
        v["time"],
        v["tick"],
        v["fps"],
        v["frame"]
    );
    if let Some(check) = v["check_id"].as_str() {
        println!("run {} check {check} step {}", v["run_id"], v["step_path"]);
    }
    if !v["player"].is_null() {
        println!("player: {}", v["player"]);
    }
    for o in v["nearby"].as_array().into_iter().flatten() {
        println!(
            "  near: {} {} at {} m",
            o["class"].as_str().unwrap_or("?"),
            o["net_id"].as_str().unwrap_or("?"),
            o["distance"]
        );
    }
    for s in v["scripts"].as_array().into_iter().flatten() {
        println!(
            "  script: {} {}:{}",
            s["name"].as_str().unwrap_or("?"),
            s["file"].as_str().unwrap_or(""),
            s["line"]
        );
    }
    print_errors(&v["errors"]);
}

/// `status` for a person: the runner, the session and one line per machine.
fn print_status(status: &Value) {
    println!(
        "runner: {}{}",
        status["runner"].as_str().unwrap_or("?"),
        status["runner_pid"]
            .as_u64()
            .map(|p| format!(" (process {p})"))
            .unwrap_or_default()
    );
    let session = &status["session"];
    if session.is_null() {
        println!("session: none");
        return;
    }
    println!(
        "session: {} of setup {}, {}",
        session["id"].as_str().unwrap_or("?"),
        session["setup"].as_str().unwrap_or("?"),
        session["state"].as_str().unwrap_or("?")
    );
    if let Some(reason) = session["reason"]["message"].as_str() {
        println!("  reason: {reason}");
    }
    for m in session["machines"].as_array().into_iter().flatten() {
        println!(
            "  {:<7} {:<9} {:<10} start {} process {} profile {}",
            m["name"].as_str().unwrap_or("?"),
            m["kind"].as_str().unwrap_or("?"),
            m["state"].as_str().unwrap_or("?"),
            m["start"],
            m["pid"],
            m["profile"].as_str().unwrap_or("?")
        );
    }
    if let Some(run) = status["run"].as_object() {
        println!(
            "run: {} {} of {} done, now {}",
            run["id"].as_str().unwrap_or("?"),
            run["done"],
            run["total"],
            run["current_check"]
        );
    }
}

struct Out {
    json: bool,
}

impl Out {
    fn json(&self, value: &Value) {
        println!("{}", serde_json::to_string_pretty(value).unwrap());
    }

    /// Progress and check lines go to stdout for a person, and to stderr with --json.
    fn progress(&self, event: &Event) {
        let text = match event {
            Event::Progress { message } => message.clone(),
            Event::Check { text, .. } => text.clone(),
            _ => return,
        };
        if self.json {
            eprintln!("{text}");
        } else {
            println!("{text}");
        }
    }
}
