//! M7's tests: a reload or a reset asked during a run is refused (`US-058`, `RLD-01`), and a
//! changed function reloaded with no files given and the checks a finding names rerun by its tag
//! (`US-054`, `RUN-01`, `RLD-03`, `RLD-06`), timed against `NFR-PF-02`, in singleplayer and on a
//! dedicated session with two clients. They drive the installed `shakedown` and the game, so they
//! run only on the test machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m07 -- --ignored --nocapture --test-threads=1
//!
//! `@skd_fixture` is watched in a staged copy of its installed sources; its `rebuild` copies the
//! staged `fn_version.sqf` into the installed copy while no game process runs, as `FIX-03`
//! allows, and the installed file is put back once the session has stopped. Every process a test
//! starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::{Mcp, repo};

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

/// The reload target of `@skd_fixture`, under its `addons` folder (`FIX-03`).
const TARGET: &str = r"main\functions\fn_version.sqf";

/// The project's staged copy of the fixture's sources, which `reload` watches.
const STAGE: &str = r"stage\skdfx\addons";

/// The finding the rerun checks carry as a tag.
const FINDING: &str = "m07-1";

/// `NFR-PF-02`'s target for a `game-restart` reload, in seconds, before the rebuild's own time.
const PF_02_S: f64 = 90.0;

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

fn check(name: &str, header: &str, body: &str) -> String {
    format!("// @check {name}\n{header}{body}\n")
}

/// A project with the setups `sp` and `ded2` (two clients), both loading `@skd_fixture` alone,
/// watched in its staged copy with a `rebuild` that copies the reload target into the installed
/// copy (`RLD-06`, `FIX-03`). Its checks: `slow`, which takes about 12 s and sees `"1"` on every
/// machine, and three checks tagged with the finding that see `"2"` on every one of `machines`.
fn project(name: &str, machines: usize) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m07-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let fx = install().join("fixtures");
    let stage = dir.join(STAGE);
    copy_tree(&fx.join(r"skdfx\addons"), &stage);
    let toml = format!(
        r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"
source = '{stage}'
prefix = 'skdfx\addons'
rebuild = 'copy /y "{stage}\{TARGET}" "{fx}\skdfx\addons\{TARGET}"'

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.ded2]
mode = "dedicated"
clients = 2
mods = ["fixture"]
"#,
        fx = fx.display(),
        stage = stage.display(),
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    let checks = [
        (
            "slow",
            check(
                "US-058 a run that a reload asked during it leaves alone",
                "",
                r#"["the run goes on for about 12 s", {
    private _t = diag_tickTime;
    waitUntil { diag_tickTime > _t + 12 };
}] call skd_fnc_step;
["every machine still runs version 1", {
    ["all", { call skdfx_fnc_version }, "==", "1", "skdfx_fnc_version is 1"] call skd_fnc_expectEach;
}] call skd_fnc_step;"#,
            ),
        ),
        (
            "version-each",
            check(
                "US-054 every machine runs the reloaded function",
                &format!("// @tags {FINDING}\n"),
                &format!(
                    r#"["skd_fnc_onEach returns 2 on every machine", {{
    private _v = ["all", {{ call skdfx_fnc_version }}] call skd_fnc_onEach;
    [count _v, "==", {machines}, "machines that answered"] call skd_fnc_expect;
    {{ [_y, "==", "2", format ["skdfx_fnc_version on %1", _x]] call skd_fnc_expect }} forEach _v;
}}] call skd_fnc_step;"#
                ),
            ),
        ),
        (
            "version-lead",
            check(
                "US-054 the lead runs the reloaded function",
                &format!("// @tags {FINDING}\n"),
                r#"["the lead returns 2", {
    [call skdfx_fnc_version, "==", "2", "skdfx_fnc_version"] call skd_fnc_expect;
}] call skd_fnc_step;"#,
            ),
        ),
        (
            "version-expect",
            check(
                "US-054 every machine agrees on the reloaded function",
                &format!("// @tags {FINDING} smoke\n"),
                r#"["every machine returns 2", {
    ["all", { call skdfx_fnc_version }, "==", "2", "skdfx_fnc_version is 2"] call skd_fnc_expectEach;
}] call skd_fnc_step;"#,
            ),
        ),
    ];
    for (id, text) in checks {
        std::fs::write(dir.join("checks").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

/// Writes `value` as the whole of the staged reload target, a quoted string (`FIX-03`).
fn stage(dir: &Path, value: &str) {
    std::fs::write(dir.join(STAGE).join(TARGET), format!("\"{value}\"\n")).unwrap();
}

fn kill(pid: u64) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
}

/// The machines of the running session: (name, start, pid, state).
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

/// `run` with these arguments; its exit code and its result file.
fn run(dir: &Path, args: &[&str]) -> (i32, Value) {
    let mut all = vec!["run"];
    all.extend(args);
    let (code, run, err) = cmd(dir, &all);
    let file = run["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result: {run} {err}"));
    (code, validate(Path::new(file)))
}

/// Each execution's check id, outcome and reason kind.
fn outcomes(result: &Value) -> Vec<(String, String, Value)> {
    result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["check_id"].as_str().unwrap().to_string(),
                c["outcome"].as_str().unwrap().to_string(),
                c["reason"]["kind"].clone(),
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

/// The installed reload target, put back when dropped, after the session has stopped.
struct Changed {
    path: PathBuf,
    was: Vec<u8>,
}

impl Changed {
    fn new() -> Changed {
        let path = install().join(r"fixtures\skdfx\addons").join(TARGET);
        let was = std::fs::read(&path).unwrap();
        Changed { path, was }
    }
}

impl Drop for Changed {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.path, &self.was);
    }
}

