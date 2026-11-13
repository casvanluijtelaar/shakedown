//! M10's tests: the reset gives each player ACE's full heal when ACE medical is loaded
//! (`INT-ACE-02`); the answers to the findings of the port of arma-grappling-hook (`m10-1`,
//! `m10-2`, `m10-5`); and the measurement of `SPK-14`. The ACE tests load CBA_A3 and ACE3 from
//! the Steam Workshop folder, the others the installed fixture. They start the game,
//! so they run only on the test machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m10 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";
const WORKSHOP: &str = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410";

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// The ACE medical state of a player: open wounds, pain, blood volume, damage and the summed
/// damage of its body parts.
const STATE: &str = r#"private _u = _this select 0;
[
    count (_u getVariable ["ace_medical_openWounds", createHashMap]),
    _u getVariable ["ace_medical_pain", 0],
    _u getVariable ["ace_medical_bloodVolume", 6],
    damage _u,
    (_u getVariable ["ace_medical_bodyPartDamage", [0, 0, 0, 0, 0, 0]]) call { private _s = 0; { _s = _s + _x } forEach _this; _s }
]"#;

/// A project that loads CBA and ACE, with a singleplayer setup and a dedicated one with two
/// clients, and two checks: one wounds every player through ACE, the next finds each healed.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m10-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.cba]
path = '{WORKSHOP}\450814997'
patch = "cba_main"

[mods.ace]
path = '{WORKSHOP}\463939057'
patch = "ace_medical"

[setups.sp-ace]
mode = "singleplayer"
mods = ["cba", "ace"]

