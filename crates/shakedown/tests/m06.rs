//! M6's tests: the mission restart of a check with `@reset mission` (`US-011`, `US-012`,
//! `RST-06`), orders across a mission restart (spike `SPK-01`, `NFR-RL-04`), the game's hold on
//! the files it read (spike `SPK-08`), and `reload` (`US-040`, `US-055`, `US-056`, `US-057`,
//! `RLD-01` to `RLD-06`, and the time of `NFR-PF-02`). They drive the installed `shakedown` and the game, so they run only on the test
//! machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m06 -- --ignored --nocapture --test-threads=1
//!
//! Every file a test changes is in the fixtures' installed copy, as `FIX-03` allows, and is put
//! back after: a reload target is changed by the `rebuild` of `@skd_fixture_cba`, which copies the
//! project's staged copy of its sources over the installed one while no game process runs. Every
//! process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{Mcp, repo};

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

/// A project in a temporary folder whose fixtures are watched by `reload`, with the setups `sp`
/// (the fixture), `sp-cba`, `ded-cba` (CBA and both fixtures, two clients and a headless client)
/// and `ded2-cba` (the same with two clients and no headless client, the demo's shape), and the
/// given checks. `@skd_fixture_cba` is watched in `stage\skdfx_cba\addons`, a copy of its
/// installed sources, and its `rebuild` copies the reload target from there into the installed
/// copy (`FIX-03`, `RLD-06`).
fn project(name: &str, checks: &[(&str, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m06-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let fx = install().join("fixtures");
    let stage = dir.join(STAGE);
    copy_tree(&fx.join(r"skdfx_cba\addons"), &stage);
    let toml = format!(
        r#"game = '{GAME}'

[mods.cba]
path = '{CBA}'
patch = "cba_main"

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"
source = '{fx}\skdfx\addons'
prefix = 'skdfx\addons'

[mods.fixture-cba]
path = '{fx}\@skd_fixture_cba'
patch = "skdfx_cba"
source = '{stage}'
prefix = 'skdfx_cba\addons'
rebuild = 'copy /y "{stage}\{target}" "{fx}\skdfx_cba\addons\{target}"'

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.sp-cba]
mode = "singleplayer"
mods = ["cba", "fixture", "fixture-cba"]

[setups.ded-cba]
mode = "dedicated"
clients = 2
headless = 1
mods = ["cba", "fixture", "fixture-cba"]

[setups.ded2-cba]
mode = "dedicated"
clients = 2
mods = ["cba", "fixture", "fixture-cba"]
"#,
        fx = fx.display(),
        stage = stage.display(),
        target = TARGET,
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

/// The CBA fixture's reload target, under its `addons` folder (`FIX-03`).
const TARGET: &str = r"main\functions\fnc_version.sqf";

/// The project's staged copy of the CBA fixture's sources, which `reload` watches.
const STAGE: &str = r"stage\skdfx_cba\addons";

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

/// Writes `value` as the whole of the staged reload target, a quoted string (`FIX-03`).
fn stage(dir: &Path, value: &str) {
    std::fs::write(dir.join(STAGE).join(TARGET), format!("\"{value}\"\n")).unwrap();
}

/// `skdfx_cba_fnc_version` on every machine of the session.
fn versions(dir: &Path) -> Value {
    let (code, v, err) = cmd(
        dir,
        &["exec", "--machine", "all", "--code", "call skdfx_cba_fnc_version"],
    );
    assert_eq!(code, 0, "{v} {err}");
    v["results"].clone()
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

fn kill(pid: u64) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
}

/// The machines of the running session by name: (start, pid, state).
fn machines(dir: &Path) -> Vec<(String, u64, u64, String)> {
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
                m["state"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

/// A started session, stopped when dropped, also when the test panics; any process of it left
/// after the stop is ended by its id.
struct Session {
    dir: PathBuf,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        let steam = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&steam.stdout).contains("steam.exe"),
            "Steam must run before a game start"
        );
        let t = Instant::now();
        let (code, s, err) = cmd(dir, &["session", "start", "--setup", setup]);
        println!(
            "session start --setup {setup}: exit {code} in {:.1} s",
            t.elapsed().as_secs_f64()
        );
        assert_eq!(code, 0, "{s} {err}");
        Session { dir: dir.to_path_buf() }
    }

    fn run(&self, ids: &[&str]) -> (i32, Value) {
        let mut args = vec!["run"];
        args.extend(ids);
        let (code, run, err) = cmd(&self.dir, &args);
        let file = run["result_file"]
            .as_str()
            .unwrap_or_else(|| panic!("no result: {run} {err}"));
        (code, validate(Path::new(file)))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let left = machines(&self.dir);
        let (code, _, _) = cmd(&self.dir, &["session", "stop"]);
        if code != 0 {
            for (_, _, pid, _) in left {
                kill(pid);
            }
        }
    }
}

/// A file of the fixtures' installed copy that `FIX-03` lets a test change, put back when dropped.
struct Changed {
    path: PathBuf,
    was: Option<Vec<u8>>,
}

impl Changed {
    fn new(rel: &str) -> Changed {
        let path = install().join("fixtures").join(rel);
        let was = std::fs::read(&path).ok();
        Changed { path, was }
    }
}

impl Drop for Changed {
    fn drop(&mut self) {
        match &self.was {
            Some(b) => {
                let _ = std::fs::write(&self.path, b);
            }
            None => {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

fn check(name: &str, header: &str, body: &str) -> String {
    format!("// @check {name}\n{header}{body}\n")
}

fn by_id(result: &Value, id: &str) -> Vec<Value> {
    result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["check_id"] == id)
        .cloned()
        .collect()
}

/// The 95th percentile of durations in seconds.
fn p95(mut d: Vec<f64>) -> f64 {
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    d[((d.len() as f64) * 0.95).ceil() as usize - 1]
}

/// Sends `1 + 1` to `machine` again and again while `running`, at least `min` times, each one
/// also counted in the machine's `uiNamespace`, which outlives a mission restart; the values and
/// the round trip times.
fn round_trips(dir: PathBuf, machine: &'static str, min: usize, running: Arc<AtomicBool>) -> Vec<(Value, f64)> {
    let mut out = Vec::new();
    for _ in 0..10_000 {
        if out.len() >= min && !running.load(Ordering::SeqCst) {
            break;
        }
        let t = Instant::now();
        let (code, v, err) = cmd(
            &dir,
            &[
                "exec",
                "--machine",
                machine,
                "--code",
                r#"uiNamespace setVariable ["skd_spk01", (uiNamespace getVariable ["skd_spk01", 0]) + 1]; 1 + 1"#,
                "--ceiling",
                "30",
            ],
        );
        let took = t.elapsed().as_secs_f64();
        let r = &v["results"][machine];
        assert_eq!(code, 0, "{v} {err}");
        out.push((json!({"state": r["state"], "value": r["value"]}), took));
    }
    out
}

const SET_VAR: &str = r#"["set a public variable on the server", {
    ["server", { skd_m06_left = 1; publicVariable "skd_m06_left"; true }] call skd_fnc_on;
    { [_x, { !isNil "skd_m06_left" }, format ["%1 sees it", _x], 10] call skd_fnc_waitUntil } forEach (call skd_fnc_machines);
}] call skd_fnc_step;"#;

const IS_NIL: &str = r#"["every machine starts with no such variable", {
    ["all", { isNil "skd_m06_left" }, "==", true, "skd_m06_left is nil"] call skd_fnc_expectEach;
}] call skd_fnc_step;"#;

/// `SPK-08`: with a session just started, the game holds a mod file it read open for reading
/// only, so the file cannot be changed, and it reads the content it holds; once its process has
/// ended the file is free.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn spk_08_the_game_holds_a_mod_file_it_read_and_a_stopped_session_frees_it() {
    let dir = project("spk08", &[]);
    let file = install().join(r"fixtures\skdfx_cba\addons\main\functions\fnc_version.sqf");
    {
        let _s = Session::start(&dir, "sp-cba");
        let write = std::fs::OpenOptions::new().write(true).open(&file);
        println!("open for writing while p1 runs: {write:?}");
        assert!(write.is_err(), "the game should hold the file");
        assert!(std::fs::rename(&file, file.with_extension("old")).is_err());
        let (_, v, _) = cmd(
            &dir,
            &[
                "exec",
                "--machine",
                "p1",
                "--code",
                r#"[call skdfx_cba_fnc_version, ["compile"] call CBA_fnc_isRecompileEnabled]"#,
            ],
        );
        assert_eq!(v["results"]["p1"]["value"], json!(["1", true]), "{v}");
    }
    let write = std::fs::OpenOptions::new().write(true).open(&file);
    assert!(write.is_ok(), "the file is free once the session stopped: {write:?}");
}

