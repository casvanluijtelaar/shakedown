//! Joining late, leaving and joining again, for the game clients of a dedicated session
//! (`MCH-01`): a check asks for a join or a leave through the lead's agent, and the reset before
//! the next check brings the session back to the machines it started with.

use super::*;

/// The ceiling of a leave by default: the server's disconnect timeout of 15 s plus 5 s (`MCH-01`).
pub const LEAVE_CEILING_S: f64 = 20.0;
/// How often a leave asks the server again to drop the user of the process it ended (`SPK-18`).
const KICK_EVERY: Duration = Duration::from_secs(1);
/// How long a leave kicks a client whose process still runs before it ends the process and waits
/// for the server's disconnect timeout instead (`SPK-18`).
const KICK_FOR: Duration = Duration::from_secs(5);
/// The most server `PlayerDisconnected` events a session keeps.
const DISCONNECTS_LIMIT: usize = 100;

/// Why a join or a leave did not complete.
#[derive(Debug)]
pub enum MachineError {
    /// The check asked for a machine that cannot join or leave: the check's mistake (`DM-04`).
    Refused(String),
    /// The environment stopped it: the machine was not `ready` in time, its process exited, or
    /// the server did not see it go (`DM-05`). The message and the machine's last RPT lines.
    Lost(String, Vec<String>),
}

/// The order of machines in a session: `server`, `p1`..`pN`, `hc1`..`hcN` (`DM-10`).
fn rank(name: &str) -> (u8, u32) {
    let number = |prefix: &str| name.strip_prefix(prefix).and_then(|n| n.parse().ok()).unwrap_or(0);
    if name == "server" {
        (0, 0)
    } else if name.starts_with("hc") {
        (2, number("hc"))
    } else {
        (1, number("p"))
    }
}

/// A list of machine names for a message, or `none`.
fn listed(names: &[String]) -> String {
    if names.is_empty() {
        "none".into()
    } else {
        names.join(", ")
    }
}

/// Whether `name` can `join` (or leave) a session of these clients whose machines are `current`;
/// the check-error message of `MCH-01`, naming the machines that can join and leave, if not.
fn refusal(
    dedicated: bool,
    started: &[String],
    late: &[String],
    current: &[String],
    join: bool,
    name: &str,
) -> Option<String> {
    let verb = if join { "join" } else { "leave" };
    if !dedicated {
        return Some(format!(
            "skd_fnc_{verb}: joining and leaving needs a dedicated session; this session is singleplayer. Give the check `@setups` with a dedicated setup."
        ));
    }
    let clients: Vec<String> = started.iter().chain(late.iter()).cloned().collect();
    let here = current.iter().any(|n| n == name);
    let why = if name == "server" {
        "server is the dedicated server, which cannot join or leave".to_string()
    } else if name.starts_with("hc") && here {
        format!("{name} is a headless client; only game clients join and leave")
    } else if !clients.iter().any(|n| n == name) {
        format!(
            "the setup names no game client {name}; its game clients are {}",
            listed(&clients)
        )
    } else if join && here {
        format!("{name} is in the session already")
    } else if !join && !here {
        format!("{name} is not in the session, so it cannot leave")
    } else {
        return None;
    };
    let can_join: Vec<String> = clients.iter().filter(|n| !current.contains(n)).cloned().collect();
    let can_leave: Vec<String> = current.iter().filter(|n| rank(n).0 == 1).cloned().collect();
    Some(format!(
        "skd_fnc_{verb}: {why}. The machines that can join now: {}; that can leave now: {}.",
        listed(&can_join),
        listed(&can_leave)
    ))
}

impl Session {
    /// The game clients of the setup: those the session starts, and the late ones (`MCH-01`).
    pub fn client_names(&self) -> (Vec<String>, Vec<String>) {
        let setup = &self.spec.setup;
        if setup.mode != Mode::Dedicated {
            return (vec!["p1".into()], vec![]);
        }
        let started = (1..=setup.clients).map(|n| format!("p{n}")).collect();
        let late = (setup.clients + 1..=setup.clients + setup.late)
            .map(|n| format!("p{n}"))
            .collect();
        (started, late)
    }

    /// Whether `name` can `join` (or leave) now; the check-error message of `MCH-01` if not.
    fn refusal(&self, join: bool, name: &str) -> Option<String> {
        let (started, late) = self.client_names();
        refusal(
            self.spec.setup.mode == Mode::Dedicated,
            &started,
            &late,
            &self.machine_names(),
            join,
            name,
        )
    }

