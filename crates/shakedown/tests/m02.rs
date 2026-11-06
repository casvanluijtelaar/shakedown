//! M2's end-to-end tests: a dedicated server with two game clients and a headless client on the
//! one machine, the stories `US-002`, `US-004`, `US-008`, `US-009`, `US-017`, `US-019`, `US-020`,
//! `US-025` and `US-026`, the reset across machines (`NFR-RL-02`, `NFR-PF-04`, the dedicated half
//! of `SPK-04`) and the demo. They drive the installed `shakedown` command against the game, so
//! they run only on the test machine, by hand, one at a time:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m02 -- --ignored --nocapture --test-threads=1
//!
//! The demo alone (exit criterion 2) is `m2_demo_us_009_ten_times_on_a_dedicated_session`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::{Value, json};

fn repo() -> PathBuf {
    let p = std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

fn install() -> PathBuf {
    repo().join("target").join("shakedown")
}

fn shakedown() -> PathBuf {
    let exe = install().join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";
const CBA: &str = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997";
const AGH: &str = r"C:\Users\Shadow\Documents\arma-grappling-hook\.hemttout\build";

/// Runs the installed command in `project`; returns the exit code and stdout parsed as JSON.
fn cmd(project: &Path, args: &[&str]) -> (i32, Value) {
    let out = Command::new(shakedown())
        .args(args)
        .arg("--json")
        .current_dir(project)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    eprintln!(
        "$ shakedown {} -> {:?}\n{}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    (
        out.status.code().unwrap_or(-1),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
    )
}

/// A project in a temporary folder with the setups `sp` (the fixture), `ded` (the fixture, two
/// clients and a headless client), `ded-slow` (`ded` with `start_timeout = 30`) and `ded-agh`
/// (CBA and arma-grappling-hook, two clients), and the given checks.
fn project(name: &str, checks: &[(String, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m02-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.cba]
path = '{CBA}'
patch = "cba_main"

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[mods.agh]
path = '{AGH}'
patch = "agh_main"

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.ded]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]

[setups.ded-slow]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]
start_timeout = 30

[setups.ded-agh]
mode = "dedicated"
clients = 2
mods = ["cba", "agh"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks {
        let path = dir.join("checks").join(format!("{id}.sqf"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    dir
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

/// The game processes running now, by image name and process id.
fn game_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    for image in ["arma3_x64.exe", "arma3server_x64.exe"] {
        let out = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {image}"), "/FO", "CSV", "/NH"])
            .output()
            .unwrap();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(pid) = line.split("\",\"").nth(1).and_then(|p| p.parse().ok()) {
                pids.push(pid);
            }
        }
    }
    pids
}

/// A started session, stopped when dropped, also when the test panics.
struct Session {
    dir: PathBuf,
    id: String,
    pids: Vec<u64>,
    view: Value,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        let t = Instant::now();
        let (code, s) = cmd(dir, &["session", "start", "--setup", setup]);
        println!(
            "session start --setup {setup}: exit {code} in {:.1} s",
            t.elapsed().as_secs_f64()
        );
        assert_eq!(code, 0, "{s}");
        Session {
            dir: dir.to_path_buf(),
            id: s["id"].as_str().unwrap().to_string(),
            pids: s["machines"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|m| m["pid"].as_u64())
                .collect(),
            view: s,
        }
    }

    /// Runs checks by id; returns the exit code and the validated result file.
    fn run(&self, ids: &[&str]) -> (i32, Value) {
        let mut args = vec!["run"];
        args.extend(ids);
        let (code, run) = cmd(&self.dir, &args);
        let file = run["result_file"]
            .as_str()
            .unwrap_or_else(|| panic!("no result: {run}"));
        let result = validate(Path::new(file));
        assert_eq!(result["session"]["id"], self.id.as_str());
        (code, result)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let (code, _) = cmd(&self.dir, &["session", "stop"]);
        if code != 0 {
            for pid in &self.pids {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

fn by_id(result: &Value, id: &str) -> Value {
    result["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check_id"] == id)
        .unwrap_or_else(|| panic!("no execution of {id}"))
        .clone()
}

fn assert_outcome(c: &Value, outcome: &str, kind: Option<&str>, needles: &[&str]) {
    assert_eq!(c["outcome"], outcome, "{c:#}");
    if let Some(kind) = kind {
        assert_eq!(c["reason"]["kind"], kind, "{c:#}");
    }
    let message = c["reason"]["message"].as_str().unwrap_or_default();
    for n in needles {
        assert!(message.contains(n), "{}: `{n}` not in `{message}`", c["check_id"]);
    }
}

fn check(name: &str, body: &str) -> String {
    format!("// @check {name}\n{body}\n")
}

fn checks_of(list: &[(&str, &str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(id, name, body)| (id.to_string(), check(name, body)))
        .collect()
}

/// The state every check of `US-008` and `NFR-RL-02` compares with the session's start: on each
/// machine its own player (position, direction, damage, alive, on foot, loadout, animation), the
/// date to the hour, the overcast, and the counts of vehicles, units, dead and groups.
const STATE: &str = r#"{
    private _p = if (hasInterface && { !isNull player }) then {
        [getPosASL player, getDir player, damage player, alive player, vehicle player isEqualTo player, getUnitLoadout player, animationState player]
    } else { [] };
    [_p, date select [0, 4], (round (overcast * 100)) / 100, count vehicles, count allUnits, count allDeadMen, count allGroups]
}"#;

/// `US-017` 1 to 3: one check file that passes in singleplayer and on a dedicated session.
const PORTABLE_SCENE: &str = r#"
private _pos = AGLToASL [120, 115, 0];
private _p1 = ["p1"] call skd_fnc_player;
["place p1", { ["p1", _pos, 135] call skd_fnc_place }] call skd_fnc_step;
["every machine sees p1 there", {
    private _seen = ["all", { [getPosASL (_this select 0), getDir (_this select 0)] }, [_p1]] call skd_fnc_onEach;
    { [(_seen get _x) select 0, "~=", [_pos, 0.1], format ["%1 sees p1's position", _x]] call skd_fnc_expect;
      [(_seen get _x) select 1, "~=", [135, 3], format ["%1 sees p1's direction", _x]] call skd_fnc_expect } forEach (call skd_fnc_machines);
}] call skd_fnc_step;
["a unit", {
    private _u = ["B_Soldier_F", AGLToASL [125, 125, 0], 90] call skd_fnc_create;
    [side group _u, "==", west, "a unit of side WEST"] call skd_fnc_expect;
}] call skd_fnc_step;
["date and weather", {
    [[[2035, 6, 1, 19, 30]] call skd_fnc_setDate, "==", true, "setDate"] call skd_fnc_expect;
    [[0.8, 0.2, 0] call skd_fnc_setWeather, "==", true, "setWeather"] call skd_fnc_expect;
    ["all", { date select [0, 5] }, "==", [2035, 6, 1, 19, 30], "every machine's date"] call skd_fnc_expectEach;
    ["all", { overcast }, "~=", [0.8, 0.01], "every machine's overcast"] call skd_fnc_expectEach;
}] call skd_fnc_step;
"#;

/// `US-019` 1 and 2: one check file that passes on a dedicated session and in singleplayer. The
/// machines it asserts on are the server and the clients, which in singleplayer are `p1`.
const PORTABLE_CRATE: &str = r#"
private _p1 = ["p1"] call skd_fnc_player;
private _on = (call skd_fnc_machines) select { (_x select [0, 2]) != "hc" };
private _crate = ["put the crate 2 m in front of p1", {
    ["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
    ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create
}] call skd_fnc_step;
private _kits = { { _x == "FirstAidKit" } count items (_this select 0) };
private _before = ["server", _kits, [_p1]] call skd_fnc_on;
["open it", { [["p1", _crate, "Open crate"] call skd_fnc_useAction, "==", true, "useAction"] call skd_fnc_expect }] call skd_fnc_step;
["every machine sees one more kit", {
    [_on, _kits, "==", _before + 1, "p1's first aid kits", 10, [_p1]] call skd_fnc_expectEach;
    ["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "the crate is open", 10, [_crate]] call skd_fnc_expectEach;
}] call skd_fnc_step;
"#;

/// A check that records the start state of every machine on the lead, for the checks after it.
fn probe() -> String {
    format!(
        r#"["record the start state", {{
    skd_test_start = ["all", {STATE}] call skd_fnc_onEach;
    skd_test_ids = [];
}}] call skd_fnc_step;"#
    )
}

/// A check that changes everything the reset restores, on every kind of machine, `n` its number.
fn dirty(n: usize) -> String {
    format!(
        r#"["change the world, check {n}", {{
    private _ids = [];
    _ids pushBack netId (["B_Soldier_F", AGLToASL [130, 100, 0], 0] call skd_fnc_create);
    _ids pushBack netId (["C_Quadbike_01_F", AGLToASL [130, 110, 0], 90] call skd_fnc_create);
    _ids pushBack netId (["Box_NATO_Ammo_F", AGLToASL [130, 120, 0], 0] call skd_fnc_create);
    private _mrap = ["B_MRAP_01_F", AGLToASL [110, 130, 0], 0] call skd_fnc_create;
    _ids pushBack netId _mrap;
    _ids pushBack (["p2", {{ netId ("Land_Cargo20_military_green_F" createVehicle [140, 140, 0]) }}] call skd_fnc_on);
    _ids pushBack (["hc1", {{ private _g = createGroup east; netId (_g createUnit ["O_Soldier_F", [150, 150, 0], [], 0, "NONE"]) }}] call skd_fnc_on);
    ["p1", {{ player setDamage 0.5; removeAllWeapons player; player moveInDriver (_this select 0); true }}, [_mrap]] call skd_fnc_on;
    ["p1", {{ vehicle player isNotEqualTo player }}, "p1 sits in the MRAP"] call skd_fnc_waitUntil;
    ["p2", {{ player setDamage 1; true }}] call skd_fnc_on;
    [[2030, 1, 1, {hour}, 0]] call skd_fnc_setDate;
    [0.9, 0.5, 0.5] call skd_fnc_setWeather;
    skd_test_ids = skd_test_ids + _ids;
}}] call skd_fnc_step;"#,
        hour = 3 + (n % 10)
    )
}

/// A check that asserts every machine shows the start state and that nothing the checks before it
/// created exists, by net id, on any machine.
fn baseline() -> String {
    format!(
        r#"["nothing created before exists on any machine", {{
    ["all", {{ (_this select 0) findIf {{ !isNull objectFromNetId _x }} }}, "==", -1, "no earlier object exists", 10, [skd_test_ids]] call skd_fnc_expectEach;
}}] call skd_fnc_step;
["every machine shows the start state", {{
    private _now = ["all", {STATE}] call skd_fnc_onEach;
    {{
        private _a = skd_test_start get _x;
        private _b = _now get _x;
        [_b select [1, 6], "==", _a select [1, 6], format ["%1's date, weather and counts", _x]] call skd_fnc_expect;
        private _pa = _a select 0;
        private _pb = _b select 0;
        if (_pa isNotEqualTo []) then {{
            [_pb select 0, "~=", [_pa select 0, 0.1], format ["%1's player position", _x]] call skd_fnc_expect;
            private _d = abs ((_pb select 1) - (_pa select 1)) % 360;
            [_d min (360 - _d), "<=", 3, format ["%1's player direction", _x]] call skd_fnc_expect;
            [_pb select [2, 5], "==", _pa select [2, 5], format ["%1's player damage, life, foot, loadout and animation", _x]] call skd_fnc_expect;
        }};
    }} forEach (keys skd_test_start);
}}] call skd_fnc_step;"#
    )
}

