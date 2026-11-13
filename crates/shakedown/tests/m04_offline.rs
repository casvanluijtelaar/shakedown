//! M4's tests that need no game and no runner: the MCP server and the command line on a machine
//! where nothing runs (`AGT-05`, `NFR-PT-02`), and the mistakes refused before any runner starts.
//! They run on Windows and Linux, in CI. Each gets its own runner folder, so a runner the
//! machine runs is never touched.

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{Mcp, run, run_json};

fn exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_shakedown"))
}

/// A project folder and a runner folder of its own, and the environment that points there.
struct Place {
    dir: PathBuf,
    home: PathBuf,
}

impl Place {
    fn new(name: &str, toml: &str, checks: &[(&str, &str)]) -> Place {
        let root = std::env::temp_dir().join(format!("skd-m04-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("project");
        let home = root.join("home");
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
        for (id, text) in checks {
            std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
        }
        Place { dir, home }
    }

    fn env(&self) -> Vec<(&'static str, String)> {
        let h = self.home.display().to_string();
        vec![("LOCALAPPDATA", h.clone()), ("HOME", h)]
    }

    fn with<T>(&self, f: impl FnOnce(&Path, &[(&str, &str)]) -> T) -> T {
        let env = self.env();
        let refs: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
        f(&self.dir, &refs)
    }

    /// No runner was started: it would have written its file here.
    fn no_runner(&self) {
        let file = self.home.join("Shakedown").join("runner.json");
        let file2 = self
            .home
            .join(".local")
            .join("share")
            .join("shakedown")
            .join("runner.json");
        assert!(!file.exists() && !file2.exists(), "a runner was started");
    }
}

impl Drop for Place {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

const GOOD: &str = r#"game = 'C:\Arma 3'

[mods.fixture]
path = 'C:\Arma 3\@skd_fixture'

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]
"#;

fn checks() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "a",
            "// @check US-031 first\n// @tags m01-3 smoke\n[] call skd_fnc_machines;\n",
        ),
        (
            "b",
            "// @check US-031 second\n// @tags smoke\n[] call skd_fnc_machines;\n",
        ),
        (
            "c",
            "// @check US-031 third\n// @tags m01-3\n[] call skd_fnc_machines;\n",
        ),
    ]
}

fn mcp_error(answer: &support::Answer, code: &str) -> String {
    assert!(answer.is_error(), "not an error: {}", answer.result);
    let text = answer.text();
    assert!(text.starts_with(&format!("{code}: ")), "`{text}` is not {code}");
    // `<code>: <what is wrong>. <what to do next>.` (AGT-02)
    assert!(text.ends_with('.') && text.matches(". ").count() >= 1, "{text}");
    text
}

#[test]
fn us_046_agt_05_nfr_pt_02_mcp_answers_status_list_checks_and_guide_with_no_runner() {
    let place = Place::new("status", GOOD, &checks());
    place.with(|dir, env| {
        let mut mcp = Mcp::start(&exe(), dir, env);
        let status = mcp.call("status", json!({})).data();
        assert_eq!(status["runner"], "not-running");
        assert_eq!(status["session"], Value::Null);
        let listed = mcp.call("list_checks", json!({})).data();
        assert_eq!(listed["checks"].as_array().unwrap().len(), 3);
        let guide = mcp.call("guide", json!({})).data();
        assert!(guide["text"].as_str().unwrap().chars().count() < 8000);
        // The same JSON as the command line (NFR-US-03).
        let (code, cli, _) = run_json(&exe(), dir, env, &["status"]);
        assert_eq!(code, 0);
        assert_eq!(cli, status);
        let (_, cli, _) = run_json(&exe(), dir, env, &["list"]);
        assert_eq!(cli, listed);
        let (_, cli, _) = run_json(&exe(), dir, env, &["guide"]);
        assert_eq!(cli, guide);
    });
    place.no_runner();
}

#[test]
fn us_046_status_with_no_runner_exits_0_saying_so_and_starts_nothing() {
    let place = Place::new("norunner", GOOD, &checks());
    place.with(|dir, env| {
        let (code, out, _) = run(&exe(), dir, env, &["status"]);
        assert_eq!(code, 0);
        assert!(out.contains("not-running"), "{out}");
    });
    place.no_runner();
}

#[test]
fn us_048_agt_02_every_tool_of_the_contract_is_listed_with_an_output_schema() {
    let place = Place::new("tools", GOOD, &checks());
    place.with(|dir, env| {
        let mut mcp = Mcp::start(&exe(), dir, env);
        let tools = mcp.tools();
        let names: Vec<&str> = tools
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        for want in [
            "status",
            "session_start",
            "session_stop",
            "list_checks",
            "run_start",
            "run_status",
            "run_result",
            "reset",
            "exec",
            "wait_until",
            "expect",
            "inspect",
            "logs",
            "errors",
            "guide",
        ] {
            assert!(names.contains(&want), "{want} is not listed");
        }
        for t in tools.as_array().unwrap() {
            assert_eq!(t["outputSchema"]["type"], "object", "{}", t["name"]);
            assert_eq!(t["inputSchema"]["type"], "object", "{}", t["name"]);
        }
        let read_only: Vec<&str> = tools
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["annotations"]["readOnlyHint"] == true)
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        for want in [
            "status",
            "list_checks",
            "run_status",
            "run_result",
            "inspect",
            "logs",
            "errors",
            "guide",
        ] {
            assert!(read_only.contains(&want), "{want} is not read-only");
        }
    });
}

