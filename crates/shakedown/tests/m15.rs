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
        (x - 130.0).abs() < 0.5 && (y - 130.0).abs() < 0.5,
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
        "1. An ammo crate stands 2 m in front of you.",
        "3. You carry no first aid kit; opening the crate puts one in your inventory.",
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

/// Reads `code` on p1 through `exec` and returns its value.
fn on_p1(dir: &Path, code: &str) -> Value {
    let (exit, r, err) = cmd(dir, &["exec", "--machine", "p1", "--code", code]);
    assert_eq!(exit, 0, "{r} {err}");
    r["results"]["p1"]["value"].clone()
}

/// Where the box lies on p1's screen against the game's default HUD layout, in screen units:
/// [the box [x, y, w, h], the safe zone [x, y, w, h], the boxes of the action menu, the
/// communication menu, the vehicle info, the weapon info, the stance, the stamina bar and the
/// hints as the layout macros of the game's `a3\ui_f\hpp\defineCommonGrids.inc` give them].
const LAYOUT: &str = "private _c = uiNamespace getVariable ['skd_handOver', controlNull]; \
private _gw = ((safeZoneW / safeZoneH) min 1.2) / 40; \
private _gh = (((safeZoneW / safeZoneH) min 1.2) / 1.2) / 25; \
private _r = safeZoneX + safeZoneW; \
[ctrlPosition _c, [safeZoneX, safeZoneY, safeZoneW, safeZoneH], [\
 ['menu', safeZoneX + 1.5 * _gw, safeZoneY + 5 * _gh, 4.5 * _gw, 13.5 * _gh], \
 ['comm', safeZoneX, safeZoneY + 5 * _gh, 1.5 * _gw, 15 * _gh], \
 ['vehicle', safeZoneX + 0.5 * _gw, safeZoneY + 0.5 * _gh, 10 * _gw, 4.5 * _gh], \
 ['weapon', _r - 14.3 * _gw, safeZoneY + 0.5 * _gh, 10 * _gw, 4.5 * _gh], \
 ['stance', _r - 4.2 * _gw, safeZoneY + 0.5 * _gh, 3.7 * _gw, 3.7 * _gh], \
 ['stamina', _r - 14.3 * _gw, safeZoneY + 4.05 * _gh, 10 * _gw, 1 * _gh], \
 ['hint', _r - 13 * _gw, safeZoneY + 6 * _gh, 12 * _gw, 8 * _gh]]]";

/// The crate as p1 sees it: [p1 knows it, its action titles on p1, p1's distance to it, its open
/// flag, p1's first aid kits, the crate is local to p1, the other players within 1.5 m of p1].
const CRATE: &str = "private _crate = (nearestObjects [player, ['skdfx_Crate'], 20]) param [0, objNull]; \
[!isNull _crate, (actionIDs _crate) apply { (_crate actionParams _x) select 0 }, player distance _crate, \
 _crate getVariable ['skdfx_open', false], { _x == 'FirstAidKit' } count items player, local _crate, \
 count (allPlayers select { _x != player && { (_x distance player) < 1.5 } })]";

/// Fires the crate's action on p1 as the action menu does.
const FIRE: &str = "private _crate = (nearestObjects [player, ['skdfx_Crate'], 20]) param [0, objNull]; \
private _ids = actionIDs _crate; \
private _id = _ids select (_ids findIf { ((_crate actionParams _x) select 0) isEqualTo 'Open crate' }); \
player action ['User', _crate, _id]; _id";

/// `US-083` 2 with `m15-1` and `m15-2`: after the owner's command, the box on p1's screen lies in
/// the safe zone and in columns no element of the default HUD uses, so it covers none of them
/// whatever its height; on p1 the crate carries `Open crate` with p1 inside its 3 m radius, no
/// other player stands on p1, p1 carries no first aid kit, and firing the action as the action menu
/// does puts one in p1's inventory.
#[test]
#[ignore = "starts a dedicated session of Arma 3"]
fn us_083_m15_1_m15_2_man_01_the_box_leaves_the_action_menu_free_and_p1_opens_the_crate() {
    steam_runs();
    let dir = project("look", &[]);
    let stop = Stop(dir.clone());
    let (code, run, err) = cmd(&dir, &["run", "--setup", "dedicated", "handover/US-083-crate"]);
    assert_eq!(code, 0, "{run} {err}");
    assert_eq!(run["handed_over"]["machine"], "p1", "US-083 1: {run:#}");

    // m15-1: the box against the safe zone and the default HUD layout.
    let l = on_p1(&dir, LAYOUT);
    println!("box, safe zone and layout on p1: {l}");
    let f = |v: &Value, i: usize| v[i].as_f64().unwrap();
    let (bx, by, bw) = (f(&l[0], 0), f(&l[0], 1), f(&l[0], 2));
    let (sx, sy, sw) = (f(&l[1], 0), f(&l[1], 1), f(&l[1], 2));
    assert!(
        bw > 0.0 && bx >= sx && bx + bw <= sx + sw && by >= sy,
        "m15-1: the box {} lies in the safe zone {}",
        l[0],
        l[1]
    );
    for hud in l[2].as_array().unwrap() {
        let (hx, hw) = (f(hud, 1), f(hud, 3));
        assert!(
            bx >= hx + hw || bx + bw <= hx,
            "m15-1: the box {} shares columns with the {} {hud}, so it covers it at some height",
            l[0],
            hud[0]
        );
    }

    // m15-2: the crate and its action, as p1 sees them.
    let c = on_p1(&dir, CRATE);
    println!("the crate on p1: {c}");
    assert_eq!(c[0], true, "m15-2: p1 knows the crate: {c}");
    assert!(
        c[1].as_array().unwrap().iter().any(|t| t == "Open crate"),
        "m15-2: the action is on p1's side: {c}"
    );
    assert!(f(&c, 2) < 3.0, "m15-2: p1 is within the action's 3 m radius: {c}");
    assert_eq!(c[3], false, "m15-2: the crate is not open, so the condition holds: {c}");
    assert_eq!(c[4], 0, "m15-2: p1 carries no first aid kit before: {c}");
    assert_eq!(c[6], 0, "m15-2: no other player stands on p1: {c}");

    // Fire the action as the action menu does, and poll p1's inventory for the kit.
    on_p1(&dir, FIRE);
    let mut seen = Value::Null;
    for _ in 0..40 {
        seen = on_p1(&dir, CRATE);
        if seen[4] == 1 && seen[3] == true {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    assert!(
        seen[4] == 1 && seen[3] == true,
        "m15-2: opening the crate gave p1 one first aid kit: {seen}"
    );
    drop(stop);
}
