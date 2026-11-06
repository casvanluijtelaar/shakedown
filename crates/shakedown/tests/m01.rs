//! M1's end-to-end tests: the stories `US-018`, `US-021`, `US-022` and `US-039`, the rest of the
//! check API a real check needs, the reset between checks (`NFR-RL-02`, the singleplayer half of
//! `SPK-04`) and the demo. They drive the installed `shakedown` command against the game, so they
//! run only on the test machine, by hand, one at a time:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m01 -- --ignored --nocapture --test-threads=1
//!
//! The demo alone (exit criterion 2) is `m1_demo_us_009_ten_times_with_a_second_check`.

use std::path::{Path, PathBuf};
use std::process::Command;

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

/// A project in a temporary folder with the setups `sp` (no mods), `sp-fixture` (the fixture
/// alone, no CBA) and `sp-cba` (CBA and both fixtures), and the given checks.
fn project(name: &str, checks: &[(String, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m01-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let fixture_cba = install().join("fixtures").join("@skd_fixture_cba");
    let toml = format!(
        r#"game = '{GAME}'

[mods.cba]
path = '{CBA}'
patch = "cba_main"

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[mods.fixture-cba]
path = '{}'
patch = "skdfx_cba"

[setups.sp]
mode = "singleplayer"
mods = []

[setups.sp-fixture]
mode = "singleplayer"
mods = ["fixture"]

[setups.sp-cba]
mode = "singleplayer"
mods = ["cba", "fixture", "fixture-cba"]
"#,
        fixture_cba.display()
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

/// A started session, stopped when dropped, also when the test panics.
struct Session {
    dir: PathBuf,
    id: String,
    pid: u64,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        let (code, s) = cmd(dir, &["session", "start", "--setup", setup]);
        assert_eq!(code, 0, "{s}");
        Session {
            dir: dir.to_path_buf(),
            id: s["id"].as_str().unwrap().to_string(),
            pid: s["machines"][0]["pid"].as_u64().unwrap(),
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
            let _ = Command::new("taskkill")
                .args(["/PID", &self.pid.to_string(), "/F"])
                .output();
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

/// `US-018`, `US-021`, `US-022` (with CBA), `US-039`, and every other check API function M1
/// builds, in one session of `sp-cba`; then `US-022` criterion 2 in a session without CBA.
#[test]
#[ignore = "starts Arma 3 twice; run on the test machine"]
fn us_018_us_021_us_022_us_039_and_the_check_api() {
    let list = [
        // US-018 1 to 3, each in a step, so the failed step's path is in the evidence (4).
        (
            "a/US-018-unknown-class",
            "US-018 An unknown class name ends check-error",
            r#"["create", { ["B_Soldier_Typo", [100, 110, 5]] call skd_fnc_create }] call skd_fnc_step;"#,
        ),
        (
            "a/US-018-place-position",
            "US-018 A position of the wrong type ends check-error",
            r#"["place", { ["p1", "here", 0] call skd_fnc_place }] call skd_fnc_step;"#,
        ),
        (
            "a/US-018-ceiling-negative",
            "US-018 A negative ceiling ends check-error",
            r#"["wait", { ["p1", {true}, "ready", -1] call skd_fnc_waitUntil }] call skd_fnc_step;"#,
        ),
        (
            "a/US-018-ceiling-string",
            "US-018 A ceiling given as a string ends check-error",
            r#"["wait", { ["p1", {true}, "ready", "10"] call skd_fnc_waitUntil }] call skd_fnc_step;"#,
        ),
        (
            "a/US-018-next-check-passes",
            "US-018 The next check of the run executes normally",
            r#"["p1", { 1 + 1 }, "==", 2, "the sum"] call skd_fnc_expectOn;"#,
        ),
        // US-021 1 and 2, against the fixture's keybind. The counts on p1's unit are read before,
        // since the reset keeps a unit's variables.
        (
            "b/US-021-press",
            "US-021 Pressing the fixture's signal keybind runs its down and up code once",
            r#"
private _before = ["p1", { [player getVariable ["skdfx_signalDown", 0], player getVariable ["skdfx_signalUp", 0]] }] call skd_fnc_on;
["press", {
    [["p1", "skd_fixture", "signal"] call skd_fnc_pressKey, "==", true, "pressKey returns true"] call skd_fnc_expect;
}] call skd_fnc_step;
["server", { [player getVariable ["skdfx_signalDown", 0], player getVariable ["skdfx_signalUp", 0]] }, "==", [(_before select 0) + 1, (_before select 1) + 1], "the server reads one down and one up on p1's unit"] call skd_fnc_expectOn;
"#,
        ),
        (
            "b/US-021-down-then-up",
            "US-021 Mode down runs only the down code and mode up the up code",
            r#"
private _read = { [player getVariable ["skdfx_signalDown", 0], player getVariable ["skdfx_signalUp", 0]] };
private _before = ["p1", _read] call skd_fnc_on;
["p1", "skd_fixture", "signal", "down"] call skd_fnc_pressKey;
["server", _read, "==", [(_before select 0) + 1, _before select 1], "only the down count rises"] call skd_fnc_expectOn;
["p1", "skd_fixture", "signal", "up"] call skd_fnc_pressKey;
["server", _read, "==", [(_before select 0) + 1, (_before select 1) + 1], "then the up count rises"] call skd_fnc_expectOn;
"#,
        ),
        // US-022 1.
        (
            "c/US-022-unregistered",
            "US-022 A keybind that is not registered ends check-error listing those that are",
            r#"["press", { ["p1", "skd_fixture", "signl"] call skd_fnc_pressKey }] call skd_fnc_step;"#,
        ),
        // US-039 1 and 2.
        (
            "d/US-039-slow-part",
            "US-039 A body that runs past its timeout ends check-timeout in its step",
            "// @timeout 5\n[\"slow part\", { private _t = diag_tickTime; while { diag_tickTime - _t < 20 } do { [\"p1\", { false }, \"never\", 1] call skd_fnc_tryWaitUntil } }] call skd_fnc_step;",
        ),
        (
            "d/US-039-long-wait",
            "US-039 A wait whose ceiling reaches past the timeout is cut and the check ends check-timeout",
            "// @timeout 5\n[\"long wait\", { [\"p1\", { false }, \"never\", 30] call skd_fnc_waitUntil }] call skd_fnc_step;",
        ),
        // API-03: the context, values across the channel, waits, polled assertions, the scene,
        // actions and cleanups.
        (
            "e/API-03-the-check-api",
            "API-03 The context, code on machines, waits, assertions, the scene and an action pass",
            r#"
["context", {
    [call skd_fnc_machines, "==", ["p1"], "the machines"] call skd_fnc_expect;
    private _s = call skd_fnc_setup;
    [[_s get "name", _s get "mode", _s get "clients", _s get "headless", _s get "world"], "==", ["sp-cba", "singleplayer", 1, 0, "VR"], "the setup"] call skd_fnc_expect;
    [["p1"] call skd_fnc_player, "==", player, "p1's player"] call skd_fnc_expect;
    [["server"] call skd_fnc_player, "==", player, "server names p1 in singleplayer"] call skd_fnc_expect;
}] call skd_fnc_step;
private _box = ["values cross by DM-13 and DM-14", {
    private _box = ["Land_WoodenBox_F", AGLToASL [110, 110, 0], 45] call skd_fnc_create;
    [["p1", { _this select 0 }, [_box]] call skd_fnc_on, "==", _box, "an object comes back as the object"] call skd_fnc_expect;
    private _code = ["p1", { _this select 0 }, [{ 1 }]] call skd_fnc_on;
    [_code get "$type", "==", "code", "code stays a tagged value"] call skd_fnc_expect;
    private _local = 7;
    [["p1", { isNil "_local" }] call skd_fnc_on, "==", true, "sent code does not see the check's locals"] call skd_fnc_expect;
    private _each = ["all", { getDir (_this select 0) }, [_box]] call skd_fnc_onEach;
    [_each get "p1", "~=", [45, 0.5], "onEach reads the box's direction on p1"] call skd_fnc_expect;
    _box
}] call skd_fnc_step;
["waits", {
    [["p1", { time > 0 }, "the mission runs"] call skd_fnc_waitUntil, "==", true, "waitUntil returns true"] call skd_fnc_expect;
    [["p1", { false }, "never", 0.3] call skd_fnc_tryWaitUntil, "==", false, "tryWaitUntil returns false at its ceiling"] call skd_fnc_expect;
}] call skd_fnc_step;
["polled assertions", {
    ["p1", { 6 * 7 }, "==", 42, "the product"] call skd_fnc_expectOn;
    private _m = ["all", { [1, 2, 3] }, "contains", 2, "the list holds 2"] call skd_fnc_expectEach;
    [_m get "p1", "==", [1, 2, 3], "expectEach returns each machine's value"] call skd_fnc_expect;
    ["all", { getPosASL player }, "every machine agrees on p1's position", 0.1] call skd_fnc_expectAgree;
}] call skd_fnc_step;
["the scene", {
    [[2035, 1, 2, 3, 4]] call skd_fnc_setDate;
    [(date select [0, 5]), "==", [2035, 1, 2, 3, 4], "the date"] call skd_fnc_expect;
    [0.8, 0.2, 0.4] call skd_fnc_setWeather;
    [overcast, "~=", [0.8, 0.01], "the overcast"] call skd_fnc_expect;
    ["p1", AGLToASL [120, 120, 0], 90] call skd_fnc_place;
    [getPosASL player, "~=", [AGLToASL [120, 120, 0], 0.1], "p1 stands where it was placed"] call skd_fnc_expect;
}] call skd_fnc_step;
["an action, its tags stripped", {
    skd_e2e_waved = 0;
    private _id = player addAction ["<t color='#ff0000'>Wave</t>", { skd_e2e_waved = skd_e2e_waved + 1 }, nil, 1.5, true, true, "", "true", 5];
    [{ player removeAction (_this select 0); skd_e2e_cleaned = true }, [_id]] call skd_fnc_cleanup;
    ["p1", player, "Wave"] call skd_fnc_useAction;
    ["p1", { skd_e2e_waved }, "==", 1, "the action's script ran once"] call skd_fnc_expectOn;
}] call skd_fnc_step;
"#,
        ),
        (
            "e/API-03-the-cleanup-ran",
            "API-03 The previous check's cleanup ran at the reset and the reset removed its box",
            r#"
[missionNamespace getVariable ["skd_e2e_cleaned", false], "==", true, "the cleanup ran"] call skd_fnc_expect;
[count (allMissionObjects "Land_WoodenBox_F"), "==", 0, "no box is left"] call skd_fnc_expect;
[(date select [0, 3]), "!=", [2035, 1, 2], "the date is back"] call skd_fnc_expect;
[(actionIDs player) findIf { ((player actionParams _x) select 0) find "Wave" >= 0 }, "==", -1, "the action is gone"] call skd_fnc_expect;
"#,
        ),
        (
            "f/API-03-action-missing",
            "API-03 An action that does not exist fails assertion listing the titles",
            r#"
private _id = player addAction ["Salute", {}];
[{ player removeAction (_this select 0) }, [_id]] call skd_fnc_cleanup;
["use", { ["p1", player, "Nope", 0.5] call skd_fnc_useAction }] call skd_fnc_step;
"#,
        ),
        (
            "f/API-03-expect-on-fails",
            "API-03 A polled assertion that never holds fails assertion with the last value",
            r#"["p1", { 1 }, "==", 2, "one is two", 0.5] call skd_fnc_expectOn;"#,
        ),
        (
            "f/API-03-wait-ceiling",
            "API-03 A wait that never holds fails wait-ceiling with its description",
            r#"["p1", { false }, "the sky falls", 0.5] call skd_fnc_waitUntil;"#,
        ),
        (
            "f/API-03-invalid",
            "API-03 skd_fnc_invalid ends the check invalid explicit",
            r#"["the fixture is not loaded"] call skd_fnc_invalid;"#,
        ),
        (
            "f/API-03-unknown-machine",
            "API-03 A machine the session does not have ends check-error",
            r#"["p2", { 1 }] call skd_fnc_on;"#,
        ),
    ];
    let checks = checks_of(&list);
    let dir = project("api", &checks);
    let session = Session::start(&dir, "sp-cba");
    let ids: Vec<&str> = list.iter().map(|(id, _, _)| *id).collect();
    let (code, result) = session.run(&ids);
    assert_eq!(code, 1, "a check failed");

    for (id, needles) in [
        ("a/US-018-unknown-class", vec!["B_Soldier_Typo", "CfgVehicles"]),
        (
            "a/US-018-place-position",
            vec!["skd_fnc_place", "position", "ASL position [x, y, z]"],
        ),
        (
            "a/US-018-ceiling-negative",
            vec!["skd_fnc_waitUntil", "ceiling", "number of seconds above 0"],
        ),
        (
            "a/US-018-ceiling-string",
            vec!["skd_fnc_waitUntil", "ceiling", "number of seconds above 0"],
        ),
    ] {
        let c = by_id(&result, id);
        assert_outcome(&c, "fail", Some("check-error"), &needles);
        assert!(c["evidence"]["failed_step"].is_array(), "{c:#}");
    }
    assert_outcome(&by_id(&result, "a/US-018-next-check-passes"), "pass", None, &[]);
    assert_outcome(&by_id(&result, "b/US-021-press"), "pass", None, &[]);
    assert_outcome(&by_id(&result, "b/US-021-down-then-up"), "pass", None, &[]);
    let c = by_id(&result, "c/US-022-unregistered");
    assert_outcome(&c, "fail", Some("check-error"), &["signl", "signal"]);

    for id in ["d/US-039-slow-part", "d/US-039-long-wait"] {
        let c = by_id(&result, id);
        // NFR-US-01, finding m01-2: the message names the step and the next thing to do.
        assert_outcome(&c, "fail", Some("check-timeout"), &["in step \"", "Raise @timeout"]);
        let d = c["duration_s"].as_f64().unwrap();
        assert!((4.5..9.0).contains(&d), "{id} took {d} s");
        let step = c["steps"][0].clone();
        assert_eq!(step["state"], "running", "{c:#}");
        assert_eq!(c["evidence"]["failed_step"], json!([step["path"][0]]), "{c:#}");
    }
    assert_eq!(
        by_id(&result, "d/US-039-slow-part")["evidence"]["failed_step"],
        json!(["slow part"])
    );

    assert_outcome(&by_id(&result, "e/API-03-the-check-api"), "pass", None, &[]);
    assert_outcome(&by_id(&result, "e/API-03-the-cleanup-ran"), "pass", None, &[]);
    assert_outcome(
        &by_id(&result, "f/API-03-action-missing"),
        "fail",
        Some("assertion"),
        &["Nope", "Salute"],
    );
    let c = by_id(&result, "f/API-03-expect-on-fails");
    assert_outcome(&c, "fail", Some("assertion"), &["one is two", "last read 1"]);
    assert_eq!(c["evidence"]["observed"]["p1"], 1);
    assert_outcome(
        &by_id(&result, "f/API-03-wait-ceiling"),
        "fail",
        Some("wait-ceiling"),
        &["the sky falls", "0.5"],
    );
    assert_outcome(
        &by_id(&result, "f/API-03-invalid"),
        "invalid",
        Some("explicit"),
        &["not loaded"],
    );
    assert_outcome(
        &by_id(&result, "f/API-03-unknown-machine"),
        "fail",
        Some("check-error"),
        &["p2", "p1"],
    );
    drop(session);

    // US-022 2: without CBA, pressing a key is the author's mistake; a check that presses none passes.
    let list = [
        (
            "US-022-no-cba",
            "US-022 Pressing a keybind without CBA ends check-error",
            r#"["p1", "skd_fixture", "signal"] call skd_fnc_pressKey;"#,
        ),
        (
            "US-022-no-key-passes",
            "US-022 A check that presses no key passes without CBA",
            r#"[isClass (configFile >> "CfgPatches" >> "skdfx_main"), "==", true, "the fixture is loaded"] call skd_fnc_expect;"#,
        ),
    ];
    let dir = project("nocba", &checks_of(&list));
    let session = Session::start(&dir, "sp-fixture");
    let (code, result) = session.run(&["US-022-no-cba", "US-022-no-key-passes"]);
    assert_eq!(code, 1);
    assert_outcome(
        &by_id(&result, "US-022-no-cba"),
        "fail",
        Some("check-error"),
        &["CBA is not loaded on p1"],
    );
    assert_outcome(&by_id(&result, "US-022-no-key-passes"), "pass", None, &[]);
}

/// The body of check `n` of the 20 of `NFR-RL-02`: it records the state it starts from as
/// evidence, then creates units, vehicles and objects (through the API and through its own SQF)
/// and changes time, weather and the player, a different way in each of four kinds.
fn twenty(n: usize) -> String {
    let change = match n % 4 {
        0 => "player setDamage 1;",
        1 => {
            r#"private _car = ["B_MRAP_01_F", AGLToASL [104, 104, 0], 0] call skd_fnc_create;
    player moveInDriver _car;
    ["p1", { vehicle player != player }, "p1 is in the car"] call skd_fnc_waitUntil;"#
        }
        2 => {
            r#"["p1", AGLToASL [130, 90, 0], 200] call skd_fnc_place;
    player setDamage 0.6;
    removeAllWeapons player;
    removeBackpack player;"#
        }
        _ => {
            r#"player addBackpack "B_Carryall_oli";
    player switchMove "AmovPpneMstpSrasWrflDnon";
    player setDir 123;
    player setPosASL (AGLToASL [90, 95, 0]);
    player setDamage 0.3;"#
        }
    };
    format!(
        r#"// @check NFR-RL-02 Check {n} of 20 sees the start state, then changes it
// @tags NFR-RL-02 SPK-04
["the start state", {{
    private _u = player;
    private _r = {{ (round (_this * 100)) / 100 }};
    ["start", createHashMapFromArray [
        ["position", (getPosASL _u) apply {{ _x call _r }}],
        ["direction", round getDir _u],
        ["damage", (damage _u) call _r],
        ["alive", alive _u],
        ["in a vehicle", vehicle _u != _u],
        ["loadout", str getUnitLoadout _u],
        ["animation", animationState _u],
        ["date", date],
        ["weather", [overcast call _r, fog call _r, rain call _r]],
        ["vehicles", count vehicles],
        ["units", count allUnits],
        ["dead", count allDeadMen],
        ["groups", count allGroups],
        ["objects", count (allMissionObjects "All")]
    ]] call skd_fnc_evidence;
}}] call skd_fnc_step;
["create and change", {{
    ["B_Soldier_F", AGLToASL [110, 100, 0], 90] call skd_fnc_create;
    ["B_Quadbike_01_F", AGLToASL [110, 110, 0], 0] call skd_fnc_create;
    ["Land_WoodenBox_F", AGLToASL [112, 100, 0], 30] call skd_fnc_create;
    private _g = createGroup east;
    _g createUnit ["O_Soldier_F", [115, 115, 0], [], 0, "CAN_COLLIDE"];
    createVehicle ["Land_Cargo20_military_green_F", [120, 130, 0], [], 0, "CAN_COLLIDE"];
    [[2030 + {y}, 1 + {m}, 2, {h}, 30]] call skd_fnc_setDate;
    [0.9, 0.3, 0.5] call skd_fnc_setWeather;
    {change}
}}] call skd_fnc_step;
"#,
        y = n % 5,
        m = n % 12,
        h = n % 24,
    )
}

/// M1 exit criterion 3, `NFR-RL-02` and the singleplayer half of `SPK-04`: 20 checks in one
/// singleplayer session, each creating units, vehicles and objects and changing time, weather and
/// the player (dying, in a car, moved and hurt and stripped), each seeing the start state the
/// first one saw, with the reset under 2 s.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state() {
    let checks: Vec<(String, String)> = (1..=20).map(|n| (format!("rl/{n:02}"), twenty(n))).collect();
    let dir = project("twenty", &checks);
    let session = Session::start(&dir, "sp");
    let ids: Vec<String> = (1..=20).map(|n| format!("rl/{n:02}")).collect();
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let (code, result) = session.run(&ids);
    let executions = result["checks"].as_array().unwrap();
    assert_eq!(executions.len(), 20);
    for c in executions {
        assert_eq!(c["outcome"], "pass", "{c:#}");
    }
    assert_eq!(code, 0);
    let start = |c: &Value| c["evidence"]["attached"][0]["value"].clone();
    let first = start(&executions[0]);
    println!("NFR-RL-02 the start state: {first:#}");
    for c in executions {
        assert_eq!(start(c), first, "{} saw another start state", c["check_id"]);
    }

    // SPK-04: the reset times the runner logged for this session.
    let log = std::fs::read_to_string(
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
            .join("Shakedown")
            .join("runner.log"),
    )
    .unwrap();
    let times: Vec<f64> = log
        .lines()
        .filter(|l| l.contains(&format!("{} reset done in", session.id)))
        .filter_map(|l| l.rsplit("in ").next()?.trim_end_matches(" s").parse().ok())
        .collect();
    let max = times.iter().cloned().fold(0.0, f64::max);
    let mean = times.iter().sum::<f64>() / times.len().max(1) as f64;
    println!(
        "SPK-04 singleplayer: {} resets, mean {mean:.3} s, max {max:.3} s: {times:?}",
        times.len()
    );
    assert_eq!(times.len(), 20);
    assert!(max < 2.0, "every reset under 2 s");
}

/// M1 exit criterion 2: the demo check, arma-grappling-hook's `US-009` rewritten, passes 10 times
/// out of 10, each in one run together with a second check, against one session.
#[test]
#[ignore = "starts Arma 3 with CBA and arma-grappling-hook; run on the test machine"]
fn m1_demo_us_009_ten_times_with_a_second_check() {
    let dir = repo().join("e2e").join("grappling-hook");
    let session = Session::start(&dir, "sp");
    let mut passed = 0;
    for n in 1..=10 {
        let (code, result) = session.run(&["US-009-let-go-key", "US-009-let-go-action"]);
        let outcomes: Vec<String> = result["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| format!("{} {} {}", c["check_id"], c["outcome"], c["reason"]))
            .collect();
        println!("demo run {n}: exit {code}: {outcomes:?}");
        if by_id(&result, "US-009-let-go-key")["outcome"] == "pass" {
            passed += 1;
        }
        assert_eq!(code, 0, "{outcomes:?}");
    }
    assert_eq!(passed, 10);
}

/// `US-031` criterion 2, finding `m01-3`: `run a a b` against a running session executes `a`,
/// `a`, `b` in the game, three executions in that order with `repeat` 1, 2 and 1, each a step of
/// its own in the game, and prints a line per execution, the second of `a` carrying `#2` right
/// after its check id, before the duration.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_031_m01_3_a_check_given_twice_runs_twice_in_the_order_given() {
    let body = r#"["ran", { ["tick", diag_tickTime] call skd_fnc_evidence; true }] call skd_fnc_step;"#;
    let dir = project(
        "repeat",
        &checks_of(&[
            ("rep/a", "US-031 a check given twice", body),
            ("rep/b", "US-031 a check given once", body),
        ]),
    );
    let session = Session::start(&dir, "sp");

    let (code, result) = session.run(&["rep/a", "rep/a", "rep/b"]);
    assert_eq!(code, 0, "{result:#}");
    let executions = result["checks"].as_array().unwrap();
    let seen: Vec<(String, u64, String)> = executions
        .iter()
        .map(|c| {
            (
                c["check_id"].as_str().unwrap().to_string(),
                c["repeat"].as_u64().unwrap(),
                c["outcome"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    println!("US-031 m01-3 a a b: {seen:?}");
    assert_eq!(
        seen,
        [
            ("rep/a".to_string(), 1, "pass".to_string()),
            ("rep/a".to_string(), 2, "pass".to_string()),
            ("rep/b".to_string(), 1, "pass".to_string()),
        ]
    );
    assert_eq!(result["summary"]["total"], 3);
    assert_eq!(result["summary"]["pass"], 3);
    assert_eq!(result["run"]["selection"]["ids"], json!(["rep/a", "rep/a", "rep/b"]));
    // Each execution ran in the game on its own: one step each, later ticks for later executions.
    let ticks: Vec<f64> = executions
        .iter()
        .map(|c| {
            assert_eq!(c["steps"].as_array().unwrap().len(), 1, "{c:#}");
            c["evidence"]["attached"][0]["value"].as_f64().unwrap()
        })
        .collect();
    assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2], "{ticks:?}");

    // The printed lines: one per execution, in order, the second of `a` marked `#2`.
    let out = Command::new(shakedown())
        .args(["run", "rep/a", "rep/b", "rep/a"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    println!("{stdout}");
    assert_eq!(out.status.code(), Some(0), "{stdout}");
    let lines: Vec<&str> = stdout.lines().filter(|l| l.starts_with("pass")).collect();
    assert_eq!(lines.len(), 3, "{stdout}");
    assert!(lines[0].contains("rep/a  ("), "{stdout}");
    assert!(lines[1].contains("rep/b  ("), "{stdout}");
    assert!(lines[2].contains("rep/a #2  ("), "{stdout}");
    assert!(stdout.contains("3 passed, 0 failed"), "{stdout}");
}
