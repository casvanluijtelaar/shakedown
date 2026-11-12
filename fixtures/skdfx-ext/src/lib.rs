//! `skdfx_x64.dll`, the extension of the fixture mod `@skd_fixture` (`FIX-01`,
//! `spec/contract/fixture-mod.md`): it ends the game process it is loaded in, on demand, after a
//! delay in real time, so a test makes a crash or an exit at a known point of a check.
//!
//! - `"skdfx" callExtension ["crash", [delay]]`: after `delay` seconds a thread of the extension
//!   writes to an unmapped address, an access violation in native code, so the engine's own crash
//!   handling runs, writes its dumps and ends the process unasked. A process the engine's handling
//!   has not ended `WATCHDOG` after the fault, as a headless client sometimes is for a minute
//!   (`SPK-16`), is ended by the extension with the exit code of an access violation.
//! - `"skdfx" callExtension ["exit", [delay, code]]`: after `delay` seconds the process ends with
//!   exit code `code`, through `TerminateProcess`, which runs no handler and leaves no dump.
//!
//! Each returns `true` at once, or `false` for an argument out of its range.

use std::time::Duration;

use arma_rs::{Extension, arma};

/// How long after the fault the extension ends a process the engine has not ended (`SPK-16`).
const WATCHDOG: Duration = Duration::from_secs(5);
/// The exit code of an access violation, `STATUS_ACCESS_VIOLATION`.
const ACCESS_VIOLATION: u32 = 0xC000_0005;

/// The longest delay either call takes, in seconds.
const DELAY_MAX: f64 = 60.0;

#[arma]
fn init() -> Extension {
    Extension::build()
        .command("crash", crash)
        .command("exit", exit)
        .finish()
}

fn delay(seconds: f64) -> Option<Duration> {
    (seconds.is_finite() && (0.0..=DELAY_MAX).contains(&seconds)).then(|| Duration::from_secs_f64(seconds))
}

fn crash(seconds: f64) -> bool {
    let Some(wait) = delay(seconds) else { return false };
    std::thread::spawn(move || {
        std::thread::sleep(wait);
        std::thread::spawn(|| {
            std::thread::sleep(WATCHDOG);
            end(ACCESS_VIOLATION);
        });
        fault();
    });
    true
}

/// The exit code arrives as text, which keeps every digit of a large code.
fn exit(seconds: f64, code: String) -> bool {
    let Some(wait) = delay(seconds) else { return false };
    let Some(code) = exit_code(&code) else { return false };
    std::thread::spawn(move || {
        std::thread::sleep(wait);
        end(code);
    });
    true
}

/// An exit code from 1 to 2147483647, written as a whole number, quoted or not.
fn exit_code(text: &str) -> Option<u32> {
    let code: u32 = text.trim().trim_matches('"').parse().ok()?;
    (1..=i32::MAX as u32).contains(&code).then_some(code)
}

/// A write to the first page of the address space, which is never mapped on Windows: an access
/// violation the process does not handle itself.
#[allow(clippy::manual_dangling_ptr)]
fn fault() {
    let address = std::hint::black_box(0x10usize) as *mut u32;
    // SAFETY: none; the write is meant to fault, and the process does not go on after it.
    unsafe { std::ptr::write_volatile(address, 0xDEAD) };
}

#[cfg(windows)]
fn end(code: u32) {
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn TerminateProcess(process: isize, code: u32) -> i32;
    }
    // SAFETY: both are plain Win32 calls on the current process's pseudo handle.
    unsafe {
        TerminateProcess(GetCurrentProcess(), code);
    }
}

#[cfg(not(windows))]
fn end(code: u32) {
    std::process::exit(code as i32);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `FIX-01`: a delay outside 0 to 60 s and an exit code outside 1 to 2147483647 are refused
    /// with `false`, and nothing is scheduled.
    #[test]
    fn fix_01_arguments_out_of_range_are_refused() {
        assert!(delay(-0.1).is_none());
        assert!(delay(60.5).is_none());
        assert!(delay(f64::NAN).is_none());
        assert_eq!(delay(0.5), Some(Duration::from_millis(500)));
        assert!(!crash(61.0));
        assert_eq!(exit_code("\"42\""), Some(42));
        assert_eq!(exit_code("2147483647"), Some(2_147_483_647));
        assert!(exit_code("0").is_none());
        assert!(exit_code("2147483648").is_none());
        assert!(exit_code("1.5").is_none());
        assert!(!exit(1.0, "0".into()));
        assert!(!exit(-1.0, "3".into()));
    }

    /// `FIX-01`: the extension answers its two calls through the engine's interface.
    #[test]
    fn fix_01_the_extension_answers_its_calls() {
        let ext = init().testing();
        let (out, code) = ext.call("crash", Some(vec!["99".into()]));
        assert_eq!((out.as_str(), code), ("false", 0));
        let (out, code) = ext.call("exit", Some(vec!["1".into(), "0".into()]));
        assert_eq!((out.as_str(), code), ("false", 0));
    }
}
