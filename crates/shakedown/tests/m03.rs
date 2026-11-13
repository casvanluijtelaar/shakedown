//! M3's end-to-end tests: script errors on every machine traced to their step, expected errors,
//! the RPT scan, each machine's log kept with the run and per check, the `logs` command, the
//! redaction of the Steam identity, `DEF-09`, and the demo. The stories `US-024`, `US-027`,
//! `US-028`, `US-029`, `US-030`, `US-064` and `US-065`. They drive the installed `shakedown`
//! command against the game, so they run only on the test machine, by hand, one at a time:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m03 -- --ignored --nocapture --test-threads=1
//!
//! The Steam ID and the Steam names are read from this machine's Steam files at run time and
//! never printed: a test says only whether it found them.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::Value;

fn repo() -> PathBuf {
    let p = std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

fn shakedown() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

/// Runs the installed command in `project`; returns the exit code and stdout, which with
/// `--json` is one JSON document.
fn raw(project: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(shakedown())
        .args(args)
        .current_dir(project)
        .output()
        .unwrap();
    eprintln!("$ shakedown {} -> {:?}", args.join(" "), out.status.code());
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
    )
}

fn cmd(project: &Path, args: &[&str]) -> (i32, Value) {
    let mut all = args.to_vec();
    all.push("--json");
    let (code, out) = raw(project, &all);
    (code, serde_json::from_str(&out).unwrap_or(Value::Null))
}