    /// Takes a machine out of the session: its start ends `stopped` among the retired starts,
    /// which the result file lists, and no order waits on it any more.
    fn retire(&self, runner: &Runner, name: &str) {
        let mut inner = self.inner.lock().unwrap();
        let Some(i) = inner.machines.iter().position(|m| m.name == name) else {
            return;
        };
        let mut old = inner.machines.remove(i);
        if let Some(p) = old.process.take() {
            runner.log(&format!(
                "{} {name}: start {} stopped, process {}",
                self.id, old.start, p.pid
            ));
        }
        old.tx = None;
        old.state = "stopped";
        inner.retired.push((old, SystemTime::now()));
        inner.pending.retain(|_, (target, _, _)| target != name);
        drop(inner);
        self.changed.notify_waiters();
    }

    /// Ends a machine's process and waits up to `STOP_CEILING` for it to end.
    async fn end_process(&self, name: &str) {
        {
            let inner = self.inner.lock().unwrap();
            if let Some(p) = inner
                .machines
                .iter()
                .find(|m| m.name == name)
                .and_then(|m| m.process.as_ref())
            {
                p.terminate();
            }
        }
        for _ in 0..(STOP_CEILING.as_millis() / POLL.as_millis()) {
            let left = {
                let inner = self.inner.lock().unwrap();
                inner
                    .machines
                    .iter()
                    .find(|m| m.name == name)
                    .and_then(|m| m.process.as_ref())
                    .is_some_and(|p| win::exited(p).is_none())
            };
            if !left {
                break;
            }
            tokio::time::sleep(POLL).await;
        }
    }

    /// Records a `PlayerDisconnected` event the server's agent reported for the user `name`.
    pub(super) fn disconnected(inner: &mut Inner, name: &str) {
        inner.disconnects.push((name.to_string(), Instant::now()));
        if inner.disconnects.len() > DISCONNECTS_LIMIT {
            inner.disconnects.remove(0);
        }
    }

