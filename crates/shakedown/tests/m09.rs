//! M9's tests: every check ends pass, fail or invalid with its reason (`US-041`), a reset that
//! cannot complete (`US-010`), a machine that stops answering (`US-015`, `US-045`), a process
//! that ends or crashes during a check, with its log and dumps kept (`US-016`, `US-043`), a game
//! clock that stops (`US-059`), and a broken session that still answers (`US-044`). They drive the
//! installed `shakedown`, the fixture's failures on demand and the game, so they run only on the
//! test machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m09 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use support::repo;

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

const PASS: &str = r#"["one is one", { [1, "==", 1, "one"] call skd_fnc_expect }] call skd_fnc_step;"#;

/// A project with the setups `sp` and `ded2` (two clients), both loading `@skd_fixture`, and the
/// checks of M9.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m09-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    let toml = format!(
        r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[setups.sp]
mode = "singleplayer"
mods = ["fixture"]

[setups.ded2]
mode = "dedicated"
clients = 2
mods = ["fixture"]
"#
    );
    std::fs::write(dir.join("shakedown.toml"), toml).unwrap();
    let checks = [
        ("pass", format!("// @check US-041 a passing check\n{PASS}\n")),
        (
            "failing",
            "// @check US-041 a check that fails on purpose\n[\"broken on purpose\"] call skd_fnc_fail;\n".into(),
        ),
        (
            "explicit",
            "// @check US-041 a check that is invalid on purpose\n[\"no network\"] call skd_fnc_invalid;\n".into(),
        ),
        (
            "stuck",
            "// @check US-010 a cleanup that never ends\n[{ waitUntil { false } }, []] call skd_fnc_cleanup;\n".into(),
        ),
        (
            "waitp2",
            "// @check US-015 a wait on p2 that never holds\n// @timeout 90\n[\"p2\", { false }, \"a condition that never holds\", 60] call skd_fnc_waitUntil;\n"
                .into(),
        ),
        (
            "crashp2",
            "// @check US-016 p2 crashes, then the check waits on it\n// @timeout 60\n[\"p2\", { [0] call skdfx_fnc_crash }] call skd_fnc_on;\n[\"p2\", { false }, \"a condition that never holds\", 40] call skd_fnc_waitUntil;\n"
                .into(),
        ),
        (
            "failthencrash",
            "// @check US-041 an assertion fails, then p2 crashes\n[\"p2\", { [0.5] call skdfx_fnc_crash }] call skd_fnc_on;\n[1, \"==\", 2, \"fails at once\"] call skd_fnc_expect;\n"
                .into(),
        ),
        (
            "hold",
            "// @check US-059 p1's clock stops for 8 s\n// @timeout 30\n[8] call skdfx_fnc_holdClock;\nprivate _t = diag_tickTime;\n[\"p1\", { diag_tickTime > (_this select 0) + 10 }, \"ten seconds of real time\", 20, [_t]] call skd_fnc_waitUntil;\n"
                .into(),
        ),
        (
            "failthenhold",
            "// @check US-059 an assertion fails, then the clock stops\n[] spawn { uiSleep 0.3; [8] call skdfx_fnc_holdClock };\n[1, \"==\", 2, \"fails first\"] call skd_fnc_expect;\n"
                .into(),
        ),
    ];
    for (id, text) in checks {
        std::fs::write(dir.join("checks").join(format!("{id}.sqf")), text).unwrap();
    }
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

/// Whether a process with this id runs.
fn runs(pid: u64) -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
}

/// Suspends or resumes a process by its id, through `NtSuspendProcess`, as an outside tool does
/// with the id `status` gives (`INS-04`).
fn suspend(pid: u64, resume: bool) {
    let call = if resume { "NtResumeProcess" } else { "NtSuspendProcess" };
    let script = format!(
        "Add-Type -Name N -Namespace W -MemberDefinition '[DllImport(\"ntdll.dll\")] public static extern int {call}(IntPtr h);'; $p = Get-Process -Id {pid}; [W.N]::{call}($p.Handle) | Out-Null"
    );
    let ok = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "{call} of {pid}");
}

