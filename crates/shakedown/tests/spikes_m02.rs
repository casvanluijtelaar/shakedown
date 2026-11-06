//! Spikes SPK-01 and SPK-03 for a dedicated `server`, a game client and a headless client
//! (`spec/06-open-questions.md`), in one dedicated session with two clients and a headless
//! client. They start the game, so they run only on the test machine, by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes_m02 -- --ignored --nocapture
//!
//! The installed `@shakedown` is taken from `target/shakedown`. Results go to `research/`.

use std::path::PathBuf;
use std::time::Instant;

use serde_json::json;
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

/// The memory in use on the machine and the working set of every game process, in MB.
fn memory() -> (u64, u64) {
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "$os = Get-CimInstance Win32_OperatingSystem; $used = ($os.TotalVisibleMemorySize - $os.FreePhysicalMemory) / 1024; \
             $games = (Get-Process arma3_x64, arma3server_x64 -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Sum).Sum / 1MB; \
             Write-Output ([int]$used); Write-Output ([int]$games)",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut nums = text.lines().filter_map(|l| l.trim().parse::<u64>().ok());
    (nums.next().unwrap_or(0), nums.next().unwrap_or(0))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_01_spk_03_dedicated_server_client_headless() {
    let e2e = plain(repo().join("e2e"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup("dedicated").unwrap().clone();
    let mods = setup.mods.iter().map(|m| project.mods[m].clone()).collect();
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods,
        base_port: project.base_port,
    };
    let addon = plain(repo().join("target").join("shakedown").join("@shakedown"));
    let runner = Runner::bind(addon, None).await.unwrap();
    let started = Instant::now();
    let session = session::start(&runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    let view = session.view();
    println!(
        "start: {} in {:.1} s; reason {}",
        view["state"],
        started.elapsed().as_secs_f64(),
        view["reason"]
    );
    if view["state"] != "ready" {
        panic!("the session did not start: {view:#}");
    }
    let (used, games) = memory();
    println!("memory with all four up: {used} MB in use on the machine, {games} MB in game processes");

    // What the server's getUserInfo holds for each user, for INT-AR-09.
    let users = session
        .exec(
            "server",
            "allUsers apply { (getUserInfo _x) select [0, 9] }",
            json!([]),
            10.0,
        )
        .await
        .unwrap();
    println!("getUserInfo on the server: {}", users["value"]);

    let mut peak = used;
    for m in view["machines"].as_array().unwrap() {
        let name = m["name"].as_str().unwrap();
        let pid = m["pid"].as_u64().unwrap();
        // SPK-03: the hello was accepted, so the extension read the four variables; no token on
        // the command line.
        let cmdline = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine"),
            ])
            .output()
            .unwrap();
        let cmdline = String::from_utf8_lossy(&cmdline.stdout).to_string();
        assert!(cmdline.contains(&format!("-name={name}")), "{cmdline}");
        assert!(!cmdline.contains("skd_s_") && !cmdline.contains(&session.token));
        println!(
            "SPK-03 {name} ({}): hello accepted with the session token from the environment; no token on the command line",
            m["kind"]
        );

        // SPK-01: 100 sequential round trips of 1 + 1.
        let mut times = Vec::new();
        let (mut lost, mut wrong) = (0, 0);
        for _ in 0..100 {
            let t = Instant::now();
            match session.exec(name, "1 + 1", json!([]), 10.0).await {
                Ok(a) if a["value"] == json!(2) => times.push(t.elapsed().as_secs_f64() * 1000.0),
                Ok(a) => {
                    wrong += 1;
                    println!("wrong: {a}");
                }
                Err(e) => {
                    lost += 1;
                    println!("lost: {e:?}");
                }
            }
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let pct = |p: f64| {
            times[((times.len() as f64 * p).ceil() as usize)
                .saturating_sub(1)
                .min(times.len() - 1)]
        };
        println!(
            "SPK-01 {name}: {} answered 2, {lost} lost, {wrong} wrong; ms min {:.1} median {:.1} p95 {:.1} max {:.1}",
            times.len(),
            times[0],
            pct(0.5),
            pct(0.95),
            times[times.len() - 1]
        );
        assert_eq!(times.len(), 100);
        assert!(pct(0.95) < 1000.0);
        peak = peak.max(memory().0);
    }

    // No 301 warning in any RPT.
    for m in view["machines"].as_array().unwrap() {
        let profile = PathBuf::from(m["profile"].as_str().unwrap());
        let mut warnings = 0;
        for entry in std::fs::read_dir(&profile).unwrap().flatten() {
            if entry.path().extension().is_some_and(|e| e == "rpt") {
                let text = String::from_utf8_lossy(&std::fs::read(entry.path()).unwrap()).to_string();
                warnings += text.lines().filter(|l| l.contains("301")).count();
                for l in text.lines().filter(|l| l.contains("shakedown_x64")) {
                    println!("{} RPT: {l}", m["name"]);
                }
            }
        }
        println!("{}: {warnings} RPT lines naming 301", m["name"]);
    }
    println!("peak memory in use seen: {peak} MB");
    session.stop(&runner).await;
    println!("stopped: {}", session.view()["state"]);
}
