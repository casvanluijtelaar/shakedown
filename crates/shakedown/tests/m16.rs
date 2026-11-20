//! M16's tests: the environment is checked in full before a start, every gap at once
//! (`US-085`, `ENV-01`, `SES-03`), a mod in another version than the project names fails the start
//! (`US-086`, `ENV-01`), a client that hangs in its start fails it and the reason is kept in
//! `session.json` (`US-087`, `SES-04`, `SES-10`), and a client sent to another server is ended
//! (`US-088`, `SEC-05`). They drive the installed `shakedown` and, but for `US-085`, the game, so
//! they run only on the test machine, by hand, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m16 -- --ignored --nocapture --test-threads=1
//!
//! Every process a test starts is ended: the session by `session stop`, and any process of it left
//! after that, and the second server of `US-088`, by its id.

#![cfg(windows)]

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use support::repo;

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

/// A project with the e2e project's file, `extra` appended, and these checks.
fn project(name: &str, edit: impl Fn(String) -> String, checks: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m16-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let toml = std::fs::read_to_string(repo().join("e2e").join("shakedown.toml"))
        .unwrap()
        .replace(r"'..\target", &format!("'{}", repo().join("target").display()));
    std::fs::write(dir.join("shakedown.toml"), edit(toml)).unwrap();
    for (id, text) in checks {
        std::fs::write(dir.join("tests").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

/// The ids of every running game process, client and server.
fn game_pids() -> Vec<u32> {
    let out = Command::new("tasklist").args(["/FO", "CSV", "/NH"]).output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.starts_with("\"arma3_x64.exe\"") || l.starts_with("\"arma3server_x64.exe\""))
        .filter_map(|l| l.split("\",\"").nth(1).and_then(|p| p.parse().ok()))
        .collect()
}

fn session_pids(dir: &Path) -> Vec<u32> {
    let (_, s, _) = cmd(dir, &["status"]);
    s["session"]["machines"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["pid"].as_u64().map(|p| p as u32))
        .collect()
}

fn kill(pid: u32) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
}

/// Stops the session when dropped, also when the test panics; any process of it left after the
/// stop is ended by its id.
struct Stop(PathBuf);

impl Drop for Stop {
    fn drop(&mut self) {
        let left = session_pids(&self.0);
        let (code, _, _) = cmd(&self.0, &["session", "stop"]);
        if code != 0 {
            left.into_iter().for_each(kill);
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

/// The newest session folder's `session.json`.
fn session_json(dir: &Path, id: &str) -> Value {
    let file = dir.join(".shakedown").join("sessions").join(id).join("session.json");
    serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap()
}

/// `US-085` 1 and 3, `ENV-01`, `SES-03`: a missing mod folder and a held port are named together,
/// before any game process starts, by `session start` and by the `session_start` tool. Steam
/// logged out and a full drive are proven by the unit tests of the runner, since this test must
/// not log out of Steam or fill the drive.
#[test]
#[ignore]
fn us_085_env_01_every_gap_is_named_at_once() {
    let port = 2462u16;
    let dir = project(
        "gaps",
        |t| {
            t.replace("game = ", &format!("base_port = {port}\ngame = "))
                + "\n[mods.gone]\npath = 'C:\\no\\such\\mod'\n\n[setups.gaps]\nmode = \"dedicated\"\nmods = [\"fixture\", \"gone\"]\n"
        },
        &[],
    );
    // A port held by another program, bound on loopback only (NFR-SC-01).
    let holder = std::net::UdpSocket::bind(("127.0.0.1", port + 1)).unwrap();
    let before = game_pids();
    let (code, out, err) = cmd(&dir, &["session", "start", "--setup", "gaps"]);
    assert_eq!(code, 4, "{out} {err}");
    assert!(err.contains(r"the folder C:\no\such\mod of mod gone"), "{err}");
    assert!(err.contains(&format!("{} held by another program", port + 1)), "{err}");
    assert!(game_pids().iter().all(|p| before.contains(p)), "a game process started");

    let mut mcp = support::Mcp::start(&exe(), &dir, &[]);
    let answer = mcp.call("session_start", json!({"setup": "gaps"}));
    assert!(answer.is_error(), "{}", answer.result);
    let text = answer.text();
    assert!(
        text.contains("precondition-failed") || answer.result.to_string().contains("precondition-failed"),
        "{}",
        answer.result
    );
    assert!(text.contains(r"the folder C:\no\such\mod of mod gone"), "{text}");
    assert!(
        text.contains(&format!("{} held by another program", port + 1)),
        "{text}"
    );
    drop(holder);
}

/// `US-086` 1 and 2, `ENV-01`, `INT-STM-03`; `DEF-29` 3: the fixture mod named in another version
/// fails the start naming the mod, the version found and the one wanted, with no game process
/// left; named in its own version the start is ready, and a stop from ready writes reason null.
#[test]
#[ignore]
fn us_086_env_01_a_mod_in_another_version_fails_the_start() {
    steam_runs();
    let with = |v: &'static str| {
        move |t: String| {
            t.replace(
                "patch = \"skdfx_main\"",
                &format!("patch = \"skdfx_main\"\nversion = \"{v}\""),
            )
        }
    };
    let dir = project("version", with("9.9.9"), &[]);
    let _stop = Stop(dir.clone());
    let before = game_pids();
    let (code, out, err) = cmd(&dir, &["session", "start", "--setup", "sp-fixture"]);
    assert_eq!(code, 4, "{out} {err}");
    assert!(
        err.contains("mod fixture on p1 is version 0.0.6, and shakedown.toml wants version 9.9.9"),
        "{err}"
    );
    assert!(
        game_pids().iter().all(|p| before.contains(p)),
        "a game process runs after the start"
    );
    let (_, status, _) = cmd(&dir, &["status"]);
    let id = status["session"]["id"].as_str().unwrap().to_string();
    assert_eq!(status["session"]["state"], "failed");
    let end = session_json(&dir, &id);
    assert_eq!(end["state"], "failed");
    assert!(
        end["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("wants version 9.9.9"),
        "{end}"
    );
    assert_eq!(end["reason"]["machine"], "p1");
    assert_eq!(cmd(&dir, &["session", "stop"]).0, 0);

    let dir = project("version-ok", with("0.0.6"), &[]);
    let _stop = Stop(dir.clone());
    let (code, out, err) = cmd(&dir, &["session", "start", "--setup", "sp-fixture"]);
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(out["state"], "ready", "{out}");
    let id = out["id"].as_str().unwrap().to_string();
    assert_eq!(cmd(&dir, &["session", "stop"]).0, 0);
    let end = session_json(&dir, &id);
    assert_eq!(end["state"], "stopped");
    assert!(end["reason"].is_null(), "{end}");
}

/// Suspends, or resumes, every thread of a process.
fn suspend(pid: u32, on: bool) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME};
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) else {
            return;
        };
        let mut e = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        let mut ok = Thread32First(snap, &mut e).is_ok();
        while ok {
            if e.th32OwnerProcessID == pid
                && let Ok(t) = OpenThread(THREAD_SUSPEND_RESUME, false, e.th32ThreadID)
            {
                if on {
                    SuspendThread(t);
                } else {
                    ResumeThread(t);
                }
                let _ = CloseHandle(t);
            }
            ok = Thread32Next(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
}

/// `US-087` 1 and 2, `SES-04`, `SES-10`, `INS-02`; `DEF-29` 2: `p2` suspended right after its
/// launch fails the start at the start ceiling, naming `p2`, its state `launched` and its RPT
/// tail, with every process stopped; after `session stop` clears it, `session.json` holds the same
/// reason.
#[test]
#[ignore]
fn us_087_ses_10_a_hung_client_fails_the_start_and_the_reason_is_kept() {
    steam_runs();
    let ceiling = 90u64;
    let dir = project(
        "hung",
        move |t| {
            t + &format!(
                "\n[setups.hung]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\nstart_timeout = {ceiling}\n"
            )
        },
        &[],
    );
    let _stop = Stop(dir.clone());
    let started = Instant::now();
    let start = {
        let exe = exe();
        let dir = dir.clone();
        std::thread::spawn(move || support::run_json(&exe, &dir, &[], &["session", "start", "--setup", "hung"]))
    };
    // p2 is launched once the server is ready; it is suspended as soon as status names its process.
    let mut p2 = None;
    for _ in 0..(ceiling * 10) {
        let (_, s, _) = cmd(&dir, &["status"]);
        if let Some(pid) = s["session"]["machines"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|m| m["name"] == "p2")
            .and_then(|m| m["pid"].as_u64())
        {
            suspend(pid as u32, true);
            p2 = Some(pid as u32);
            break;
        }
        if start.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let p2 = p2.expect("p2 was never launched");
    let (code, out, err) = start.join().unwrap();
    let took = started.elapsed();
    assert_eq!(code, 4, "{out} {err}");
    assert!(took < Duration::from_secs(ceiling + 20), "the start took {took:?}");
    assert!(err.contains("p2 was launched"), "{err}");
    // A process suspended at once may not have written its RPT yet; the reason says which.
    assert!(
        err.contains("The last lines of p2's RPT") || err.contains("p2 wrote no RPT line."),
        "{err}"
    );
    assert!(!game_pids().contains(&p2), "p2's process still runs");
    let (_, status, _) = cmd(&dir, &["status"]);
    let reason = status["session"]["reason"].clone();
    let id = status["session"]["id"].as_str().unwrap().to_string();
    assert_eq!(reason["machine"], "p2", "{reason}");
    assert_eq!(cmd(&dir, &["session", "stop"]).0, 0);
    let (_, status, _) = cmd(&dir, &["status"]);
    assert!(status["session"].is_null(), "{status}");
    let end = session_json(&dir, &id);
    assert_eq!(end["state"], "failed");
    assert_eq!(
        end["reason"], reason,
        "session.json does not hold the reason status gave"
    );
}

/// A second dedicated server outside the session, on loopback, named `other`, with a copy of the
/// session's mission; stopped by its id and its mission copy removed when dropped.
struct OtherServer {
    child: std::process::Child,
    mission: PathBuf,
}

impl Drop for OtherServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.mission);
    }
}

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn other_server(session_folder: &Path, port: u16) -> OtherServer {
    let game = PathBuf::from(GAME);
    let source = std::fs::read_dir(session_folder.join("mission"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let name = "skd_m16_other.VR";
    let mission = game.join("MPMissions").join(name);
    std::fs::create_dir_all(&mission).unwrap();
    for f in ["mission.sqm", "description.ext"] {
        std::fs::copy(source.join(f), mission.join(f)).unwrap();
    }
    let profile = session_folder.join("other-server");
    std::fs::create_dir_all(&profile).unwrap();
    let cfg = profile.join("server.cfg");
    std::fs::write(
        &cfg,
                    "hostname = \"other\";\nloopback = 1;\nBattlEye = 0;\nverifySignatures = 0;\nkickDuplicate = 0;\nallowedFilePatching = 2;\npersistent = 1;\nmaxPlayers = 4;\nclass Missions\n{{\n    class m\n    {{\n        template = \"skd_m16_other.VR\";\n        difficulty = \"Regular\";\n    }};\n}};\n",
    )
    .unwrap();
    let child = Command::new(game.join("arma3server_x64.exe"))
        .current_dir(&game)
        .args([
            format!("-port={port}"),
            format!("-config={}", cfg.display()),
            format!("-profiles={}", profile.display()),
            format!(
                "-mod={};{}",
                game.join("@skd_fixture").display(),
                game.join("@shakedown").display()
            ),
            "-filePatching".into(),
            "-autoInit".into(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let server = OtherServer { child, mission };
    // Up once its RPT says the mission runs, within 120 s.
    for _ in 0..240 {
        let up = std::fs::read_dir(&profile)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "rpt"))
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .contains("Mission id:")
            });
        if up {
            return server;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    panic!("the other server did not start its mission within 120 s");
}

/// `US-088` 1 and 2, `SEC-05`, `DM-04`, `NFR-LG-02`: code sent to `p2` that makes it join another
/// server ends `p2`'s process once its agent starts there, the check ends `fail` / `check-error`
/// saying a client may join only the session's own server, the session is `broken` with that
/// reason, and a `game-restart` reload brings every machine back.
///
/// The check's code aborts `p2` to the main menu through the multiplayer pause menu and joins the
/// other server from there, since `connectToServer` does nothing inside a mission (research
/// `33-client-to-another-server.md`). Without the owner connected through Shadow PC, so that the
/// client windows render, the abort did nothing on the test machine and the test fails at the
/// wait; it is an owner look.
#[test]
#[ignore]
fn us_088_sec_05_a_client_sent_to_another_server_is_ended() {
    steam_runs();
    let port = 2432u16;
    let check = format!(
        "// @check US-088 p2 is sent to another server\n// @setups other\n// @timeout 200\n[\"p2\", {{ with uiNamespace do {{ [_this select 0] spawn {{ params [\"_port\"]; private _t = diag_tickTime; waitUntil {{ getClientStateNumber < 9 || {{ diag_tickTime - _t > 60 }} }}; connectToServer [\"127.0.0.1\", _port, \"\"]; }}; }}; createDialog \"RscDisplayMPInterrupt\"; ctrlActivate ((findDisplay 49) displayCtrl 104); true }}, [{port}]] call skd_fnc_on;\n[\"server\", {{ false }}, \"p2 to be ended\", 180] call skd_fnc_waitUntil;\n"
    );
    let dir = project(
        "other",
        |t| t + "\n[setups.other]\nmode = \"dedicated\"\nclients = 2\nmods = [\"fixture\"]\n",
        &[("US-088-other-server", check.as_str())],
    );
    let _stop = Stop(dir.clone());
    let (code, out, err) = cmd(&dir, &["session", "start", "--setup", "other"]);
    assert_eq!(code, 0, "{out} {err}");
    let id = out["id"].as_str().unwrap().to_string();
    let p2 = session_pids(&dir)[2];
    let _other = other_server(&dir.join(".shakedown").join("sessions").join(&id), port);

    let t = Instant::now();
    let (code, run, err) = cmd(&dir, &["run", "US-088-other-server"]);
    println!("run: exit {code} in {:.1} s", t.elapsed().as_secs_f64());
    let file = run["result_file"]
        .as_str()
        .unwrap_or_else(|| panic!("no result: {run} {err}"));
    let result: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let c = &result["checks"][0];
    assert_eq!(c["outcome"], "fail", "{c:#}");
    assert_eq!(c["reason"]["kind"], "check-error", "{c:#}");
    assert_eq!(c["reason"]["machine"], "p2", "{c:#}");
    let message = c["reason"]["message"].as_str().unwrap();
    assert!(
        message.contains("a client may join only the session's own server") && message.contains("\"other\""),
        "{message}"
    );
    let lost = result["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "p2")
        .unwrap()["lost"]
        .clone();
    assert_eq!(lost["reason"], "foreign-server", "{lost}");
    assert_eq!(code, 1);
    assert!(!game_pids().contains(&(p2)), "p2's process still runs");
    let (_, status, _) = cmd(&dir, &["status"]);
    assert_eq!(status["session"]["state"], "broken", "{status}");
    assert!(
        status["session"]["reason"]["message"]
            .as_str()
            .unwrap()
            .contains("a client may join only the session's own server"),
        "{status}"
    );

    let (code, out, err) = cmd(&dir, &["reload"]);
    assert_eq!(code, 0, "{out} {err}");
    let (_, status, _) = cmd(&dir, &["status"]);
    assert_eq!(status["session"]["state"], "ready", "{status}");
    for m in status["session"]["machines"].as_array().unwrap() {
        assert_eq!(m["state"], "ready", "{m}");
    }
}
