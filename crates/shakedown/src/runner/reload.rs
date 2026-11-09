//! A reload (`P-08`, `RLD-01` to `RLD-06`): the changed files found or given, each classified by
//! `RLD-02`, the highest level applied, and the result in the Reload shape of
//! `contract/mcp-tools.md`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value, json};
use shakedown_protocol::time;

use super::Runner;
use super::session::{Session, SessionState, stamp};
use crate::error::CliError;

/// The levels of `P-08`, lowest first.
const LEVELS: [&str; 4] = ["none", "mission", "mission-restart", "game-restart"];

fn rank(level: &str) -> usize {
    LEVELS.iter().position(|l| *l == level).unwrap_or(0)
}

const CHECK: &str = "Checks are read fresh for every execution (RLD-04).";
const NEW_FILE: &str = "A new or deleted file: file patching finds only files that exist in the built PBO, so the mod must be rebuilt before this reload, which restarts the game.";
const CONFIG: &str = "Config is read at game start, so the game restarts.";
const OTHER: &str = "File patching does not reload this kind of file; it is read at game start, so the game restarts.";

/// The extensions of the files the game reads as config (`RLD-02`).
const CONFIG_EXTS: [&str; 6] = ["cpp", "hpp", "h", "inc", "ext", "xml"];

/// One changed file, classified.
#[derive(Debug, Clone, PartialEq)]
pub struct Classified {
    pub path: PathBuf,
    pub level: &'static str,
    pub reason: String,
}

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

/// Where a file is: a check, or a file of a watched source with its game path (`INT-HEM-01`).
#[derive(Debug, PartialEq)]
pub enum Place {
    Check,
    Source { game_path: String },
}

/// Places a file by the folders it is under; an error names the watched folders.
pub fn place(path: &Path, checks: &Path, sources: &[(String, PathBuf, String)]) -> Result<Place, CliError> {
    for (_, dir, prefix) in sources {
        if let Some(rel) = below(path, dir) {
            let prefix = prefix.replace('/', "\\").trim_matches('\\').to_lowercase();
            return Ok(Place::Source {
                game_path: format!("{prefix}\\{rel}"),
            });
        }
    }
    if below(path, checks).is_some() {
        return Ok(Place::Check);
    }
    let watched: Vec<String> = sources.iter().map(|(_, d, _)| d.display().to_string()).collect();
    Err(CliError::wrong(format!(
        "{} is not under a watched source folder or the checks folder. Give a file under {}{}.",
        path.display(),
        if watched.is_empty() {
            "a mod's `source` (no mod of this setup sets one in shakedown.toml)".to_string()
        } else {
            watched.join(" or ")
        },
        format_args!(", or under {}", checks.display())
    ))
    .tag("bad-arguments"))
}

/// Classifies a file that is not an `.sqf` of a watched source, or is new or deleted
/// (`RLD-02`). `None` for an `.sqf` that exists in the snapshot, which the lead must look at.
pub fn classify_plain(path: &Path, place: &Place, is_new_or_deleted: bool) -> Option<(&'static str, String)> {
    match place {
        Place::Check => Some(("none", CHECK.into())),
        Place::Source { .. } if is_new_or_deleted => Some(("game-restart", NEW_FILE.into())),
        Place::Source { .. } => {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if ext == "sqf" {
                None
            } else if CONFIG_EXTS.contains(&ext.as_str()) {
                Some(("game-restart", CONFIG.into()))
            } else {
                Some(("game-restart", OTHER.into()))
            }
        }
    }
}

/// The level and reason of an `.sqf` of a watched source from what every machine said of the
/// `CfgFunctions` functions it defines: each function's name and whether it is final there.
pub fn classify_sqf(defined: &[(String, bool)]) -> (&'static str, String) {
    if defined.is_empty() {
        return (
            "mission-restart",
            "It defines no CfgFunctions function, so it is final code or code run at mission start, such as a CBA PREP function, which is final: final code changes only at mission start, with CBA's cache off in the session mission.".into(),
        );
    }
    let mut names: Vec<&str> = defined.iter().map(|(n, _)| n.as_str()).collect();
    names.sort();
    names.dedup();
    let finals: Vec<&str> = defined.iter().filter(|(_, f)| *f).map(|(n, _)| n.as_str()).collect();
    if finals.is_empty() {
        (
            "mission",
            format!(
                "It defines {}, a CfgFunctions function that is not final on any machine: recompiled in place.",
                names.join(", ")
            ),
        )
    } else {
        let mut finals = finals;
        finals.sort();
        finals.dedup();
        (
            "mission-restart",
            format!(
                "It defines {}, which is final on a machine: final code changes only at mission start, with CBA's cache off in the session mission.",
                finals.join(", ")
            ),
        )
    }
}

