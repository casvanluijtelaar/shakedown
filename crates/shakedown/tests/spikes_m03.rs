//! Spikes SPK-07 and SPK-13 (`spec/06-open-questions.md`), run before the M3 work that depends
//! on them: the `ScriptError` mission event on every machine kind and every kind of code, and
//! reading a running game's RPT. They start the game, so they run only on the test machine, by
//! hand:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test spikes_m03 -- --ignored --nocapture --test-threads=1
//!
//! The installed `@shakedown` and fixtures are taken from `target/shakedown`. Results go to
//! `research/`. Nothing here prints a line of an RPT that is not a script error or a marker of
//! the spike's own, so no Steam ID or Steam name reaches the output.

use std::io::{Read, Seek, SeekFrom};
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

async fn start(setup: &str) -> (Arc<Runner>, Arc<session::Session>) {
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
    let addon = plain(repo().join("target").join("shakedown").join("@shakedown"));
    let runner = Runner::bind(addon, None).await.unwrap();
    let session = session::start(&runner, e2e, spec, &|m| println!("{m}")).await.unwrap();
    let view = session.view();
    assert_eq!(
        view["state"], "ready",
        "the session did not start: {}",
        view["reason"]["message"]
    );
    (runner, session)
}

async fn value(session: &session::Session, machine: &str, source: &str, args: Value) -> Value {
    match session.exec(machine, source, args, 10.0).await {
        Ok(a) if a["state"] == "done" => a["value"].clone(),
        Ok(a) => json!({"answer": a}),
        Err(e) => json!({"no-answer": format!("{e:?}")}),
    }
}

/// The newest RPT in a machine's profile folder.
fn rpt(profile: &Path) -> PathBuf {
    std::fs::read_dir(profile)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("rpt")))
        .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
        .expect("an RPT")
}

/// Reads the RPT from `offset` on, opened anew each time as a reader beside the game would.
fn read_from(path: &Path, offset: u64) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    f.seek(SeekFrom::Start(offset)).unwrap();
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Waits up to `ceiling` for `needle` to appear in the RPT after `offset`; the time it took.
fn wait_for(path: &Path, offset: u64, needle: &str, ceiling: Duration) -> Option<Duration> {
    let t = Instant::now();
    for _ in 0..((ceiling.as_millis() / 10) as usize + 1) {
        if read_from(path, offset).contains(needle) {
            return Some(t.elapsed());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    None
}

/// The lines of an RPT text that belong to script errors: a line holding `Error` after its
/// timestamp, the indented `Error position:` and `File` lines, and their continuation lines.
fn error_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| l.contains("Error") || l.contains(" File ") || l.trim_start().starts_with("File "))
        .map(String::from)
        .collect()
}

const HANDLER: &str = r#"
if (isNil "skd_spk07") then {
    skd_spk07 = [];
    addMissionEventHandler ["ScriptError", {
        params ["_text", "_file", "_line", "_pos", "_content", "_stack"];
        skd_spk07 pushBack [_text, _file, _line, _pos, str _stack, diag_tickTime, canSuspend];
    }];
};
skd_spk07 = [];
clientOwner
"#;

