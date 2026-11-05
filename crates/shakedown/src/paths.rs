//! Where the runner keeps its own files (`INT-WIN-03`), and its tokens (`SEC-06`).

use std::path::PathBuf;

/// `%LOCALAPPDATA%\Shakedown`, or `~/.local/share/shakedown` where there is no such folder.
pub fn runner_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(dir).join("Shakedown");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join(".local").join("share").join("shakedown")
}

pub fn runner_file() -> PathBuf {
    runner_dir().join("runner.json")
}

pub fn runner_log() -> PathBuf {
    runner_dir().join("runner.log")
}

/// A token: the prefix and 64 lowercase hexadecimal digits of a random 256-bit value (`SEC-06`).
pub fn token(prefix: &str) -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the system's random source works");
    let mut out = String::with_capacity(prefix.len() + 64);
    out.push_str(prefix);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// The folder `.shakedown/` in a project, created with a `.gitignore` that ignores everything in
/// it (`CFG-02`).
pub fn project_state_dir(project: &std::path::Path) -> std::io::Result<PathBuf> {
    let dir = project.join(".shakedown");
    std::fs::create_dir_all(&dir)?;
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        std::fs::write(&ignore, "*\n")?;
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sec_06_tokens_have_the_fixed_form() {
        for prefix in ["skd_r_", "skd_s_"] {
            let t = token(prefix);
            assert!(t.starts_with(prefix));
            let hex = &t[prefix.len()..];
            assert_eq!(hex.len(), 64);
            assert!(hex.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
        }
        assert_ne!(token("skd_s_"), token("skd_s_"));
    }

    #[test]
    fn cfg_02_the_state_folder_ignores_everything() {
        let dir = std::env::temp_dir().join(format!("skd-cfg02-{}", std::process::id()));
        let state = project_state_dir(&dir).unwrap();
        assert_eq!(std::fs::read_to_string(state.join(".gitignore")).unwrap(), "*\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
