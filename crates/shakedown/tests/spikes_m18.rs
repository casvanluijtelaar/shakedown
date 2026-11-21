//! Spike SPK-22 (`spec/06-open-questions.md`), run before M18 sets the thresholds of the frame
//! rate and answer time watch (`HLT-01`): each machine's frame rate and answer time over 50
//! passing checks in singleplayer and on a dedicated session with two clients, and how fast a
//! client starved of CPU is flagged. They start the game, so they run only on the test machine,
//! by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes_m18 -- --ignored --nocapture --test-threads=1
//!
//! The installed `@shakedown` and fixtures are taken from `target/shakedown`. Results go to
//! `research/36-spike-frame-rate-and-answer-time.md`. Nothing here prints a line of an RPT.

#![cfg(windows)]

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use shakedown::config;
use shakedown::runner::{Runner, session};
use shakedown::wire::SetupSpec;
use support::starve::Starve;

fn plain(p: PathBuf) -> PathBuf {
    let p = std::fs::canonicalize(p).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

async fn start(runner: &Arc<Runner>, setup: &str) -> Option<Arc<session::Session>> {
    let e2e = plain(support::repo().join("e2e"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup(setup).unwrap().clone();
    let mods = setup.mods.iter().map(|m| project.mods[m].clone()).collect();
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
        base_port: project.base_port,
        project: project.root.clone(),
        lib: project.lib.clone(),
    };
    let session = session::start(runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    if session.view()["state"] != "ready" {
        println!("the session did not start: {}", session.view()["reason"]["message"]);
        session.stop(runner).await;
        return None;
    }
    Some(session)
}

async fn runner() -> Arc<Runner> {
    let addon = plain(support::repo().join("target").join("shakedown").join("@shakedown"));
    Runner::bind(addon, None).await.unwrap()
}

/// What `status` showed of one machine over a sampling.
#[derive(Default)]
struct Seen {
    samples: u32,
    rate_min: f64,
    rate_sum: f64,
    rates: u32,
    answer_max: f64,
    answer_sum: f64,
    answers: u32,
    flagged: BTreeMap<String, u32>,
}

/// Each machine's `health` as `status` shows it, sampled every 100 ms while `until` runs.
async fn watch<F: std::future::Future>(session: &session::Session, until: F) -> (F::Output, Vec<String>) {
    tokio::pin!(until);
    let mut seen: BTreeMap<String, Seen> = BTreeMap::new();
    let out = loop {
        tokio::select! {
            out = &mut until => break out,
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        for m in session.view()["machines"].as_array().into_iter().flatten() {
            let s = seen
                .entry(m["name"].as_str().unwrap_or_default().to_string())
                .or_insert(Seen {
                    rate_min: f64::INFINITY,
                    ..Seen::default()
                });
            let h = &m["health"];
            s.samples += 1;
            if let Some(r) = h["frame_rate"].as_f64() {
                s.rate_min = s.rate_min.min(r);
                s.rate_sum += r;
                s.rates += 1;
            }
            if let Some(a) = h["answer_s"].as_f64() {
                s.answer_max = s.answer_max.max(a);
                s.answer_sum += a;
                s.answers += 1;
            }
            if let Some(f) = h["flag"].as_str() {
                *s.flagged.entry(f.to_string()).or_default() += 1;
            }
        }
    };
    let lines = seen
        .iter()
        .map(|(name, s)| {
            format!(
                "{name}: {} samples, 5 s frame rate smallest {:.1} mean {:.1}, answer time largest {:.3} s mean {:.3} s, flagged {:?}",
                s.samples,
                s.rate_min,
                s.rate_sum / s.rates.max(1) as f64,
                s.answer_max,
                s.answer_sum / s.answers.max(1) as f64,
                s.flagged
            )
        })
        .collect();
    (out, lines)
}

/// SPK-22, part one: 50 passing checks in singleplayer and 50 on a dedicated session with two
/// clients and a headless client, each machine's frame rate and answer time recorded.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_22_frame_rate_and_answer_time_over_50_passing_checks() {
    let runner = runner().await;
    let project = std::env::temp_dir().join(format!("skd-spk22-{}", std::process::id()));
    std::fs::create_dir_all(&project).unwrap();
    let check = project.join("spk22.sqf");
    std::fs::write(
        &check,
        "// @check SPK-22 a passing check\n// @timeout 30\nsleep 1.5;\n[time, \">\", 0, \"the clock\"] call skd_fnc_expect;\n",
    )
    .unwrap();
    for setup in ["sp-fixture", "dedicated"] {
        let Some(s) = start(&runner, setup).await else { continue };
        let refs: Vec<shakedown::checks::CheckRef> = (0..50)
            .map(|_| shakedown::checks::CheckRef {
                id: "spk22".into(),
                path: check.clone(),
            })
            .collect();
        let run = shakedown::runner::run::run(
            &runner,
            project.clone(),
            vec![setup.to_string()],
            refs,
            vec!["spk22".into()],
            vec![],
            None,
            (1, 10.0),
            &|_, _| {},
            &|_, _| {},
        );
        let (result, lines) = watch(&s, run).await;
        let summary = result
            .map(|r| r["summary"].clone())
            .unwrap_or_else(|e| json!(e.message));
        println!("SPK-22 {setup}: summary {summary}");
        for l in lines {
            println!("SPK-22 {setup} {l}");
        }
        s.stop(&runner).await;
    }
}

/// Waits up to `ceiling` for `machine`'s flag to be raised (`want` true) or cleared; how long
/// it took and the machine's health then.
async fn until_flag(
    session: &session::Session,
    machine: &str,
    want: bool,
    ceiling: Duration,
) -> (Option<f64>, serde_json::Value) {
    let clock = Instant::now();
    let tries = ceiling.as_millis() / 100;
    for i in 0..tries {
        let m = session.machine(machine).unwrap_or_default();
        if m["health"]["flag"].is_null() != want {
            return (Some(clock.elapsed().as_secs_f64()), m["health"].clone());
        }
        if i % 20 == 0 {
            let h = &m["health"];
            println!(
                "  {machine} at {:.1} s: frame rate {} answer {} s",
                clock.elapsed().as_secs_f64(),
                h["frame_rate"],
                h["answer_s"]
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    (None, session.machine(machine).unwrap_or_default()["health"].clone())
}

/// How long `p2` is suspended and then runs, in milliseconds: `SKD_STARVE` as `<off>,<on>`, by
/// default 480 and 20.
fn duty() -> (u64, u64) {
    std::env::var("SKD_STARVE")
        .ok()
        .and_then(|v| {
            let (a, b) = v.split_once(',')?;
            Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
        })
        .unwrap_or((480, 20))
}

/// SPK-22, part two: `p2` starved of CPU, suspended 480 ms of every 500 ms by its process id (`duty`), is
/// flagged within 15 s, 5 rounds on one dedicated session with two clients and a headless client;
/// and its flag clears once it runs freely again.
#[tokio::test]
#[ignore = "starts the game; run on the test machine"]
async fn spk_22_a_starved_client_is_flagged() {
    let runner = runner().await;
    let Some(s) = start(&runner, "dedicated").await else {
        panic!("the session did not start");
    };
    // Settle: every machine's frame rate known and none flagged.
    let (settled, _) = until_flag(&s, "p2", false, Duration::from_secs(30)).await;
    assert!(settled.is_some());
    let mut flagged = 0;
    for round in 1..=5 {
        let p2 = s.machine_pid("p2").unwrap();
        let before = s.machine("p2").unwrap()["health"].clone();
        let (off, on) = duty();
        let starve = Starve::start(p2, Duration::from_millis(off), Duration::from_millis(on));
        let (took, health) = until_flag(&s, "p2", true, Duration::from_secs(30)).await;
        drop(starve);
        flagged += took.is_some_and(|t| t <= 15.0) as u32;
        println!("SPK-22 round {round}: before {before}; p2 starved, flagged after {took:?} s: {health}");
        let others: Vec<String> = s.view()["machines"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["name"] != "p2")
            .map(|m| format!("{} {}", m["name"], m["health"]["flag"]))
            .collect();
        println!("SPK-22 round {round}: the others {others:?}");
        let (cleared, health) = until_flag(&s, "p2", false, Duration::from_secs(40)).await;
        println!("SPK-22 round {round}: p2 runs freely, cleared after {cleared:?} s: {health}");
    }
    println!("SPK-22 starved client flagged within 15 s: {flagged} of 5");
    s.stop(&runner).await;
}