fn pid_of(dir: &Path, machine: &str) -> u64 {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|m| m["name"] == machine)
        .and_then(|m| m["pid"].as_u64())
        .unwrap_or_else(|| panic!("no {machine} in {s}"))
}

/// Every game process the session started, by id.
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
        stop(&self.dir, &self.pids);
    }
}

fn start(dir: &Path, setup: &str) -> Session {
    steam();
    let (code, s, err) = cmd(dir, &["session", "start", "--setup", setup]);
    assert_eq!(code, 0, "{s} {err}");
    Session {
        dir: dir.to_path_buf(),
        pids: pids(dir),
    }
}

/// Stops the session, then any of `pids` still running, by id.
fn stop(dir: &Path, pids: &[u64]) {
    let _ = cmd(dir, &["session", "stop"]);
    for pid in pids {
        if runs(*pid) {
            let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
        }
    }
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

/// `run <ids> --json`: the exit code and the result file, validated.
fn run(dir: &Path, ids: &[&str]) -> (i32, Value, PathBuf) {
    let mut args = vec!["run"];
    args.extend_from_slice(ids);
    let (code, out, err) = cmd(dir, &args);
    let file = PathBuf::from(out["result_file"].as_str().unwrap_or_else(|| panic!("{out} {err}")));
    (code, validate(&file), file)
}

fn outcome(result: &Value, i: usize) -> (String, String, String) {
    let c = &result["checks"][i];
    (
        c["outcome"].as_str().unwrap_or_default().to_string(),
        c["reason"]["kind"].as_str().unwrap_or_default().to_string(),
        c["reason"]["message"].as_str().unwrap_or_default().to_string(),
    )
}

fn machine<'a>(result: &'a Value, name: &str) -> &'a Value {
    result["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|m| m["name"] == name)
        .unwrap()
}

/// `US-041` 1 and 2, `US-010`, `US-059` in singleplayer.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn us_041_us_010_us_059_outcomes_a_failed_reset_and_a_stopped_clock_in_singleplayer() {
    let dir = project("sp");
    let _session = start(&dir, "sp");

    // US-041 1 and 2: pass, fail / explicit and invalid / explicit, and the exit codes.
    let (code, r, _) = run(&dir, &["pass", "failing", "explicit"]);
    assert_eq!(code, 1);
    assert_eq!(outcome(&r, 0).0, "pass");
    assert_eq!(
        outcome(&r, 1),
        ("fail".into(), "explicit".into(), "broken on purpose".into())
    );
    assert_eq!(
        outcome(&r, 2),
        ("invalid".into(), "explicit".into(), "no network".into())
    );
    assert_eq!(
        (
            &r["summary"]["pass"],
            &r["summary"]["fail"],
            &r["summary"]["invalid"],
            &r["summary"]["total"]
        ),
        (&Value::from(1), &Value::from(1), &Value::from(1), &Value::from(3))
    );
    assert_eq!(run(&dir, &["explicit"]).0, 2);
    assert_eq!(run(&dir, &["pass"]).0, 0);

    // US-059 1 and 2: the clock held for 8 s in a check; status shows the flag while it stands.
    let watcher = {
        let dir = dir.clone();
        std::thread::spawn(move || {
            let mut seen = Vec::new();
            for _ in 0..60 {
                let (_, s, _) = cmd(&dir, &["status"]);
                let m = &s["session"]["machines"][0];
                seen.push(m["health"]["flag"].clone());
                std::thread::sleep(Duration::from_millis(500));
            }
            seen
        })
    };
    let (code, r, _) = run(&dir, &["hold"]);
    let (o, kind, message) = outcome(&r, 0);
    println!("US-059 hold: {o} {kind}: {message}");
    assert_eq!((code, o.as_str(), kind.as_str()), (2, "invalid", "unhealthy"));
    assert_eq!(r["checks"][0]["reason"]["machine"], "p1");
    assert!(message.contains("mission time stopped"), "{message}");
    let seen = watcher.join().unwrap();
    assert!(
        seen.iter().any(|f| f == "mission-time"),
        "status never showed the flag: {seen:?}"
    );
    assert!(seen.last().unwrap().is_null(), "the flag did not clear: {seen:?}");
    assert_eq!(outcome(&run(&dir, &["pass"]).1, 0).0, "pass");

    // US-059 3: an assertion that failed before the clock stopped stays fail.
    let (_, r, _) = run(&dir, &["failthenhold"]);
    assert_eq!(&outcome(&r, 0).0[..], "fail");
    assert_eq!(&outcome(&r, 0).1[..], "assertion");
    // The hold goes on after the check: its flag is raised about 5 s after the clock stopped and
    // clears 5 s after it runs again.
    let mut raised = false;
    for _ in 0..60 {
        let (_, s, _) = cmd(&dir, &["status"]);
        let flagged = !s["session"]["machines"][0]["health"]["flag"].is_null();
        raised |= flagged;
        if raised && !flagged {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(raised, "the hold after the failed assertion was never flagged");

    // US-010 3: a reset on a ready session.
    let (code, reset, _) = cmd(&dir, &["reset"]);
    assert_eq!((code, &reset["state"]), (0, &Value::from("done")), "{reset}");
    assert!(reset["duration_s"].as_f64().is_some());

    // US-010 1 and 2: a cleanup that never ends.
    let (code, r, _) = run(&dir, &["stuck", "pass", "pass"]);
    assert_eq!(code, 2);
    let (o, kind, message) = outcome(&r, 1);
    println!("US-010: {o} {kind}: {message}");
    assert_eq!((o.as_str(), kind.as_str()), ("invalid", "reset-failed"));
    assert!(message.contains("stuck"), "{message}");
    assert_eq!(outcome(&r, 2).1, "session-broken");
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "broken");
    let (code, _, err) = cmd(&dir, &["reset"]);
    assert_eq!(code, 4, "{err}");
    assert!(err.contains("reload") && err.contains("session stop"), "{err}");
}

