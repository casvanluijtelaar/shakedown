//! M17's tests: a check makes a player use a mod's ACE interaction action by its path, on an
//! object and on itself, through ACE's key-up code (`US-089`, `ACT-04`, `INT-ACE-03`, `RES-01`),
//! and an ACE action that cannot be used says why (`US-090`, `DM-03`, `DM-04`). They run the e2e
//! project's ACE checks, `e2e/tests/ace/`, against the fixture's ACE actions with ACE3 from the
//! Steam Workshop. They drive the installed `shakedown` and the game, so they run only on the test
//! machine, by hand, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m17 -- --ignored --nocapture --test-threads=1
//!
//! Every session a test starts is one `run --setup` starts and stops; any process of it left after
//! the run is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

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

/// A project with the e2e project's file and setups and its ACE checks.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m17-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(r"tests\ace")).unwrap();
    let e2e = repo().join("e2e");
    let toml = std::fs::read_to_string(e2e.join("shakedown.toml"))
        .unwrap()
        .replace(r"'..\target", &format!("'{}", repo().join("target").display()));
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for f in std::fs::read_dir(e2e.join(r"tests\ace")).unwrap() {
        let f = f.unwrap().path();
        std::fs::copy(&f, dir.join(r"tests\ace").join(f.file_name().unwrap())).unwrap();
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

/// Ends any game process of the project's session still running after its run, by its id.
struct Leftovers(PathBuf);

impl Drop for Leftovers {
    fn drop(&mut self) {
        let (_, s, _) = cmd(&self.0, &["status"]);
        let pids: Vec<u64> = s["session"]["machines"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m["pid"].as_u64())
            .collect();
        if !pids.is_empty() {
            let _ = cmd(&self.0, &["session", "stop"]);
            for pid in pids {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

/// Runs the checks in a session of `setup` that the run starts and stops; the result file, valid
/// against the schema, and the checks by id.
fn run(dir: &Path, setup: &str, ids: &[&str]) -> Value {
    let mut args = vec!["run", "--setup", setup];
    args.extend_from_slice(ids);
    let (_, out, err) = cmd(dir, &args);
    let file = out["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result: {out} {err}"));
    validate(file)
}

fn check<'a>(result: &'a Value, id: &str) -> &'a Value {
    result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check_id"] == id)
        .unwrap_or_else(|| panic!("{id} did not run: {result:#}"))
}

fn assert_us_089_and_us_090(result: &Value) {
    for id in ["ace/US-089-crate", "ace/US-089-self"] {
        assert_eq!(
            check(result, id)["outcome"],
            "pass",
            "US-089 1, 2: {:#}",
            check(result, id)
        );
    }
    let missing = check(result, "ace/US-090-missing");
    assert_eq!(missing["outcome"], "fail", "US-090 1: {missing:#}");
    assert_eq!(missing["reason"]["kind"], "assertion", "US-090 1");
    assert_eq!(missing["reason"]["machine"], "p1", "US-090 1");
    let message = missing["reason"]["message"].as_str().unwrap();
    assert!(
        message.contains("\"skdfx_noSuchAction\"")
            && message.contains("\"skdfx_aceOpen\"")
            && message.contains("\"ACE_OpenBox\""),
        "US-090 1: the names at the parent: {message}"
    );
    let condition = check(result, "ace/US-090-condition");
    assert_eq!(condition["outcome"], "fail", "US-090 2: {condition:#}");
    assert_eq!(condition["reason"]["kind"], "assertion", "US-090 2");
    let message = condition["reason"]["message"].as_str().unwrap();
    assert!(
        message.contains("condition") && message.contains("skdfx_aceNever") && message.contains("false"),
        "US-090 2: {message}"
    );
    // US-089 3: the result file records the ACE version each machine loaded.
    for m in result["session"]["machines"].as_array().unwrap() {
        let ace = m["mods"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["mod"] == "ace")
            .unwrap_or_else(|| panic!("US-089 3: no ACE version for {m}"));
        assert!(
            ace["version"].as_str().unwrap().starts_with("3.21.2"),
            "US-089 3: {ace}"
        );
    }
}

/// `US-089` 1 to 3 and `US-090` 1 and 2 in singleplayer: `p1` opens the crate and uses its self
/// action through ACE, a missing name lists the names at its parent, a false condition says so,
/// and the result file has `p1`'s ACE version.
#[test]
#[ignore = "starts Arma 3 in singleplayer with CBA and ACE"]
fn us_089_us_090_act_04_ace_actions_in_singleplayer() {
    steam_runs();
    let dir = project("sp");
    let _left = Leftovers(dir.clone());
    let result = run(
        &dir,
        "sp-ace",
        &[
            "ace/US-089-crate",
            "ace/US-089-self",
            "ace/US-090-missing",
            "ace/US-090-condition",
        ],
    );
    assert_us_089_and_us_090(&result);
}

/// `US-089` 1 to 3 and `US-090` 1 and 2 on a dedicated session with two clients and a headless
/// client: the server and every client see the crate open and the self action's count.
#[test]
#[ignore = "starts a dedicated session of Arma 3 with CBA and ACE"]
fn us_089_us_090_act_04_ace_actions_on_a_dedicated_session() {
    steam_runs();
    let dir = project("dedicated");
    let _left = Leftovers(dir.clone());
    let result = run(
        &dir,
        "dedicated-ace",
        &[
            "ace/US-089-crate",
            "ace/US-089-self",
            "ace/US-090-missing",
            "ace/US-090-condition",
        ],
    );
    assert_us_089_and_us_090(&result);
}

/// `US-090` 3: on a machine without ACE the check ends `fail` / `check-error` saying ACE is not
/// loaded, at once rather than at the ceiling.
#[test]
#[ignore = "starts Arma 3 in singleplayer with CBA"]
fn us_090_dm_04_an_ace_action_without_ace_is_a_check_error() {
    steam_runs();
    let dir = project("no-ace");
    let _left = Leftovers(dir.clone());
    let result = run(&dir, "sp-cba", &["ace/US-090-no-ace"]);
    let c = check(&result, "ace/US-090-no-ace");
    assert_eq!(c["outcome"], "fail", "US-090 3: {c:#}");
    assert_eq!(c["reason"]["kind"], "check-error", "US-090 3");
    let message = c["reason"]["message"].as_str().unwrap();
    assert!(message.contains("ACE is not loaded on p1"), "US-090 3: {message}");
    assert!(
        c["duration_s"].as_f64().unwrap_or(0.0) < 10.0,
        "US-090 3: not at the ceiling: {c:#}"
    );
}
