//! M11's tests: a client that joins a dedicated session late (`US-066`), leaves and joins again
//! (`US-067`), a join that never becomes ready (`US-068`), a join or leave of a machine that
//! cannot (`US-069`), and the measurement of `SPK-18`. They drive the installed `shakedown`, the
//! fixture `@skd_fixture` and the game, so they run only on the test machine, by hand, one at a
//! time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m11 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// The checks of M11, by file name.
fn checks() -> Vec<(&'static str, String)> {
    vec![
        (
            "late",
            r#"// @check US-066 a late client joins and sees what the others see
// @timeout 240
private _crate = ["a crate with a public flag on the server", {
    private _c = ["skdfx_Crate", AGLToASL [130, 110, 0], 0] call skd_fnc_create;
    ["server", { (_this select 0) setVariable ["skd_m11_flag", 7, true] }, [_c]] call skd_fnc_on;
    _c
}] call skd_fnc_step;
["machines before", [] call skd_fnc_machines] call skd_fnc_evidence;
private _joined = ["p3"] call skd_fnc_join;
[_joined, "==", true, "skd_fnc_join returns true"] call skd_fnc_expect;
[[] call skd_fnc_machines, "==", ["server", "p1", "p2", "p3"], "the machines after the join"] call skd_fnc_expect;
["every machine agrees", {
    [["server", "p1", "p2", "p3"], { getPosASL (_this select 0) }, "the crate's position", 0.1, 30, [_crate]] call skd_fnc_expectAgree;
    [["server", "p1", "p2", "p3"], { (_this select 0) getVariable ["skd_m11_flag", -1] }, "the crate's flag", 0, 30, [_crate]] call skd_fnc_expectAgree;
}] call skd_fnc_step;
"#
            .into(),
        ),
        (
            "after",
            r#"// @check US-066 US-067 the next check starts with the machines the session started with
["machines", [] call skd_fnc_machines] call skd_fnc_evidence;
[[] call skd_fnc_machines, "==", ["server", "p1", "p2"], "the machines"] call skd_fnc_expect;
"#
            .into(),
        ),
        (
            "leave",
            r#"// @check US-067 p2 leaves, the fixture sees it go, and p2 joins again
// @timeout 240
private _left = ["p2"] call skd_fnc_leave;
[_left, "==", true, "skd_fnc_leave returns true"] call skd_fnc_expect;
[[] call skd_fnc_machines, "==", ["server", "p1"], "the machines after the leave"] call skd_fnc_expect;
["server", { missionNamespace getVariable ["skdfx_disconnected", []] }, "contains", "p2", "the fixture's record of who left", 10] call skd_fnc_expectOn;
["p2"] call skd_fnc_join;
[[] call skd_fnc_machines, "==", ["server", "p1", "p2"], "the machines after the join"] call skd_fnc_expect;
["p2's new unit", {
    private _unit = ["p2"] call skd_fnc_player;
    [isNull _unit, "==", false, "skd_fnc_player gives p2's new unit"] call skd_fnc_expect;
    ["p2", { name player }, "==", "p2", "p2 controls its unit", 10] call skd_fnc_expectOn;
}] call skd_fnc_step;
"#
            .into(),
        ),
        (
            "leaveonly",
            r#"// @check US-067 p2 leaves and stays out
["p2"] call skd_fnc_leave;
"#
            .into(),
        ),
        (
            "hang",
            r#"// @check US-068 p3 joins with a ceiling of 30 s
// @timeout 120
["p3", 30] call skd_fnc_join;
"#
            .into(),
        ),
        (
            "pass",
            r#"// @check US-068 a check that passes
[1, "==", 1, "one is one"] call skd_fnc_expect;
"#
            .into(),
        ),
        (
            "bodies",
            r#"// @check US-067 RST-01 the check after a leave sees no unit a client left behind
private _dead = ["server", { allDeadMen apply { netId _x } }] call skd_fnc_on;
["bodies on the server", _dead] call skd_fnc_evidence;
[_dead, "==", [], "the bodies on the server"] call skd_fnc_expect;
private _near = ["p2", { (nearestObjects [player, ["CAManBase"], 3]) - [player] apply { netId _x } }] call skd_fnc_on;
[_near, "==", [], "the units within 3 m of p2's player"] call skd_fnc_expect;
"#
            .into(),
        ),
        ("w-server", wrong(r#"["server"] call skd_fnc_leave;"#)),
        ("w-hc1", wrong(r#"["hc1"] call skd_fnc_leave;"#)),
        ("w-p1", wrong(r#"["p1"] call skd_fnc_join;"#)),
        ("w-p9", wrong(r#"["p9"] call skd_fnc_join;"#)),
        ("w-twice", wrong("[\"p2\"] call skd_fnc_leave;\n[\"p2\"] call skd_fnc_leave;")),
        ("w-sp", wrong(r#"["p1"] call skd_fnc_leave;"#)),
        (
            "spike",
            r#"// @check SPK-18 p3 joins, leaves and joins again
// @timeout 400
["p3", 60] call skd_fnc_join;
["p3"] call skd_fnc_leave;
["p3", 60] call skd_fnc_join;
"#
            .into(),
        ),
    ]
}

fn wrong(body: &str) -> String {
    format!("// @check US-069 a machine that cannot join or leave\n{body}\n")
}

/// A project with the setups `ded` (two clients and the late `p3`) and `sp`, both loading
/// `@skd_fixture`, and the checks of M11.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m11-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[setups.ded]
mode = "dedicated"
clients = 2
late = 1
mods = ["fixture"]

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks() {
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

/// Whether a process with this id runs.
fn runs(pid: u64) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
}

/// Suspends a process by its id, through `NtSuspendProcess`, as an outside tool does with the id
/// `status` gives (`INS-04`).
fn suspend(pid: u64) {
    let script = format!(
        "Add-Type -Name N -Namespace W -MemberDefinition '[DllImport(\"ntdll.dll\")] public static extern int NtSuspendProcess(IntPtr h);'; $p = Get-Process -Id {pid}; [W.N]::NtSuspendProcess($p.Handle) | Out-Null"
    );
    let ok = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "NtSuspendProcess of {pid}");
}

/// The session's machines in `status`: name, start number and process id.
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

fn names(dir: &Path) -> Vec<String> {
    machines(dir).into_iter().map(|m| m.0).collect()
}

/// Stops the session it started when dropped, also when a test fails, and then every game
/// process of it the test saw, by id.
struct Session {
    dir: PathBuf,
    pids: Arc<Mutex<Vec<u64>>>,
}

impl Session {
    fn see(&self) {
        let mut pids = self.pids.lock().unwrap();
        for (_, _, pid) in machines(&self.dir) {
            if pid != 0 && !pids.contains(&pid) {
                pids.push(pid);
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = cmd(&self.dir, &["session", "stop"]);
        for pid in self.pids.lock().unwrap().iter() {
            if runs(*pid) {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

fn start(dir: &Path, setup: &str) -> Session {
    steam();
    let (code, s, err) = cmd(dir, &["session", "start", "--setup", setup]);
    assert_eq!(code, 0, "{s} {err}");
    let session = Session {
        dir: dir.to_path_buf(),
        pids: Arc::new(Mutex::new(vec![])),
    };
    session.see();
    session
}

/// Runs checks by id and returns the result file's content.
fn run(dir: &Path, ids: &[&str]) -> Value {
    let mut args = vec!["run"];
    args.extend_from_slice(ids);
    let (_, out, err) = cmd(dir, &args);
    let file = out["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result file: {out} {err}"));
    serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
}

/// Starts `shakedown run` of `ids` without waiting for it.
fn spawn_run(dir: &Path, ids: &[&str]) -> std::process::Child {
    Command::new(exe())
        .arg("run")
        .args(ids)
        .arg("--json")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// Waits for a spawned run, at most `ceiling`, and returns its result file's content.
fn finish(mut child: std::process::Child, ceiling: Duration) -> Value {
    let deadline = Instant::now() + ceiling;
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("the run did not end within {ceiling:?}");
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    let mut out = String::new();
    std::io::Read::read_to_string(child.stdout.as_mut().unwrap(), &mut out).unwrap();
    let out: Value = serde_json::from_str(&out).unwrap();
    serde_json::from_str(&std::fs::read_to_string(out["result_file"].as_str().unwrap()).unwrap()).unwrap()
}

fn check(result: &Value, n: usize) -> &Value {
    &result["checks"][n]
}

fn outcome(c: &Value) -> (String, String, String) {
    (
        c["outcome"].as_str().unwrap_or_default().to_string(),
        c["reason"]["kind"].as_str().unwrap_or_default().to_string(),
        c["reason"]["message"].as_str().unwrap_or_default().to_string(),
    )
}

/// The duration of the step at `path` in a check execution.
fn step(c: &Value, path: &[&str]) -> Option<f64> {
    c["steps"]
        .as_array()?
        .iter()
        .find(|s| s["path"] == json!(path))
        .and_then(|s| s["duration_s"].as_f64())
}

/// The starts of `name` the result file lists, with their end states.
fn starts(result: &Value, name: &str) -> Vec<(u64, String)> {
    result["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["name"] == name)
        .map(|m| {
            (
                m["start"].as_u64().unwrap(),
                m["end_state"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_066_us_067_mch_01_a_client_joins_late_leaves_and_joins_again() {
    let dir = project("feature");
    let session = start(&dir, "ded");

    // US-066 1: the session starts server, p1 and p2 only.
    assert_eq!(names(&dir), ["server", "p1", "p2"]);

    // US-066 2 and 3: p3 joins during the check, and status lists it with start 1.
    let child = spawn_run(&dir, &["late", "after"]);
    let mut seen = None;
    for _ in 0..600 {
        if let Some(p3) = machines(&dir).into_iter().find(|m| m.0 == "p3") {
            seen = Some(p3);
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let p3 = seen.expect("status never listed p3 during the check");
    assert_eq!(p3.1, 1, "p3's start number");
    assert!(p3.2 != 0 && runs(p3.2), "p3's process {}", p3.2);
    session.see();
    let result = finish(child, Duration::from_secs(400));
    let late = check(&result, 0);
    assert_eq!(outcome(late).0, "pass", "{late:#}");
    let join = step(late, &["join p3"]).expect("a step join p3");
    eprintln!("US-066: join p3 took {join} s");
    assert!(
        late["evidence"]["attached"][0]["value"] == json!(["server", "p1", "p2"]),
        "skd_fnc_machines before the join: {late:#}"
    );
    let p3m = result["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "p3")
        .expect("p3 in the result file")
        .clone();
    assert_eq!(p3m["start"], 1);
    assert_eq!(
        p3m["mods"][0]["version"].as_str().map(|v| !v.is_empty()),
        Some(true),
        "{p3m}"
    );

    // US-066 4: the next check starts without p3, its process ended; p1 and p2 keep start 1.
    let after = check(&result, 1);
    assert_eq!(outcome(after).0, "pass", "{after:#}");
    assert!(!runs(p3.2), "p3's process still runs");
    let now = machines(&dir);
    assert_eq!(
        now.iter().map(|m| m.0.as_str()).collect::<Vec<_>>(),
        ["server", "p1", "p2"]
    );
    assert!(now.iter().all(|m| m.1 == 1), "{now:?}");
    assert_eq!(starts(&result, "p3"), [(1, "stopped".to_string())]);

    // US-067 1 to 3 and 5: p2 leaves, the fixture sees it go, p2 joins again with start 2.
    let p2 = machines(&dir).into_iter().find(|m| m.0 == "p2").unwrap();
    let result = run(&dir, &["leave", "after"]);
    session.see();
    let leave = check(&result, 0);
    assert_eq!(outcome(leave).0, "pass", "{leave:#}");
    let left = step(leave, &["leave p2"]).expect("a step leave p2");
    assert!(left <= 20.0, "the leave took {left} s");
    assert!(step(leave, &["join p2"]).is_some(), "a step join p2");
    assert!(!runs(p2.2), "p2's first process still runs");
    assert_eq!(
        starts(&result, "p2"),
        [(1, "stopped".to_string()), (2, "ready".to_string())]
    );
    // US-067 4: after a check that made p2 join again, the next check has that same start.
    let after = check(&result, 1);
    assert_eq!(outcome(after).0, "pass", "{after:#}");
    let p2now = machines(&dir).into_iter().find(|m| m.0 == "p2").unwrap();
    assert_eq!(p2now.1, 2);

    // US-067 4: after a check that left p2 out, the next check starts with p2 joined again and
    // the reset verified on every machine.
    let result = run(&dir, &["leaveonly", "after"]);
    session.see();
    assert_eq!(outcome(check(&result, 0)).0, "pass", "{result:#}");
    assert_eq!(outcome(check(&result, 1)).0, "pass", "{result:#}");
    let p2now = machines(&dir).into_iter().find(|m| m.0 == "p2").unwrap();
    assert_eq!(p2now.1, 3);
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready");

    // US-066 5: the same check through MCP ends the same way, no call over 60 s.
    let mut mcp = support::Mcp::start(&exe(), &dir, &[]);
    let started = mcp.call("run_start", json!({"ids": ["late"]}));
    assert!(started.took < Duration::from_secs(60));
    let run_id = started.data()["run_id"].as_str().unwrap().to_string();
    let mut state = String::new();
    for _ in 0..20 {
        let status = mcp.call("run_status", json!({"run_id": run_id, "wait_s": 50}));
        assert!(
            status.took < Duration::from_secs(60),
            "run_status took {:?}",
            status.took
        );
        state = status.data()["state"].as_str().unwrap_or_default().to_string();
        session.see();
        if state != "running" {
            break;
        }
    }
    assert_eq!(state, "finished");
    let result = mcp.call("run_result", json!({"run_id": run_id}));
    assert_eq!(result.data()["outcomes"][0]["outcome"], "pass", "{}", result.text());
    drop(session);
}

/// `US-067` 4, `RST-01`, `MCH-01`: the unit a client leaves behind is deleted by the next reset,
/// so a client joined again does not stand among the bodies of its earlier starts, and the reset
/// after each rejoin verifies, five times over in one session, with a leave alone and a leave and
/// a join in one check (m12-1).
#[test]
#[ignore = "drives the game on the test machine"]
fn us_067_rst_01_the_unit_a_client_left_is_deleted_by_the_next_reset_5_of_5() {
    let dir = project("bodies");
    let session = start(&dir, "ded");
    for round in 0..5 {
        let first = if round == 2 { "leave" } else { "leaveonly" };
        let result = run(&dir, &[first, "bodies", "pass", "bodies"]);
        session.see();
        for n in 0..4 {
            let c = check(&result, n);
            assert_eq!(outcome(c).0, "pass", "round {round}, check {n}: {c:#}");
        }
    }
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready");
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_068_mch_01_a_joining_client_that_never_becomes_ready() {
    let dir = project("hang");
    let session = start(&dir, "ded");

    // 1: p3 suspended right after its launch: invalid / machine-lost within 30 s plus the grace.
    let child = spawn_run(&dir, &["hang", "pass"]);
    let mut p3 = None;
    for _ in 0..600 {
        if let Some(m) = machines(&dir).into_iter().find(|m| m.0 == "p3" && m.2 != 0) {
            suspend(m.2);
            p3 = Some(m);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let p3 = p3.expect("p3 was never launched");
    session.pids.lock().unwrap().push(p3.2);
    let result = finish(child, Duration::from_secs(300));
    let hang = check(&result, 0);
    let (o, kind, message) = outcome(hang);
    assert_eq!((o.as_str(), kind.as_str()), ("invalid", "machine-lost"), "{hang:#}");
    assert!(message.contains("p3"), "{message}");
    assert_eq!(hang["reason"]["machine"], "p3");
    let took = step(hang, &["join p3"]).unwrap_or(f64::MAX);
    assert!(took <= 40.0 + 2.0, "the join ended after {took} s");
    let tail = hang["evidence"]["attached"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"].as_str().unwrap_or_default().contains("RPT"))
        .unwrap_or_else(|| panic!("no RPT lines: {hang:#}"));
    assert!(tail["value"].as_array().is_some_and(|l| l.len() <= 20));
    // 2: p3's process is gone, the session is ready with server, p1 and p2, the next check passes.
    assert!(!runs(p3.2), "p3's process still runs");
    assert_eq!(outcome(check(&result, 1)).0, "pass", "{result:#}");
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready");
    assert_eq!(names(&dir), ["server", "p1", "p2"]);

    // 3: p3's process ends during its join: invalid / machine-lost at once, with its exit code.
    let child = spawn_run(&dir, &["hang"]);
    let mut killed = None;
    for _ in 0..600 {
        if let Some(m) = machines(&dir).into_iter().find(|m| m.0 == "p3" && m.2 != 0) {
            std::thread::sleep(Duration::from_secs(3));
            let _ = Command::new("taskkill").args(["/PID", &m.2.to_string(), "/F"]).output();
            killed = Some((m, Instant::now()));
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let (m, at) = killed.expect("p3 was never launched");
    session.pids.lock().unwrap().push(m.2);
    let result = finish(child, Duration::from_secs(300));
    let hang = check(&result, 0);
    let (o, kind, message) = outcome(hang);
    assert_eq!((o.as_str(), kind.as_str()), ("invalid", "machine-lost"), "{hang:#}");
    assert!(message.contains("p3") && message.contains("exit code"), "{message}");
    let ended: f64 = hang["duration_s"].as_f64().unwrap();
    eprintln!(
        "US-068 3: the check ended {:?} after the kill, {ended} s in all",
        at.elapsed()
    );
    assert!(step(hang, &["join p3"]).unwrap_or(f64::MAX) < 20.0, "{hang:#}");
    drop(session);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_069_mch_01_dm_04_joining_or_leaving_a_machine_that_cannot() {
    let dir = project("wrong");
    let session = start(&dir, "ded");
    let result = run(&dir, &["w-server", "w-hc1", "w-p1", "w-p9", "w-twice", "after"]);
    session.see();
    for (n, (machine, why)) in [
        ("server", "server is the dedicated server"),
        ("hc1", "no game client hc1"),
        ("p1", "p1 is in the session already"),
        ("p9", "no game client p9"),
        ("p2", "p2 is not in the session"),
    ]
    .into_iter()
    .enumerate()
    {
        let c = check(&result, n);
        let (o, kind, message) = outcome(c);
        assert_eq!((o.as_str(), kind.as_str()), ("fail", "check-error"), "{c:#}");
        assert!(message.contains(why) && message.contains(machine), "{message}");
        assert!(
            message.contains("can join now:") && message.contains("can leave now:"),
            "{message}"
        );
    }
    // 3: the session is ready with the machines it had.
    assert_eq!(outcome(check(&result, 5)).0, "pass", "{result:#}");
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready");
    drop(session);

    // 2: in singleplayer.
    let session = start(&dir, "sp");
    let result = run(&dir, &["w-sp"]);
    let (o, kind, message) = outcome(check(&result, 0));
    assert_eq!((o.as_str(), kind.as_str()), ("fail", "check-error"));
    assert!(message.contains("needs a dedicated session"), "{message}");
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready");
    drop(session);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn spk_18_a_third_client_joins_leaves_and_joins_again_10_of_10() {
    let dir = project("spike");
    let session = start(&dir, "ded");
    let ids = vec!["spike"; 10];
    let result = run(&dir, &ids);
    session.see();
    let mut joins = vec![];
    let mut leaves = vec![];
    for c in result["checks"].as_array().unwrap() {
        assert_eq!(outcome(c).0, "pass", "{c:#}");
        let steps: Vec<(String, f64)> = c["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["path"][0].as_str().unwrap().to_string(),
                    s["duration_s"].as_f64().unwrap(),
                )
            })
            .collect();
        for (path, d) in steps {
            if path.starts_with("join") {
                assert!(d < 60.0, "a join took {d} s");
                joins.push(d);
            } else {
                assert!(d < 20.0, "a leave took {d} s");
                leaves.push(d);
            }
        }
    }
    eprintln!("SPK-18 joins: {joins:?}");
    eprintln!("SPK-18 leaves: {leaves:?}");
    drop(session);
}