/// A project in a temporary folder with the setup `ded` (the fixture, two clients and a headless
/// client) and the given checks.
fn project(name: &str, checks: &[(String, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m03-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[setups.ded]
mode = "dedicated"
clients = 2
headless = 1
mods = ["fixture"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
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

/// A started session, stopped when dropped, also when the test panics; any process left after
/// the stop is ended by its id.
struct Session {
    dir: PathBuf,
    view: Value,
    stopped: std::cell::Cell<bool>,
}

impl Session {
    fn start(dir: &Path, setup: &str) -> Session {
        let t = Instant::now();
        let (code, s) = cmd(dir, &["session", "start", "--setup", setup]);
        println!(
            "session start --setup {setup}: exit {code} in {:.1} s",
            t.elapsed().as_secs_f64()
        );
        assert_eq!(code, 0, "{}", s["reason"]["message"]);
        Session {
            dir: dir.to_path_buf(),
            view: s,
            stopped: std::cell::Cell::new(false),
        }
    }

    fn pids(&self) -> Vec<u64> {
        self.view["machines"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["pid"].as_u64())
            .collect()
    }

    /// Runs checks by id; returns the exit code, the validated result and the run folder.
    fn run(&self, ids: &[&str]) -> (i32, Value, PathBuf) {
        let mut args = vec!["run"];
        args.extend(ids);
        let (code, run) = cmd(&self.dir, &args);
        let file = PathBuf::from(
            run["result_file"]
                .as_str()
                .unwrap_or_else(|| panic!("no result: {run}")),
        );
        let result = validate(&file);
        (code, result, file.parent().unwrap().to_path_buf())
    }

    fn stop(&self) {
        if self.stopped.replace(true) {
            return;
        }
        let (code, _) = cmd(&self.dir, &["session", "stop"]);
        if code != 0 {
            for pid in self.pids() {
                let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
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

fn checks_of(list: &[(&str, &str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(id, name, body)| (id.to_string(), format!("// @check {name}\n{body}\n")))
        .collect()
}

/// The Steam ID and the two names of the account Steam last logged in with, from this machine's
/// `loginusers.vdf`, read at run time.
fn identity() -> Vec<String> {
    let out = Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let steam = text
        .lines()
        .find(|l| l.contains("SteamPath"))
        .and_then(|l| l.split("REG_SZ").nth(1))
        .unwrap()
        .trim()
        .to_string();
    let vdf = std::fs::read_to_string(Path::new(&steam).join("config").join("loginusers.vdf")).unwrap();
    // Each account: [steamID64, AccountName, PersonaName, MostRecent, timestamp].
    let mut accounts: Vec<[String; 5]> = Vec::new();
    for line in vdf.lines() {
        let parts: Vec<&str> = line.trim().split('"').collect();
        if parts.len() == 3 && parts[1].len() == 17 {
            accounts.push([
                parts[1].to_string(),
                String::new(),
                String::new(),
                String::new(),
                "0".into(),
            ]);
        } else if parts.len() >= 5
            && let Some(a) = accounts.last_mut()
        {
            match parts[1] {
                "AccountName" => a[1] = parts[3].to_string(),
                "PersonaName" => a[2] = parts[3].to_string(),
                "MostRecent" => a[3] = parts[3].to_string(),
                "timestamp" => a[4] = parts[3].to_string(),
                _ => {}
            }
        }
    }
    let found: Vec<String> = accounts
        .iter()
        .find(|a| a[3] == "1")
        .or_else(|| accounts.iter().max_by_key(|a| a[4].parse::<u64>().unwrap_or(0)))
        .map(|a| a[..3].to_vec())
        .unwrap_or_default();
    assert_eq!(found.len(), 3, "no most recent account in loginusers.vdf");
    assert!(found.iter().all(|v| !v.is_empty()));
    found
}

/// Whether `text` holds `word` as a whole word, case-sensitively, as `LOG-02` matches it.
fn holds(text: &str, word: &str) -> bool {
    let w = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(w) && !after.is_some_and(w)
    })
}

/// Every file under `dir` whose text holds any of `values`, by path; the values are not named.
fn leaks(dir: &Path, values: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "json" || x == "log" || x == "rpt") {
                let text = String::from_utf8_lossy(&std::fs::read(&p).unwrap()).to_string();
                for (i, v) in values.iter().enumerate() {
                    if holds(&text, v) {
                        out.push(format!("{} holds identity value {i}", p.display()));
                    }
                }
            }
        }
    }
    out
}

const WAIT: &str = r#"["wait for the flag", { ["p1", { !isNil "skdfx_flag" }, "flag reached p1", 3] call skd_fnc_waitUntil }] call skd_fnc_step;"#;
const EACH: &str = r#"["read on every machine", { ["all", { 1 }, "==", 2, "every machine reads 2", 2] call skd_fnc_expectEach }] call skd_fnc_step;"#;
const DEFAULT_CEILING: &str = r#"["wait without a ceiling", { ["p1", { false }, "a condition that never holds"] call skd_fnc_waitUntil }] call skd_fnc_step;"#;
const EXPECTED: &str = r#"["trigger on p2", { ["_skdfx_undefined", { ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on }] call skd_fnc_expectError }] call skd_fnc_step;"#;
const EXPECTED_AND_OTHER: &str = r#"["trigger on p2 and p1", {
    ["_SKDFX_UNDEFINED", {
        ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on;
        ["p1", { private _a = "skd" + 1; true }] call skd_fnc_on;
    }] call skd_fnc_expectError;
}] call skd_fnc_step;"#;
const ON_P2_CALL: &str =
    r#"["trigger on p2", { ["p2", { [] call skdfx_fnc_throw; true }] call skd_fnc_on }] call skd_fnc_step;"#;
const ON_P2_SPAWN: &str =
    r#"["trigger on p2", { ["p2", { [] spawn skdfx_fnc_throw; true }] call skd_fnc_on }] call skd_fnc_step;"#;
const ON_P2_REMOTE: &str = r#"["trigger on p2", {
    ["server", { private _p2 = (allPlayers select { name _x == "p2" }) select 0; [] remoteExec ["skdfx_fnc_throw", owner _p2]; true }] call skd_fnc_on;
    ["server", { false }, "a moment for the error to come", 2] call skd_fnc_tryWaitUntil;
}] call skd_fnc_step;"#;
const MISSING: &str = r#"["expect it", { ["_skdfx_undefined", { 1 }] call skd_fnc_expectError }] call skd_fnc_step;"#;
const RPT_ONLY: &str = r#"["write an error line on p1", { ["p1", { diag_log text "Error in expression <skd_us030 + 1>"; true }] call skd_fnc_on }] call skd_fnc_step;"#;
const NAMES: &str = r#"["every client is its machine name", {
    ["p1", { name player }, "==", "p1", "p1's name"] call skd_fnc_expectOn;
    ["p2", { name player }, "==", "p2", "p2's name"] call skd_fnc_expectOn;
}] call skd_fnc_step;"#;
const IDENTITY: &str = r#"private _id = ["p1", { [getPlayerUID player, profileNameSteam] }] call skd_fnc_on;
["write them to p1's log", { ["p1", { diag_log format ["uid %1 name %2", getPlayerUID player, profileNameSteam]; true }] call skd_fnc_on }] call skd_fnc_step;
["uid", _id select 0] call skd_fnc_evidence;
["name", _id select 1] call skd_fnc_evidence;
[format ["uid %1 name %2", _id select 0, _id select 1]] call skd_fnc_fail;"#;