/// `US-058` then `US-054` in one session of `setup`:
///
/// - `US-058` 1 and 2: with the staged target changed, a reload, a reset and the `reload` tool
///   asked during a run are refused naming the run, its progress and `run stop`; the run's checks
///   end as in a run with no reload asked, and no start number changes.
/// - `US-054` 1 to 3: `reload` with no files restarts the game with the rebuild, reporting the
///   level, the file and its reason, every machine and the time; the checks tagged with the
///   finding, rerun, see `"2"` on every machine with every start one higher and `reload` `full`,
///   within `NFR-PF-02`'s target plus 10 s from the reload.
fn reload_loop(name: &str, setup: &str, names: &[&str]) {
    let dir = project(name, names.len());
    // Dropped after the session, so the file is put back when no process holds it.
    let _target = Changed::new();
    let _s = Session::start(&dir, setup);

    // US-058 2: the outcomes of the run with no reload asked.
    let (code, alone) = run(&dir, &["slow"]);
    assert_eq!(code, 0, "{alone:#}");
    let alone = outcomes(&alone);

    // US-058 1: the change waits in the staged copy while a run is active.
    stage(&dir, "2");
    let before = machines(&dir);
    let during = {
        let dir = dir.clone();
        std::thread::spawn(move || run(&dir, &["slow"]))
    };
    let mut progress = Value::Null;
    for _ in 0..300 {
        let (_, s, _) = cmd(&dir, &["status"]);
        if !s["run"].is_null() {
            progress = s["run"].clone();
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let run_id = progress["id"]
        .as_str()
        .expect("the run never showed in status")
        .to_string();
    let (code, r, err) = cmd(&dir, &["reload"]);
    println!("US-058 reload during {run_id}: exit {code}: {err}");
    assert_eq!(code, 4, "{r} {err}");
    for part in [run_id.as_str(), "checks done", "`shakedown run stop`"] {
        assert!(err.contains(part), "{part}: {err}");
    }
    let (code, _, err) = cmd(&dir, &["reset"]);
    println!("US-058 reset during {run_id}: exit {code}: {err}");
    assert_eq!(code, 4, "{err}");
    assert!(err.contains(&run_id) && err.contains("`shakedown run stop`"), "{err}");
    let mut mcp = Mcp::start(&exe(), &dir, &[]);
    let answer = mcp.call("reload", json!({}));
    println!("US-058 reload tool during {run_id}: {}", answer.text());
    assert!(answer.is_error(), "{}", answer.result);
    assert!(
        answer.text().starts_with("session-busy:") && answer.text().contains(&run_id),
        "{}",
        answer.text()
    );
    drop(mcp);
    let (code, result) = during.join().unwrap();
    assert_eq!(code, 0, "{result:#}");
    assert_eq!(outcomes(&result), alone);
    let after = machines(&dir);
    assert_eq!(
        before.iter().map(|m| (&m.0, m.1, m.2)).collect::<Vec<_>>(),
        after.iter().map(|m| (&m.0, m.1, m.2)).collect::<Vec<_>>(),
        "a machine was restarted"
    );

    // US-054 1: the change, still staged, reloaded with no files.
    let t = Instant::now();
    let (code, r, err) = cmd(&dir, &["reload"]);
    let reload_wall = t.elapsed().as_secs_f64();
    println!("US-054 reload: exit {code} in {reload_wall:.1} s: {r}");
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!(
        (&r["state"], &r["level"]),
        (&json!("done"), &json!("game-restart")),
        "{r}"
    );
    let files = r["files"].as_array().unwrap();
    assert_eq!(files.len(), 1, "{r}");
    assert!(files[0]["path"].as_str().unwrap().ends_with("fn_version.sqf"), "{r}");
    let reason = files[0]["reason"].as_str().unwrap();
    assert!(reason.contains("new process") && reason.contains("rebuild"), "{reason}");
    assert_eq!(r["machines"], json!(names), "{r}");
    let duration = r["duration_s"].as_f64().expect("the reload reports its time");

    // US-054 2 and 3: the checks the finding names, rerun by its tag.
    let (code, result) = run(&dir, &["--tag", FINDING]);
    let total = t.elapsed().as_secs_f64();
    assert_eq!(code, 0, "{result:#}");
    let checks = result["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 3, "{result:#}");
    for c in checks {
        assert_eq!((&c["outcome"], &c["reload"]), (&json!("pass"), &json!("full")), "{c:#}");
    }
    for (name, start, _, state) in machines(&dir) {
        let old = after.iter().find(|m| m.0 == name).map_or(0, |m| m.1);
        assert_eq!((start, state.as_str()), (old + 1, "ready"), "{name}");
    }
    println!(
        "NFR-PF-02 on {setup}: reload duration_s {duration:.1} s, reload command {reload_wall:.1} s, reload to the rerun's last outcome {total:.1} s (target {PF_02_S} s plus 10 s)"
    );
    assert!(total < PF_02_S + 10.0, "{total:.1} s");
}

#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_054_us_058_nfr_pf_02_reload_during_a_run_and_rerun_by_finding_in_singleplayer() {
    reload_loop("sp", "sp", &["p1"]);
}

#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_054_us_058_nfr_pf_02_reload_during_a_run_and_rerun_by_finding_on_a_dedicated_session() {
    reload_loop("ded2", "ded2", &["server", "p1", "p2"]);
}
