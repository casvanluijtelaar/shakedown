//! Real key input to the game's own window (`ACT-05`, `SPK-11`): posts the window messages a
//! key press makes, `WM_KEYDOWN` and `WM_KEYUP`, to the top-level window of the game process the
//! extension is loaded in. A posted message goes to that one window's queue, so neither the
//! foreground, the focus nor the desktop's input changes, and no other window receives anything.

/// The lParam of a key message: repeat count 1, the scan code, the extended-key flag, and for a
/// key going up the previous-state and transition flags.
pub fn lparam(dik: u32, up: bool) -> isize {
    let (scan, extended) = scan(dik);
    let mut l: u32 = 1 | (scan << 16);
    if extended {
        l |= 1 << 24;
    }
    if up {
        l |= (1 << 30) | (1 << 31);
    }
    l as i32 as isize
}

/// A DIK code is a set-1 scan code; the codes from 0x80 are the keys whose scan code is prefixed
/// with 0xE0, such as the arrow keys and the right-hand Ctrl.
pub fn scan(dik: u32) -> (u32, bool) {
    if dik >= 0x80 { (dik - 0x80, true) } else { (dik, false) }
}

/// Whether a DIK code names a key, 1 to 255.
pub fn valid(dik: f64) -> Option<u32> {
    (dik.fract() == 0.0 && (1.0..=255.0).contains(&dik)).then_some(dik as u32)
}

#[cfg(windows)]
mod imp {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{MAPVK_VSC_TO_VK_EX, MapVirtualKeyW};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GW_OWNER, GetWindow, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, WM_KEYDOWN,
        WM_KEYUP,
    };
    use windows::core::BOOL;

    unsafe extern "system" fn each(hwnd: HWND, found: LPARAM) -> BOOL {
        let mut pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            let owned = GetWindow(hwnd, GW_OWNER).map(|o| !o.is_invalid()).unwrap_or(false);
            if pid == GetCurrentProcessId() && IsWindowVisible(hwnd).as_bool() && !owned {
                *(found.0 as *mut isize) = hwnd.0 as isize;
                return BOOL(0);
            }
        }
        BOOL(1)
    }

    /// The game process's own visible top-level window, if it has one.
    pub fn window() -> Option<HWND> {
        let mut found: isize = 0;
        unsafe {
            let _ = EnumWindows(Some(each), LPARAM(&mut found as *mut isize as isize));
        }
        (found != 0).then(|| HWND(found as *mut _))
    }

    /// Posts the key's down or up message to the window.
    pub fn post(hwnd: HWND, dik: u32, up: bool) -> bool {
        let (scan, extended) = super::scan(dik);
        let code = if extended { scan | 0xE000 } else { scan };
        let vk = unsafe { MapVirtualKeyW(code, MAPVK_VSC_TO_VK_EX) };
        let msg = if up { WM_KEYUP } else { WM_KEYDOWN };
        unsafe { PostMessageW(Some(hwnd), msg, WPARAM(vk as usize), LPARAM(super::lparam(dik, up))).is_ok() }
    }
}

/// `"shakedown" callExtension ["key", [dik, up]]`: `ok` once the message is posted, `no-window`
/// when this game process has no window, `bad-key` for a DIK code out of 1 to 255.
pub fn key(dik: f64, up: bool) -> &'static str {
    let Some(dik) = valid(dik) else { return "bad-key" };
    #[cfg(windows)]
    {
        match imp::window() {
            Some(hwnd) => {
                if imp::post(hwnd, dik, up) {
                    "ok"
                } else {
                    "no-window"
                }
            }
            None => "no-window",
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (dik, up);
        "no-window"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn act_05_a_key_down_carries_its_scan_code() {
        assert_eq!(lparam(65, false), 1 | (65 << 16));
    }

    #[test]
    fn act_05_a_key_up_sets_the_transition_flags() {
        assert_eq!(lparam(65, true) as u32, 1 | (65 << 16) | (1 << 30) | (1 << 31));
    }

    #[test]
    fn act_05_a_dik_from_0x80_is_an_extended_key() {
        assert_eq!(scan(0xC8), (0x48, true));
        assert_eq!(lparam(0xC8, false) as u32, 1 | (0x48 << 16) | (1 << 24));
    }

    #[test]
    fn act_05_a_dik_out_of_range_is_refused() {
        assert_eq!(valid(0.0), None);
        assert_eq!(valid(256.0), None);
        assert_eq!(valid(6.5), None);
        assert_eq!(valid(50.0), Some(50));
        assert_eq!(key(0.0, false), "bad-key");
    }
}
