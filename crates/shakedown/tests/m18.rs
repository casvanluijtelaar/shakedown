//! M18's tests: each machine's frame rate and answer time in `status`, through the command line
//! and through MCP (`US-093`, `INS-04`, `HLT-01`), and a client starved of CPU during a check
//! makes it `invalid` / `unhealthy` naming the client and its frame rate, the flag clears, and the
//! next check passes (`US-094`, `HLT-01`, `DM-05`, `DM-06`). They drive the installed `shakedown`
//! and the game on a dedicated session with two clients and a headless client loading the
//! fixture, so they run only on the test machine, by hand, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m18 -- --ignored --nocapture --test-threads=1
//!
//! `p2` is starved by suspending its process 450 ms of every 500 ms, by the id `status` gives;
//! every game process of the session is ended by its id after the test.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::starve::Starve;
use support::{Mcp, repo};

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// A project with the e2e project's file and setups and the checks of M18.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m18-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = std::fs::read_to_string(repo().join(r"e2e\shakedown.toml"))
        .unwrap()
        .replace(r"'..\target", &format!("'{}", repo().join("target").display()));
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    std::fs::write(
        dir.join(r"tests\starve.sqf"),
        "// @check US-094 a check that runs 20 s of real time while p2 is starved\n// @timeout 60\nuiSleep 20;\n[1, \"==\", 1, \"one\"] call skd_fnc_expect;\n",
    )
    .unwrap();
    std::fs::write(
        dir.join(r"tests\pass.sqf"),
        "// @check US-094 the next check passes\n[\"p2\", { [time, \">\", 0, \"p2's clock\"] call skd_fnc_expect }] call skd_fnc_on;\n",
    )
    .unwrap();
    dir
}

fn steam_runs() {
    let steam = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&steam.stdout).contains("steam.exe"),
        "Steam must run before a game start"
    );
}

fn machines(status: &Value) -> Vec<Value> {
    status["session"]["machines"].as_array().cloned().unwrap_or_default()
}

fn health(dir: &Path, machine: &str) -> Value {
    let (_, s, _) = cmd(dir, &["status"]);
    machines(&s)
        .into_iter()
        .find(|m| m["name"] == machine)
        .map(|m| m["health"].clone())
        .unwrap_or_default()
}

/// Stops the session when dropped, also when a test fails, and then every game process of it
/// still running, by id.
struct Session {
    dir: PathBuf,
    pids: Vec<u64>,
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = cmd(&self.dir, &["session", "stop"]);
        for pid in &self.pids {
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
        }
    }
}

