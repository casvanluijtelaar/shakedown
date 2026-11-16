//! M13's tests: one command runs a selection in every setup it needs (`US-074`), one setup's
//! session fails to start (`US-075`), a check repeated with the spread of its outcomes (`US-076`),
//! the session breaks during repeats (`US-077`), slow steps and slow checks flagged (`US-078`),
//! and a slow step threshold that is not a number of seconds (`US-080`). They drive the installed
//! `shakedown`, the fixture `@skd_fixture` and the game, so they run only on the test machine, by
//! hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m13 -- --ignored --nocapture --test-threads=1
//!
//! Every game process a test sees is ended by its id when the test ends, passed or failed.

#![cfg(windows)]

mod support;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn install() -> PathBuf {
    repo().join("target").join("shakedown")
}

fn exe() -> PathBuf {
    let exe = install().join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// A project in the temp folder with `setups` (the TOML of each, in order), the mod `fixture`, and
/// `checks` (id and text).
fn project(name: &str, setups: &str, checks: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m13-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        "game = '{GAME}'\ntests = \"tests\"\n\n[mods.fixture]\npath = '{GAME}\\@skd_fixture'\npatch = \"skdfx_main\"\n\n[mods.gone]\npath = '{}'\n\n{setups}",
        dir.join("no-such-mod").display()
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

fn steam() {
    let steam = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&steam.stdout).contains("steam.exe"),
        "Steam must run before a game start"
    );
}

fn runs(pid: u64) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
}

/// The ids of every running game process.
fn game_pids() -> Vec<u64> {
    let mut pids = Vec::new();
    for image in ["arma3_x64.exe", "arma3server_x64.exe"] {
        let out = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {image}"), "/FO", "CSV", "/NH"])
            .output()
            .unwrap();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(pid) = line.split(',').nth(1).and_then(|p| p.trim_matches('"').parse().ok()) {
                pids.push(pid);
            }
        }
    }
    pids
}

/// Every game process of the project's sessions the test saw; ends those still running by id when
/// dropped, after a `session stop`, also when a test fails.
struct Seen {
    dir: PathBuf,
    pids: Arc<Mutex<Vec<u64>>>,
}

