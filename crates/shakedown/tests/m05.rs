//! M5's tests: a session that outlives its client (`US-007`, `SES-06`, `SES-08`, spike `SPK-06`),
//! a runner that dies (`US-035`, `NFR-RL-03`, `STP-03`, `P-12`), local control and the audit
//! (`US-052`, `NFR-SC-01`, `NFR-SC-03`, `SEC-04`), a process without the token (`US-053`,
//! `SEC-03`, `NFR-SC-02`) and a runner a client cannot reach (`US-060`, `P-01`). They drive the
//! installed `shakedown` through its command line, its MCP server and its port, and the game, so
//! they run only on the test machine, by hand, one at a time, with no runner running:
//!
//!     python tools/install.py
//!     cargo test -p shakedown --test m05 -- --ignored --nocapture --test-threads=1
//!
//! `spk_06` also starts headless Claude Code sessions (`claude -p`) in `e2e/grappling-hook`,
//! with that project's `.mcp.json` passed by `--mcp-config` and the install folder on `PATH`,
//! and needs a built arma-grappling-hook. Every process a test starts is ended by its id.

#![cfg(windows)]

mod support;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use regex::Regex;
use serde_json::{Value, json};
use support::{Mcp, repo};

const GAME: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Arma 3";

fn exe() -> PathBuf {
    let exe = repo().join("target").join("shakedown").join("shakedown.exe");
    assert!(exe.is_file(), "run `python tools/install.py` first");
    exe
}

fn cli(dir: &Path, args: &[&str]) -> (i32, String, String) {
    support::run(&exe(), dir, &[], args)
}

fn cli_json(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    support::run_json(&exe(), dir, &[], args)
}

fn runner_dir() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap()).join("Shakedown")
}

