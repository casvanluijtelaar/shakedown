//! Spikes SPK-16, SPK-17 and SPK-15 (`spec/06-open-questions.md`), run before the M9 work that
//! depends on them: a crash on demand on each kind of game process, mission time held while
//! frames go on, and the thresholds of the clock watch. They start the game, so they run only on
//! the test machine, by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes_m09 -- --ignored --nocapture --test-threads=1
//!
//! The installed `@shakedown` and fixtures are taken from `target/shakedown`. Results go to
//! `research/28-spikes-health.md`. Nothing here prints a line of an RPT.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shakedown::config;
use shakedown::runner::{Runner, session};
use shakedown::wire::SetupSpec;

fn plain(p: PathBuf) -> PathBuf {
    let p = std::fs::canonicalize(p).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

async fn start(runner: &Arc<Runner>, setup: &str) -> Option<Arc<session::Session>> {
    let e2e = plain(repo().join("e2e"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup(setup).unwrap().clone();
    let mods = setup.mods.iter().map(|m| project.mods[m].clone()).collect();
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
        base_port: project.base_port,
    };
    let session = session::start(runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    let view = session.view();
    if view["state"] != "ready" {
        println!("the session did not start: {}", view["reason"]["message"]);
        return None;
    }
    Some(session)
}

async fn runner() -> Arc<Runner> {
    let addon = plain(repo().join("target").join("shakedown").join("@shakedown"));
    Runner::bind(addon, None).await.unwrap()
}

async fn value(session: &session::Session, machine: &str, source: &str, args: Value) -> Value {
    match session.exec(machine, source, args, 10.0).await {
        Ok(a) if a["state"] == "done" => a["value"].clone(),
        Ok(a) => json!({"answer": a}),
        Err(e) => json!({"no-answer": format!("{e:?}")}),
    }
}

/// The crash dumps in a profile folder.
fn dumps(profile: &Path) -> Vec<String> {
    std::fs::read_dir(profile)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("mdmp") || e.eq_ignore_ascii_case("bidmp"))
        })
        .map(|p| {
            format!(
                "{} ({} bytes)",
                p.file_name().unwrap().to_string_lossy(),
                std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)
            )
        })
        .collect()
}

/// Crashes `machine` with `skdfx_fnc_crash` after 0.5 s and waits up to 10 s for its process to
/// end; prints and returns whether it ended, its exit code, how long it took and its dumps.
async fn crash(session: &session::Session, machine: &str, label: &str) -> bool {
    let asked = value(session, machine, "[0.5] call skdfx_fnc_crash", json!([])).await;
    let clock = Instant::now();
    let mut code = None;
    for _ in 0..100 {
        code = session.exit_code(machine);
        if code.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let took = clock.elapsed();
    // A dump still being written is given up to 5 s.
    let profile = session.profile(machine).unwrap();
    let mut found = dumps(&profile);
    for _ in 0..50 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let now = dumps(&profile);
        if !now.is_empty() && now == found {
            break;
        }
        found = now;
    }
    println!(
        "SPK-16 {label} {machine}: asked {asked}; exit code {} ({}) after {:.2} s; dumps {found:?}",
        code.map_or("none".into(), |c| c.to_string()),
        code.map_or("running".into(), |c| format!("{c:#010X}")),
        took.as_secs_f64(),
    );
    code.is_some_and(|c| c != 0) && !found.is_empty() && took < Duration::from_secs(10)
}

/// SPK-16: the fixture's crash on singleplayer `p1`, a dedicated `server`, a client and a
/// headless client, 5 rounds each.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_16_a_crash_on_demand_on_each_kind_of_game_process() {
    let runner = runner().await;
    let mut passed = std::collections::BTreeMap::<&str, u32>::new();
    for round in 1..=5 {
        if let Some(s) = start(&runner, "sp-fixture").await {
            let ok = crash(&s, "p1", &format!("round {round} singleplayer")).await;
            *passed.entry("singleplayer p1").or_default() += ok as u32;
            s.stop(&runner).await;
        }
        if let Some(s) = start(&runner, "dedicated").await {
            let label = format!("round {round} dedicated");
            let ok = crash(&s, "p1", &label).await;
            *passed.entry("client").or_default() += ok as u32;
            let ok = crash(&s, "hc1", &label).await;
            *passed.entry("headless client").or_default() += ok as u32;
            let ok = crash(&s, "server", &label).await;
            *passed.entry("server").or_default() += ok as u32;
            s.stop(&runner).await;
        }
    }
    println!("SPK-16 passed rounds per kind: {passed:?}");
}

