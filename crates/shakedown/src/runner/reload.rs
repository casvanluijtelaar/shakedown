//! A reload (`P-08`, `RLD-01` to `RLD-06`): the changed files found or given, each classified by
//! `RLD-02`, the highest level applied, and the result in the Reload shape of
//! `contract/mcp-tools.md`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::time;

use super::Runner;
use super::session::{SessionState, stamp};
use crate::error::CliError;
use crate::wire::Rebuild;

/// The levels of `P-08`, lowest first.
const LEVELS: [&str; 2] = ["none", "game-restart"];

fn rank(level: &str) -> usize {
    LEVELS.iter().position(|l| *l == level).unwrap_or(0)
}

const CHECK: &str = "Checks are read fresh for every execution (RLD-04).";
const SQF: &str =
    "The game holds every mod file it has read and reads a changed one only in a new process, so the game restarts.";
const NEW_FILE: &str = "A new or deleted file: file patching finds only files that exist in the built PBO, so the mod must be rebuilt before this reload, which restarts the game.";
const NEW_FILE_REBUILT: &str = "A new or deleted file: file patching finds only files that exist in the built PBO, so the game restarts after the mod's rebuild.";
const CONFIG: &str = "Config is read at game start, so the game restarts.";
const OTHER: &str = "File patching does not reload this kind of file; it is read at game start, so the game restarts.";
const REBUILT: &str = " The mod's rebuild runs while no game process runs.";

/// The extensions of the files the game reads as config (`RLD-02`).
const CONFIG_EXTS: [&str; 6] = ["cpp", "hpp", "h", "inc", "ext", "xml"];

/// What changed in a watched folder since the snapshot: a file whose content differs, a new file
/// and a deleted file (`P-08` step 1). Size and modification time alone do not make a change.
pub fn changed(
    before: &std::collections::BTreeMap<PathBuf, super::session::Stamp>,
    now: &std::collections::BTreeMap<PathBuf, super::session::Stamp>,
) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = now
        .iter()
        .filter(|(p, s)| before.get(*p).is_none_or(|b| b.hash != s.hash || b.size != s.size))
        .map(|(p, _)| p.clone())
        .collect();
    out.extend(before.keys().filter(|p| !now.contains_key(*p)).cloned());
    out.sort();
    out
}

/// A path as a lowercase string with `.` and `..` resolved by its text, so two spellings of one
/// folder compare equal without following links.
fn lower(p: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in p.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            other => parts.push(
                other
                    .as_os_str()
                    .to_string_lossy()
                    .trim_end_matches(['\\', '/'])
                    .to_string(),
            ),
        }
    }
    parts.join("\\").to_lowercase()
}

/// The part of `path` below `dir`, if it is below it.
fn below(path: &Path, dir: &Path) -> Option<String> {
    let (p, d) = (lower(path), lower(dir));
    p.strip_prefix(&format!("{d}\\")).map(String::from)
}

/// Where a file is: a check, or a file of a mod's watched source.
#[derive(Debug, PartialEq)]
pub enum Place {
    Check,
    Source { mod_name: String },
}

/// Places a file by the folders it is under; an error names the watched folders.
pub fn place(path: &Path, tests: &Path, sources: &[(String, PathBuf, String)]) -> Result<Place, CliError> {
    for (name, dir, _) in sources {
        if below(path, dir).is_some() {
            return Ok(Place::Source { mod_name: name.clone() });
        }
    }
    if below(path, tests).is_some() {
        return Ok(Place::Check);
    }
    let watched: Vec<String> = sources.iter().map(|(_, d, _)| d.display().to_string()).collect();
    Err(CliError::wrong(format!(
        "{} is not under a watched source folder or the tests folder. Give a file under {}{}.",
        path.display(),
        if watched.is_empty() {
            "a mod's `source` (no mod of this setup sets one in shakedown.toml)".to_string()
        } else {
            watched.join(" or ")
        },
        format_args!(", or under {}", tests.display())
    ))
    .tag("bad-arguments"))
}