fn runner_file() -> Value {
    std::fs::read_to_string(runner_dir().join("runner.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
}

fn runner_log() -> String {
    std::fs::read_to_string(runner_dir().join("runner.log")).unwrap_or_default()
}

fn alive(pid: u32) -> bool {
    shakedown::win::process_started(pid).is_some()
}

/// Ends a process this test started, by its id; with `tree`, its children too.
fn kill(pid: u32, tree: bool) {
    let mut args = vec!["/PID".to_string(), pid.to_string(), "/F".into()];
    if tree {
        args.push("/T".into());
    }
    let _ = Command::new("taskkill").args(&args).output();
}

/// Polls `f` every 100 ms until it is true or `ceiling` passes; whether it became true.
fn within(ceiling: Duration, mut f: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    for _ in 0..100_000 {
        if f() {
            return true;
        }
        if start.elapsed() > ceiling {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

/// The ids of the running game processes.
fn game_pids() -> Vec<u32> {
    let mut out = Vec::new();
    for image in ["arma3_x64.exe", "arma3server_x64.exe"] {
        let o = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {image}"), "/FO", "CSV", "/NH"])
            .output()
            .unwrap();
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if let Some(pid) = line.split("\",\"").nth(1).and_then(|p| p.parse().ok()) {
                out.push(pid);
            }
        }
    }
    out
}

/// Every process's id, image name and command line.
fn processes() -> Vec<(u32, String, String)> {
    let o = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_Process | ForEach-Object { [pscustomobject]@{p=$_.ProcessId;n=$_.Name;c=$_.CommandLine} } | ConvertTo-Json -Compress",
        ])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
    v.as_array()
        .into_iter()
        .flatten()
        .map(|p| {
            (
                p["p"].as_u64().unwrap_or(0) as u32,
                p["n"].as_str().unwrap_or_default().to_string(),
                p["c"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

/// The ids of the `shakedown` processes whose command line ends in `word`.
fn shakedown_pids(word: &str) -> Vec<u32> {
    processes()
        .into_iter()
        .filter(|(_, n, c)| n.eq_ignore_ascii_case("shakedown.exe") && c.trim_end().ends_with(word))
        .map(|(p, _, _)| p)
        .collect()
}

fn free_mb() -> u64 {
    let o = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1024)",
        ])
        .output()
        .unwrap();
    String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(0)
}

/// No runner may run when a test starts: a test only ends processes it started.
fn assert_no_runner() {
    let pid = runner_file()["pid"].as_u64().map(|p| p as u32);
    assert!(
        !pid.is_some_and(alive),
        "a runner runs already (process {pid:?}); stop it first"
    );
    assert!(game_pids().is_empty(), "a game runs already");
}

/// Stops the session when dropped, also when the test panics, and ends by id every game process
/// and runner that started after it was made.
struct Guard {
    dir: PathBuf,
    games: Vec<u32>,
    runners: Vec<u32>,
}

impl Guard {
    fn new(dir: &Path) -> Guard {
        assert_no_runner();
        println!("free memory before: {} MB", free_mb());
        Guard {
            dir: dir.to_path_buf(),
            games: game_pids(),
            runners: shakedown_pids("runner"),
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        // A suspended runner is resumed so that its stop can run.
        if let Some(pid) = runner_file()["pid"].as_u64() {
            w::suspend(pid as u32, false);
        }
        let _ = cli(&self.dir, &["session", "stop"]);
        for pid in game_pids().into_iter().filter(|p| !self.games.contains(p)) {
            kill(pid, false);
        }
        for pid in shakedown_pids("runner")
            .into_iter()
            .filter(|p| !self.runners.contains(p))
        {
            kill(pid, false);
        }
        println!("free memory after: {} MB", free_mb());
    }
}

/// A project with the fixture mod: `ded`, a dedicated server with a client and a headless client.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("skd-m05-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("checks")).unwrap();
    std::fs::write(
        dir.join("shakedown.toml"),
        format!(
            r#"game = '{GAME}'

[mods.fixture]
path = '{GAME}\@skd_fixture'
patch = "skdfx_main"

[setups.ded]
mode = "dedicated"
clients = 1
headless = 1
mods = ["fixture"]
"#
        ),
    )
    .unwrap();
    let checks = [
        (
            "US-052-on-p1",
            "// @check US-052 code a check sends to p1 is in the audit under the check's id\n// @setups ded\n// @timeout 60\n\nprivate _v = [\"p1\", { 40 + 2 }] call skd_fnc_on;\n[_v, \"==\", 42, \"p1 answered\"] call skd_fnc_expect;\n",
        ),
        (
            "US-035-long",
            "// @check US-035 a check that is still running when the runner is killed\n// @setups ded\n// @timeout 300\n\n[\"p1\", { sleep 200; true }, [], 250] call skd_fnc_on;\n",
        ),
    ];
    for (id, text) in checks {
        std::fs::write(dir.join("checks").join(format!("{id}.sqf")), text).unwrap();
    }
    dir
}

fn start(dir: &Path) -> Value {
    let (code, out, err) = cli_json(dir, &["session", "start", "--setup", "ded"]);
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(out["state"], "ready", "{out}");
    out
}

fn status(dir: &Path) -> Value {
    let (code, out, err) = cli_json(dir, &["status"]);
    assert_eq!(code, 0, "{err}");
    out
}

fn mission_copy(session: &Value) -> PathBuf {
    Path::new(GAME)
        .join("MPMissions")
        .join(format!("shakedown_{}.VR", session["id"].as_str().unwrap()))
}

/// `SEC-06`'s pattern of a token.
fn token_pattern() -> Regex {
    Regex::new("skd_[rs]_[0-9a-f]{64}").unwrap()
}

/// Every file under `dir`, read as text.
fn files_text(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut todo = vec![dir.to_path_buf()];
    while let Some(d) = todo.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                todo.push(p);
            } else if let Ok(b) = std::fs::read(&p) {
                out.push((p, String::from_utf8_lossy(&b).into_owned()));
            }
        }
    }
    out
}

/// What the tests need of Windows that the framework does not offer: a job object around a
/// process, suspending a process, and reading another process's environment.
mod w {
    use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessBasicInformation};
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation, SetInformationJobObject,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, OpenThread, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE, PROCESS_VM_READ,
        ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME,
    };
    use windows::core::PCWSTR;

    pub struct Job(HANDLE);

    impl Job {
        /// A job with kill-on-close, as a host such as Claude Code holds its servers in, with or
        /// without breakaway allowed.
        pub fn new(breakaway: bool) -> Job {
            unsafe {
                let job = CreateJobObjectW(None, PCWSTR::null()).unwrap();
                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if breakaway {
                    info.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_BREAKAWAY_OK;
                }
                SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
                .unwrap();
                Job(job)
            }
        }

        pub fn assign(&self, pid: u32) {
            unsafe {
                let p = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid).unwrap();
                AssignProcessToJobObject(self.0, p).unwrap();
                let _ = CloseHandle(p);
            }
        }

        /// Closes the job, which ends every process in it.
        pub fn close(self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    /// A handle on a process that keeps its exit code readable after it ends.
    pub struct Watch(HANDLE);

    impl Watch {
        pub fn new(pid: u32) -> Option<Watch> {
            use windows::Win32::System::Threading::{PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE};
            unsafe {
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE, false, pid)
                    .ok()
                    .map(Watch)
            }
        }

        /// The exit code once the process ended; `None` while it runs.
        pub fn exit_code(&self) -> Option<u32> {
            use windows::Win32::System::Threading::GetExitCodeProcess;
            let mut code = 0u32;
            unsafe { GetExitCodeProcess(self.0, &mut code).ok()? };
            (code != 259).then_some(code)
        }
    }

    impl Drop for Watch {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    /// Suspends, or resumes, every thread of a process.
    pub fn suspend(pid: u32, on: bool) {
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

    fn read<T: Default>(p: HANDLE, at: usize) -> Option<T> {
        let mut v = T::default();
        unsafe {
            ReadProcessMemory(
                p,
                at as *const _,
                &mut v as *mut T as *mut _,
                std::mem::size_of::<T>(),
                None,
            )
            .ok()?;
        }
        Some(v)
    }

    /// A process's environment, read from its process parameters (64-bit layout).
    pub fn environment(pid: u32) -> Option<Vec<(String, String)>> {
        unsafe {
            let p = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
            let mut basic = [0usize; 6];
            let mut len = 0u32;
            let status = NtQueryInformationProcess(
                p,
                ProcessBasicInformation,
                basic.as_mut_ptr() as *mut _,
                std::mem::size_of_val(&basic) as u32,
                &mut len,
            );
            let out = (|| {
                if status.is_err() {
                    return None;
                }
                let params: usize = read(p, basic[1] + 0x20)?;
                let env: usize = read(p, params + 0x80)?;
                let size: usize = read(p, params + 0x3F0)?;
                let mut buf = vec![0u16; size / 2];
                ReadProcessMemory(p, env as *const _, buf.as_mut_ptr() as *mut _, size, None).ok()?;
                Some(
                    String::from_utf16_lossy(&buf)
                        .split('\0')
                        .filter_map(|kv| kv.split_once('='))
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                )
            })();
            let _ = CloseHandle(p);
            out
        }
    }
}

/// Sends one line on a new connection to the runner's port and reads its answer: `None` when the
/// runner closed the connection without one.
fn hello(port: u16, line: &Value) -> Option<String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    writeln!(s, "{line}").unwrap();
    // A message after the hello, as a game process would send; the runner must not take it.
    let _ = writeln!(s, "{}", json!({"kind": "ready", "mission_start": 99}));
    let mut answer = String::new();
    match BufReader::new(s).read_line(&mut answer) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(answer),
    }
}

