//! M14's tests: a changed `.sqf` of a watched mod reloaded at level `mission-restart`, the game
//! made to let go of the file by reading the release files (`US-081`, `US-054`, `US-055`,
//! `RLD-02`, `RLD-05`, `RLD-07`, `P-08`, `NFR-PF-01`, `NFR-PF-02`), and a file still held falling
//! back to a game restart that names its holder (`US-082`, `FIX-04`). They drive the installed
//! `shakedown` and the game, so they run only on a Windows machine with Arma 3, by hand, one at a time, with no
//! runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m14 -- --ignored --nocapture --test-threads=1
//!
//! Every file a test changes is in the fixtures' installed copy, as `FIX-03` allows, and is put
//! back after the session has stopped: the reload targets are changed by the mods' `rebuild`, which
//! copies a staged target into the installed copy only when it differs, so it writes only the file
//! the reload freed. Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::{Value, json};
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";
const CBA: &str = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997";

/// The CBA fixture's reload target, a CBA `PREP` function, under its `addons` folder.
const CBA_TARGET: &str = r"main\functions\fnc_version.sqf";
/// The fixture's reload target, a `CfgFunctions` function, under its `addons` folder.
const CFG_TARGET: &str = r"main\functions\fn_version.sqf";

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

/// A rebuild that copies one staged file into the installed copy only when it differs.
fn rebuild(from: &Path, to: &Path) -> String {
    format!(
        r#"fc /b "{0}" "{1}" >nul 2>nul || copy /y "{0}" "{1}""#,
        from.display(),
        to.display()
    )
}

/// A project whose two fixtures are watched in staged copies of their sources, with the setups
/// `sp-cba` and `ded2-cba` (CBA and both fixtures; two clients), and a check `version` that
/// expects the given values of both reload targets on every machine.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m14-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let fx = install().join("fixtures");
    let (stage, stage_cba) = (dir.join(r"stage\skdfx\addons"), dir.join(r"stage\skdfx_cba\addons"));
    copy_tree(&fx.join(r"skdfx\addons"), &stage);
    copy_tree(&fx.join(r"skdfx_cba\addons"), &stage_cba);
    let toml = format!(
        r#"game = '{GAME}'

[mods.cba]
path = '{CBA}'
patch = "cba_main"

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"
source = '{stage}'
prefix = 'skdfx\addons'
rebuild = '{rb}'

[mods.fixture-cba]
path = '{fx}\@skd_fixture_cba'
patch = "skdfx_cba"
source = '{stage_cba}'
prefix = 'skdfx_cba\addons'
rebuild = '{rb_cba}'

[setups.sp-cba]
mode = "singleplayer"
mods = ["cba", "fixture", "fixture-cba"]

[setups.ded2-cba]
mode = "dedicated"
clients = 2
mods = ["cba", "fixture", "fixture-cba"]
"#,
        fx = fx.display(),
        stage = stage.display(),
        stage_cba = stage_cba.display(),
        rb = rebuild(&stage.join(CFG_TARGET), &fx.join(r"skdfx\addons").join(CFG_TARGET)),
        rb_cba = rebuild(
            &stage_cba.join(CBA_TARGET),
            &fx.join(r"skdfx_cba\addons").join(CBA_TARGET)
        ),
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    expect(&dir, "1", "1");
    dir
}

/// Writes the check `version`, which expects `cba` from the CBA fixture's function and `cfg` from
/// the fixture's on every machine.
fn expect(dir: &Path, cba: &str, cfg: &str) {
    let text = format!(
        r#"// @check US-081 Every machine runs the reloaded functions
["the reloaded functions on every machine", {{
    ["all", {{ call skdfx_cba_fnc_version }}, "==", "{cba}", "the PREP function"] call skd_fnc_expectEach;
    ["all", {{ call skdfx_fnc_version }}, "==", "{cfg}", "the CfgFunctions function"] call skd_fnc_expectEach;
}}] call skd_fnc_step;
"#
    );
    std::fs::write(dir.join(r"tests\version.sqf"), text).unwrap();
}

/// Writes `value` as the whole of a staged reload target, a quoted string (`FIX-03`).
fn stage(dir: &Path, mod_folder: &str, target: &str, value: &str) {
    let path = dir.join("stage").join(mod_folder).join("addons").join(target);
    std::fs::write(path, format!("\"{value}\"\n")).unwrap();
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

fn kill(pid: u64) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
}

