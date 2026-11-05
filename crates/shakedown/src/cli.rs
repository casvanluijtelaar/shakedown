//! The `shakedown` command line (`contract/cli.md`).

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::checks;
use crate::client;
use crate::config;
use crate::error::CliError;
use crate::wire::{Event, Request, SetupSpec};

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
    /// Runs the selection against the session; with --setup and no session, starts one and stops it after (RUN-04).
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
    /// The framework version.
    Version,
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
    if let Command::Runner = cli.command {
        return crate::runner::main();
    }
    match dispatch(&cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e.message);
            e.code
        }
    }
}

fn dispatch(cli: &Cli) -> Result<u8, CliError> {
    let out = Out { json: cli.json };
    match &cli.command {
        Command::Version => {
            if cli.json {
                out.json(&json!({"version": shakedown_protocol::VERSION}));
            } else {
                println!("shakedown {}", shakedown_protocol::VERSION);
            }
            Ok(0)
        }
        Command::List { ids, tags } => {
            let project = project(cli)?;
            let all = checks::discover(&project.checks)?;
            let selected = checks::select(&all, ids, tags)?;
            let listed: Vec<checks::Listed> = selected.iter().map(checks::read_listed).collect();
            if cli.json {
                out.json(&json!({"checks": listed}));
            } else {
                for c in &listed {
                    match (&c.name, &c.error) {
                        (Some(name), _) => {
                            let tags = if c.tags.is_empty() {
                                String::new()
                            } else {
                                format!("  [{}]", c.tags.join(" "))
                            };
                            println!("{}  {name}{tags}", c.id);
                        }
                        (None, Some(e)) => println!("{}  (header error: {e})", c.id),
                        (None, None) => println!("{}", c.id),
                    }
                }
            }
            Ok(0)
        }
        Command::Session {
            action: SessionAction::Start { setup },
        } => {
            let project = project(cli)?;
            let spec = spec(&project, setup)?;
            let mut conn = client::connect()?;
            let last = conn.request(
                &Request::SessionStart {
                    project: project.root.clone(),
                    spec,
                },
                |e| out.progress(e),
            )?;
            let session = session_of(last);
            if cli.json {
                out.json(&session);
            } else {
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
            let session = session_of(last);
            if cli.json {
                out.json(&session);
            } else {
                println!("session {} stopped", session["id"].as_str().unwrap_or("?"));
            }
            Ok(0)
        }
        Command::Run { ids, tags, setup } => run(cli, &out, ids, tags, setup.as_deref()),
        Command::Runner => unreachable!("handled before dispatch"),
    }
}

fn run(cli: &Cli, out: &Out, ids: &[String], tags: &[String], setup: Option<&str>) -> Result<u8, CliError> {
    let project = project(cli)?;
    let all = checks::discover(&project.checks)?;
    let selected = checks::select(&all, ids, tags)?;
    let spec = setup.map(|s| spec(&project, s)).transpose()?;
    let mut conn = client::connect()?;
    let running = session_of(conn.request(&Request::SessionInfo, |_| {})?);
    let mut started_here = false;
    match (&spec, running.is_null()) {
        (Some(spec), true) => {
            conn.request(
                &Request::SessionStart {
                    project: project.root.clone(),
                    spec: spec.clone(),
                },
                |e| out.progress(e),
            )?;
            started_here = true;
        }
        (Some(spec), false) if running["setup"] != spec.setup.name.as_str() => {
            return Err(CliError::not_now(format!(
                "Session {} of setup {} runs, not one of setup {}. Stop it with `shakedown session stop`, or run without --setup against it.",
                running["id"].as_str().unwrap_or("?"),
                running["setup"].as_str().unwrap_or("?"),
                spec.setup.name
            )));
        }
        (None, true) => {
            return Err(CliError::not_now(
                "No session runs. Give `run` a --setup to start one for this run, or start one with `shakedown session start --setup <name>`.",
            ));
        }
        _ => {}
    }
    let request = Request::Run {
        project: project.root.clone(),
        checks: selected,
        ids: ids.to_vec(),
        tags: tags.to_vec(),
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
    if cli.json {
        out.json(&result);
    } else {
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
    Ok(if count("fail") > 0 {
        1
    } else if count("invalid") > 0 {
        2
    } else {
        0
    })
}

fn project(cli: &Cli) -> Result<config::Project, CliError> {
    let cwd =
        std::env::current_dir().map_err(|e| CliError::framework(format!("Cannot read the current folder: {e}.")))?;
    let root = config::find_root(cli.project.as_deref(), &cwd)?;
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

fn spec(project: &config::Project, name: &str) -> Result<SetupSpec, CliError> {
    let setup = project.setup(name)?.clone();
    let mods = setup.mods.iter().map(|m| project.mods[m].clone()).collect();
    Ok(SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
    })
}

fn session_of(event: Event) -> Value {
    match event {
        Event::Session { session } => session.unwrap_or(Value::Null),
        _ => Value::Null,
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
