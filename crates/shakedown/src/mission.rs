//! The session mission and the singleplayer `-autotest` list the runner generates (`INT-AR-04`).

use std::path::Path;

/// The mission's folder name: `shakedown_<session id>.<world>`.
pub fn folder_name(session_id: &str, world: &str) -> String {
    format!("shakedown_{session_id}.{world}")
}

/// `mission.sqm` with one playable rifleman per game client, one playable `HeadlessClient_F` per
/// headless client and a `respawn_west` marker at the spawn point. Positions in a mission file are
/// `{x, height, y}`.
pub fn mission_sqm(clients: u32, headless: u32, spawn: [f64; 2]) -> String {
    let mut items = String::new();
    let mut id = 0;
    for n in 0..clients {
        let x = spawn[0] + 10.0 * n as f64;
        items.push_str(&format!(
            r#"        class Item{n}
        {{
            dataType="Group";
            side="West";
            class Entities
            {{
                items=1;
                class Item0
                {{
                    dataType="Object";
                    class PositionInfo
                    {{
                        position[]={{{x},0,{y}}};
                    }};
                    side="West";
                    flags=7;
                    class Attributes
                    {{
                        isPlayer={is_player};
                        isPlayable=1;
                    }};
                    id={unit};
                    type="B_Soldier_F";
                }};
            }};
            class Attributes {{}};
            id={group};
        }};
"#,
            y = spawn[1],
            is_player = i32::from(n == 0),
            group = id,
            unit = id + 1,
        ));
        id += 2;
    }
    for h in 0..headless {
        items.push_str(&format!(
            r#"        class Item{n}
        {{
            dataType="Logic";
            class PositionInfo
            {{
                position[]={{{x},0,{y}}};
            }};
            name="skd_hc{number}";
            isPlayable=1;
            id={id};
            type="HeadlessClient_F";
        }};
"#,
            n = clients + h,
            x = spawn[0] - 10.0 * (h + 1) as f64,
            y = spawn[1] - 10.0,
            number = h + 1,
        ));
        id += 1;
    }
    items.push_str(&format!(
        r#"        class Item{n}
        {{
            dataType="Marker";
            position[]={{{x},0,{y}}};
            name="respawn_west";
            type="Empty";
            id={id};
        }};
"#,
        n = clients + headless,
        x = spawn[0],
        y = spawn[1],
    ));
    format!(
        r#"version=54;
binarizationWanted=0;
addons[]={{"A3_Characters_F","A3_Modules_F"}};
randomSeed=1;
class ScenarioData
{{
    author="shakedown";
}};
class Mission
{{
    class Intel
    {{
        timeOfChanges=1800.0002;
        startWeather=0;
        startWind=0.1;
        forecastWeather=0;
        forecastWind=0.1;
        year=2035;
        month=6;
        day=24;
        hour=12;
        minute=0;
    }};
    class Entities
    {{
        items={count};
{items}    }};
}};
"#,
        count = clients + headless + 1,
    )
}

/// `description.ext` of the session mission.
pub fn description_ext(max_players: u32) -> String {
    format!(
        r#"skipLobby = 1;
respawn = 3;
respawnDelay = 0;
respawnOnStart = -1;
disabledAI = 1;
allowFunctionsRecompile = 1;
briefing = 0;
debriefing = 0;
class Header
{{
    gameType = "Coop";
    minPlayers = 1;
    maxPlayers = {max_players};
}};
class CfgSettings
{{
    class CBA
    {{
        class Caching
        {{
            compile = 0;
            xeh = 0;
            functions = 0;
        }};
    }};
}};
"#
    )
}

/// `server.cfg` of a dedicated session (`INT-AR-05`).
pub fn server_cfg(session_id: &str, world: &str, password: &str, command_password: &str, max_players: u32) -> String {
    format!(
        r#"hostname = "shakedown {session_id}";
password = "{password}";
serverCommandPassword = "{command_password}";
loopback = 1;
kickDuplicate = 0;
BattlEye = 0;
verifySignatures = 0;
allowedFilePatching = 2;
persistent = 1;
kickTimeout[] = {{{{0, 0}}, {{1, 0}}, {{2, 0}}, {{3, 0}}}};
headlessClients[] = {{"127.0.0.1"}};
localClient[] = {{"127.0.0.1"}};
maxPlayers = {max_players};
class Missions
{{
    class session
    {{
        template = "{folder}";
        difficulty = "Regular";
    }};
}};
"#,
        folder = folder_name(session_id, world),
    )
}

/// `autotest.cfg`: the session mission listed 100 times, so ending it loads it again.
pub fn autotest_cfg(mission: &Path) -> String {
    let mut out = String::from("class TestMissions\n{\n");
    for n in 1..=100 {
        out.push_str(&format!(
            "    class m{n:03}\n    {{\n        campaign = \"\";\n        mission = \"{}\";\n    }};\n",
            mission.display()
        ));
    }
    out.push_str("};\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_ar_04_one_player_and_the_respawn_marker() {
        let sqm = mission_sqm(1, 0, [100.0, 100.0]);
        assert_eq!(sqm.matches("B_Soldier_F").count(), 1);
        assert!(sqm.contains("respawn_west"));
        assert!(sqm.contains("position[]={100,0,100};"));
        assert!(sqm.contains("items=2;"));
    }

    #[test]
    fn int_ar_04_two_clients_and_a_headless_client_are_playable() {
        let sqm = mission_sqm(2, 1, [100.0, 100.0]);
        assert_eq!(sqm.matches("B_Soldier_F").count(), 2);
        assert!(sqm.contains("position[]={110,0,100};"));
        assert_eq!(sqm.matches("HeadlessClient_F").count(), 1);
        assert!(sqm.contains("items=4;"));
    }

    #[test]
    fn int_ar_05_server_cfg_names_the_mission_and_the_one_machine_rules() {
        let cfg = server_cfg("s-1", "VR", "pw", "cpw", 3);
        for line in [
            "password = \"pw\";",
            "serverCommandPassword = \"cpw\";",
            "loopback = 1;",
            "kickDuplicate = 0;",
            "BattlEye = 0;",
            "persistent = 1;",
            "kickTimeout[] = {{0, 0}, {1, 0}, {2, 0}, {3, 0}};",
            "maxPlayers = 3;",
            "template = \"shakedown_s-1.VR\";",
        ] {
            assert!(
                cfg.contains(line),
                "{line}
{cfg}"
            );
        }
    }

    #[test]
    fn int_ar_04_description_turns_cba_caching_off() {
        let ext = description_ext(1);
        for line in [
            "skipLobby = 1;",
            "respawn = 3;",
            "allowFunctionsRecompile = 1;",
            "compile = 0;",
            "maxPlayers = 1;",
        ] {
            assert!(ext.contains(line), "{line}");
        }
    }

    #[test]
    fn int_ar_04_autotest_lists_the_mission_100_times() {
        let cfg = autotest_cfg(Path::new(r"C:\s\missions\shakedown_s-1.VR"));
        assert_eq!(cfg.matches("mission = ").count(), 100);
        assert!(cfg.contains("class m100"));
    }
}
