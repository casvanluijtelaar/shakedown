//! The release of a reload of level `mission-restart` (`RLD-07`, `P-08` step 2): the release
//! files every machine reads so the game lets go of the loose files it holds, the check that no
//! game process holds a changed file, and, for a file still held, the processes that hold it.

use std::path::{Path, PathBuf};

/// The folder of the game folder that holds the release files; no PBO covers it, so file
/// patching finds the files (`RLD-07`).
pub const FOLDER: &str = "skd_release";
/// The size of the one release file.
pub const SIZE: u64 = 8 * 1024 * 1024;
/// The number of junctions to the release file's folder: 640 MiB through as many paths, since the
/// game keeps loose files by path (research 31).
pub const LINKS: usize = 80;
/// The release file's name in its folder.
const FILE: &str = "release.txt";

/// The paths the machines read, relative to the game folder, as `loadFile` takes them.
pub fn paths() -> Vec<String> {
    (0..LINKS).map(|i| format!(r"{FOLDER}\l{i:02}\{FILE}")).collect()
}

/// Makes the release files in `game` where any is missing: the file of `SIZE` bytes of text in
/// `skd_release\data` and the junctions `skd_release\l00` to `l79` to that folder. Files already
/// there are kept, so only the first reload that needs them makes them.
pub fn ensure(game: &Path) -> Result<(), String> {
    let root = game.join(FOLDER);
    let data = root.join("data");
    let file = data.join(FILE);
    if std::fs::metadata(&file).map(|m| m.len()).ok() != Some(SIZE) {
        std::fs::create_dir_all(&data).map_err(|e| format!("cannot make {}: {e}", data.display()))?;
        // Text, so `loadFile` reads the whole file as one string.
        let line = b"// The release file of shakedown reload (RLD-07); it is read and never run.\n";
        let mut bytes = Vec::with_capacity(SIZE as usize);
        while bytes.len() < SIZE as usize {
            bytes.extend_from_slice(line);
        }
        bytes.truncate(SIZE as usize);
        std::fs::write(&file, bytes).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    }
    for i in 0..LINKS {
        let link = root.join(format!("l{i:02}"));
        if link.join(FILE).is_file() {
            continue;
        }
        super::session::junction(&link, &data)?;
    }
    Ok(())
}

/// The file the game reads for a changed file of a watched `source`: the same path below the
/// mod's `prefix` in the game folder (`P-08` step 2).
pub fn game_file(game: &Path, source: &Path, prefix: &str, file: &Path) -> Option<PathBuf> {
    // Compared as Windows compares paths, without case; the part below is used as written in lower case.
    let (f, s) = (super::reload::lower(file), super::reload::lower(source));
    let rel = f.strip_prefix(&format!("{s}\\"))?;
    Some(game.join(prefix).join(rel))
}

/// Whether a game process holds `file` so it cannot be written: opening it for writing fails
/// with a sharing violation. A file that is not there is free.
pub fn is_held(file: &Path) -> bool {
    match std::fs::OpenOptions::new().write(true).open(file) {
        Ok(_) => false,
        // ERROR_SHARING_VIOLATION and ERROR_LOCK_VIOLATION.
        Err(e) => matches!(e.raw_os_error(), Some(32) | Some(33)),
    }
}

/// The ids of the processes that hold `file`, through the Windows Restart Manager (`P-08` step 2,
/// `US-082`); empty where it cannot tell.
#[cfg(windows)]
pub fn holders(file: &Path) -> Vec<u32> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
    use windows::Win32::System::RestartManager::{
        CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList, RmRegisterResources, RmStartSession,
    };
    use windows::core::{PCWSTR, PWSTR};

    let wide: Vec<u16> = file.as_os_str().encode_wide().chain([0]).collect();
    let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
    let mut session = 0u32;
    // SAFETY: plain Restart Manager calls on buffers that outlive them; the session is ended.
    unsafe {
        if RmStartSession(&mut session, None, PWSTR(key.as_mut_ptr())) != ERROR_SUCCESS {
            return vec![];
        }
        let mut out = vec![];
        let names = [PCWSTR(wide.as_ptr())];
        if RmRegisterResources(session, Some(&names), None, None) == ERROR_SUCCESS {
            let mut infos = vec![RM_PROCESS_INFO::default(); 16];
            for _ in 0..3 {
                let (mut needed, mut count, mut reasons) = (0u32, infos.len() as u32, 0u32);
                let r = RmGetList(session, &mut needed, &mut count, Some(infos.as_mut_ptr()), &mut reasons);
                if r == ERROR_SUCCESS {
                    out = infos[..count as usize].iter().map(|i| i.Process.dwProcessId).collect();
                    break;
                }
                if r != ERROR_MORE_DATA {
                    break;
                }
                infos = vec![RM_PROCESS_INFO::default(); needed as usize + 4];
            }
        }
        let _ = RmEndSession(session);
        out
    }
}