/// Runs `ids` in the background while `act` acts on the session, and returns the run's exit
/// code, its result and how long it took.
fn run_while(dir: &Path, ids: &[&str], act: impl FnOnce()) -> (i32, Value, PathBuf, Duration) {
    let clock = Instant::now();
    let child = Command::new(exe())
        .arg("run")
        .args(ids)
        .arg("--json")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    act();
    let out = child.wait_with_output().unwrap();
    let took = clock.elapsed();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let file = PathBuf::from(v["result_file"].as_str().unwrap_or_else(|| panic!("{v}")));
    (out.status.code().unwrap_or(-1), validate(&file), file, took)
}

/// `US-015`, `US-045`, `US-044` 3 on a dedicated session with two clients: `p2` suspended during
/// a wait on it.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn us_015_us_045_us_044_a_suspended_client_on_a_dedicated_session() {
    let dir = project("stall");
    let session = start(&dir, "ded2");
    let all = session.pids.clone();
    let p2 = pid_of(&dir, "p2");
    let mut inspect = None;
    let (code, r, _, took) = run_while(&dir, &["waitp2", "pass"], || {
        std::thread::sleep(Duration::from_secs(5));
        suspend(p2, false);
        let clock = Instant::now();
        let (_, v, _) = cmd(&dir, &["inspect", "--machine", "p2"]);
        inspect = Some((clock.elapsed(), v));
    });
    let (inspect_took, v) = inspect.unwrap();
    println!(
        "US-045 inspect of the suspended p2 took {inspect_took:?}: {}",
        v["machine"]
    );
    assert!(inspect_took < Duration::from_secs(20));
    assert_eq!(v["machine"]["state"], "lost");
    let (o, kind, message) = outcome(&r, 0);
    println!("US-015: {o} {kind}: {message} (the run took {took:?})");
    assert_eq!((code, o.as_str(), kind.as_str()), (2, "invalid", "machine-lost"));
    assert_eq!(r["checks"][0]["reason"]["machine"], "p2");
    assert!(
        message.contains("stopped answering") && message.contains("poll"),
        "{message}"
    );
    assert!(
        r["checks"][0]["duration_s"].as_f64().unwrap() < 60.0 + 10.0,
        "the check ended past the wait's ceiling plus 10 s"
    );
    assert_eq!(outcome(&r, 1).1, "session-broken");
    let p2r = machine(&r, "p2");
    assert_eq!(
        (&p2r["end_state"], &p2r["lost"]["reason"]),
        (&Value::from("lost"), &Value::from("no-answer"))
    );
    let (_, s, _) = cmd(&dir, &["status"]);
    assert_eq!(s["session"]["state"], "broken");
    let m = s["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "p2")
        .unwrap()
        .clone();
    assert_eq!(m["state"], "lost");
    assert!(m["last_answer_at"].is_string());
    for pid in &all {
        assert!(runs(*pid), "game process {pid} ended");
    }
    let (code, logs, _) = cmd(&dir, &["logs", "--machine", "p2"]);
    assert_eq!(code, 0);
    assert!(!logs["lines"].as_array().unwrap().is_empty());
    for other in ["server", "p1"] {
        let clock = Instant::now();
        let (code, v, err) = cmd(&dir, &["inspect", "--machine", other]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(v["machine"]["state"], "ready", "{v}");
        assert!(clock.elapsed() < Duration::from_secs(10));
    }
    let (code, _, _) = cmd(&dir, &["errors"]);
    assert_eq!(code, 0);
    suspend(p2, true);
}

/// `US-016`, `US-043`, `US-041` 3 on dedicated sessions with two clients: `p2` ended from outside
/// during a check, crashed by the fixture before a wait on it, and crashed after an assertion
/// failed.
#[test]
#[ignore = "starts the game; run on the test machine"]
fn us_016_us_043_us_041_a_client_ends_or_crashes_during_a_check() {
    let dir = project("crash");

    // US-016 1 to 3, US-043 1: p2 ended by its id.
    let session = start(&dir, "ded2");
    let p2 = pid_of(&dir, "p2");
    let (code, r, file, _) = run_while(&dir, &["waitp2", "pass"], || {
        std::thread::sleep(Duration::from_secs(5));
        let _ = Command::new("taskkill").args(["/PID", &p2.to_string(), "/F"]).output();
    });
    let (o, kind, message) = outcome(&r, 0);
    println!("US-016: {o} {kind}: {message}");
    assert_eq!((code, o.as_str(), kind.as_str()), (2, "invalid", "machine-lost"));
    assert_eq!(r["checks"][0]["reason"]["machine"], "p2");
    assert!(message.contains("exit code 1"), "{message}");
    assert_eq!(outcome(&r, 1).1, "session-broken");
    let p2r = machine(&r, "p2");
    assert_eq!(p2r["lost"]["reason"], "exited");
    assert_eq!(p2r["lost"]["exit_code"], 1);
    let log = r["checks"][0]["evidence"]["logs"]["p2"].as_str().expect("p2's excerpt");
    let text = std::fs::read_to_string(file.parent().unwrap().join(log)).unwrap();
    assert!(text.contains("[shakedown]"), "{text}");
    drop(session);

    // US-016, US-043 2: p2 crashed by the fixture, its dumps in the run folder.
    let session = start(&dir, "ded2");
    let (code, r, file) = run(&dir, &["crashp2", "pass"]);
    let (o, kind, message) = outcome(&r, 0);
    println!("US-043: {o} {kind}: {message}");
    assert_eq!((code, o.as_str(), kind.as_str()), (2, "invalid", "machine-lost"));
    let p2r = machine(&r, "p2");
    assert_eq!(p2r["lost"]["reason"], "exited");
    let dumps = p2r["lost"]["dumps"].as_array().unwrap();
    assert!(!dumps.is_empty(), "{p2r}");
    for d in dumps {
        let d = d.as_str().unwrap();
        assert!(
            d.starts_with("dumps/p2/") && file.parent().unwrap().join(d).is_file(),
            "{d}"
        );
        assert!(message.contains(d), "{message}");
    }
    drop(session);

    // US-041 3: an assertion that failed before p2 was lost stays fail.
    let _session = start(&dir, "ded2");
    let (_, r, _) = run(&dir, &["failthencrash", "pass"]);
    assert_eq!(outcome(&r, 0).0, "fail");
    assert_eq!(outcome(&r, 0).1, "assertion");
}
