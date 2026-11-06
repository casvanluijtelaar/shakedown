//! The Windows parts of the runner: the job object that owns every game process (`STP-01`,
//! `INT-WIN-01`), starting a process suspended with its own environment, and finding Steam.
//! On other systems these report that sessions need Windows, so the rest builds and tests there
//! (`NFR-PT-01`).

use std::path::Path;
use std::time::Duration;

/// A started game process, owned through the session's job object.
pub struct Child {
    pub pid: u32,
    /// The process's start time, which names it together with its id.
    pub started_at: String,
    #[cfg(windows)]
    handle: imp::Handle,
}

#[cfg(windows)]
pub use imp::*;

#[cfg(windows)]
mod imp {
    use super::Child;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::time::Duration;

    use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };
    use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows::Win32::System::Threading::{
        CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW, GetCurrentProcessId, GetExitCodeProcess,
        GetProcessTimes, PROCESS_INFORMATION, ResumeThread, STARTUPINFOW, TerminateProcess, WaitForSingleObject,
    };
    use windows::core::{PCWSTR, PWSTR};

    /// A kernel handle closed on drop.
    pub(super) struct Handle(pub(super) HANDLE);
    // A kernel handle may be used and closed from any thread.
    unsafe impl Send for Handle {}
    unsafe impl Sync for Handle {}
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    /// A job object with kill-on-close: every process assigned to it ends when the last handle to
    /// the job closes, which happens when the runner ends, however it ends.
    pub struct Job(Handle);

    impl Job {
        pub fn new() -> std::io::Result<Job> {
            unsafe {
                let job = CreateJobObjectW(None, PCWSTR::null()).map_err(std::io::Error::other)?;
                let job = Handle(job);
                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.BasicLimitInformation.LimitFlags =
                    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
                SetInformationJobObject(
                    job.0,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
                .map_err(std::io::Error::other)?;
                Ok(Job(job))
            }
        }

        /// Starts a process suspended, assigns it to the job, then resumes it (`STP-01`). The
        /// process gets the runner's environment plus `env`.
        pub fn spawn(
            &self,
            exe: &Path,
            args: &[String],
            cwd: &Path,
            env: &[(String, String)],
        ) -> std::io::Result<Child> {
            let mut command_line: Vec<u16> = std::iter::once(quote(&exe.to_string_lossy()))
                .chain(args.iter().map(|a| quote(a)))
                .collect::<Vec<_>>()
                .join(" ")
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let block = environment_block(env);
            let exe_w: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let cwd_w: Vec<u16> = cwd.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let startup = STARTUPINFOW {
                cb: std::mem::size_of::<STARTUPINFOW>() as u32,
                ..Default::default()
            };
            let mut info = PROCESS_INFORMATION::default();
            unsafe {
                CreateProcessW(
                    PCWSTR(exe_w.as_ptr()),
                    Some(PWSTR(command_line.as_mut_ptr())),
                    None,
                    None,
                    false,
                    CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
                    Some(block.as_ptr() as *const _),
                    PCWSTR(cwd_w.as_ptr()),
                    &startup,
                    &mut info,
                )
                .map_err(std::io::Error::other)?;
                let process = Handle(info.hProcess);
                let thread = Handle(info.hThread);
                if let Err(e) = AssignProcessToJobObject(self.0.0, process.0) {
                    let _ = TerminateProcess(process.0, 1);
                    return Err(std::io::Error::other(e));
                }
                ResumeThread(thread.0);
                let started_at = process_start(process.0).unwrap_or_else(shakedown_protocol::time::now);
                Ok(Child {
                    pid: info.dwProcessId,
                    started_at,
                    handle: process,
                })
            }
        }
    }

    /// Starts a process detached: no console, its own process group, out of the caller's job
    /// when the job allows it (`INT-WIN-02`), and inheriting no handle, so it holds none of the
    /// caller's pipes open.
    pub fn spawn_detached(exe: &Path, args: &[String]) -> std::io::Result<u32> {
        use windows::Win32::System::Threading::{
            CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, PROCESS_CREATION_FLAGS,
        };
        let start = |flags: PROCESS_CREATION_FLAGS| -> std::io::Result<u32> {
            let mut command_line: Vec<u16> = std::iter::once(quote(&exe.to_string_lossy()))
                .chain(args.iter().map(|a| quote(a)))
                .collect::<Vec<_>>()
                .join(" ")
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let exe_w: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let startup = STARTUPINFOW {
                cb: std::mem::size_of::<STARTUPINFOW>() as u32,
                ..Default::default()
            };
            let mut info = PROCESS_INFORMATION::default();
            unsafe {
                CreateProcessW(
                    PCWSTR(exe_w.as_ptr()),
                    Some(PWSTR(command_line.as_mut_ptr())),
                    None,
                    None,
                    false,
                    flags,
                    None,
                    PCWSTR::null(),
                    &startup,
                    &mut info,
                )
                .map_err(std::io::Error::other)?;
                let _process = Handle(info.hProcess);
                let _thread = Handle(info.hThread);
            }
            Ok(info.dwProcessId)
        };
        let flags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
        // A job that forbids breakaway refuses the first form; the process then starts inside it.
        start(flags | CREATE_BREAKAWAY_FROM_JOB).or_else(|_| start(flags))
    }

    impl Child {
        /// The exit code once the process has ended, waiting up to `wait`.
        pub fn wait(&self, wait: Duration) -> Option<u32> {
            unsafe {
                if WaitForSingleObject(self.handle.0, wait.as_millis() as u32) != WAIT_OBJECT_0 {
                    return None;
                }
                let mut code = 0u32;
                GetExitCodeProcess(self.handle.0, &mut code).ok()?;
                Some(code)
            }
        }

        pub fn terminate(&self) {
            unsafe {
                let _ = TerminateProcess(self.handle.0, 1);
            }
        }
    }

    fn process_start(process: HANDLE) -> Option<String> {
        let (mut created, mut exited, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user).ok()? };
        let ticks = ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64;
        // FILETIME counts 100 ns from 1601-01-01; the Unix epoch is 11644473600 s later.
        let unix_ms = (ticks / 10_000) as i64 - 11_644_473_600_000;
        Some(shakedown_protocol::time::format_unix_millis(unix_ms))
    }

    /// The runner's own environment plus `extra`, sorted by name as Windows expects.
    fn environment_block(extra: &[(String, String)]) -> Vec<u16> {
        let mut vars: Vec<(String, String)> = std::env::vars()
            .filter(|(k, _)| !extra.iter().any(|(e, _)| e.eq_ignore_ascii_case(k)))
            .collect();
        vars.extend(extra.iter().cloned());
        vars.sort_by_key(|(k, _)| k.to_uppercase());
        let mut block: Vec<u16> = Vec::new();
        for (k, v) in vars {
            block.extend(format!("{k}={v}").encode_utf16());
            block.push(0);
        }
        block.push(0);
        block
    }

    /// Quotes one command-line argument when it holds a space. The game's options hold no quotes.
    fn quote(arg: &str) -> String {
        if arg.contains(' ') || arg.contains('\t') {
            format!("\"{arg}\"")
        } else {
            arg.to_string()
        }
    }

    /// Whether a process `steam.exe` runs in the current user's Windows session (`SES-03`).
    pub fn steam_running() -> bool {
        unsafe {
            let mut own_session = 0u32;
            if ProcessIdToSessionId(GetCurrentProcessId(), &mut own_session).is_err() {
                return false;
            }
            let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
                return false;
            };
            let snapshot = Handle(snapshot);
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut ok = Process32FirstW(snapshot.0, &mut entry).is_ok();
            while ok {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                if name.eq_ignore_ascii_case("steam.exe") {
                    let mut session = 0u32;
                    if ProcessIdToSessionId(entry.th32ProcessID, &mut session).is_ok() && session == own_session {
                        return true;
                    }
                }
                ok = Process32NextW(snapshot.0, &mut entry).is_ok();
            }
            false
        }
    }

    /// Sets the title of every visible top-level window of a process, so a person can tell the
    /// game clients apart (`INT-AR-02`). Touches no input, focus or foreground. Returns whether a
    /// window was found.
    pub fn set_window_title(pid: u32, title: &str) -> bool {
        use windows::Win32::Foundation::{HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetWindowTextW,
        };
        struct Search {
            pid: u32,
            found: Vec<HWND>,
        }
        unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
            let search = unsafe { &mut *(lparam.0 as *mut Search) };
            let mut owner = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, Some(&mut owner)) };
            if owner == search.pid && unsafe { IsWindowVisible(hwnd) }.as_bool() {
                search.found.push(hwnd);
            }
            true.into()
        }
        let mut search = Search { pid, found: vec![] };
        unsafe {
            let _ = EnumWindows(Some(each), LPARAM(&mut search as *mut Search as isize));
        }
        let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        for hwnd in &search.found {
            unsafe {
                let _ = SetWindowTextW(*hwnd, PCWSTR(wide.as_ptr()));
            }
        }
        !search.found.is_empty()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn stp_01_a_process_in_the_job_ends_when_the_job_closes() {
            let job = Job::new().unwrap();
            let cmd = std::path::PathBuf::from(
                std::env::var("ComSpec").unwrap_or_else(|_| r"C:\Windows\System32\cmd.exe".into()),
            );
            let child = job
                .spawn(
                    &cmd,
                    &["/c".into(), "ping -n 30 127.0.0.1 >nul".into()],
                    Path::new("."),
                    &[("SHAKEDOWN_TOKEN".into(), "x".into())],
                )
                .unwrap();
            assert!(child.wait(Duration::from_millis(200)).is_none(), "still running");
            drop(job);
            assert!(child.wait(Duration::from_secs(10)).is_some(), "ended with its job");
        }

        #[test]
        fn sec_02_the_environment_reaches_the_process() {
            let job = Job::new().unwrap();
            let cmd = std::path::PathBuf::from(
                std::env::var("ComSpec").unwrap_or_else(|_| r"C:\Windows\System32\cmd.exe".into()),
            );
            let child = job
                .spawn(
                    &cmd,
                    &[
                        "/c".into(),
                        "if \"%SHAKEDOWN_TOKEN%\"==\"skd_s_1\" (exit 7) else (exit 3)".into(),
                    ],
                    Path::new("."),
                    &[("SHAKEDOWN_TOKEN".into(), "skd_s_1".into())],
                )
                .unwrap();
            assert_eq!(child.wait(Duration::from_secs(10)), Some(7));
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Child;
    use std::path::Path;
    use std::time::Duration;

    pub struct Job;

    impl Job {
        pub fn new() -> std::io::Result<Job> {
            Err(std::io::Error::other("sessions need Windows"))
        }
        pub fn spawn(
            &self,
            _exe: &Path,
            _args: &[String],
            _cwd: &Path,
            _env: &[(String, String)],
        ) -> std::io::Result<Child> {
            Err(std::io::Error::other("sessions need Windows"))
        }
    }

    pub fn spawn_detached(_exe: &Path, _args: &[String]) -> std::io::Result<u32> {
        Err(std::io::Error::other("needs Windows"))
    }

    impl Child {
        pub fn wait(&self, _wait: Duration) -> Option<u32> {
            Some(0)
        }
        pub fn terminate(&self) {}
    }

    pub fn steam_running() -> bool {
        false
    }

    pub fn set_window_title(_pid: u32, _title: &str) -> bool {
        false
    }
}

#[cfg(not(windows))]
pub use imp::*;

/// Whether `child` has ended, without waiting.
pub fn exited(child: &Child) -> Option<u32> {
    child.wait(Duration::ZERO)
}

/// The game executable of a machine kind in the game folder.
pub fn game_exe(game: &Path) -> std::path::PathBuf {
    game.join("arma3_x64.exe")
}

/// The dedicated server and headless client executable in the game folder.
pub fn server_exe(game: &Path) -> std::path::PathBuf {
    game.join("arma3server_x64.exe")
}
