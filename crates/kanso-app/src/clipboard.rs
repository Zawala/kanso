//! Copying a secret (the recovery key) so it stays on this Mac and doesn't
//! stay long (docs/ACCOUNTS.md › Recovery key screen), and asking macOS
//! whether a folder syncs to iCloud. A few calls into AppKit and Foundation
//! through the Objective-C runtime; GPUI links both frameworks already.

use std::path::Path;
use std::time::Duration;

use gpui_kit::*;

/// How long a copied secret stays on the clipboard.
pub const SECRET_LIFETIME: Duration = Duration::from_secs(60);

/// Whether the clipboard still holds what was copied: nothing was copied
/// over it since (its change count is still the one after copying).
pub fn still_ours(copied: isize, now: isize) -> bool {
    copied == now
}

/// Copies `secret` to the clipboard for this Mac only (Universal Clipboard
/// doesn't send it to other devices), marked concealed and transient so
/// clipboard managers skip it, and clears it after [`SECRET_LIFETIME`]
/// unless something else was copied meanwhile. Returns false when the
/// clipboard couldn't be written.
pub fn copy_secret(secret: &str, cx: &mut App) -> bool {
    let Some(copied) = mac::copy_secret(secret) else {
        return false;
    };
    cx.spawn(async move |cx| {
        cx.background_executor().timer(SECRET_LIFETIME).await;
        // AppKit's pasteboard, from the main thread.
        cx.update(|_| {
            if mac::change_count().is_some_and(|now| still_ours(copied, now)) {
                mac::clear();
            }
        });
    })
    .detach();
    true
}

/// Whether `folder` syncs to iCloud: macOS says it is ubiquitous (iCloud
/// Drive, or Desktop and Documents in iCloud), or it is inside iCloud
/// Drive's own folder.
pub fn syncs_to_icloud(folder: &Path, home: &Path) -> bool {
    crate::account_words::is_icloud_path(folder, home) || mac::is_ubiquitous(folder)
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{CString, c_char, c_void};
    use std::path::Path;

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

    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {
        static NSURLIsUbiquitousItemKey: Id;
    }

    /// `NSPasteboardContentsCurrentHostOnly`.
    const CURRENT_HOST_ONLY: usize = 1;
    const UTF8_TEXT: &str = "public.utf8-plain-text";
    /// nspasteboard.org's markers: don't show it, don't keep it.
    const CONCEALED: &str = "org.nspasteboard.ConcealedType";
    const TRANSIENT: &str = "org.nspasteboard.TransientType";

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

    /// `[NSString stringWithUTF8String:]`, autoreleased.
    fn ns_string(text: &str) -> Option<Id> {
        let text = CString::new(text).ok()?;
        let class = class("NSString")?;
        // SAFETY: `stringWithUTF8String:` takes a C string and returns an
        // autoreleased NSString (or nil).
        let send: unsafe extern "C" fn(Id, Sel, *const c_char) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let string = unsafe { send(class, sel("stringWithUTF8String:"), text.as_ptr()) };
        (!string.is_null()).then_some(string)
    }

    fn general_pasteboard() -> Option<Id> {
        let class = class("NSPasteboard")?;
        // SAFETY: `generalPasteboard` takes no arguments and returns an object.
        let send: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let pasteboard = unsafe { send(class, sel("generalPasteboard")) };
        (!pasteboard.is_null()).then_some(pasteboard)
    }

    fn set_string(pasteboard: Id, text: &str, kind: &str) -> bool {
        let (Some(text), Some(kind)) = (ns_string(text), ns_string(kind)) else {
            return false;
        };
        // SAFETY: `setString:forType:` takes two NSStrings and returns BOOL.
        let send: unsafe extern "C" fn(Id, Sel, Id, Id) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { send(pasteboard, sel("setString:forType:"), text, kind) }
    }

    /// Writes the secret; returns the pasteboard's change count after it.
    pub fn copy_secret(secret: &str) -> Option<isize> {
        let pasteboard = general_pasteboard()?;
        // SAFETY: `prepareForNewContentsWithOptions:` takes an NSUInteger
        // option set and returns the new change count.
        let prepare: unsafe extern "C" fn(Id, Sel, usize) -> isize =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            prepare(
                pasteboard,
                sel("prepareForNewContentsWithOptions:"),
                CURRENT_HOST_ONLY,
            )
        };
        let written = set_string(pasteboard, secret, UTF8_TEXT)
            && set_string(pasteboard, "", CONCEALED)
            && set_string(pasteboard, "", TRANSIENT);
        if !written {
            clear();
            return None;
        }
        change_count()
    }

    pub fn change_count() -> Option<isize> {
        let pasteboard = general_pasteboard()?;
        // SAFETY: `changeCount` takes no arguments and returns NSInteger.
        let send: unsafe extern "C" fn(Id, Sel) -> isize =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        Some(unsafe { send(pasteboard, sel("changeCount")) })
    }

    pub fn clear() {
        if let Some(pasteboard) = general_pasteboard() {
            // SAFETY: `clearContents` takes no arguments and returns NSInteger.
            let send: unsafe extern "C" fn(Id, Sel) -> isize =
                unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
            unsafe { send(pasteboard, sel("clearContents")) };
        }
    }

    /// `NSURLIsUbiquitousItemKey` for the folder.
    pub fn is_ubiquitous(folder: &Path) -> bool {
        let (Some(class), Some(path)) = (class("NSURL"), folder.to_str().and_then(ns_string))
        else {
            return false;
        };
        // SAFETY: `fileURLWithPath:` takes an NSString and returns an
        // autoreleased NSURL.
        let file_url: unsafe extern "C" fn(Id, Sel, Id) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let url = unsafe { file_url(class, sel("fileURLWithPath:"), path) };
        if url.is_null() {
            return false;
        }
        let mut value: Id = std::ptr::null_mut();
        // SAFETY: `getResourceValue:forKey:error:` writes an autoreleased
        // NSNumber (or nil) through the first pointer; nil error is allowed.
        let get: unsafe extern "C" fn(Id, Sel, *mut Id, Id, *mut Id) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        let found = unsafe {
            get(
                url,
                sel("getResourceValue:forKey:error:"),
                &mut value,
                NSURLIsUbiquitousItemKey,
                std::ptr::null_mut(),
            )
        };
        if !found || value.is_null() {
            return false;
        }
        // SAFETY: the value is an NSNumber; `boolValue` returns BOOL.
        let bool_value: unsafe extern "C" fn(Id, Sel) -> bool =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { bool_value(value, sel("boolValue")) }
    }
}

#[cfg(not(target_os = "macos"))]
mod mac {
    use std::path::Path;

    pub fn copy_secret(_: &str) -> Option<isize> {
        None
    }

    pub fn change_count() -> Option<isize> {
        None
    }

    pub fn clear() {}

    pub fn is_ubiquitous(_: &Path) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::still_ours;
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert!(still_ours(7, 7));
        assert!(!still_ours(7, 8));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// The secret is cleared only while nothing else was copied over it.
        #[test]
        fn prop_only_our_copy_is_cleared(copied in any::<isize>(), later in 0isize..3) {
            let now = copied.wrapping_add(later);
            prop_assert_eq!(still_ours(copied, now), later == 0);
        }
    }
}