/// `US-024`, `US-027`, `US-028`, `US-029`, `US-030`, `US-064` and `US-065` in one dedicated
/// session with two clients and a headless client.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_024_us_027_us_028_us_029_us_030_us_064_us_065_on_a_dedicated_session() {
    let ids = identity();
    let list = [
        (
            "US-024-wait",
            "US-024 a wait that never holds ends at its ceiling",
            WAIT,
        ),
        (
            "US-024-each",
            "US-024 an assertion on every machine keeps each machine's last value",
            EACH,
        ),
        (
            "US-024-default",
            "US-024 a wait without a ceiling ends at 10 s",
            DEFAULT_CEILING,
        ),
        ("US-027-expected", "US-027 an expected error on p2 passes", EXPECTED),
        (
            "US-027-other",
            "US-027 another error in an expectError block fails",
            EXPECTED_AND_OTHER,
        ),
        (
            "US-028-call",
            "US-028 an error on p2 fails naming p2 and its step",
            ON_P2_CALL,
        ),
        ("US-028-spawn", "US-028 an error in code p2 spawned", ON_P2_SPAWN),
        (
            "US-028-remote",
            "US-028 an error p2 gets through remoteExec",
            ON_P2_REMOTE,
        ),
        ("US-029-missing", "US-029 an expected error that never comes", MISSING),
        ("US-030-rpt", "US-030 an error line the handler never saw", RPT_ONLY),
        ("US-064-names", "US-064 every client runs as its machine name", NAMES),
        (
            "US-065-identity",
            "US-065 a check that writes out the Steam identity",
            IDENTITY,
        ),
    ];
    let dir = project("stories", &checks_of(&list));
    let session = Session::start(&dir, "ded");
    let session_folder = PathBuf::from(session.view["folder"].as_str().unwrap());

    // US-064 2: every process runs under its machine name.
    for m in session.view["machines"].as_array().unwrap() {
        let pid = m["pid"].as_u64().unwrap();
        let out = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine"),
            ])
            .output()
            .unwrap();
        let line = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(line.contains(&format!("-name={}", m["name"].as_str().unwrap())));
    }

    let names: Vec<&str> = list.iter().map(|(id, _, _)| *id).collect();
    let (code, result, run) = session.run(&names);
    for c in result["checks"].as_array().unwrap() {
        println!("{} {} {}", c["check_id"], c["outcome"], c["reason"]["message"]);
    }
    assert_eq!(code, 1);

    // US-024
    assert_outcome(
        &by_id(&result, "US-024-wait"),
        "fail",
        Some("wait-ceiling"),
        &["flag reached p1", "3 s"],
    );
    let each = by_id(&result, "US-024-each");
    assert_outcome(&each, "fail", Some("assertion"), &["every machine reads 2"]);
    for m in ["server", "p1", "p2", "hc1"] {
        assert_eq!(each["evidence"]["observed"][m], 1, "{each:#}");
        let log = each["evidence"]["logs"][m].as_str().unwrap();
        assert!(run.join(log).is_file(), "{log}");
    }
    assert_eq!(
        each["evidence"]["failed_step"],
        serde_json::json!(["read on every machine"])
    );
    assert_outcome(
        &by_id(&result, "US-024-default"),
        "fail",
        Some("wait-ceiling"),
        &["10 s"],
    );

    // US-027
    let c = by_id(&result, "US-027-expected");
    assert_outcome(&c, "pass", None, &[]);
    let e = &c["errors"][0];
    assert_eq!(
        (e["machine"].as_str(), e["expected"].as_bool()),
        (Some("p2"), Some(true)),
        "{c:#}"
    );
    let c = by_id(&result, "US-027-other");
    assert_outcome(&c, "fail", Some("script-error"), &["p1"]);
    assert!(
        c["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["machine"] == "p2" && e["expected"] == true)
    );

    // US-028
    for id in ["US-028-call", "US-028-spawn", "US-028-remote"] {
        let c = by_id(&result, id);
        assert_outcome(&c, "fail", Some("script-error"), &["_skdfx_undefined", "fn_throw.sqf"]);
        assert_eq!(c["reason"]["machine"], "p2");
        let e = c["errors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["machine"] == "p2")
            .unwrap()
            .clone();
        assert_eq!(e["source"], "handler", "{e:#}");
        assert_eq!(e["step_path"], serde_json::json!(["trigger on p2"]), "{id}: {e:#}");
        assert!(e["file"].as_str().unwrap().ends_with("fn_throw.sqf"));
        assert_eq!(e["line"], 4);
        assert!(!e["stack"].as_array().unwrap().is_empty());
        assert!(e["at"]["utc"].is_string() && e["at"]["frame"].as_u64().unwrap() > 0);
    }

    // US-029
    let c = by_id(&result, "US-029-missing");
    assert_outcome(&c, "fail", Some("expected-error-missing"), &["_skdfx_undefined"]);
    assert_eq!(c["evidence"]["failed_step"], serde_json::json!(["expect it"]));

    // US-030
    let c = by_id(&result, "US-030-rpt");
    assert_outcome(&c, "fail", Some("script-error"), &["p1"]);
    let e = c["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["machine"] == "p1" && e["source"] == "rpt")
        .unwrap_or_else(|| panic!("{c:#}"))
        .clone();
    assert_eq!(
        e["step_path"],
        serde_json::json!(["write an error line on p1"]),
        "{e:#}"
    );
    // US-030 2 and NFR-RL-05: every error line of every check's excerpt is in its errors.
    for c in result["checks"].as_array().unwrap() {
        for (m, log) in c["evidence"]["logs"].as_object().unwrap() {
            let text = std::fs::read_to_string(run.join(log.as_str().unwrap())).unwrap();
            let lines = text.lines().filter(|l| l.contains("Error in expression")).count();
            let listed = c["errors"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["machine"] == m.as_str())
                .count();
            assert!(
                listed >= lines,
                "{} {m}: {lines} error lines, {listed} listed",
                c["check_id"]
            );
        }
    }

    // US-064 2: name player on each client.
    assert_outcome(&by_id(&result, "US-064-names"), "pass", None, &[]);

    // US-065 1 and 2
    let c = by_id(&result, "US-065-identity");
    assert_outcome(&c, "fail", Some("explicit"), &["uid <steamid> name <steamname>"]);
    assert_eq!(c["evidence"]["attached"][0]["value"], "<steamid>");
    assert_eq!(c["evidence"]["attached"][1]["value"], "<steamname>");
    let p1_log = std::fs::read_to_string(run.join(c["evidence"]["logs"]["p1"].as_str().unwrap())).unwrap();
    assert!(p1_log.contains("uid <steamid> name <steamname>"));
    let (_, out) = raw(
        &dir,
        &["logs", "--machine", "p1", "--check", "US-065-identity", "--last", "50"],
    );
    assert!(out.contains("uid <steamid> name <steamname>"));
    let (_, out) = raw(&dir, &["logs", "--machine", "p1", "--last", "50"]);
    println!(
        "logs --machine p1 --last 50 holds the placeholders: {}",
        out.contains("uid <steamid> name <steamname>")
    );

    // LOG-03: --grep, --run and --json.
    let (code, v) = cmd(
        &dir,
        &[
            "logs",
            "--machine",
            "server",
            "--run",
            run.file_name().unwrap().to_str().unwrap(),
            "--grep",
            r"\[shakedown\] r-",
            "--last",
            "3",
        ],
    );
    assert_eq!(code, 0);
    assert_eq!(v["lines"].as_array().unwrap().len(), 3, "{v}");
    assert_eq!(v["truncated"], true);

    // US-064 1 and 3
    for m in ["server", "p1"] {
        let (code, out) = raw(&dir, &["logs", "--machine", m, "--last", "500"]);
        assert_eq!(code, 0);
        assert!(
            ids.iter().all(|v| !holds(&out, v)),
            "logs --machine {m} names the account"
        );
    }
    session.stop();
    let mut found = leaks(&run, &ids);
    found.extend(leaks(&session_folder, &ids));
    assert!(found.is_empty(), "{found:#?}");
    println!("US-064: no Steam ID or Steam name in the run folder or the session's RPTs");
}

/// A check that creates objects on every kind of machine and puts p1 in a vehicle the server made.
const DIRTY: &str = r#"["create on every machine", {
    ["B_Soldier_F", AGLToASL [130, 100, 0], 0] call skd_fnc_create;
    ["C_Quadbike_01_F", AGLToASL [130, 110, 0], 90] call skd_fnc_create;
    ["Box_NATO_Ammo_F", AGLToASL [130, 120, 0], 0] call skd_fnc_create;
    private _mrap = ["B_MRAP_01_F", AGLToASL [110, 130, 0], 0] call skd_fnc_create;
    ["p1", { "Land_Cargo20_military_green_F" createVehicle [140, 140, 0]; true }] call skd_fnc_on;
    ["p2", { "Box_NATO_Ammo_F" createVehicle [145, 140, 0]; true }] call skd_fnc_on;
    ["hc1", { private _g = createGroup east; _g createUnit ["O_Soldier_F", [150, 150, 0], [], 0, "NONE"]; true }] call skd_fnc_on;
    ["p1", { player moveInDriver (_this select 0); true }, [_mrap]] call skd_fnc_on;
    ["p1", { vehicle player isNotEqualTo player }, "p1 sits in the MRAP"] call skd_fnc_waitUntil;
}] call skd_fnc_step;"#;