/// What `time`, `diag_tickTime` and `diag_frameNo` read on `p1`.
async fn clocks(session: &session::Session) -> (f64, f64, f64) {
    let v = value(session, "p1", "[time, diag_tickTime, diag_frameNo]", json!([])).await;
    let n = |i: usize| v[i].as_f64().unwrap_or(f64::NAN);
    (n(0), n(1), n(2))
}

/// The two candidate holds of SPK-17: each holds `time` on `p1` for `_this select 0` seconds of
/// real time and ends itself from an `EachFrame` handler gated on `diag_tickTime`. Each also
/// records, from its own `EachFrame` handler, the real time of a beat every second, as the
/// heartbeat of `HLT-02` would send it.
const HOLD: &str = r#"
params ["_seconds", "_how"];
skd_spk17_beats = [];
skd_spk17_last = diag_tickTime;
skd_spk17_until = diag_tickTime + _seconds;
skd_spk17_how = _how;
if (_how == "accTime") then { setAccTime 0 } else {
    (findDisplay 46) createDisplay "RscDisplayInterrupt";
};
addMissionEventHandler ["EachFrame", {
    if (diag_tickTime - skd_spk17_last >= 1) then {
        skd_spk17_last = diag_tickTime;
        skd_spk17_beats pushBack [diag_tickTime, time, diag_frameNo];
    };
    if (diag_tickTime >= skd_spk17_until) then {
        if (skd_spk17_how == "accTime") then { setAccTime 1 } else {
            (findDisplay 49) closeDisplay 2;
        };
        removeMissionEventHandler ["EachFrame", _thisEventHandler];
    };
}];
[accTime, isGamePaused]
"#;

/// SPK-17: each candidate hold, 10 times for 8 s in singleplayer.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_17_mission_time_held_while_frames_go_on() {
    let runner = runner().await;
    let s = start(&runner, "sp-fixture").await.expect("a session");
    for how in ["accTime", "pause"] {
        let mut passed = 0;
        for round in 1..=10 {
            let (t0, k0, f0) = clocks(&s).await;
            let started = value(&s, "p1", HOLD, json!([8, how])).await;
            tokio::time::sleep(Duration::from_secs(2)).await;
            let clock = Instant::now();
            let answer = s.exec("p1", "1 + 1", json!([]), 10.0).await;
            let answered = clock.elapsed();
            let answered_ok = answer.as_ref().is_ok_and(|a| a["value"] == 2);
            let (t1, k1, f1) = clocks(&s).await;
            // Until the hold has ended, with a margin.
            let left = 8.0 - (k1 - k0) + 0.5;
            if left > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(left)).await;
            }
            let (t2, k2, f2) = clocks(&s).await;
            tokio::time::sleep(Duration::from_secs(3)).await;
            let (t3, k3, _) = clocks(&s).await;
            let beats = value(&s, "p1", "skd_spk17_beats", json!([])).await;
            let ticks: Vec<f64> = beats
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|b| b[0].as_f64())
                .collect();
            let gap = ticks.windows(2).map(|w| w[1] - w[0]).fold(0.0, f64::max);
            let held = t2 - t0;
            let rate = (t3 - t2) / (k3 - k2);
            let ok = held < 0.1 + (k2 - k0 - 8.0).max(0.0)
                && f2 > f1
                && f1 > f0
                && answered_ok
                && answered < Duration::from_secs(20)
                && gap <= 1.5
                && ticks.len() >= 6
                && (0.9..=1.1).contains(&rate);
            passed += ok as u32;
            println!(
                "SPK-17 {how} round {round}: start {started}; time advanced {held:.3} s over {:.2} s real; frames {f0} -> {f1} -> {f2}; exec answered {answered_ok} in {:.2} s; {} beats, largest gap {gap:.2} s; after the hold time ran at {rate:.3} of real time (t1 {t1:.2} k1 {k1:.2}); pass {ok}",
                k2 - k0,
                answered.as_secs_f64(),
                ticks.len(),
            );
        }
        println!("SPK-17 {how}: {passed} of 10");
    }
    s.stop(&runner).await;
}

