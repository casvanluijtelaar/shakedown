//! M12's tests: helpers shared from the library folder (`US-070`), a library file that does not
//! compile (`US-071`), events recorded on every machine and waited for (`US-072`), and an awaited
//! event that never comes (`US-073`). They drive the installed `shakedown`, CBA, both fixtures
//! and the game, so they run only on the test machine, by hand, one at a time, with no runner
//! running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m12 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";
const CBA: &str = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997";

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

/// The checks both projects share, by id.
fn checks() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "lib-a",
            r#"// @check US-070 a check calls the library in its body and on every machine
["the body", call skd_lib_rope] call skd_fnc_evidence;
["machines", ["all", { call skd_lib_rope }] call skd_fnc_onEach] call skd_fnc_evidence;
[call skd_lib_rope, "in", ["v1", "v2"], "the library's value in the body"] call skd_fnc_expect;
"#,
        ),
        (
            "lib-b",
            r#"// @check US-070 a second check calls the library on p1
["p1", { call skd_lib_rope }, "==", call skd_lib_rope, "p1 reads what the body reads", 10] call skd_fnc_expectOn;
"#,
        ),
        (
            "ev-feature",
            r#"// @check US-072 the fixture's signal and a kill are recorded and awaited
// @setups sp-cba ded-cba
["all", "skdfx_signalled"] call skd_fnc_record;
["server", "EntityKilled", "mission"] call skd_fnc_record;
["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
private _last = ([] call skd_fnc_machines) select -1;
private _args = [_last, "skdfx_signalled", 10] call skd_fnc_waitEvent;
[_args select 0, "==", ["p1"] call skd_fnc_player, "the signal names p1's unit on " + _last] call skd_fnc_expect;
private _u = ["B_Soldier_F", AGLToASL [130, 130, 0], 0] call skd_fnc_create;
["server", { (_this select 0) setDamage 1 }, [_u]] call skd_fnc_on;
private _killed = ["server", "EntityKilled", 10] call skd_fnc_waitEvent;
[_killed select 0, "==", _u, "EntityKilled names the unit"] call skd_fnc_expect;
private _events = ["p1", "skdfx_signalled"] call skd_fnc_events;
[count _events, "==", 1, "one signal on p1"] call skd_fnc_expect;
[((_events select 0) get "at") get "frame", ">", 0, "the event's frame"] call skd_fnc_expect;
"#,
        ),
        (
            "ev-next",
            r#"// @check US-072 the next check finds no handler left
["p1", { count skd_recorders }, "==", 0, "no recording left on p1", 5] call skd_fnc_expectOn;
["server", { count skd_recorders }, "==", 0, "no recording left on the server", 5] call skd_fnc_expectOn;
["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;
["all", "skdfx_signalled"] call skd_fnc_record;
[count (["p1", "skdfx_signalled"] call skd_fnc_events), "==", 0, "the signal before the recording is not recorded"] call skd_fnc_expect;
"#,
        ),
        (
            "ev-local",
            r#"// @check US-073 p1 raises the signal locally and p2 waits for it
// @setups ded-cba
["all", "skdfx_signalled"] call skd_fnc_record;
["p1", { ["skdfx_signalled", [player]] call CBA_fnc_localEvent }] call skd_fnc_on;
["p2", "skdfx_signalled", 3] call skd_fnc_waitEvent;
"#,
        ),
        (
            "ev-never",
            r#"// @check US-073 a recorded signal that never comes
["p1", "skdfx_signalled"] call skd_fnc_record;
["p1", "skdfx_signalled", 3] call skd_fnc_waitEvent;
"#,
        ),
        (
            "ev-unrecorded",
            r#"// @check US-073 a wait for an event the check does not record
["p1", "skdfx_signalled", 3] call skd_fnc_waitEvent;
"#,
        ),
        (
            "ev-bad-mission",
            r#"// @check US-072 a mission event the engine does not have
["server", "NoSuchEventSkd", "mission"] call skd_fnc_record;
"#,
        ),
        (
            "lib-late",
            r#"// @check US-070 a late client has the library once it joins
// @setups ded-cba
// @timeout 240
["p3"] call skd_fnc_join;
["p3", { call skd_lib_rope }, "==", call skd_lib_rope, "p3 has the library", 10] call skd_fnc_expectOn;
"#,
        ),
    ]
}

