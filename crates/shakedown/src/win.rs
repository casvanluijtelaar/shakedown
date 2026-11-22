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

    /// The variable that tells `shakedown runner` it is the relay of [`start_runner`].
    pub const RELAY_VAR: &str = "SHAKEDOWN_RUNNER_RELAY";

    /// Starts a process with no console, in its own process group, out of the caller's job
    /// (`CREATE_BREAKAWAY_FROM_JOB`), inheriting no handle, so it holds none of the caller's pipes
    /// open. `env` is added to the caller's environment. With `breakaway` it leaves the caller's
    /// job, and fails when that job forbids it.
    pub fn spawn_breakaway(
        exe: &Path,
        args: &[String],
        env: &[(String, String)],
        breakaway: bool,
    ) -> std::io::Result<u32> {
        use windows::Win32::System::Threading::{
            CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS,
        };
        let mut command_line: Vec<u16> = std::iter::once(quote(&exe.to_string_lossy()))
            .chain(args.iter().map(|a| quote(a)))
            .collect::<Vec<_>>()
            .join(" ")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let exe_w: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let block = environment_block(env);
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
                DETACHED_PROCESS
                    | CREATE_NEW_PROCESS_GROUP
                    | CREATE_UNICODE_ENVIRONMENT
                    | if breakaway {
                        CREATE_BREAKAWAY_FROM_JOB
                    } else {
                        Default::default()
                    },
                Some(block.as_ptr() as *const _),
                PCWSTR::null(),
                &startup,
                &mut info,
            )
            .map_err(std::io::Error::other)?;
            let _process = Handle(info.hProcess);
            let _thread = Handle(info.hThread);
        }
        Ok(info.dwProcessId)
    }

    /// Starts `shakedown runner` outside the caller's job object and process tree (`SES-08`,
    /// `INT-WIN-02`). It breaks away from the job into a relay, `shakedown runner` with
    /// [`RELAY_VAR`] set, which starts the runner itself and exits at once, so the runner's parent
    /// is a process that has ended and no walk of the caller's tree reaches it. When the job
    /// forbids breakaway, [`start_by_wmi`] starts the runner, outside both. The relay does the
    /// same when a job around the caller's job still holds it. Returns the way it went:
    /// `breakaway` or `wmi`.
    pub fn start_runner(exe: &Path) -> std::io::Result<&'static str> {
        let args = ["runner".to_string()];
        if spawn_breakaway(exe, &args, &[(RELAY_VAR.into(), "1".into())], true).is_ok() {
            return Ok("breakaway");
        }
        start_by_wmi(exe).map(|_| "wmi")
    }

    /// Starts `shakedown runner` through WMI's `Win32_Process.Create`, as a child of the WMI
    /// service: outside every job and process tree of the caller.
    pub fn start_by_wmi(exe: &Path) -> std::io::Result<()> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let command_line = format!("\"{}\" runner", exe.display()).replace('\'', "''");
        // DETACHED_PROCESS and CREATE_NEW_PROCESS_GROUP, as the breakaway start gives: without
        // them WMI gives the runner a console of its own, hidden, whose control events end it.
        let script = format!(
            "$s = New-CimInstance -ClassName Win32_ProcessStartup -ClientOnly -Property @{{CreateFlags=[uint32]520}}; $r = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{{CommandLine='{command_line}'; ProcessStartupInformation=$s}}; exit $r.ReturnValue"
        );
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "WMI's Win32_Process.Create returned {}",
                status.code().unwrap_or(-1)
            )))
        }
    }

    /// Calls `f` with each console control event the process receives (`CTRL_C_EVENT` 0,
    /// `CTRL_BREAK_EVENT` 1, `CTRL_CLOSE_EVENT` 2, `CTRL_LOGOFF_EVENT` 5, `CTRL_SHUTDOWN_EVENT` 6).
    /// Ctrl+C and Ctrl+Break are then ignored; the others go on to end the process.
    pub fn on_console_event(f: fn(u32)) {
        use std::sync::OnceLock;
        use windows::Win32::System::Console::SetConsoleCtrlHandler;
        static HANDLER: OnceLock<fn(u32)> = OnceLock::new();
        unsafe extern "system" fn each(event: u32) -> windows::core::BOOL {
            if let Some(f) = HANDLER.get() {
                f(event);
            }
            (event <= 1).into()
        }
        let _ = HANDLER.set(f);
        unsafe {
            let _ = SetConsoleCtrlHandler(Some(each), true);
        }
    }

    /// Like [`on_console_event`], and also takes Ctrl+C in a process started in a new process
    /// group, which starts with Ctrl+C ignored.
    pub fn on_stop_keys(f: fn(u32)) {
        use windows::Win32::System::Console::SetConsoleCtrlHandler;
        unsafe {
            let _ = SetConsoleCtrlHandler(None, false);
        }
        on_console_event(f);
    }

    /// The start time of the running process `pid`, in the form of [`Child::started_at`]; `None`
    /// when no such process runs or it cannot be read.
    pub fn process_started(pid: u32) -> Option<String> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        unsafe {
            let process = Handle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
            let mut code = 0u32;
            GetExitCodeProcess(process.0, &mut code).ok()?;
            // 259 is STILL_ACTIVE: an ended process whose handle someone still holds does not run.
            if code != 259 {
                return None;
            }
            process_start(process.0)
        }
    }

    /// Ends the process `pid` if it runs and started at `started_at`; true when it was ended.
    pub fn terminate_pid(pid: u32, started_at: &str) -> bool {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE};
        unsafe {
            let Ok(process) = OpenProcess(PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
                return false;
            };
            let process = Handle(process);
            if process_start(process.0).as_deref() != Some(started_at) {
                return false;
            }
            TerminateProcess(process.0, 1).is_ok()
        }
    }

    /// The process id at the other end of a loopback TCP connection to `local_port` from
    /// `peer_port`, read from the system's TCP table (`SEC-03`).
    pub fn tcp_peer_pid(peer_port: u16, local_port: u16) -> Option<u32> {
        use windows::Win32::NetworkManagement::IpHelper::{
            GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
        };
        const AF_INET: u32 = 2;
        let mut size = 0u32;
        unsafe {
            GetExtendedTcpTable(None, &mut size, false, AF_INET, TCP_TABLE_OWNER_PID_ALL, 0);
        }
        // The table can grow between the two calls; some room covers that.
        let mut buf: Vec<u64> = vec![0; (size as usize + 4096) / 8 + 1];
        let mut size = (buf.len() * 8) as u32;
        let rc = unsafe {
            GetExtendedTcpTable(
                Some(buf.as_mut_ptr() as *mut _),
                &mut size,
                false,
                AF_INET,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };
        if rc != 0 {
            return None;
        }
        let table = buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let rows: &[MIB_TCPROW_OWNER_PID] =
            unsafe { std::slice::from_raw_parts((*table).table.as_ptr(), (*table).dwNumEntries as usize) };
        // A port is in network byte order in the low 16 bits.
        let port = |p: u32| u16::from_be(p as u16);
        rows.iter()
            .find(|r| port(r.dwLocalPort) == peer_port && port(r.dwRemotePort) == local_port)
            .map(|r| r.dwOwningPid)
    }

    /// Reads one of the system's IPv4 socket tables (`GetExtendedTcpTable` or
    /// `GetExtendedUdpTable`) into a buffer of 8-byte words; `None` when the call fails.
    fn socket_table(read: impl Fn(Option<*mut std::ffi::c_void>, &mut u32) -> u32) -> Option<Vec<u64>> {
        let mut size = 0u32;
        read(None, &mut size);
        // The table can grow between the two calls; some room covers that.
        let mut buf: Vec<u64> = vec![0; (size as usize + 4096) / 8 + 1];
        let mut size = (buf.len() * 8) as u32;
        (read(Some(buf.as_mut_ptr() as *mut _), &mut size) == 0).then_some(buf)
    }

    /// Every UDP endpoint on the machine, as (local address, local port, owning process id).
    fn udp_endpoints() -> Vec<(std::net::Ipv4Addr, u16, u32)> {
        use windows::Win32::NetworkManagement::IpHelper::{
            GetExtendedUdpTable, MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID, UDP_TABLE_OWNER_PID,
        };
        const AF_INET: u32 = 2;
        let Some(buf) =
            socket_table(|b, size| unsafe { GetExtendedUdpTable(b, size, false, AF_INET, UDP_TABLE_OWNER_PID, 0) })
        else {
            return vec![];
        };
        let table = buf.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
        let rows: &[MIB_UDPROW_OWNER_PID] =
            unsafe { std::slice::from_raw_parts((*table).table.as_ptr(), (*table).dwNumEntries as usize) };
        rows.iter()
            .map(|r| {
                (
                    std::net::Ipv4Addr::from(u32::from_be(r.dwLocalAddr)),
                    u16::from_be(r.dwLocalPort as u16),
                    r.dwOwningPid,
                )
            })
            .collect()
    }

    /// The ports of `ports` that a UDP socket on the machine is bound to, read from the system's
    /// UDP table. Reading the table opens no socket, so the check itself never listens on an
    /// address other than loopback (`NFR-SC-01`, `SES-03`).
    pub fn udp_ports_held(ports: &[u16]) -> Vec<u16> {
        let bound = udp_endpoints();
        ports
            .iter()
            .copied()
            .filter(|p| bound.iter().any(|(_, port, _)| port == p))
            .collect()
    }

    /// The addresses process `pid` listens on: its TCP sockets in the listening state and its
    /// UDP sockets, read from the system's tables (`NFR-SC-01`).
    pub fn listening_addresses(pid: u32) -> Vec<std::net::SocketAddrV4> {
        use windows::Win32::NetworkManagement::IpHelper::{
            GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
        };
        const AF_INET: u32 = 2;
        let mut out: Vec<std::net::SocketAddrV4> = udp_endpoints()
            .into_iter()
            .filter(|(_, _, owner)| *owner == pid)
            .map(|(ip, port, _)| std::net::SocketAddrV4::new(ip, port))
            .collect();
        if let Some(buf) = socket_table(|b, size| unsafe {
            GetExtendedTcpTable(b, size, false, AF_INET, TCP_TABLE_OWNER_PID_LISTENER, 0)
        }) {
            let table = buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
            let rows: &[MIB_TCPROW_OWNER_PID] =
                unsafe { std::slice::from_raw_parts((*table).table.as_ptr(), (*table).dwNumEntries as usize) };
            out.extend(rows.iter().filter(|r| r.dwOwningPid == pid).map(|r| {
                std::net::SocketAddrV4::new(
                    std::net::Ipv4Addr::from(u32::from_be(r.dwLocalAddr)),
                    u16::from_be(r.dwLocalPort as u16),
                )
            }));
        }
        out
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
    /// Writes `bytes` to `path` as a file only the current user can open: its access list is
    /// protected, so it takes nothing from its folder, and grants full control to the current user
    /// and to no other account or group (`SEC-02`, `INT-WIN-03`). The file is made under a
    /// temporary name and renamed into place, so a reader never sees it part written and it never
    /// exists with another access list.
    pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        use std::os::windows::io::FromRawHandle;
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        };
        use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
        use windows::Win32::Storage::FileSystem::{
            CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_NONE,
        };

        let sddl = format!("D:P(A;;FA;;;{})", current_user_sid()?);
        let sddl: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut sd = PSECURITY_DESCRIPTOR::default();
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(sddl.as_ptr()), SDDL_REVISION_1, &mut sd, None)
        }
        .map_err(std::io::Error::other)?;
        let attrs = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.0,
            bInheritHandle: false.into(),
        };
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = std::path::PathBuf::from(tmp);
        let _ = std::fs::remove_file(&tmp);
        let wide: Vec<u16> = tmp.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let created = unsafe {
            CreateFileW(
                PCWSTR(wide.as_ptr()),
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                Some(&attrs),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
        };
        unsafe {
            LocalFree(Some(HLOCAL(sd.0)));
        }
        let handle = created.map_err(std::io::Error::other)?;
        let mut file = unsafe { std::fs::File::from_raw_handle(handle.0) };
        let written = file.write_all(bytes);
        drop(file);
        if let Err(e) = written {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        std::fs::rename(&tmp, path)
    }

    /// The current user's security identifier, in its string form.
    pub fn current_user_sid() -> std::io::Result<String> {
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
        use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        let mut token = HANDLE::default();
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.map_err(std::io::Error::other)?;
        let token = Handle(token);
        let mut len = 0u32;
        let _ = unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &mut len) };
        // u64 words keep the buffer aligned for TOKEN_USER.
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        unsafe { GetTokenInformation(token.0, TokenUser, Some(buf.as_mut_ptr().cast()), len, &mut len) }
            .map_err(std::io::Error::other)?;
        let user = unsafe { &*(buf.as_ptr() as *const TOKEN_USER) };
        let mut text = PWSTR::null();
        unsafe { ConvertSidToStringSidW(user.User.Sid, &mut text) }.map_err(std::io::Error::other)?;
        let sid = unsafe { text.to_string() }.map_err(std::io::Error::other);
        unsafe {
            LocalFree(Some(HLOCAL(text.0.cast())));
        }
        sid
    }

    /// The bytes free to the current user on the drive that holds `path` (`ENV-01`).
    pub fn free_bytes(path: &Path) -> Option<u64> {
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut free = 0u64;
        unsafe { GetDiskFreeSpaceExW(PCWSTR(wide.as_ptr()), Some(&mut free), None, None) }.ok()?;
        Some(free)
    }

    /// Whether Steam has an account logged in: the value `ActiveUser` under
    /// `HKCU\Software\Valve\Steam\ActiveProcess` is not 0 (`ENV-01`, `INT-STM-01`). Only that
    /// comparison leaves this function; the value itself is the account's number and is kept
    /// nowhere. None when the value cannot be read.
    pub fn steam_active_user() -> Option<bool> {
        user_dword(r"Software\Valve\Steam\ActiveProcess", "ActiveUser").map(|n| n != 0)
    }

    /// A `REG_DWORD` value under `HKEY_CURRENT_USER`, read in-process; None when the value is
    /// missing or of another type.
    pub fn user_dword(key: &str, value: &str) -> Option<u32> {
        use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
        let key = wide(key);
        let value = wide(value);
        let mut data = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(key.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_DWORD,
                None,
                Some(&mut data as *mut u32 as *mut _),
                Some(&mut size),
            )
        };
        status.is_ok().then_some(data)
    }

    /// A `REG_SZ` value under `HKEY_CURRENT_USER`, read in-process; None when the value is missing
    /// or of another type.
    pub fn user_string(key: &str, value: &str) -> Option<String> {
        use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
        let key = wide(key);
        let value = wide(value);
        let read = |buf: Option<&mut Vec<u16>>, size: &mut u32| unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                PCWSTR(key.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                buf.map(|b| b.as_mut_ptr() as *mut _),
                Some(size),
            )
        };
        // The value can grow between the two calls; a second miss reads it once more.
        for _ in 0..3 {
            let mut size = 0u32;
            if read(None, &mut size).is_err() {
                return None;
            }
            let mut buf = vec![0u16; (size as usize).div_ceil(2)];
            if read(Some(&mut buf), &mut size).is_ok() {
                let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                return Some(String::from_utf16_lossy(&buf[..len]));
            }
        }
        None
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Makes the empty folder `link` a directory junction to `target`, an absolute path, through
    /// `FSCTL_SET_REPARSE_POINT`: what `mklink /J` does, in-process. `link` is made here and
    /// removed again when the junction cannot be set.
    pub fn make_junction(link: &Path, target: &Path) -> std::io::Result<()> {
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_WRITE, FILE_SHARE_NONE,
            OPEN_EXISTING,
        };
        use windows::Win32::System::IO::DeviceIoControl;
        const FSCTL_SET_REPARSE_POINT: u32 = 0x0009_00A4;

        let buffer = mount_point_buffer(target)?;
        std::fs::create_dir(link)?;
        let set = (|| {
            let link_w: Vec<u16> = link.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let handle = unsafe {
                CreateFileW(
                    PCWSTR(link_w.as_ptr()),
                    FILE_GENERIC_WRITE.0,
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
                    None,
                )
            }
            .map_err(std::io::Error::other)?;
            let handle = Handle(handle);
            let mut returned = 0u32;
            unsafe {
                DeviceIoControl(
                    handle.0,
                    FSCTL_SET_REPARSE_POINT,
                    Some(buffer.as_ptr() as *const _),
                    buffer.len() as u32,
                    None,
                    0,
                    Some(&mut returned),
                    None,
                )
            }
            .map_err(std::io::Error::other)
        })();
        if set.is_err() {
            let _ = std::fs::remove_dir(link);
        }
        set
    }

    /// The `REPARSE_DATA_BUFFER` of a mount point: the tag, the data length, a reserved
    /// word, the offset and length of the substitute name (`\??\` and the target) and of the
    /// print name (the target), then both names, each ending in a NUL.
    fn mount_point_buffer(target: &Path) -> std::io::Result<Vec<u8>> {
        const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
        let print: Vec<u16> = target.as_os_str().encode_wide().collect();
        let substitute: Vec<u16> = r"\??\".encode_utf16().chain(print.iter().copied()).collect();
        let (sub_bytes, print_bytes) = (substitute.len() * 2, print.len() * 2);
        // Four u16 of name offsets and lengths, then both names with their NULs.
        let data_len = 8 + sub_bytes + 2 + print_bytes + 2;
        if data_len > 16 * 1024 - 8 {
            return Err(std::io::Error::other("the junction's target path is too long"));
        }
        let mut buf = Vec::with_capacity(8 + data_len);
        buf.extend_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
        buf.extend_from_slice(&(data_len as u16).to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&(sub_bytes as u16).to_le_bytes());
        buf.extend_from_slice(&((sub_bytes + 2) as u16).to_le_bytes());
        buf.extend_from_slice(&(print_bytes as u16).to_le_bytes());
        for c in substitute
            .iter()
            .chain(std::iter::once(&0))
            .chain(print.iter())
            .chain(std::iter::once(&0))
        {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        Ok(buf)
    }

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
        fn int_ar_06_a_junction_is_made_in_process_and_reaches_its_target() {
            let dir = std::env::temp_dir().join(format!("skd-junction-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            let target = dir.join("target folder");
            std::fs::create_dir_all(&target).unwrap();
            std::fs::write(target.join("f.txt"), b"x").unwrap();
            let link = dir.join("@link");
            make_junction(&link, &target).unwrap();
            assert_eq!(std::fs::read(link.join("f.txt")).unwrap(), b"x");
            assert_eq!(std::fs::read_link(&link).unwrap(), target);
            // A second junction where a folder is already fails and leaves that folder alone.
            assert!(make_junction(&link, &target).is_err());
            assert!(link.join("f.txt").is_file());
            std::fs::remove_dir(&link).unwrap();
            assert!(target.join("f.txt").is_file());
            std::fs::remove_dir_all(&dir).unwrap();
        }

        #[test]
        fn int_stm_01_a_missing_registry_value_reads_as_none() {
            assert_eq!(user_dword(r"Software\shakedown-no-such-key", "ActiveUser"), None);
            assert_eq!(user_string(r"Software\shakedown-no-such-key", "SteamPath"), None);
            // Every user hive has its keyboard layout preload list, a string.
            assert!(user_string(r"Keyboard Layout\Preload", "1").is_some_and(|v| !v.is_empty()));
            assert_eq!(user_dword(r"Keyboard Layout\Preload", "1"), None);
        }

        #[test]
        fn ses_03_a_bound_udp_port_is_held_and_read_with_its_address() {
            let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            let port = socket.local_addr().unwrap().port();
            assert_eq!(udp_ports_held(&[port]), vec![port]);
            assert!(
                listening_addresses(std::process::id())
                    .contains(&std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, port)),
                "the UDP table lists the test's socket"
            );
            drop(socket);
            assert_eq!(udp_ports_held(&[port]), Vec::<u16>::new());
        }

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
        fn sec_02_a_private_file_grants_the_current_user_only() {
            use windows::Win32::Foundation::{HLOCAL, LocalFree};
            use windows::Win32::Security::Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            };
            use windows::Win32::Security::{DACL_SECURITY_INFORMATION, GetFileSecurityW, PSECURITY_DESCRIPTOR};

            let dir = std::env::temp_dir().join(format!("skd-sec02-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let file = dir.join("runner.json");
            // A file already there, as a runner that died leaves it, is replaced.
            std::fs::write(&file, "old").unwrap();
            write_private(&file, b"{\"port\":1}").unwrap();
            assert_eq!(std::fs::read_to_string(&file).unwrap(), "{\"port\":1}");

            let wide: Vec<u16> = file.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let mut len = 0u32;
            let _ = unsafe { GetFileSecurityW(PCWSTR(wide.as_ptr()), DACL_SECURITY_INFORMATION.0, None, 0, &mut len) };
            let mut buf = vec![0u64; (len as usize).div_ceil(8)];
            let sd = PSECURITY_DESCRIPTOR(buf.as_mut_ptr().cast());
            assert!(
                unsafe {
                    GetFileSecurityW(
                        PCWSTR(wide.as_ptr()),
                        DACL_SECURITY_INFORMATION.0,
                        Some(sd),
                        len,
                        &mut len,
                    )
                }
                .as_bool()
            );
            let sddl = dacl_text(sd);
            std::fs::remove_dir_all(&dir).unwrap();

            // Protected (P), one rule: full control (FA) for the current user, nothing inherited.
            // The expected access list is built from the user's SID and written out the same way,
            // because SDDL text names some accounts by an alias, such as `LA` for the built-in
            // Administrator, instead of by their SID.
            let expected = format!("D:P(A;;FA;;;{})", current_user_sid().unwrap());
            let expected: Vec<u16> = expected.encode_utf16().chain(std::iter::once(0)).collect();
            let mut want = PSECURITY_DESCRIPTOR::default();
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(
                    PCWSTR(expected.as_ptr()),
                    SDDL_REVISION_1,
                    &mut want,
                    None,
                )
            }
            .unwrap();
            let expected = dacl_text(want);
            unsafe {
                LocalFree(Some(HLOCAL(want.0)));
            }
            assert_eq!(sddl, expected);
        }

        /// The access list of a security descriptor as SDDL text.
        fn dacl_text(sd: windows::Win32::Security::PSECURITY_DESCRIPTOR) -> String {
            use windows::Win32::Foundation::{HLOCAL, LocalFree};
            use windows::Win32::Security::Authorization::{
                ConvertSecurityDescriptorToStringSecurityDescriptorW, SDDL_REVISION_1,
            };
            use windows::Win32::Security::DACL_SECURITY_INFORMATION;

            let mut text = PWSTR::null();
            unsafe {
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    sd,
                    SDDL_REVISION_1,
                    DACL_SECURITY_INFORMATION,
                    &mut text,
                    None,
                )
            }
            .unwrap();
            let sddl = unsafe { text.to_string() }.unwrap();
            unsafe {
                LocalFree(Some(HLOCAL(text.0.cast())));
            }
            sddl
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

    pub const RELAY_VAR: &str = "SHAKEDOWN_RUNNER_RELAY";

    pub fn start_runner(_exe: &Path) -> std::io::Result<&'static str> {
        Err(std::io::Error::other("needs Windows"))
    }

    pub fn start_by_wmi(_exe: &Path) -> std::io::Result<()> {
        Err(std::io::Error::other("needs Windows"))
    }

    pub fn on_console_event(_f: fn(u32)) {}
    pub fn on_stop_keys(_f: fn(u32)) {}

    pub fn spawn_breakaway(
        _exe: &Path,
        _args: &[String],
        _env: &[(String, String)],
        _breakaway: bool,
    ) -> std::io::Result<u32> {
        Err(std::io::Error::other("needs Windows"))
    }

    /// Unknown on other systems: a client then goes by the runner's port alone.
    pub fn process_started(_pid: u32) -> Option<String> {
        None
    }

    pub fn terminate_pid(_pid: u32, _started_at: &str) -> bool {
        false
    }

    pub fn tcp_peer_pid(_peer_port: u16, _local_port: u16) -> Option<u32> {
        None
    }

    /// Without the system's UDP table, a port counts as held when a loopback bind to it fails.
    pub fn udp_ports_held(ports: &[u16]) -> Vec<u16> {
        ports
            .iter()
            .copied()
            .filter(|p| std::net::UdpSocket::bind(("127.0.0.1", *p)).is_err())
            .collect()
    }

    pub fn listening_addresses(_pid: u32) -> Vec<std::net::SocketAddrV4> {
        vec![]
    }

    impl Child {
        pub fn wait(&self, _wait: Duration) -> Option<u32> {
            Some(0)
        }
        pub fn terminate(&self) {}
    }

    /// Writes `bytes` to `path` as a file only the current user can read (`SEC-02`).
    pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = std::path::PathBuf::from(tmp);
        let _ = std::fs::remove_file(&tmp);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp)?;
        file.write_all(bytes)?;
        drop(file);
        std::fs::rename(&tmp, path)
    }

    pub fn free_bytes(_path: &Path) -> Option<u64> {
        None
    }

    pub fn steam_active_user() -> Option<bool> {
        None
    }

    pub fn make_junction(_link: &Path, _target: &Path) -> std::io::Result<()> {
        Err(std::io::Error::other("needs Windows"))
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
