//! What the M4 tests share: running the command line, and an MCP client that drives
//! `shakedown mcp` over its protocol, newline-delimited JSON-RPC on stdio, and checks every
//! structured result against its tool's output schema.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// The ceiling on any one answer of the server: above the 60 s of `AGT-03`.
const ANSWER_CEILING: Duration = Duration::from_secs(75);

pub fn repo() -> PathBuf {
    let p = std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")).unwrap();
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

/// Runs `exe` in `dir` with extra environment; the exit code, stdout and stderr.
pub fn run(exe: &Path, dir: &Path, env: &[(&str, &str)], args: &[&str]) -> (i32, String, String) {
    let out = Command::new(exe)
        .args(args)
        .envs(env.iter().copied())
        .current_dir(dir)
        .output()
        .unwrap();
    eprintln!("$ shakedown {} -> {:?}", args.join(" "), out.status.code());
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The same with `--json`, its stdout read as JSON.
pub fn run_json(exe: &Path, dir: &Path, env: &[(&str, &str)], args: &[&str]) -> (i32, Value, String) {
    let mut all = args.to_vec();
    all.push("--json");
    let (code, out, err) = run(exe, dir, env, &all);
    (code, serde_json::from_str(&out).unwrap_or(Value::Null), err)
}

pub struct Mcp {
    child: Child,
    stdin: ChildStdin,
    rx: mpsc::Receiver<Value>,
    next: u64,
    /// Messages read while waiting for another answer.
    early: Vec<Value>,
    schemas: HashMap<String, Value>,
}

/// One answer of a tool call: the result, the notifications that came before it with the time
/// each came, and how long the call took.
pub struct Answer {
    pub result: Value,
    pub notes: Vec<(Duration, Value)>,
    pub took: Duration,
}

impl Answer {
    pub fn is_error(&self) -> bool {
        self.result["isError"] == true
    }

    pub fn text(&self) -> String {
        self.result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    pub fn data(&self) -> Value {
        assert!(!self.is_error(), "the call failed: {}", self.text());
        self.result["structuredContent"].clone()
    }
}

impl Mcp {
    /// Starts `shakedown mcp` in `dir`, initialises the session and reads the tools.
    pub fn start(exe: &Path, dir: &Path, env: &[(&str, &str)]) -> Mcp {
        let mut child = Command::new(exe)
            .arg("mcp")
            .envs(env.iter().copied())
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(v) = serde_json::from_str::<Value>(&line)
                    && tx.send(v).is_err()
                {
                    break;
                }
            }
        });
        let mut mcp = Mcp {
            child,
            stdin,
            rx,
            next: 1,
            early: vec![],
            schemas: HashMap::new(),
        };
        let init = mcp.request(
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "skd-m04", "version": "0"}}),
        );
        assert!(init.0["result"]["serverInfo"]["name"] == "shakedown", "{}", init.0);
        mcp.notify("notifications/initialized", json!({}));
        let tools = mcp.request("tools/list", json!({})).0;
        for t in tools["result"]["tools"].as_array().unwrap() {
            mcp.schemas
                .insert(t["name"].as_str().unwrap().to_string(), t["outputSchema"].clone());
        }
        mcp
    }

    pub fn tools(&mut self) -> Value {
        self.request("tools/list", json!({})).0["result"]["tools"].clone()
    }

    fn send(&mut self, message: Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Sends a request and returns its id without waiting.
    pub fn send_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next;
        self.next += 1;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        id
    }

    /// Waits for the answer to `id`, with the notifications before it.
    pub fn wait(&mut self, id: u64, start: Instant) -> (Value, Vec<(Duration, Value)>) {
        let mut notes = Vec::new();
        let deadline = Instant::now() + ANSWER_CEILING;
        let mut early = std::mem::take(&mut self.early).into_iter();
        loop {
            let message = match early.next() {
                Some(m) => m,
                None => {
                    let left = deadline.saturating_duration_since(Instant::now());
                    self.rx
                        .recv_timeout(left)
                        .unwrap_or_else(|_| panic!("no answer to request {id} within {ANSWER_CEILING:?}"))
                }
            };
            if message["id"] == id && message.get("method").is_none() {
                self.early.extend(early);
                return (message, notes);
            }
            if message.get("id").is_some() && message.get("method").is_none() {
                self.early.push(message);
            } else {
                notes.push((start.elapsed(), message));
            }
        }
    }

    pub fn request(&mut self, method: &str, params: Value) -> (Value, Vec<(Duration, Value)>) {
        let start = Instant::now();
        let id = self.send_request(method, params);
        self.wait(id, start)
    }

    /// Calls a tool; a result that is not an error is checked against the tool's output schema.
    pub fn call(&mut self, tool: &str, args: Value) -> Answer {
        self.call_meta(tool, args, None)
    }

    /// Calls a tool with a progress token.
    pub fn call_progress(&mut self, tool: &str, args: Value) -> Answer {
        self.call_meta(tool, args, Some("p-1"))
    }

    fn call_meta(&mut self, tool: &str, args: Value, token: Option<&str>) -> Answer {
        let mut params = json!({"name": tool, "arguments": args});
        if let Some(t) = token {
            params["_meta"] = json!({"progressToken": t});
        }
        let start = Instant::now();
        let (message, notes) = self.request("tools/call", params);
        let took = start.elapsed();
        let result = message["result"].clone();
        assert!(!result.is_null(), "{tool}: no result: {message}");
        let answer = Answer { result, notes, took };
        if !answer.is_error() {
            self.validate(tool, &answer.result);
        }
        answer
    }

    /// `AGT-02`: the structured content matches the output schema and is the one text block.
    pub fn validate(&self, tool: &str, result: &Value) {
        let data = &result["structuredContent"];
        let text: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap_or("null")).unwrap();
        assert_eq!(&text, data, "{tool}: the text block is not the structured content");
        let schema = self.schemas.get(tool).unwrap_or_else(|| panic!("no tool {tool}"));
        let validator = jsonschema::validator_for(schema).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(data)
            .map(|e| format!("{e} at {}", e.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{tool}: {errors:#?}\n{data:#}");
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// `value` without the keys named, at any depth: for comparing two answers whose times differ.
pub fn without(value: &Value, keys: &[&str]) -> Value {
    match value {
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(k, _)| !keys.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), without(v, keys)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(|v| without(v, keys)).collect()),
        other => other.clone(),
    }
}

/// The shape of a value: its keys, with each value's JSON type, at any depth.
pub fn shape(value: &Value) -> Value {
    match value {
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), shape(v))).collect()),
        Value::Array(a) => Value::Array(a.iter().take(1).map(shape).collect()),
        Value::Null => json!("null"),
        Value::Bool(_) => json!("bool"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
    }
}
