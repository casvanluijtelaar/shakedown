//! The run record (`STP-03`): every session the runner started, with each process it started for
//! it (id and start time) and each file and folder it created outside the project, kept in
//! `%LOCALAPPDATA%\Shakedown\record.json` so that the next runner start clears what a runner
//! that died left behind (`P-12`).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::paths;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub sessions: Vec<SessionEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: String,
    /// The session folder in the project, where the session's end is written.
    pub folder: PathBuf,
    pub processes: Vec<ProcessEntry>,
    /// Files and folders outside the project: the mission copy in `MPMissions`.
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub started_at: String,
}

/// One writer at a time; the file is rewritten whole on each change.
static LOCK: Mutex<()> = Mutex::new(());

fn read(file: &Path) -> Record {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn write(file: &Path, record: &Record) {
    let _ = std::fs::write(file, serde_json::to_string_pretty(record).unwrap());
}

fn change(f: impl FnOnce(&mut Record)) {
    let _guard = LOCK.lock().unwrap();
    let file = paths::record_file();
    let mut record = read(&file);
    f(&mut record);
    write(&file, &record);
}

fn entry<'a>(record: &'a mut Record, session: &str) -> Option<&'a mut SessionEntry> {
    record.sessions.iter_mut().find(|s| s.id == session)
}

/// Records a session before anything of it is created.
pub fn add_session(id: &str, folder: &Path) {
    change(|r| {
        r.sessions.push(SessionEntry {
            id: id.into(),
            folder: folder.into(),
            processes: vec![],
            paths: vec![],
        })
    });
}

/// Records a file or folder before it is created.
pub fn add_path(session: &str, path: &Path) {
    change(|r| {
        if let Some(s) = entry(r, session) {
            s.paths.push(path.into());
        }
    });
}

/// Records a process the moment it exists, before it is resumed (`STP-01`).
pub fn add_process(session: &str, pid: u32, started_at: &str) {
    change(|r| {
        if let Some(s) = entry(r, session) {
            s.processes.push(ProcessEntry {
                pid,
                started_at: started_at.into(),
            });
        }
    });
}

/// Removes a session once it stopped and nothing of it is left (`P-10`).
pub fn remove_session(session: &str) {
    change(|r| r.sessions.retain(|s| s.id != session));
}

/// Writes a session's end into its folder, as `session.json`: its id, end state (`stopped`, or
/// `failed` for a start that failed), reason and time.
pub fn write_end(folder: &Path, id: &str, state: &str, reason: Option<&str>) {
    let end = json!({
        "id": id,
        "state": state,
        "reason": reason,
        "stopped_at": shakedown_protocol::time::now(),
    });
    let _ = std::fs::write(folder.join("session.json"), serde_json::to_string_pretty(&end).unwrap());
}

/// The recovery of `P-12`, at every runner start before it accepts a client: ends each recorded
/// process that still runs with its recorded start time, deletes each recorded file and folder,
/// marks each recorded session `stopped` with reason `runner-died`, and empties the record.
/// Returns a line for `runner.log` for each thing it did.
pub fn recover() -> Vec<String> {
    recover_at(&paths::record_file())
}

fn recover_at(file: &Path) -> Vec<String> {
    let _guard = LOCK.lock().unwrap();
    let record = read(file);
    let mut done = Vec::new();
    for s in &record.sessions {
        for p in &s.processes {
            if crate::win::terminate_pid(p.pid, &p.started_at) {
                done.push(format!("recovery: {}: ended process {}", s.id, p.pid));
            }
        }
        for path in &s.paths {
            let removed = if path.is_dir() {
                std::fs::remove_dir_all(path).is_ok()
            } else {
                std::fs::remove_file(path).is_ok()
            };
            if removed {
                done.push(format!("recovery: {}: deleted {}", s.id, path.display()));
            }
        }
        if s.folder.is_dir() {
            write_end(&s.folder, &s.id, "stopped", Some("runner-died"));
        }
        done.push(format!("recovery: {}: stopped, reason runner-died", s.id));
    }
    write(file, &Record::default());
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `STP-03` and `P-12` without a game: a recorded session's mission copy and a recorded
    /// process that is not running are cleared, the session is marked stopped with reason
    /// `runner-died`, and the record is emptied.
    #[test]
    fn stp_03_p_12_recovery_clears_what_the_record_lists() {
        let root = std::env::temp_dir().join(format!("skd-p12-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let folder = root.join("project").join("s-1");
        let copy = root.join("MPMissions").join("shakedown_s-1.VR");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::create_dir_all(&copy).unwrap();
        std::fs::write(copy.join("mission.sqm"), "x").unwrap();
        let file = root.join("record.json");
        let record = Record {
            sessions: vec![SessionEntry {
                id: "s-1".into(),
                folder: folder.clone(),
                processes: vec![ProcessEntry {
                    pid: u32::MAX - 1,
                    started_at: "2026-01-01T00:00:00.000Z".into(),
                }],
                paths: vec![copy.clone()],
            }],
        };
        write(&file, &record);
        assert_eq!(read(&file), record);
        let done = recover_at(&file);
        assert!(done.iter().any(|l| l.contains("deleted")), "{done:?}");
        assert!(!copy.exists());
        let end: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(folder.join("session.json")).unwrap()).unwrap();
        assert_eq!(end["state"], "stopped");
        assert_eq!(end["reason"], "runner-died");
        assert!(read(&file).sessions.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