/// Asks every machine which `CfgFunctions` functions each game path defines and whether each is
/// final there (`P-08` step 1).
async fn ask_functions(session: &Session, game_paths: &[String]) -> Result<Vec<Vec<(String, bool)>>, String> {
    let source = r#"params ["_paths"];
private _out = _paths apply { [] };
{
    private _tagCfg = _x;
    private _tag = getText (_tagCfg >> "tag");
    if (_tag isEqualTo "") then { _tag = configName _tagCfg };
    private _tagFile = getText (_tagCfg >> "file");
    {
        private _cat = _x;
        private _catFile = getText (_cat >> "file");
        {
            private _fn = _x;
            private _ext = getText (_fn >> "ext");
            if (_ext isEqualTo "") then { _ext = ".sqf" };
            private _file = getText (_fn >> "file");
            if (_file isEqualTo "") then {
                _file = switch (true) do {
                    case (_catFile isNotEqualTo ""): { format ["%1\fn_%2%3", _catFile, configName _fn, _ext] };
                    case (_tagFile isNotEqualTo ""): { format ["%1\%2\fn_%3%4", _tagFile, configName _cat, configName _fn, _ext] };
                    default { format ["functions\%1\fn_%2%3", configName _cat, configName _fn, _ext] };
                };
            };
            _file = toLower _file;
            if ((_file select [0, 1]) isEqualTo "\") then { _file = _file select [1] };
            private _i = _paths find _file;
            if (_i >= 0) then {
                private _name = format ["%1_fnc_%2", _tag, configName _fn];
                private _code = missionNamespace getVariable [_name, {}];
                (_out select _i) pushBack [_name, isFinal _code];
            };
        } forEach ("true" configClasses _cat);
    } forEach ("true" configClasses _tagCfg);
} forEach ("true" configClasses (configFile >> "CfgFunctions"));
_out"#;
    let mut all: Vec<Vec<(String, bool)>> = vec![vec![]; game_paths.len()];
    for machine in session.machine_names() {
        let answer = session
            .exec(&machine, source, json!([game_paths]), 10.0)
            .await
            .map_err(|e| format!("{machine} did not answer which functions the files define ({e:?})"))?;
        let value = &answer["value"];
        for (i, defined) in value.as_array().into_iter().flatten().enumerate() {
            for d in defined.as_array().into_iter().flatten() {
                if let (Some(name), Some(fin)) = (d[0].as_str(), d[1].as_bool())
                    && i < all.len()
                {
                    all[i].push((name.to_string(), fin));
                }
            }
        }
    }
    Ok(all)
}

/// `reload [files...]` (`RLD-01`): refused with no session, and while a run, a reload or a reset
/// is in progress; on a `broken` session it restarts the game, the one way back (`P-07`). Returns
/// once the reload is done, or after `wait_s` with state `applying`, the reload going on.
pub async fn reload(
    runner: &Arc<Runner>,
    files: Vec<PathBuf>,
    checks: PathBuf,
    wait_s: Option<f64>,
) -> Result<Value, CliError> {
    let Some(session) = runner.session() else {
        return Err(CliError::not_now(
            "No session runs, so there is nothing to reload. Start one with `shakedown session start --setup <name>`.",
        )
        .tag("no-session"));
    };
    let broken = match session.state() {
        SessionState::Ready => false,
        SessionState::Broken => true,
        SessionState::Running => {
            let progress = runner.progress.lock().unwrap().clone().unwrap_or(Value::Null);
            return Err(CliError::not_now(format!(
                "Session {} is running run {} ({} of {} checks done), so it cannot reload now. Wait for the run to end, or stop it with `shakedown run stop`.",
                session.id,
                progress["id"].as_str().unwrap_or("?"),
                progress["done"],
                progress["total"]
            ))
            .tag("session-busy"));
        }
        SessionState::Reloading => {
            return Err(CliError::not_now(format!(
                "Session {} is busy with a reload or a reset. Wait for it to end; `shakedown status` shows it.",
                session.id
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
        let place = place(p, &checks, &sources)?;
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
    if !session.transition(from, SessionState::Reloading) {
        return Err(CliError::not_now(format!(
            "Session {} became busy; try the reload again when it is ready.",
            session.id
        ))
        .tag("session-busy"));
    }
    let id = format!("rl-{}-{}", time::stamp(SystemTime::now()), runner.next_run_number());
    let started_at = time::now();
    let clock = Instant::now();

    // Classify (RLD-02): what needs the lead's answer is asked for together.
    let mut classified: Vec<Option<(&'static str, String)>> = Vec::new();
    let mut ask: Vec<(usize, String)> = Vec::new();
    for (i, (p, place)) in paths.iter().zip(&places).enumerate() {
        let new_or_deleted = !in_before.contains(&lower(p)) || !p.exists();
        let plain = classify_plain(p, place, new_or_deleted);
        if plain.is_none()
            && let Place::Source { game_path } = place
        {
            ask.push((i, game_path.clone()));
        }
        classified.push(plain);
    }
    if !ask.is_empty() {
        let game_paths: Vec<String> = ask.iter().map(|(_, g)| g.clone()).collect();
        match ask_functions(&session, &game_paths).await {
            Ok(defined) => {
                for ((i, _), d) in ask.iter().zip(defined) {
                    classified[*i] = Some(classify_sqf(&d));
                }
            }
            Err(why) if broken => {
                // A broken session restarts the game whatever the files need.
                for (i, _) in &ask {
                    classified[*i] = Some(("mission-restart", why.clone()));
                }
            }
            Err(why) => {
                session.transition(SessionState::Reloading, SessionState::Ready);
                return Err(CliError::not_now(format!(
                    "The reload could not classify the changed files: {why}. Look at the session with `shakedown status`, and try again."
                )));
            }
        }
    }
    let files: Vec<Value> = paths
        .iter()
        .zip(&classified)
        .map(|(p, c)| {
            let (level, reason) = c.clone().unwrap_or(("none", String::new()));
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
    // A change that needs a mission restart, or a recompile in place, is not applied: Arma 3
    // keeps every loose file it has read open and serves it from that handle for the life of its
    // process, so no running process takes a changed file (SPK-08, research 27).
    if matches!(level.as_str(), "mission" | "mission-restart") {
        session.transition(SessionState::Reloading, SessionState::Ready);
        let names: Vec<String> = files
            .iter()
            .filter(|f| matches!(f["level"].as_str(), Some("mission" | "mission-restart")))
            .map(|f| f["path"].as_str().unwrap_or("?").to_string())
            .collect();
        return Err(CliError::not_now(format!(
            "{} needs level {level}, which this version does not apply: the game keeps every mod file it has read open for the life of its process, so a running machine never sees the change. Stop the session with `shakedown session stop` and start it again to load the change.",
            names.join(", ")
        ))
        .tag("session-busy"));
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
                session.game_restart(&runner, &|_| {}).await
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
        let checks = Path::new(r"C:\p\checks");
        let p = Path::new(r"C:\p\checks\smoke.sqf");
        let place = place(p, checks, &sources()).unwrap();
        assert_eq!(place, Place::Check);
        assert_eq!(classify_plain(p, &place, false), Some(("none", CHECK.into())));
    }

    #[test]
    fn us_056_rld_06_a_new_sqf_needs_a_rebuild_and_a_game_restart() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\functions\fn_new.sqf");
        let place = place(p, Path::new(r"C:\p\checks"), &sources()).unwrap();
        assert_eq!(
            place,
            Place::Source {
                game_path: r"skdfx\addons\main\functions\fn_new.sqf".into()
            }
        );
        let (level, reason) = classify_plain(p, &place, true).unwrap();
        assert_eq!(level, "game-restart");
        assert!(reason.contains("rebuilt before this reload") && reason.contains("built PBO"));
    }

    #[test]
    fn us_056_rld_02_config_is_a_game_restart() {
        let p = Path::new(r"C:\i\fixtures\skdfx\addons\main\config.cpp");
        let place = place(p, Path::new(r"C:\p\checks"), &sources()).unwrap();
        let (level, reason) = classify_plain(p, &place, false).unwrap();
        assert_eq!(level, "game-restart");
        assert!(reason.contains("Config is read at game start"));
    }

    #[test]
    fn rld_02_an_sqf_is_classified_by_the_functions_it_defines() {
        assert_eq!(classify_sqf(&[("skdfx_fnc_version".into(), false)]).0, "mission");
        assert_eq!(
            classify_sqf(&[("skdfx_fnc_version".into(), false), ("skdfx_fnc_version".into(), true)]).0,
            "mission-restart"
        );
        let (level, reason) = classify_sqf(&[]);
        assert_eq!(level, "mission-restart");
        assert!(reason.contains("final"));
    }

    #[test]
    fn rld_01_a_file_outside_every_watched_folder_is_refused_naming_them() {
        let e = place(Path::new(r"C:\elsewhere\x.sqf"), Path::new(r"C:\p\checks"), &sources()).unwrap_err();
        assert_eq!(e.code, 3);
        assert!(e.message.contains(r"C:\i\fixtures\skdfx\addons"), "{}", e.message);
    }
}
