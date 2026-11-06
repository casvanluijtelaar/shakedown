//! Machine logs: the redaction of the Steam identity (`LOG-02`, `INT-STM-04`), reading a machine's
//! RPT while its game runs (`SPK-13`), the agent's marker lines that cut a check's lines out of it
//! (`LOG-04`), and the script errors an RPT shows (`ERR-02`, `INT-AR-07`).

use std::path::{Path, PathBuf};

use serde_json::Value;

/// What stands in for a Steam ID and for a Steam name (`LOG-02`).
pub const STEAM_ID: &str = "<steamid>";
pub const STEAM_NAME: &str = "<steamname>";

/// The logged-in Steam account's names, held in memory only (`INT-STM-04`), and the rule that
/// replaces them and every Steam ID in a text (`LOG-02`).
#[derive(Debug, Clone, Default)]
pub struct Redactor {
    names: Vec<String>,
}

impl Redactor {
    pub fn new(names: Vec<String>) -> Redactor {
        let mut names: Vec<String> = names.into_iter().filter(|n| !n.trim().is_empty()).collect();
        // A longer name first, so a name that holds another is replaced whole.
        names.sort_by_key(|n| std::cmp::Reverse(n.len()));
        names.dedup();
        Redactor { names }
    }

    /// Reads the names of the account Steam last logged in with: `SteamPath` under
    /// `HKCU\Software\Valve\Steam`, then the `AccountName` and `PersonaName` of the entry of
    /// `config\loginusers.vdf` whose `MostRecent` is `1`. A machine without them redacts Steam IDs
    /// only.
    pub fn from_steam() -> Redactor {
        let Some(steam) = steam_path() else {
            return Redactor::default();
        };
        let text = std::fs::read(steam.join("config").join("loginusers.vdf"))
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        Redactor::new(most_recent_names(&text))
    }

    /// The text with every Steam ID and every name of the account replaced.
    pub fn redact(&self, text: &str) -> String {
        let mut out = steam_ids(text);
        for name in &self.names {
            out = whole_word(&out, name, STEAM_NAME);
        }
        out
    }

    /// Redacts every string in a JSON value, keys included.
    pub fn redact_value(&self, value: &mut Value) {
        match value {
            Value::String(s) => *s = self.redact(s),
            Value::Array(a) => a.iter_mut().for_each(|v| self.redact_value(v)),
            Value::Object(o) => {
                let entries: Vec<(String, Value)> = std::mem::take(o).into_iter().collect();
                for (k, mut v) in entries {
                    self.redact_value(&mut v);
                    o.insert(self.redact(&k), v);
                }
            }
            _ => {}
        }
    }

    /// Rewrites a file with its text redacted.
    pub fn redact_file(&self, path: &Path) -> std::io::Result<()> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        let redacted = self.redact(&text);
        if redacted != text {
            std::fs::write(path, redacted)?;
        }
        Ok(())
    }
}

#[cfg(windows)]
fn steam_path() -> Option<PathBuf> {
    let out = std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.contains("SteamPath"))?;
    let value = line.split("REG_SZ").nth(1)?.trim();
    Some(PathBuf::from(value))
}

#[cfg(not(windows))]
fn steam_path() -> Option<PathBuf> {
    None
}

/// The `AccountName` and `PersonaName` of the `loginusers.vdf` entry of the account Steam last
/// logged in with: the one whose `MostRecent` is `1`, or, in a file whose entries carry no
/// `MostRecent`, the one with the newest `timestamp`.
fn most_recent_names(vdf: &str) -> Vec<String> {
    // The file is a tree of quoted keys and values; each account is one block of pairs.
    let mut blocks: Vec<Vec<(String, String)>> = Vec::new();
    let mut current: Vec<(String, String)> = Vec::new();
    for line in vdf.lines() {
        let line = line.trim();
        if line == "}" {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
            continue;
        }
        let parts: Vec<&str> = line.split('"').collect();
        // `"key"  "value"` splits into ["", key, ws, value, ""].
        if parts.len() >= 5 {
            current.push((parts[1].to_string(), parts[3].to_string()));
        }
    }
    let get = |b: &Vec<(String, String)>, k: &str| {
        b.iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(k))
            .map(|(_, v)| v.clone())
    };
    let stamp = |b: &Vec<(String, String)>| get(b, "timestamp").and_then(|t| t.parse::<u64>().ok()).unwrap_or(0);
    blocks
        .iter()
        .find(|b| get(b, "MostRecent").as_deref() == Some("1"))
        .or_else(|| blocks.iter().max_by_key(|b| stamp(b)))
        .map(|b| {
            ["AccountName", "PersonaName"]
                .iter()
                .filter_map(|k| get(b, k))
                .collect()
        })
        .unwrap_or_default()
}