/// The installed reload targets, put back when dropped, after the session has stopped.
struct Targets(Vec<(PathBuf, Vec<u8>)>);

impl Targets {
    fn new() -> Targets {
        let fx = install().join("fixtures");
        Targets(
            [
                fx.join(r"skdfx\addons").join(CFG_TARGET),
                fx.join(r"skdfx_cba\addons").join(CBA_TARGET),
            ]
            .into_iter()
            .map(|p| {
                let b = std::fs::read(&p).unwrap();
                (p, b)
            })
            .collect(),
        )
    }
}

impl Drop for Targets {
    fn drop(&mut self) {
        for (p, b) in &self.0 {
            let _ = std::fs::write(p, b);
        }
    }
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

    /// Runs the check `version`; its exit code and result file.
    fn run(&self) -> (i32, Value) {
        let (code, run, err) = cmd(&self.dir, &["run", "version"]);
        let file = run["result_file"]
            .as_str()
            .unwrap_or_else(|| panic!("no result: {run} {err}"));
        let result: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        (code, result)
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

/// Reloads with no files and checks the result is done at `level` with one file per changed
/// target, every file at `level`; the result and its wall time.
fn reload(dir: &Path, level: &str, files: usize) -> (Value, f64) {
    let t = Instant::now();
    let (code, r, err) = cmd(dir, &["reload"]);
    let wall = t.elapsed().as_secs_f64();
    println!("reload: exit {code} in {wall:.1} s: {r}");
    assert_eq!(code, 0, "{r} {err}");
    assert_eq!((&r["state"], &r["level"]), (&json!("done"), &json!(level)), "{r}");
    assert_eq!(r["files"].as_array().unwrap().len(), files, "{r}");
    (r, wall)
}

/// `US-081` 3, `US-054` and `US-055` in singleplayer, and `US-082`:
/// - A changed CBA `PREP` function reloads at `mission-restart`, its reason saying the game is
///   made to let go of the file and the rebuild runs; `p1` keeps its process and start number, and
///   from the save to the check's outcome takes under 30 s, three times (`NFR-PF-01`).
/// - A changed `CfgFunctions` function reloads the same way; every machine recompiles it, and the
///   check after records `reload` `partial` (`RLD-05`).
/// - With the CBA target held by `p1`'s process through `skdfx_fnc_holdFile`, the next reload is a
///   `game-restart` whose file names the file, `p1` and that the game did not let go of it, and the
///   check after sees the new value on a new start of `p1`.
#[test]
#[ignore = "starts Arma 3; run by hand on a Windows machine with Arma 3"]
fn us_081_us_082_us_054_us_055_reload_by_mission_restart_in_singleplayer() {
    let dir = project("sp");
    let _targets = Targets::new();
    let s = Session::start(&dir, "sp-cba");
    let (code, first) = s.run();
    assert_eq!(code, 0, "{first:#}");
    assert_eq!(first["checks"][0]["reload"], "full");
    let p1 = machines(&dir)[0].clone();

    let mut loops = Vec::new();
    for n in 2..=4 {
        let value = n.to_string();
        let saved = Instant::now();
        stage(&dir, "skdfx_cba", CBA_TARGET, &value);
        expect(&dir, &value, "1");
        let (r, _) = reload(&dir, "mission-restart", 1);
        let reason = r["files"][0]["reason"].as_str().unwrap();
        assert!(
            reason.contains("let go of the file") && reason.contains("rebuild"),
            "{reason}"
        );
        assert_eq!(r["machines"], json!(["p1"]));
        let (code, result) = s.run();
        let took = saved.elapsed().as_secs_f64();
        assert_eq!(code, 0, "{result:#}");
        assert_eq!(result["checks"][0]["reload"], "full");
        let now = machines(&dir)[0].clone();
        assert_eq!((now.1, now.2), (p1.1, p1.2), "p1 keeps its start and process");
        println!(
            "US-081 3: from the save to the outcome {took:.1} s (reload duration_s {})",
            r["duration_s"]
        );
        loops.push(took);
        assert!(took < 30.0, "{took:.1} s");
    }

    // US-054 in singleplayer: the CfgFunctions function, recompiled after the mission restart.
    stage(&dir, "skdfx", CFG_TARGET, "2");
    expect(&dir, "4", "2");
    let (r, _) = reload(&dir, "mission-restart", 1);
    assert!(r["files"][0]["path"].as_str().unwrap().ends_with("fn_version.sqf"));
    let (code, result) = s.run();
    assert_eq!(code, 0, "{result:#}");
    assert_eq!(result["checks"][0]["reload"], "partial", "RLD-05");
    assert_eq!(machines(&dir)[0].2, p1.2);

    // US-082: the CBA target held by p1's process; two reads leave it held, so the game restarts.
    let (code, v, err) = cmd(
        &dir,
        &[
            "exec",
            "--machine",
            "p1",
            "--code",
            r#"["skdfx_cba\addons\main\functions\fnc_version.sqf"] call skdfx_fnc_holdFile"#,
        ],
    );
    assert_eq!((code, &v["results"]["p1"]["value"]), (0, &json!(true)), "{v} {err}");
    stage(&dir, "skdfx_cba", CBA_TARGET, "9");
    expect(&dir, "9", "2");
    let (r, _) = reload(&dir, "game-restart", 1);
    let reason = r["files"][0]["reason"].as_str().unwrap();
    println!("US-082 1: {reason}");
    assert_eq!(r["files"][0]["level"], "game-restart");
    assert!(
        reason.contains("fnc_version.sqf") && reason.contains("p1's process") && reason.contains("did not let go"),
        "{reason}"
    );
    let (code, result) = s.run();
    assert_eq!(code, 0, "US-082 2: {result:#}");
    assert_eq!(
        result["checks"][0]["reload"], "full",
        "a game restart compiles every function afresh"
    );
    assert_eq!(machines(&dir)[0].1, p1.1 + 1);
    println!("US-081 3: the three loops from the save to the outcome {loops:?}");
}

/// `US-081` 1, 2 and 4, `US-054` and `US-055` on a dedicated session with two clients, the demo's
/// shape: ten reloads of a changed CBA `PREP` function at `mission-restart`, the first with the
/// `CfgFunctions` function changed too; the server keeps its process, the clients are new starts,
/// and every machine runs the new code, 10 of 10. Prints the times `NFR-PF-02` is set from.
#[test]
#[ignore = "starts Arma 3; run by hand on a Windows machine with Arma 3"]
fn us_081_us_054_us_055_nfr_pf_02_ten_reloads_by_mission_restart_on_a_dedicated_session() {
    let dir = project("ded");
    let _targets = Targets::new();
    let s = Session::start(&dir, "ded2-cba");
    let server = machines(&dir).into_iter().find(|m| m.0 == "server").unwrap();
    let (mut durations, mut walls) = (Vec::new(), Vec::new());
    for n in 2..=11 {
        let before = machines(&dir);
        let value = n.to_string();
        stage(&dir, "skdfx_cba", CBA_TARGET, &value);
        let cfg = if n == 2 {
            stage(&dir, "skdfx", CFG_TARGET, "2");
            2
        } else {
            1
        };
        expect(&dir, &value, "2");
        let (r, wall) = reload(&dir, "mission-restart", cfg);
        assert_eq!(r["machines"], json!(["server", "p1", "p2"]));
        let (code, result) = s.run();
        assert_eq!(code, 0, "reload {n}: {result:#}");
        assert_eq!(result["checks"][0]["reload"], "partial", "RLD-05");
        for (name, start, pid, state) in machines(&dir) {
            let old = before.iter().find(|m| m.0 == name).unwrap();
            assert_eq!(state, "ready", "{name}");
            if name == "server" {
                assert_eq!((start, pid), (server.1, server.2), "the server keeps its process");
            } else {
                assert_eq!(start, old.1 + 1, "{name} is a new start");
            }
        }
        durations.push(r["duration_s"].as_f64().unwrap());
        walls.push(wall);
    }
    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    let max = durations.iter().cloned().fold(0.0, f64::max);
    let min = durations.iter().cloned().fold(f64::MAX, f64::min);
    println!(
        "NFR-PF-02 mission-restart on ded2-cba: 10 reloads, duration_s {durations:?}, min {min:.1} s, mean {mean:.1} s, max {max:.1} s; command wall times {walls:?}"
    );
}
