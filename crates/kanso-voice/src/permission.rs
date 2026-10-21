//! Whether Kanso may use the microphone.
//!
//! On macOS the system asks the person once, and remembers; the app's
//! bundle must carry `NSMicrophoneUsageDescription` (and, when sandboxed or
//! hardened, the audio-input entitlement) or the question never shows.
//! Elsewhere there is nothing to ask, so the answer is always yes.

/// The person's answer about the microphone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    /// Kanso may record.
    Granted,
    /// The person said no, or a policy forbids it; only System Settings can
    /// change it now.
    Denied,
    /// Not asked yet: [`request`] shows the question.
    Undetermined,
}

#[cfg(target_os = "macos")]
mod platform {
    use std::sync::Mutex;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

    use super::Permission;

    /// Maps AVFoundation's status onto [`Permission`].
    // no-prop-test: four values; `every_status_maps` checks each.
    pub(super) fn from_status(status: AVAuthorizationStatus) -> Permission {
        match status {
            AVAuthorizationStatus::Authorized => Permission::Granted,
            AVAuthorizationStatus::NotDetermined => Permission::Undetermined,
            _ => Permission::Denied,
        }
    }

    // no-prop-test: asks the system; `from_status` holds the mapping.
    pub fn status() -> Permission {
        // SAFETY: AVMediaTypeAudio is a constant AVFoundation exports.
        let Some(audio) = (unsafe { AVMediaTypeAudio }) else {
            return Permission::Undetermined;
        };
        // SAFETY: audio is a valid media type for this method.
        from_status(unsafe { AVCaptureDevice::authorizationStatusForMediaType(audio) })
    }

    // no-prop-test: shows the system's question.
    pub fn request(done: impl FnOnce(bool) + Send + 'static) {
        // SAFETY: as in `status`.
        let Some(audio) = (unsafe { AVMediaTypeAudio }) else {
            done(false);
            return;
        };
        // The block may be called on any queue, once; the Mutex hands the
        // FnOnce to that call.
        let done = Mutex::new(Some(done));
        let handler = RcBlock::new(move |granted: Bool| {
            let callback = done.lock().ok().and_then(|mut slot| slot.take());
            if let Some(callback) = callback {
                callback(granted.as_bool());
            }
        });
        // SAFETY: audio is a valid media type, and the handler is a block
        // AVFoundation copies before returning.
        unsafe { AVCaptureDevice::requestAccessForMediaType_completionHandler(audio, &handler) };
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::Permission;

    // no-prop-test: a constant answer.
    pub fn status() -> Permission {
        Permission::Granted
    }

    // no-prop-test: answers at once.
    pub fn request(done: impl FnOnce(bool) + Send + 'static) {
        done(true);
    }
}

/// Whether Kanso may use the microphone now. Cheap; never shows anything.
pub fn status() -> Permission {
    platform::status()
}

/// Asks for the microphone if the person hasn't answered yet, and calls
/// `done` with the answer, on some other thread, once. When they have
/// answered before, `done` gets that answer without a question.
pub fn request(done: impl FnOnce(bool) + Send + 'static) {
    platform::request(done)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use objc2_av_foundation::AVAuthorizationStatus;

    use super::*;

    #[test]
    fn every_status_maps() {
        assert_eq!(
            platform::from_status(AVAuthorizationStatus::Authorized),
            Permission::Granted
        );
        assert_eq!(
            platform::from_status(AVAuthorizationStatus::NotDetermined),
            Permission::Undetermined
        );
        assert_eq!(
            platform::from_status(AVAuthorizationStatus::Denied),
            Permission::Denied
        );
        assert_eq!(
            platform::from_status(AVAuthorizationStatus::Restricted),
            Permission::Denied
        );
    }

    #[test]
    fn asking_for_the_status_is_safe_anywhere() {
        // Whatever this terminal was granted, the call answers.
        let _ = status();
    }
}
