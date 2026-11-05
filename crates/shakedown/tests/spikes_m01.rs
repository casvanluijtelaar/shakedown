//! Spikes SPK-12, SPK-10 and SPK-05 for singleplayer `p1` (`spec/06-open-questions.md`), run
//! before the M1 work that depends on them. They start the game, so they run only on the test
//! machine, by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes_m01 -- --ignored --nocapture
//!
//! `SPK-04`'s singleplayer half runs through the command line in `e2e.rs`
//! (`nfr_rl_02_spk_04_twenty_checks_each_see_the_start_state`). Results go to `research/`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shakedown::config;
use shakedown::runner::{Runner, session};
use shakedown::wire::SetupSpec;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn plain(p: PathBuf) -> PathBuf {
    let p = std::fs::canonicalize(p).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

async fn start() -> (std::sync::Arc<Runner>, std::sync::Arc<session::Session>) {
    let e2e = plain(repo().join("e2e"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup("sp").unwrap().clone();
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods: vec![],
    };
    let addon = plain(repo().join("target").join("shakedown").join("@shakedown"));
    let runner = Runner::bind(addon, None).await.unwrap();
    let session = session::start(&runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    (runner, session)
}

async fn value(session: &session::Session, source: &str, ceiling: f64) -> Value {
    match session.exec("p1", source, json!([]), ceiling).await {
        Ok(a) if a["state"] == "done" => a["value"].clone(),
        Ok(a) => json!({"answer": a}),
        Err(e) => json!({"no-answer": format!("{e:?}")}),
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_12_spk_10_spk_05_singleplayer_p1() {
    let (runner, session) = start().await;

    // SPK-12: objectFromNetId (netId _obj) in singleplayer, for a created object, a created unit,
    // the player and a group.
    let spk12 = value(
        &session,
        r#"
        private _o = createVehicle ["Land_WoodenBox_F", player getPos [8, 0], [], 0, "CAN_COLLIDE"];
        private _o2 = createVehicle ["Land_WoodenBox_F", player getPos [8, 45], [], 0, "CAN_COLLIDE"];
        private _g = createGroup west;
        private _u = _g createUnit ["B_Soldier_F", player getPos [8, 90], [], 0, "CAN_COLLIDE"];
        private _r = [
            ["object", netId _o, objectFromNetId (netId _o) isEqualTo _o],
            ["object2", netId _o2, objectFromNetId (netId _o2) isEqualTo _o2],
            ["unit", netId _u, objectFromNetId (netId _u) isEqualTo _u],
            ["player", netId player, objectFromNetId (netId player) isEqualTo player],
            ["group", netId _g, groupFromNetId (netId _g) isEqualTo _g],
            ["null", netId objNull, isNull (objectFromNetId (netId objNull))]
        ];
        deleteVehicle _o; deleteVehicle _o2; deleteVehicle _u; deleteGroup _g;
        _r"#,
        10.0,
    )
    .await;
    println!("SPK-12 {spk12}");
    let spk12_pass = spk12
        .as_array()
        .is_some_and(|rows| rows.iter().all(|r| r[2] == json!(true)) && rows[0][1] != rows[1][1]);
    println!("SPK-12 pass: {spk12_pass}");

    // SPK-10: does action ["User", target, id] honour the action's radius and condition? An action
    // on a box 10 m away (radius 3), then 2 m away with its condition false, then true; and an
    // action on the player's own unit with its condition false, then true.
    let spk10 = value(
        &session,
        r#"
        skd_spk10 = 0;
        private _box = createVehicle ["Box_NATO_Ammo_F", [0, 0, 0], [], 0, "CAN_COLLIDE"];
        _box setPosASL (AGLToASL (player getPos [10, getDir player]));
        _box setVariable ["spk10", true];
        private _id = _box addAction ["Spike ten", { skd_spk10 = skd_spk10 + 1 }, nil, 1.5, true, true, "", "_target getVariable ['spk10', false]", 3];
        player action ["User", _box, _id];
        private _t = diag_tickTime; waitUntil { diag_tickTime - _t > 1 };
        private _far = skd_spk10;
        _box setPosASL (AGLToASL (player getPos [2, getDir player]));
        _box setVariable ["spk10", false];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 0.5 };
        player action ["User", _box, _id];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 1 };
        private _falseNear = skd_spk10 - _far;
        _box setVariable ["spk10", true];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 0.5 };
        player action ["User", _box, _id];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 1 };
        private _trueNear = skd_spk10 - _far - _falseNear;
        deleteVehicle _box;
        skd_spk10self = 0;
        player setVariable ["spk10", false];
        private _sid = player addAction ["Spike ten self", { skd_spk10self = skd_spk10self + 1 }, nil, 1.5, true, true, "", "_this isEqualTo _target && {_target getVariable ['spk10', false]}"];
        player action ["User", player, _sid];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 1 };
        private _selfFalse = skd_spk10self;
        player setVariable ["spk10", true];
        player action ["User", player, _sid];
        _t = diag_tickTime; waitUntil { diag_tickTime - _t > 1 };
        private _selfTrue = skd_spk10self - _selfFalse;
        player removeAction _sid;
        [_far, _falseNear, _trueNear, _selfFalse, _selfTrue]"#,
        20.0,
    )
    .await;
    println!(
        "SPK-10 runs of the action's script: out of radius {}, in radius with the condition false {}, in radius with it true {}; on the player's own unit with the condition false {}, true {}",
        spk10[0], spk10[1], spk10[2], spk10[3], spk10[4]
    );
    let spk10_pass = spk10 == json!([0, 0, 1, 0, 1]);
    println!("SPK-10 pass: {spk10_pass}");

    // SPK-05, part 1: the player dies in an -autotest session with respawn = 3. Does the mission
    // keep running, and does the engine bring the player back? The measurement in research was
    // taken with M0's agent, which had no Killed handler; the agent registers its own handler
    // (RST-05), so with it this part measures that handler instead of the engine.
    let probe = r#"[alive player, netId player, typeOf player, getPosASL player, uiNamespace getVariable ["skd_missionStarts", 0], time, diag_tickTime]"#;
    let before = value(&session, probe, 10.0).await;
    println!("SPK-05 before death: {before}");
    let mut engine_respawn = Vec::new();
    for n in 1..=3 {
        let starts = session.mission_start("p1");
        let _ = value(&session, "player setDamage 1; true", 10.0).await;
        let t = Instant::now();
        let mut back = None;
        let mut last = Value::Null;
        while t.elapsed() < Duration::from_secs(15) {
            last = value(&session, probe, 3.0).await;
            if last[0] == json!(true) {
                back = Some(t.elapsed().as_secs_f64());
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        println!(
            "SPK-05 death {n} without a handler: alive again after {back:?} s; last probe {last}; mission start {} -> {}",
            starts,
            session.mission_start("p1")
        );
        engine_respawn.push(back);
        if back.is_none() {
            break;
        }
    }

    // The first fallback: a Killed handler hands the player a new unit of the same class at the
    // spot with selectPlayer. Tried on a dead player first (as a reset would), then installed and
    // the player killed 10 times.
    let rescue = r#"
        if (alive player) exitWith { "alive" };
        private _old = player;
        private _g = createGroup (side group _old);
        if (isNull _g) then { _g = createGroup west };
        private _u = _g createUnit [typeOf _old, [0, 0, 0], [], 0, "CAN_COLLIDE"];
        _u setPosASL (getPosASL _old);
        selectPlayer _u;
        [alive player, player isEqualTo _u]"#;
    let rescued = value(&session, rescue, 10.0).await;
    println!("SPK-05 selectPlayer on a dead player: {rescued}");
    let after_rescue = value(&session, probe, 5.0).await;
    println!("SPK-05 after selectPlayer: {after_rescue}");

    let install = r#"
        skd_spk05_eh = {
            params ["_unit"];
            if (_unit isNotEqualTo player) exitWith {};
            private _g = createGroup west;
            private _u = _g createUnit [typeOf _unit, [0, 0, 0], [], 0, "CAN_COLLIDE"];
            _u setPosASL (getPosASL _unit);
            selectPlayer _u;
            _u addEventHandler ["Killed", skd_spk05_eh];
            skd_spk05_count = (missionNamespace getVariable ["skd_spk05_count", 0]) + 1;
        };
        player addEventHandler ["Killed", skd_spk05_eh];
        true"#;
    let _ = value(&session, install, 10.0).await;
    let mut handled = 0;
    for n in 1..=10 {
        let starts = session.mission_start("p1");
        let _ = value(&session, "player setDamage 1; true", 10.0).await;
        let t = Instant::now();
        let mut ok = false;
        let mut last = Value::Null;
        while t.elapsed() < Duration::from_secs(10) {
            last = value(&session, probe, 3.0).await;
            if last[0] == json!(true) {
                ok = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let same_mission = session.mission_start("p1") == starts;
        println!(
            "SPK-05 death {n} with the Killed handler: alive {ok} after {:.2} s, same mission {same_mission}; {last}",
            t.elapsed().as_secs_f64()
        );
        if ok && same_mission {
            handled += 1;
        }
    }

    // SPK-05, part 2: endMission "END1" loads the next entry of the -autotest list.
    let mut restarts = Vec::new();
    for n in 1..=10 {
        let starts = session.mission_start("p1");
        let t = Instant::now();
        let _ = session
            .exec("p1", r#"[] spawn { endMission "END1" }; true"#, json!([]), 5.0)
            .await;
        let mut took = None;
        while t.elapsed() < Duration::from_secs(20) {
            if session.mission_start("p1") > starts {
                took = Some(t.elapsed().as_secs_f64());
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let alive = value(&session, probe, 5.0).await;
        println!(
            "SPK-05 endMission {n}: mission start {starts} -> {} after {took:?} s; then {alive}",
            session.mission_start("p1")
        );
        restarts.push(took);
    }

    session.stop(&runner).await;
    let under3 = restarts.iter().filter(|t| t.is_some_and(|t| t < 3.0)).count();
    println!(
        "SPK-05 summary: engine respawn {engine_respawn:?}; Killed handler kept the mission and the player {handled}/10; endMission under 3 s {under3}/10"
    );
    // SPK-10 is a measurement either way (ACT-01 checks condition and radius itself); SPK-12
    // passes, and SPK-05 holds with its first fallback, the Killed handler.
    assert!(spk12_pass, "SPK-12: objects round-trip by net id");
    assert_eq!(
        handled, 10,
        "SPK-05: the Killed handler kept the mission and the player"
    );
    assert_eq!(under3, 10, "SPK-05: endMission under 3 s");
}

/// Not a spike of the spec: how CBA 3.19 keeps its keybinds, read in a running game, for
/// `skd_fnc_pressKey` (`ACT-02`, `ACT-03`), and whether a created vehicle's randomisation is off
/// when its variable is set before its init runs (`SCN-01`).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3 with CBA and arma-grappling-hook; run on the test machine"]
async fn int_cba_01_keybind_registry_probe() {
    let e2e = plain(repo().join("e2e"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup("sp").unwrap().clone();
    let mods = vec![
        config::Mod {
            name: "cba".into(),
            path: PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\workshop\content\107410\450814997"),
            patch: Some("cba_main".into()),
            source: None,
            prefix: None,
        },
        config::Mod {
            name: "agh".into(),
            path: PathBuf::from(r"C:\Users\Shadow\Documents\arma-grappling-hook\.hemttout\build"),
            patch: Some("agh_main".into()),
            source: None,
            prefix: None,
        },
    ];
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
    };
    let addon = plain(repo().join("target").join("shakedown").join("@shakedown"));
    let runner = Runner::bind(addon, None).await.unwrap();
    let session = session::start(&runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    for (label, source) in [
        (
            "getKeybind",
            r#"str (["Grappling hooks", "agh_letGo"] call CBA_fnc_getKeybind)"#,
        ),
        (
            "getKeybind lower",
            r#"str (["grappling hooks", "AGH_LETGO"] call CBA_fnc_getKeybind)"#,
        ),
        (
            "missing",
            r#"isNil {["Grappling hooks", "nope"] call CBA_fnc_getKeybind}"#,
        ),
        ("vars", r#"(allVariables missionNamespace) select { "keybind" in _x } "#),
        ("uivars", r#"(allVariables uiNamespace) select { "keybind" in _x } "#),
        (
            "actions type",
            r#"typeName (missionNamespace getVariable "cba_keybinding_actions")"#,
        ),
        (
            "actions",
            r#"private _a = missionNamespace getVariable "cba_keybinding_actions"; if (_a isEqualType createHashMap) then { keys _a } else { if (_a isEqualType locationNull || {_a isEqualType objNull}) then { allVariables _a } else { str _a } }"#,
        ),
        (
            "modPrettyNames",
            r#"str (missionNamespace getVariable "cba_keybinding_modPrettyNames")"#,
        ),
        (
            "offroad textures",
            r#"private _r = []; for "_i" from 1 to 3 do { private _v = createVehicle ["C_Offroad_01_F", [500, 500 + _i * 10, 0], [], 0, "CAN_COLLIDE"]; _r pushBack (getObjectTextures _v); deleteVehicle _v }; _r"#,
        ),
    ] {
        let v = value(&session, source, 10.0).await;
        println!("PROBE {label}: {v}");
    }
    session.stop(&runner).await;
}