/// SPK-07 on the machines of a session: a deliberate error in unscheduled code, spawned code, a
/// `remoteExec`'d function and spawned inline code on each; what the handler caught and what the
/// RPT shows. Returns the count of RPT errors the handler missed, over every machine.
async fn spk_07(session: &session::Session, lead: &str) -> usize {
    let view = session.view();
    let machines: Vec<(String, PathBuf)> = view["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["name"].as_str().unwrap().to_string(),
                PathBuf::from(m["profile"].as_str().unwrap()),
            )
        })
        .collect();
    let mut owners = std::collections::BTreeMap::new();
    for (name, _) in &machines {
        let owner = value(session, name, HANDLER, json!([])).await;
        println!("SPK-07 {name}: handler installed, clientOwner {owner}");
        owners.insert(name.clone(), owner);
    }
    let mut missed = 0;
    for (name, profile) in &machines {
        let path = rpt(profile);
        let marker = format!("skd-spk07-start-{name}");
        value(session, name, &format!("diag_log text {marker:?}; true"), json!([])).await;
        wait_for(&path, 0, &marker, Duration::from_secs(5)).expect("the start marker");
        let offset = std::fs::metadata(&path).unwrap().len();

        // Each kind of code in turn; the handler's count after each tells which ones raised it.
        let kinds: [(&str, &str, bool); 9] = [
            // isNil runs its code unscheduled, and also hides an undefined variable.
            ("unscheduled in isNil", "isNil { [] call skdfx_fnc_throw }; true", false),
            // A one-shot EachFrame handler runs its code unscheduled.
            (
                "unscheduled in EachFrame",
                r#"skd_spk07_eh = addMissionEventHandler ["EachFrame", { removeMissionEventHandler ["EachFrame", skd_spk07_eh]; [] call skdfx_fnc_throw }]; true"#,
                false,
            ),
            ("spawned", "[] spawn skdfx_fnc_throw; true", false),
            (
                "remoteExec from the lead",
                r#"[] remoteExec ["skdfx_fnc_throw", _this select 0]; true"#,
                true,
            ),
            (
                "remoteExecCall from the lead",
                r#"[] remoteExecCall ["skdfx_fnc_throw", _this select 0]; true"#,
                true,
            ),
            (
                "spawned inline code",
                "[] spawn { private _a = _skd_spk07_inline + 1; }; true",
                false,
            ),
            // The engine reports no undefined variable in unscheduled code; a type error it does.
            (
                "unscheduled type error in EachFrame",
                r#"skd_spk07_eh = addMissionEventHandler ["EachFrame", { removeMissionEventHandler ["EachFrame", skd_spk07_eh]; private _a = "skd" + 1 }]; true"#,
                false,
            ),
            (
                "unscheduled type error in isNil",
                r#"isNil { private _a = "skd" + 1 }; true"#,
                false,
            ),
            (
                "remoteExecCall of a type error from the lead",
                r#"[[], { private _a = "skd" + 1 }] remoteExecCall ["call", _this select 0]; true"#,
                true,
            ),
        ];
        let mut caught = json!([]);
        let mut before = 0;
        for (kind, source, from_lead) in kinds {
            let on = if from_lead { lead } else { name.as_str() };
            value(session, on, source, json!([owners[name]])).await;
            let t = Instant::now();
            for _ in 0..40 {
                caught = value(session, name, "skd_spk07", json!([])).await;
                if caught.as_array().is_some_and(|a| a.len() > before) || t.elapsed() > Duration::from_secs(2) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            let now = caught.as_array().map_or(0, Vec::len);
            println!("SPK-07 {name}: {kind}: the handler caught {}", now - before);
            before = now;
        }
        let end = format!("skd-spk07-end-{name}");
        value(session, name, &format!("diag_log text {end:?}; true"), json!([])).await;
        wait_for(&path, offset, &end, Duration::from_secs(5)).expect("the end marker");
        let text = read_from(&path, offset);
        let lines = error_lines(&text);
        let rpt_errors = shakedown::logs::scan_errors(&text).len();
        let handled = caught.as_array().map_or(0, Vec::len);
        println!("SPK-07 {name}: the handler caught {handled}; the RPT shows {rpt_errors} 'Error in expression'");
        for c in caught.as_array().into_iter().flatten() {
            println!("  handler: {c}");
        }
        for l in &lines {
            println!("  rpt: {l}");
        }
        missed += rpt_errors.saturating_sub(handled);
    }
    missed
}

/// SPK-13 on one machine: 100 `diag_log` lines, each looked for in the RPT by a reader that
/// opens it anew, with a ceiling of 2 s. Returns the latencies, in ms, of the lines seen.
async fn spk_13(session: &session::Session, name: &str, profile: &Path) -> (usize, Vec<f64>) {
    let path = rpt(profile);
    let mut seen = Vec::new();
    let mut missed = 0;
    for n in 0..100 {
        let offset = std::fs::metadata(&path).unwrap().len();
        let line = format!("skd-spk13-{name}-{n}-x");
        let t = Instant::now();
        value(session, name, &format!("diag_log text {line:?}; true"), json!([])).await;
        let answered = t.elapsed();
        match wait_for(&path, offset, &line, Duration::from_secs(2)) {
            Some(d) => seen.push((answered + d).as_secs_f64() * 1000.0),
            None => missed += 1,
        }
    }
    (missed, seen)
}