fn validate(file: &str) -> Value {
    let schema: Value =
        serde_json::from_str(&std::fs::read_to_string(repo().join("spec/contract/result.schema.json")).unwrap())
            .unwrap();
    let result: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&result)
        .map(|e| format!("{e} at {}", e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    result
}

/// `US-093` 1 and 2, then `US-094` 1 and 2, on one dedicated session with two clients and a
/// headless client.
#[test]
#[ignore = "starts a dedicated session of Arma 3"]
fn us_093_us_094_hlt_01_frame_rate_and_answer_time_on_a_dedicated_session() {
    steam_runs();
    let dir = project("health");
    let (code, out, err) = cmd(&dir, &["session", "start", "--setup", "dedicated"]);
    let (_, s, _) = cmd(&dir, &["status"]);
    let session = Session {
        dir: dir.clone(),
        pids: machines(&s).iter().filter_map(|m| m["pid"].as_u64()).collect(),
    };
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(session.pids.len(), 4, "{s}");

    // US-093 1: every machine's frame rate over the last 5 s and its answer time beside the clocks
    // of its last heartbeat, within 30 s of the start.
    let mut status = Value::Null;
    for _ in 0..60 {
        let (_, s, _) = cmd(&dir, &["status"]);
        status = s;
        if machines(&status)
            .iter()
            .all(|m| m["health"]["frame_rate"].is_number() && m["health"]["answer_s"].is_number())
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    let names: Vec<String> = machines(&status)
        .iter()
        .map(|m| m["name"].as_str().unwrap_or("?").to_string())
        .collect();
    assert_eq!(names, ["server", "p1", "p2", "hc1"], "{status}");
    for m in machines(&status) {
        let h = &m["health"];
        println!("US-093 {}: {h}", m["name"]);
        assert!(h["frame_rate"].as_f64().is_some_and(|r| r > 0.0), "US-093 1: {m}");
        assert!(h["answer_s"].as_f64().is_some_and(|a| a >= 0.0), "US-093 1: {m}");
        assert!(h["heartbeat"]["frame"].is_number(), "US-093 1: {m}");
        assert!(h["flag"].is_null(), "US-093 1: {m}");
    }
    let text = Command::new(exe()).arg("status").current_dir(&dir).output().unwrap();
    let text = String::from_utf8_lossy(&text.stdout).to_string();
    println!("{text}");
    assert_eq!(
        text.matches("health: ").count(),
        4,
        "US-093 1: a health line per machine: {text}"
    );
    assert!(text.contains("frames a second, answers in "), "{text}");

    // US-093 2: status through MCP gives the same fields, valid against the tool's schema.
    {
        let mut mcp = Mcp::start(&exe(), &dir, &[]);
        let data = mcp.call("status", json!({})).data();
        for m in machines(&data) {
            assert!(m["health"]["frame_rate"].is_number(), "US-093 2: {m}");
            assert!(m["health"]["answer_s"].is_number(), "US-093 2: {m}");
            assert!(m["health"]["heartbeat"].is_object(), "US-093 2: {m}");
        }
    }

    // US-094 1: p2 starved during the check until it is flagged; the check ends invalid /
    // unhealthy naming p2 and its frame rate. US-094 2: the flag clears and the next check passes.
    let p2 = machines(&status)
        .iter()
        .find(|m| m["name"] == "p2")
        .and_then(|m| m["pid"].as_u64())
        .unwrap() as u32;
    let child = Command::new(exe())
        .args(["run", "starve", "pass", "--json"])
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut running = false;
    for _ in 0..100 {
        let (_, s, _) = cmd(&dir, &["status"]);
        if s["run"]["current_check"] == "starve" {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(running, "the run did not start the check");
    let clock = Instant::now();
    let starve = Starve::start(p2, Duration::from_millis(450), Duration::from_millis(50));
    let mut flagged = Value::Null;
    for _ in 0..30 {
        let h = health(&dir, "p2");
        if !h["flag"].is_null() {
            flagged = h;
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    drop(starve);
    let took = clock.elapsed();
    println!("US-094 p2 flagged after {took:?}: {flagged}");
    assert_eq!(flagged["flag"], "frame-rate", "US-094 1: {flagged}");
    assert!(took < Duration::from_secs(15), "US-094 1: flagged within 15 s");
    let mut cleared = false;
    for _ in 0..40 {
        if health(&dir, "p2")["flag"].is_null() {
            cleared = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    println!(
        "US-094 p2's flag cleared {:?} after the start of its starving",
        clock.elapsed()
    );
    assert!(cleared, "US-094 2: the flag clears");
    let out = child.wait_with_output().unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let result = validate(v["result_file"].as_str().unwrap_or_else(|| panic!("{v}")));
    let starved = &result["checks"][0];
    println!("US-094 starve: {starved:#}");
    assert_eq!(starved["check_id"], "starve");
    assert_eq!(
        (&starved["outcome"], &starved["reason"]["kind"]),
        (&Value::from("invalid"), &Value::from("unhealthy")),
        "US-094 1"
    );
    assert_eq!(starved["reason"]["machine"], "p2", "US-094 1");
    let message = starved["reason"]["message"].as_str().unwrap();
    assert!(
        message.contains("p2's frame rate fell to") && message.contains("frames a second"),
        "US-094 1: {message}"
    );
    let next = &result["checks"][1];
    assert_eq!(
        (&next["check_id"], &next["outcome"]),
        (&Value::from("pass"), &Value::from("pass")),
        "US-094 2: {next:#}"
    );
    assert_eq!(out.status.code(), Some(2), "a run with an invalid check exits 2");
    drop(session);
}