/// Replaces every 17-digit number starting `7656119`, not part of a longer number.
fn steam_ids(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut last = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i - start == 17 && text[start..i].starts_with("7656119") {
                out.push_str(&text[last..start]);
                out.push_str(STEAM_ID);
                last = i;
            }
        } else {
            i += 1;
        }
    }
    out.push_str(&text[last..]);
    out
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Replaces `word` where it stands as a whole word, case-sensitively.
fn whole_word(text: &str, word: &str, with: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut before: Option<char> = None;
    while let Some(at) = rest.find(word) {
        let prev = rest[..at].chars().next_back().or(before);
        let next = rest[at + word.len()..].chars().next();
        let starts = word.chars().next().is_some_and(is_word);
        let ends = word.chars().next_back().is_some_and(is_word);
        let whole = !(starts && prev.is_some_and(is_word)) && !(ends && next.is_some_and(is_word));
        let (head, tail) = rest.split_at(at + word.len());
        if whole {
            out.push_str(&head[..at]);
            out.push_str(with);
        } else {
            out.push_str(head);
        }
        before = head.chars().next_back();
        rest = tail;
    }
    out.push_str(rest);
    out
}

/// The newest RPT in a machine's profile folder: each start of the game writes a new one.
pub fn newest_rpt(profile: &Path) -> Option<PathBuf> {
    std::fs::read_dir(profile)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("rpt")))
        .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
}