#[test]
fn us_006_a_malformed_project_file_is_refused_with_the_key_and_its_line() {
    for (name, toml, needle) in [
        ("unknown", format!("{GOOD}colour = 1\n"), "colour"),
        ("nogame", GOOD.replace("game = 'C:\\Arma 3'\n", ""), "game"),
        (
            "clients",
            GOOD.replace("mode = \"singleplayer\"", "mode = \"dedicated\"\nclients = \"two\""),
            "clients",
        ),
    ] {
        let place = Place::new(&format!("cfg-{name}"), &toml, &checks());
        place.with(|dir, env| {
            for args in [&["session", "start", "--setup", "sp"][..], &["run"], &["list"]] {
                let (code, _, err) = run(&exe(), dir, env, args);
                assert_eq!(code, 3, "{args:?}: {err}");
                assert!(err.contains("line") && err.contains(needle), "{args:?}: {err}");
            }
            let mut mcp = Mcp::start(&exe(), dir, env);
            for (tool, args) in [
                ("session_start", json!({"setup": "sp"})),
                ("run_start", json!({})),
                ("list_checks", json!({})),
            ] {
                let text = mcp_error(&mcp.call(tool, args), "config-error");
                assert!(text.contains(needle), "{tool}: {text}");
            }
        });
        place.no_runner();
    }
}

#[test]
fn us_006_an_unknown_setup_is_refused_naming_the_setups_declared() {
    let place = Place::new("nosuch", GOOD, &checks());
    place.with(|dir, env| {
        let (code, _, err) = run(&exe(), dir, env, &["session", "start", "--setup", "nosuch"]);
        assert_eq!(code, 3);
        assert!(err.contains("nosuch") && err.contains("sp"), "{err}");
        let mut mcp = Mcp::start(&exe(), dir, env);
        let text = mcp_error(&mcp.call("session_start", json!({"setup": "nosuch"})), "unknown-setup");
        assert!(text.contains("sp"), "{text}");
    });
    place.no_runner();
}

#[test]
fn us_032_a_selection_that_matches_nothing_is_refused_listing_what_exists() {
    let place = Place::new("nomatch", GOOD, &checks());
    place.with(|dir, env| {
        let (code, _, err) = run(&exe(), dir, env, &["run", "--tag", "US-999"]);
        assert_eq!(code, 3);
        assert!(
            err.contains("US-999") && err.contains("m01-3") && err.contains("smoke"),
            "{err}"
        );
        let mut mcp = Mcp::start(&exe(), dir, env);
        let text = mcp_error(&mcp.call("run_start", json!({"tags": ["US-999"]})), "unknown-check");
        assert!(text.contains("m01-3"), "{text}");
        mcp_error(&mcp.call("list_checks", json!({"tags": ["US-999"]})), "unknown-check");
        mcp_error(&mcp.call("run_start", json!({"ids": ["nosuch"]})), "unknown-check");
        assert!(
            !dir.join(".shakedown").join("runs").exists(),
            "a run folder was created"
        );
    });
    place.no_runner();
}

#[test]
fn us_031_list_checks_and_list_select_the_same_checks_by_tag() {
    let place = Place::new("tags", GOOD, &checks());
    place.with(|dir, env| {
        let (code, cli, _) = run_json(&exe(), dir, env, &["list", "--tag", "m01-3"]);
        assert_eq!(code, 0);
        let mut mcp = Mcp::start(&exe(), dir, env);
        let listed = mcp.call("list_checks", json!({"tags": ["m01-3"]})).data();
        assert_eq!(cli, listed);
        let ids: Vec<&str> = listed["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["a", "c"]);
    });
}

#[test]
fn us_049_agt_03_a_ceiling_above_50_and_an_unknown_run_get_a_code_and_the_next_step() {
    let place = Place::new("fixable", GOOD, &checks());
    place.with(|dir, env| {
        let mut mcp = Mcp::start(&exe(), dir, env);
        let text = mcp_error(
            &mcp.call("exec", json!({"machine": "p1", "code": "1", "ceiling": 55})),
            "bad-arguments",
        );
        assert!(text.contains("at most 50"), "{text}");
        mcp_error(
            &mcp.call(
                "wait_until",
                json!({"machine": "p1", "condition": "true", "ceiling": 51}),
            ),
            "bad-arguments",
        );
        mcp_error(
            &mcp.call("run_status", json!({"run_id": "r-1", "wait_s": 60})),
            "bad-arguments",
        );
        mcp_error(&mcp.call("exec", json!({"machine": "p1"})), "bad-arguments");
        mcp_error(&mcp.call("run_result", json!({"run_id": "r-nosuch"})), "unknown-run");
    });
    place.no_runner();
}