/// `DEF-09`: twenty resets after checks that create objects on each machine leave no `Object ...
/// not found` line in the server's RPT. The reset that had every machine delete every entity it
/// recorded left 125 to 253 such lines over 21 resets of the same kind.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn def_09_twenty_resets_leave_no_object_not_found_line_on_the_server() {
    let mut list: Vec<(String, String)> = (1..=20)
        .map(|n| {
            (
                format!("DEF-09-dirty-{n:02}"),
                format!("// @check DEF-09 dirty check {n}\n{DIRTY}\n"),
            )
        })
        .collect();
    list.push((
        "DEF-09-clean".into(),
        "// @check DEF-09 a check after the last reset\ntrue\n".into(),
    ));
    let dir = project("def09", &list);
    let session = Session::start(&dir, "ded");
    let ids: Vec<&str> = list.iter().map(|(id, _)| id.as_str()).collect();
    let (code, result, run) = session.run(&ids);
    let lines = |c: &Value| -> usize {
        let text = std::fs::read_to_string(run.join(c["evidence"]["logs"]["server"].as_str().unwrap())).unwrap();
        text.lines().filter(|l| l.contains("not found (message")).count()
    };
    let mut total = 0;
    for c in result["checks"].as_array().unwrap() {
        let n = lines(c);
        println!(
            "{} {} {}: {n} 'not found' lines on the server",
            c["check_id"], c["outcome"], c["reason"]["message"]
        );
        assert_eq!(c["outcome"], "pass", "{c:#}");
        total += n;
    }
    assert_eq!(total, 0, "the resets left 'not found' lines on the server");
    assert_eq!(code, 0);
}