/// A project with CBA and both fixtures, the setup `sp-cba` or `ded-cba` (two clients and the
/// late `p3`), the checks of M12 and the library file `rope.sqf` in `lib` (relative to the project,
/// or the default when `None`).
fn project(name: &str, lib: Option<&str>) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("skd-m12-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let fixture_cba = install().join("fixtures").join("@skd_fixture_cba");
    let lib_key = lib.map(|l| format!("lib = \"{l}\"\n")).unwrap_or_default();
    let toml = format!(
        r#"game = '{GAME}'
tests = "tests"
{lib_key}
[mods.cba]
path = '{CBA}'
patch = "cba_main"

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[mods.fixture-cba]
path = '{}'
patch = "skdfx_cba"

[setups.sp-cba]
mode = "singleplayer"
mods = ["cba", "fixture", "fixture-cba"]

[setups.ded-cba]
mode = "dedicated"
clients = 2
late = 1
mods = ["cba", "fixture", "fixture-cba"]
"#,
        fixture_cba.display()
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks() {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
    }
    let lib = dir.join(lib.unwrap_or("tests/_lib"));
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join("rope.sqf"), "// the rope kit\n\"v1\"\n").unwrap();
    (dir, lib)
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

/// Stops the session it started when dropped, also when a test fails, and then every game
/// process of it the test saw, by id.
struct Session {
    dir: PathBuf,
    pids: Arc<Mutex<Vec<u64>>>,
}

impl Session {
    fn see(&self) {
        let (_, s, _) = cmd(&self.dir, &["status"]);
        let mut pids = self.pids.lock().unwrap();
        for m in s["session"]["machines"].as_array().into_iter().flatten() {
            if let Some(pid) = m["pid"].as_u64()
                && !pids.contains(&pid)
            {
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

/// Runs checks by id and returns the result file's content, validated against the schema.
fn run(dir: &Path, ids: &[&str]) -> Value {
    let mut args = vec!["run"];
    args.extend_from_slice(ids);
    let (_, out, err) = cmd(dir, &args);
    let file = out["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result file: {out} {err}"));
    let result: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let schema: Value =
        serde_json::from_str(&std::fs::read_to_string(repo().join("spec/contract/result.schema.json")).unwrap())
            .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator.iter_errors(&result).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?}");
    result
}

fn outcome(c: &Value) -> (String, String, String) {
    (
        c["outcome"].as_str().unwrap_or_default().to_string(),
        c["reason"]["kind"].as_str().unwrap_or_default().to_string(),
        c["reason"]["message"].as_str().unwrap_or_default().to_string(),
    )
}

fn pass(c: &Value) {
    assert_eq!(outcome(c).0, "pass", "{c:#}");
}

fn attached(c: &Value, label: &str) -> Value {
    c["evidence"]["attached"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|a| a["label"] == label)
        .map(|a| a["value"].clone())
        .unwrap_or(Value::Null)
}

/// `reload` and its answer, which must be done.
fn reload(dir: &Path) -> Value {
    let (code, r, err) = cmd(dir, &["reload"]);
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!(r["state"], "done", "{r}");
    r
}

/// The library steps of `US-070` and `US-071` that both setups share: two checks use the
/// library, a changed helper reaches every machine after a reload at level none, a file that
/// does not compile fails every check naming it, and a fixed one lets them pass again.
fn library_round(dir: &Path, lib: &Path, machines: &[&str]) {
    let shown = lib
        .join("rope.sqf")
        .strip_prefix(dir)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let result = run(dir, &["lib-a", "lib-b"]);
    pass(&result["checks"][0]);
    pass(&result["checks"][1]);
    let a = &result["checks"][0];
    assert_eq!(attached(a, "the body"), "v1");
    let every: serde_json::Map<String, Value> = machines.iter().map(|m| (m.to_string(), json!("v1"))).collect();
    assert_eq!(attached(a, "machines"), Value::Object(every), "{a:#}");

    // US-070 3: an edited helper, after a reload that lists it at level none.
    std::fs::write(lib.join("rope.sqf"), "\"v2\"\n").unwrap();
    let pid_before = machine_pids(dir);
    let r = reload(dir);
    eprintln!("reload: {r}");
    assert_eq!(r["level"], "none", "{r}");
    let files = r["files"].as_array().unwrap();
    assert!(
        files
            .iter()
            .any(|f| f["path"].as_str().unwrap().ends_with("rope.sqf") && f["level"] == "none"),
        "{r}"
    );
    assert_eq!(
        machine_pids(dir),
        pid_before,
        "a reload of the library restarts nothing"
    );
    let result = run(dir, &["lib-a"]);
    let a = &result["checks"][0];
    pass(a);
    let every: serde_json::Map<String, Value> = machines.iter().map(|m| (m.to_string(), json!("v2"))).collect();
    assert_eq!(attached(a, "machines"), Value::Object(every), "{a:#}");

    // US-071 1: a library file that does not compile fails every check, naming it and its line.
    std::fs::write(lib.join("rope.sqf"), "\n\n1 +\n").unwrap();
    reload(dir);
    let result = run(dir, &["lib-a", "lib-b"]);
    for c in result["checks"].as_array().unwrap() {
        let (o, kind, message) = outcome(c);
        assert_eq!((o.as_str(), kind.as_str()), ("fail", "check-error"), "{c:#}");
        assert!(message.contains(&shown) && message.contains("line"), "{message}");
        eprintln!("US-071: {message}");
    }
    assert_eq!(result["summary"]["total"], 2);

    // US-071 2: fixed and reloaded, the checks reach their own outcomes.
    std::fs::write(lib.join("rope.sqf"), "\"v1\"\n").unwrap();
    reload(dir);
    let result = run(dir, &["lib-a", "lib-b"]);
    pass(&result["checks"][0]);
    pass(&result["checks"][1]);
}

fn machine_pids(dir: &Path) -> Vec<u64> {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["pid"].as_u64())
        .collect()
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_070_to_us_073_singleplayer() {
    let (dir, lib) = project("sp", None);

    // US-070 2: no library file is listed or run as a check.
    let (_, listed, _) = cmd(&dir, &["list"]);
    let ids: Vec<&str> = listed["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("{listed}"))
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(!ids.iter().any(|i| i.contains("_lib") || i.contains("rope")), "{ids:?}");

    let session = start(&dir, "sp-cba");
    library_round(&dir, &lib, &["p1"]);

    // US-072 1 to 4, US-073 1 and 2.
    let result = run(
        &dir,
        &["ev-feature", "ev-next", "ev-never", "ev-unrecorded", "ev-bad-mission"],
    );
    session.see();
    let c = &result["checks"];
    pass(&c[0]);
    let events = c[0]["events"].as_array().unwrap();
    eprintln!("US-072 events: {events:#?}");
    assert!(
        events
            .iter()
            .any(|e| e["name"] == "skdfx_signalled" && e["kind"] == "cba" && e["machine"] == "p1"),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| e["name"] == "EntityKilled" && e["kind"] == "mission"),
        "{events:?}"
    );
    pass(&c[1]);
    assert!(c[1]["events"].as_array().unwrap().is_empty(), "{:#}", c[1]);
    let (o, kind, message) = outcome(&c[2]);
    assert_eq!((o.as_str(), kind.as_str()), ("fail", "wait-ceiling"), "{:#}", c[2]);
    assert!(
        message.contains("skdfx_signalled") && message.contains("p1: none"),
        "{message}"
    );
    let d = c[2]["duration_s"].as_f64().unwrap();
    assert!(d >= 3.0, "the wait ended after {d} s");
    let (o, kind, message) = outcome(&c[3]);
    assert_eq!((o.as_str(), kind.as_str()), ("fail", "check-error"), "{:#}", c[3]);
    assert!(
        message.contains("does not record the event skdfx_signalled on p1"),
        "{message}"
    );
    let (o, kind, message) = outcome(&c[4]);
    eprintln!("bad mission event: {o} {kind} {message}");
    assert_eq!((o.as_str(), kind.as_str()), ("fail", "check-error"), "{:#}", c[4]);
    drop(session);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "drives the game on the test machine"]
fn us_070_to_us_073_dedicated() {
    // US-070 4: the library comes from the project's `lib`.
    let (dir, lib) = project("ded", Some("helpers"));
    let session = start(&dir, "ded-cba");
    library_round(&dir, &lib, &["server", "p1", "p2"]);

    let result = run(&dir, &["ev-feature", "ev-next", "ev-local", "lib-late"]);
    session.see();
    let c = &result["checks"];
    pass(&c[0]);
    let events = c[0]["events"].as_array().unwrap();
    for m in ["server", "p1", "p2"] {
        assert!(
            events
                .iter()
                .any(|e| e["name"] == "skdfx_signalled" && e["machine"] == m),
            "no signal on {m}: {events:?}"
        );
    }
    pass(&c[1]);
    let (o, kind, message) = outcome(&c[2]);
    eprintln!("US-073 1: {message}");
    assert_eq!((o.as_str(), kind.as_str()), ("fail", "wait-ceiling"), "{:#}", c[2]);
    assert!(
        message.contains("on p2") && message.contains("p2: none") && message.contains("p1: 1 skdfx_signalled"),
        "{message}"
    );
    assert_eq!(c[2]["reason"]["machine"], "p2");
    pass(&c[3]);
    drop(session);
    let _ = std::fs::remove_dir_all(&dir);
}
