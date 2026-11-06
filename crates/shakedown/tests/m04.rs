//! M4's end-to-end tests: the MCP server and the command line's `exec`, `wait`, `expect`,
//! `inspect` and `errors` against the game, driven through their public interfaces, the
//! command line and the MCP protocol on stdio. The stories `US-001`, `US-003`, `US-005`,
//! `US-013`, `US-014`, `US-023`, `US-031`, `US-032`, `US-042`, `US-046`, `US-047`, `US-048`,
//! `US-049`, `US-050`, `US-051`, `US-061` and `US-062`, `NFR-PF-03`, `DEF-10` and `DEF-12`. They
//! drive the installed `shakedown` against the game, so they run only on the test machine, by
//! hand, one at a time:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m04 -- --ignored --nocapture --test-threads=1
//!
//! The Steam ID and the Steam names are read from this machine's Steam files at run time and
//! never printed: a test says only how many it found.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{Answer, Mcp, repo, shape, without};

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cli(dir: &Path, args: &[&str]) -> (i32, String, String) {
    support::run(&exe(), dir, &[], args)
}

fn cli_json(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

fn project(name: &str, checks: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m04-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[mods.ghost]
path = '{GAME}\@skd_no_such_mod'

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.missing]
mode = "singleplayer"
mods = ["fixture", "ghost"]

[setups.ded]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]

[setups.ded30]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]
start_timeout = 30
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("checks").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

/// The ids of the running game processes.
fn game_pids() -> Vec<u32> {
    let mut out = Vec::new();
    for image in ["arma3_x64.exe", "arma3server_x64.exe"] {
        let o = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {image}"), "/FO", "CSV", "/NH"])
            .output()
            .unwrap();
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if let Some(pid) = line.split("\",\"").nth(1).and_then(|p| p.parse().ok()) {
                out.push(pid);
            }
        }
    }
    out
}

fn free_mb() -> u64 {
    let o = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1024)",
        ])
        .output()
        .unwrap();
    String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0)
}

/// Stops the session when dropped, also when the test panics, and ends by id any game process
/// that outlives the stop.
struct Guard {
    dir: PathBuf,
    before: Vec<u32>,
}

impl Guard {
    fn new(dir: &Path) -> Guard {
        let before = game_pids();
        assert!(before.is_empty(), "a game runs already: {before:?}");
        println!("free memory before: {} MB", free_mb());
        Guard {
            dir: dir.to_path_buf(),
            before,
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = cli(&self.dir, &["session", "stop"]);
        for pid in game_pids().into_iter().filter(|p| !self.before.contains(p)) {
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
        }
        println!("free memory after: {} MB", free_mb());
    }
}

fn error_text(a: &Answer, code: &str) -> String {
    assert!(a.is_error(), "not an error: {}", a.result);
    let t = a.text();
    assert!(t.starts_with(&format!("{code}: ")), "`{t}` is not {code}");
    t
}

fn p95(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() as f64) * 0.95).ceil() as usize - 1]
}

/// `NFR-PF-03`: 100 `exec` round trips of `1 + 1` to a machine through the command line, each
/// answering 2; the 95th percentile of the whole command's time, in seconds.
fn exec_p95(dir: &Path, machine: &str) -> f64 {
    let mut times = Vec::new();
    for _ in 0..100 {
        let t = Instant::now();
        let out = Command::new(exe())
            .args(["exec", "--machine", machine, "--code", "1 + 1", "--json"])
            .current_dir(dir)
            .output()
            .unwrap();
        times.push(t.elapsed().as_secs_f64());
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        let name = if machine == "server" && v["results"]["server"].is_null() {
            "p1"
        } else {
            machine
        };
        assert_eq!(v["results"][name]["value"], 2, "{v}");
    }
    p95(times)
}