/// `SPK-01` across a mission restart, `NFR-RL-04` and `US-011` 1 in singleplayer: 100 or more
/// round trips to `p1` while three checks with `@reset mission` restart its mission; each is
/// answered `2` once, and the count `p1` kept equals the round trips sent.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn spk_01_nfr_rl_04_us_011_no_order_lost_or_run_twice_across_mission_restarts_in_singleplayer() {
    let checks = [
        ("set", check("US-011 sets a public variable", "", SET_VAR)),
        (
            "fresh",
            check("US-011 a mission restart clears it", "// @reset mission\n", IS_NIL),
        ),
    ];
    let dir = project("spk01-sp", &checks);
    let s = Session::start(&dir, "sp");
    let running = Arc::new(AtomicBool::new(true));
    let trips = {
        let (dir, running) = (dir.clone(), Arc::clone(&running));
        std::thread::spawn(move || round_trips(dir, "p1", 100, running))
    };
    let (code, result) = s.run(&["set", "fresh", "set", "fresh", "set", "fresh"]);
    running.store(false, Ordering::SeqCst);
    let trips = trips.join().unwrap();
    assert_eq!(code, 0, "{result:#}");
    for c in by_id(&result, "fresh") {
        assert_eq!(c["outcome"], "pass", "{c:#}");
    }
    let lost: Vec<&(Value, f64)> = trips.iter().filter(|(v, _)| v["value"] != 2).collect();
    assert!(lost.is_empty(), "lost or wrong: {lost:?}");
    let (_, v, _) = cmd(
        &dir,
        &[
            "exec",
            "--machine",
            "p1",
            "--code",
            r#"uiNamespace getVariable ["skd_spk01", 0]"#,
        ],
    );
    assert_eq!(v["results"]["p1"]["value"], json!(trips.len()), "each order ran once");
    let times: Vec<f64> = trips.iter().map(|(_, t)| *t).collect();
    println!(
        "SPK-01 p1 across 3 mission restarts: {} round trips, none lost, none twice, p95 {:.3} s, max {:.3} s",
        trips.len(),
        p95(times.clone()),
        times.iter().cloned().fold(0.0, f64::max)
    );
}

