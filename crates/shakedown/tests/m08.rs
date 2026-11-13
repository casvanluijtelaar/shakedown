//! M8's tests: a run that starts its own session and leaves nothing running (`US-033`), a run
//! stopped by hand with Ctrl+Break, `run stop` and `run_stop` (`US-034`), `session stop` and
//! `session_stop` (`US-036`), and a run stopped at its runner ceiling (`US-063`), in singleplayer
//! and on a dedicated session with two clients. They drive the installed `shakedown` and the
//! game, so they run only on the test machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m08 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{Mcp, repo};

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";
const BASE_PORT: u16 = 2402;

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

const QUICK: &str = r#"["one is one", { [1, "==", 1, "one"] call skd_fnc_expect }] call skd_fnc_step;"#;

/// The check the stop ends: it waits 60 s of game time in a step.
const SLOW: &str = r#"["a minute", {
    private _t = diag_tickTime;
    waitUntil { diag_tickTime > _t + 60 };
}] call skd_fnc_step;"#;

/// A check of `@timeout 1` whose cleanup runs for 14 s of real time (`US-063`).
const CLEANUP: &str = r#"[{
    private _t = diag_tickTime;
    waitUntil { diag_tickTime > _t + 14 };
}, []] call skd_fnc_cleanup;"#;

/// A project with the setups `sp` and `ded2` (two clients), both loading `@skd_fixture`, and the
/// checks `first`, `slow`, `third`, `every` (each machine answers) and `c1` to `c8`.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m08-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'
base_port = {BASE_PORT}

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.ded2]
mode = "dedicated"
clients = 2
mods = ["fixture"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    let mut checks = vec![
        (
            "first".to_string(),
            format!("// @check US-034 the first check\n{QUICK}\n"),
        ),
        (
            "slow".to_string(),
            format!("// @check US-034 the check a stop ends\n// @timeout 120\n{SLOW}\n"),
        ),
        (
            "third".to_string(),
            format!("// @check US-034 the check not run\n{QUICK}\n"),
        ),
        (
            "every".to_string(),
            r#"// @check US-033 every machine answers
["each machine", {
    private _v = ["all", { 1 }] call skd_fnc_onEach;
    [count _v, "==", count (call skd_fnc_machines), "machines that answered"] call skd_fnc_expect;
}] call skd_fnc_step;
"#
            .to_string(),
        ),
    ];
    for i in 1..=8 {
        checks.push((
            format!("c{i}"),
            format!("// @check US-063 a check whose cleanup runs 14 s\n// @timeout 1\n{CLEANUP}\n"),
        ));
    }
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

fn kill(pid: u64) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
}

/// The running session's machines: (name, start, pid).
fn machines(dir: &Path) -> Vec<(String, u64, u64)> {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|m| {
            (
                m["name"].as_str().unwrap_or_default().to_string(),
                m["start"].as_u64().unwrap_or(0),
                m["pid"].as_u64().unwrap_or(0),
            )
        })
        .collect()
}

/// Whether a process with this id runs.
fn runs(pid: u64) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
}

/// The ports from the base port to the base port plus 4 that a program holds (`STP-04`).
fn held_ports() -> Vec<u16> {
    (BASE_PORT..BASE_PORT + 5)
        .filter(|p| std::net::UdpSocket::bind(("0.0.0.0", *p)).is_err())
        .collect()
}