/// The text of a file that a running game may be writing, read beside it: the engine writes the
/// RPT in blocks about every 0.5 s, and a reader may open it at any time (`SPK-13`).
pub fn read_text(path: &Path) -> String {
    std::fs::read(path)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

/// The marker the agent writes into its RPT at the start and end of a run or a check execution
/// (`LOG-04`): `[shakedown] <key> <edge> <utc>`, with `key` `<run id>` for the run and
/// `<run id>#<n>` for its `n`th check execution.
pub fn marker(key: &str, edge: &str) -> String {
    format!("[shakedown] {key} {edge} ")
}

/// The lines from the line holding the start marker of `key` to the line holding its end marker,
/// both included; to the end of the text when the end marker is not there.
pub fn cut(text: &str, key: &str) -> Option<String> {
    let start = marker(key, "start");
    let end = marker(key, "end");
    let lines: Vec<&str> = text.lines().collect();
    let from = lines.iter().position(|l| l.contains(&start))?;
    let to = lines[from..]
        .iter()
        .position(|l| l.contains(&end))
        .map_or(lines.len(), |i| from + i + 1);
    Some(lines[from..to].iter().map(|l| format!("{l}\n")).collect())
}

/// The UTC wall-clock time, to the second, of an RPT line written at local time of day `local_s`,
/// from a marker line, which carries both its local time and its UTC time: the RPT's times are
/// local, and the offset is taken to the nearest 15 minutes.
pub fn utc_of(marker_line: &str, local_s: u32) -> Option<String> {
    let (marker_local, rest) = timestamp(marker_line)?;
    let utc = rest.split_whitespace().last()?;
    let (date, clock) = utc.split_once('T')?;
    let parts: Vec<i64> = clock
        .trim_end_matches('Z')
        .split([':', '.'])
        .take(3)
        .map(|p| p.parse().ok())
        .collect::<Option<_>>()?;
    if parts.len() != 3 {
        return None;
    }
    let marker_utc = parts[0] * 3600 + parts[1] * 60 + parts[2];
    let offset = ((i64::from(marker_local) - marker_utc) as f64 / 900.0).round() as i64 * 900;
    let t = (i64::from(local_s) - offset).rem_euclid(86_400);
    Some(format!(
        "{date}T{:02}:{:02}:{:02}.000Z",
        t / 3600,
        (t % 3600) / 60,
        t % 60
    ))
}

/// One script error an RPT shows (`INT-AR-07`).
#[derive(Debug, Clone, PartialEq)]
pub struct RptError {
    /// The message the engine gave, as the `ScriptError` handler gets it.
    pub text: String,
    /// The file, as the engine names it; empty for code that has none.
    pub file: String,
    pub line: Option<i64>,
    /// The local time of day of the error's first line, in seconds.
    pub local_s: Option<u32>,
}

/// The local time of day an RPT line starts with, `h:mm:ss` or `hh:mm:ss`, in seconds, and the
/// rest of the line after the one space that follows it.
pub fn timestamp(line: &str) -> Option<(u32, &str)> {
    let t = line.trim_start();
    let (stamp, rest) = t.split_once(' ').unwrap_or((t, ""));
    let parts: Vec<&str> = stamp.split(':').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || p.len() > 2 || !p.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let n: Vec<u32> = parts.iter().map(|p| p.parse().unwrap()).collect();
    if n[0] > 23 || n[1] > 59 || n[2] > 60 {
        return None;
    }
    Some((n[0] * 3600 + n[1] * 60 + n[2], rest))
}

/// The script errors in an RPT text (`INT-AR-07`). An error starts at a line whose text after
/// the timestamp starts with `Error ` or holds `Error in expression`. The lines after it that
/// carry no timestamp continue the code it quotes; the indented `Error position:` line and the
/// indented `Error <message>` line, and a `File <file>..., line <n>` line, belong to it.
pub fn scan_errors(text: &str) -> Vec<RptError> {
    let mut out: Vec<RptError> = Vec::new();
    // Whether the last line with a timestamp was part of the error at the end of `out`.
    let mut open = false;
    for line in text.lines() {
        let Some((sod, rest)) = timestamp(line) else {
            // A line without a timestamp continues the line before it.
            continue;
        };
        let trimmed = rest.trim_start();
        let indented = rest.len() != trimmed.len();
        if open && indented && trimmed.starts_with("Error position:") {
            continue;
        }
        if open
            && indented
            && let Some(message) = trimmed.strip_prefix("Error ")
        {
            out.last_mut().unwrap().text = message.trim().to_string();
            continue;
        }
        if open && let Some(file) = trimmed.strip_prefix("File ") {
            let e = out.last_mut().unwrap();
            let (name, line) = file.rsplit_once(", line ").unwrap_or((file, ""));
            e.file = name.trim_end_matches("...").trim().to_string();
            e.line = line.trim().parse().ok();
            open = false;
            continue;
        }
        if (!indented && rest.starts_with("Error ")) || rest.contains("Error in expression") {
            out.push(RptError {
                text: rest.trim().to_string(),
                file: String::new(),
                line: None,
                local_s: Some(sod),
            });
            open = true;
            continue;
        }
        open = false;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "76561190000000042";

    #[test]
    fn log_02_steam_ids_are_replaced_and_other_numbers_kept() {
        let r = Redactor::default();
        assert_eq!(r.redact(&format!("id={ID})")), "id=<steamid>)");
        assert_eq!(r.redact(&format!("uid {ID} name")), "uid <steamid> name");
        // 18 digits, or 17 that do not start 7656119, are no Steam ID.
        assert_eq!(r.redact(&format!("{ID}1")), format!("{ID}1"));
        assert_eq!(r.redact("12345678901234567"), "12345678901234567");
    }

    #[test]
    fn log_02_names_are_replaced_as_whole_words_case_sensitively() {
        let r = Redactor::new(vec!["Zed".into(), "Zed Alpha".into()]);
        assert_eq!(r.redact("Player Zed connected"), "Player <steamname> connected");
        assert_eq!(r.redact("name Zed Alpha here"), "name <steamname> here");
        assert_eq!(r.redact("Zedd zed _Zed Zed."), "Zedd zed _Zed <steamname>.");
        assert_eq!(r.redact("Zed"), "<steamname>");
    }

    #[test]
    fn us_065_values_in_a_result_are_redacted() {
        let r = Redactor::new(vec!["Zed".into()]);
        let mut v =
            serde_json::json!({"message": format!("uid {ID} name Zed"), "attached": [{"value": ID}, {"value": "Zed"}]});
        r.redact_value(&mut v);
        assert_eq!(v["message"], "uid <steamid> name <steamname>");
        assert_eq!(v["attached"][0]["value"], "<steamid>");
        assert_eq!(v["attached"][1]["value"], "<steamname>");
    }

    #[test]
    fn int_stm_04_the_most_recent_account_gives_the_names() {
        let vdf = "\"users\"\n{\n\t\"1\"\n\t{\n\t\t\"AccountName\"\t\t\"old\"\n\t\t\"PersonaName\"\t\t\"Old\"\n\t\t\"MostRecent\"\t\t\"0\"\n\t}\n\t\"2\"\n\t{\n\t\t\"AccountName\"\t\t\"acct\"\n\t\t\"PersonaName\"\t\t\"Persona Name\"\n\t\t\"MostRecent\"\t\t\"1\"\n\t}\n}\n";
        assert_eq!(
            most_recent_names(vdf),
            vec!["acct".to_string(), "Persona Name".to_string()]
        );
        let vdf = "\"users\"
{
	\"1\"
	{
		\"AccountName\"		\"old\"
		\"PersonaName\"		\"Old\"
		\"timestamp\"		\"100\"
	}
	\"2\"
	{
		\"AccountName\"		\"acct\"
		\"PersonaName\"		\"P\"
		\"timestamp\"		\"200\"
	}
}
";
        assert_eq!(most_recent_names(vdf), vec!["acct".to_string(), "P".to_string()]);
    }

    #[test]
    fn log_04_a_check_is_cut_between_its_markers() {
        let text = " 7:00:00 a\n 7:00:01 [shakedown] r-1#2 start 2026\n 7:00:02 b\n 7:00:03 [shakedown] r-1#2 end 2026\n 7:00:04 c\n";
        assert_eq!(
            cut(text, "r-1#2").unwrap(),
            " 7:00:01 [shakedown] r-1#2 start 2026\n 7:00:02 b\n 7:00:03 [shakedown] r-1#2 end 2026\n"
        );
        // The start of r-1#20 is not that of r-1#2.
        assert!(cut(" 7:00:01 [shakedown] r-1#20 start x\n", "r-1#2").is_none());
    }

    #[test]
    fn err_02_an_rpt_time_is_turned_into_utc_from_a_marker() {
        let m = " 9:00:01 [shakedown] r-1#1 start 2026-11-06T08:00:01.250Z";
        assert_eq!(utc_of(m, 9 * 3600 + 3).as_deref(), Some("2026-11-06T08:00:03.000Z"));
        let m = "23:30:00 [shakedown] r-1 start 2026-11-06T18:00:00.100Z";
        assert_eq!(
            utc_of(m, 23 * 3600 + 31 * 60).as_deref(),
            Some("2026-11-06T18:01:00.000Z")
        );
    }

    #[test]
    fn int_ar_07_an_rpt_error_with_its_position_message_and_file() {
        let text = " 7:06:28 Error in expression <private _skdfx_sum = _skdfx_undefined + 1;\nnil>\n 7:06:28   Error position: <_skdfx_undefined + 1;\nnil>\n 7:06:28   Error Undefined variable in expression: _skdfx_undefined\n 7:06:28 File skdfx\\addons\\main\\functions\\fn_throw.sqf..., line 4\n 7:06:29 something else\n17:06:30 Error in expression <[] spawn { _x + 1 }>\n17:06:30   Error position: <_x + 1 }>\n17:06:30   Error Undefined variable in expression: _x\n17:06:31 \"Error in expression <made by a check>\"\n";
        let errors = scan_errors(text);
        assert_eq!(errors.len(), 3, "{errors:#?}");
        assert_eq!(errors[0].text, "Undefined variable in expression: _skdfx_undefined");
        assert_eq!(errors[0].file, "skdfx\\addons\\main\\functions\\fn_throw.sqf");
        assert_eq!(errors[0].line, Some(4));
        assert_eq!(errors[0].local_s, Some(7 * 3600 + 6 * 60 + 28));
        assert_eq!(errors[1].text, "Undefined variable in expression: _x");
        assert_eq!(errors[1].file, "");
        assert_eq!(errors[2].text, "\"Error in expression <made by a check>\"");
    }
}