/// `US-002`, `US-017`, `US-019`, `US-020`, `US-025`, `US-026`, `US-008` and `US-009` in one
/// dedicated session with two clients and a headless client, the `reset` command after a check
/// that changed everything (`US-008` 3), and `status` (`INS-02`, `INS-04`).
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_002_us_008_us_009_us_017_us_019_us_020_us_025_us_026_on_a_dedicated_session() {
    let probe = probe();
    let dirty = dirty(1);
    let baseline = baseline();
    let list = [
        ("a/US-008-probe", "US-008 Record the start state", probe.as_str()),
        (
            "b/US-002-machines",
            "US-002 Every machine is addressed by its name",
            r#"
[call skd_fnc_machines, "==", ["server", "p1", "p2", "hc1"], "the machines"] call skd_fnc_expect;
[["hc1"] call skd_fnc_player, "==", objNull, "hc1 has no player"] call skd_fnc_expect;
[["server"] call skd_fnc_player, "==", objNull, "server has no player"] call skd_fnc_expect;
[name (["p2"] call skd_fnc_player), "==", "p2", "p2's player"] call skd_fnc_expect;
private _s = call skd_fnc_setup;
[[_s get "mode", _s get "clients", _s get "headless"], "==", ["dedicated", 2, 1], "the setup"] call skd_fnc_expect;
private _each = ["all", { [isServer, hasInterface, profileName] }] call skd_fnc_onEach;
[_each get "server", "==", [true, false, "server"], "the server"] call skd_fnc_expect;
[_each get "p1", "==", [false, true, "p1"], "p1"] call skd_fnc_expect;
[_each get "p2", "==", [false, true, "p2"], "p2"] call skd_fnc_expect;
[(_each get "hc1") select [0, 2], "==", [false, false], "hc1"] call skd_fnc_expect;
private _clients = ["clients", { profileName }] call skd_fnc_onEach;
[count _clients, "==", 2, "the group clients"] call skd_fnc_expect;
[keys (["headless", { 1 }] call skd_fnc_onEach), "==", ["hc1"], "the group headless"] call skd_fnc_expect;
private _box = ["Land_WoodenBox_F", AGLToASL [110, 110, 0], 45] call skd_fnc_create;
[["p2", { _this select 0 }, [_box]] call skd_fnc_on, "==", _box, "an object crosses to p2 and back"] call skd_fnc_expect;
["hc1", { !isNull (_this select 0) }, "hc1 sees the box", 10, [_box]] call skd_fnc_waitUntil;
[["p1", { false }, "never", 0.5] call skd_fnc_tryWaitUntil, "==", false, "tryWaitUntil on p1 returns false at its ceiling"] call skd_fnc_expect;
"#,
        ),
        (
            "c/US-017-scene",
            "US-017 A scene built on the server is the same on every machine",
            PORTABLE_SCENE,
        ),
        (
            "d/US-019-open-crate",
            "US-019 p1 opens the fixture's crate through its action and every machine sees the kit",
            PORTABLE_CRATE,
        ),
        (
            "e/US-020-wrong-title",
            "US-020 A wrong title fails assertion listing the titles",
            r#"
private _crate = ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
["use", { ["p1", _crate, "Open Crate", 2] call skd_fnc_useAction }] call skd_fnc_step;
"#,
        ),
        (
            "e/US-020-too-far",
            "US-020 A player out of the radius fails assertion with the distance and the radius",
            r#"
private _crate = ["skdfx_Crate", AGLToASL [100, 110, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
["use", { ["p1", _crate, "Open crate", 2] call skd_fnc_useAction }] call skd_fnc_step;
"#,
        ),
        (
            "e/US-020-already-open",
            "US-020 An opened crate fails assertion saying the condition is false",
            r#"
private _crate = ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["p1", AGLToASL [100, 100, 0], 0] call skd_fnc_place;
["p1", _crate, "Open crate"] call skd_fnc_useAction;
["all", { (_this select 0) getVariable ["skdfx_open", false] }, "==", true, "the crate is open", 10, [_crate]] call skd_fnc_expectEach;
["use again", { ["p1", _crate, "Open crate", 2] call skd_fnc_useAction }] call skd_fnc_step;
"#,
        ),
        (
            "e/US-020-server",
            "US-020 The server has no player to use an action",
            r#"
private _crate = ["skdfx_Crate", AGLToASL [100, 102, 0], 0] call skd_fnc_create;
["use", { ["server", _crate, "Open crate"] call skd_fnc_useAction }] call skd_fnc_step;
"#,
        ),
        (
            "f/US-025-agree",
            "US-025 Every machine agrees with the server on the fixture's flag",
            r#"
["server", { [5] call skdfx_fnc_setFlag }] call skd_fnc_on;
private _m = [["server", "p1", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees"] call skd_fnc_expectAgree;
[[_m get "server", _m get "p1", _m get "p2"], "==", [5, 5, 5], "each machine read 5"] call skd_fnc_expect;
["clients", { missionNamespace getVariable "skdfx_flag" }, "==", 5, "clients see 5"] call skd_fnc_expectEach;
["all", { private _u = (allPlayers select { name _x == "p2" }) select 0; getPosASL _u }, "every machine agrees on p2's position", 0.1] call skd_fnc_expectAgree;
"#,
        ),
        (
            "f/US-026-disagree",
            "US-026 One machine that disagrees is named with every machine's value",
            r#"
["server", { [5] call skdfx_fnc_setFlag }] call skd_fnc_on;
[["server", "p1", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees"] call skd_fnc_expectAgree;
["p2", { skdfx_flag = 6; true }] call skd_fnc_on;
["agree", { [["server", "p1", "p2"], { missionNamespace getVariable "skdfx_flag" }, "flag agrees", 0, 2] call skd_fnc_expectAgree }] call skd_fnc_step;
"#,
        ),
        (
            "g/US-008-dirty",
            "US-008 A check that changes everything",
            dirty.as_str(),
        ),
    ];
    let after = [
        (
            "h/US-008-baseline",
            "US-008 The next check sees the start state on every machine",
            baseline.as_str(),
        ),
        (
            "h/US-009-p2-alive",
            "US-009 p2, killed by the check before, is alive at its start position",
            r#"
["p2", { alive player }, "==", true, "p2 alive"] call skd_fnc_expectOn;
private _start = ((skd_test_start get "p2") select 0) select 0;
["server", { private _u = (allPlayers select { name _x == "p2" }) select 0; alive _u && { ((getPosASL _u) distance (_this select 0)) < 0.1 } }, "the server sees p2 alive at its start position", 10, [_start]] call skd_fnc_waitUntil;
"#,
        ),
    ];
    let mut all: Vec<(String, String)> = checks_of(&list);
    all.extend(checks_of(&after));
    let dir = project("api", &all);
    let session = Session::start(&dir, "ded");

    // US-002 1 and INS-04: four machines, each ready with its process and profile.
    let names: Vec<&str> = session.view["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["server", "p1", "p2", "hc1"]);
    for m in session.view["machines"].as_array().unwrap() {
        assert_eq!(m["state"], "ready", "{m}");
        assert!(m["pid"].is_u64() && m["profile"].is_string(), "{m}");
    }
    let (code, status) = cmd(&dir, &["status"]);
    assert_eq!(code, 0);
    assert_eq!(status["runner"], "running");
    assert_eq!(status["session"]["state"], "ready");
    assert_eq!(status["session"]["machines"].as_array().unwrap().len(), 4);

    let ids: Vec<&str> = list.iter().map(|(id, _, _)| *id).collect();
    let (code, result) = session.run(&ids);
    println!("{:#}", result["summary"]);
    for c in result["checks"].as_array().unwrap() {
        println!("{} {} {}", c["check_id"], c["outcome"], c["reason"]);
    }
    assert_eq!(code, 1, "the US-020 and US-026 checks fail");

    // US-002 3: every machine in the result, with its versions.
    let machines = result["session"]["machines"].as_array().unwrap();
    assert_eq!(machines.len(), 4);
    for m in machines {
        assert_eq!(m["start"], 1, "{m}");
        assert!(m["game_version"].as_str().unwrap().contains("2."), "{m}");
        assert_eq!(m["mods"][0]["patch"], "skdfx_main", "{m}");
        assert!(m["mods"][0]["version"].is_string(), "{m}");
    }
    assert_eq!(
        machines.iter().map(|m| m["kind"].as_str().unwrap()).collect::<Vec<_>>(),
        ["server", "client", "client", "headless"]
    );

    for id in [
        "a/US-008-probe",
        "b/US-002-machines",
        "c/US-017-scene",
        "d/US-019-open-crate",
        "f/US-025-agree",
        "g/US-008-dirty",
    ] {
        assert_outcome(&by_id(&result, id), "pass", None, &[]);
    }
    assert_outcome(
        &by_id(&result, "e/US-020-wrong-title"),
        "fail",
        Some("assertion"),
        &["no action titled \"Open Crate\"", "\"Open crate\""],
    );
    assert_outcome(
        &by_id(&result, "e/US-020-too-far"),
        "fail",
        Some("assertion"),
        &["outside the radius 3 m"],
    );
    assert_outcome(
        &by_id(&result, "e/US-020-already-open"),
        "fail",
        Some("assertion"),
        &["condition", "was false"],
    );
    assert_outcome(
        &by_id(&result, "e/US-020-server"),
        "fail",
        Some("check-error"),
        &["server has no player"],
    );
    let c = by_id(&result, "f/US-026-disagree");
    assert_outcome(&c, "fail", Some("assertion"), &["p2 read 6 where server read 5"]);
    assert_eq!(c["reason"]["machine"], "p2", "{c:#}");
    assert_eq!(
        c["evidence"]["observed"],
        json!({"server": 5, "p1": 5, "p2": 6}),
        "{c:#}"
    );

    // US-008 3: `reset` after the check that changed everything, under 2 s.
    let (code, reset) = cmd(&dir, &["reset"]);
    println!("US-008 reset: {reset}");
    assert_eq!(code, 0, "{reset}");
    assert_eq!(reset["state"], "done");
    assert!(reset["duration_s"].as_f64().unwrap() < 2.0, "{reset}");

    // The dirty check again, then the checks that look at the start state (US-008 1, US-009 2).
    let (code, result) = session.run(&["g/US-008-dirty", "h/US-008-baseline", "h/US-009-p2-alive"]);
    for c in result["checks"].as_array().unwrap() {
        println!("{} {} {}", c["check_id"], c["outcome"], c["reason"]);
    }
    assert_eq!(code, 0);
    for m in result["session"]["machines"].as_array().unwrap() {
        assert_eq!(m["start"], 1, "{m}");
    }
    drop(session);

    // US-017 3 and US-019 2: the same two check files, unchanged, in singleplayer.
    let sp = Session::start(&dir, "sp");
    let (code, result) = sp.run(&["c/US-017-scene", "d/US-019-open-crate"]);
    for c in result["checks"].as_array().unwrap() {
        println!("singleplayer: {} {} {}", c["check_id"], c["outcome"], c["reason"]);
    }
    assert_eq!(code, 0);
}

/// `US-004` 1 and 3: a game client ended during the start makes the start `failed` with exit
/// code 4, naming that machine, its state and its RPT lines, and leaves nothing running; then
/// `US-004` 2: a start ceiling of 30 s, shorter than a dedicated start, ends `failed` naming each
/// machine that was not ready. A new start is accepted after each.
#[test]
#[ignore = "starts Arma 3 three times; run on the test machine"]
fn us_004_a_start_that_never_becomes_ready_ends_failed_and_leaves_nothing() {
    let dir = project("start", &[]);
    assert!(game_pids().is_empty(), "no game may run when this test starts");

    // 1: end p2 by its process id once it is launched, found through `status` (INS-04).
    let child = Command::new(shakedown())
        .args(["session", "start", "--setup", "ded", "--json"])
        .current_dir(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut killed = None;
    let mut seen_server_alone = false;
    for _ in 0..240 {
        let (_, status) = cmd(&dir, &["status"]);
        // INS-04: a machine is listed only once its process is launched, with its process id and
        // process start time; while the server loads its mission, it is listed alone.
        let listed = status["session"]["machines"].as_array().cloned().unwrap_or_default();
        for m in &listed {
            assert!(
                m["pid"].is_u64() && m["process_started_at"].is_string(),
                "INS-04: a machine listed without its process: {m}"
            );
        }
        if listed.len() == 1 && listed[0]["name"] == "server" {
            seen_server_alone = true;
        }
        let p2 = listed.iter().find(|m| m["name"] == "p2").cloned();
        // p2 is ended once it has written its RPT, whose last lines the failed start names.
        let has_rpt = |m: &Value| {
            std::fs::read_dir(m["profile"].as_str().unwrap_or_default())
                .into_iter()
                .flatten()
                .flatten()
                .any(|e| e.path().extension().is_some_and(|x| x == "rpt"))
        };
        if let Some(pid) = p2.as_ref().filter(|m| has_rpt(m)).and_then(|m| m["pid"].as_u64()) {
            println!(
                "p2 launched as {pid} in state {}; ending it",
                p2.as_ref().unwrap()["state"]
            );
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            killed = Some(pid);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    assert!(killed.is_some(), "p2 was never launched");
    assert!(
        seen_server_alone,
        "status never listed the server alone while it loaded its mission"
    );
    let out = child.wait_with_output().unwrap();
    let session: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    println!("US-004 1: exit {:?}\n{stderr}\n{session:#}", out.status.code());
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(session["state"], "failed");
    assert_eq!(session["reason"]["machine"], "p2");
    let message = session["reason"]["message"].as_str().unwrap();
    assert!(
        message.contains("p2 exited") && message.contains("it was "),
        "{message}"
    );
    assert!(
        !session["reason"]["rpt_tail"].as_array().unwrap().is_empty(),
        "{session:#}"
    );
    assert!(game_pids().is_empty(), "a game process outlived the failed start");

    // 2: start ceiling 30 s.
    let t = Instant::now();
    let (code, session) = cmd(&dir, &["session", "start", "--setup", "ded-slow"]);
    let took = t.elapsed().as_secs_f64();
    println!("US-004 2: exit {code} after {took:.1} s: {:#}", session["reason"]);
    assert_eq!(code, 4);
    // SES-04: the start ends at its ceiling of 30 s, plus the stop of what it started.
    assert!(took < 45.0, "the start of ceiling 30 s returned after {took:.1} s");
    assert_eq!(session["state"], "failed");
    let message = session["reason"]["message"].as_str().unwrap();
    assert!(message.contains("start ceiling of 30 s"), "{message}");
    for m in session["machines"].as_array().unwrap() {
        if m["state"] != "ready" && m["state"] != "stopped" {
            panic!("{m}");
        }
    }
    assert!(game_pids().is_empty(), "a game process outlived the failed start");

    // 3: a new start is accepted.
    let s = Session::start(&dir, "ded");
    assert_eq!(s.view["state"], "ready");
}

/// Exit criterion 3, `NFR-RL-02`, `NFR-PF-04`, the dedicated half of `SPK-04`: 20 checks in one
/// dedicated session with two clients and a headless client, each changing everything, each
/// seeing the start state on every machine; every reset under 2 s.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn nfr_rl_02_nfr_pf_04_spk_04_twenty_checks_on_a_dedicated_session() {
    let mut checks = vec![("n/00".to_string(), check("NFR-RL-02 Record the start state", &probe()))];
    for n in 1..=20 {
        checks.push((
            format!("n/{n:02}"),
            check(
                &format!("NFR-RL-02 Check {n} sees the start state and changes everything"),
                &format!("{}\n{}", baseline(), dirty(n)),
            ),
        ));
    }
    let dir = project("twenty", &checks);
    let session = Session::start(&dir, "ded");
    let log = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
        .join("Shakedown")
        .join("runner.log");
    let ids: Vec<String> = (0..=20).map(|n| format!("n/{n:02}")).collect();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let (code, result) = session.run(&refs);
    for c in result["checks"].as_array().unwrap() {
        println!("{} {} {} {}", c["check_id"], c["outcome"], c["duration_s"], c["reason"]);
    }
    let resets: Vec<f64> = std::fs::read_to_string(&log)
        .unwrap()
        .lines()
        .filter(|l| l.contains(&session.id) && l.contains(" reset done in "))
        .filter_map(|l| l.split(" reset done in ").nth(1)?.trim_end_matches(" s").parse().ok())
        .collect();
    let max = resets.iter().cloned().fold(0.0, f64::max);
    let mean = resets.iter().sum::<f64>() / resets.len().max(1) as f64;
    println!(
        "SPK-04 dedicated: {} resets, mean {mean:.3} s, max {max:.3} s: {resets:?}",
        resets.len()
    );
    assert_eq!(code, 0);
    assert_eq!(result["summary"]["pass"], 21);
    for m in result["session"]["machines"].as_array().unwrap() {
        assert_eq!(m["start"], 1, "{m}");
        assert_eq!(m["end_state"], "ready", "{m}");
    }
    assert_eq!(resets.len(), 21);
    assert!(max < 2.0, "a reset took {max} s");
}

/// Exit criterion 2: the demo check, arma-grappling-hook's `US-009` with a freed rope on a
/// dedicated session with two clients, 10 times against one session; and exit criterion 4: the
/// M1 demo check unchanged in the same session.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn m2_demo_us_009_ten_times_on_a_dedicated_session() {
    let demo_dir = repo().join("e2e").join("grappling-hook");
    let t = Instant::now();
    let (code, s) = cmd(&demo_dir, &["session", "start", "--setup", "dedicated"]);
    println!("start: exit {code} in {:.1} s", t.elapsed().as_secs_f64());
    assert_eq!(code, 0, "{s}");
    let session = Session {
        dir: demo_dir.clone(),
        id: s["id"].as_str().unwrap().to_string(),
        pids: s["machines"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["pid"].as_u64())
            .collect(),
        view: s,
    };
    let mut passed = 0;
    let mut times = Vec::new();
    for n in 1..=10 {
        let (code, result) = session.run(&["US-009-dedicated-freed-rope"]);
        let c = by_id(&result, "US-009-dedicated-freed-rope");
        println!(
            "demo run {n}: exit {code}: {} {} {}",
            c["outcome"], c["duration_s"], c["reason"]
        );
        times.push(c["duration_s"].as_f64().unwrap());
        if c["outcome"] == "pass" {
            passed += 1;
        }
    }
    println!("demo: {passed} of 10 passed; durations {times:?}");
    let (code, result) = session.run(&["US-009-let-go-key", "US-009-let-go-action"]);
    for c in result["checks"].as_array().unwrap() {
        println!(
            "M1 demo in the dedicated setup: {} {} {}",
            c["check_id"], c["outcome"], c["reason"]
        );
    }
    assert_eq!(passed, 10);
    // Exit criterion 4: the M1 demo checks name the lead's own `player`, which a dedicated server
    // does not have, so they end check-error with the way to name p1's unit instead.
    assert_eq!(code, 1);
    for c in result["checks"].as_array().unwrap() {
        assert_outcome(c, "fail", Some("check-error"), &["skd_fnc_player"]);
    }
}