/// The level and reason of a file (`RLD-02`): a check needs nothing, and every file of a
/// watched source needs a game restart, since the game holds each mod file it has read and only a
/// new process reads a changed one (`SPK-08`, research 27). `rebuilt` says whether the file's mod
/// has a `rebuild`, which the restart runs.
pub fn classify(path: &Path, place: &Place, is_new_or_deleted: bool, rebuilt: bool) -> (&'static str, String) {
    let Place::Source { .. } = place else {
        return ("none", CHECK.into());
    };
    if is_new_or_deleted {
        let reason = if rebuilt { NEW_FILE_REBUILT } else { NEW_FILE };
        return ("game-restart", reason.into());
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let reason = if ext == "sqf" {
        SQF
    } else if CONFIG_EXTS.contains(&ext.as_str()) {
        CONFIG
    } else {
        OTHER
    };
    let suffix = if rebuilt { REBUILT } else { "" };
    ("game-restart", format!("{reason}{suffix}"))
}

/// `reload [files...]` (`RLD-01`): refused with no session, and while a run, a reload or a reset
/// is in progress; on a `broken` session it restarts the game, the one way back (`P-07`). Returns
/// once the reload is done, or after `wait_s` with state `applying`, the reload going on.
pub async fn reload(
    runner: &Arc<Runner>,
    files: Vec<PathBuf>,
    tests: PathBuf,
    rebuild: Rebuild,
    wait_s: Option<f64>,
) -> Result<Value, CliError> {
    let Some(session) = runner.session() else {
        return Err(CliError::not_now(
            "No session runs, so there is nothing to reload. Start one with `shakedown session start --setup <name>`.",
        )
        .tag("no-session"));
    };
    // A run is refused first: its session may be broken partway through it (`RLD-01`, `US-058`).
    if let Some(refusal) = super::run::refuse_during_run(runner, &session.id, "reload") {
        return Err(refusal);
    }
    let broken = match session.state() {
        SessionState::Ready => false,
        SessionState::Broken => true,
        SessionState::Reloading => {
            return Err(CliError::not_now(format!(
                "Session {} is busy with {}, so it cannot reload now. Wait for it to end; `shakedown status` shows it.",
                session.id,
                session.busy_with()
            ))
            .tag("session-busy"));
        }
        other => {
            return Err(CliError::not_now(format!(
                "Session {} is {}, so it cannot reload. Wait for it to be ready, or start one.",
                session.id,
                other.as_str()
            ))
            .tag("session-busy"));
        }
    };
    let sources = session.sources();
    // The files given, or what changed under the watched folders since the snapshot (P-08 step 1).
    let before = session.snapshot.lock().unwrap().clone();
    let mut now = std::collections::BTreeMap::new();
    for (_, dir, _) in &sources {
        for (p, s) in walk(dir) {
            now.insert(p, s);
        }
    }
    let paths: Vec<PathBuf> = if files.is_empty() {
        changed(&before, &now)
    } else {
        files
    };
    // Files are compared by their spelling with `.`, `..` and case resolved.
    let in_before: std::collections::HashSet<String> = before.keys().map(|p| lower(p)).collect();
    let in_now: std::collections::HashSet<String> = now.keys().map(|p| lower(p)).collect();
    let mut places = Vec::new();
    for p in &paths {
        let place = place(p, &tests, &sources)?;
        let known = in_before.contains(&lower(p)) || in_now.contains(&lower(p));
        if matches!(place, Place::Source { .. }) && !known && !p.exists() {
            return Err(CliError::wrong(format!(
                "{} does not exist and was not in the build when the session became ready. Give a file that exists or that was deleted since.",
                p.display()
            ))
            .tag("bad-arguments"));
        }
        places.push(place);
    }
    let from = if broken {
        SessionState::Broken
    } else {
        SessionState::Ready
    };
    let id = format!("rl-{}-{}", time::stamp(SystemTime::now()), runner.next_run_number());
    let started_at = time::now();
    let clock = Instant::now();

    // Classify (RLD-02), and the rebuild of each setup mod that has one, in the setup's order.
    let rebuilds: Vec<(String, String)> = session
        .mod_names()
        .into_iter()
        .filter_map(|name| {
            rebuild
                .commands
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(n, c)| (n.clone(), c.clone()))
        })
        .collect();
    let files: Vec<Value> = paths
        .iter()
        .zip(&places)
        .map(|(p, place)| {
            let new_or_deleted = !in_before.contains(&lower(p)) || !p.exists();
            let rebuilt = matches!(place, Place::Source { mod_name } if rebuilds.iter().any(|(n, _)| n == mod_name));
            let (level, reason) = classify(p, place, new_or_deleted, rebuilt);
            json!({"path": p, "level": level, "reason": reason})
        })
        .collect();
    let mut level = files
        .iter()
        .map(|f| f["level"].as_str().unwrap_or("none"))
        .max_by_key(|l| rank(l))
        .unwrap_or("none")
        .to_string();
    if broken {
        level = "game-restart".into();
    }
    let shape = {
        let (id, level, files, started_at) = (id.clone(), level.clone(), files.clone(), started_at.clone());
        move |state: &str, machines: Vec<String>, reason: Value, duration: f64| {
            json!({
                "id": id, "state": state, "level": level, "files": files, "machines": machines,
                "reason": reason, "started_at": started_at, "duration_s": duration,
            })
        }
    };
    if !session.begin_busy(from, format!("reload {id} (level {level})")) {
        return Err(CliError::not_now(format!(
            "Session {} became busy; try the reload again when it is ready.",
            session.id
        ))
        .tag("session-busy"));
    }
    *runner.last_reload.lock().unwrap() = Some(shape("applying", vec![], Value::Null, 0.0));
    runner.log(&format!(
        "{} reload {id}: level {level}, {} files",
        session.id,
        files.len()
    ));

    let task = {
        let runner = Arc::clone(runner);
        let session = Arc::clone(&session);
        let level = level.clone();
        let shape = shape.clone();
        let id = id.clone();
        tokio::spawn(async move {
            let outcome = if level == "game-restart" {
                session.game_restart(&runner, &rebuilds, &rebuild.folder, &|_| {}).await
            } else {
                Ok(())
            };
            let duration = (clock.elapsed().as_secs_f64() * 1000.0).round() / 1000.0;
            let value = match outcome {
                Ok(()) => {
                    if level == "none" {
                        session.take_snapshot();
                    }
                    session.set_ready();
                    runner.log(&format!("{} reload {id}: done in {duration} s", session.id));
                    shape("done", session.machine_names(), Value::Null, duration)
                }
                Err((machine, message)) => {
                    session.release();
                    let step = level.as_str();
                    session.set_broken(
                        &machine,
                        format!("the reload {id} failed at its {step} on {machine}: {message}"),
                    );
                    runner.log(&format!("{} reload {id}: failed on {machine}: {message}", session.id));
                    shape(
                        "failed",
                        vec![],
                        json!({"step": step, "machine": machine, "message": message}),
                        duration,
                    )
                }
            };
            *runner.last_reload.lock().unwrap() = Some(value);
        })
    };
    let wait = wait_s.map(Duration::from_secs_f64);
    match wait {
        Some(w) => {
            let _ = tokio::time::timeout(w, task).await;
        }
        None => {
            let _ = task.await;
        }
    }
    Ok(runner.last_reload.lock().unwrap().clone().unwrap_or(Value::Null))
}