#[cfg(not(windows))]
pub fn holders(_file: &Path) -> Vec<u32> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rld_07_eighty_paths_under_the_release_folder() {
        let p = paths();
        assert_eq!(p.len(), 80);
        assert_eq!(p[0], r"skd_release\l00\release.txt");
        assert_eq!(p[79], r"skd_release\l79\release.txt");
    }

    /// Windows only: the game file is a path in the Windows game folder, compared and built as
    /// Windows paths, and a reload runs only in a session, which starts only on Windows.
    #[cfg(windows)]
    #[test]
    fn p_08_the_game_file_of_a_changed_file_is_below_the_prefix() {
        let f = game_file(
            Path::new(r"C:\game"),
            Path::new(r"C:\p\stage\skdfx_cba\addons"),
            r"skdfx_cba\addons",
            Path::new(r"C:\p\stage\skdfx_cba\addons\main\functions\fnc_version.sqf"),
        );
        assert_eq!(
            f,
            Some(PathBuf::from(
                r"C:\game\skdfx_cba\addons\main\functions\fnc_version.sqf"
            ))
        );
        let f = game_file(
            Path::new(r"C:\game"),
            Path::new(r"C:\p\Stage\addons"),
            r"z\agh\addons",
            Path::new(r"C:\p\stage\addons\climb\fn_x.sqf"),
        );
        assert_eq!(f, Some(PathBuf::from(r"C:\game\z\agh\addons\climb\fn_x.sqf")));
    }

    /// `RLD-07`: the release files are made once, 8 MiB through 80 junctions, and kept.
    #[cfg(windows)]
    #[test]
    fn rld_07_the_release_files_are_made_once_and_kept() {
        let game = std::env::temp_dir().join(format!("skd-release-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&game);
        std::fs::create_dir_all(&game).unwrap();
        ensure(&game).unwrap();
        for p in paths() {
            assert_eq!(std::fs::metadata(game.join(&p)).unwrap().len(), SIZE, "{p}");
        }
        let made = std::fs::metadata(game.join(FOLDER).join("data").join(FILE))
            .unwrap()
            .modified()
            .unwrap();
        ensure(&game).unwrap();
        let again = std::fs::metadata(game.join(FOLDER).join("data").join(FILE))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(made, again);
        for i in 0..LINKS {
            let _ = std::fs::remove_dir(game.join(FOLDER).join(format!("l{i:02}")));
        }
        let _ = std::fs::remove_dir_all(&game);
    }

    /// `P-08` step 2 and `US-082`: a file opened for reading only, shared for reading only, as
    /// the game holds a loose file, is held, and the Restart Manager names this process as its
    /// holder; once closed it is free.
    #[cfg(windows)]
    #[test]
    fn us_082_a_file_held_for_reading_only_is_held_and_its_holder_named() {
        use std::os::windows::fs::OpenOptionsExt;
        let file = std::env::temp_dir().join(format!("skd-held-{}.sqf", std::process::id()));
        std::fs::write(&file, "\"1\"").unwrap();
        assert!(!is_held(&file));
        let hold = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1) // FILE_SHARE_READ
            .open(&file)
            .unwrap();
        assert!(is_held(&file));
        assert!(holders(&file).contains(&std::process::id()), "{:?}", holders(&file));
        drop(hold);
        assert!(!is_held(&file));
        assert!(!is_held(&file.with_extension("missing")));
        let _ = std::fs::remove_file(&file);
    }
}
