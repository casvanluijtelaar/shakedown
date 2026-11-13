//! The project file `shakedown.toml` (`CFG-01`, `contract/cli.md`). Read fresh by every command;
//! an unknown key, a missing required key or a wrong value is refused with the key and its line.

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml::Spanned;

use crate::error::CliError;

pub const FILE: &str = "shakedown.toml";

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub game: PathBuf,
    pub checks: PathBuf,
    pub base_port: u16,
    pub mods: BTreeMap<String, Mod>,
    pub setups: BTreeMap<String, Setup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mod {
    pub name: String,
    pub path: PathBuf,
    pub patch: Option<String>,
    pub source: Option<PathBuf>,
    pub prefix: Option<String>,
    /// The command a game-restart reload runs while no game process runs, which makes the
    /// build the game loads from the developer's files (`RLD-06`).
    #[serde(default)]
    pub rebuild: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Setup {
    pub name: String,
    pub mode: Mode,
    pub clients: u32,
    pub headless: u32,
    pub mods: Vec<String>,
    pub world: String,
    pub spawn: [f64; 2],
    pub start_timeout: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Singleplayer,
    Dedicated,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Singleplayer => "singleplayer",
            Mode::Dedicated => "dedicated",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProject {
    game: Spanned<String>,
    checks: Option<Spanned<String>>,
    base_port: Option<Spanned<i64>>,
    #[serde(default)]
    mods: BTreeMap<Spanned<String>, RawMod>,
    #[serde(default)]
    setups: BTreeMap<Spanned<String>, RawSetup>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMod {
    path: Spanned<String>,
    patch: Option<String>,
    source: Option<Spanned<String>>,
    prefix: Option<String>,
    rebuild: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSetup {
    mode: Mode,
    clients: Option<Spanned<i64>>,
    headless: Option<Spanned<i64>>,
    mods: Spanned<Vec<Spanned<String>>>,
    world: Option<String>,
    spawn: Option<[f64; 2]>,
    start_timeout: Option<Spanned<i64>>,
}

/// Finds the project: the folder given, or the nearest folder from `start` upward with a
/// `shakedown.toml`.
pub fn find_root(given: Option<&Path>, start: &Path) -> Result<PathBuf, CliError> {
    if let Some(dir) = given {
        if dir.join(FILE).is_file() {
            return Ok(dir.to_path_buf());
        }
        return Err(CliError::wrong(format!(
            "{} holds no {FILE}. Give --project the folder that holds the project file.",
            dir.display()
        ))
        .tag("config-error"));
    }
    let mut dir = Some(start);
    while let Some(d) = dir {
        if d.join(FILE).is_file() {
            return Ok(d.to_path_buf());
        }
        dir = d.parent();
    }
    Err(CliError::wrong(format!(
        "No {FILE} in {} or any folder above it. Run the command inside a project, or give --project <dir>.",
        start.display()
    ))
    .tag("config-error"))
}

pub fn load(root: &Path) -> Result<Project, CliError> {
    let path = root.join(FILE);
    let text =
        std::fs::read_to_string(&path).map_err(|e| CliError::wrong(format!("Cannot read {}: {e}.", path.display())))?;
    parse(root, &text).map_err(|e| e.tag("config-error"))
}

/// The path with every `.` dropped and every `..` taking away the part before it, without asking
/// the file system. The game refuses a `-mod` folder whose path holds `..` (`CFG-01`), so every
/// path of the project file reaches a game start in this form.
fn lexical(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !matches!(out.components().next_back(), Some(Component::Normal(_))) || !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Parses the project file text; `root` resolves relative paths.
pub fn parse(root: &Path, text: &str) -> Result<Project, CliError> {
    let line = |span: Range<usize>| text[..span.start.min(text.len())].matches('\n').count() + 1;
    let fail = |span: Range<usize>, msg: String| CliError::wrong(format!("{FILE}, line {}: {msg}", line(span)));
    let raw: RawProject = toml::from_str(text).map_err(|e| {
        let msg = e.message().trim().to_string();
        match e.span() {
            Some(span) => {
                // The key of a value of the wrong type is the left side of its line.
                let start = text[..span.start.min(text.len())].rfind('\n').map_or(0, |i| i + 1);
                let key = text[start..]
                    .lines()
                    .next()
                    .and_then(|l| l.split_once('='))
                    .map(|(k, _)| k.trim())
                    .filter(|k| !k.is_empty() && !msg.contains(&format!("`{k}`")))
                    .map(|k| format!("key `{k}`: "))
                    .unwrap_or_default();
                fail(
                    span,
                    format!("{key}{msg}. Fix the key as contract/cli.md describes the project file."),
                )
            }
            None => CliError::wrong(format!("{FILE}: {msg}.")),
        }
    })?;
    let resolve = |p: &str| {
        let p = PathBuf::from(p);
        lexical(&if p.is_absolute() { p } else { root.join(p) })
    };
    let name_ok = |n: &str| {
        n.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };

    let base_port = match raw.base_port {
        None => 2402,
        Some(p) => {
            let v = *p.get_ref();
            if !(1024..=65530).contains(&v) {
                return Err(fail(
                    p.span(),
                    format!("base_port must be a whole number from 1024 to 65530, got {v}."),
                ));
            }
            v as u16
        }
    };

    let mut mods = BTreeMap::new();
    for (name, m) in raw.mods {
        if !name_ok(name.get_ref()) {
            return Err(fail(
                name.span(),
                format!(
                    "the mod name `{}` must be letters, digits and -, starting with a letter.",
                    name.get_ref()
                ),
            ));
        }
        if m.source.is_some() && m.prefix.is_none() {
            let span = m.source.as_ref().map(|s| s.span()).unwrap_or(name.span());
            return Err(fail(
                span,
                format!("mods.{}.prefix is required with source.", name.get_ref()),
            ));
        }
        let name = name.into_inner();
        mods.insert(
            name.clone(),
            Mod {
                name,
                path: resolve(m.path.get_ref()),
                patch: m.patch,
                source: m.source.map(|s| resolve(s.get_ref())),
                prefix: m.prefix,
                rebuild: m.rebuild,
            },
        );
    }

    let mut setups = BTreeMap::new();
    for (name, s) in raw.setups {
        if !name_ok(name.get_ref()) {
            return Err(fail(
                name.span(),
                format!(
                    "the setup name `{}` must be letters, digits and -, starting with a letter.",
                    name.get_ref()
                ),
            ));
        }
        let ranged = |v: &Option<Spanned<i64>>, key: &str, lo: i64, hi: i64, default: i64| -> Result<i64, CliError> {
            match v {
                None => Ok(default),
                Some(v) if (lo..=hi).contains(v.get_ref()) => Ok(*v.get_ref()),
                Some(v) => Err(fail(
                    v.span(),
                    format!(
                        "setups.{}.{key} must be a whole number from {lo} to {hi}, got {}.",
                        name.get_ref(),
                        v.get_ref()
                    ),
                )),
            }
        };
        let single = s.mode == Mode::Singleplayer;
        let clients = ranged(&s.clients, "clients", 1, 8, 1)?;
        let headless = ranged(&s.headless, "headless", 0, 8, 0)?;
        let start_timeout = ranged(&s.start_timeout, "start_timeout", 30, 1800, 180)?;
        for m in s.mods.get_ref() {
            if !mods.contains_key(m.get_ref()) {
                let known: Vec<&str> = mods.keys().map(String::as_str).collect();
                return Err(fail(
                    m.span(),
                    format!(
                        "setups.{}.mods names the mod `{}`, which [mods] does not declare; it declares: {}.",
                        name.get_ref(),
                        m.get_ref(),
                        if known.is_empty() {
                            "none".to_string()
                        } else {
                            known.join(", ")
                        }
                    ),
                ));
            }
        }
        let name = name.into_inner();
        setups.insert(
            name.clone(),
            Setup {
                name,
                mode: s.mode,
                clients: if single { 1 } else { clients as u32 },
                headless: if single { 0 } else { headless as u32 },
                mods: s.mods.into_inner().into_iter().map(Spanned::into_inner).collect(),
                world: s.world.unwrap_or_else(|| "VR".into()),
                spawn: s.spawn.unwrap_or([100.0, 100.0]),
                start_timeout: start_timeout as u64,
            },
        );
    }

    Ok(Project {
        root: root.to_path_buf(),
        game: resolve(raw.game.get_ref()),
        checks: resolve(raw.checks.as_ref().map(|c| c.get_ref().as_str()).unwrap_or("checks")),
        base_port,
        mods,
        setups,
    })
}

impl Project {
    /// The named setup, or a refusal listing the setups the file declares.
    pub fn setup(&self, name: &str) -> Result<&Setup, CliError> {
        self.setups.get(name).ok_or_else(|| {
            let known: Vec<&str> = self.setups.keys().map(String::as_str).collect();
            CliError::wrong(format!(
                "{FILE} has no setup `{name}`; it declares: {}. Use one of those with --setup.",
                if known.is_empty() {
                    "none".to_string()
                } else {
                    known.join(", ")
                }
            ))
            .tag("unknown-setup")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
game = 'C:\Games\Arma 3'
checks = "checks"

[mods.cba]
path = 'C:\mods\cba'
patch = "cba_main"

[setups.sp]
mode = "singleplayer"
mods = ["cba"]
"#;

    fn root() -> PathBuf {
        PathBuf::from("project")
    }

    #[test]
    fn cfg_01_reads_a_singleplayer_setup_with_defaults() {
        let p = parse(&root(), GOOD).unwrap();
        let sp = p.setup("sp").unwrap();
        assert_eq!(sp.mode, Mode::Singleplayer);
        assert_eq!((sp.clients, sp.headless), (1, 0));
        assert_eq!(sp.world, "VR");
        assert_eq!(sp.spawn, [100.0, 100.0]);
        assert_eq!(sp.start_timeout, 180);
        assert_eq!(p.base_port, 2402);
        assert_eq!(p.checks, root().join("checks"));
        assert_eq!(p.mods["cba"].patch.as_deref(), Some("cba_main"));
    }

    #[test]
    fn cfg_01_m10_1_a_relative_path_that_climbs_out_of_the_project_folder_holds_no_dot_dot() {
        let root = PathBuf::from(r"C:\repo\acceptance\shakedown");
        let text = GOOD.replace(r"C:\mods\cba", r"..\..\.hemttout\.\build");
        let p = parse(&root, &text).unwrap();
        assert_eq!(p.mods["cba"].path, PathBuf::from(r"C:\repo\.hemttout\build"));
        assert_eq!(lexical(Path::new(r"a\..\..\b")), PathBuf::from(r"..\b"));
    }

    #[test]
    fn cfg_01_an_unknown_key_is_refused_with_its_line() {
        let text = format!("{GOOD}colour = \"red\"\n");
        let e = parse(&root(), &text).unwrap_err();
        assert_eq!(e.code, 3);
        assert!(e.message.contains("colour"), "{}", e.message);
        assert!(e.message.contains("line 12"), "{}", e.message);
    }

    #[test]
    fn cfg_01_a_wrong_type_is_refused_with_its_line() {
        let text = GOOD.replace("checks = \"checks\"", "checks = 5");
        let e = parse(&root(), &text).unwrap_err();
        assert!(e.message.contains("line 3"), "{}", e.message);
    }

    #[test]
    fn cfg_01_a_missing_required_key_is_refused() {
        let e = parse(&root(), "[setups.sp]\nmode = \"singleplayer\"\nmods = []\n").unwrap_err();
        assert!(e.message.contains("game"), "{}", e.message);
    }

    #[test]
    fn cfg_01_an_unknown_mod_in_a_setup_is_refused_with_its_line() {
        let text = GOOD.replace("mods = [\"cba\"]", "mods = [\"ace\"]");
        let e = parse(&root(), &text).unwrap_err();
        assert!(
            e.message.contains("line 11") && e.message.contains("ace"),
            "{}",
            e.message
        );
    }

    #[test]
    fn cfg_01_a_value_out_of_range_is_refused() {
        let text = format!("{GOOD}start_timeout = 5\n");
        let e = parse(&root(), &text).unwrap_err();
        assert!(
            e.message.contains("start_timeout") && e.message.contains("line 12"),
            "{}",
            e.message
        );
    }

    #[test]
    fn cfg_01_an_unknown_setup_lists_the_declared_ones() {
        let p = parse(&root(), GOOD).unwrap();
        let e = p.setup("dedicated-2").unwrap_err();
        assert!(e.message.contains("sp"), "{}", e.message);
    }
}
