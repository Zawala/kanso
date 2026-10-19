// no-prop-test: no observable oracle; these only ask AppKit to move focus.
//! Bringing Kanso to the front after an update restarts it.
//!
//! Since macOS 14 an app can't simply take focus: the app that has it must
//! yield it first, or `activate` is ignored. So the copy that's quitting
//! yields to its own bundle identifier before it opens the new copy, and
//! the new copy activates itself once its window is up. On older macOS,
//! and elsewhere, these fall back to what was there before.

/// Lets the next Kanso to start take focus from this one.
pub fn yield_to_next_launch() {
    #[cfg(target_os = "macos")]
    mac::yield_to_own_bundle();
}

/// Brings this Kanso to the front (with the cooperative `activate` where
/// macOS has it).
pub fn activate() {
    #[cfg(target_os = "macos")]
    mac::activate();
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{CString, c_char, c_void};

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    fn class(name: &str) -> Option<Id> {
        let name = CString::new(name).ok()?;
        // SAFETY: a NUL-terminated name; the runtime returns nil if unknown.
        let class = unsafe { objc_getClass(name.as_ptr()) };
        (!class.is_null()).then_some(class)
    }

    fn sel(name: &str) -> Sel {
        let name = CString::new(name).expect("selector names have no NUL");
        // SAFETY: a NUL-terminated selector name; registering is idempotent.
        unsafe { sel_registerName(name.as_ptr()) }
    }

    /// Sends a message that takes no argument and returns an object.
    fn send0(receiver: Id, selector: &str) -> Option<Id> {
        // SAFETY: the selectors used with this take no arguments and return
        // an object (or nil).
        let send: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let result = unsafe { send(receiver, sel(selector)) };
        (!result.is_null()).then_some(result)
    }

    fn responds(receiver: Id, selector: &str) -> bool {
        // SAFETY: `respondsToSelector:` takes a selector and returns a BOOL.
        let send: unsafe extern "C" fn(Id, Sel, Sel) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { send(receiver, sel("respondsToSelector:"), sel(selector)) }
    }

    fn shared_app() -> Option<Id> {
        send0(class("NSApplication")?, "sharedApplication")
    }

    pub fn yield_to_own_bundle() {
        let Some(app) = shared_app() else { return };
        if !responds(app, "yieldActivationToApplicationWithBundleIdentifier:") {
            return;
        }
        let Some(bundle) = class("NSBundle").and_then(|c| send0(c, "mainBundle")) else {
            return;
        };
        let Some(identifier) = send0(bundle, "bundleIdentifier") else {
            return;
        };
        // SAFETY: takes an NSString and returns nothing.
        let send: unsafe extern "C" fn(Id, Sel, Id) =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            send(
                app,
                sel("yieldActivationToApplicationWithBundleIdentifier:"),
                identifier,
            )
        };
    }

    pub fn activate() {
        let Some(app) = shared_app() else { return };
        if responds(app, "activate") {
            // SAFETY: `activate` (macOS 14) takes no arguments and returns nothing.
            let send: unsafe extern "C" fn(Id, Sel) =
                unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
            unsafe { send(app, sel("activate")) };
        } else {
            // SAFETY: `activateIgnoringOtherApps:` takes a BOOL.
            let send: unsafe extern "C" fn(Id, Sel, bool) =
                unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
            unsafe { send(app, sel("activateIgnoringOtherApps:"), true) };
        }
    }
}
