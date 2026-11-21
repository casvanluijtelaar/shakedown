//! Starving a game process the test started of CPU, by its process id: it is suspended for
//! `off` and runs for `on`, over and over, until the `Starve` is dropped, which leaves it running.
//! Used on processes a test started only (`HLT-01`, `US-094`, `SPK-22`).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtSuspendProcess(process: HANDLE) -> i32;
    fn NtResumeProcess(process: HANDLE) -> i32;
}

pub struct Starve {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Starve {
    pub fn start(pid: u32, off: Duration, on: Duration) -> Starve {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = unsafe { OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) }.expect("open the process");
        // A suspend that fails, such as for want of access, starves nothing: the test stops there.
        let status = unsafe { NtSuspendProcess(handle) };
        unsafe { NtResumeProcess(handle) };
        assert_eq!(status, 0, "NtSuspendProcess of {pid}: {status:#x}");
        let raw = handle.0 as usize;
        let thread = std::thread::spawn(move || {
            let h = HANDLE(raw as *mut _);
            while !flag.load(Ordering::SeqCst) {
                unsafe { NtSuspendProcess(h) };
                std::thread::sleep(off);
                unsafe { NtResumeProcess(h) };
                std::thread::sleep(on);
            }
            unsafe {
                let _ = CloseHandle(h);
            }
        });
        Starve {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Starve {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