/// `SES-08`, `US-007`: the runner that `shakedown mcp` starts outlives it, however its host ends
/// it: by closing a kill-on-close job that holds it, without and with breakaway allowed, and by
/// ending each process of its tree. Needs no game.
#[test]
#[ignore = "starts runners on this machine; run by hand"]
fn ses_08_us_007_the_runner_outlives_the_host_ending_shakedown_mcp() {
    assert_no_runner();
    let dir = project("ses08");
    for case in ["job without breakaway", "job with breakaway", "tree kill"] {
        let mark = runner_log().len();
        let mut mcp = Mcp::start(&exe(), &dir, &[]);
        let job = match case {
            "job without breakaway" => Some(w::Job::new(false)),
            "job with breakaway" => Some(w::Job::new(true)),
            _ => None,
        };
        if let Some(j) = &job {
            j.assign(mcp.pid());
        }
        let a = mcp.call("session_stop", json!({}));
        assert!(a.text().starts_with("no-session: "), "{case}: {}", a.text());
        let runner = runner_file()["pid"].as_u64().unwrap() as u32;
        // A job that forbids breakaway leaves the client only WMI. Otherwise the client starts
        // the relay, which starts the runner by WMI when a job around the client's still holds
        // it; a job around this test itself that forbids breakaway sends the client to WMI too.
        let log = runner_log()[mark..].to_string();
        let way: Vec<&str> = log
            .lines()
            .filter_map(|l| l.split("started the runner by ").nth(1))
            .collect();
        println!("{case}: started by {way:?}");
        if case == "job without breakaway" {
            assert_eq!(way, ["wmi"], "{log}");
        } else {
            assert!(!way.is_empty(), "{case}: {log}");
        }
        let server = mcp.pid();
        match job {
            Some(j) => j.close(),
            None => kill(server, true),
        }
        assert!(
            within(Duration::from_secs(10), || !alive(server)),
            "{case}: mcp still runs"
        );
        let s = status(&dir);
        println!(
            "{case}: runner {runner}, after the host ended shakedown mcp: {}",
            s["runner"]
        );
        assert!(alive(runner), "{case}: the runner ended with its client");
        assert_eq!(s["runner"], "running", "{case}: {s}");
        assert_eq!(s["runner_pid"], runner, "{case}: {s}");
        drop(mcp);
        kill(runner, false);
        assert!(within(Duration::from_secs(10), || !alive(runner)));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Runs a headless Claude Code session in `e2e/grappling-hook` whose only tool is `tool`, with
/// the install folder on `PATH`, ended by its id after `ceiling`; its reply.
fn claude(prompt: &str, tool: &str, ceiling: Duration) -> String {
    let dir = repo().join("e2e").join("grappling-hook");
    let path = format!(
        "{};{}",
        exe().parent().unwrap().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out_file = std::env::temp_dir().join(format!("skd-m05-claude-{}.txt", std::process::id()));
    let mut child = Command::new("claude")
        .args([
            "-p",
            prompt,
            "--mcp-config",
            ".mcp.json",
            "--strict-mcp-config",
            "--allowedTools",
            tool,
        ])
        .env("PATH", path)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&out_file).unwrap())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let ended = within(ceiling, || child.try_wait().ok().flatten().is_some());
    if !ended {
        kill(pid, true);
        let _ = child.wait();
    }
    let reply = std::fs::read_to_string(&out_file).unwrap_or_default();
    let _ = std::fs::remove_file(&out_file);
    assert!(ended, "claude did not end within {ceiling:?}: {reply}");
    reply.trim().to_string()
}

/// `SPK-06`, `US-007` criterion 1 and the M5 demo: an MCP call from a Claude Code session starts
/// a dedicated session; that Claude Code session ends; a new one finds the session `ready`; the
/// M2 check runs against it; the runner is killed with `taskkill /F` and no game process is left
/// within 10 s; the next runner start clears the mission copy and marks the session `runner-died`.
#[test]
#[ignore = "starts Claude Code and the game; run by hand on the test machine"]
fn spk_06_us_007_a_session_started_through_claude_code_outlives_it() {
    let dir = repo().join("e2e").join("grappling-hook");
    let _guard = Guard::new(&dir);
    let reply = claude(
        "Call the tool mcp__shakedown__session_start once with the argument setup set to \"dedicated\". Then reply with only the session id it returned.",
        "mcp__shakedown__session_start",
        Duration::from_secs(300),
    );
    println!("first Claude Code session replied: {reply}");
    // The first Claude Code session and its MCP server have ended; the runner has not.
    assert!(
        within(Duration::from_secs(10), || shakedown_pids("mcp").is_empty()),
        "shakedown mcp outlived its Claude Code session"
    );
    let runner = runner_file()["pid"].as_u64().expect("a runner runs") as u32;
    assert!(alive(runner));
    assert!(
        within(Duration::from_secs(300), || status(&dir)["session"]["state"] == "ready"),
        "the session did not become ready: {}",
        status(&dir)
    );
    let session = status(&dir)["session"].clone();
    let id = session["id"].as_str().unwrap().to_string();
    assert!(
        reply.contains(&id),
        "the first session's reply names another session: {reply}"
    );
    let reply = claude(
        "Call the tool mcp__shakedown__status once. Reply with only the session's id and the session's state, separated by one space.",
        "mcp__shakedown__status",
        Duration::from_secs(180),
    );
    println!("second Claude Code session replied: {reply}");
    assert!(
        reply.contains(&id) && reply.contains("ready"),
        "a new Claude Code session did not find {id} ready: {reply}"
    );
    // The M2 check against the session the first Claude Code session started.
    let (code, run, err) = cli_json(&dir, &["run", "US-009-dedicated-freed-rope"]);
    println!("the M2 check: exit {code}, {}", run["summary"]);
    let result: Value =
        serde_json::from_str(&std::fs::read_to_string(run["result_file"].as_str().expect(&err)).unwrap()).unwrap();
    assert_eq!(result["session"]["id"], id.as_str());
    let pids: Vec<u32> = status(&dir)["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["pid"].as_u64().unwrap() as u32)
        .collect();
    let killed = Instant::now();
    kill(runner, false);
    assert!(
        within(Duration::from_secs(10), || pids.iter().all(|p| !alive(*p))),
        "game processes left 10 s after the runner was killed"
    );
    println!(
        "every game process ended {:.1} s after the kill",
        killed.elapsed().as_secs_f64()
    );
    let copy = mission_copy(&session);
    assert!(copy.exists(), "the mission copy is there until the next runner start");
    // The next command starts a runner, whose recovery clears what the record lists (P-12).
    let (code, _, _) = cli(&dir, &["session", "stop"]);
    assert_eq!(code, 4);
    assert!(!copy.exists(), "the next runner start left the mission copy");
    let end: Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(session["folder"].as_str().unwrap()).join("session.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(end["reason"], "runner-died");
}

/// `US-035`, `NFR-RL-03`, `STP-03`, `P-12`: the runner killed with `taskkill /F` during a run on a
/// dedicated session; no game process of the session is left within 10 s, 5 times out of 5; the
/// next `status` shows no runner and no session, and the next `session start` succeeds, with no
/// mission copy of the dead session left and the dead session marked `runner-died`.
#[test]
#[ignore = "starts the game; run by hand on the test machine"]
fn us_035_nfr_rl_03_the_runner_killed_during_a_run_leaves_no_game_5_of_5() {
    let dir = project("us035");
    let _guard = Guard::new(&dir);
    let mut dead: Option<Value> = None;
    let mut times = Vec::new();
    for round in 1..=5 {
        let session = start(&dir);
        if let Some(d) = &dead {
            assert!(
                !mission_copy(d).exists(),
                "round {round}: the dead session's mission copy is left"
            );
            let end: Value = serde_json::from_str(
                &std::fs::read_to_string(PathBuf::from(d["folder"].as_str().unwrap()).join("session.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(end["state"], "stopped");
            assert_eq!(end["reason"], "runner-died");
        }
        let runner = status(&dir)["runner_pid"].as_u64().unwrap() as u32;
        let pids: Vec<u32> = session["machines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["pid"].as_u64().unwrap() as u32)
            .collect();
        assert_eq!(pids.len(), 3);
        let mut run = Command::new(exe())
            .args(["run", "US-035-long"])
            .current_dir(&dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        assert!(
            within(Duration::from_secs(60), || status(&dir)["run"]["current_check"]
                == "US-035-long"),
            "round {round}: the run did not begin: {}",
            status(&dir)
        );
        let killed = Instant::now();
        kill(runner, false);
        let gone = within(Duration::from_secs(10), || pids.iter().all(|p| !alive(*p)));
        let took = killed.elapsed().as_secs_f64();
        assert!(gone, "round {round}: game processes left 10 s after the kill");
        times.push(took);
        println!("round {round}: every game process ended {took:.1} s after the kill");
        if !within(Duration::from_secs(10), || run.try_wait().ok().flatten().is_some()) {
            kill(run.id(), false);
        }
        let _ = run.wait();
        let s = status(&dir);
        assert_eq!(s["runner"], "not-running", "round {round}: {s}");
        assert!(s["session"].is_null(), "round {round}: {s}");
        dead = Some(session);
    }
    // The last dead session is cleared by the next start, which then stops cleanly.
    let session = start(&dir);
    let d = dead.unwrap();
    assert!(!mission_copy(&d).exists());
    let (code, _, err) = cli(&dir, &["session", "stop"]);
    assert_eq!(code, 0, "{err}");
    assert!(!mission_copy(&session).exists());
    println!("NFR-RL-03: 5 of 5, times {times:?}");
}

/// A runner started the way this machine starts it when breakaway is refused, through WMI
/// (`INT-WIN-02`), lives through its session's start, a run and the next runner's recovery,
/// round after round (`US-007`, `US-035`, `NFR-RL-03`). Each round starts the runner by WMI,
/// holds a handle on it, starts a dedicated session, runs a check that is still running when the
/// test kills the runner, and checks that the runner ended only by that kill: a runner that ends
/// before it fails the test with its exit code. `SKD_ROUNDS` sets the rounds, 10 by default.
#[test]
#[ignore = "starts the game; run by hand on the test machine"]
fn us_035_nfr_rl_03_a_runner_started_by_wmi_ends_only_when_killed() {
    let rounds: u32 = std::env::var("SKD_ROUNDS")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(10);
    let dir = project("wmi");
    let _guard = Guard::new(&dir);
    for round in 1..=rounds {
        let _ = std::fs::remove_file(runner_dir().join("runner.json"));
        let asked_at = shakedown_protocol::time::now();
        let clock = Instant::now();
        shakedown::win::start_by_wmi(&exe()).expect("WMI starts the runner");
        let wmi_s = clock.elapsed().as_secs_f64();
        // A client waits 10 s for runner.json (P-01); this waits longer to see how long it took.
        let found = within(Duration::from_secs(60), || runner_file()["pid"].as_u64().is_some());
        let file_s = clock.elapsed().as_secs_f64();
        let created = runner_file()["started_at"].as_str().unwrap_or("?").to_string();
        println!(
            "round {round}: asked WMI at {asked_at}, WMI answered after {wmi_s:.2} s, the runner process was created at {created}, runner.json after {file_s:.2} s"
        );
        assert!(found, "round {round}: no runner.json within 60 s");
        assert!(
            file_s <= 10.0,
            "round {round}: runner.json came after {file_s:.2} s, past a client's 10 s; runner.log:
{}",
            runner_log().lines().rev().take(8).collect::<Vec<_>>().join(
                "
"
            )
        );
        let runner = runner_file()["pid"].as_u64().unwrap() as u32;
        let watch = w::Watch::new(runner).expect("a handle on the runner");
        let (code, out, err) = cli_json(&dir, &["session", "start", "--setup", "ded"]);
        let died = |what: &str| -> String {
            format!(
                "round {round}: runner {runner} ended by itself {what}, exit code {:#x}; runner.log:
{}",
                watch.exit_code().unwrap_or(0),
                runner_log().lines().rev().take(15).collect::<Vec<_>>().join(
                    "
"
                )
            )
        };
        assert!(watch.exit_code().is_none(), "{}", died("during its session start"));
        assert_eq!(
            (code, out["state"].clone()),
            (0, json!("ready")),
            "round {round}: {out} {err}"
        );
        let pids: Vec<u32> = out["machines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["pid"].as_u64().unwrap() as u32)
            .collect();
        let mut run = Command::new(exe())
            .args(["run", "US-035-long"])
            .current_dir(&dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        assert!(
            within(Duration::from_secs(60), || status(&dir)["run"]["current_check"]
                == "US-035-long"),
            "round {round}: the run did not begin: {}",
            status(&dir)
        );
        // The check runs on for 200 s; 20 s of it with the runner holding the session.
        let early = within(Duration::from_secs(20), || watch.exit_code().is_some());
        assert!(!early, "{}", died("during the run"));
        kill(runner, false);
        assert!(within(Duration::from_secs(10), || watch.exit_code().is_some()));
        assert_eq!(
            watch.exit_code(),
            Some(1),
            "round {round}: taskkill /F ends with exit code 1"
        );
        assert!(
            within(Duration::from_secs(10), || pids.iter().all(|p| !alive(*p))),
            "round {round}: game processes left 10 s after the kill"
        );
        if !within(Duration::from_secs(10), || run.try_wait().ok().flatten().is_some()) {
            kill(run.id(), false);
        }
        let _ = run.wait();
        println!("round {round}: runner {runner} by WMI lived until it was killed");
    }
    // The next runner clears the last dead session.
    let (code, _, _) = cli(&dir, &["session", "stop"]);
    assert_eq!(code, 4);
}

/// `US-052`, `US-053`, `US-060` and `US-007` criterion 2, in one dedicated session.
#[test]
#[ignore = "starts the game; run by hand on the test machine"]
fn us_052_us_053_us_060_us_007_local_control_audit_and_an_unreachable_runner() {
    let dir = project("us052");
    let _guard = Guard::new(&dir);
    let mut outputs: Vec<String> = Vec::new();
    let session = start(&dir);
    let id = session["id"].as_str().unwrap().to_string();
    let folder = PathBuf::from(session["folder"].as_str().unwrap());
    let file = runner_file();
    let runner = file["pid"].as_u64().unwrap() as u32;
    let port = file["port"].as_u64().unwrap() as u16;
    let machine_pids: Vec<(String, u32)> = session["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["name"].as_str().unwrap().to_string(),
                m["pid"].as_u64().unwrap() as u32,
            )
        })
        .collect();

    // US-007 criterion 2: a later `run` in another process runs against the session that
    // `session start` left, and its result file names it.
    let (code, run, err) = cli_json(&dir, &["run", "US-052-on-p1"]);
    outputs.push(run.to_string());
    outputs.push(err.clone());
    assert_eq!(code, 0, "{run} {err}");
    let result_file = PathBuf::from(run["result_file"].as_str().unwrap());
    let result: Value = serde_json::from_str(&std::fs::read_to_string(&result_file).unwrap()).unwrap();
    assert_eq!(result["session"]["id"], id.as_str(), "US-007");

    // US-052 criterion 2: exec from the command line and from MCP, and the check's, audited.
    let (code, out, err) = cli_json(&dir, &["exec", "--machine", "server", "--code", "6 * 7"]);
    outputs.push(out.to_string());
    outputs.push(err);
    assert_eq!(code, 0);
    let mut mcp = Mcp::start(&exe(), &dir, &[]);
    let a = mcp.call("exec", json!({"machine": "hc1", "code": "6 * 8"}));
    outputs.push(a.text());
    assert!(!a.is_error(), "{}", a.text());
    let audit: Vec<Value> = std::fs::read_to_string(folder.join("audit.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let has = |source: &str, target: &str, code: &str| {
        audit.iter().any(|e| {
            e["source"] == source
                && e["target"] == target
                && e["code"].as_str().unwrap_or_default().contains(code)
                && e["time"].is_string()
        })
    };
    assert!(has("cli", "server", "6 * 7"), "US-052: {audit:#?}");
    assert!(has("mcp", "hc1", "6 * 8"), "US-052: {audit:#?}");
    assert!(has("US-052-on-p1", "server", "skd_fnc_on"), "US-052: {audit:#?}");
    assert!(has("US-052-on-p1", "p1", "40 + 2"), "US-052: {audit:#?}");

    // US-052 criterion 1, NFR-SC-01: every socket of the runner, and every socket of a game
    // process to the runner, is on 127.0.0.1.
    let netstat = String::from_utf8_lossy(&Command::new("netstat").arg("-ano").output().unwrap().stdout).into_owned();
    let mut seen = 0;
    for line in netstat.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 4 || !f[0].starts_with("TCP") && !f[0].starts_with("UDP") {
            continue;
        }
        let owner: u32 = f.last().unwrap().parse().unwrap_or(0);
        let to_runner = f[2].ends_with(&format!(":{port}"));
        let of_runner = owner == runner;
        let of_game = machine_pids.iter().any(|(_, p)| *p == owner);
        if of_runner || (of_game && to_runner) {
            seen += 1;
            assert!(f[1].starts_with("127.0.0.1:"), "NFR-SC-01: {line}");
        }
    }
    println!("NFR-SC-01: {seen} sockets of the runner and of the extensions, all on 127.0.0.1");
    assert!(seen > machine_pids.len());

    // US-053 criterion 1: a wrong runner token, and a hello as an extension with a wrong session
    // token, are closed without an answer and logged.
    let me = std::process::id();
    let mark = runner_log().len();
    let bad = format!("skd_r_{}", "0".repeat(64));
    assert_eq!(
        hello(
            port,
            &json!({"hello": "client", "version": shakedown_protocol::VERSION, "token": bad})
        ),
        None
    );
    let bad = format!("skd_s_{}", "0".repeat(64));
    assert_eq!(
        hello(
            port,
            &json!({"hello": "extension", "session": id, "machine": "p1", "token": bad})
        ),
        None
    );
    // US-053 criterion 2: the right session token, taken from a game process's environment,
    // from a process the runner did not start for that machine.
    let (_, p1) = machine_pids.iter().find(|(n, _)| n == "p1").unwrap();
    let env = w::environment(*p1).expect("p1's environment");
    let stolen = env
        .iter()
        .find(|(_, v)| v.starts_with("skd_s_"))
        .map(|(_, v)| v.clone())
        .expect("a session token in p1's environment");
    assert_eq!(
        hello(
            port,
            &json!({"hello": "extension", "session": id, "machine": "p1", "token": stolen})
        ),
        None
    );
    let log = runner_log()[mark..].to_string();
    println!("US-053 runner.log:\n{log}");
    assert!(
        log.contains(&format!("refused client connection from pid {me}: bad-token")),
        "{log}"
    );
    assert!(
        log.contains(&format!("refused extension connection from pid {me}: bad-token")),
        "{log}"
    );
    assert!(
        log.contains(&format!("refused extension connection from pid {me}: unknown-process")),
        "{log}"
    );
    // No order reached a machine: p1 is still the same process, ready, and answers.
    let s = status(&dir);
    let p1_now = s["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "p1")
        .unwrap()
        .clone();
    assert_eq!(p1_now["pid"], *p1);
    assert_eq!(p1_now["state"], "ready");
    let (code, out, _) = cli_json(&dir, &["exec", "--machine", "p1", "--code", "1 + 1"]);
    assert_eq!((code, out["results"]["p1"]["value"].clone()), (0, json!(2)));
    let after: usize = std::fs::read_to_string(folder.join("audit.jsonl"))
        .unwrap()
        .lines()
        .count();
    assert_eq!(
        after,
        audit.len() + 1,
        "only the one exec of this test went into the audit"
    );

    // US-060: the runner suspended by the runner_pid status gave.
    let starts: Vec<(Value, Value)> = s["session"]["machines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["name"].clone(), m["start"].clone()))
        .collect();
    assert_eq!(s["runner_pid"], runner);
    w::suspend(runner, true);
    let suspended = Instant::now();
    let t = Instant::now();
    let (code, st, _) = cli_json(&dir, &["status"]);
    println!("US-060 status took {:.1} s", t.elapsed().as_secs_f64());
    assert!(t.elapsed() < Duration::from_secs(10));
    assert_eq!(code, 0);
    assert_eq!(st["runner"], "unreachable", "{st}");
    assert_eq!(st["runner_pid"], runner);
    assert!(st["session"].is_null());
    let t = Instant::now();
    let (code, out, err) = cli(&dir, &["run", "US-052-on-p1"]);
    outputs.push(out);
    outputs.push(err.clone());
    println!("US-060 run took {:.1} s: {err}", t.elapsed().as_secs_f64());
    assert!(t.elapsed() < Duration::from_secs(10));
    assert_eq!(code, 4);
    assert!(
        err.contains(&runner.to_string()) && err.contains("ends its session"),
        "{err}"
    );
    let a = mcp.call("run_start", json!({}));
    outputs.push(a.text());
    assert!(a.text().starts_with("runner-unreachable: "), "{}", a.text());
    let a = mcp.call("exec", json!({"machine": "server", "code": "1"}));
    outputs.push(a.text());
    assert!(a.text().starts_with("runner-unreachable: "), "{}", a.text());
    let a = mcp.call("status", json!({}));
    outputs.push(a.text());
    assert_eq!(a.data()["runner"], "unreachable");
    assert_eq!(shakedown_pids("runner"), vec![runner], "a second runner started");
    // US-060 3, SES-06, HLT-02: the runner stays suspended past the heartbeat gap of 10 s, as
    // long as the machines' heartbeats would take to make them lost, while status keeps
    // answering that it is unreachable.
    let held = within(Duration::from_secs(40), || {
        let (_, st, _) = cli_json(&dir, &["status"]);
        assert_eq!(st["runner"], "unreachable", "{st}");
        suspended.elapsed() >= Duration::from_secs(20)
    });
    assert!(held);
    println!(
        "US-060 the runner was suspended for {:.1} s",
        suspended.elapsed().as_secs_f64()
    );
    w::suspend(runner, false);
    // Once resumed, the same session stays ready with every machine ready at its start number,
    // for longer than the heartbeat gap.
    let resumed = Instant::now();
    let mut seen = 0;
    within(Duration::from_secs(20), || {
        let s = status(&dir);
        assert_eq!(s["session"]["id"], id.as_str());
        assert_eq!(
            s["session"]["state"],
            "ready",
            "US-060 after {:?}: {s}",
            resumed.elapsed()
        );
        let now: Vec<(Value, Value, Value)> = s["session"]["machines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| (m["name"].clone(), m["start"].clone(), m["state"].clone()))
            .collect();
        let want: Vec<(Value, Value, Value)> = starts
            .iter()
            .map(|(n, st)| (n.clone(), st.clone(), json!("ready")))
            .collect();
        assert_eq!(now, want, "US-060: a machine was restarted or lost");
        seen += 1;
        resumed.elapsed() >= Duration::from_secs(15)
    });
    println!("US-060 status was ready {seen} times over 15 s after the resume");
    assert!(seen >= 5);

    // US-060 criterion 4: a stand-in that runner.json names answers with another version.
    let real = std::fs::read_to_string(runner_dir().join("runner.json")).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let stand_port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(50) {
            let Ok(mut s) = stream else { continue };
            let mut line = String::new();
            let _ = BufReader::new(s.try_clone().unwrap()).read_line(&mut line);
            let _ = writeln!(s, "{}", json!({"hello": "runner", "version": "0.0.0-standin"}));
        }
    });
    std::fs::write(
        runner_dir().join("runner.json"),
        json!({"port": stand_port, "token": "x", "pid": me,
               "started_at": shakedown::win::process_started(me).unwrap()})
        .to_string(),
    )
    .unwrap();
    let (code, out, err) = cli(&dir, &["run", "US-052-on-p1"]);
    let (_, st, _) = cli_json(&dir, &["status"]);
    std::fs::write(runner_dir().join("runner.json"), real).unwrap();
    outputs.push(out);
    outputs.push(err.clone());
    println!("US-060 stand-in: {err}");
    assert_eq!(code, 4);
    assert!(
        err.contains("0.0.0-standin") && err.contains(shakedown_protocol::VERSION),
        "{err}"
    );
    assert!(err.contains(&me.to_string()), "{err}");
    assert_eq!(st["runner"], "unreachable");
    assert_eq!(st["runner_pid"], me);
    drop(mcp);

    // US-052 criterion 3, NFR-SC-03: no token in any process's command line, in the result
    // file, in a log the framework writes or in any tool output, during the session.
    let pattern = token_pattern();
    for (pid, name, line) in processes() {
        assert!(
            !pattern.is_match(&line),
            "a token on the command line of {name} ({pid})"
        );
    }
    for out in &outputs {
        assert!(!pattern.is_match(out), "a token in tool output");
    }
    let mut searched = 0;
    for root in [folder.clone(), result_file.parent().unwrap().to_path_buf()] {
        for (path, text) in files_text(&root) {
            searched += 1;
            assert!(!pattern.is_match(&text), "a token in {}", path.display());
        }
    }
    assert!(!pattern.is_match(&runner_log()), "a token in runner.log");
    println!("NFR-SC-03: no token in {searched} files, runner.log, the command lines or tool output");
    let (code, _, _) = cli(&dir, &["session", "stop"]);
    assert_eq!(code, 0);
    for (path, text) in files_text(&folder) {
        assert!(!pattern.is_match(&text), "a token in {} after the stop", path.display());
    }
    let end: Value = serde_json::from_str(&std::fs::read_to_string(folder.join("session.json")).unwrap()).unwrap();
    assert_eq!(end["state"], "stopped");
    assert!(end["reason"].is_null());
}
