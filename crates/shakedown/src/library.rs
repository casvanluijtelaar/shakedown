//! The library folder of shared helpers (`API-06`): the runner reads its `.sqf` files when a
//! session starts and at each reload, and every machine's agent compiles them into the global
//! variables `skd_lib_<file name>`.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// One library file as the runner read it.
#[derive(Debug, Clone, PartialEq)]
pub struct LibFile {
    /// The file name without `.sqf`; its variable is `skd_lib_<name>`.
    pub name: String,
    /// The path relative to the project folder, with `/`, as messages name it.
    pub shown: String,
    pub path: PathBuf,
    /// The text as read.
    pub text: String,
}

/// The library as the runner read it last.
#[derive(Debug, Clone, Default)]
pub struct Library {
    pub dir: PathBuf,
    pub files: Vec<LibFile>,
    /// What keeps the library from loading, which ends every check `fail` / `check-error`: a file
    /// that cannot be read or whose name is not letters, digits and `_`, or the first compile
    /// error a machine reported.
    pub problem: Option<String>,
}

/// Whether two paths name one folder, by their text with `.`, `..` and case resolved.
pub fn same(a: &Path, b: &Path) -> bool {
    crate::runner::reload::lower(a) == crate::runner::reload::lower(b)
}

/// Whether `path` is under the folder `dir`.
pub fn below(path: &Path, dir: &Path) -> bool {
    let (p, d) = (crate::runner::reload::lower(path), crate::runner::reload::lower(dir));
    !d.is_empty() && p.starts_with(&format!("{d}\\"))
}

/// A path relative to the project folder with `/`, or the whole path when it is not under it.
fn shown(path: &Path, project: &Path) -> String {
    let rel = path
        .strip_prefix(project)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf());
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn name_ok(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Reads every `.sqf` file directly in `dir`, in name order. A folder that does not exist is an
/// empty library.
pub fn read(dir: &Path, project: &Path) -> Library {
    let mut library = Library {
        dir: dir.to_path_buf(),
        ..Library::default()
    };
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("sqf")))
        .collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let file_shown = shown(&path, project);
        if !name_ok(&name) && library.problem.is_none() {
            library.problem = Some(format!(
                "The library file {file_shown} has a name that is not letters, digits and `_`, so it can have no variable skd_lib_<name>. Rename it and run `shakedown reload`; until then every check fails."
            ));
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => library.files.push(LibFile {
                name,
                shown: file_shown,
                path,
                text,
            }),
            Err(e) => {
                if library.problem.is_none() {
                    library.problem = Some(format!(
                        "The library file {file_shown} cannot be read: {e}. Fix it and run `shakedown reload`; until then every check fails."
                    ));
                }
            }
        }
    }
    library
}

impl Library {
    /// The arguments of the agent's `skd_fnc_library`: each file's name, its path as shown and
    /// its text with every comment turned into spaces, line breaks kept, since `compile` takes no
    /// comments (`API-02`).
    pub fn order_args(&self) -> Value {
        let files: Vec<Value> = self
            .files
            .iter()
            .filter(|f| name_ok(&f.name))
            .map(|f| json!([f.name, f.shown, crate::checks::body_source(&f.text)]))
            .collect();
        json!([files])
    }

    /// The message of a compile error a machine reported for a library file.
    pub fn compile_problem(file: &str, line: &Value, text: &str) -> String {
        format!(
            "The library file {file} does not compile: {text}, at line {line}. Fix it and run `shakedown reload`; until then every check fails."
        )
    }
}

