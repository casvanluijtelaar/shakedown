//! M10's tests: the reset gives each player ACE's full heal when ACE medical is loaded
//! (`INT-ACE-02`). They load CBA_A3 and ACE3 from the Steam Workshop folder and start the game,
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
    std::fs::create_dir_all(dir.join("checks")).unwrap();
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
    std::fs::write(dir.join("checks").join("wound.sqf"), wound).unwrap();
    std::fs::write(dir.join("checks").join("healed.sqf"), healed).unwrap();
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