fn walk(dir: &Path) -> Vec<(PathBuf, super::session::Stamp)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if let Some(s) = stamp(&path) {
            out.push((path, s));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::session::Stamp;
    use super::*;

    fn s(hash: u64) -> Stamp {
        Stamp {
            size: 4,
            modified: None,
            hash,
        }
    }

    fn sources() -> Vec<(String, PathBuf, String)> {
        vec![(
            "fixture".into(),
            PathBuf::from(r"C:\i\fixtures\skdfx\addons"),
            r"skdfx\addons".into(),
        )]
    }

    #[test]
    fn rld_01_a_changed_a_new_and_a_deleted_file_are_found_by_content() {
        let before = [("a", 1), ("b", 2), ("c", 3)]
            .map(|(p, h)| (PathBuf::from(p), s(h)))
            .into_iter()
            .collect();
        let now = [("a", 1), ("b", 9), ("d", 4)]
            .map(|(p, h)| (PathBuf::from(p), s(h)))
            .into_iter()
            .collect();
        assert_eq!(
            changed(&before, &now),
            vec![PathBuf::from("b"), PathBuf::from("c"), PathBuf::from("d")]
        );
    }

    #[test]
    fn us_040_rld_02_a_check_file_is_level_none() {
        let tests = Path::new(r"C:\p\tests");
        let p = Path::new(r"C:\p\tests\smoke.sqf");
        let place = place(p, tests, &sources()).unwrap();
        assert_eq!(place, Place::Check);
        assert_eq!(classify(p, &place, false, true), ("none", CHECK.into()));
    }

    #[test]
    fn us_056_rld_06_a_new_sqf_needs_a_rebuild_and_a_game_restart() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\functions\fn_new.sqf");
        let place = place(p, Path::new(r"C:\p\tests"), &sources()).unwrap();
        assert_eq!(
            place,
            Place::Source {
                mod_name: "fixture".into()
            }
        );
        let (level, reason) = classify(p, &place, true, false);
        assert_eq!(level, "game-restart");
        assert!(reason.contains("rebuilt before this reload") && reason.contains("built PBO"));
        let (level, reason) = classify(p, &place, true, true);
        assert_eq!(level, "game-restart");
        assert!(reason.contains("after the mod's rebuild"), "{reason}");
    }

    #[test]
    fn us_056_rld_02_config_is_a_game_restart() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\config.cpp");
        let place = place(p, Path::new(r"C:\p\tests"), &sources()).unwrap();
        let (level, reason) = classify(p, &place, false, false);
        assert_eq!(level, "game-restart");
        assert!(reason.contains("Config is read at game start"));
    }

    #[test]
    fn us_055_rld_02_every_changed_sqf_is_a_game_restart_that_runs_the_rebuild() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\functions\fn_version.sqf");
        let place = place(p, Path::new(r"C:\p\tests"), &sources()).unwrap();
        let (level, reason) = classify(p, &place, false, false);
        assert_eq!(level, "game-restart");
        assert!(
            reason.contains("only in a new process") && !reason.contains("rebuild"),
            "{reason}"
        );
        let (_, reason) = classify(p, &place, false, true);
        assert!(
            reason.ends_with("The mod's rebuild runs while no game process runs."),
            "{reason}"
        );
    }

    #[test]
    fn rld_01_a_file_outside_every_watched_folder_is_refused_naming_them() {
        let e = place(Path::new(r"C:\elsewhere\x.sqf"), Path::new(r"C:\p\tests"), &sources()).unwrap_err();
        assert_eq!(e.code, 3);
        assert!(e.message.contains(r"C:\i\fixtures\skdfx\addons"), "{}", e.message);
    }
}
