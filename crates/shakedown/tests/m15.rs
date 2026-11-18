//! M15's tests: a check hands its situation over to a manual tester (`US-083`, `MAN-01`,
//! `RUN-04`, `STP-02`, `SES-06`, `INS-02`), and a hand over to a machine with no screen is the
//! author's mistake (`US-084`, `DM-04`). They drive the installed `shakedown` and the game, so they
//! run only on the test machine, by hand, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m15 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended: the session by `session stop`, and any process of it left
//! after that by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::Value;
use support::repo;

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// A project with the e2e project's file and setups, the e2e hand over check, and these checks.
fn project(name: &str, checks: &[(&str, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m15-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(r"tests\handover")).unwrap();
    // The e2e file's relative paths name the install folder from the repository's e2e folder.
    let e2e = repo().join("e2e");
    let toml = std::fs::read_to_string(e2e.join("shakedown.toml"))
        .unwrap()
        .replace(r"'..\target", &format!("'{}", repo().join("target").display()));
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    std::fs::copy(
        e2e.join(r"tests\handover\US-083-crate.sqf"),
        dir.join(r"tests\handover\US-083-crate.sqf"),
    )
    .unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
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

fn pids(dir: &Path) -> Vec<u64> {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["pid"].as_u64())
        .collect()
}

/// Stops the session when dropped, also when the test panics; any process of it left after the
/// stop is ended by its id.
struct Stop(PathBuf);

impl Drop for Stop {
    fn drop(&mut self) {
        let left = pids(&self.0);
        let (code, _, _) = cmd(&self.0, &["session", "stop"]);
        if code != 0 {
            for pid in left {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
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

/// What p1 shows and where it stands: [the list is on its screen, its position ASL].
fn p1_screen(dir: &Path) -> Value {
    let code = "private _c = uiNamespace getVariable ['skd_handOver', controlNull]; [!isNull _c, getPosASL player]";
    let (exit, r, err) = cmd(dir, &["exec", "--machine", "p1", "--code", code]);
    assert_eq!(exit, 0, "{r} {err}");
    r["results"]["p1"]["value"].clone()
}

fn refuse(machine: &str) -> String {
    format!(
        "// @check US-084 A hand over to {machine} is refused\n[\"{machine}\", [\"anything\"]] call skd_fnc_handOver;\n"
    )
}

const AFTER: &str = "// @check US-083 A check after the hand over\n[1, \"==\", 1, \"one\"] call skd_fnc_expect;\n";

/// `US-084` 1 and `US-083` 1 to 4: on a dedicated session with two clients and a headless client,
/// a hand over to `server` or `hc1` ends the check `fail` / `check-error` naming `p1` and `p2`
/// and the run goes on; the check that hands over to `p1` ends the run with it handed over and
/// the check after it not run; the session `run --setup` started stays `ready`, the list is on
/// p1's screen and p1 stands where the check put it; the next run's reset takes the list away,
/// as does `reset`, and `session stop` ends the session.
#[test]
#[ignore = "starts a dedicated session of Arma 3"]
fn us_083_us_084_man_01_run_04_a_hand_over_keeps_the_session_for_the_manual_tester() {
    steam_runs();
    let dir = project(
        "handover",
        &[
            ("refuse-server", refuse("server")),
            ("refuse-hc", refuse("hc1")),
            ("after", AFTER.to_string()),
        ],
    );
    let stop = Stop(dir.clone());
    let t = Instant::now();
    let args = [
        "run",
        "--setup",
        "dedicated",
        "refuse-server",
        "refuse-hc",
        "handover/US-083-crate",
        "after",
    ];
    let (code, run, err) = cmd(&dir, &args);
    println!(
        "run with a hand over: exit {code} in {:.1} s",
        t.elapsed().as_secs_f64()
    );
    let file = run["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result: {run} {err}"));
    let result = validate(file);
    let checks = result["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 3, "{result:#}");
    for (c, m) in checks[..2].iter().zip(["server", "hc1"]) {
        assert_eq!(c["outcome"], "fail", "US-084 1: {c:#}");
        assert_eq!(c["reason"]["kind"], "check-error", "US-084 1");
        let message = c["reason"]["message"].as_str().unwrap();
        assert!(message.contains(m) && message.contains("p1, p2"), "US-084 1: {message}");
        assert!(c.get("handed_over").is_none(), "US-084 1");
    }
    let handed = &checks[2];
    assert_eq!(handed["outcome"], "pass", "US-083 1: {handed:#}");
    assert_eq!(handed["handed_over"]["machine"], "p1", "US-083 1");
    assert_eq!(handed["handed_over"]["items"].as_array().unwrap().len(), 3, "US-083 1");
    assert_eq!(result["not_run"][0]["check_id"], "after", "US-083 1");
    assert_eq!(result["not_run"][0]["reason"], "handed-over", "US-083 1");
    assert_eq!(code, 1, "the two refusals failed");
    let h = &run["handed_over"];
    assert_eq!(h["machine"], "p1", "US-083 1: {run:#}");
    assert_eq!(h["items"].as_array().unwrap().len(), 3, "US-083 1");
    assert_eq!(h["stop"], "shakedown session stop", "US-083 1");

    // US-083 3: the session the run started stays up, ready.
    let (_, status, _) = cmd(&dir, &["status"]);
    assert_eq!(status["session"]["state"], "ready", "US-083 3: {status:#}");
    assert_eq!(status["session"]["id"], h["session_id"], "US-083 3");

    // US-083 2: the list is on p1's screen and p1 stands where the check put it.
    let screen = p1_screen(&dir);
    assert_eq!(screen[0], true, "US-083 2: {screen}");
    let (x, y) = (screen[1][0].as_f64().unwrap(), screen[1][1].as_f64().unwrap());
    assert!(
        (x - 100.0).abs() < 0.5 && (y - 100.0).abs() < 0.5,
        "US-083 2: p1 at {screen}"
    );

    // US-083 4: the next run takes the session back: its reset removes the list.
    let (code, after, err) = cmd(&dir, &["run", "after"]);
    assert_eq!(code, 0, "US-083 4: {after} {err}");
    assert!(after["handed_over"].is_null(), "US-083 4");
    assert_eq!(p1_screen(&dir)[0], false, "US-083 4: the reset took the list away");

    // A hand over again, against the running session: the person's lines carry the machine, the
    // items and the stop command; then `reset` takes the list away too.
    let (code, out, err) = support::run(&exe(), &dir, &[], &["run", "handover/US-083-crate"]);
    assert_eq!(code, 0, "{out} {err}");
    for want in [
        "Handed over to p1",
        "1. An ammo crate stands 3 m in front of you.",
        "3. Opening it puts a first aid kit in your inventory.",
        "shakedown session stop",
    ] {
        assert!(out.contains(want), "US-083 1: {want:?} in\n{out}");
    }
    println!("{out}");
    assert_eq!(p1_screen(&dir)[0], true, "US-083 2");
    let (code, r, err) = cmd(&dir, &["reset"]);
    assert_eq!(code, 0, "US-083 4: {r} {err}");
    assert_eq!(p1_screen(&dir)[0], false, "US-083 4: reset took the list away");
    drop(stop);
    let (_, status, _) = cmd(&dir, &["status"]);
    assert!(
        status["session"].is_null(),
        "US-083 4: session stop ended it: {status:#}"
    );
}