/// The library files that differ between two reads: changed, new and deleted (`P-08` step 1).
pub fn changed(before: &Library, now: &Library) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = now
        .files
        .iter()
        .filter(|f| !before.files.iter().any(|b| same(&b.path, &f.path) && b.text == f.text))
        .map(|f| f.path.clone())
        .collect();
    out.extend(
        before
            .files
            .iter()
            .filter(|b| !now.files.iter().any(|f| same(&b.path, &f.path)))
            .map(|b| b.path.clone()),
    );
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("skd-lib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("tests").join("_lib").join("sub")).unwrap();
        dir
    }

    #[test]
    fn us_070_api_06_reads_the_sqf_files_directly_in_the_folder() {
        let dir = folder("read");
        let lib = dir.join("tests").join("_lib");
        std::fs::write(lib.join("rope.sqf"), "// kit\n1 + 1").unwrap();
        std::fs::write(lib.join("b_2.sqf"), "2").unwrap();
        std::fs::write(lib.join("notes.txt"), "x").unwrap();
        std::fs::write(lib.join("sub").join("deep.sqf"), "3").unwrap();
        let l = read(&lib, &dir);
        assert!(l.problem.is_none(), "{:?}", l.problem);
        let names: Vec<&str> = l.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["b_2", "rope"]);
        assert_eq!(l.files[1].shown, "tests/_lib/rope.sqf");
        // Comments go, line breaks stay, so a compile error names the file's line.
        assert_eq!(
            l.order_args(),
            json!([[
                ["b_2", "tests/_lib/b_2.sqf", "2"],
                ["rope", "tests/_lib/rope.sqf", "      \n1 + 1"]
            ]])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn api_06_a_missing_folder_is_an_empty_library() {
        let dir = folder("missing");
        let l = read(&dir.join("helpers"), &dir);
        assert!(l.files.is_empty() && l.problem.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn us_071_api_06_a_file_name_that_cannot_be_a_variable_is_a_problem_naming_the_file() {
        let dir = folder("name");
        let lib = dir.join("tests").join("_lib");
        std::fs::write(lib.join("rope-kit.sqf"), "1").unwrap();
        let l = read(&lib, &dir);
        let p = l.problem.clone().expect("a problem");
        assert!(p.contains("tests/_lib/rope-kit.sqf"), "{p}");
        assert_eq!(l.order_args(), json!([[]]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn us_070_rld_01_changed_library_files_are_found_by_their_text() {
        let dir = folder("changed");
        let lib = dir.join("tests").join("_lib");
        std::fs::write(lib.join("a.sqf"), "1").unwrap();
        std::fs::write(lib.join("b.sqf"), "2").unwrap();
        let before = read(&lib, &dir);
        assert!(changed(&before, &read(&lib, &dir)).is_empty());
        std::fs::write(lib.join("a.sqf"), "10").unwrap();
        std::fs::remove_file(lib.join("b.sqf")).unwrap();
        std::fs::write(lib.join("c.sqf"), "3").unwrap();
        let got = changed(&before, &read(&lib, &dir));
        assert_eq!(got, [lib.join("a.sqf"), lib.join("b.sqf"), lib.join("c.sqf")]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `same` also decides which folder `list` skips on macOS and Linux (`NFR-PT-02`), so the
    /// comparison is proven on the platform's own separator.
    #[test]
    fn api_06_below_and_same_compare_paths_by_their_text() {
        let p = std::env::temp_dir().join("p");
        let up = std::env::temp_dir().join("P");
        assert!(below(
            &p.join("tests").join("_lib").join("a.sqf"),
            &up.join("tests").join(".").join("_lib")
        ));
        assert!(!below(
            &p.join("tests").join("_libx").join("a.sqf"),
            &p.join("tests").join("_lib")
        ));
        assert!(same(
            &p.join("tests").join("_lib"),
            &p.join("x").join("..").join("tests").join("_lib")
        ));
    }

    /// Windows paths, with a drive letter in either case, as a session on Windows sees them.
    #[cfg(windows)]
    #[test]
    fn api_06_below_and_same_compare_windows_paths_by_their_text() {
        assert!(below(
            Path::new(r"C:\p\tests\_lib\a.sqf"),
            Path::new(r"c:\P\tests\.\_lib")
        ));
        assert!(!below(
            Path::new(r"C:\p\tests\_libx\a.sqf"),
            Path::new(r"C:\p\tests\_lib")
        ));
        assert!(same(Path::new(r"C:\p\tests\_lib"), Path::new(r"C:\p\x\..\tests\_lib")));
    }
}
