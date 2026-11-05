//! The connection between one game process and the runner, independent of the engine so it can be
//! tested without the game.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shakedown_protocol::{ENV_MACHINE, ENV_RUNNER, ENV_SESSION, ENV_TOKEN, Hello, PART_SIZE};

/// What the runner put in the game process's environment (`SEC-02`).
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub runner: Option<String>,
    pub session: Option<String>,
    pub machine: Option<String>,
    pub token: Option<String>,
}

impl Env {
    pub fn from_process() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Env {
            runner: var(ENV_RUNNER),
            session: var(ENV_SESSION),
            machine: var(ENV_MACHINE),
            token: var(ENV_TOKEN),
        }
    }
}

/// The state of the connection, as `status` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Connecting,
    Connected,
    Refused,
}

impl Link {
    pub fn as_str(self) -> &'static str {
        match self {
            Link::Connecting => "connecting",
            Link::Connected => "connected",
            Link::Refused => "refused",
        }
    }
}

type Notify = Box<dyn Fn(&str) + Send + 'static>;

pub struct Bridge {
    orders: Mutex<HashMap<String, String>>,
    link: Mutex<Link>,
    who: Mutex<(String, String)>,
    out: Mutex<Option<Sender<String>>>,
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge {
    pub fn new() -> Self {
        Bridge {
            orders: Mutex::new(HashMap::new()),
            link: Mutex::new(Link::Connecting),
            who: Mutex::new((String::new(), String::new())),
            out: Mutex::new(None),
        }
    }

    /// Connects to the runner on a thread of its own. Messages sent before the connection is up
    /// are queued and written once it is.
    pub fn start(self: &Arc<Self>, env: Env, notify: Notify) {
        let (tx, rx) = channel::<String>();
        *self.out.lock().unwrap() = Some(tx);
        let (Some(runner), Some(session), Some(machine), Some(token)) =
            (env.runner, env.session, env.machine, env.token)
        else {
            self.set_link(Link::Refused);
            return;
        };
        *self.who.lock().unwrap() = (session.clone(), machine.clone());
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            let hello = Hello::Extension {
                session,
                machine,
                token,
            };
            if me.connect(&runner, &hello, rx, notify).is_err() {
                me.set_link(Link::Refused);
            }
        });
    }

    fn connect(&self, runner: &str, hello: &Hello, rx: Receiver<String>, notify: Notify) -> std::io::Result<()> {
        let mut stream = TcpStream::connect(runner)?;
        stream.set_nodelay(true)?;
        let mut line = serde_json::to_string(hello).map_err(std::io::Error::other)?;
        line.push('\n');
        stream.write_all(line.as_bytes())?;
        self.set_link(Link::Connected);

        let mut writer = stream.try_clone()?;
        std::thread::spawn(move || {
            for message in rx {
                // A message is one line: a raw line break can only be JSON whitespace here, since
                // line breaks inside JSON strings are escaped.
                let mut line: String = message
                    .chars()
                    .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
                    .collect();
                line.push('\n');
                if writer.write_all(line.as_bytes()).is_err() {
                    break;
                }
            }
        });

        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = line?;
            let Some(id) = shakedown_protocol::order_id(&line) else {
                continue;
            };
            self.orders.lock().unwrap().insert(id.clone(), line);
            notify(&id);
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "runner closed the connection",
        ))
    }

    fn set_link(&self, link: Link) {
        *self.link.lock().unwrap() = link;
        if link == Link::Refused {
            // Nothing sent from now on can reach the runner.
            self.out.lock().unwrap().take();
        }
    }

    /// One part of an order: `1<text>` for the last part, `0<text>` when more follow. The order is
    /// forgotten once its last part was fetched.
    pub fn fetch(&self, id: &str, part: usize) -> Result<String, String> {
        let mut orders = self.orders.lock().unwrap();
        let Some(text) = orders.get(id) else {
            return Err("unknown-order".into());
        };
        let parts = shakedown_protocol::split_parts(text, PART_SIZE);
        let Some(chunk) = parts.get(part) else {
            return Err("unknown-part".into());
        };
        let last = part + 1 == parts.len();
        let out = format!("{}{}", if last { '1' } else { '0' }, chunk);
        if last {
            orders.remove(id);
        }
        Ok(out)
    }

    /// Queues a message for the runner; false when the connection is down.
    pub fn send(&self, json: String) -> bool {
        match self.out.lock().unwrap().as_ref() {
            Some(tx) => tx.send(json).is_ok(),
            None => false,
        }
    }

    pub fn status(&self) -> (Link, String, String) {
        let (session, machine) = self.who.lock().unwrap().clone();
        (*self.link.lock().unwrap(), session, machine)
    }

    /// Waits up to `ceiling` for the link to leave `Connecting`; for tests.
    pub fn wait_link(&self, ceiling: Duration) -> Link {
        let step = Duration::from_millis(10);
        for _ in 0..(ceiling.as_millis() / step.as_millis()).max(1) {
            let link = *self.link.lock().unwrap();
            if link != Link::Connecting {
                return link;
            }
            std::thread::sleep(step);
        }
        *self.link.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::mpsc;

    fn env(addr: &str) -> Env {
        Env {
            runner: Some(addr.into()),
            session: Some("s-20261105-120000".into()),
            machine: Some("p1".into()),
            token: Some("skd_s_ab".into()),
        }
    }

    #[test]
    fn spk_03_hello_carries_the_environment() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let bridge = Arc::new(Bridge::new());
        bridge.start(env(&addr), Box::new(|_| {}));
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).unwrap();
        let hello: Hello = serde_json::from_str(&line).unwrap();
        assert_eq!(
            hello,
            Hello::Extension {
                session: "s-20261105-120000".into(),
                machine: "p1".into(),
                token: "skd_s_ab".into()
            }
        );
    }

    #[test]
    fn int_ar_03_an_order_is_announced_then_fetched_in_parts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let bridge = Arc::new(Bridge::new());
        let (tx, rx) = mpsc::channel::<String>();
        let tx = Mutex::new(tx);
        bridge.start(
            env(&addr),
            Box::new(move |id| tx.lock().unwrap().send(id.to_string()).unwrap()),
        );
        let (mut stream, _) = listener.accept().unwrap();
        let big = "x".repeat(25_000);
        let order = format!("{{\"id\":\"o1\",\"kind\":\"exec\",\"source\":\"{big}\"}}\n");
        stream.write_all(order.as_bytes()).unwrap();
        let id = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(id, "o1");
        let mut text = String::new();
        for part in 0..10 {
            let got = bridge.fetch("o1", part).unwrap();
            assert!(got.len() <= PART_SIZE + 1);
            text.push_str(&got[1..]);
            if got.starts_with('1') {
                break;
            }
        }
        assert_eq!(format!("{text}\n"), order);
        assert_eq!(bridge.fetch("o1", 0), Err("unknown-order".into()));
    }

    #[test]
    fn int_ar_03_send_reaches_the_runner_as_one_line() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let bridge = Arc::new(Bridge::new());
        bridge.start(env(&addr), Box::new(|_| {}));
        assert!(
            bridge.send("{\"kind\":\"ready\",\n\"x\":1}".into()),
            "queued while connecting"
        );
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream);
        let mut hello = String::new();
        reader.read_line(&mut hello).unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["kind"], "ready");
        assert_eq!(bridge.wait_link(Duration::from_secs(5)), Link::Connected);
    }

    #[test]
    fn int_ar_03_without_the_environment_the_link_is_refused() {
        let bridge = Arc::new(Bridge::new());
        bridge.start(Env::default(), Box::new(|_| {}));
        assert_eq!(bridge.status().0, Link::Refused);
        assert!(!bridge.send("{}".into()));
    }
}
