//! Touch ID, for unlocking locked notes without typing the lock password.
//!
//! [`Biometrics`] is the seam: [`SystemTouchId`] asks macOS's
//! LocalAuthentication in the app, tests set their own. Without the global,
//! or off macOS, there is no Touch ID and the password is asked for.

use std::sync::Arc;

use gpui_kit::{App, Global};

/// What Touch ID can do.
pub trait Biometrics: Send + Sync {
    /// Whether this device has Touch ID set up for its user now.
    fn available(&self) -> bool;
    /// Asks for a fingerprint, saying why with `reason`; the answer comes
    /// on the channel (`false` for a cancel or a failure).
    fn authenticate(&self, reason: &str) -> async_channel::Receiver<bool>;
}

/// The Touch ID the app uses.
pub struct TouchId(pub Arc<dyn Biometrics>);

impl Global for TouchId {}

/// Whether Touch ID can be used now.
pub fn available(cx: &App) -> bool {
    cx.try_global::<TouchId>()
        .is_some_and(|touch_id| touch_id.0.available())
}

/// Asks for a fingerprint, or `None` when there is no Touch ID.
pub fn authenticate(reason: &str, cx: &App) -> Option<async_channel::Receiver<bool>> {
    let touch_id = cx.try_global::<TouchId>()?;
    touch_id
        .0
        .available()
        .then(|| touch_id.0.authenticate(reason))
}

/// macOS's LocalAuthentication, biometrics only: the fallback is Kanso's own
/// lock password form, not the account password.
pub struct SystemTouchId;

#[cfg(target_os = "macos")]
mod system {
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool};
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;

    #[link(name = "LocalAuthentication", kind = "framework")]
    unsafe extern "C" {}

    /// `LAPolicyDeviceOwnerAuthenticationWithBiometrics`.
    const BIOMETRICS: isize = 1;

    fn context() -> Retained<AnyObject> {
        // SAFETY: `+[LAContext new]` returns a new, retained context.
        unsafe { msg_send![class!(LAContext), new] }
    }

    impl super::Biometrics for super::SystemTouchId {
        fn available(&self) -> bool {
            let context = context();
            // SAFETY: a valid context and policy; a null error pointer is
            // allowed.
            unsafe {
                msg_send![
                    &*context,
                    canEvaluatePolicy: BIOMETRICS,
                    error: std::ptr::null_mut::<*mut AnyObject>()
                ]
            }
        }

        fn authenticate(&self, reason: &str) -> async_channel::Receiver<bool> {
            let (tx, rx) = async_channel::bounded(1);
            let context = context();
            let reason = NSString::from_str(reason);
            // The context stays alive until it answers.
            let kept = context.clone();
            let reply = RcBlock::new(move |success: Bool, _error: *mut AnyObject| {
                let _ = &kept;
                tx.try_send(success.as_bool()).ok();
            });
            // SAFETY: a valid context, policy, reason and reply block of
            // the `void (^)(BOOL, NSError *)` shape LocalAuthentication
            // calls once, on a queue of its own.
            unsafe {
                let _: () = msg_send![
                    &*context,
                    evaluatePolicy: BIOMETRICS,
                    localizedReason: &*reason,
                    reply: &*reply
                ];
            }
            rx
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl Biometrics for SystemTouchId {
    fn available(&self) -> bool {
        false
    }

    fn authenticate(&self, _: &str) -> async_channel::Receiver<bool> {
        let (tx, rx) = async_channel::bounded(1);
        tx.try_send(false).ok();
        rx
    }
}

/// A Touch ID that answers as told, for tests.
#[cfg(test)]
pub struct FakeTouchId {
    pub available: bool,
    pub answer: std::sync::Mutex<bool>,
    pub asked: std::sync::atomic::AtomicUsize,
}

#[cfg(test)]
impl FakeTouchId {
    pub fn new(answer: bool) -> Arc<FakeTouchId> {
        Arc::new(FakeTouchId {
            available: true,
            answer: std::sync::Mutex::new(answer),
            asked: Default::default(),
        })
    }

    pub fn answer(&self, answer: bool) {
        *self.answer.lock().unwrap() = answer;
    }

    pub fn asked(&self) -> usize {
        self.asked.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
impl Biometrics for FakeTouchId {
    fn available(&self) -> bool {
        self.available
    }

    fn authenticate(&self, _: &str) -> async_channel::Receiver<bool> {
        self.asked
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = async_channel::bounded(1);
        tx.try_send(*self.answer.lock().unwrap()).ok();
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::{Biometrics as _, SystemTouchId};

    /// The real LocalAuthentication links and answers whether Touch ID is
    /// set up, without asking for a fingerprint. Run on demand.
    #[test]
    #[ignore = "asks this Mac's LocalAuthentication"]
    fn the_system_says_whether_touch_id_is_set_up() {
        println!("Touch ID available: {}", SystemTouchId.available());
    }
}
