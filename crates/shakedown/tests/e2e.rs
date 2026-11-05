//! End-to-end tests that drive the installed `shakedown` command against the game. They run only
//! on the test machine, by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test e2e -- --ignored --nocapture --test-threads=1
//!
//! Each test writes its own project into a temporary folder, so it can change its checks.

use std::path::{Path, PathBuf};
use std::process::Command;

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

fn project(name: &str, checks: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-e2e-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let toml = std::fs::read_to_string(repo().join("e2e").join("shakedown.toml")).unwrap();
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

fn game_processes() -> String {
    let out = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq arma3_x64.exe", "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

const PASSING: &str = r#"// @check US-037 Nested steps, a value from p1 and evidence
// @tags US-037

private _n = ["ask p1", {
    ["p1", { (_this select 0) * 2 }, [21]] call skd_fnc_on
}] call skd_fnc_step;
["outer", {
    ["inner", {
        [_this select 0, "==", 42, "21 * 2 on p1"] call skd_fnc_expect;
    }, _this] call skd_fnc_step;
}, [_n]] call skd_fnc_step;
["answer", _n] call skd_fnc_evidence;
"#;

const FAILING: &str = r#"// @check US-037 Evidence is kept when a check fails
// @tags US-037
["kept", [1, "two"]] call skd_fnc_evidence;
["fails on purpose", {
    ["the check says so"] call skd_fnc_fail;
}] call skd_fnc_step;
"#;

const SYNTAX: &str = "// @check US-038 A body with a syntax error\n\nprivate _a = 1;\n_a = _a +* 2;\n";
const SYNTAX_FIXED: &str = "// @check US-038 A body with a syntax error\n\nprivate _a = 1;\n_a = _a * 2;\n";

#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn us_037_us_038_checks_in_one_session() {
    let dir = project(
        "us037",
        &[
            ("a/US-038-no-check-line", "// @tags US-038\n[] call skd_fnc_step;\n"),
            (
                "a/US-038-timeout-fast",
                "// @check US-038 A bad timeout\n// @timeout fast\n",
            ),
            (
                "a/US-038-key-twice",
                "// @check US-038 A key twice\n// @tags a\n// @tags b\n",
            ),
            ("b/US-038-syntax-error", SYNTAX),
            (
                "c/US-038-expect-in-sent-code",
                "// @check US-038 skd_fnc_expect inside code sent to a machine\n[\"p1\", { [1, \"==\", 1, \"one\"] call skd_fnc_expect }] call skd_fnc_on;\n",
            ),
            ("d/US-037-passes", PASSING),
            ("e/US-037-evidence-on-fail", FAILING),
        ],
    );

    // US-037 1: list shows the name starting with the story ID, with no game.
    let (code, listed) = cmd(&dir, &["list"]);
    assert_eq!(code, 0);
    let checks = listed["checks"].as_array().unwrap();
    let passing = checks.iter().find(|c| c["id"] == "d/US-037-passes").unwrap();
    assert!(passing["name"].as_str().unwrap().starts_with("US-037"));
    assert!(
        checks.iter().find(|c| c["id"] == "a/US-038-timeout-fast").unwrap()["error"]
            .as_str()
            .unwrap()
            .contains("@timeout")
    );

    let (code, session) = cmd(&dir, &["session", "start", "--setup", "sp"]);
    assert_eq!(code, 0, "{session}");
    assert_eq!(session["state"], "ready");
    let session_id = session["id"].as_str().unwrap().to_string();
    let pid = session["machines"][0]["pid"].as_u64().unwrap();

    let ids = [
        "a/US-038-no-check-line",
        "a/US-038-timeout-fast",
        "a/US-038-key-twice",
        "b/US-038-syntax-error",
        "c/US-038-expect-in-sent-code",
        "d/US-037-passes",
        "e/US-037-evidence-on-fail",
    ];
    let mut args = vec!["run"];
    args.extend(ids);
    let (code, run) = cmd(&dir, &args);
    assert_eq!(code, 1, "a check failed: {run}");
    let result = validate(Path::new(run["result_file"].as_str().unwrap()));
    let by_id = |id: &str| {
        result["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["check_id"] == id)
            .unwrap()
            .clone()
    };

    // US-038 1: header errors name the line and the key, without touching the game.
    for (id, needles) in [
        ("a/US-038-no-check-line", vec!["@check"]),
        ("a/US-038-timeout-fast", vec!["line 2", "@timeout", "fast"]),
        ("a/US-038-key-twice", vec!["line 3", "@tags"]),
    ] {
        let c = by_id(id);
        assert_eq!(
            (c["outcome"].as_str(), c["reason"]["kind"].as_str()),
            (Some("fail"), Some("check-error")),
            "{c}"
        );
        let message = c["reason"]["message"].as_str().unwrap();
        for n in needles {
            assert!(message.contains(n), "{id}: {message}");
        }
        assert!(c["steps"].as_array().unwrap().is_empty());
    }
    // US-038 2: the engine's compile message, and skd_fnc_* in sent code.
    let c = by_id("b/US-038-syntax-error");
    assert_eq!(
        (c["outcome"].as_str(), c["reason"]["kind"].as_str()),
        (Some("fail"), Some("check-error")),
        "{c}"
    );
    let message = c["reason"]["message"].as_str().unwrap();
    assert!(message.contains("line 4"), "{message}");
    let c = by_id("c/US-038-expect-in-sent-code");
    assert_eq!(
        (c["outcome"].as_str(), c["reason"]["kind"].as_str()),
        (Some("fail"), Some("check-error")),
        "{c}"
    );
    assert!(
        c["reason"]["message"].as_str().unwrap().contains("skd_fnc_expect"),
        "{c}"
    );
    // US-038 3 and US-037 2: the next check runs and passes, with every step and its clocks.
    let c = by_id("d/US-037-passes");
    assert_eq!(c["outcome"], "pass", "{c}");
    let paths: Vec<Value> = c["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["path"].clone())
        .collect();
    assert_eq!(
        paths,
        vec![
            serde_json::json!(["ask p1"]),
            serde_json::json!(["outer"]),
            serde_json::json!(["outer", "inner"])
        ]
    );
    for s in c["steps"].as_array().unwrap() {
        assert_eq!(
            (s["machine"].as_str(), s["state"].as_str()),
            (Some("p1"), Some("passed"))
        );
        for clock in ["utc", "tick", "time", "frame"] {
            assert!(!s["start"][clock].is_null() && !s["end"][clock].is_null(), "{s}");
        }
    }
    // US-037 3: evidence for a passing and a failing check.
    assert_eq!(c["evidence"]["attached"][0]["value"], 42);
    let c = by_id("e/US-037-evidence-on-fail");
    assert_eq!(
        (c["outcome"].as_str(), c["reason"]["kind"].as_str()),
        (Some("fail"), Some("explicit"))
    );
    assert_eq!(c["evidence"]["attached"][0]["value"], serde_json::json!([1, "two"]));
    assert_eq!(c["evidence"]["failed_step"], serde_json::json!(["fails on purpose"]));

    // US-038 4: fixed in the file, the same check passes with no reload, on the same machine start.
    std::fs::write(dir.join("checks/b/US-038-syntax-error.sqf"), SYNTAX_FIXED).unwrap();
    let (code, run) = cmd(&dir, &["run", "b/US-038-syntax-error"]);
    assert_eq!(code, 0, "{run}");
    let result = validate(Path::new(run["result_file"].as_str().unwrap()));
    assert_eq!(result["checks"][0]["outcome"], "pass");
    assert_eq!(result["session"]["id"], session_id.as_str());
    assert_eq!(result["session"]["machines"][0]["start"], 1);
    assert_eq!(result["session"]["machines"][0]["end_state"], "ready");

    let (code, stopped) = cmd(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    assert_eq!(stopped["state"], "stopped");
    assert!(
        !game_processes().contains(&format!("\"{pid}\"")),
        "the game process {pid} is gone"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "starts Arma 3; run on the test machine"]
fn run_04_the_demo_starts_and_stops_its_own_session() {
    let (code, run) = cmd(&repo().join("e2e"), &["run", "--setup", "sp", "smoke"]);
    assert_eq!(code, 0, "{run}");
    let result = validate(Path::new(run["result_file"].as_str().unwrap()));
    assert_eq!(result["checks"][0]["outcome"], "pass");
    assert!(
        game_processes().trim().is_empty() || !game_processes().contains("arma3_x64"),
        "no game process left"
    );
}
