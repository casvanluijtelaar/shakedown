//! A reload (`P-08`, `RLD-01` to `RLD-06`): the changed files found or given, each classified by
//! `RLD-02`, the highest level applied, and the result in the Reload shape of
//! `contract/mcp-tools.md`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::time;

use super::Runner;
use super::session::{SessionState, stamp};
use crate::error::CliError;
use crate::wire::Rebuild;

/// The levels of `P-08`, lowest first.
const LEVELS: [&str; 3] = ["none", "mission-restart", "game-restart"];

fn rank(level: &str) -> usize {
    LEVELS.iter().position(|l| *l == level).unwrap_or(0)
}

const CHECK: &str = "Checks are read fresh for every execution (RLD-04).";
const LIBRARY: &str = "Every reload reads the library folder again and has every machine compile it (API-06).";
const SQF: &str = "The game is made to let go of the file by reading the framework's release files, and a mission restart compiles it again.";
const NEW_FILE: &str = "A new or deleted file: file patching finds only files that exist in the built PBO, so the mod must be rebuilt before this reload, which restarts the game.";
const NEW_FILE_REBUILT: &str = "A new or deleted file: file patching finds only files that exist in the built PBO, so the game restarts after the mod's rebuild.";
const CONFIG: &str = "Config is read at game start, so the game restarts.";
const OTHER: &str = "File patching does not reload this kind of file; it is read at game start, so the game restarts.";
const REBUILT: &str = " The mod's rebuild runs while no game process runs.";
const REBUILT_FREED: &str = " The mod's rebuild runs once no game process holds the file.";

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
pub(crate) fn lower(p: &Path) -> String {
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

/// Where a file is: a check, a file of the library folder, or a file of a mod's watched source.
#[derive(Debug, PartialEq)]
pub enum Place {
    Check,
    Library,
    Source { mod_name: String },
}

/// Places a file by the folders it is under; an error names the watched folders.
pub fn place(path: &Path, tests: &Path, lib: &Path, sources: &[(String, PathBuf, String)]) -> Result<Place, CliError> {
    for (name, dir, _) in sources {
        if below(path, dir).is_some() {
            return Ok(Place::Source { mod_name: name.clone() });
        }
    }
    if crate::library::below(path, lib) {
        return Ok(Place::Library);
    }
    if below(path, tests).is_some() {
        return Ok(Place::Check);
    }
    let watched: Vec<String> = sources.iter().map(|(_, d, _)| d.display().to_string()).collect();
    Err(CliError::wrong(format!(
        "{} is not under a watched source folder, the tests folder or the library folder. Give a file under {}{}.",
        path.display(),
        if watched.is_empty() {
            "a mod's `source` (no mod of this setup sets one in shakedown.toml)".to_string()
        } else {
            watched.join(" or ")
        },
        format_args!(", under {} or under {}", tests.display(), lib.display())
    ))
    .tag("bad-arguments"))
}

/// The level and reason of a file (`RLD-02`): a check needs nothing; a changed `.sqf` of a
/// watched source that the build holds needs a mission restart once the game has let go of it
/// (`SPK-19`, research 31); a new or deleted file and every other kind of file needs a game
/// restart. `rebuilt` says whether the file's mod has a `rebuild`, which the restart runs.
pub fn classify(path: &Path, place: &Place, is_new_or_deleted: bool, rebuilt: bool) -> (&'static str, String) {
    let Place::Source { .. } = place else {
        let reason = if *place == Place::Library { LIBRARY } else { CHECK };
        return ("none", reason.into());
    };
    if is_new_or_deleted {
        let reason = if rebuilt { NEW_FILE_REBUILT } else { NEW_FILE };
        return ("game-restart", reason.into());
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if ext == "sqf" {
        let suffix = if rebuilt { REBUILT_FREED } else { "" };
        return ("mission-restart", format!("{SQF}{suffix}"));
    }
    let reason = if CONFIG_EXTS.contains(&ext.as_str()) {
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
    lib: PathBuf,
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
    // The library folder as it is now, compared with the library the session read last (API-06).
    let library = crate::library::read(&lib, &rebuild.folder);
    let paths: Vec<PathBuf> = if files.is_empty() {
        let mut paths = changed(&before, &now);
        paths.extend(crate::library::changed(&session.library.lock().unwrap(), &library));
        paths
    } else {
        files
    };
    // Files are compared by their spelling with `.`, `..` and case resolved.
    let in_before: std::collections::HashSet<String> = before.keys().map(|p| lower(p)).collect();
    let in_now: std::collections::HashSet<String> = now.keys().map(|p| lower(p)).collect();
    let mut places = Vec::new();
    for p in &paths {
        let place = place(p, &tests, &lib, &sources)?;
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
    // The level and the files change when a held file turns the reload into a game restart
    // (`US-082`), so the shape is built from them as they are.
    let state = Arc::new(Mutex::new((level.clone(), files)));
    let shape = {
        let (id, started_at, state) = (id.clone(), started_at.clone(), Arc::clone(&state));
        move |run_state: &str, machines: Vec<String>, reason: Value, duration: f64| {
            let (level, files) = state.lock().unwrap().clone();
            json!({
                "id": id, "state": run_state, "level": level, "files": files, "machines": machines,
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
        state.lock().unwrap().1.len()
    ));
    // The changed `.sqf` files a mission restart takes: each with the file the game reads for it
    // and its game path (`P-08` step 2).
    let freed: Vec<(usize, PathBuf, String)> = {
        let files = &state.lock().unwrap().1;
        files
            .iter()
            .enumerate()
            .filter(|(_, f)| f["level"] == "mission-restart")
            .filter_map(|(i, f)| {
                let path = PathBuf::from(f["path"].as_str()?);
                let (_, dir, prefix) = sources.iter().find(|(_, d, _)| below(&path, d).is_some())?;
                let game_path = format!(r"{}\{}", prefix.trim_matches('\\'), below(&path, dir)?);
                let file = super::release::game_file(&session.spec.game, dir, prefix, &path)?;
                Some((i, file, game_path))
            })
            .collect()
    };

    let task = {
        let runner = Arc::clone(runner);
        let session = Arc::clone(&session);
        let shape = shape.clone();
        let state = Arc::clone(&state);
        let id = id.clone();
        tokio::spawn(async move {
            // A game restart compiles the library on every machine it brings up; a reload of
            // level none has every machine compile it here (API-06).
            *session.library.lock().unwrap() = library;
            let level = state.lock().unwrap().0.clone();
            if level == "none" {
                session.load_library(&session.machine_names()).await;
            }
            let outcome = match level.as_str() {
                "mission-restart" => match release(&runner, &session, &freed).await {
                    Ok(held) if held.is_empty() => {
                        mission_restart(&runner, &session, &rebuilds, &rebuild.folder, &freed).await
                    }
                    Ok(held) => {
                        // US-082: a file still held makes the reload a game restart.
                        {
                            let mut st = state.lock().unwrap();
                            st.0 = "game-restart".into();
                            for (i, why) in held {
                                st.1[i]["level"] = json!("game-restart");
                                st.1[i]["reason"] = json!(why);
                            }
                        }
                        *runner.last_reload.lock().unwrap() = Some(shape("applying", vec![], Value::Null, 0.0));
                        runner.log(&format!(
                            "{} reload {id}: a changed file is still held, so the game restarts",
                            session.id
                        ));
                        session.game_restart(&runner, &rebuilds, &rebuild.folder, &|_| {}).await
                    }
                    Err(e) => Err(e),
                },
                "game-restart" => session.game_restart(&runner, &rebuilds, &rebuild.folder, &|_| {}).await,
                _ => Ok(()),
            };
            let level = state.lock().unwrap().0.clone();
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

/// How often every machine reads the release files before a held file makes the reload a game
/// restart (`P-08` step 2).
const READS: usize = 2;

/// The release of `P-08` step 2: every machine reads the release files, then each changed file is
/// checked free, once more if one is held. The files still held after the second read, by their
/// index in the reload's files, each with its reason naming the machine that holds it; a machine
/// that did not answer the read fails the reload.
async fn release(
    runner: &Runner,
    session: &Arc<super::session::Session>,
    freed: &[(usize, PathBuf, String)],
) -> Result<Vec<(usize, String)>, (String, String)> {
    if let Err(why) = super::release::ensure(&session.spec.game) {
        // Without the release files the game cannot be made to let go: the game restarts.
        return Ok(freed
            .iter()
            .map(|(i, _, _)| {
                (
                    *i,
                    format!("The release files could not be made ({why}), so the game restarts."),
                )
            })
            .collect());
    }
    let paths = super::release::paths();
    for read in 1..=READS {
        let t = Instant::now();
        session.read_release(runner, &paths).await?;
        let held: Vec<&(usize, PathBuf, String)> = freed
            .iter()
            .filter(|(_, file, _)| super::release::is_held(file))
            .collect();
        runner.log(&format!(
            "{} read {read} of the release files in {:.1} s: {} of {} changed files still held",
            session.id,
            t.elapsed().as_secs_f64(),
            held.len(),
            freed.len()
        ));
        if held.is_empty() {
            return Ok(vec![]);
        }
        if read == READS {
            return Ok(held
                .into_iter()
                .map(|(i, file, _)| (*i, held_reason(session, file)))
                .collect());
        }
    }
    Ok(vec![])
}

/// The reason of a file still held after the second read: the file, the machine whose process
/// holds it, found through the Restart Manager, and that the game did not let go of it (`US-082`).
fn held_reason(session: &super::session::Session, file: &Path) -> String {
    let pids = super::release::holders(file);
    let names: Vec<String> = session
        .machine_names()
        .into_iter()
        .filter(|n| session.machine_pid(n).is_some_and(|p| pids.contains(&p)))
        .collect();
    let who = if !names.is_empty() {
        format!("{}'s process", names.join(" and "))
    } else if pids.is_empty() {
        "a process the Restart Manager did not name".to_string()
    } else {
        let ids: Vec<String> = pids.iter().map(u32::to_string).collect();
        format!("process {}, which is no machine of the session,", ids.join(", "))
    };
    format!(
        "{} is still held by {who} after two reads of the framework's release files: the game did not let go of it, so the game restarts.",
        file.display()
    )
}

/// Level `mission-restart` once every changed file is free: each setup mod's `rebuild`, then the
/// mission restart, after which every machine recompiles the `CfgFunctions` functions of the
/// changed files (`P-08`, `RLD-02`, `RLD-06`). A failed rebuild still restarts the mission and then
/// fails the reload, naming the mod, with the lead as the machine.
async fn mission_restart(
    runner: &Runner,
    session: &super::session::Session,
    rebuilds: &[(String, String)],
    folder: &Path,
    freed: &[(usize, PathBuf, String)],
) -> Result<(), (String, String)> {
    let failed = session.rebuild_all(runner, rebuilds, folder, &|_| {}).await;
    {
        let mut recompile = session.recompile.lock().unwrap();
        for (_, _, game_path) in freed {
            if !recompile.0.contains(game_path) {
                recompile.0.push(game_path.clone());
            }
        }
    }
    session.mission_restart(runner, &|_| {}).await?;
    match failed {
        Some(why) => Err((session.lead().to_string(), why)),
        None => Ok(()),
    }
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
        let place = place(p, tests, &tests.join("_lib"), &sources()).unwrap();
        assert_eq!(place, Place::Check);
        assert_eq!(classify(p, &place, false, true), ("none", CHECK.into()));
    }

    #[test]
    fn us_070_api_06_rld_02_a_library_file_is_level_none_inside_or_outside_the_tests_folder() {
        let tests = Path::new(r"C:\p\tests");
        let p = Path::new(r"C:\p\tests\_lib\rope.sqf");
        let at = place(p, tests, &tests.join("_lib"), &sources()).unwrap();
        assert_eq!(at, Place::Library);
        assert_eq!(classify(p, &at, false, false), ("none", LIBRARY.into()));
        let p = Path::new(r"C:\p\helpers\rope.sqf");
        let at = place(p, tests, Path::new(r"C:\p\helpers"), &sources()).unwrap();
        assert_eq!(at, Place::Library);
    }

    #[test]
    fn us_056_rld_06_a_new_sqf_needs_a_rebuild_and_a_game_restart() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\functions\fn_new.sqf");
        let place = place(p, Path::new(r"C:\p\tests"), Path::new(r"C:\p\tests\_lib"), &sources()).unwrap();
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
        let place = place(p, Path::new(r"C:\p\tests"), Path::new(r"C:\p\tests\_lib"), &sources()).unwrap();
        let (level, reason) = classify(p, &place, false, false);
        assert_eq!(level, "game-restart");
        assert!(reason.contains("Config is read at game start"));
    }

    #[test]
    fn us_055_us_081_rld_02_a_changed_sqf_is_a_mission_restart_that_runs_the_rebuild() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\functions\fn_version.sqf");
        let place = place(p, Path::new(r"C:\p\tests"), Path::new(r"C:\p\tests\_lib"), &sources()).unwrap();
        let (level, reason) = classify(p, &place, false, false);
        assert_eq!(level, "mission-restart");
        assert!(
            reason.contains("let go of the file") && reason.contains("mission restart") && !reason.contains("rebuild"),
            "{reason}"
        );
        let (_, reason) = classify(p, &place, false, true);
        assert!(
            reason.ends_with("The mod's rebuild runs once no game process holds the file."),
            "{reason}"
        );
        // A config file among the files makes the reload a game restart: the highest level wins.
        assert!(rank("game-restart") > rank("mission-restart") && rank("mission-restart") > rank("none"));
    }

    #[test]
    fn rld_01_a_file_outside_every_watched_folder_is_refused_naming_them() {
        let e = place(
            Path::new(r"C:\elsewhere\x.sqf"),
            Path::new(r"C:\p\tests"),
            Path::new(r"C:\p\tests\_lib"),
            &sources(),
        )
        .unwrap_err();
        assert_eq!(e.code, 3);
        assert!(e.message.contains(r"C:\i\fixtures\skdfx\addons"), "{}", e.message);
    }
}
