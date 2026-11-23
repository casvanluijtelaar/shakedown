//! The tests of the findings raised by checking the docs after M19: a clock flag of a machine's
//! earlier start ends with that start, so the checks after a `shakedown reload` that restarted the
//! game pass (`US-059`, `HLT-02`, finding m19-5), and on a dedicated session every client starts
//! at its own start position, whatever order the clients joined in, and the reset puts it back
//! there (`US-008`, `INT-AR-04`, `RST-02`, finding m19-6), which fails while `DEF-37` is open. They drive the installed `shakedown`
//! and the game, so they run only on a Windows machine with Arma 3, by hand, one at a time, with no runner
//! running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m19_findings -- --ignored --nocapture --test-threads=1
//!
//! Every game process a test's session started is ended by its id after the test.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;
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

/// A project with the setups `sp` (singleplayer) and `ded2` (a dedicated server with two
/// clients), both loading `@skd_fixture`, and the checks `pass` and `stuck`.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m19f-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

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
    std::fs::write(
        dir.join(r"tests\pass.sqf"),
        "// @check US-059 a check that passes\n[1, \"==\", 1, \"one\"] call skd_fnc_expect;\n",
    )
    .unwrap();
    std::fs::write(
        dir.join(r"tests\stuck.sqf"),
        "// @check US-010 a cleanup that never ends\n[{ waitUntil { false } }, []] call skd_fnc_cleanup;\n",
    )
    .unwrap();
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

fn machines(dir: &Path) -> Vec<Value> {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"].as_array().cloned().unwrap_or_default()
}

/// Every game process the session started, by id.
fn pids(dir: &Path) -> Vec<u64> {
    machines(dir).iter().filter_map(|m| m["pid"].as_u64()).collect()
}

/// Stops the session when dropped, also when a test fails, and then every game process it
/// started, by id: those of the start and those of any game restart.
struct Session {
    dir: PathBuf,
    pids: Vec<u64>,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        steam();
        let (code, s, err) = cmd(dir, &["session", "start", "--setup", setup]);
        let session = Session {
            dir: dir.to_path_buf(),
            pids: pids(dir),
        };
        assert_eq!(code, 0, "{s} {err}");
        session
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.pids.extend(pids(&self.dir));
        let _ = cmd(&self.dir, &["session", "stop"]);
        for pid in &self.pids {
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
        }
    }
}

/// `run <ids> --json`: the exit code and the result file.
fn run(dir: &Path, ids: &[&str]) -> (i32, Value) {
    let mut args = vec!["run"];
    args.extend_from_slice(ids);
    let (code, out, err) = cmd(dir, &args);
    let file = out["result_file"].as_str().unwrap_or_else(|| panic!("{out} {err}"));
    (
        code,
        serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap(),
    )
}

fn outcome(result: &Value, i: usize) -> (String, String, String) {
    let c = &result["checks"][i];
    (
        c["outcome"].as_str().unwrap_or_default().to_string(),
        c["reason"]["kind"].as_str().unwrap_or_default().to_string(),
        c["reason"]["message"].as_str().unwrap_or_default().to_string(),
    )
}

fn flag(dir: &Path, machine: &str) -> Value {
    machines(dir)
        .into_iter()
        .find(|m| m["name"] == machine)
        .map(|m| m["health"]["flag"].clone())
        .unwrap_or_default()
}

/// `US-059`, `HLT-02`, finding m19-5: a session broken while `p1` is flagged `mission-time`,
/// then `shakedown reload`, which restarts its game; the old start's flag does not reach the
/// checks of the new start, which pass.
#[test]
#[ignore = "starts the game; run by hand on a Windows machine with Arma 3"]
fn us_059_hlt_02_a_flag_of_an_earlier_start_ends_with_the_game_restart_of_a_reload() {
    let dir = project("flag");
    let _session = Session::start(&dir, "sp");

    // p1's clock held for 30 s, until the watch flags it.
    let (code, out, err) = cmd(
        &dir,
        &["exec", "--machine", "p1", "--code", "[30] call skdfx_fnc_holdClock"],
    );
    assert_eq!(code, 0, "{out} {err}");
    let mut flagged = false;
    for _ in 0..40 {
        if flag(&dir, "p1") == "mission-time" {
            flagged = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(flagged, "p1 was never flagged mission-time");

    // A cleanup that never ends breaks the session at the next reset.
    let (_, r) = run(&dir, &["stuck", "pass"]);
    println!("broken by: {:?} / {:?}", outcome(&r, 0), outcome(&r, 1));
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "broken", "{s}");

    // The reload restarts the game; the new p1 is ready, unflagged, and the checks pass.
    let (code, out, err) = cmd(&dir, &["reload"]);
    println!("reload: {out}");
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(out["level"], "game-restart", "{out}");
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "ready", "{s}");
    assert!(flag(&dir, "p1").is_null(), "{s}");
    let (code, r) = run(&dir, &["pass", "pass"]);
    for i in 0..2 {
        println!("after the reload: {:?}", outcome(&r, i));
        assert_eq!(outcome(&r, i).0, "pass", "US-059 2: {:#}", r["checks"][i]);
    }
    assert_eq!(code, 0);
}

/// Each client's position on the ground and its direction, as `exec` on `clients` gives them.
fn positions(dir: &Path) -> Vec<(String, f64, f64, f64)> {
    let (code, out, err) = cmd(
        dir,
        &[
            "exec",
            "--machine",
            "clients",
            "--code",
            "[getPosASL player, getDir player]",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    out["results"]
        .as_object()
        .unwrap_or_else(|| panic!("{out}"))
        .iter()
        .map(|(name, m)| {
            let v = &m["value"];
            (
                name.clone(),
                v[0][0].as_f64().unwrap_or(f64::NAN),
                v[0][1].as_f64().unwrap_or(f64::NAN),
                v[1].as_f64().unwrap_or(f64::NAN),
            )
        })
        .collect()
}

fn at_start(seen: &[(String, f64, f64, f64)], when: &str) {
    let want = [("p1", 100.0, 100.0), ("p2", 110.0, 100.0)];
    assert_eq!(seen.len(), 2, "{when}: {seen:?}");
    for (name, x, y) in want {
        let (_, sx, sy, dir) = seen
            .iter()
            .find(|s| s.0 == name)
            .unwrap_or_else(|| panic!("{when}: no {name} in {seen:?}"));
        let off = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
        let turn = dir.min(360.0 - dir);
        assert!(
            off < 0.1 && turn < 1.0,
            "US-008 INT-AR-04 {when}: {name} at [{sx}, {sy}] facing {dir}, not at [{x}, {y}] facing 0"
        );
    }
}

/// `US-008`, `INT-AR-04`, `RST-02`, finding m19-6: in each of six fresh dedicated sessions with
/// two clients, `p1` starts at the spawn point and `p2` 10 m east of it, facing north, whichever
/// client joined first, and the reset puts each back there.
#[test]
#[ignore = "starts dedicated sessions of Arma 3; run by hand on a Windows machine with Arma 3"]
fn us_008_int_ar_04_every_client_starts_at_its_own_start_position() {
    let dir = project("spawn");
    for n in 1..=6 {
        let session = Session::start(&dir, "ded2");
        let seen = positions(&dir);
        println!("start {n}: {seen:?}");
        at_start(&seen, &format!("start {n}, after the start"));
        let (code, reset, err) = cmd(&dir, &["reset"]);
        assert_eq!(code, 0, "{reset} {err}");
        let seen = positions(&dir);
        println!("start {n} after the reset: {seen:?}");
        at_start(&seen, &format!("start {n}, after the reset"));
        drop(session);
    }
}