fn summary(name: &str, missed: usize, mut times: Vec<f64>) {
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if times.is_empty() {
        println!("SPK-13 {name}: 0 of 100 lines seen within 2 s");
        return;
    }
    let pct = |p: f64| {
        times[((times.len() as f64 * p).ceil() as usize)
            .saturating_sub(1)
            .min(times.len() - 1)]
    };
    println!(
        "SPK-13 {name}: {} of 100 seen within 2 s, {missed} not; ms from the order to the reader seeing it: min {:.1} median {:.1} p95 {:.1} max {:.1}",
        times.len(),
        times[0],
        pct(0.5),
        pct(0.95),
        times[times.len() - 1]
    );
}

/// Lines of the RPT that a file error of the game would leave, such as a failed open or write.
fn file_errors(profile: &Path) -> Vec<String> {
    let text = read_from(&rpt(profile), 0);
    text.lines()
        .filter(|l| {
            let l = l.to_lowercase();
            (l.contains(".rpt") || l.contains("log file"))
                && (l.contains("cannot") || l.contains("error") || l.contains("fail"))
        })
        .map(String::from)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_07_spk_13_dedicated_server_client_headless() {
    let (runner, session) = start("dedicated").await;
    let missed = spk_07(&session, "server").await;
    println!("SPK-07 dedicated: RPT errors the handler missed: {missed}");
    for m in session.view()["machines"].as_array().unwrap() {
        let name = m["name"].as_str().unwrap();
        let profile = PathBuf::from(m["profile"].as_str().unwrap());
        let (missed, times) = spk_13(&session, name, &profile).await;
        summary(name, missed, times);
        println!(
            "SPK-13 {name}: RPT lines that name a file error: {:?}",
            file_errors(&profile)
        );
    }
    session.stop(&runner).await;
    println!("stopped: {}", session.view()["state"]);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn spk_07_spk_13_singleplayer_p1() {
    let (runner, session) = start("sp-fixture").await;
    let missed = spk_07(&session, "p1").await;
    println!("SPK-07 singleplayer: RPT errors the handler missed: {missed}");
    let profile = PathBuf::from(session.view()["machines"][0]["profile"].as_str().unwrap());
    let (missed, times) = spk_13(&session, "p1", &profile).await;
    summary("p1", missed, times);
    println!(
        "SPK-13 p1: RPT lines that name a file error: {:?}",
        file_errors(&profile)
    );
    session.stop(&runner).await;
    println!("stopped: {}", session.view()["state"]);
}

/// Which ways of taking the value of called code whose last statement is an assignment, which
/// returns nothing, make the engine report `GIAS pre stack size violation`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts Arma 3; run on the test machine"]
async fn engine_fact_the_value_of_code_ending_in_an_assignment() {
    let (runner, session) = start("sp-fixture").await;
    value(&session, "p1", HANDLER, json!([])).await;
    let forms = [
        "private _r = [] call { skd_a = 1 }; true",
        "private _r = [] call { skd_a = 1; nil }; true",
        "private _b = [nil]; _b set [0, [] call { skd_a = 1 }]; true",
        "private _r = [[] call { skd_a = 1 }] param [0, nil]; true",
        "private _c = { skd_a = 1 }; private _r = [] call _c; true",
        "private _c = { skd_a = 1 }; private _r = 7; isNil { _r = [] call _c }; isNil \"_r\"",
        "private _c = { skd_a = 1; 5 }; private _r = 7; isNil { _r = [] call _c }; _r",
        "private _c = { skd_a = 1 }; private _r = [0] apply { [] call _c }; true",
        "private _c = { skd_a = 1 }; private _r = switch (true) do { default { [] call _c } }; true",
        "private _c = { skd_a = 1 }; private _r = switch (true) do { default { [] call _c } }; isNil \"_r\"",
        "private _c = { 5 }; private _r = switch (true) do { default { [] call _c } }; _r",
        "private _c = { [1, nil] }; private _r = switch (true) do { default { [] call _c } }; _r",
        "private _c = { nil }; private _r = switch (true) do { default { [] call _c } }; isNil \"_r\"",
    ];
    for f in forms {
        value(&session, "p1", "skd_spk07 = []; true", json!([])).await;
        let answer = value(&session, "p1", f, json!([])).await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let caught = value(&session, "p1", "skd_spk07 apply { _x select 0 }", json!([])).await;
        println!("GIAS {f}\n  -> answer {answer}, errors {caught}");
    }
    session.stop(&runner).await;
}