/// The window titles and the responding state of a process, from PowerShell.
fn windows_of(pid: u32) -> String {
    let script = format!(
        "Get-Process -Id {pid} -ErrorAction SilentlyContinue | ForEach-Object {{ \"title='$($_.MainWindowTitle)' responding=$($_.Responding) threads=$($_.Threads.Count)\" }}"
    );
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if out.is_empty() { "gone".into() } else { out }
}

/// SPK-16, the headless client alone: how long it lives after the fixture's crash, with the
/// server and the clients still up, and what its process shows meanwhile.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_16_the_headless_client_after_its_crash() {
    let runner = runner().await;
    for round in 1..=4 {
        let Some(s) = start(&runner, "dedicated").await else {
            continue;
        };
        let pid = s.machine_pid("hc1").unwrap();
        // Rounds 3 and 4 crash p1 first, as the whole spike does.
        if round > 2 {
            crash(&s, "p1", &format!("hc follow-up round {round}")).await;
        }
        let asked = value(&s, "hc1", "[0.5] call skdfx_fnc_crash", json!([])).await;
        let clock = Instant::now();
        let mut code = None;
        for i in 0..900 {
            code = s.exit_code("hc1");
            if code.is_some() {
                break;
            }
            if i % 50 == 25 {
                println!(
                    "SPK-16 hc1 round {round} at {:.1} s: {}",
                    clock.elapsed().as_secs_f64(),
                    windows_of(pid)
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        println!(
            "SPK-16 hc1 alone round {round}: asked {asked}; exit code {} after {:.1} s; dumps {:?}",
            code.map_or("none".into(), |c| format!("{c:#010X}")),
            clock.elapsed().as_secs_f64(),
            dumps(&s.profile("hc1").unwrap())
        );
        s.stop(&runner).await;
    }
}

/// Each machine's heartbeats as `status` shows them, sampled every 100 ms while `until` runs:
/// per machine the largest gap between two heartbeats as they arrived, the smallest ratio of
/// mission time to real time over any 5 s window, the smallest frame advance over one, and how
/// often a machine was flagged.
async fn watch_beats<F: std::future::Future>(session: &session::Session, until: F) -> (F::Output, Vec<String>) {
    use std::collections::BTreeMap;
    tokio::pin!(until);
    let mut beats: BTreeMap<String, Vec<(Instant, f64, f64, f64)>> = BTreeMap::new();
    let mut flagged: BTreeMap<String, u32> = BTreeMap::new();
    let out = loop {
        tokio::select! {
            out = &mut until => break out,
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        for m in session.view()["machines"].as_array().into_iter().flatten() {
            let name = m["name"].as_str().unwrap_or_default().to_string();
            if !m["health"]["flag"].is_null() {
                *flagged.entry(name.clone()).or_default() += 1;
            }
            let hb = &m["health"]["heartbeat"];
            let (Some(tick), Some(time), Some(frame)) =
                (hb["tick"].as_f64(), hb["time"].as_f64(), hb["frame"].as_f64())
            else {
                continue;
            };
            let list = beats.entry(name).or_default();
            if list.last().is_none_or(|l| l.1 != tick) {
                list.push((Instant::now(), tick, time, frame));
            }
        }
    };
    let mut lines = Vec::new();
    for (name, list) in &beats {
        let gap = list
            .windows(2)
            .map(|w| (w[1].0 - w[0].0).as_secs_f64())
            .fold(0.0, f64::max);
        let tick_gap = list.windows(2).map(|w| w[1].1 - w[0].1).fold(0.0, f64::max);
        let mut ratio = f64::INFINITY;
        let mut frames = f64::INFINITY;
        for (i, b) in list.iter().enumerate() {
            if let Some(a) = list[..i].iter().rev().find(|a| b.1 - a.1 >= 5.0) {
                ratio = ratio.min((b.2 - a.2) / (b.1 - a.1));
                frames = frames.min(b.3 - a.3);
            }
        }
        lines.push(format!(
            "{name}: {} beats, largest gap on arrival {gap:.2} s and in real time {tick_gap:.2} s, smallest 5 s ratio of mission to real time {ratio:.3}, smallest 5 s frame advance {frames}, samples flagged {}",
            list.len(),
            flagged.get(name).copied().unwrap_or(0)
        ));
    }
    (out, lines)
}

/// SPK-15, part one: 50 passing checks in singleplayer and 50 on a dedicated session with two
/// clients and a headless client, each machine's heartbeat gap and clock ratio recorded.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_15_heartbeats_over_50_passing_checks() {
    let runner = runner().await;
    let project = std::env::temp_dir().join(format!("skd-spk15-{}", std::process::id()));
    std::fs::create_dir_all(&project).unwrap();
    let check = project.join("spk15.sqf");
    std::fs::write(
        &check,
        "// @check SPK-15 a passing check\n// @timeout 30\nsleep 1.5;\n[time, \">\", 0, \"the clock\"] call skd_fnc_expect;\n",
    )
    .unwrap();
    for setup in ["sp-fixture", "dedicated"] {
        let Some(s) = start(&runner, setup).await else { continue };
        let refs: Vec<shakedown::checks::CheckRef> = (0..50)
            .map(|_| shakedown::checks::CheckRef {
                id: "spk15".into(),
                path: check.clone(),
            })
            .collect();
        let run = shakedown::runner::run::run(
            &runner,
            project.clone(),
            vec![setup.to_string()],
            refs,
            vec!["spk15".into()],
            vec![],
            None,
            &|_, _| {},
            &|_, _| {},
        );
        let (result, lines) = watch_beats(&s, run).await;
        let summary = result
            .map(|r| r["summary"].clone())
            .unwrap_or_else(|e| json!(e.message));
        println!("SPK-15 {setup}: summary {summary}");
        for l in lines {
            println!("SPK-15 {setup} {l}");
        }
        s.stop(&runner).await;
    }
}

/// Suspends or resumes a process by its id, through `NtSuspendProcess` from PowerShell.
fn suspend(pid: u32, resume: bool) {
    let call = if resume { "NtResumeProcess" } else { "NtSuspendProcess" };
    let script = format!(
        "Add-Type -Name N -Namespace W -MemberDefinition '[DllImport(\"ntdll.dll\")] public static extern int {call}(IntPtr h);'; $p = Get-Process -Id {pid}; [W.N]::{call}($p.Handle) | Out-Null"
    );
    let ok = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .is_ok_and(|s| s.success());
    assert!(ok, "{call} of {pid}");
}

/// Waits up to 20 s for `machine` to show `lost` in `status`; how long it took.
async fn until_lost(session: &session::Session, machine: &str) -> Option<f64> {
    let clock = Instant::now();
    for _ in 0..200 {
        if session.machine(machine).is_some_and(|m| m["state"] == "lost") {
            return Some(clock.elapsed().as_secs_f64());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    None
}

/// SPK-15, part two: a suspended client and a killed client are each flagged within 15 s, 5
/// rounds on a dedicated session with two clients and a headless client.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_15_a_suspended_and_a_killed_client_are_flagged() {
    let runner = runner().await;
    let (mut suspended, mut killed) = (0, 0);
    for round in 1..=5 {
        let Some(s) = start(&runner, "dedicated").await else {
            continue;
        };
        let p2 = s.machine_pid("p2").unwrap();
        suspend(p2, false);
        let took = until_lost(&s, "p2").await;
        suspended += took.is_some_and(|t| t <= 15.0) as u32;
        println!(
            "SPK-15 round {round}: p2 suspended, lost after {took:?} s: {}",
            s.view()["reason"]["message"]
        );
        let p1 = s.machine_pid("p1").unwrap();
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &p1.to_string()])
            .output();
        let took = until_lost(&s, "p1").await;
        killed += took.is_some_and(|t| t <= 15.0) as u32;
        println!(
            "SPK-15 round {round}: p1 killed, lost after {took:?} s; status {}",
            s.machine("p1").unwrap()
        );
        suspend(p2, true);
        s.stop(&runner).await;
    }
    println!("SPK-15 suspended flagged within 15 s: {suspended} of 5; killed: {killed} of 5");
}