    fn disconnected_since(&self, name: &str, since: Instant) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.disconnects.iter().any(|(n, at)| n == name && *at >= since)
    }

    /// Starts a late client or a client that left and returns its start number once it is
    /// `ready` (`MCH-01`, `P-02` steps 5 to 8). A join that is not `ready` within `ceiling`, or
    /// whose process exits first, stops that process and leaves the session without it.
    pub async fn join(&self, runner: &Runner, name: &str, ceiling: Duration) -> Result<u32, MachineError> {
        if let Some(why) = self.refusal(true, name) {
            return Err(MachineError::Refused(why));
        }
        let start = {
            let mut inner = self.inner.lock().unwrap();
            let start = inner
                .retired
                .iter()
                .map(|(m, _)| m)
                .chain(inner.machines.iter())
                .filter(|m| m.name == name)
                .map(|m| m.start)
                .max()
                .unwrap_or(0)
                + 1;
            let profile = self.folder.join("machines").join(format!("{name}-{start}"));
            inner.machines.push(Machine::new(name, "client", start, profile));
            inner.machines.sort_by_key(|m| rank(&m.name));
            start
        };
        runner.log(&format!("{} {name}: joins as start {start}", self.id));
        if let Err(cause) = self.launch(runner, &[name.to_string()], &|_| {}) {
            self.retire(runner, name);
            return Err(MachineError::Lost(
                format!("{name} could not be started: {cause}"),
                vec![],
            ));
        }
        let deadline = Instant::now() + ceiling;
        let mut asked = Instant::now() - Duration::from_secs(1);
        loop {
            let notified = self.changed.notified();
            let (state, agent_ready, exit, pid, profile) = {
                let inner = self.inner.lock().unwrap();
                let Some(m) = inner.machines.iter().find(|m| m.name == name) else {
                    return Err(MachineError::Lost(
                        format!("{name} left the session during its join"),
                        vec![],
                    ));
                };
                (
                    m.state,
                    m.agent_ready,
                    m.process.as_ref().and_then(win::exited),
                    m.pid,
                    m.profile.clone(),
                )
            };
            if state == "ready" {
                break;
            }
            if let Some(code) = exit {
                let tail = rpt_tail(&profile, 20);
                self.retire(runner, name);
                return Err(MachineError::Lost(
                    format!(
                        "{name}'s process {} exited during its join with exit code {code} ({code:#010X}); it was {state}",
                        pid.map_or("?".into(), |p| p.to_string())
                    ),
                    tail,
                ));
            }
            if Instant::now() >= deadline {
                self.end_process(name).await;
                let tail = rpt_tail(&profile, 20);
                self.retire(runner, name);
                return Err(MachineError::Lost(
                    format!(
                        "{name} was not ready within the join's ceiling of {} s; it was {state}, and its process {} is stopped",
                        ceiling.as_secs_f64(),
                        pid.map_or("?".into(), |p| p.to_string())
                    ),
                    tail,
                ));
            }
            // A dedicated client whose agent is ready is ready once the server sees it at client
            // state 10 (P-02 step 7, INT-AR-09).
            if agent_ready && asked.elapsed() >= Duration::from_millis(500) {
                asked = Instant::now();
                let answer = self
                    .exec("server", "_this call skd_fnc_clientState", json!([name]), 5.0)
                    .await;
                let client = answer.as_ref().ok().and_then(|a| a["value"].as_i64()).unwrap_or(-1);
                if client >= CLIENT_READY_STATE {
                    let mut inner = self.inner.lock().unwrap();
                    if let Some(m) = inner.machines.iter_mut().find(|m| m.name == name) {
                        m.state = "ready";
                        m.held = false;
                    }
                    continue;
                }
            }
            tokio::select! {
                _ = notified => {}
                _ = tokio::time::sleep(POLL) => {}
            }
        }
        self.versions_of(&[name.to_string()]).await;
        self.load_library(&[name.to_string()]).await;
        // A machine that joins during a check writes the run's and the check's start markers and
        // takes the check's context, so its log is cut out and its script errors count (LOG-04,
        // ERR-01).
        let executing = runner.executing.lock().unwrap().clone();
        if let Some((key, check)) = executing {
            let run = key.split('#').next().unwrap_or_default().to_string();
            for args in [json!([run, "start", ""]), json!([key, "start", check])] {
                let _ = self.exec(name, "_this call skd_fnc_context", args, 5.0).await;
            }
        }
        runner.log(&format!("{} {name}: joined, start {start}", self.id));
        Ok(start)
    }

    /// Makes a game client leave and returns once the server's `PlayerDisconnected` has fired
    /// for it (`MCH-01`): the server kicks it, so it goes at once rather than after the
    /// disconnect timeout (`SPK-18`), and its process is ended, so the machine is `stopped`, not
    /// `lost`.
    pub async fn leave(&self, runner: &Runner, name: &str, ceiling: Duration) -> Result<(), MachineError> {
        if let Some(why) = self.refusal(false, name) {
            return Err(MachineError::Refused(why));
        }
        let since = Instant::now();
        let deadline = since + ceiling;
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(m) = inner.machines.iter_mut().find(|m| m.name == name) {
                // Out of the watch of ready machines, which would take its end for a crash.
                m.state = "leaving";
            }
        }
        let profile = self.profile(name).unwrap_or_default();
        // Kicks the user by its profile name, which the server takes where a user id is ignored,
        // and says whether the server listed it.
        let kick = r##"params ["_pw", "_name"];
            private _listed = (allUsers findIf { ((getUserInfo _x) param [3, ""]) isEqualTo _name }) >= 0;
            if (_listed) then { _pw serverCommand format ["#kick %1", _name] };
            _listed"##;
        // The server drops a user it kicks while the user's process runs at once, but one whose
        // process has ended only after its disconnect timeout (SPK-18): so the client is kicked
        // first and its process ended once the server has seen it go, or after KICK_FOR when it
        // has not. A user the server no longer lists has gone already.
        let mut kicked = Instant::now() - KICK_EVERY;
        let mut ended = false;
        let outcome = loop {
            let notified = self.changed.notified();
            if self.disconnected_since(name, since) {
                break Ok(());
            }
            if Instant::now() >= deadline {
                break Err(MachineError::Lost(
                    format!(
                        "the server's PlayerDisconnected did not fire for {name} within the leave's ceiling of {} s; its process is stopped",
                        ceiling.as_secs_f64()
                    ),
                    rpt_tail(&profile, 20),
                ));
            }
            if !ended && since.elapsed() >= KICK_FOR {
                self.end_process(name).await;
                ended = true;
                continue;
            }
            if !ended && kicked.elapsed() >= KICK_EVERY {
                kicked = Instant::now();
                let listed = self
                    .exec("server", kick, json!([self.command_password(), name]), 5.0)
                    .await;
                if listed.as_ref().is_ok_and(|a| a["value"] == false) {
                    break Ok(());
                }
                continue;
            }
            tokio::select! {
                _ = notified => {}
                _ = tokio::time::sleep(POLL) => {}
            }
        };
        if !ended {
            self.end_process(name).await;
        }
        self.retire(runner, name);
        if outcome.is_ok() {
            runner.log(&format!(
                "{} {name}: left, the server saw it go after {:.3} s",
                self.id,
                since.elapsed().as_secs_f64()
            ));
        }
        outcome
    }

    /// Brings the session back to the machines it started with before a reset (`MCH-01`,
    /// `RST-01`): every late client that joined leaves, then every client of the start that left
    /// joins again, each within its default ceiling. The error says which did not.
    pub async fn restore_machines(&self, runner: &Runner) -> Result<(), String> {
        if self.spec.setup.mode != Mode::Dedicated {
            return Ok(());
        }
        let (started, _) = self.client_names();
        let current = self.machine_names();
        for name in current.iter().filter(|n| rank(n).0 == 1 && !started.contains(n)) {
            self.leave(runner, name, Duration::from_secs_f64(LEAVE_CEILING_S))
                .await
                .map_err(|e| format!("the late client {name} did not leave: {}", message(&e)))?;
        }
        for name in started.iter().filter(|n| !current.contains(n)) {
            self.join(runner, name, Duration::from_secs(self.spec.setup.start_timeout))
                .await
                .map_err(|e| format!("{name}, which left, did not join again: {}", message(&e)))?;
        }
        Ok(())
    }

    /// Serves a join or a leave the lead asked for in a check (`MCH-01`) and gives the lead the
    /// answer: `done` with the machines of the session, `refused` with the check-error message,
    /// or `lost` with the message and the machine's last 20 RPT lines.
    pub(super) async fn machine_request(self: Arc<Self>, runner: Arc<Runner>, from: String, message: Value) {
        let reference = message["ref"].as_str().unwrap_or_default().to_string();
        let name = message["machine"].as_str().unwrap_or_default().to_string();
        let join = message["kind"] == "join";
        let ceiling = message["ceiling"].as_f64().filter(|c| *c > 0.0);
        let outcome = if join {
            let ceiling = ceiling.unwrap_or(self.spec.setup.start_timeout as f64);
            self.join(&runner, &name, Duration::from_secs_f64(ceiling))
                .await
                .map(|start| json!({"start": start}))
        } else {
            let ceiling = ceiling.unwrap_or(LEAVE_CEILING_S);
            self.leave(&runner, &name, Duration::from_secs_f64(ceiling))
                .await
                .map(|_| json!({}))
        };
        let answer = match outcome {
            Ok(mut done) => {
                done["state"] = json!("done");
                done["machines"] = json!(self.machine_names());
                done
            }
            Err(MachineError::Refused(why)) => json!({"state": "refused", "message": why}),
            Err(MachineError::Lost(why, tail)) => {
                runner.log(&format!("{} {name}: {why}", self.id));
                json!({"state": "lost", "message": why, "rpt_tail": tail, "machines": self.machine_names()})
            }
        };
        self.push(&from, json!({"kind": "routed", "ref": reference, "answer": answer}));
    }
}

