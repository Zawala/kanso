//! Keeping the library out of backups.
//!
//! On the phone the notes are not in the device's backups (iCloud, a
//! computer's): the app says so in Settings, and the notes reach other
//! devices through sync alone. On iOS the library's folder carries the
//! "excluded from backup" flag (`kCFURLIsExcludedFromBackupKey`), set on every
//! open, so the app, the background job and anything else that opens the
//! library all set it. Android excludes the app's files in its manifest
//! (`allowBackup`, `fullBackupContent`, `dataExtractionRules`), not here.
//!
//! The flag is also set and read on a Mac, so the tests can check it there.

use std::path::Path;

/// Marks the folder `folder` as excluded from backups. Does nothing where
/// the platform has no such flag. A failure is an error: the library must not
/// quietly end up in a backup.
pub(crate) fn exclude(folder: &Path) -> Result<(), String> {
    #[cfg(target_vendor = "apple")]
    {
        apple::set(folder, true)
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        let _ = folder;
        Ok(())
    }
}

/// Whether `folder` is excluded from backups; `None` where the platform has
/// no such flag (Android's exclusion is in the manifest) or the folder can't
/// be read.
pub(crate) fn is_excluded(folder: &Path) -> Option<bool> {
    if !folder.is_dir() {
        return None;
    }
    #[cfg(target_vendor = "apple")]
    {
        apple::get(folder)
    }
    #[cfg(not(target_vendor = "apple"))]
    {
        let _ = folder;
        None
    }
}

#[cfg(target_vendor = "apple")]
mod apple {
    use std::ffi::c_void;
    use std::path::Path;
    use std::ptr;

    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::url::CFURL;
    use core_foundation_sys::base::{Boolean, CFRelease, CFTypeRef};
    use core_foundation_sys::error::CFErrorRef;
    use core_foundation_sys::string::CFStringRef;
    use core_foundation_sys::url::{
        CFURLRef, CFURLSetResourcePropertyForKey, kCFURLIsExcludedFromBackupKey,
    };

    // Declared in CFURL.h; core-foundation-sys leaves it out.
    unsafe extern "C" {
        fn CFURLCopyResourcePropertyForKey(
            url: CFURLRef,
            key: CFStringRef,
            value_out: *mut c_void,
            error: *mut CFErrorRef,
        ) -> Boolean;
    }

    pub(super) fn set(folder: &Path, excluded: bool) -> Result<(), String> {
        let url = CFURL::from_path(folder, true)
            .ok_or_else(|| "the library's folder has no URL".to_owned())?;
        let value = if excluded {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };
        let mut error: CFErrorRef = ptr::null_mut();
        // SAFETY: the URL, the key and the value are live CF objects for the
        // call; `error` is a valid out pointer, released below when set.
        let done = unsafe {
            CFURLSetResourcePropertyForKey(
                url.as_concrete_TypeRef(),
                kCFURLIsExcludedFromBackupKey,
                value.as_CFTypeRef(),
                &mut error,
            )
        };
        release(error);
        if done == 0 {
            return Err("the library's folder couldn't be kept out of backups".to_owned());
        }
        Ok(())
    }

    pub(super) fn get(folder: &Path) -> Option<bool> {
        let url = CFURL::from_path(folder, true)?;
        let mut value: CFTypeRef = ptr::null();
        let mut error: CFErrorRef = ptr::null_mut();
        // SAFETY: as in `set`; `value` receives a retained CF object or is
        // left null.
        let done = unsafe {
            CFURLCopyResourcePropertyForKey(
                url.as_concrete_TypeRef(),
                kCFURLIsExcludedFromBackupKey,
                (&mut value as *mut CFTypeRef).cast(),
                &mut error,
            )
        };
        release(error);
        if done == 0 {
            return None;
        }
        if value.is_null() {
            return Some(false);
        }
        // SAFETY: the copy rule gave us the value's one reference, and the
        // key's value is a CFBoolean.
        let boolean = unsafe { CFBoolean::wrap_under_create_rule(value as _) };
        Some(boolean.into())
    }

    fn release(error: CFErrorRef) {
        if !error.is_null() {
            // SAFETY: an error out parameter is owned by the caller.
            unsafe { CFRelease(error as CFTypeRef) };
        }
    }
}