/// `SPK-01` across a mission restart and `NFR-RL-04` on a dedicated session with CBA, `US-011` 1
/// and 2, and the time of ten mission restarts (`SPK-08`'s measurement of level B): round trips to
/// the server and to `p1` while ten checks with `@reset mission` restart the mission.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn spk_01_nfr_rl_04_us_011_mission_restarts_on_a_dedicated_session() {
    let checks = [
        ("set", check("US-011 sets a public variable", "", SET_VAR)),
        (
            "fresh",
            check("US-011 a mission restart clears it", "// @reset mission\n", IS_NIL),
        ),
        (
            "quick",
            check(
                "US-011 a check after a mission restart",
                "// @reset mission\n",
                r#"["nothing", { true }] call skd_fnc_step;"#,
            ),
        ),
    ];
    let dir = project("spk01-ded", &checks);
    let s = Session::start(&dir, "ded-cba");
    let running = Arc::new(AtomicBool::new(true));
    let server = {
        let (dir, running) = (dir.clone(), Arc::clone(&running));
        std::thread::spawn(move || round_trips(dir, "server", 100, running))
    };
    let p1 = {
        let (dir, running) = (dir.clone(), Arc::clone(&running));
        std::thread::spawn(move || round_trips(dir, "p1", 100, running))
    };
    let mut ids = vec!["set", "fresh"];
    ids.extend(std::iter::repeat_n("quick", 9));
    let (code, result) = s.run(&ids);
    running.store(false, Ordering::SeqCst);
    let (server, p1) = (server.join().unwrap(), p1.join().unwrap());
    assert_eq!(code, 0, "{result:#}");
    assert_eq!(by_id(&result, "fresh")[0]["outcome"], "pass");
    // US-011 2: every start of p1, p2 and hc1 took part, the last one ten higher than the first.
    let starts = |name: &str| -> Vec<u64> {
        result["session"]["machines"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["name"] == name)
            .map(|m| m["start"].as_u64().unwrap())
            .collect()
    };
    for name in ["p1", "p2", "hc1"] {
        assert_eq!(starts(name), (1..=11).collect::<Vec<_>>(), "{name}");
    }
    assert_eq!(starts("server"), vec![1]);
    assert!(machines(&dir).iter().all(|(_, _, _, st)| st == "ready"));
    for (name, trips) in [("server", &server), ("p1", &p1)] {
        let bad: Vec<_> = trips.iter().filter(|(v, _)| v["value"] != 2).collect();
        assert!(bad.is_empty(), "{name}: {bad:?}");
        let times: Vec<f64> = trips.iter().map(|(_, t)| *t).collect();
        println!(
            "SPK-01 {name} across 10 mission restarts: {} round trips, none lost, p95 {:.3} s, max {:.3} s",
            trips.len(),
            p95(times.clone()),
            times.iter().cloned().fold(0.0, f64::max)
        );
    }
    let (_, v, _) = cmd(
        &dir,
        &[
            "exec",
            "--machine",
            "server",
            "--code",
            r#"uiNamespace getVariable ["skd_spk01", 0]"#,
        ],
    );
    assert_eq!(
        v["results"]["server"]["value"],
        json!(server.len()),
        "each order ran once on the server"
    );
    let restarts: Vec<f64> = by_id(&result, "quick")
        .iter()
        .chain(by_id(&result, "fresh").iter())
        .map(|c| c["duration_s"].as_f64().unwrap())
        .collect();
    let mean = restarts.iter().sum::<f64>() / restarts.len() as f64;
    println!(
        "level B on ded-cba (2 clients, 1 headless): 10 mission restarts, check durations {restarts:?}, mean {mean:.1} s, max {:.1} s",
        restarts.iter().cloned().fold(0.0, f64::max)
    );
}

