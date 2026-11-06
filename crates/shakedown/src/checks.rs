//! Check files: finding them, reading their header (`API-01`), selecting them (`RUN-01`,
//! `RUN-03`) and preparing the body that is sent to the game.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CliError;

/// A check file found under the checks folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRef {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub tags: Vec<String>,
    pub setups: Option<Vec<String>>,
    pub timeout_s: u64,
    pub reset: String,
}

const KEYS: [&str; 5] = ["check", "tags", "setups", "timeout", "reset"];

/// Every `.sqf` file under `dir`, with its check id, in check id order.
pub fn discover(dir: &Path) -> Result<Vec<CheckRef>, CliError> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<CheckRef>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(base, &path, out)?;
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sqf")) {
                let rel = path.strip_prefix(base).unwrap_or(&path).with_extension("");
                let id = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(CheckRef { id, path });
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    if !dir.is_dir() {
        return Err(CliError::wrong(format!(
            "The checks folder {} does not exist. Create it, or set `checks` in shakedown.toml.",
            dir.display()
        )));
    }
    walk(dir, dir, &mut out)
        .map_err(|e| CliError::framework(format!("Cannot read the checks folder {}: {e}.", dir.display())))?;
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Whether a word has the form of a story or requirement ID: capital letters and digits in parts
/// joined by `-`, starting with a letter and holding a digit, such as `US-012`, `NFR-PF-02` or `B3`.
fn is_id(word: &str) -> bool {
    word.starts_with(|c: char| c.is_ascii_uppercase())
        && word.chars().any(|c| c.is_ascii_digit())
        && word
            .split('-')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
}

/// Reads the header of a check file. An error names the line and the key (`US-038`). A UTF-8 byte
/// order mark before the first line is skipped.
pub fn parse_header(text: &str) -> Result<Header, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut seen: Vec<(String, usize, String)> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let Some(rest) = line.strip_prefix("//").map(str::trim_start) else {
            break;
        };
        let Some(rest) = rest.strip_prefix('@') else { break };
        let (key, value) = match rest.split_once(char::is_whitespace) {
            Some((k, v)) => (k, v.trim()),
            None => (rest, ""),
        };
        if !KEYS.contains(&key) {
            return Err(format!(
                "line {n}: unknown header key @{key}; a header has only @check, @tags, @setups, @timeout and @reset."
            ));
        }
        if let Some((_, first, _)) = seen.iter().find(|(k, _, _)| k == key) {
            return Err(format!(
                "line {n}: @{key} is given twice, first on line {first}. Keep one."
            ));
        }
        seen.push((key.to_string(), n, value.to_string()));
    }
    let get = |k: &str| seen.iter().find(|(key, _, _)| key == k);
    let Some((_, line, name)) = get("check") else {
        return Err("the header has no @check line. Start the file with `// @check <story or requirement ID> <what it proves>`.".into());
    };
    if name.is_empty() {
        return Err(format!(
            "line {line}: @check has no name. Give the story or requirement ID it proves, then a short sentence."
        ));
    }
    let first = name.split_whitespace().next().unwrap_or_default();
    if !is_id(first) {
        return Err(format!(
            "line {line}: @check must start with the story or requirement ID the check proves, such as US-012 or NFR-PF-02, then a short sentence; it starts with `{first}`."
        ));
    }
    let words = |v: &str| v.split_whitespace().map(String::from).collect::<Vec<_>>();
    let timeout_s = match get("timeout") {
        None => 120,
        Some((_, line, v)) => match v.parse::<u64>() {
            Ok(t) if (1..=3600).contains(&t) => t,
            _ => {
                return Err(format!(
                    "line {line}: @timeout must be a whole number of seconds from 1 to 3600, got `{v}`."
                ));
            }
        },
    };
    let reset = match get("reset") {
        None => "in-place".to_string(),
        Some((_, line, v)) => {
            if v != "in-place" && v != "mission" {
                return Err(format!(
                    "line {line}: @reset must be `in-place` or `mission`, got `{v}`."
                ));
            }
            v.clone()
        }
    };
    if let Some((_, line, v)) = get("setups")
        && v.is_empty()
    {
        return Err(format!(
            "line {line}: @setups names no setup. Name one or more, or leave the line out."
        ));
    }
    Ok(Header {
        name: name.clone(),
        tags: get("tags").map(|(_, _, v)| words(v)).unwrap_or_default(),
        setups: get("setups").map(|(_, _, v)| words(v)),
        timeout_s,
        reset,
    })
}