/// Polls inspect until no exec order's script runs on the machine, up to `ceiling`.
fn no_exec_script_within(mcp: &mut Mcp, machine: &str, ceiling: Duration) -> bool {
    let start = Instant::now();
    for _ in 0..100 {
        let v = mcp.call("inspect", json!({"machine": machine})).data();
        let busy = v["scripts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"].as_str().unwrap_or_default().starts_with("skd_exec"));
        if !busy {
            return true;
        }
        if start.elapsed() > ceiling {
            return false;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

/// The longest time between two progress notifications, and before the first.
fn longest_gap(notes: &[(Duration, Value)], took: Duration) -> Duration {
    let mut times: Vec<Duration> = notes
        .iter()
        .filter(|(_, n)| n["method"] == "notifications/progress")
        .map(|(t, _)| *t)
        .collect();
    times.insert(0, Duration::ZERO);
    times.push(took);
    times.windows(2).map(|w| w[1] - w[0]).max().unwrap()
}

const PASS: &str = "// @check US-031 a check that passes\n// @tags m04\n[\"p1\", {1}] call skd_fnc_on;\n";
const FAILS: &str = "// @check US-048 a check that fails\n// @tags m04\n[\"p1\", {1}, \"==\", 2, \"one is two\", 1] call skd_fnc_expectOn;\n";
const LONG: &str = "// @check US-046 a check that takes 8 s\n// @tags long\n[\"p1\", {false}, \"eight seconds\", 8] call skd_fnc_tryWaitUntil;\n";
const RAISE: &str = "// @check US-062 a wait whose condition raises\n[\"wait\", { [\"p1\", {_skdfx_undefined > 1}, \"flag above one\", 10] call skd_fnc_waitUntil }] call skd_fnc_step;\n";
const STRING: &str = "// @check US-062 a wait whose condition is a String\n[\"p1\", {\"yes\"}, \"a string, not a Boolean\"] call skd_fnc_waitUntil;\n";
const TRY_RAISE: &str = "// @check US-062 a try wait whose condition raises\n[\"p1\", {_skdfx_undefined > 1}, \"flag above one\", 10] call skd_fnc_tryWaitUntil;\n";
const TRY_STRING: &str = "// @check US-062 a try wait whose condition is a String\n[\"p1\", {\"yes\"}, \"a string, not a Boolean\"] call skd_fnc_tryWaitUntil;\n";
const THROW: &str = "// @check US-061 code sent to p1 raises\nprivate _v = [\"throw on p1\", { [\"p1\", {[] call skdfx_fnc_throw; 7}] call skd_fnc_on }] call skd_fnc_step;\n[\"value\", _v] call skd_fnc_evidence;\n[\"after\", true] call skd_fnc_evidence;\n";

fn sp_checks() -> Vec<(&'static str, &'static str)> {
    vec![
        ("pass", PASS),
        ("fails", FAILS),
        ("long", LONG),
        ("raise", RAISE),
        ("string", STRING),
        ("tryraise", TRY_RAISE),
        ("trystring", TRY_STRING),
        ("throw", THROW),
    ]
}

fn by_id<'a>(result: &'a Value, id: &str) -> &'a Value {
    result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check_id"] == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

fn result_of(run_result: &Value) -> Value {
    serde_json::from_str(&std::fs::read_to_string(run_result["result_file"].as_str().unwrap()).unwrap()).unwrap()
}

/// Follows a run with `run_status` and `wait_s` 50 until it ends; every call answers within 60 s
/// (`US-050`). Returns the calls' answers.
fn follow(mcp: &mut Mcp, run_id: &str) -> Vec<Answer> {
    let mut calls = Vec::new();
    for _ in 0..60 {
        let a = mcp.call_progress("run_status", json!({"run_id": run_id, "wait_s": 50}));
        assert!(a.took < Duration::from_secs(60), "run_status took {:?}", a.took);
        let ended = a.data()["state"] == "finished";
        calls.push(a);
        if ended {
            return calls;
        }
    }
    panic!("run {run_id} did not end");
}

#[test]
#[ignore = "starts the game"]
fn us_001_us_003_us_005_us_013_us_014_us_023_us_031_us_032_us_042_us_046_us_047_us_048_us_049_us_050_us_051_us_061_us_062_agent_tools_in_singleplayer()
 {
    let dir = project("sp", &sp_checks());
    let _guard = Guard::new(&dir);
    let mut mcp = Mcp::start(&exe(), &dir, &[]);

    // US-047: commands that need a session, with none.
    for args in [
        &["exec", "--machine", "p1", "--code", "1"][..],
        &["inspect", "--machine", "p1"],
        &["reset"],
        &["run"],
    ] {
        let (code, _, err) = cli(&dir, args);
        assert_eq!(code, 4, "{args:?}: {err}");
        assert!(
            err.contains("shakedown session start --setup <name>") && err.contains("missing") && err.contains("ded30"),
            "{err}"
        );
    }
    for (tool, args) in [
        ("exec", json!({"machine": "p1", "code": "1"})),
        ("inspect", json!({"machine": "p1"})),
        ("reset", json!({})),
        ("run_start", json!({})),
    ] {
        let t = error_text(&mcp.call(tool, args), "no-session");
        assert!(
            t.contains("shakedown session start --setup <name>") && t.contains("missing") && t.contains("ded30"),
            "{t}"
        );
    }

    // US-003: a missing mod is refused before any process starts.
    let (code, _, err) = cli(&dir, &["session", "start", "--setup", "missing"]);
    assert_eq!(code, 4);
    assert!(err.contains("ghost") && err.contains("@skd_no_such_mod"), "{err}");
    let t = error_text(
        &mcp.call("session_start", json!({"setup": "missing"})),
        "precondition-failed",
    );
    assert_eq!(t, format!("precondition-failed: {}", err.trim()));
    assert_eq!(mcp.call("status", json!({})).data()["session"], Value::Null);
    assert!(game_pids().is_empty(), "a game process was started");

    // US-001 criterion 3: the same session shape from the command line and the tool.
    let (code, a, _) = cli_json(&dir, &["session", "start", "--setup", "sp"]);
    assert_eq!(code, 0, "{a}");
    let (code, _, _) = cli(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    let b = mcp.call("session_start", json!({"setup": "sp"})).data();
    assert_eq!(b["state"], "ready");
    assert_eq!(shape(&a), shape(&b));
    // US-001 criterion 1.
    let status = mcp.call("status", json!({})).data();
    assert_eq!(status["session"]["state"], "ready");
    let machines = status["session"]["machines"].as_array().unwrap().clone();
    assert_eq!(machines.len(), 1);
    assert_eq!(machines[0]["name"], "p1");
    assert_eq!(machines[0]["state"], "ready");

    // US-005: a second start is refused and leaves the session as it was.
    error_text(&mcp.call("session_start", json!({"setup": "sp"})), "session-exists");
    let (code, _, err) = cli(&dir, &["session", "start", "--setup", "sp"]);
    assert_eq!(code, 4);
    let id = b["id"].as_str().unwrap();
    assert!(
        err.contains(id) && err.contains("sp") && err.contains("shakedown session stop"),
        "{err}"
    );
    let after = mcp.call("status", json!({})).data();
    assert_eq!(after["session"]["state"], "ready");
    assert_eq!(
        without(&after["session"]["machines"], &["last_answer_at"]),
        without(&Value::Array(machines), &["last_answer_at"])
    );

    // NFR-PF-03 for a singleplayer p1.
    let sp = exec_p95(&dir, "p1");
    println!("NFR-PF-03 singleplayer p1: 95th percentile {sp:.3} s over 100");
    assert!(sp < 1.0);

    // US-048 criterion 2: exec, as the command line gives it.
    let (code, c, _) = cli_json(&dir, &["exec", "--machine", "p1", "--code", "1 + 1"]);
    assert_eq!(code, 0);
    let m = mcp.call("exec", json!({"machine": "p1", "code": "1 + 1"})).data();
    assert_eq!(without(&c, &["duration_s"]), without(&m, &["duration_s"]));
    assert_eq!(m["results"]["p1"]["state"], "done");

    // US-014 criterion 3 and US-049 criterion 1.
    let (code, _, err) = cli(&dir, &["exec", "--machine", "p3", "--code", "1"]);
    assert_eq!(code, 3);
    assert!(err.contains("p3") && err.contains("p1"), "{err}");
    let t = error_text(
        &mcp.call("exec", json!({"machine": "p9", "code": "1"})),
        "unknown-machine",
    );
    assert!(t.contains("p9") && t.contains("p1") && t.ends_with('.'), "{t}");
    let (code, _, err) = cli(&dir, &["exec", "--machine", "p1", "--code", "1 +"]);
    assert_eq!(code, 3);
    let t = error_text(
        &mcp.call("exec", json!({"machine": "p1", "code": "1 +"})),
        "compile-error",
    );
    assert!(
        err.contains("Invalid number in expression") && t.contains("Invalid number in expression"),
        "{err} / {t}"
    );

    // US-061 criteria 2 and 3: the error comes back beside the value.
    let (code, c, _) = cli_json(
        &dir,
        &["exec", "--machine", "p1", "--code", "[] call skdfx_fnc_throw; 7"],
    );
    assert_eq!(code, 0);
    assert_eq!(c["results"]["p1"]["state"], "done");
    assert_eq!(c["results"]["p1"]["value"], 7);
    let e = &c["errors"][0];
    assert_eq!(e["machine"], "p1");
    assert!(e["text"].as_str().unwrap().contains("_skdfx_undefined"));
    assert!(e["file"].as_str().unwrap().contains("fn_throw.sqf") && e["line"].is_number());
    let m = mcp.call("exec", json!({"machine": "p1", "code": "[] call skdfx_fnc_throw; 7"}));
    assert!(!m.is_error());
    assert_eq!(
        without(&m.data(), &["duration_s", "at"]),
        without(&c, &["duration_s", "at"])
    );
    let (code, errs, _) = cli_json(&dir, &["errors", "--machine", "p1"]);
    assert_eq!(code, 0);
    let listed = errs["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["at"] == e["at"])
        .expect("the error is listed");
    assert_eq!(listed["check_id"], Value::Null);
    assert!(listed["at"]["utc"].is_string());
    assert_eq!(mcp.call("errors", json!({"machine": "p1"})).data(), errs);

    // US-023 criterion 3 and US-062 criterion 3: waits through the command line and the tool.
    let t = Instant::now();
    let (code, c, _) = cli_json(
        &dir,
        &["wait", "--machine", "p1", "--condition", "false", "--ceiling", "2"],
    );
    let took = t.elapsed().as_secs_f64();
    assert_eq!(code, 0);
    assert_eq!(c["met"], false);
    assert!((1.9..4.0).contains(&took), "{took}");
    let m = mcp
        .call(
            "wait_until",
            json!({"machine": "p1", "condition": "false", "ceiling": 2}),
        )
        .data();
    assert_eq!(without(&m, &["waited_s"]), without(&c, &["waited_s"]));
    let t = Instant::now();
    let (code, c, _) = cli_json(
        &dir,
        &["wait", "--machine", "p1", "--condition", "_skdfx_undefined > 1"],
    );
    assert_eq!(code, 0);
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
    assert_eq!(c["met"], false);
    assert!(c["error"].as_str().unwrap().contains("_skdfx_undefined"), "{c}");
    let m = mcp
        .call(
            "wait_until",
            json!({"machine": "p1", "condition": "_skdfx_undefined > 1"}),
        )
        .data();
    assert_eq!(without(&m, &["waited_s"]), without(&c, &["waited_s"]));

    // OBS-06: expect, through both.
    let (code, c, _) = cli_json(
        &dir,
        &[
            "expect",
            "--machine",
            "p1",
            "--value",
            "1 + 1",
            "--op",
            "==",
            "--expected",
            "2",
        ],
    );
    assert_eq!(code, 0);
    assert_eq!(c["passed"], true);
    let m = mcp
        .call(
            "expect",
            json!({"machine": "p1", "value": "1 + 1", "op": "==", "expected": 2}),
        )
        .data();
    assert_eq!(without(&m, &["waited_s"]), without(&c, &["waited_s"]));

    // US-048 criterion 2: inspect.
    let (code, c, _) = cli_json(&dir, &["inspect", "--machine", "p1"]);
    assert_eq!(code, 0);
    let m = mcp.call("inspect", json!({"machine": "p1"})).data();
    assert_eq!(
        shape(&without(&c, &["nearby", "scripts", "errors"])),
        shape(&without(&m, &["nearby", "scripts", "errors"]))
    );
    assert_eq!(c["mission"], m["mission"]);
    assert_eq!(m["machine"]["name"], "p1");
    assert!(
        m["player"]["class"].is_string() && m["player"]["alive"] == true && m["player"].get("vehicle").is_some(),
        "{m}"
    );

    // US-051: a cancelled exec and a cancelled wait stop their work in the game.
    let start = Instant::now();
    let id = mcp.send_request(
        "tools/call",
        json!({"name": "exec", "arguments": {"machine": "p1", "code": "waitUntil {false}; 1", "ceiling": 50}}),
    );
    std::thread::sleep(Duration::from_secs(2));
    let (_, running, _) = cli_json(&dir, &["inspect", "--machine", "p1"]);
    let named = |v: &Value| {
        v["scripts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"].as_str().unwrap_or_default().starts_with("skd_exec"))
    };
    assert!(
        named(&running),
        "the exec's script is not listed while it runs: {running}"
    );
    while start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(100));
    }
    mcp.notify("notifications/cancelled", json!({"requestId": id, "reason": "US-051"}));
    assert!(no_exec_script_within(&mut mcp, "p1", Duration::from_secs(10)));
    assert_eq!(mcp.call("status", json!({})).data()["session"]["state"], "ready");
    let id = mcp.send_request(
        "tools/call",
        json!({"name": "wait_until", "arguments": {"machine": "p1", "condition": "false", "ceiling": 50}}),
    );
    std::thread::sleep(Duration::from_secs(2));
    mcp.notify("notifications/cancelled", json!({"requestId": id, "reason": "US-051"}));
    assert!(no_exec_script_within(&mut mcp, "p1", Duration::from_secs(10)));
    assert_eq!(
        mcp.call("exec", json!({"machine": "p1", "code": "1 + 1"})).data()["results"]["p1"]["value"],
        2
    );

    // US-031 criterion 4, US-050 and US-049 criterion 2: a run through the tools.
    let listed = mcp.call("list_checks", json!({"tags": ["m04"]})).data();
    let ids: Vec<Value> = listed["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].clone())
        .collect();
    let started = mcp.call("run_start", json!({"tags": ["m04"]}));
    assert!(started.took < Duration::from_secs(5), "{:?}", started.took);
    let started = started.data();
    assert_eq!(started["total"], ids.len());
    let run_id = started["run_id"].as_str().unwrap().to_string();
    let t = error_text(&mcp.call("run_result", json!({"run_id": run_id})), "unknown-run");
    assert!(t.contains("run_status"), "{t}");
    let calls = follow(&mut mcp, &run_id);
    let first = calls[0].data();
    assert!(!first["outcomes"].as_array().unwrap().is_empty() || first["state"] == "finished");
    let result = mcp.call("run_result", json!({"run_id": run_id})).data();
    let got: Vec<Value> = result["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["check_id"].clone())
        .collect();
    assert_eq!(got, ids);
    let failed = result["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["check_id"] == "fails")
        .unwrap();
    assert_eq!(failed["outcome"], "fail");

    // US-046 criterion 1 and US-050 criterion 2: status during a run, and progress while waiting.
    let started = mcp.call("run_start", json!({"ids": ["long", "long"]})).data();
    let run_id = started["run_id"].as_str().unwrap().to_string();
    std::thread::sleep(Duration::from_secs(3));
    let (_, c, _) = cli_json(&dir, &["status"]);
    let m = mcp.call("status", json!({})).data();
    for s in [&c, &m] {
        assert_eq!(s["run"]["id"], run_id.as_str(), "{s}");
        assert_eq!(s["run"]["total"], 2);
        assert_eq!(s["run"]["current_check"], "long");
        assert!(s["session"]["machines"][0]["last_answer_at"].is_string());
    }
    assert_eq!(c["run"], m["run"]);
    let calls = follow(&mut mcp, &run_id);
    let gap = longest_gap(&calls[0].notes, calls[0].took);
    println!(
        "run_status: first answer after {:?}, longest gap between progress notifications {gap:?}",
        calls[0].took
    );
    assert!(gap <= Duration::from_secs(5), "{gap:?}");
    assert_eq!(
        calls[0].data()["outcomes"].as_array().unwrap().len(),
        1,
        "run_status did not return when the first check ended"
    );

    // US-062 criteria 1 and 2.
    let started = mcp
        .call(
            "run_start",
            json!({"ids": ["raise", "string", "tryraise", "trystring"]}),
        )
        .data();
    let run_id = started["run_id"].as_str().unwrap().to_string();
    follow(&mut mcp, &run_id);
    let file = result_of(&mcp.call("run_result", json!({"run_id": run_id})).data());
    for id in ["raise", "tryraise"] {
        let c = by_id(&file, id);
        assert_eq!(c["outcome"], "fail");
        assert_eq!(c["reason"]["kind"], "script-error", "{c:#}");
        assert_eq!(c["reason"]["machine"], "p1");
        assert!(c["reason"]["message"].as_str().unwrap().contains("_skdfx_undefined"));
        assert_eq!(c["errors"].as_array().unwrap().len(), 1, "{:#}", c["errors"]);
        assert!(c["duration_s"].as_f64().unwrap() < 5.0, "{}", c["duration_s"]);
    }
    for id in ["string", "trystring"] {
        let c = by_id(&file, id);
        assert_eq!(c["outcome"], "fail");
        assert_eq!(c["reason"]["kind"], "check-error", "{c:#}");
        let msg = c["reason"]["message"].as_str().unwrap();
        assert!(
            msg.contains("a string, not a Boolean") && msg.contains("a String"),
            "{msg}"
        );
        assert!(c["errors"].as_array().unwrap().is_empty(), "{:#}", c["errors"]);
    }

    // US-061 criterion 1.
    let (code, c, _) = cli_json(&dir, &["run", "throw"]);
    assert_eq!(code, 1);
    let file = result_of(&c);
    let t = by_id(&file, "throw");
    assert_eq!(t["reason"]["kind"], "script-error");
    assert_eq!(t["reason"]["machine"], "p1");
    let msg = t["reason"]["message"].as_str().unwrap();
    assert!(
        msg.contains("_skdfx_undefined") && msg.contains("fn_throw.sqf"),
        "{msg}"
    );
    assert_eq!(t["evidence"]["failed_step"], json!(["throw on p1"]));
    assert_eq!(t["evidence"]["attached"][0]["value"], 7);
    assert_eq!(t["evidence"]["attached"][1]["value"], true);

    // US-048 criterion 2: run_result of a run the command line ran is its --json output.
    let (code, c, _) = cli_json(&dir, &["run", "fails"]);
    assert_eq!(code, 1);
    let m = mcp.call("run_result", json!({"run_id": c["run_id"]}));
    assert!(!m.is_error());
    assert_eq!(m.data(), c);
    let run_fails = c["run_id"].as_str().unwrap().to_string();

    // US-032 criterion 2: a selection that matches nothing leaves no run folder.
    let runs = || std::fs::read_dir(dir.join(".shakedown").join("runs")).unwrap().count();
    let before = runs();
    let (code, _, _) = cli(&dir, &["run", "--tag", "US-999"]);
    assert_eq!(code, 3);
    assert_eq!(runs(), before);
    assert_eq!(mcp.call("status", json!({})).data()["session"]["state"], "ready");

    // US-042 criterion 1 for p1, through both.
    let (code, c, _) = cli_json(
        &dir,
        &["logs", "--machine", "p1", "--check", "fails", "--run", &run_fails],
    );
    assert_eq!(code, 0);
    assert!(!c["lines"].as_array().unwrap().is_empty());
    let m = mcp
        .call(
            "logs",
            json!({"machine": "p1", "check_id": "fails", "run_id": run_fails}),
        )
        .data();
    assert_eq!(m, c);
    let (_, g, _) = cli_json(
        &dir,
        &[
            "logs",
            "--machine",
            "p1",
            "--check",
            "fails",
            "--run",
            &run_fails,
            "--grep",
            "shakedown",
            "--last",
            "1",
        ],
    );
    let lines = g["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].as_str().unwrap().contains("shakedown"));

    // US-048 criterion 3: a result over 20,000 characters is a file.
    mcp.call(
        "exec",
        json!({"machine": "p1", "code": "for \"_i\" from 1 to 3000 do { diag_log format [\"skd m04 filler line %1 of a long log to read\", _i] }"}),
    )
    .data();
    std::thread::sleep(Duration::from_secs(2));
    let a = mcp.call("logs", json!({"machine": "p1", "last": 5000}));
    let text = serde_json::to_string(&a.data()).unwrap();
    assert!(text.len() < 20_000, "{}", text.len());
    let path = a.data()["path"]
        .as_str()
        .or(a.data()["result_path"].as_str())
        .map(String::from);
    let path = path.expect("a path to the whole result");
    assert!(std::fs::metadata(&path).unwrap().len() > 20_000);

    let stopped = mcp.call("session_stop", json!({})).data();
    assert_eq!(stopped["state"], "stopped");
    assert_eq!(mcp.call("status", json!({})).data()["session"], Value::Null);
}

#[test]
#[ignore = "starts the game"]
fn us_013_us_014_us_023_us_042_us_050_us_062_nfr_pf_03_on_a_dedicated_session() {
    let checks = [
        (
            "on42",
            "// @check US-013 a value and an object cross machines\nprivate _r = [\"p2\", {[_this select 0, name player]}, [42]] call skd_fnc_on;\n[(_r select 0), \"==\", 42, \"the first element\"] call skd_fnc_expect;\nprivate _o = [\"server\", {\"Land_Can_V3_F\" createVehicle [100, 120, 0]}] call skd_fnc_on;\nprivate _back = [\"p1\", {netId (_this select 0)}, [_o]] call skd_fnc_on;\n[_back, \"==\", netId _o, \"the same object on p1\"] call skd_fnc_expect;\n",
        ),
        (
            "each",
            "// @check US-013 one key per machine\nprivate _m = [\"all\", {clientOwner}] call skd_fnc_onEach;\n[\"keys\", keys _m] call skd_fnc_evidence;\n",
        ),
        (
            "p3",
            "// @check US-014 a machine the session lacks\n[\"p3\", {1}] call skd_fnc_on;\n",
        ),
        (
            "syntax",
            "// @check US-014 code that does not compile\n[\"p1\", {1 + }] call skd_fnc_on;\n",
        ),
        (
            "flag",
            "// @check US-023 a wait goes on as soon as it holds\n[\"server\", {[true, 3] call skdfx_fnc_setFlagLater}] call skd_fnc_on;\n[\"wait\", { [\"p1\", {!isNil \"skdfx_flag\"}, \"flag reached p1\", 10] call skd_fnc_waitUntil }] call skd_fnc_step;\n",
        ),
        (
            "try",
            "// @check US-023 a try wait returns false at its ceiling\nprivate _r = [\"p1\", {false}, \"never\", 1] call skd_fnc_tryWaitUntil;\n[_r, \"==\", false, \"the try wait\"] call skd_fnc_expect;\n",
        ),
        ("raise", RAISE),
        ("string", STRING),
    ];
    let dir = project("ded", &checks);
    let _guard = Guard::new(&dir);

    // US-050 criterion 2: a start that outlasts the tool's wait returns starting; status then
    // shows it ready. The wait is cut to 15 s so the 45 s start outlasts it.
    let mut mcp = Mcp::start(&exe(), &dir, &[("SHAKEDOWN_MCP_START_WAIT_S", "15")]);
    let a = mcp.call_progress("session_start", json!({"setup": "ded"}));
    assert!(a.took < Duration::from_secs(25), "{:?}", a.took);
    assert_eq!(a.data()["state"], "starting", "{}", a.data());
    let gap = longest_gap(&a.notes, a.took);
    assert!(gap <= Duration::from_secs(5), "{gap:?}");
    let mut ready = false;
    for _ in 0..90 {
        let s = mcp.call("status", json!({})).data();
        if s["session"]["state"] == "ready" {
            ready = true;
            break;
        }
        assert_eq!(s["session"]["state"], "starting", "{s}");
        std::thread::sleep(Duration::from_secs(2));
    }
    assert!(ready, "the session did not become ready");

    // NFR-PF-03 and US-013 criterion 4 for each machine kind of a dedicated session.
    for machine in ["server", "p1", "hc1"] {
        let p = exec_p95(&dir, machine);
        println!("NFR-PF-03 dedicated {machine}: 95th percentile {p:.3} s over 100");
        assert!(p < 1.0);
    }

    // US-013 criterion 3.
    let (code, c, _) = cli_json(&dir, &["exec", "--machine", "server", "--code", "sleep 1; 1 + 1"]);
    assert_eq!(code, 0);
    assert_eq!(c["results"]["server"]["state"], "done");
    assert_eq!(c["results"]["server"]["value"], 2);
    assert!(c["results"]["server"]["duration_s"].as_f64().unwrap() >= 1.0);
    let m = mcp
        .call("exec", json!({"machine": "server", "code": "sleep 1; 1 + 1"}))
        .data();
    assert_eq!(without(&m, &["duration_s"]), without(&c, &["duration_s"]));
    let (_, all, _) = cli_json(&dir, &["exec", "--machine", "all", "--code", "clientOwner"]);
    assert_eq!(all["results"].as_object().unwrap().len(), 4, "{all}");

    // US-023 criterion 3 and US-062 criterion 3 on the server and a client.
    let (code, c, _) = cli_json(
        &dir,
        &["wait", "--machine", "server", "--condition", "false", "--ceiling", "2"],
    );
    assert_eq!(code, 0);
    assert_eq!(c["met"], false);
    let m = mcp
        .call(
            "wait_until",
            json!({"machine": "server", "condition": "false", "ceiling": 2}),
        )
        .data();
    assert_eq!(without(&m, &["waited_s"]), without(&c, &["waited_s"]));
    let t = Instant::now();
    let (code, c, _) = cli_json(
        &dir,
        &["wait", "--machine", "p1", "--condition", "_skdfx_undefined > 1"],
    );
    assert_eq!(code, 0);
    assert!(t.elapsed() < Duration::from_secs(2));
    assert!(c["error"].as_str().unwrap().contains("_skdfx_undefined"), "{c}");

    // The checks: US-013 criteria 1 and 2, US-014 criteria 1 and 2, US-023 criteria 1 and 2, and
    // US-062 criteria 1 and 2 through a wait on a client the lead polls through the runner.
    let ids: Vec<&str> = checks.iter().map(|(id, _)| *id).collect();
    let mut args = vec!["run"];
    args.extend(ids.iter());
    let (_, c, _) = cli_json(&dir, &args);
    let file = result_of(&c);
    assert_eq!(by_id(&file, "on42")["outcome"], "pass", "{:#}", by_id(&file, "on42"));
    let each = by_id(&file, "each");
    assert_eq!(each["outcome"], "pass");
    let mut keys: Vec<String> = each["evidence"]["attached"][0]["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap().to_string())
        .collect();
    keys.sort();
    assert_eq!(keys, ["hc1", "p1", "p2", "server"]);
    let p3 = by_id(&file, "p3");
    assert_eq!(p3["reason"]["kind"], "check-error");
    let msg = p3["reason"]["message"].as_str().unwrap();
    assert!(msg.contains("p3") && msg.contains("server, p1, p2, hc1"), "{msg}");
    let syntax = by_id(&file, "syntax");
    assert_eq!(syntax["reason"]["kind"], "check-error");
    assert!(
        syntax["reason"]["message"].as_str().unwrap().contains("line"),
        "{syntax:#}"
    );
    let flag = by_id(&file, "flag");
    assert_eq!(flag["outcome"], "pass", "{flag:#}");
    let wait = flag["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["path"] == json!(["wait"]))
        .unwrap();
    assert!(wait["duration_s"].as_f64().unwrap() < 5.0);
    assert_eq!(by_id(&file, "try")["outcome"], "pass");
    let raise = by_id(&file, "raise");
    assert_eq!(raise["reason"]["kind"], "script-error", "{raise:#}");
    assert_eq!(raise["reason"]["machine"], "p1");
    assert_eq!(raise["errors"].as_array().unwrap().len(), 1, "{:#}", raise["errors"]);
    let step = raise["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["path"] == json!(["wait"]))
        .unwrap();
    assert!(step["duration_s"].as_f64().unwrap() < 2.0, "{step}");
    let string = by_id(&file, "string");
    assert_eq!(string["reason"]["kind"], "check-error");
    assert!(string["reason"]["message"].as_str().unwrap().contains("a String"));

    // US-042 criterion 1 for p2.
    let run_id = c["run_id"].as_str().unwrap();
    let (code, l, _) = cli_json(&dir, &["logs", "--machine", "p2", "--check", "on42", "--run", run_id]);
    assert_eq!(code, 0);
    let lines = l["lines"].as_array().unwrap();
    assert!(lines.first().unwrap().as_str().unwrap().contains("start"));
    assert!(lines.last().unwrap().as_str().unwrap().contains("end"));
    let m = mcp
        .call("logs", json!({"machine": "p2", "check_id": "on42", "run_id": run_id}))
        .data();
    assert_eq!(m, l);
    let (_, g, _) = cli_json(
        &dir,
        &[
            "logs",
            "--machine",
            "p2",
            "--check",
            "on42",
            "--run",
            run_id,
            "--grep",
            "\\[shakedown\\]",
            "--last",
            "1",
        ],
    );
    assert_eq!(g["lines"].as_array().unwrap().len(), 1);

    assert_eq!(mcp.call("session_stop", json!({})).data()["state"], "stopped");
}

/// The Steam ID and the two names of the account Steam last logged in with, from this machine's
/// `loginusers.vdf`, read at run time.
fn identity() -> Vec<String> {
    let out = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let steam = text
        .lines()
        .find(|l| l.contains("SteamPath"))
        .and_then(|l| l.split("REG_SZ").nth(1))
        .unwrap()
        .trim()
        .to_string();
    let vdf = std::fs::read_to_string(Path::new(&steam).join("config").join("loginusers.vdf")).unwrap();
    let mut accounts: Vec<[String; 5]> = Vec::new();
    for line in vdf.lines() {
        let parts: Vec<&str> = line.trim().split('"').collect();
        if parts.len() == 3 && parts[1].len() == 17 {
            accounts.push([
                parts[1].to_string(),
                String::new(),
                String::new(),
                String::new(),
                "0".into(),
            ]);
        } else if parts.len() >= 5
            && let Some(a) = accounts.last_mut()
        {
            match parts[1] {
                "AccountName" => a[1] = parts[3].to_string(),
                "PersonaName" => a[2] = parts[3].to_string(),
                "MostRecent" => a[3] = parts[3].to_string(),
                "timestamp" => a[4] = parts[3].to_string(),
                _ => {}
            }
        }
    }
    let found: Vec<String> = accounts
        .iter()
        .find(|a| a[3] == "1")
        .or_else(|| accounts.iter().max_by_key(|a| a[4].parse::<u64>().unwrap_or(0)))
        .map(|a| a[..3].to_vec())
        .unwrap_or_default();
    assert_eq!(found.len(), 3, "no most recent account in loginusers.vdf");
    assert!(found.iter().all(|v| !v.is_empty()));
    found
}

/// Whether `text` holds `word` as a whole word, case-sensitively, as `LOG-02` matches it.
fn holds(text: &str, word: &str) -> bool {
    let w = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(w) && !after.is_some_and(w)
    })
}

/// How many of `values` `text` holds; the values are never printed.
fn count_held(text: &str, values: &[String]) -> usize {
    values.iter().filter(|v| holds(text, v)).count()
}

#[test]
#[ignore = "starts the game"]
fn def_10_def_12_us_004_a_failed_start_is_kept_by_status_without_the_steam_identity() {
    let dir = project("failed", &[("pass", PASS)]);
    let _guard = Guard::new(&dir);
    let who = identity();
    let mut mcp = Mcp::start(&exe(), &dir, &[]);

    // A dedicated start that cannot meet its 30 s ceiling: the clients have logged in by then.
    let (code, out, err) = cli(&dir, &["session", "start", "--setup", "ded30", "--json"]);
    assert_eq!(code, 4, "{err}");
    let session: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(session["state"], "failed");
    let reason = session["reason"].clone();
    assert!(
        !reason["rpt_tail"].as_array().unwrap().is_empty(),
        "the failed start kept no RPT lines"
    );

    // DEF-10: status keeps the failed start and its reason, through both.
    let (_, c, _) = cli_json(&dir, &["status"]);
    let m = mcp.call("status", json!({})).data();
    for s in [&c, &m] {
        assert_eq!(s["session"]["state"], "failed", "{s}");
        assert_eq!(s["session"]["id"], session["id"]);
        assert_eq!(s["session"]["reason"], reason);
    }
    let (_, human, _) = cli(&dir, &["status"]);

    // DEF-12: no Steam ID and no Steam name in what the failed start printed and kept.
    let mut held = count_held(&out, &who) + count_held(&err, &who) + count_held(&human, &who);
    held += count_held(&c.to_string(), &who) + count_held(&m.to_string(), &who);
    let folder = PathBuf::from(session["folder"].as_str().unwrap());
    let mut rpts = 0;
    let mut redacted = 0;
    let mut stack = vec![folder.join("machines")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rpt")) {
                rpts += 1;
                let text = String::from_utf8_lossy(&std::fs::read(&p).unwrap()).to_string();
                held += count_held(&text, &who);
                // What the redaction replaced: evidence that the RPTs held the identity once.
                redacted += text.matches("<steamid>").count() + text.matches("<steamname>").count();
            }
        }
    }
    println!(
        "DEF-12: searched the output, status and {rpts} RPTs for {} identity values: {held} found; {redacted} places redacted",
        who.len()
    );
    assert!(rpts >= 2, "only {rpts} RPTs");
    assert_eq!(held, 0);

    // DEF-10: stop clears it; a start after a failed one starts as usual.
    let (code, cleared, _) = cli_json(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    assert_eq!(cleared["id"], session["id"]);
    assert_eq!(mcp.call("status", json!({})).data()["session"], Value::Null);
    let (code, _, _) = cli(&dir, &["session", "start", "--setup", "ded30"]);
    assert_eq!(code, 4);
    let started = mcp.call("session_start", json!({"setup": "sp"})).data();
    assert_eq!(started["state"], "ready");
    let s = mcp.call("status", json!({})).data();
    assert_eq!(s["session"]["id"], started["id"]);
    assert_eq!(mcp.call("session_stop", json!({})).data()["state"], "stopped");
}