/// `STP-02`, `STP-04`: none of `pids` runs, no port is held, no mission copy of `session` is
/// left, and `status` shows no session.
fn nothing_left(dir: &Path, pids: &[u64], session: &str) {
    for pid in pids {
        assert!(!runs(*pid), "game process {pid} still runs");
    }
    assert_eq!(held_ports(), Vec::<u16>::new(), "ports held");
    let copies: Vec<String> = std::fs::read_dir(Path::new(GAME).join("MPMissions"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(session))
        .collect();
    assert!(copies.is_empty(), "mission copies left: {copies:?}");
    let (_, s, _) = cmd(dir, &["status"]);
    assert!(s["session"].is_null(), "{s}");
}

fn validate(file: &Path) -> Value {
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

/// Seconds of the day of an ISO 8601 UTC time.
fn seconds(t: &str) -> f64 {
    let clock = &t[11..t.len() - 1];
    let parts: Vec<f64> = clock.split(':').map(|p| p.parse().unwrap()).collect();
    parts[0] * 3600.0 + parts[1] * 60.0 + parts[2]
}

fn between(a: &str, b: &str) -> f64 {
    let d = seconds(b) - seconds(a);
    if d < 0.0 { d + 86400.0 } else { d }
}

/// `RES-04`, `US-033` 2: every path in the result file is relative to the run folder and exists.
fn paths_exist(result: &Value, folder: &Path) {
    let mut paths: Vec<String> = result["logs"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    for c in result["checks"].as_array().unwrap() {
        paths.extend(
            c["evidence"]["logs"]
                .as_object()
                .unwrap()
                .values()
                .map(|v| v.as_str().unwrap().to_string()),
        );
    }
    assert!(!paths.is_empty());
    for p in paths {
        assert!(Path::new(&p).is_relative(), "{p}");
        assert!(folder.join(&p).is_file(), "{p}");
    }
}

/// A `shakedown run --json` started in its own process group, so a Ctrl+Break reaches it alone.
struct Background {
    child: std::process::Child,
    out: PathBuf,
}

impl Background {
    fn start(dir: &Path, args: &[&str]) -> Background {
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        let out = dir.join(format!("run-{}.json", args.join("-")));
        let child = Command::new(exe())
            .arg("run")
            .args(args)
            .arg("--json")
            .current_dir(dir)
            .stdout(std::fs::File::create(&out).unwrap())
            .stderr(Stdio::inherit())
            .creation_flags(CREATE_NEW_PROCESS_GROUP)
            .spawn()
            .unwrap();
        Background { child, out }
    }

    /// Sends the command a Ctrl+Break console event (`RUN-05`).
    fn ctrl_break(&self) {
        use windows::Win32::System::Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent};
        unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, self.child.id()) }.expect("a Ctrl+Break to the run");
    }

    /// The exit code and the JSON it printed, within `ceiling`.
    fn wait(mut self, ceiling: Duration) -> (i32, Value) {
        let clock = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                let text = std::fs::read_to_string(&self.out).unwrap_or_default();
                return (
                    status.code().unwrap_or(-1),
                    serde_json::from_str(&text).unwrap_or(Value::Null),
                );
            }
            if clock.elapsed() > ceiling {
                let _ = self.child.kill();
                panic!("the run did not end within {ceiling:?}");
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

/// Polls `status` until the active run is at check `id` with `done` checks done; the run's id.
fn at_check(dir: &Path, id: &str, done: u64, ceiling: Duration) -> String {
    let clock = Instant::now();
    while clock.elapsed() < ceiling {
        let (_, s, _) = cmd(dir, &["status"]);
        if s["run"]["current_check"] == id && s["run"]["done"] == done {
            return s["run"]["id"].as_str().unwrap().to_string();
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    panic!("the run never reached check {id}");
}

/// `US-034` 1: the second of three checks ended `invalid` / `run-stopped`, the third not run, the
/// run `stopped` by hand.
fn stopped_by_hand(result: &Value) {
    assert_eq!(result["run"]["state"], "stopped", "{result:#}");
    assert_eq!(result["run"]["stopped_reason"], "by-hand");
    let checks = result["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 2, "{result:#}");
    assert_eq!(checks[0]["outcome"], "pass");
    assert_eq!(
        (
            &checks[1]["check_id"],
            &checks[1]["outcome"],
            &checks[1]["reason"]["kind"]
        ),
        (&json!("slow"), &json!("invalid"), &json!("run-stopped"))
    );
    assert_eq!(
        result["not_run"],
        json!([{"check_id": "third", "reason": "run-stopped"}])
    );
}

/// A session started for a test, stopped when dropped, also when the test panics; any process of
/// it left after the stop is ended by its id.
struct Session {
    dir: PathBuf,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        steam();
        let (code, s, err) = cmd(dir, &["session", "start", "--setup", setup]);
        assert_eq!(code, 0, "{s} {err}");
        Session { dir: dir.to_path_buf() }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let left = machines(&self.dir);
        let (code, _, _) = cmd(&self.dir, &["session", "stop"]);
        if code != 0 {
            for (_, _, pid) in left {
                kill(pid);
            }
        }
    }
}

/// Ends by id every game process a run that started its own session left, if the test fails.
struct Watch {
    dir: PathBuf,
    pids: Vec<u64>,
}

impl Watch {
    fn new(dir: &Path) -> Watch {
        Watch {
            dir: dir.to_path_buf(),
            pids: vec![],
        }
    }

    /// Records the machines of the session the run started, once it shows in `status`.
    fn record(&mut self) -> String {
        let clock = Instant::now();
        while clock.elapsed() < Duration::from_secs(240) {
            let (_, s, _) = cmd(&self.dir, &["status"]);
            if s["session"]["state"] == "running" {
                self.pids = machines(&self.dir).iter().map(|m| m.2).collect();
                return s["session"]["id"].as_str().unwrap().to_string();
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        panic!("the run's session never ran");
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        for pid in &self.pids {
            if runs(*pid) {
                kill(*pid);
            }
        }
    }
}

/// `US-034` 1 and 3 with `--setup`: Ctrl+Break during the second of three checks.
fn ctrl_break_with_setup(dir: &Path, setup: &str) {
    steam();
    let mut watch = Watch::new(dir);
    let run = Background::start(dir, &["first", "slow", "third", "--setup", setup]);
    let session = watch.record();
    at_check(dir, "slow", 1, Duration::from_secs(120));
    run.ctrl_break();
    let (code, out) = run.wait(Duration::from_secs(120));
    println!("US-034 Ctrl+Break on {setup}: exit {code}: {out}");
    assert_eq!(code, 2, "{out}");
    assert_eq!(out["state"], "stopped");
    let file = PathBuf::from(out["result_file"].as_str().unwrap());
    let result = validate(&file);
    stopped_by_hand(&result);
    paths_exist(&result, file.parent().unwrap());
    nothing_left(dir, &watch.pids, &session);
}

#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_034_stp_02_ctrl_break_stops_a_run_that_started_its_own_session_in_singleplayer() {
    let dir = project("brk-sp");
    ctrl_break_with_setup(&dir, "sp");
}

/// `US-034` 1 to 3 against a running session, `US-063` 2 and 3: `run stop` and `run_stop` stop a
/// run and leave the session `ready` with every start number kept; through MCP, the eight checks
/// of `US-063` stop at the runner ceiling.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_034_us_063_a_stopped_run_leaves_its_session_ready_in_singleplayer() {
    let dir = project("stop-sp");
    let _s = Session::start(&dir, "sp");
    let before = machines(&dir);

    // US-034 1: `run stop` from another command.
    let run = Background::start(&dir, &["first", "slow", "third"]);
    let run_id = at_check(&dir, "slow", 1, Duration::from_secs(60));
    let (code, stopped, err) = cmd(&dir, &["run", "stop"]);
    println!("US-034 run stop: exit {code}: {stopped}");
    assert_eq!(code, 0, "{stopped} {err}");
    assert_eq!(
        (&stopped["run_id"], &stopped["state"]),
        (&json!(run_id), &json!("stopped"))
    );
    assert_eq!((&stopped["done"], &stopped["total"]), (&json!(2), &json!(3)));
    let (code, out) = run.wait(Duration::from_secs(60));
    assert_eq!(code, 2, "{out}");
    stopped_by_hand(&validate(Path::new(out["result_file"].as_str().unwrap())));
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready", "{s}");
    assert!(s["run"].is_null(), "{s}");
    let (code, _, err) = cmd(&dir, &["run", "stop"]);
    assert_eq!(code, 4, "with no run active: {err}");

    // US-034 2: `run_stop` through MCP.
    let mut mcp = Mcp::start(&exe(), &dir, &[]);
    let started = mcp.call("run_start", json!({"ids": ["first", "slow", "third"]})).data();
    let run_id = started["run_id"].as_str().unwrap().to_string();
    at_check(&dir, "slow", 1, Duration::from_secs(60));
    let answer = mcp.call("run_stop", json!({"run_id": run_id}));
    println!("US-034 run_stop: {}", answer.text());
    let stopped = answer.data();
    assert_eq!(stopped["state"], "stopped", "{stopped}");
    assert_eq!(stopped["outcomes"][1]["reason"]["kind"], "run-stopped", "{stopped}");
    let result = mcp.call("run_result", json!({"run_id": run_id})).data();
    stopped_by_hand(&validate(Path::new(result["result_file"].as_str().unwrap())));

    // US-063 3: the runner ceiling through MCP.
    let ids: Vec<String> = (1..=8).map(|i| format!("c{i}")).collect();
    let started = mcp.call("run_start", json!({"ids": ids})).data();
    let run_id = started["run_id"].as_str().unwrap().to_string();
    let clock = Instant::now();
    let mut status = Value::Null;
    while clock.elapsed() < Duration::from_secs(150) {
        status = mcp.call("run_status", json!({"run_id": run_id, "wait_s": 30})).data();
        if status["state"] != "running" {
            break;
        }
    }
    println!("US-063 run_status: {status}");
    assert_eq!(status["state"], "stopped", "{status}");
    let result = mcp.call("run_result", json!({"run_id": run_id})).data();
    let file = validate(Path::new(result["result_file"].as_str().unwrap()));
    assert_eq!(result["summary"], file["summary"]);
    ceiling_stopped(&file);
    drop(mcp);

    // US-034 3, US-063 2: the session is ready and every machine keeps its start number.
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready", "{s}");
    assert_eq!(machines(&dir), before, "a machine was restarted");
}

/// `US-063` 1: the run of the eight checks stopped between 68 s and 80 s after its first check
/// started, the check it was on `invalid` / `run-stopped` and every check not started not run.
fn ceiling_stopped(result: &Value) {
    assert_eq!(
        (&result["run"]["state"], &result["run"]["stopped_reason"]),
        (&json!("stopped"), &json!("run-ceiling")),
        "{result:#}"
    );
    let checks = result["checks"].as_array().unwrap();
    let last = checks.last().unwrap();
    assert_eq!(
        (&last["outcome"], &last["reason"]["kind"]),
        (&json!("invalid"), &json!("run-stopped")),
        "{result:#}"
    );
    assert!(last["reason"]["message"].as_str().unwrap().contains("68 s"), "{last}");
    for c in &checks[..checks.len() - 1] {
        assert_eq!(c["outcome"], "pass", "{c}");
    }
    let not_run = result["not_run"].as_array().unwrap();
    assert_eq!(checks.len() + not_run.len(), 8, "{result:#}");
    assert!(not_run.iter().all(|n| n["reason"] == "run-stopped"), "{result:#}");
    let took = between(
        checks[0]["started_at"].as_str().unwrap(),
        last["ended_at"].as_str().unwrap(),
    );
    println!(
        "US-063: {} checks executed, the stop {took:.1} s after the first check started",
        checks.len()
    );
    assert!((68.0..80.0).contains(&took), "{took:.1} s");
}

/// `US-063` 1 and 2 with `--setup sp`: the run stops at its ceiling and stops its session.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_063_run_06_the_runner_ceiling_stops_a_run_that_started_its_own_session() {
    let dir = project("ceil-sp");
    steam();
    let mut watch = Watch::new(&dir);
    let ids: Vec<String> = (1..=8).map(|i| format!("c{i}")).collect();
    let mut args: Vec<&str> = ids.iter().map(String::as_str).collect();
    args.extend(["--setup", "sp"]);
    let run = Background::start(&dir, &args);
    let session = watch.record();
    let (code, out) = run.wait(Duration::from_secs(300));
    println!("US-063 run --setup sp: exit {code}");
    assert_eq!(code, 2, "{out}");
    ceiling_stopped(&validate(Path::new(out["result_file"].as_str().unwrap())));
    nothing_left(&dir, &watch.pids, &session);
}

/// `US-033` 1 to 3 and `US-034` 1 and 3 on a dedicated session with two clients, and `US-036`
/// through the command line and `session_stop`.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_033_us_034_us_036_walk_away_on_a_dedicated_session() {
    let dir = project("ded2");

    // US-033: the whole run with a session it starts and stops.
    steam();
    let mut watch = Watch::new(&dir);
    let run = Background::start(&dir, &["first", "every", "third", "--setup", "ded2"]);
    let session = watch.record();
    let (code, out) = run.wait(Duration::from_secs(400));
    println!("US-033 run --setup ded2: exit {code}");
    assert_eq!(code, 0, "{out}");
    let file = PathBuf::from(out["result_file"].as_str().unwrap());
    let result = validate(&file);
    assert_eq!(result["run"]["state"], "finished");
    let folder = file.parent().unwrap();
    for m in ["server", "p1", "p2"] {
        assert!(folder.join("logs").join(format!("{m}.log")).is_file(), "{m}");
        for n in 1..=3 {
            assert!(folder.join(format!("checks/{n}/{m}.log")).is_file(), "{n} {m}");
        }
    }
    paths_exist(&result, folder);
    nothing_left(&dir, &watch.pids, &session);
    let ignore = std::fs::read_to_string(dir.join(".shakedown").join(".gitignore")).unwrap();
    assert_eq!(ignore.trim(), "*");
    let mut written: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| !n.starts_with("run-"))
        .collect();
    written.sort();
    assert_eq!(written, [".shakedown", "shakedown.toml", "tests"]);
    drop(watch);

    // US-034 with --setup on a dedicated session.
    ctrl_break_with_setup(&dir, "ded2");

    // US-036 1: `session stop`.
    let s = Session::start(&dir, "ded2");
    let pids: Vec<u64> = machines(&dir).iter().map(|m| m.2).collect();
    let (_, st, _) = cmd(&dir, &["status"]);
    let id = st["session"]["id"].as_str().unwrap().to_string();
    let (code, stopped, err) = cmd(&dir, &["session", "stop"]);
    assert_eq!(code, 0, "{stopped} {err}");
    nothing_left(&dir, &pids, &id);
    std::mem::forget(s);

    // US-036 2: `session_stop`.
    let s = Session::start(&dir, "ded2");
    let pids: Vec<u64> = machines(&dir).iter().map(|m| m.2).collect();
    let mut mcp = Mcp::start(&exe(), &dir, &[]);
    let stopped = mcp.call("session_stop", json!({})).data();
    assert_eq!(stopped["state"], "stopped", "{stopped}");
    let id = stopped["id"].as_str().unwrap().to_string();
    let status = mcp.call("status", json!({})).data();
    assert!(status["session"].is_null(), "{status}");
    drop(mcp);
    nothing_left(&dir, &pids, &id);
    std::mem::forget(s);
}