/// The body sent to the game: the whole file with every comment, the header's included, turned
/// into spaces, because `compile` takes no comments. Line breaks stay, so the engine's line
/// numbers are the file's.
pub fn body_source(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    #[derive(PartialEq)]
    enum S {
        Code,
        Double,
        Single,
        Line,
        Block,
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut state = S::Code;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match state {
            S::Code => match (c, next) {
                ('/', Some('/')) => {
                    state = S::Line;
                    out.push_str("  ");
                    i += 1;
                }
                ('/', Some('*')) => {
                    state = S::Block;
                    out.push_str("  ");
                    i += 1;
                }
                ('"', _) => {
                    state = S::Double;
                    out.push(c);
                }
                ('\'', _) => {
                    state = S::Single;
                    out.push(c);
                }
                _ => out.push(c),
            },
            S::Double | S::Single => {
                let quote = if state == S::Double { '"' } else { '\'' };
                out.push(c);
                if c == quote {
                    if next == Some(quote) {
                        out.push(quote);
                        i += 1;
                    } else {
                        state = S::Code;
                    }
                }
            }
            S::Line => {
                if c == '\n' {
                    state = S::Code;
                    out.push(c);
                } else if c != '\r' {
                    out.push(' ');
                } else {
                    out.push(c);
                }
            }
            S::Block => {
                if c == '*' && next == Some('/') {
                    state = S::Code;
                    out.push_str("  ");
                    i += 1;
                } else if c == '\n' || c == '\r' {
                    out.push(c);
                } else {
                    out.push(' ');
                }
            }
        }
        i += 1;
    }
    out
}

/// One listed check, as `list` and `list_checks` return it. `error` names what is wrong with a
/// header that does not parse; the other fields are then empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listed {
    pub id: String,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub setups: Option<Vec<String>>,
    pub timeout_s: Option<u64>,
    pub reset: Option<String>,
    pub error: Option<String>,
}

pub fn read_listed(check: &CheckRef) -> Listed {
    let parsed = std::fs::read_to_string(&check.path)
        .map_err(|e| format!("cannot read the file: {e}"))
        .and_then(|t| parse_header(&t));
    match parsed {
        Ok(h) => Listed {
            id: check.id.clone(),
            name: Some(h.name),
            tags: h.tags,
            setups: h.setups,
            timeout_s: Some(h.timeout_s),
            reset: Some(h.reset),
            error: None,
        },
        Err(e) => Listed {
            id: check.id.clone(),
            name: None,
            tags: vec![],
            setups: None,
            timeout_s: None,
            reset: None,
            error: Some(e),
        },
    }
}