/// `US-012`, and `US-057` 2 and `US-056` 2 on a dedicated session: `p2`'s new process ended while
/// the mission restart before a `@reset mission` check waits for it makes that check `invalid` /
/// `reset-failed` naming `p2` and the next `invalid` / `session-broken`, exit 2; `status` shows
/// the session broken; then `reload` of `config.cpp` restarts the game and brings every machine
/// back `ready` with a new start number.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_012_us_057_a_mission_restart_that_loses_p2_breaks_the_session_and_a_game_restart_mends_it() {
    let checks = [
        (
            "fresh",
            check("US-012 a mission restart that loses p2", "// @reset mission\n", IS_NIL),
        ),
        (
            "after",
            check(
                "US-012 a check after it",
                "",
                r#"["nothing", { true }] call skd_fnc_step;"#,
            ),
        ),
    ];
    let dir = project("us012", &checks);
    let s = Session::start(&dir, "ded-cba");
    let killer = {
        let dir = dir.clone();
        std::thread::spawn(move || {
            // p2's second start, from the moment it is launched (INS-04).
            for _ in 0..1200 {
                if let Some((_, _, pid, _)) = machines(&dir)
                    .into_iter()
                    .find(|(n, st, p, _)| n == "p2" && *st == 2 && *p > 0)
                {
                    kill(pid);
                    return pid;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            0
        })
    };
    let (code, result) = s.run(&["fresh", "after"]);
    let killed = killer.join().unwrap();
    assert!(killed > 0, "p2's new process was never seen");
    assert_eq!(code, 2, "{result:#}");
    let fresh = &by_id(&result, "fresh")[0];
    assert_eq!(
        (&fresh["outcome"], &fresh["reason"]["kind"]),
        (&json!("invalid"), &json!("reset-failed")),
        "{fresh:#}"
    );
    assert_eq!(fresh["reason"]["machine"], "p2");
    assert!(fresh["reason"]["message"].as_str().unwrap().contains("p2"));
    let after = &by_id(&result, "after")[0];
    assert_eq!(
        (&after["outcome"], &after["reason"]["kind"]),
        (&json!("invalid"), &json!("session-broken"))
    );
    let (_, st, _) = cmd(&dir, &["status"]);
    assert_eq!(st["session"]["state"], "broken");
    assert!(
        st["session"]["reason"]["message"].as_str().unwrap().contains("p2"),
        "{st}"
    );

    // US-057 2, US-056 2: a reload of config restarts the game, every machine ready, start + 1.
    let before = machines(&dir);
    let config = install().join(r"fixtures\skdfx\addons\main\config.cpp");
    let t = Instant::now();
    let (code, r, err) = cmd(&dir, &["reload", config.to_str().unwrap()]);
    println!(
        "reload of config.cpp on the broken session: exit {code} in {:.1} s: {r}",
        t.elapsed().as_secs_f64()
    );
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!((&r["state"], &r["level"]), (&json!("done"), &json!("game-restart")));
    assert!(
        r["files"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("Config is read at game start")
    );
    let after = machines(&dir);
    assert_eq!(after.len(), 4);
    for (name, start, _, state) in &after {
        let old = before.iter().find(|m| &m.0 == name).map_or(0, |m| m.1);
        assert_eq!(state, "ready", "{name}");
        assert_eq!(*start, old + 1, "{name}");
    }
    let (_, st, _) = cmd(&dir, &["status"]);
    assert_eq!(st["session"]["state"], "ready");
    assert_eq!(st["last_reload"]["id"], r["id"]);
}

/// `US-040`, `US-056` 1 and 2, and a change `RLD-02` gives level `mission-restart` in
/// singleplayer, through the command line and the `reload` tool.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_040_us_056_reload_a_check_a_new_file_and_config_in_singleplayer() {
    let one = check(
        "US-040 an assertion edited between runs",
        "",
        r#"["the answer", { [1, "==", 1, "one"] call skd_fnc_expect }] call skd_fnc_step;"#,
    );
    let dir = project("us040", &[("edit", one.clone())]);
    // Put back after the session has stopped, when no process holds it.
    let _target = Changed::new(&format!(r"skdfx_cba\addons\{TARGET}"));
    let s = Session::start(&dir, "sp-cba");
    let (code, first) = s.run(&["edit"]);
    assert_eq!(code, 0, "{first:#}");
    let start = machines(&dir)[0].1;

    // US-040 2: a reload naming only the check file is level none.
    let file = dir.join("tests").join("edit.sqf");
    let (code, r, _) = cmd(&dir, &["reload", file.to_str().unwrap()]);
    assert_eq!(code, 0, "{r}");
    assert_eq!((&r["state"], &r["level"]), (&json!("done"), &json!("none")));
    assert_eq!(r["files"][0]["level"], "none");
    assert!(
        r["files"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("read fresh for every execution")
    );

    // US-040 1 and 3: the edited check runs at once, on the same start of p1.
    let saved = Instant::now();
    std::fs::write(&file, one.replace(r#"[1, "==", 1, "one"]"#, r#"[1, "==", 2, "one"]"#)).unwrap();
    let (code, second) = s.run(&["edit"]);
    let took = saved.elapsed().as_secs_f64();
    assert_eq!(code, 1);
    let message = by_id(&second, "edit")[0]["reason"]["message"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(message.contains("== 2"), "{message}");
    assert_eq!(second["session"]["machines"][0]["start"], json!(start));
    println!("US-040 3: from the saved edit to the outcome {took:.1} s");
    assert!(took < 30.0);

    // The reload tool over MCP: the check file, by its path relative to the project.
    let mut mcp = Mcp::start(&exe(), &dir, &[]);
    let a = mcp.call("reload", json!({"files": ["tests/edit.sqf"]}));
    let d = a.data();
    assert_eq!((&d["state"], &d["level"]), (&json!("done"), &json!("none")), "{d}");

    // US-055 1 and 2 in singleplayer, through the reload tool: the staged CBA PREP function
    // changed, a reload with no files is a game restart that runs the mod's rebuild, and p1, a
    // new start, compiles the new file (RLD-02, RLD-06).
    stage(&dir, "2");
    let a = mcp.call("reload", json!({}));
    let d = a.data();
    println!("US-055 reload in singleplayer: {d}");
    assert_eq!(
        (&d["state"], &d["level"]),
        (&json!("done"), &json!("game-restart")),
        "{d}"
    );
    assert_eq!(d["files"].as_array().unwrap().len(), 1, "{d}");
    assert!(d["files"][0]["path"].as_str().unwrap().ends_with("fnc_version.sqf"));
    let reason = d["files"][0]["reason"].as_str().unwrap();
    assert!(
        reason.contains("only in a new process") && reason.contains("rebuild"),
        "{reason}"
    );
    assert!(d["duration_s"].as_f64().unwrap() > 0.0);
    drop(mcp);
    assert_eq!(versions(&dir)["p1"]["value"], "2");
    let start = start + 1;
    assert_eq!(machines(&dir)[0].1, start);
    let (code, after) = s.run(&["edit"]);
    assert_eq!(code, 1, "the edited check still fails as edited: {after:#}");
    assert_eq!(after["session"]["machines"][0]["start"], json!(start));

    // US-056 1: a new .sqf under the watched source needs a rebuild and a game restart.
    let new = Changed::new(r"skdfx\addons\main\functions\fn_new.sqf");
    std::fs::write(&new.path, "true\n").unwrap();
    let t = Instant::now();
    let (code, r, err) = cmd(&dir, &["reload", new.path.to_str().unwrap()]);
    println!(
        "reload of a new file: exit {code} in {:.1} s: {r}",
        t.elapsed().as_secs_f64()
    );
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!((&r["state"], &r["level"]), (&json!("done"), &json!("game-restart")));
    let reason = r["files"][0]["reason"].as_str().unwrap();
    assert!(
        reason.contains("rebuilt before this reload") && reason.contains("built PBO"),
        "{reason}"
    );
    assert_eq!(machines(&dir)[0].1, start + 1);
    drop(new);

    // RLD-01: with no file named, the reload finds the new file deleted since the snapshot the
    // game restart took, which needs a rebuild and a game restart too.
    let (code, r, _) = cmd(&dir, &["reload"]);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["level"], "game-restart", "{r}");
    assert_eq!(r["files"].as_array().unwrap().len(), 1, "{r}");
    assert!(r["files"][0]["path"].as_str().unwrap().ends_with("fn_new.sqf"));
    let after = machines(&dir);
    assert_eq!((after[0].1, after[0].3.as_str()), (start + 2, "ready"));
    let (code, third) = s.run(&["edit"]);
    assert_eq!(code, 1, "{third:#}");
}

/// `US-055` 1 and 2 and `NFR-PF-02` on a dedicated session with two clients, the demo's setup:
/// ten reloads of a changed CBA `PREP` function, each a game restart that runs the mod's rebuild
/// while no game process runs, each timed; every machine returns the new value, and the check
/// after the first passes from the start state with every start one higher.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_055_nfr_pf_02_ten_reloads_of_a_changed_function_on_a_dedicated_session() {
    let dir = project(
        "us055",
        &[("fresh", check("US-055 the next check starts fresh", "", IS_NIL))],
    );
    // Put back after the session has stopped, when no process holds it.
    let _target = Changed::new(&format!(r"skdfx_cba\addons\{TARGET}"));
    let s = Session::start(&dir, "ded2-cba");
    let names = ["server", "p1", "p2"];
    let mut durations = Vec::new();
    let mut walls = Vec::new();
    for n in 2..=11 {
        let before = machines(&dir);
        let value = n.to_string();
        stage(&dir, &value);
        let t = Instant::now();
        let (code, r, err) = cmd(&dir, &["reload"]);
        let wall = t.elapsed().as_secs_f64();
        println!("reload {n}: exit {code} in {wall:.1} s: {r}");
        assert_eq!(code, 0, "{r} {err}");
        assert_eq!(
            (&r["state"], &r["level"]),
            (&json!("done"), &json!("game-restart")),
            "{r}"
        );
        assert_eq!(r["files"].as_array().unwrap().len(), 1, "{r}");
        assert!(r["files"][0]["path"].as_str().unwrap().ends_with("fnc_version.sqf"));
        let reason = r["files"][0]["reason"].as_str().unwrap();
        assert!(
            reason.contains("only in a new process") && reason.contains("rebuild"),
            "{reason}"
        );
        assert_eq!(r["machines"], json!(names));
        let v = versions(&dir);
        for name in names {
            assert_eq!(v[name]["value"], json!(value), "{name}: {v}");
        }
        for (name, start, _, state) in machines(&dir) {
            let old = before.iter().find(|m| m.0 == name).map_or(0, |m| m.1);
            assert_eq!((start, state.as_str()), (old + 1, "ready"), "{name}");
        }
        durations.push(r["duration_s"].as_f64().unwrap());
        walls.push(wall);
        if n == 2 {
            // US-055 2: the next check passes from the start state, every start one higher.
            let (code, result) = s.run(&["fresh"]);
            assert_eq!(code, 0, "{result:#}");
            for m in result["session"]["machines"].as_array().unwrap() {
                assert_eq!(m["start"], 2, "{m}");
            }
        }
    }
    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    let max = durations.iter().cloned().fold(0.0, f64::max);
    let min = durations.iter().cloned().fold(f64::MAX, f64::min);
    println!(
        "NFR-PF-02 on ded2-cba (2 clients): 10 reloads, duration_s {durations:?}, min {min:.1} s, mean {mean:.1} s, max {max:.1} s; command wall times {walls:?}"
    );
}

/// `US-057` 1 and 2 on a dedicated session with two clients: `p2`'s new process ended during a
/// reload of a changed function fails it naming the step and `p2`, the session is broken, a run
/// asked after it is refused naming `shakedown reload`, and a reload brings every machine back
/// with a new start number on the new code.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_057_a_reload_that_loses_p2_breaks_the_session_and_a_reload_mends_it() {
    let dir = project(
        "us057",
        &[("fresh", check("US-057 a check after the failed reload", "", IS_NIL))],
    );
    // Put back after the session has stopped, when no process holds it.
    let _target = Changed::new(&format!(r"skdfx_cba\addons\{TARGET}"));
    let _s = Session::start(&dir, "ded2-cba");
    let names = ["server", "p1", "p2"];

    // US-057 1: p2's next process ended as soon as it is launched, during a reload.
    let before = machines(&dir);
    let p2_next = before.iter().find(|m| m.0 == "p2").unwrap().1 + 1;
    stage(&dir, "99");
    let killer = {
        let dir = dir.clone();
        std::thread::spawn(move || {
            for _ in 0..1200 {
                if let Some((_, _, pid, _)) = machines(&dir)
                    .into_iter()
                    .find(|(n, st, p, _)| n == "p2" && *st == p2_next && *p > 0)
                {
                    kill(pid);
                    return pid;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            0
        })
    };
    let (code, r, err) = cmd(&dir, &["reload"]);
    assert!(killer.join().unwrap() > 0, "p2's new process was never seen");
    println!("US-057 reload with p2 ended: exit {code}: {r} {err}");
    assert_eq!(code, 4, "{r} {err}");
    assert_eq!(r["state"], "failed", "{r}");
    assert_eq!(
        (&r["reason"]["step"], &r["reason"]["machine"]),
        (&json!("game-restart"), &json!("p2"))
    );
    let (_, st, _) = cmd(&dir, &["status"]);
    assert_eq!(st["session"]["state"], "broken");
    assert!(
        st["session"]["reason"]["message"].as_str().unwrap().contains("p2"),
        "{st}"
    );

    // US-057 2: a run asked after it is refused naming the reload; a reload brings every machine
    // back.
    let (code, _, err) = cmd(&dir, &["run", "fresh"]);
    assert_eq!(code, 4, "{err}");
    assert!(err.contains("broken") && err.contains("shakedown reload"), "{err}");
    let broken = machines(&dir);
    let (code, r, err) = cmd(&dir, &["reload"]);
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!(
        (&r["state"], &r["level"]),
        (&json!("done"), &json!("game-restart")),
        "{r}"
    );
    for (name, start, _, state) in machines(&dir) {
        let old = broken.iter().find(|m| m.0 == name).map_or(0, |m| m.1);
        assert_eq!((start, state.as_str()), (old + 1, "ready"), "{name}");
    }
    let v = versions(&dir);
    for name in names {
        assert_eq!(v[name]["value"], "99", "{name}: {v}");
    }
}
