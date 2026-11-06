//! The client side of `P-01`: finds the runner through `runner.json`, or starts one detached,
//! and sends it requests.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use serde::Deserialize;
use shakedown_protocol::Hello;

use crate::error::CliError;
use crate::paths;
use crate::wire::{Event, Request};

const HELLO_CEILING: Duration = Duration::from_secs(5);
const START_CEILING: Duration = Duration::from_secs(10);
const START_POLL: Duration = Duration::from_millis(100);

#[derive(Deserialize)]
struct RunnerFile {
    port: u16,
    token: String,
    pid: u32,
}

pub struct Conn {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

/// A handle that closes a connection from another thread, which tells the runner the client went
/// away (`AGT-04`).
pub struct Closer(TcpStream);

impl Closer {
    pub fn close(&self) {
        let _ = self.0.shutdown(std::net::Shutdown::Both);
    }
}

/// Connects to the running runner, or starts one when none runs.
pub fn connect() -> Result<Conn, CliError> {
    match try_existing()? {
        Some(conn) => Ok(conn),
        None => {
            start_runner()?;
            try_existing()?.ok_or_else(|| {
                CliError::framework(format!(
                    "The runner started but does not answer. See {}.",
                    paths::runner_log().display()
                ))
            })
        }
    }
}

/// The running runner, without starting one: `Ok(None)` when none runs, an error naming its process
/// when it does not answer or runs another version (`P-01`).
pub fn existing() -> Result<Option<Conn>, CliError> {
    try_existing()
}

/// The process id `runner.json` names, if the file can be read.
pub fn runner_pid() -> Option<u32> {
    let text = std::fs::read_to_string(paths::runner_file()).ok()?;
    serde_json::from_str::<RunnerFile>(&text).ok().map(|f| f.pid)
}

/// `Ok(None)` when no runner runs: no `runner.json`, or nothing listens on its port.
fn try_existing() -> Result<Option<Conn>, CliError> {
    let Ok(text) = std::fs::read_to_string(paths::runner_file()) else {
        return Ok(None);
    };
    let Ok(file) = serde_json::from_str::<RunnerFile>(&text) else {
        return Ok(None);
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], file.port));
    let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(2)) else {
        // Nothing listens: the runner that wrote the file is gone.
        let _ = std::fs::remove_file(paths::runner_file());
        return Ok(None);
    };
    let unreachable = |detail: &str| {
        CliError::not_now(format!(
            "The runner (process {}) {detail}. Ending that process ends its session's game processes with it; the next command then starts a fresh runner.",
            file.pid
        ))
        .tag("runner-unreachable")
    };
    let _ = stream.set_nodelay(true);
    let mut writer = stream
        .try_clone()
        .map_err(|e| CliError::framework(format!("Cannot use the runner connection: {e}.")))?;
    let hello = serde_json::to_string(&Hello::Client {
        version: shakedown_protocol::VERSION.into(),
        token: file.token,
    })
    .unwrap();
    writer
        .write_all(format!("{hello}\n").as_bytes())
        .map_err(|_| unreachable("does not answer"))?;
    stream.set_read_timeout(Some(HELLO_CEILING)).ok();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        // A runner that is exiting closes new connections after it removed its file.
        if !paths::runner_file().exists() {
            return Ok(None);
        }
        return Err(unreachable("does not answer"));
    }
    match serde_json::from_str::<Hello>(&line) {
        Ok(Hello::Runner { version }) if version == shakedown_protocol::VERSION => {}
        Ok(Hello::Runner { version }) => {
            return Err(unreachable(&format!(
                "runs version {version}, and this command is version {}",
                shakedown_protocol::VERSION
            )));
        }
        _ => return Err(unreachable("does not answer")),
    }
    reader.get_ref().set_read_timeout(None).ok();
    Ok(Some(Conn { reader, writer }))
}

/// Starts `shakedown runner` detached, outside this process's job and console (`SES-08`,
/// `INT-WIN-02`), and waits for its `runner.json`.
fn start_runner() -> Result<(), CliError> {
    let exe =
        std::env::current_exe().map_err(|e| CliError::framework(format!("Cannot find the shakedown binary: {e}.")))?;
    std::fs::create_dir_all(paths::runner_dir())
        .map_err(|e| CliError::framework(format!("Cannot create {}: {e}.", paths::runner_dir().display())))?;
    #[cfg(windows)]
    let spawned = crate::win::spawn_detached(&exe, &["runner".to_string()]).map(|_| ());
    #[cfg(not(windows))]
    let spawned = std::process::Command::new(&exe)
        .arg("runner")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ());
    spawned.map_err(|e| CliError::framework(format!("Cannot start the runner: {e}.")))?;
    let started = Instant::now();
    while started.elapsed() < START_CEILING {
        if paths::runner_file().is_file() {
            return Ok(());
        }
        std::thread::sleep(START_POLL);
    }
    Err(CliError::framework(format!(
        "The runner did not start within {} s. See {}.",
        START_CEILING.as_secs(),
        paths::runner_log().display()
    )))
}

impl Conn {
    pub fn closer(&self) -> Option<Closer> {
        self.writer.try_clone().ok().map(Closer)
    }

    /// Sends a request and returns its last event; `on_event` sees every event before it.
    pub fn request(&mut self, request: &Request, mut on_event: impl FnMut(&Event)) -> Result<Event, CliError> {
        let lost = || {
            CliError::framework(format!(
                "The runner closed the connection. See {}.",
                paths::runner_log().display()
            ))
        };
        let line = serde_json::to_string(request).unwrap();
        self.writer
            .write_all(format!("{line}\n").as_bytes())
            .map_err(|_| lost())?;
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line).map_err(|_| lost())? == 0 {
                return Err(lost());
            }
            let event: Event = serde_json::from_str(&line)
                .map_err(|e| CliError::framework(format!("The runner sent an unreadable answer: {e}.")))?;
            match event {
                Event::Progress { .. } | Event::Check { .. } | Event::RunStarted { .. } => on_event(&event),
                Event::Error { error } => return Err(error),
                last => return Ok(last),
            }
        }
    }
}