/// Selects checks by ids and tags (`RUN-01`): the union, ids in the order given, an id given more
/// than once each time, then the checks a tag selects and no id gave, in check id order (`RUN-03`).
/// No selection selects every check. An id that names no
/// check, or a selection that matches nothing, is refused with what exists.
pub fn select(all: &[CheckRef], ids: &[String], tags: &[String]) -> Result<Vec<CheckRef>, CliError> {
    if ids.is_empty() && tags.is_empty() {
        if all.is_empty() {
            return Err(CliError::wrong(
                "The checks folder holds no check. Add a .sqf file with a `// @check` header.",
            ));
        }
        return Ok(all.to_vec());
    }
    let mut out: Vec<CheckRef> = Vec::new();
    for id in ids {
        let id = id.trim_end_matches(".sqf").replace('\\', "/");
        match all.iter().find(|c| c.id == id) {
            Some(c) => out.push(c.clone()),
            None => {
                return Err(CliError::wrong(format!(
                    "No check has the id `{id}`. The checks are: {}.",
                    all.iter().map(|c| c.id.as_str()).collect::<Vec<_>>().join(", ")
                )));
            }
        }
    }
    if !tags.is_empty() {
        for c in all {
            let listed = read_listed(c);
            if listed.tags.iter().any(|t| tags.contains(t)) && !out.contains(c) {
                out.push(c.clone());
            }
        }
    }
    if out.is_empty() {
        return Err(CliError::wrong(format!(
            "No check carries the tag {}. Run `shakedown list` to see each check's tags.",
            tags.join(" or ")
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_01_reads_a_full_header() {
        let h = parse_header(
            "// @check US-019 A player uses the hook\n// @tags US-019 smoke\n\n// @setups sp\n// @timeout 30\n// @reset mission\n[] call x;",
        )
        .unwrap();
        assert_eq!(h.name, "US-019 A player uses the hook");
        assert_eq!(h.tags, vec!["US-019", "smoke"]);
        assert_eq!(h.setups, Some(vec!["sp".to_string()]));
        assert_eq!((h.timeout_s, h.reset.as_str()), (30, "mission"));
    }

    #[test]
    fn api_01_defaults() {
        let h = parse_header("// @check US-037 x\nfoo").unwrap();
        assert_eq!((h.timeout_s, h.reset.as_str(), h.setups), (120, "in-place", None));
        assert!(h.tags.is_empty());
    }

    #[test]
    fn us_038_a_missing_check_line_is_named() {
        let e = parse_header("// @tags smoke\nfoo").unwrap_err();
        assert!(e.contains("@check"), "{e}");
    }

    #[test]
    fn us_038_a_bad_timeout_names_the_line_and_key() {
        let e = parse_header("// @check US-038 x\n// @timeout fast\n").unwrap_err();
        assert!(
            e.contains("line 2") && e.contains("@timeout") && e.contains("fast"),
            "{e}"
        );
    }

    #[test]
    fn us_038_a_key_given_twice_names_both_lines() {
        let e = parse_header("// @check US-038 x\n// @tags a\n// @tags b\n").unwrap_err();
        assert!(
            e.contains("line 3") && e.contains("@tags") && e.contains("line 2"),
            "{e}"
        );
    }

    #[test]
    fn us_038_m01_1_a_name_without_an_id_is_refused_naming_the_line() {
        let e = parse_header("// @tags smoke\n// @check A hook holds a player\n").unwrap_err();
        assert!(e.contains("line 2") && e.contains("@check") && e.contains("`A`"), "{e}");
        for name in ["US-012 x", "NFR-PF-02 x", "B3 x", "SPK-04 x", "RST-01"] {
            assert!(parse_header(&format!("// @check {name}\n")).is_ok(), "{name}");
        }
        for name in ["us-012 x", "US- x", "Smoke test", "US-012x y"] {
            assert!(parse_header(&format!("// @check {name}\n")).is_err(), "{name}");
        }
    }

    #[test]
    fn def_05_a_byte_order_mark_is_skipped() {
        let text = "\u{feff}// @check US-037 x\n// @tags a\n[] call x;\n";
        let h = parse_header(text).unwrap();
        assert_eq!((h.name.as_str(), h.tags.clone()), ("US-037 x", vec!["a".to_string()]));
        let body = body_source(text);
        assert!(!body.contains('\u{feff}'));
        assert_eq!(body.lines().count(), 3);
    }

    #[test]
    fn us_038_an_unknown_key_is_named() {
        let e = parse_header("// @check US-038 x\n// @owner me\n").unwrap_err();
        assert!(e.contains("line 2") && e.contains("@owner"), "{e}");
    }

    #[test]
    fn api_01_the_header_ends_at_the_first_other_line() {
        let h = parse_header("// @check US-037 x\nprivate _a = 1;\n// @timeout fast\n").unwrap();
        assert_eq!(h.timeout_s, 120);
    }

    #[test]
    fn api_02_comments_become_spaces_and_lines_stay() {
        let src = "// @check x\nprivate _s = \"a // not a comment\"; /* gone\n still */ _s // end\n'it''s'";
        let body = body_source(src);
        assert_eq!(body.lines().count(), src.lines().count());
        assert!(body.contains("\"a // not a comment\""));
        assert!(!body.contains("gone") && !body.contains("still") && !body.contains("end") && !body.contains("@check"));
        assert!(body.contains("'it''s'"));
    }

    fn refs(ids: &[&str]) -> Vec<CheckRef> {
        ids.iter()
            .map(|id| CheckRef {
                id: id.to_string(),
                path: PathBuf::from(format!("{id}.sqf")),
            })
            .collect()
    }

    #[test]
    fn run_01_ids_keep_their_order_and_unknown_ids_are_refused() {
        let all = refs(&["a", "b", "c"]);
        let got = select(&all, &["c".into(), "a".into()], &[]).unwrap();
        assert_eq!(got.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), vec!["c", "a"]);
        let e = select(&all, &["zz".into()], &[]).unwrap_err();
        assert_eq!(e.code, 3);
        assert!(e.message.contains("a, b, c"), "{}", e.message);
    }

    #[test]
    fn us_031_m01_3_an_id_given_more_than_once_is_selected_each_time() {
        let all = refs(&["a", "b"]);
        let ids = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let names = |v: Vec<CheckRef>| v.into_iter().map(|c| c.id).collect::<Vec<_>>();
        assert_eq!(
            names(select(&all, &ids(&["a", "a", "b"]), &[]).unwrap()),
            ["a", "a", "b"]
        );
        assert_eq!(
            names(select(&all, &ids(&["a", "b", "a"]), &[]).unwrap()),
            ["a", "b", "a"]
        );
        assert_eq!(
            names(select(&all, &ids(&["a", "a", "a"]), &[]).unwrap()),
            ["a", "a", "a"]
        );
    }

    #[test]
    fn run_01_no_selection_selects_every_check() {
        let all = refs(&["a", "b"]);
        assert_eq!(select(&all, &[], &[]).unwrap().len(), 2);
    }
}