[setups.ded-ace]
mode = "dedicated"
clients = 2
mods = ["cba", "ace"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    let state = STATE.replace('\n', " ");
    let wound = format!(
        r#"// @check INT-ACE-02 ACE wounds every player
{{
    private _who = _x;
    ["wound " + _who, {{
        [_this select 0, {{ [player, 0.6, "leftleg", "bullet"] call ace_medical_fnc_addDamageToUnit }}] call skd_fnc_on;
        [_this select 0, {{ private _s = [player] call {{ {state} }}; (_s select 0) > 0 && {{ (_s select 1) > 0 }} }}, "an open wound and pain", 10] call skd_fnc_waitUntil;
    }}, [_who]] call skd_fnc_step;
}} forEach ((call skd_fnc_machines) select {{ (_x select [0, 1]) == "p" }});
"#
    );
    let healed = format!(
        r#"// @check INT-ACE-02 the reset heals every player in full
{{
    private _who = _x;
    ["healed " + _who, {{
        private _s = [_this select 0, {{ [player] call {{ {state} }} }}] call skd_fnc_on;
        ["state", _s] call skd_fnc_evidence;
        [_s select 0, "==", 0, "open wounds"] call skd_fnc_expect;
        [_s select 1, "==", 0, "pain"] call skd_fnc_expect;
        [_s select 2, "==", 6, "blood volume"] call skd_fnc_expect;
        [_s select 3, "==", 0, "damage"] call skd_fnc_expect;
        [_s select 4, "==", 0, "body part damage"] call skd_fnc_expect;
    }}, [_who]] call skd_fnc_step;
}} forEach ((call skd_fnc_machines) select {{ (_x select [0, 1]) == "p" }});
"#
    );
    std::fs::write(dir.join("tests").join("wound.sqf"), wound).unwrap();
    std::fs::write(dir.join("tests").join("healed.sqf"), healed).unwrap();
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

fn runs(pid: u64) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
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

/// Stops the session it started when dropped, also when a test fails, and then every game
/// process of it still running, by id.
struct Session {
    dir: PathBuf,
    pids: Vec<u64>,
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = cmd(&self.dir, &["session", "stop"]);
        for pid in &self.pids {
            if runs(*pid) {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

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

/// Runs `wound` then `healed` and asserts both pass, so the reset between them healed every
/// player that the first check had wounded.
fn wound_then_heal(dir: &Path) {
    let (code, out, err) = cmd(dir, &["run", "wound", "healed"]);
    let file = PathBuf::from(out["result_file"].as_str().unwrap_or_else(|| panic!("{out} {err}")));
    let result: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    for c in result["checks"].as_array().unwrap() {
        println!("{}: {} {} {}", c["check_id"], c["outcome"], c["reason"], c["evidence"]);
    }
    assert_eq!(code, 0, "{}", file.display());
    assert_eq!(result["summary"]["pass"], 2, "{}", file.display());
}

/// `INT-ACE-02` in singleplayer with CBA and ACE.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn int_ace_02_the_reset_heals_the_player_in_full_in_singleplayer() {
    let dir = project("sp");
    let _session = start(&dir, "sp-ace");
    wound_then_heal(&dir);
}

/// `INT-ACE-02` on a dedicated server with two clients, each healed where its player is local.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn int_ace_02_the_reset_heals_every_player_in_full_on_a_dedicated_server() {
    let dir = project("ded");
    let _session = start(&dir, "ded-ace");
    wound_then_heal(&dir);
}

/// The checks of the findings of the M10 port: a cleanup named for `p1` (`m10-2`) and a wait on
/// an object created just before (`m10-5`), with the fixture alone.
const CLEANUP_LEFT: &str = r#"// @check RST-01 m10-2 a cleanup named for p1 runs on p1 though the check fails first
["p1", { player allowDamage false; missionNamespace setVariable ["skdm10_left", true]; true }] call skd_fnc_on;
[{ player allowDamage true; missionNamespace setVariable ["skdm10_left", nil] }, [], "p1"] call skd_fnc_cleanup;
missionNamespace setVariable ["skdm10_lead", true];
[{ missionNamespace setVariable ["skdm10_lead", nil] }] call skd_fnc_cleanup;
["fails on purpose before it puts p1 back"] call skd_fnc_fail;
"#;

const CLEANUP_BACK: &str = r#"// @check RST-01 m10-2 the next check finds p1 and the lead put back
private _p1 = ["p1", { [isDamageAllowed player, isNil "skdm10_left"] }] call skd_fnc_on;
[_p1, "==", [true, true], "p1 can be hurt and holds no variable of the last check"] call skd_fnc_expect;
[isNil "skdm10_lead", "==", true, "the lead holds no variable of the last check"] call skd_fnc_expect;
"#;

const JUST_CREATED: &str = r#"// @check DM-14 m10-5 a wait finds an object created just before on every machine
for "_i" from 1 to 20 do {
    private _c = ["skdfx_Crate", AGLToASL [100 + 2 * _i, 130, 0], 0] call skd_fnc_create;
    ["all", { !isNull (_this select 0) }, "==", true, "the crate is known", 5, [_c]] call skd_fnc_expectEach;
    private _last = (call skd_fnc_machines) select -1;
    private _seen = [_last, { !isNull (_this select 0) }, [_c]] call skd_fnc_on;
    [_seen, "==", true, "skd_fnc_on after the wait gets the crate"] call skd_fnc_expect;
};
"#;

/// A project under `target\` whose fixture path climbs out of the project folder with `..`, with
/// a singleplayer and a dedicated setup and the checks of `m10-2` and `m10-5`.
fn findings_project() -> PathBuf {
    let dir = repo()
        .join("target")
        .join("skd-m10")
        .join(format!("findings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("shakedown.toml"),
        format!(
            r#"game = '{GAME}'

[mods.fixture]
path = '..\..\shakedown\fixtures\@skd_fixture'
patch = "skdfx_main"

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.ded]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]
"#
        ),
    )
    .unwrap();
    for (name, text) in [
        ("a-left", CLEANUP_LEFT),
        ("b-back", CLEANUP_BACK),
        ("c-created", JUST_CREATED),
    ] {
        std::fs::write(dir.join("tests").join(format!("{name}.sqf")), text).unwrap();
    }
    dir
}

/// Runs the three checks: the first fails on purpose, the next two pass.
fn run_findings(dir: &Path) {
    let (code, out, err) = cmd(dir, &["run", "a-left", "b-back", "c-created"]);
    let file = PathBuf::from(out["result_file"].as_str().unwrap_or_else(|| panic!("{out} {err}")));
    let result: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let outcomes: Vec<(String, String)> = result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            println!(
                "{}: {} {} {}",
                c["check_id"], c["outcome"], c["reason"], c["evidence"]["observed"]
            );
            (
                c["check_id"].as_str().unwrap().to_string(),
                c["outcome"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let want: Vec<(String, String)> = [("a-left", "fail"), ("b-back", "pass"), ("c-created", "pass")]
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert_eq!(outcomes, want, "{}", file.display());
    assert_eq!(code, 1, "{}", file.display());
    assert_eq!(result["checks"][0]["reason"]["kind"], "explicit");
}

/// `CFG-01` (`m10-1`): a mod path that climbs out of the project folder loads in the game; with it,
/// `RST-01` (`m10-2`) and `DM-14` (`m10-5`) in singleplayer.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn cfg_01_rst_01_dm_14_m10_findings_in_singleplayer() {
    let dir = findings_project();
    let _session = start(&dir, "sp");
    run_findings(&dir);
}

/// `CFG-01` (`m10-1`), `RST-01` (`m10-2`: a cleanup on a client) and `DM-14` (`m10-5`: a wait on
/// every machine for an object the server just created) on a dedicated server with two clients
/// and a headless client.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn cfg_01_rst_01_dm_14_m10_findings_on_a_dedicated_server() {
    let dir = findings_project();
    let _session = start(&dir, "ded");
    run_findings(&dir);
}

/// `SPK-14`: the wait for an action's effect to reach every machine of a dedicated session with
/// two clients and a headless client, 100 times in two runs of 50 checks, and how near each
/// passing check came to its runner ceiling. Prints every measure; the pass condition is in
/// `spec/06-open-questions.md`.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn spk_14_an_actions_effect_reaches_every_machine_within_the_default_wait_ceiling() {
    let dir = repo()
        .join("target")
        .join("skd-m10")
        .join(format!("spk14-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("shakedown.toml"),
        format!(
            r#"game = '{GAME}'

[mods.fixture]
path = '..\..\shakedown\fixtures\@skd_fixture'
patch = "skdfx_main"

[setups.ded]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]
"#
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("tests").join("act.sqf"),
        r#"// @check SPK-14 an action's effect reaches every machine of a dedicated session
private _c = ["build", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create
}] call skd_fnc_step;
["p1 has the crate", { ["p1", { !isNull (_this select 0) }, "p1 has the crate", 10, [_this select 0]] call skd_fnc_waitUntil }, [_c]] call skd_fnc_step;
["open", { ["p1", _this select 0, "Open crate"] call skd_fnc_useAction }, [_c]] call skd_fnc_step;
["every machine sees it open", {
    ["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "the crate is open", 10, [_this select 0]] call skd_fnc_expectEach
}, [_c]] call skd_fnc_step;
"#,
    )
    .unwrap();
    let clock = std::time::Instant::now();
    let session = start(&dir, "ded");
    println!("SPK-14 session start {:.1} s", clock.elapsed().as_secs_f64());
    let mut waits = Vec::new();
    let mut checks = Vec::new();
    for batch in 0..2 {
        let mut args = vec!["run"];
        args.extend(std::iter::repeat_n("act", 50));
        let (code, out, err) = cmd(&dir, &args);
        let file = PathBuf::from(out["result_file"].as_str().unwrap_or_else(|| panic!("{out} {err}")));
        println!("SPK-14 batch {batch}: exit {code}, {}", file.display());
        let result: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        for c in result["checks"].as_array().unwrap() {
            checks.push(c["duration_s"].as_f64().unwrap());
            if c["outcome"] != "pass" {
                println!("SPK-14 not passed: {} {}", c["outcome"], c["reason"]);
            }
            for s in c["steps"].as_array().unwrap() {
                if s["path"][0] == "every machine sees it open" {
                    waits.push(s["duration_s"].as_f64().unwrap_or(f64::INFINITY));
                }
            }
        }
    }
    drop(session);
    let within = waits.iter().filter(|w| **w <= 10.0).count();
    let mut sorted = waits.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pick = |q: f64| sorted[((sorted.len() as f64 - 1.0) * q).round() as usize];
    println!(
        "SPK-14 effect waits: n {}, within 10 s {within}, median {:.3}, p95 {:.3}, max {:.3}",
        waits.len(),
        pick(0.5),
        pick(0.95),
        pick(1.0)
    );
    println!(
        "SPK-14 check durations: n {}, max {:.3}",
        checks.len(),
        checks.iter().cloned().fold(0.0, f64::max)
    );
    assert_eq!(waits.len(), 100);
    assert!(within >= 95, "SPK-14: {within} of 100 within 10 s");
}
