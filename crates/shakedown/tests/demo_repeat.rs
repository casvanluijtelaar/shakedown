//! M0 exit criterion 2: the demo five times in a row, each leaving nothing running. It takes
//! about nine minutes, so it is kept out of the end-to-end run and runs only by name, when a
//! milestone closes, on the test machine:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test demo_repeat -- --ignored --nocapture

use std::path::PathBuf;
use std::process::Command;

fn repo() -> PathBuf {
    let p = std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

fn shakedown() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn game_processes() -> String {
    let out = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq arma3_x64.exe", "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// M0 exit criterion 2 (`P-01`, `RUN-04`, `STP-02`): the demo five times in a row. After each run
/// no game process is left, and the runner exits by itself within 70 s of the session stopping.
#[test]
#[ignore = "starts Arma 3 five times; run by name when M0 closes"]
fn p_01_run_04_the_demo_five_times_leaves_nothing_running() {
    let log_path = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
        .join("Shakedown")
        .join("runner.log");
    let mut failures = Vec::new();
    for n in 1..=5 {
        let started = std::time::Instant::now();
        let out = Command::new(shakedown())
            .args(["run", "--setup", "sp", "smoke"])
            .current_dir(repo().join("e2e"))
            .output()
            .unwrap();
        let run_s = started.elapsed().as_secs_f64();
        let code = out.status.code().unwrap_or(-1);
        let games = game_processes();
        let game_left = games.contains("arma3_x64");
        // The runner's pid, from the log line of the runner that served this run.
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        let pid: u32 = log
            .lines()
            .rev()
            .find_map(|l| {
                l.split(" runner ")
                    .nth(1)
                    .and_then(|r| r.split(' ').next())
                    .and_then(|p| p.parse().ok())
            })
            .unwrap_or(0);
        let stopped_at = std::time::Instant::now();
        let mut exited_after = None;
        for _ in 0..90 {
            let out = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
                .output()
                .unwrap();
            if !String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\"")) {
                exited_after = Some(stopped_at.elapsed().as_secs_f64());
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        println!(
            "run {n}: exit {code} in {run_s:.1} s; arma3_x64 left: {}; runner {pid} exited {} after the command returned",
            if game_left { "yes" } else { "no" },
            exited_after
                .map(|s| format!("{s:.1} s"))
                .unwrap_or_else(|| "NOT within 90 s".into())
        );
        if code != 0 || game_left || exited_after.is_none_or(|s| s > 70.0) {
            failures.push(n);
        }
    }
    assert!(failures.is_empty(), "runs that missed the criterion: {failures:?}");
}