/// The M3 demo: the M2 rope check with a last assertion arma-grappling-hook cannot meet fails in
/// that step with each machine's last value and log excerpt, and the run folder names no Steam
/// account.
#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn m3_demo_us_024_the_rope_check_fails_with_its_evidence() {
    let ids = identity();
    let demo = repo().join("e2e").join("grappling-hook");
    let session = Session::start(&demo, "dedicated");
    let (code, result, run) = session.run(&["US-024-dedicated-rope-never-held-by-p2"]);
    let c = by_id(&result, "US-024-dedicated-rope-never-held-by-p2");
    println!("{:#}", c["reason"]);
    println!("observed {} errors {}", c["evidence"]["observed"], c["errors"]);
    assert_eq!(code, 1);
    assert_outcome(&c, "fail", Some("assertion"), &["the rope's holder is p2"]);
    assert_eq!(
        c["evidence"]["failed_step"],
        serde_json::json!(["the server and both clients see p2 holding the rope"])
    );
    for m in ["server", "p1", "p2"] {
        assert!(c["evidence"]["observed"][m].is_string(), "{m}");
        assert!(run.join(c["evidence"]["logs"][m].as_str().unwrap()).is_file());
    }
    session.stop();
    let found = leaks(&run, &ids);
    assert!(found.is_empty(), "{found:#?}");
}