/// The message of a join or a leave that did not complete.
pub fn message(e: &MachineError) -> String {
    match e {
        MachineError::Refused(m) | MachineError::Lost(m, _) => m.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn us_069_mch_01_a_machine_that_cannot_join_or_leave_is_refused_naming_those_that_can() {
        let started = names(&["p1", "p2"]);
        let late = names(&["p3"]);
        let current = names(&["server", "p1", "p2"]);
        let refused = |join, name: &str, current: &[String]| refusal(true, &started, &late, current, join, name);
        for (join, name, why) in [
            (false, "server", "server is the dedicated server"),
            (false, "hc1", "names no game client hc1"),
            (true, "p1", "p1 is in the session already"),
            (true, "p9", "names no game client p9; its game clients are p1, p2, p3"),
        ] {
            let m = refused(join, name, &current).unwrap_or_else(|| panic!("{name} was not refused"));
            assert!(m.contains(why), "{m}");
            assert!(m.contains("can join now: p3; that can leave now: p1, p2."), "{m}");
            assert!(
                m.starts_with(if join { "skd_fnc_join:" } else { "skd_fnc_leave:" }),
                "{m}"
            );
        }
        let after = names(&["server", "p1"]);
        let m = refused(false, "p2", &after).unwrap();
        assert!(
            m.contains("p2 is not in the session") && m.contains("can join now: p2, p3; that can leave now: p1."),
            "{m}"
        );
        let with_hc = names(&["server", "p1", "p2", "hc1"]);
        assert!(
            refused(false, "hc1", &with_hc)
                .unwrap()
                .contains("hc1 is a headless client")
        );
        assert!(refused(true, "p3", &current).is_none());
        assert!(refused(false, "p2", &current).is_none());
        assert!(refused(true, "p2", &after).is_none());
    }

    #[test]
    fn us_069_mch_01_in_singleplayer_joining_and_leaving_needs_a_dedicated_session() {
        let m = refusal(false, &names(&["p1"]), &[], &names(&["p1"]), false, "p1").unwrap();
        assert!(m.contains("needs a dedicated session"), "{m}");
    }

    #[test]
    fn mch_01_machines_keep_the_order_of_dm_10() {
        let mut names = vec!["hc1", "p3", "p10", "server", "p1"];
        names.sort_by_key(|n| rank(n));
        assert_eq!(names, ["server", "p1", "p3", "p10", "hc1"]);
    }
}