impl Seen {
    fn new(dir: &Path) -> Seen {
        Seen {
            dir: dir.to_path_buf(),
            pids: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Records the machines `status` shows; the session's setup and the run, if any.
    fn see(&self) -> Value {
        let (_, s, _) = cmd(&self.dir, &["status"]);
        let mut pids = self.pids.lock().unwrap();
        for m in s["session"]["machines"].as_array().into_iter().flatten() {
            if let Some(pid) = m["pid"].as_u64()
                && !pids.contains(&pid)
            {
                pids.push(pid);
            }
        }
        s
    }
}

impl Drop for Seen {
    fn drop(&mut self) {
        let _ = cmd(&self.dir, &["session", "stop"]);
        for pid in self.pids.lock().unwrap().iter() {
            if runs(*pid) {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

/// Runs `shakedown <args>` in the project while watching `status` every second, so every game
/// process it starts is seen; calls `during` with each status. The exit code, stdout and stderr.
fn watched(seen: &Seen, args: &[&str], ceiling: Duration, during: &mut dyn FnMut(&Value)) -> (i32, String, String) {
    let mut child = Command::new(exe())
        .args(args)
        .current_dir(&seen.dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let clock = Instant::now();
    let code = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status.code().unwrap_or(-1);
        }
        assert!(
            clock.elapsed() < ceiling,
            "shakedown {} ran past {ceiling:?}",
            args.join(" ")
        );
        let status = seen.see();
        during(&status);
        std::thread::sleep(Duration::from_secs(1));
    };
    let (out, err) = (out.join().unwrap(), err.join().unwrap());
    eprintln!("$ shakedown {} -> {code}\n{out}{err}", args.join(" "));
    (code, out, err)
}

fn schema_errors(result: &Value) -> Vec<String> {
    let schema: Value =
        serde_json::from_str(&std::fs::read_to_string(repo().join("spec/contract/result.schema.json")).unwrap())
            .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    validator.iter_errors(result).map(|e| e.to_string()).collect()
}

/// Every result file of the project, oldest run first, each valid against the schema.
fn results(dir: &Path) -> Vec<Value> {
    let mut folders: Vec<PathBuf> = std::fs::read_dir(dir.join(".shakedown").join("runs"))
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    // Run ids are `r-<time>-<n>`; the counter orders runs within one runner.
    folders.sort_by_key(|f| {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let n: u64 = name.rsplit('-').next().and_then(|n| n.parse().ok()).unwrap_or(0);
        (name[..name.len().min(17)].to_string(), n)
    });
    folders
        .iter()
        .map(|f| {
            let r: Value = serde_json::from_str(&std::fs::read_to_string(f.join("result.json")).unwrap()).unwrap();
            let errors = schema_errors(&r);
            assert!(errors.is_empty(), "{}: {errors:?}", f.display());
            r
        })
        .collect()
}

/// The result file a run's text output names on its last `Result:` line.
fn result_named(out: &str) -> Value {
    let path = out
        .lines()
        .filter_map(|l| l.split("Result: ").nth(1))
        .next_back()
        .unwrap_or_else(|| panic!("no result file in {out}"));
    let r: Value = serde_json::from_str(&std::fs::read_to_string(path.trim()).unwrap()).unwrap();
    let errors = schema_errors(&r);
    assert!(errors.is_empty(), "{errors:?}");
    r
}

/// After a test: no session, and no game process the test saw still runs.
fn nothing_left(seen: &Seen) {
    let (_, s, _) = cmd(&seen.dir, &["status"]);
    assert!(s["session"].is_null(), "a session is left: {s}");
    let left: Vec<u64> = seen.pids.lock().unwrap().iter().copied().filter(|p| runs(*p)).collect();
    assert!(left.is_empty(), "game processes left: {left:?}");
}

const PASSES: &str = "[1, \"==\", 1, \"one is one\"] call skd_fnc_expect;\n";

#[test]
#[ignore = "drives the game on the test machine"]
fn us_074_one_command_runs_every_setup_and_the_agent_tool_follows_it() {
    let every = format!("// @check US-074 runs in every setup\n// @tags m00\n{PASSES}");
    let two = format!("// @check US-074 runs in two setups\n// @setups solo-fx duo\n// @tags m00\n{PASSES}");
    let ded = format!("// @check US-074 runs on the dedicated server only\n// @setups duo\n// @tags m01\n{PASSES}");
    // Declared out of name order, so the run's order is the file's.
    let setups = "[setups.solo]\nmode = \"singleplayer\"\nmods = []\n\n[setups.solo-fx]\nmode = \"singleplayer\"\nmods = [\"fixture\"]\n\n[setups.duo]\nmode = \"dedicated\"\nclients = 1\nmods = [\"fixture\"]\n";
    let dir = project("each", setups, &[("every", &every), ("two", &two), ("ded", &ded)]);
    let seen = Seen::new(&dir);
    steam();

    // US-074 1 to 3.
    let mut setups_seen: Vec<String> = Vec::new();
    let (code, out, _) = watched(&seen, &["run", "--each-setup"], Duration::from_secs(900), &mut |s| {
        if let Some(setup) = s["session"]["setup"].as_str()
            && setups_seen.last().map(String::as_str) != Some(setup)
        {
            setups_seen.push(setup.to_string());
        }
        let sessions = s["session"].is_object() as usize;
        assert!(sessions <= 1);
    });
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        setups_seen,
        ["solo", "solo-fx", "duo"],
        "one session at a time, in the file's order"
    );
    let setup_lines: Vec<&str> = out.lines().filter(|l| l.starts_with("setup ")).collect();
    assert_eq!(setup_lines.len(), 3, "{out}");
    assert!(setup_lines[0].starts_with("setup solo:") && setup_lines[2].starts_with("setup duo:"));
    let check_lines = out.lines().filter(|l| l.starts_with("pass ")).count();
    assert_eq!(check_lines, 6, "one line per check execution: {out}");
    let all = results(&dir);
    assert_eq!(all.len(), 3);
    let ran: Vec<(String, Vec<String>)> = all
        .iter()
        .map(|r| {
            (
                r["session"]["setup"]["name"].as_str().unwrap().to_string(),
                r["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["check_id"].as_str().unwrap().to_string())
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        ran,
        vec![
            ("solo".to_string(), vec!["every".to_string()]),
            ("solo-fx".to_string(), vec!["every".to_string(), "two".to_string()]),
            (
                "duo".to_string(),
                vec!["ded".to_string(), "every".to_string(), "two".to_string()]
            ),
        ]
    );
    for r in &all {
        assert!(r["not_run"].as_array().unwrap().is_empty(), "{r:#}");
    }
    nothing_left(&seen);

    // US-074 4 and 5: through MCP, a selection by tag starts only the setup it names.
    let mut mcp = support::Mcp::start(&exe(), &dir, &[]);
    let started = mcp
        .call("run_start", json!({"tags": ["m01"], "each_setup": true}))
        .data();
    eprintln!("run_start: {started}");
    let handle = started["run_id"].as_str().unwrap().to_string();
    assert!(handle.starts_with("e-"), "{started}");
    assert_eq!(started["setups"], json!(["duo"]));
    let clock = Instant::now();
    let mut named = None;
    let status = loop {
        assert!(
            clock.elapsed() < Duration::from_secs(400),
            "the run over every setup did not end"
        );
        let s = mcp.call("run_status", json!({"run_id": handle, "wait_s": 5})).data();
        seen.see();
        if s["setup"].is_string() {
            named = s["setup"].as_str().map(String::from);
        }
        if s["state"] == "finished" || s["state"] == "stopped" {
            break s;
        }
    };
    eprintln!("run_status: {status}");
    assert_eq!(named.as_deref(), Some("duo"), "run_status names the setup it is in");
    assert_eq!(status["state"], "finished");
    assert_eq!(status["runs"][0]["state"], "finished");
    let result = mcp.call("run_result", json!({"run_id": handle})).data();
    assert_eq!(result["summary"]["pass"], 1, "{result}");
    assert_eq!(result["runs"].as_array().unwrap().len(), 1);
    assert_eq!(result["outcomes"][0]["setup"], "duo");
    drop(mcp);
    nothing_left(&seen);
    drop(seen);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_075_a_setup_whose_session_cannot_start_costs_only_its_own_checks() {
    let every = format!("// @check US-075 runs in every setup\n{PASSES}");
    let other = format!("// @check US-075 a second check\n{PASSES}");
    let setups = "[setups.first]\nmode = \"singleplayer\"\nmods = []\n\n[setups.nomod]\nmode = \"singleplayer\"\nmods = [\"gone\"]\n\n[setups.noworld]\nmode = \"singleplayer\"\nmods = []\nworld = \"SkdNoSuchWorld\"\nstart_timeout = 30\n\n[setups.last]\nmode = \"singleplayer\"\nmods = []\n";
    let dir = project("fail", setups, &[("every", &every), ("other", &other)]);
    let seen = Seen::new(&dir);
    steam();
    let (code, out, _) = watched(&seen, &["run", "--each-setup"], Duration::from_secs(900), &mut |_| {});
    // US-075 3.
    assert_eq!(code, 2, "{out}");
    let all = results(&dir);
    let by_setup = |name: &str| {
        all.iter()
            .find(|r| r["session"]["setup"]["name"] == name)
            .unwrap_or_else(|| panic!("no result for {name}"))
    };
    for name in ["first", "last"] {
        let r = by_setup(name);
        assert_eq!(r["summary"]["pass"], 2, "{name} runs to its end: {r:#}");
    }
    // US-075 1: refused, naming the missing folder.
    let r = by_setup("nomod");
    assert!(r["session"]["id"].is_null(), "{r:#}");
    let not_run = r["not_run"].as_array().unwrap();
    assert_eq!(not_run.len(), 2);
    for n in not_run {
        assert_eq!(n["reason"], "session-failed");
        let message = n["message"].as_str().unwrap();
        assert!(message.contains("no-such-mod"), "{message}");
    }
    eprintln!("US-075 1: {}", not_run[0]["message"]);
    // US-075 2: a start that fails as SES-04 says.
    let r = by_setup("noworld");
    let not_run = r["not_run"].as_array().unwrap();
    assert_eq!(not_run.len(), 2, "{r:#}");
    assert_eq!(not_run[0]["reason"], "session-failed");
    assert!(
        not_run[0]["message"].as_str().unwrap().contains("failed to start"),
        "{}",
        not_run[0]["message"]
    );
    assert!(r["session"]["id"].is_string(), "{r:#}");
    eprintln!("US-075 2: {}", not_run[0]["message"]);
    assert!(out.contains("setup nomod: did not run"), "{out}");
    nothing_left(&seen);
    assert!(game_pids().iter().all(|p| !seen.pids.lock().unwrap().contains(p)));
    drop(seen);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_076_us_078_us_080_repeats_flaky_and_slow_steps() {
    let coin = "// @check US-076 passes about one time in two\n[random 1 < 0.5, \"==\", true, \"the coin came up heads\"] call skd_fnc_expect;\n";
    let steady = format!("// @check US-076 always passes\n{PASSES}");
    let slow_step = "// @check US-078 a step waits 12 s on a condition that then holds\n[\"wait 12 s\", { [\"p1\", {diag_tickTime > (_this select 0)}, \"12 s of real time\", 20, [diag_tickTime + 12]] call skd_fnc_waitUntil }] call skd_fnc_step;\n";
    let slow_check = "// @check US-078 takes 11 s of a 20 s timeout\n// @timeout 20\n[\"p1\", {diag_tickTime > (_this select 0)}, \"11 s of real time\", 18, [diag_tickTime + 11]] call skd_fnc_waitUntil;\n";
    let setups = "[setups.solo]\nmode = \"singleplayer\"\nmods = []\n";
    let dir = project(
        "repeat",
        setups,
        &[
            ("coin", coin),
            ("steady", &steady),
            ("slowstep", slow_step),
            ("slowcheck", slow_check),
        ],
    );
    let toml = std::fs::read_to_string(dir.join("shakedown.toml")).unwrap();
    let with_slow = |v: &str| {
        std::fs::write(
            dir.join("shakedown.toml"),
            toml.replace("tests = \"tests\"\n", &format!("tests = \"tests\"\nslow_step = {v}\n")),
        )
        .unwrap()
    };

    // US-080 1: refused before anything starts.
    for v in ["-1", "\"10\""] {
        with_slow(v);
        for args in [vec!["list"], vec!["run", "steady", "--setup", "solo"]] {
            let (code, _, err) = cmd(&dir, &args);
            assert_eq!(code, 3, "{args:?} with slow_step = {v}: {err}");
            assert!(err.contains("slow_step") && err.contains("line 3"), "{err}");
        }
    }
    assert!(game_pids().is_empty(), "a refused run started a game");
    std::fs::write(dir.join("shakedown.toml"), &toml).unwrap();

    steam();
    let seen = Seen::new(&dir);
    let (code, s, err) = cmd(&dir, &["session", "start", "--setup", "solo"]);
    assert_eq!(code, 0, "{s} {err}");
    seen.see();

    // US-076 1 and 2.
    let (code, out, _) = watched(
        &seen,
        &["run", "coin", "--repeat", "10"],
        Duration::from_secs(600),
        &mut |_| {},
    );
    let r = result_named(&out);
    let checks = r["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 10);
    let repeats: Vec<u64> = checks.iter().map(|c| c["repeat"].as_u64().unwrap()).collect();
    assert_eq!(repeats, (1..=10).collect::<Vec<u64>>());
    assert_eq!(r["run"]["repeat"], 10);
    let summary = &r["repeats"][0];
    eprintln!("US-076 2: {summary}");
    assert_eq!(
        summary["outcome"], "flaky",
        "about one in two passes; ten alike comes 2 times in 1024"
    );
    assert_eq!(code, 1);
    let line = out
        .lines()
        .find(|l| l.starts_with("flaky "))
        .unwrap_or_else(|| panic!("{out}"));
    assert!(
        line.contains("coin") && line.contains("pass") && line.contains("fail"),
        "{line}"
    );

    // US-076 3.
    let (code, r, err) = cmd(&dir, &["run", "steady", "--repeat", "10"]);
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!(r["repeats"][0]["outcome"], "pass");
    assert_eq!(r["repeats"][0]["counts"]["pass"], 10);

    // US-078 1 and 3.
    let (_, out, _) = watched(
        &seen,
        &["run", "slowstep", "slowcheck"],
        Duration::from_secs(300),
        &mut |_| {},
    );
    let r = result_named(&out);
    let step = &r["checks"][0]["steps"][0];
    assert_eq!(r["checks"][0]["outcome"], "pass", "{:#}", r["checks"][0]);
    assert_eq!(step["slow"], true, "{step:#}");
    assert!(step["duration_s"].as_f64().unwrap() >= 12.0, "{step:#}");
    assert!(out.contains("slow step: wait 12 s"), "{out}");
    let slow_check = &r["checks"][1];
    assert_eq!(slow_check["outcome"], "pass", "{slow_check:#}");
    assert_eq!(slow_check["slow"], true, "{slow_check:#}");
    assert_eq!(r["run"]["slow_step_s"], 10.0);

    // US-078 2.
    with_slow("15");
    let (_, r, _) = cmd(&dir, &["run", "slowstep"]);
    let r: Value = serde_json::from_str(&std::fs::read_to_string(r["result_file"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(r["checks"][0]["steps"][0]["slow"], false, "{:#}", r["checks"][0]);
    assert_eq!(r["run"]["slow_step_s"], 15.0);
    std::fs::write(dir.join("shakedown.toml"), &toml).unwrap();

    // US-076 4: the same summary through MCP.
    let mut mcp = support::Mcp::start(&exe(), &dir, &[]);
    let started = mcp.call("run_start", json!({"ids": ["steady"], "repeat": 3})).data();
    assert_eq!(started["total"], 3);
    let handle = started["run_id"].as_str().unwrap().to_string();
    let clock = Instant::now();
    loop {
        assert!(
            clock.elapsed() < Duration::from_secs(120),
            "the repeated run did not end"
        );
        let s = mcp.call("run_status", json!({"run_id": handle, "wait_s": 5})).data();
        if s["state"] == "finished" {
            assert_eq!(s["setup"], "solo");
            break;
        }
    }
    let result = mcp.call("run_result", json!({"run_id": handle})).data();
    eprintln!("US-076 4: {}", result["repeats"]);
    assert_eq!(result["repeats"][0]["outcome"], "pass");
    assert_eq!(result["repeats"][0]["counts"]["pass"], 3);
    drop(mcp);

    let (code, _, _) = cmd(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    nothing_left(&seen);
    drop(seen);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_077_the_session_breaks_during_repeats() {
    let waits = "// @check US-077 waits 8 s on the server\n[\"server\", {diag_tickTime > (_this select 0)}, \"8 s of real time\", 20, [diag_tickTime + 8]] call skd_fnc_waitUntil;\n";
    let setups = "[setups.duo]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n";
    let dir = project("broken", setups, &[("waits", waits)]);
    steam();
    let seen = Seen::new(&dir);
    let (code, s, err) = cmd(&dir, &["session", "start", "--setup", "duo"]);
    assert_eq!(code, 0, "{s} {err}");
    seen.see();
    let p2 = s["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "p2")
        .and_then(|m| m["pid"].as_u64())
        .unwrap();

    // US-077 1: p2's process is ended during the third of five repeats.
    let mut ended = false;
    let (code, out, _) = watched(
        &seen,
        &["run", "waits", "--repeat", "5"],
        Duration::from_secs(600),
        &mut |s| {
            if !ended && s["run"]["done"] == 2 {
                let _ = Command::new("taskkill").args(["/PID", &p2.to_string(), "/F"]).output();
                ended = true;
                eprintln!("ended p2 ({p2}) during the third repeat");
            }
        },
    );
    assert!(ended, "the run never reached its third repeat: {out}");
    let r = result_named(&out);
    let outcomes: Vec<(String, String)> = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["outcome"].as_str().unwrap().to_string(),
                c["reason"]["kind"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    assert_eq!(
        outcomes,
        vec![
            ("pass".to_string(), String::new()),
            ("pass".to_string(), String::new()),
            ("invalid".to_string(), "machine-lost".to_string()),
        ],
        "{r:#}"
    );
    let not_run: Vec<(u64, String)> = r["not_run"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["repeat"].as_u64().unwrap(), n["reason"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(
        not_run,
        vec![(4, "session-broken".to_string()), (5, "session-broken".to_string())]
    );
    let summary = &r["repeats"][0];
    eprintln!("US-077 1: {summary}");
    assert_eq!(summary["outcome"], "pass");
    assert_eq!(
        summary["counts"],
        json!({"pass": 2, "fail": 0, "invalid": 1, "not_run": 2})
    );
    assert_eq!(code, 2);
    let (code, _, _) = cmd(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    nothing_left(&seen);
    drop(seen);
    let _ = std::fs::remove_dir_all(&dir);
}
