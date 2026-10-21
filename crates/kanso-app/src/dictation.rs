//! Voice input through the system's own dictation, started from the
//! editor's Dictate button: macOS dictation, or Windows voice typing. Both
//! type into the focused text the way an input method does, so the note
//! takes what is said like typing (the editor's `dictation_lands_as_spoken`
//! test). The editor puts the keyboard in the note first: dictation types
//! wherever the keyboard is, and anywhere else its words are lost.

/// Whether this system has dictation Kanso can start.
pub const AVAILABLE: bool = cfg!(any(target_os = "macos", target_os = "windows"));

/// Starts the system's dictation, into whatever has the keyboard.
// no-prop-test: no observable oracle; it only asks the system to listen.
pub fn start() {
    #[cfg(target_os = "macos")]
    macos::start();
    #[cfg(target_os = "windows")]
    windows::start();
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    /// `[NSApp startDictation:nil]`, as Edit › Start Dictation sends it.
    pub fn start() {
        let Some(class) = AnyClass::get(c"NSApplication") else {
            return;
        };
        // SAFETY: `sharedApplication` and `startDictation:` are AppKit's
        // own, called on the main thread (from a click) with a nil sender.
        unsafe {
            let app: *mut AnyObject = msg_send![class, sharedApplication];
            if app.is_null() {
                return;
            }
            let sender: *const AnyObject = std::ptr::null();
            let _: () = msg_send![app, startDictation: sender];
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
        VK_LWIN,
    };

    /// Windows has no call that opens voice typing, so this presses its
    /// shortcut, Windows key + H.
    pub fn start() {
        const VK_H: VIRTUAL_KEY = 0x48;
        let key = |vk: VIRTUAL_KEY, up: bool| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let inputs = [
            key(VK_LWIN, false),
            key(VK_H, false),
            key(VK_H, true),
            key(VK_LWIN, true),
        ];
        // SAFETY: four keyboard inputs, with their count and size.
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }
}
