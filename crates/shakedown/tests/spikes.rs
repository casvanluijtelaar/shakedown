//! Spikes SPK-01, SPK-02 and SPK-03 for singleplayer `p1` (`spec/06-open-questions.md`). They
//! start the game, so they run only on the test machine, by hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes -- --ignored --nocapture
//!
//! The installed `@shakedown` is taken from `target/shakedown`. Results go to `research/`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use shakedown::config;
use shakedown::runner::{Runner, session};
use shakedown::wire::SetupSpec;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

async fn start() -> (std::sync::Arc<Runner>, std::sync::Arc<session::Session>) {
    let e2e = std::fs::canonicalize(repo().join("e2e")).unwrap();
    let e2e = PathBuf::from(e2e.to_string_lossy().trim_start_matches(r"\\?\"));
    let project = config::load(&e2e).unwrap();
    let setup = project.setup("sp").unwrap().clone();
    let spec = SetupSpec {
        setup,
        game: project.game.clone(),
        mods: vec![],
        base_port: project.base_port,
    };
    let addon = std::fs::canonicalize(repo().join("target").join("shakedown").join("@shakedown")).unwrap();
    let addon = PathBuf::from(addon.to_string_lossy().trim_start_matches(r"\\?\"));
    let runner = Runner::bind(addon, None).await.unwrap();
    let session = session::start(&runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    (runner, session)
}

fn rpt_text(session: &session::Session) -> String {
    let profile = PathBuf::from(session.view()["machines"][0]["profile"].as_str().unwrap());
    let mut text = String::new();
    for entry in std::fs::read_dir(profile).unwrap().flatten() {
        if entry.path().extension().is_some_and(|e| e == "rpt") {
            text.push_str(&String::from_utf8_lossy(&std::fs::read(entry.path()).unwrap()));
        }
    }
    text
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_01_spk_02_spk_03_singleplayer_p1() {
    let (runner, session) = start().await;
    let pid = session.view()["machines"][0]["pid"].as_u64().unwrap();

    // SPK-03: the hello was accepted, so the extension read the four variables. The token must not
    // be on the game's command line.
    let cmdline = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine"),
        ])
        .output()
        .unwrap();
    let cmdline = String::from_utf8_lossy(&cmdline.stdout).to_string();
    println!("SPK-03 p1 command line: {}", cmdline.trim());
    assert!(cmdline.contains("-name=p1"), "the command line was read");
    assert!(
        !cmdline.contains(&session.token) && !cmdline.contains("skd_s_"),
        "no token on the command line"
    );
    println!("SPK-03 p1: hello accepted with the session token from the environment; no token on the command line");

    // SPK-01: 100 sequential round trips of 1 + 1.
    let mut times = Vec::new();
    let mut lost = 0;
    let mut wrong = 0;
    for _ in 0..100 {
        let t = Instant::now();
        match session.exec("p1", "1 + 1", json!([]), 10.0).await {
            Ok(answer) if answer["value"] == json!(2) => times.push(t.elapsed().as_secs_f64() * 1000.0),
            Ok(answer) => {
                wrong += 1;
                println!("wrong answer: {answer}");
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
        "SPK-01 p1: {} answered 2, {lost} lost, {wrong} wrong; ms min {:.1} median {:.1} p95 {:.1} max {:.1}",
        times.len(),
        times[0],
        pct(0.5),
        pct(0.95),
        times[times.len() - 1]
    );

    // SPK-02: frame times while a 1 MB order and a 1 MB result move.
    let install = r#"
        skd_spk_max = 0; skd_spk_last = diag_tickTime;
        if (isNil "skd_spk_eh") then {
            skd_spk_eh = addMissionEventHandler ["EachFrame", {
                private _t = diag_tickTime; skd_spk_max = skd_spk_max max (_t - skd_spk_last); skd_spk_last = _t;
            }];
        };
        true"#;
    session.exec("p1", install, json!([]), 10.0).await.unwrap();
    let max_frame = async || {
        let v = session
            .exec("p1", "private _m = skd_spk_max; skd_spk_max = 0; _m", json!([]), 10.0)
            .await
            .unwrap();
        v["value"].as_f64().unwrap() * 1000.0
    };
    let _ = max_frame().await;
    let baseline = session.exec("p1", "sleep 3; true", json!([]), 10.0).await.unwrap();
    assert_eq!(baseline["value"], json!(true));
    let baseline_ms = max_frame().await;

    // A 1 MB order through fetch in parts: the order carries a 1 MB string, with quotes in it,
    // as an argument, so the game parses it but compiles only a short source.
    let unit = "abc\"de";
    let repeats = 1_048_576 / unit.len() + 1;
    let big = unit.repeat(repeats);
    let t = Instant::now();
    let answer = session
        .exec(
            "p1",
            "private _s = _this select 0; [(count _s) toFixed 0, ({_x == 34} count toArray _s) toFixed 0, _s select [0, 12], _s select [count _s - 12]]",
            json!([big]),
            30.0,
        )
        .await;
    let order_ms = t.elapsed().as_secs_f64() * 1000.0;
    let order_frame_ms = max_frame().await;
    let got = answer.as_ref().map(|a| a["value"].clone()).unwrap_or(Value::Null);
    let quotes = big.matches('"').count();
    let order_intact = got
        == json!([
            big.len().to_string(),
            quotes.to_string(),
            &big[..12],
            &big[big.len() - 12..]
        ]);
    println!(
        "SPK-02 1 MB order ({} bytes of argument, {quotes} quotes): game read {got}, intact {order_intact}, in {order_ms:.0} ms; max frame {order_frame_ms:.1} ms (baseline {baseline_ms:.1} ms)",
        big.len()
    );

    // A 1 MB check-like source: 1 MB of short statements, fetched in parts and compiled.
    let line = "_a = _a + 1;\n";
    let lines = 1_048_576 / line.len() + 1;
    let source = format!("private _a = 0;\n{}_a", line.repeat(lines));
    let t = Instant::now();
    let answer = session.exec("p1", &source, json!([]), 50.0).await;
    let source_ms = t.elapsed().as_secs_f64() * 1000.0;
    let source_frame_ms = max_frame().await;
    let source_value = answer.as_ref().map(|a| a["value"].clone()).unwrap_or(Value::Null);
    println!(
        "SPK-02 1 MB source ({} bytes, {lines} statements): value {source_value} (expected {lines}) in {source_ms:.0} ms; max frame {source_frame_ms:.1} ms",
        source.len()
    );

    // A 1 MB result: a string built by doubling, 6 * 2^18 = 1572864 characters.
    let t = Instant::now();
    let answer = session
        .exec(
            "p1",
            "private _s = \"abc\"\"de\"; for \"_i\" from 1 to 18 do { _s = _s + _s }; _s",
            json!([]),
            30.0,
        )
        .await
        .unwrap();
    let result_ms = t.elapsed().as_secs_f64() * 1000.0;
    let result_frame_ms = max_frame().await;
    let value = answer["value"].as_str().unwrap_or_default();
    let intact = value == unit.repeat(1 << 18);
    println!(
        "SPK-02 1 MB result: {} characters, intact {intact}, in {result_ms:.0} ms; max frame {result_frame_ms:.1} ms",
        value.len()
    );

    session.stop(&runner).await;
    let rpt = rpt_text(&session);
    let warnings: Vec<&str> = rpt
        .lines()
        .filter(|l| l.contains("301") || l.to_lowercase().contains("callextension"))
        .collect();
    println!("RPT lines naming 301 or callExtension: {warnings:#?}");

    assert_eq!(times.len(), 100, "SPK-01: every round trip answered 2");
    assert!(pct(0.95) < 1000.0, "SPK-01: p95 under 1 s");
    assert!(intact, "SPK-02: the result arrived intact");
    assert!(order_intact, "SPK-02: the order arrived intact");
    let _: Value = Value::Null;
    let _ = Duration::ZERO;
}

/// SPK-02: where the time of a large order goes inside the game, by size: fromJSON of the order,
/// compile of a source holding a long string literal, and toJSON of a long result.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_02_cost_by_size() {
    let (runner, session) = start().await;
    let probe = r#"
        params ["_n"];
        private _s = "abc""de"; for "_i" from 1 to _n do { _s = _s + _s };
        private _json = toJSON createHashMapFromArray [["source", _s]];
        private _t0 = diag_tickTime; private _h = fromJSON _json;
        private _t1 = diag_tickTime; private _c = compile ("private _x = " + str _s + "; count _x");
        private _t2 = diag_tickTime; private _j = toJSON [_s];
        private _t3 = diag_tickTime; private _p = []; for "_i" from 0 to (count _json) - 1 step 10000 do { _p pushBack (_json select [_i, 10000]) };
        private _t4 = diag_tickTime; private _joined = _p joinString "";
        private _t5 = diag_tickTime;
        [count _json, _t1 - _t0, _t2 - _t1, _t3 - _t2, _t4 - _t3, _t5 - _t4]"#;
    for n in [10, 12, 14, 15, 16] {
        match session.exec("p1", probe, json!([n]), 60.0).await {
            Ok(a) => {
                let v = &a["value"];
                println!(
                    "SPK-02 cost: json {} bytes: fromJSON {:.3} s, compile {:.3} s, toJSON {:.3} s, split {:.3} s, join {:.3} s",
                    v[0],
                    v[1].as_f64().unwrap_or(-1.0),
                    v[2].as_f64().unwrap_or(-1.0),
                    v[3].as_f64().unwrap_or(-1.0),
                    v[4].as_f64().unwrap_or(-1.0),
                    v[5].as_f64().unwrap_or(-1.0)
                );
                if v[1].as_f64().unwrap_or(0.0) + v[2].as_f64().unwrap_or(0.0) > 5.0 {
                    break;
                }
            }
            Err(e) => {
                println!("SPK-02 cost: n={n} no answer: {e:?}");
                break;
            }
        }
    }
    session.stop(&runner).await;
}
