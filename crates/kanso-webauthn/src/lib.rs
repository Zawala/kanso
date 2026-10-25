//! Native platform passkey ceremonies (docs/ACCOUNTS.md › Passkeys › Where the
//! ceremony runs).
//!
//! A platform passkey is created and used by the operating system:
//! AuthenticationServices on macOS 13 and later, the Win32 WebAuthn API
//! (`webauthn.h`) on Windows 10 and 11. There is no web page and no network
//! here: the server's start options become a platform request, and the
//! platform's answer becomes the same [`kanso_sync::PasskeyRegistration`] or
//! [`kanso_sync::PasskeyAssertion`] a Linux security key produces.
//!
//! PRF is requested only where the platform has it: macOS 15 (checked at
//! runtime), and on Windows when the WebAuthn API is V8 or newer, through
//! registration's `pPRFGlobalEval` and `pHmacSecret`; assertions use
//! `pHmacSecretSaltValues`. A credential registered without an output signs in
//! but cannot unlock a new device; the account flow says so.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// A registration's result, in the same WebAuthn shape `kanso-sync`'s
/// `PasskeyRegistration` carries. The app converts it there.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyRegistration {
    pub credential_id: Vec<u8>,
    pub attestation_object: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub transports: Vec<String>,
    pub prf: Option<[u8; 32]>,
}

/// An assertion's result, in the same WebAuthn shape `kanso-sync`'s
/// `PasskeyAssertion` carries.
#[derive(Clone, PartialEq, Eq)]
pub struct PasskeyAssertion {
    pub credential_id: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
    pub prf: Option<[u8; 32]>,
}

impl std::fmt::Debug for PasskeyRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasskeyRegistration")
            .field(
                "credential_id",
                &URL_SAFE_NO_PAD.encode(&self.credential_id),
            )
            .field("transports", &self.transports)
            .field("prf", &self.prf.map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for PasskeyAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasskeyAssertion")
            .field(
                "credential_id",
                &URL_SAFE_NO_PAD.encode(&self.credential_id),
            )
            .field("prf", &self.prf.map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

/// The fixed PRF input, duplicated here so the platform crate stays free of
/// the sync client's storage dependencies. A test keeps the two equal.
pub const PRF_INPUT: [u8; 32] = [
    0x98, 0x43, 0x20, 0xbf, 0xab, 0xbd, 0x34, 0x26, 0x97, 0xaf, 0x83, 0x64, 0x7d, 0xc7, 0x74, 0xe3,
    0x07, 0xf9, 0x8e, 0x62, 0xec, 0xf2, 0x7d, 0xd5, 0xd8, 0x4f, 0x8f, 0x00, 0x6d, 0x86, 0xfd, 0x59,
];

#[cfg(any(target_os = "windows", test))]
fn prf_output(bytes: &[u8]) -> Option<[u8; 32]> {
    bytes.try_into().ok()
}

/// The client data the Windows ceremony sends. Apple's platform builds its
/// own; the Worker checks the same fields in both.
pub fn client_data_json(kind: &str, challenge: &[u8], origin: &str) -> Vec<u8> {
    format!(
        r#"{{"type":"{}","challenge":"{}","origin":"{}","crossOrigin":false}}"#,
        json_escape(kind),
        URL_SAFE_NO_PAD.encode(challenge),
        json_escape(origin)
    )
    .into_bytes()
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// What a registration ceremony needs. The starter comes from the server's
/// `passkey/signup/start` answer.
#[derive(Clone, Debug)]
pub struct RegistrationRequest {
    /// The relying party's host, exactly `PASSKEY_RP_ID`.
    pub rp_id: String,
    /// The account's origin, for platforms that build `clientDataJSON` here.
    pub origin: String,
    /// 32 random bytes, as the start answered.
    pub challenge: Vec<u8>,
    /// `SHA-256("kanso/v1/webauthn-user" || lp(account_id))[0..16]`.
    pub user_handle: Vec<u8>,
    pub username: String,
    pub display_name: String,
    pub exclude_credentials: Vec<Vec<u8>>,
    /// The fixed Kanso PRF input, when this ceremony asks for PRF.
    pub prf_input: Option<[u8; 32]>,
    /// Windows: the top-level window (`HWND`) the sheet belongs to, or 0.
    /// macOS anchors the sheet on the key window itself.
    pub window: isize,
}

/// What an assertion ceremony needs.
#[derive(Clone, Debug)]
pub struct AssertionRequest {
    pub rp_id: String,
    pub origin: String,
    pub challenge: Vec<u8>,
    /// Empty asks for a discoverable credential (the platform picker).
    pub allow_credentials: Vec<Vec<u8>>,
    pub prf_input: Option<[u8; 32]>,
    pub user_verification: UserVerification,
    /// Windows: the top-level window (`HWND`) the sheet belongs to, or 0.
    pub window: isize,
}

/// User verification, which every Kanso ceremony requires.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UserVerification {
    #[default]
    Required,
    Preferred,
    Discouraged,
}

impl UserVerification {
    /// The WebAuthn spelling a platform API expects.
    pub fn as_str(self) -> &'static str {
        match self {
            UserVerification::Required => "required",
            UserVerification::Preferred => "preferred",
            UserVerification::Discouraged => "discouraged",
        }
    }
}

/// Why a ceremony didn't finish.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CeremonyError {
    /// This platform has no native ceremony (Linux, or macOS older than 13).
    #[error("this platform has no native passkey ceremony")]
    Unsupported,
    /// A ceremony must be started from the main thread, where the system's
    /// sheet can be presented.
    #[error("a passkey ceremony must run on the main thread")]
    NotMainThread,
    /// The system refused, the person cancelled, or there is no associated
    /// domain for this RP ID.
    #[error("the passkey ceremony failed: {0}")]
    Failed(String),
    /// The platform returned an answer without the field a ceremony needs.
    #[error("the passkey ceremony returned an incomplete answer")]
    Incomplete,
    /// The request was not usable (an empty RP ID, or the like).
    #[error("the passkey request is not valid")]
    Invalid,
    /// The person dismissed the sheet.
    #[error("the passkey sheet was dismissed")]
    Cancelled,
    /// The sheet stayed up past [`TIMEOUT`]; the caller cancels it.
    #[error("the passkey ceremony timed out")]
    TimedOut,
}

/// How long a sheet may stay up before the ceremony is given up and the
/// sheet cancelled.
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Whether this build can run a platform passkey ceremony now.
pub fn available() -> bool {
    platform::available()
}

fn check_registration(request: &RegistrationRequest) -> Result<(), CeremonyError> {
    if request.rp_id.is_empty() || request.challenge.is_empty() || request.user_handle.is_empty() {
        return Err(CeremonyError::Invalid);
    }
    Ok(())
}

fn check_assertion(request: &AssertionRequest) -> Result<(), CeremonyError> {
    if request.rp_id.is_empty() || request.challenge.is_empty() {
        return Err(CeremonyError::Invalid);
    }
    Ok(())
}

/// Runs a registration ceremony on the calling thread, which waits while
/// the sheet is up: an account task's thread, never the UI's (Windows, with
/// `request.window` as the sheet's owner; nothing elsewhere).
#[cfg(not(target_os = "macos"))]
pub fn register(request: &RegistrationRequest) -> Result<PasskeyRegistration, CeremonyError> {
    check_registration(request)?;
    platform::register(request)
}

/// Runs an assertion ceremony on the calling thread (see [`register`]).
#[cfg(not(target_os = "macos"))]
pub fn assert(request: &AssertionRequest) -> Result<PasskeyAssertion, CeremonyError> {
    check_assertion(request)?;
    platform::assert(request)
}

/// Starts a registration on macOS and returns at once: the system answers
/// later, on the main run loop the app keeps running, and the answer
/// arrives through the returned [`Pending`]. Must be called on the main
/// thread. Nothing here spins a nested run loop, so the app's main-queue
/// work, and the sheet's own callbacks, keep flowing.
#[cfg(target_os = "macos")]
pub fn begin_register(request: &RegistrationRequest) -> Result<Pending, CeremonyError> {
    check_registration(request)?;
    platform::begin_register(request)
}

/// Starts an assertion on macOS (see [`begin_register`]).
#[cfg(target_os = "macos")]
pub fn begin_assert(request: &AssertionRequest) -> Result<Pending, CeremonyError> {
    check_assertion(request)?;
    platform::begin_assert(request)
}

#[cfg(target_os = "macos")]
pub use platform::{Pending, cancel};

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, mpsc};
    use std::time::Duration;

    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject, Bool, NSObject};
    use objc2::{
        DefinedClass as _, MainThreadMarker, MainThreadOnly, available, class, define_class,
        msg_send, sel,
    };
    // The classes are looked up by name at run time, so nothing else makes
    // the linker load the framework: link it here, not only from the app.
    #[link(name = "AuthenticationServices", kind = "framework")]
    unsafe extern "C" {}

    use objc2_app_kit::NSApplication;
    use objc2_foundation::{NSArray, NSData, NSError, NSString};

    /// `ASAuthorizationErrorDomain` and its `ASAuthorizationErrorCanceled`.
    const AUTHORIZATION_ERROR_DOMAIN: &str = "com.apple.AuthenticationServices.AuthorizationError";
    const AUTHORIZATION_CANCELED: isize = 1001;

    type Answer = Result<Raw, CeremonyError>;

    enum Raw {
        Registration {
            credential_id: Vec<u8>,
            attestation_object: Vec<u8>,
            client_data_json: Vec<u8>,
            prf: Option<[u8; 32]>,
        },
        Assertion {
            credential_id: Vec<u8>,
            authenticator_data: Vec<u8>,
            client_data_json: Vec<u8>,
            signature: Vec<u8>,
            user_handle: Option<Vec<u8>>,
            prf: Option<[u8; 32]>,
        },
    }

    define_class!(
        /// Receives AuthenticationServices' answer on the main thread.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "KansoPasskeyDelegate"]
        #[ivars = Mutex<Option<mpsc::Sender<Answer>>>]
        struct Delegate;

        impl Delegate {
            #[unsafe(method(authorizationController:didCompleteWithAuthorization:))]
            fn did_complete(&self, _controller: &AnyObject, authorization: &AnyObject) {
                // SAFETY: AuthenticationServices calls this on the main thread
                // with a live authorization object.
                self.finish(unsafe { interpret(authorization) }.map_err(CeremonyError::Failed));
            }

            #[unsafe(method(authorizationController:didCompleteWithError:))]
            fn did_fail(&self, _controller: &AnyObject, error: &NSError) {
                let cancelled = error.domain().to_string() == AUTHORIZATION_ERROR_DOMAIN
                    && error.code() == AUTHORIZATION_CANCELED;
                self.finish(Err(if cancelled {
                    CeremonyError::Cancelled
                } else {
                    CeremonyError::Failed(error.localizedDescription().to_string())
                }));
            }

            #[unsafe(method(presentationAnchorForAuthorizationController:))]
            fn anchor(&self, _controller: &AnyObject) -> *mut AnyObject {
                let Some(mtm) = MainThreadMarker::new() else {
                    return std::ptr::null_mut();
                };
                let app = NSApplication::sharedApplication(mtm);
                app.keyWindow()
                    .or_else(|| app.mainWindow())
                    .map(|window| Retained::as_ptr(&window) as *mut AnyObject)
                    .unwrap_or(std::ptr::null_mut())
            }
        }
    );

    /// Reads what AuthenticationServices returned. A registration credential
    /// answers `rawAttestationObject`; an assertion credential does not.
    ///
    /// # Safety
    /// `authorization` is a live `ASAuthorization` from the system.
    unsafe fn interpret(authorization: &AnyObject) -> Result<Raw, String> {
        let authorization = authorization as *const AnyObject as *mut AnyObject;
        unsafe {
            let credential: *mut AnyObject = msg_send![authorization, credential];
            if credential.is_null() {
                return Err("no credential".to_owned());
            }
            let is_registration: Bool =
                msg_send![credential, respondsToSelector: sel!(rawAttestationObject)];
            if is_registration.as_bool() {
                let id: *mut NSData = msg_send![credential, credentialID];
                let attestation: *mut NSData = msg_send![credential, rawAttestationObject];
                let client_data: *mut NSData = msg_send![credential, rawClientDataJSON];
                Ok(Raw::Registration {
                    credential_id: data(id)?,
                    attestation_object: data(attestation)?,
                    client_data_json: data(client_data)?,
                    prf: prf_from(credential)?,
                })
            } else {
                let id: *mut NSData = msg_send![credential, credentialID];
                let auth_data: *mut NSData = msg_send![credential, rawAuthenticatorData];
                let client_data: *mut NSData = msg_send![credential, rawClientDataJSON];
                let signature: *mut NSData = msg_send![credential, signature];
                let user: *mut NSData = msg_send![credential, userID];
                Ok(Raw::Assertion {
                    credential_id: data(id)?,
                    authenticator_data: data(auth_data)?,
                    client_data_json: data(client_data)?,
                    signature: data(signature)?,
                    user_handle: if user.is_null() {
                        None
                    } else {
                        Some(data(user)?)
                    },
                    prf: prf_from(credential)?,
                })
            }
        }
    }

    impl Delegate {
        /// Sends the one answer; later calls (a cancel after an answer)
        /// find no sender and do nothing.
        fn finish(&self, outcome: Answer) {
            let sender = self.ivars().lock().ok().and_then(|mut slot| slot.take());
            if let Some(sender) = sender {
                let _ = sender.send(outcome);
            }
        }

        fn finished(&self) -> bool {
            self.ivars()
                .lock()
                .map(|slot| slot.is_none())
                .unwrap_or(true)
        }
    }

    /// A ceremony the system is running: the controller (which holds its
    /// delegate weakly) and the delegate, kept alive here until the delegate
    /// has answered, as `provider_browser::apple::LIVE` keeps its sessions.
    struct Live {
        id: u64,
        controller: Retained<AnyObject>,
        delegate: Retained<Delegate>,
    }

    thread_local! {
        static LIVE: RefCell<Vec<Live>> = const { RefCell::new(Vec::new()) };
    }

    static NEXT: AtomicU64 = AtomicU64::new(1);

    /// Drops the ceremonies whose delegate has answered. Never from inside a
    /// delegate callback, so no object is released while it runs.
    fn sweep() {
        LIVE.with(|live| live.borrow_mut().retain(|entry| !entry.delegate.finished()));
    }

    /// A started ceremony's answer, which any thread may wait for.
    pub struct Pending {
        id: u64,
        answer: mpsc::Receiver<Answer>,
    }

    impl Pending {
        /// What [`cancel`] takes, on the main thread.
        pub fn id(&self) -> u64 {
            self.id
        }

        fn wait(&self, timeout: Duration) -> Answer {
            match self.answer.recv_timeout(timeout) {
                Ok(answer) => answer,
                Err(mpsc::RecvTimeoutError::Timeout) => Err(CeremonyError::TimedOut),
                Err(mpsc::RecvTimeoutError::Disconnected) => Err(CeremonyError::Failed(
                    "the ceremony ended without an answer".to_owned(),
                )),
            }
        }

        /// Waits (off the main thread) for a registration's answer.
        pub fn wait_registration(
            &self,
            timeout: Duration,
        ) -> Result<PasskeyRegistration, CeremonyError> {
            match self.wait(timeout)? {
                Raw::Registration {
                    credential_id,
                    attestation_object,
                    client_data_json,
                    prf,
                } => Ok(PasskeyRegistration {
                    credential_id,
                    attestation_object,
                    client_data_json,
                    transports: vec!["internal".to_owned()],
                    prf,
                }),
                Raw::Assertion { .. } => Err(CeremonyError::Incomplete),
            }
        }

        /// Waits (off the main thread) for an assertion's answer.
        pub fn wait_assertion(&self, timeout: Duration) -> Result<PasskeyAssertion, CeremonyError> {
            match self.wait(timeout)? {
                Raw::Assertion {
                    credential_id,
                    authenticator_data,
                    client_data_json,
                    signature,
                    user_handle,
                    prf,
                } => Ok(PasskeyAssertion {
                    credential_id,
                    authenticator_data,
                    client_data_json,
                    signature,
                    user_handle,
                    prf,
                }),
                Raw::Registration { .. } => Err(CeremonyError::Incomplete),
            }
        }
    }

    /// Cancels ceremony `id` if it is still up (`[controller cancel]`, macOS
    /// 13+). The delegate then answers [`CeremonyError::Cancelled`] to no one.
    /// Main thread only; elsewhere, and for an unknown ID, nothing happens.
    pub fn cancel(id: u64) {
        if MainThreadMarker::new().is_none() {
            return;
        }
        let controller = LIVE.with(|live| {
            live.borrow()
                .iter()
                .find(|entry| entry.id == id && !entry.delegate.finished())
                .map(|entry| entry.controller.clone())
        });
        if let Some(controller) = controller {
            // SAFETY: a live controller, on the main thread.
            unsafe {
                if msg_send![&*controller, respondsToSelector: sel!(cancel)] {
                    let _: () = msg_send![&*controller, cancel];
                }
            }
        }
        sweep();
    }

    /// Hands `controller` to the system and returns at once.
    fn start(
        controller: Retained<AnyObject>,
        delegate: Retained<Delegate>,
        answer: mpsc::Receiver<Answer>,
    ) -> Pending {
        sweep();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        LIVE.with(|live| {
            live.borrow_mut().push(Live {
                id,
                controller: controller.clone(),
                delegate,
            })
        });
        // SAFETY: on the main thread with a live controller; the answer comes
        // through the delegate, kept in LIVE.
        unsafe {
            let _: () = msg_send![&*controller, performRequests];
        }
        Pending { id, answer }
    }

    /// # Safety
    /// `object` is a non-null `NSData` returned by AuthenticationServices.
    unsafe fn data(object: *mut NSData) -> Result<Vec<u8>, String> {
        if object.is_null() {
            return Err("missing data".to_owned());
        }
        Ok(unsafe { (*object).to_vec() })
    }

    /// The PRF output, when the platform exposed one. Only macOS 15 has the
    /// `prf` property; on 13 and 14 the credential registers without a wrap.
    /// A registration's `prf` is an
    /// `ASAuthorizationPublicKeyCredentialPRFRegistrationOutput` (`isSupported`,
    /// `first`, `second`); an assertion's is an
    /// `ASAuthorizationPublicKeyCredentialPRFAssertionOutput` (`first`,
    /// `second`). Only `first` is used: Kanso sends one salt.
    unsafe fn prf_from(credential: *mut AnyObject) -> Result<Option<[u8; 32]>, String> {
        if !available!(macos = 15.0) {
            return Ok(None);
        }
        // SAFETY: the caller holds a credential object; every selector is
        // checked before it is sent.
        unsafe {
            if !msg_send![credential, respondsToSelector: sel!(prf)] {
                return Ok(None);
            }
            let prf: *mut AnyObject = msg_send![credential, prf];
            if prf.is_null() {
                return Ok(None);
            }
            let responds_supported: bool = msg_send![prf, respondsToSelector: sel!(isSupported)];
            if responds_supported {
                let supported: Bool = msg_send![prf, isSupported];
                if !supported.as_bool() {
                    return Ok(None);
                }
            }
            if !msg_send![prf, respondsToSelector: sel!(first)] {
                return Ok(None);
            }
            let first: *mut NSData = msg_send![prf, first];
            if first.is_null() {
                return Ok(None);
            }
            let bytes = (*first).to_vec();
            let out: [u8; 32] = bytes
                .try_into()
                .map_err(|_| "a PRF output wasn't 32 bytes".to_owned())?;
            Ok(Some(out))
        }
    }

    /// `ASAuthorizationPublicKeyCredentialPRFAssertionInputValues` with `input`
    /// as `saltInput1` and no second salt, or `None` when the macOS 15 class
    /// or its initializer is missing. Registration and assertion inputs both
    /// take this class.
    unsafe fn prf_values(input: &[u8; 32]) -> Option<Retained<AnyObject>> {
        let cls = AnyClass::get(c"ASAuthorizationPublicKeyCredentialPRFAssertionInputValues")?;
        cls.instance_method(sel!(initWithSaltInput1:saltInput2:))?;
        let salt = NSData::with_bytes(input);
        // SAFETY: alloc then the class's own initializer, checked above; init
        // returns an owned (+1) object, which `from_raw` takes over.
        unsafe {
            let values: *mut AnyObject = msg_send![cls, alloc];
            let values: *mut AnyObject = msg_send![
                values,
                initWithSaltInput1: &*salt,
                saltInput2: std::ptr::null_mut::<NSData>()
            ];
            Retained::from_raw(values)
        }
    }

    /// Requests PRF on a registration through
    /// `ASAuthorizationPublicKeyCredentialPRFRegistrationInput
    /// initWithInputValues:`. Any missing class or selector leaves PRF
    /// unrequested, never crashes: the credential then registers without a wrap.
    unsafe fn set_registration_prf(request: *mut AnyObject, input: &[u8; 32]) {
        let Some(cls) = AnyClass::get(c"ASAuthorizationPublicKeyCredentialPRFRegistrationInput")
        else {
            return;
        };
        if cls.instance_method(sel!(initWithInputValues:)).is_none() {
            return;
        }
        // SAFETY: guarded by the class and selector checks; `setPrf:` retains
        // what it keeps, and the owned objects here are released on return.
        unsafe {
            if !msg_send![request, respondsToSelector: sel!(setPrf:)] {
                return;
            }
            let Some(values) = prf_values(input) else {
                return;
            };
            let prf: *mut AnyObject = msg_send![cls, alloc];
            let prf: *mut AnyObject = msg_send![prf, initWithInputValues: &*values];
            if let Some(prf) = Retained::from_raw(prf) {
                let _: () = msg_send![request, setPrf: &*prf];
            }
        }
    }

    /// Requests PRF on an assertion through
    /// `ASAuthorizationPublicKeyCredentialPRFAssertionInput
    /// initWithInputValues:perCredentialInputValues:`, with one salt for every
    /// credential.
    unsafe fn set_assertion_prf(request: *mut AnyObject, input: &[u8; 32]) {
        let Some(cls) = AnyClass::get(c"ASAuthorizationPublicKeyCredentialPRFAssertionInput")
        else {
            return;
        };
        if cls
            .instance_method(sel!(initWithInputValues:perCredentialInputValues:))
            .is_none()
        {
            return;
        }
        // SAFETY: as for `set_registration_prf`.
        unsafe {
            if !msg_send![request, respondsToSelector: sel!(setPrf:)] {
                return;
            }
            let Some(values) = prf_values(input) else {
                return;
            };
            let prf: *mut AnyObject = msg_send![cls, alloc];
            let prf: *mut AnyObject = msg_send![
                prf,
                initWithInputValues: &*values,
                perCredentialInputValues: std::ptr::null_mut::<AnyObject>()
            ];
            if let Some(prf) = Retained::from_raw(prf) {
                let _: () = msg_send![request, setPrf: &*prf];
            }
        }
    }

    fn strings(request: &RegistrationRequest) -> (Retained<NSString>, Retained<NSString>) {
        (
            NSString::from_str(&request.username),
            NSString::from_str(&request.display_name),
        )
    }

    pub(super) fn available() -> bool {
        MainThreadMarker::new().is_some() && available!(macos = 13.0)
    }

    pub(super) fn begin_register(request: &RegistrationRequest) -> Result<Pending, CeremonyError> {
        let mtm = MainThreadMarker::new().ok_or(CeremonyError::NotMainThread)?;
        if !available!(macos = 13.0) {
            return Err(CeremonyError::Unsupported);
        }
        let (tx, answer) = mpsc::channel();
        let delegate = Delegate::alloc(mtm).set_ivars(Mutex::new(Some(tx)));
        // SAFETY: NSObject's init on a fresh allocation.
        let delegate: Retained<Delegate> = unsafe { msg_send![super(delegate), init] };
        let rp_id = NSString::from_str(&request.rp_id);
        let (name, _display_name) = strings(request);
        let challenge = NSData::with_bytes(&request.challenge);
        let handle = NSData::with_bytes(&request.user_handle);

        // SAFETY: the platform classes exist on macOS 13; every object is
        // retained for the call.
        let controller: Retained<AnyObject> = unsafe {
            let provider: *mut AnyObject = msg_send![
                class!(ASAuthorizationPlatformPublicKeyCredentialProvider),
                alloc
            ];
            let provider: *mut AnyObject =
                msg_send![provider, initWithRelyingPartyIdentifier: &*rp_id];
            // init returns an owned object; it is released when this block ends.
            let provider = Retained::from_raw(provider).ok_or(CeremonyError::Unsupported)?;
            let request_object: *mut AnyObject = msg_send![
                &*provider,
                createCredentialRegistrationRequestWithChallenge: &*challenge,
                name: &*name,
                userID: &*handle
            ];
            if request_object.is_null() {
                return Err(CeremonyError::Failed(
                    "the platform made no registration request".to_owned(),
                ));
            }
            let uv = NSString::from_str(UserVerification::Required.as_str());
            let attestation = NSString::from_str("none");
            let _: () = msg_send![request_object, setUserVerificationPreference: &*uv];
            let _: () = msg_send![request_object, setAttestationPreference: &*attestation];
            if !request.exclude_credentials.is_empty() {
                let mut descriptors: Vec<Retained<AnyObject>> = Vec::new();
                for credential in &request.exclude_credentials {
                    let id = NSData::with_bytes(credential);
                    let descriptor: *mut AnyObject = msg_send![
                        class!(ASAuthorizationPlatformPublicKeyCredentialDescriptor),
                        alloc
                    ];
                    let descriptor: *mut AnyObject =
                        msg_send![descriptor, initWithCredentialID: &*id];
                    // Owned by init: take it over, don't retain it again.
                    if let Some(descriptor) = Retained::from_raw(descriptor) {
                        descriptors.push(descriptor);
                    }
                }
                let refs: Vec<&AnyObject> = descriptors.iter().map(|d| &**d).collect();
                let excluded = NSArray::from_slice(&refs);
                let _: () = msg_send![request_object, setExcludedCredentials: &*excluded];
            }
            if let Some(prf) = &request.prf_input {
                set_registration_prf(request_object, prf);
            }
            let requests = NSArray::from_slice(&[&*request_object]);
            let controller: *mut AnyObject = msg_send![class!(ASAuthorizationController), alloc];
            let controller: *mut AnyObject =
                msg_send![controller, initWithAuthorizationRequests: &*requests];
            let controller = Retained::from_raw(controller).ok_or(CeremonyError::Incomplete)?;
            let _: () = msg_send![&*controller, setDelegate: &*delegate];
            let _: () = msg_send![&*controller, setPresentationContextProvider: &*delegate];
            controller
        };
        Ok(start(controller, delegate, answer))
    }

    pub(super) fn begin_assert(request: &AssertionRequest) -> Result<Pending, CeremonyError> {
        let mtm = MainThreadMarker::new().ok_or(CeremonyError::NotMainThread)?;
        if !available!(macos = 13.0) {
            return Err(CeremonyError::Unsupported);
        }
        let (tx, answer) = mpsc::channel();
        let delegate = Delegate::alloc(mtm).set_ivars(Mutex::new(Some(tx)));
        // SAFETY: NSObject's init on a fresh allocation.
        let delegate: Retained<Delegate> = unsafe { msg_send![super(delegate), init] };
        let rp_id = NSString::from_str(&request.rp_id);
        let challenge = NSData::with_bytes(&request.challenge);

        // SAFETY: the platform classes exist on macOS 13; every object is
        // retained for the call.
        let controller: Retained<AnyObject> = unsafe {
            let provider: *mut AnyObject = msg_send![
                class!(ASAuthorizationPlatformPublicKeyCredentialProvider),
                alloc
            ];
            let provider: *mut AnyObject =
                msg_send![provider, initWithRelyingPartyIdentifier: &*rp_id];
            // init returns an owned object; it is released when this block ends.
            let provider = Retained::from_raw(provider).ok_or(CeremonyError::Unsupported)?;
            let request_object: *mut AnyObject =
                msg_send![&*provider, createCredentialAssertionRequestWithChallenge: &*challenge];
            if request_object.is_null() {
                return Err(CeremonyError::Failed(
                    "the platform made no assertion request".to_owned(),
                ));
            }
            let uv = NSString::from_str(request.user_verification.as_str());
            let _: () = msg_send![request_object, setUserVerificationPreference: &*uv];
            if !request.allow_credentials.is_empty() {
                let mut descriptors: Vec<Retained<AnyObject>> = Vec::new();
                for credential in &request.allow_credentials {
                    let id = NSData::with_bytes(credential);
                    let descriptor: *mut AnyObject = msg_send![
                        class!(ASAuthorizationPlatformPublicKeyCredentialDescriptor),
                        alloc
                    ];
                    let descriptor: *mut AnyObject =
                        msg_send![descriptor, initWithCredentialID: &*id];
                    // Owned by init: take it over, don't retain it again.
                    if let Some(descriptor) = Retained::from_raw(descriptor) {
                        descriptors.push(descriptor);
                    }
                }
                let refs: Vec<&AnyObject> = descriptors.iter().map(|d| &**d).collect();
                let allowed = NSArray::from_slice(&refs);
                let _: () = msg_send![request_object, setAllowedCredentials: &*allowed];
            }
            if let Some(prf) = &request.prf_input {
                set_assertion_prf(request_object, prf);
            }
            let requests = NSArray::from_slice(&[&*request_object]);
            let controller: *mut AnyObject = msg_send![class!(ASAuthorizationController), alloc];
            let controller: *mut AnyObject =
                msg_send![controller, initWithAuthorizationRequests: &*requests];
            let controller = Retained::from_raw(controller).ok_or(CeremonyError::Incomplete)?;
            let _: () = msg_send![&*controller, setDelegate: &*delegate];
            let _: () = msg_send![&*controller, setPresentationContextProvider: &*delegate];
            controller
        };
        Ok(start(controller, delegate, answer))
    }

    #[cfg(test)]
    mod prf_tests {
        use super::*;

        fn has(class: &std::ffi::CStr, selector: objc2::runtime::Sel) -> bool {
            AnyClass::get(class).is_some_and(|cls| cls.instance_method(selector).is_some())
        }

        /// Every PRF class and selector this module sends exists in the
        /// running AuthenticationServices, so PRF is really requested and read
        /// on macOS 15 rather than silently skipped.
        #[test]
        fn the_prf_classes_and_selectors_exist_on_macos_15() {
            if !available!(macos = 15.0) {
                return;
            }
            let values = c"ASAuthorizationPublicKeyCredentialPRFAssertionInputValues";
            assert!(has(values, sel!(initWithSaltInput1:saltInput2:)));
            assert!(has(values, sel!(saltInput1)));
            assert!(has(
                c"ASAuthorizationPublicKeyCredentialPRFRegistrationInput",
                sel!(initWithInputValues:)
            ));
            assert!(has(
                c"ASAuthorizationPublicKeyCredentialPRFAssertionInput",
                sel!(initWithInputValues:perCredentialInputValues:)
            ));
            let registration = c"ASAuthorizationPublicKeyCredentialPRFRegistrationOutput";
            assert!(has(registration, sel!(isSupported)));
            assert!(has(registration, sel!(first)));
            assert!(has(
                c"ASAuthorizationPublicKeyCredentialPRFAssertionOutput",
                sel!(first)
            ));
            for request in [
                c"ASAuthorizationPlatformPublicKeyCredentialRegistrationRequest",
                c"ASAuthorizationPlatformPublicKeyCredentialAssertionRequest",
            ] {
                assert!(has(request, sel!(setPrf:)), "{request:?}");
            }
        }

        /// The salt goes in as `saltInput1`, unchanged, with no second salt.
        #[test]
        fn prf_values_carry_the_salt() {
            if !available!(macos = 15.0) {
                return;
            }
            let input = [7u8; 32];
            // SAFETY: the class exists on macOS 15 (checked above).
            let values = unsafe { prf_values(&input) }.expect("the values object");
            let (first, second): (*mut NSData, *mut NSData) = unsafe {
                (
                    msg_send![&*values, saltInput1],
                    msg_send![&*values, saltInput2],
                )
            };
            assert!(!first.is_null());
            assert_eq!(unsafe { (*first).to_vec() }, input.to_vec());
            assert!(second.is_null());
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::c_void;

    use windows::Win32::Foundation::HWND;
    use windows::Win32::Networking::WindowsWebServices::{
        WEBAUTHN_ASSERTION, WEBAUTHN_ASSERTION_VERSION_3,
        WEBAUTHN_ATTESTATION_CONVEYANCE_PREFERENCE_NONE, WEBAUTHN_AUTHENTICATOR_ATTACHMENT_ANY,
        WEBAUTHN_AUTHENTICATOR_GET_ASSERTION_OPTIONS,
        WEBAUTHN_AUTHENTICATOR_GET_ASSERTION_OPTIONS_CURRENT_VERSION,
        WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS,
        WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS_CURRENT_VERSION, WEBAUTHN_CLIENT_DATA,
        WEBAUTHN_CLIENT_DATA_CURRENT_VERSION, WEBAUTHN_COSE_CREDENTIAL_PARAMETER,
        WEBAUTHN_COSE_CREDENTIAL_PARAMETER_CURRENT_VERSION, WEBAUTHN_COSE_CREDENTIAL_PARAMETERS,
        WEBAUTHN_CREDENTIAL, WEBAUTHN_CREDENTIAL_EX, WEBAUTHN_CREDENTIAL_EX_CURRENT_VERSION,
        WEBAUTHN_CREDENTIAL_LIST, WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY, WEBAUTHN_CTAP_TRANSPORT_BLE,
        WEBAUTHN_CTAP_TRANSPORT_HYBRID, WEBAUTHN_CTAP_TRANSPORT_INTERNAL,
        WEBAUTHN_CTAP_TRANSPORT_NFC, WEBAUTHN_CTAP_TRANSPORT_USB, WEBAUTHN_HASH_ALGORITHM_SHA_256,
        WEBAUTHN_HMAC_SECRET_SALT, WEBAUTHN_HMAC_SECRET_SALT_VALUES,
        WEBAUTHN_RP_ENTITY_INFORMATION, WEBAUTHN_RP_ENTITY_INFORMATION_CURRENT_VERSION,
        WEBAUTHN_USER_ENTITY_INFORMATION, WEBAUTHN_USER_ENTITY_INFORMATION_CURRENT_VERSION,
        WEBAUTHN_USER_VERIFICATION_REQUIRED, WebAuthNAuthenticatorGetAssertion,
        WebAuthNAuthenticatorMakeCredential, WebAuthNFreeAssertion,
        WebAuthNFreeCredentialAttestation, WebAuthNGetApiVersionNumber,
    };
    use windows::core::{BOOL, PCWSTR, w};

    /// Windows SDK WebAuthn V8 appends PRF registration input to V7 options.
    /// Keep the binding type as the first field so the API receives its exact
    /// existing layout with the extension immediately following it.
    #[repr(C)]
    #[allow(non_snake_case)]
    struct MakeCredentialOptionsV8 {
        base: WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS,
        pPRFGlobalEval: *mut WEBAUTHN_HMAC_SECRET_SALT,
        cCredentialHints: u32,
        ppwszCredentialHints: *mut PCWSTR,
        bThirdPartyPayment: BOOL,
    }

    /// Windows SDK WebAuthn V7 appends the registration PRF output to V6
    /// attestation. The pointer is read only when the returned version allows it.
    #[repr(C)]
    #[allow(non_snake_case)]
    struct CredentialAttestationV7 {
        base: windows::Win32::Networking::WindowsWebServices::WEBAUTHN_CREDENTIAL_ATTESTATION,
        pHmacSecret: *mut WEBAUTHN_HMAC_SECRET_SALT,
        bThirdPartyPayment: BOOL,
    }

    const _: () = {
        assert!(
            std::mem::offset_of!(MakeCredentialOptionsV8, pPRFGlobalEval)
                == std::mem::size_of::<WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS>()
        );
        assert!(
            std::mem::offset_of!(CredentialAttestationV7, pHmacSecret)
                == std::mem::size_of::<
                    windows::Win32::Networking::WindowsWebServices::WEBAUTHN_CREDENTIAL_ATTESTATION,
                >()
        );
    };

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn pcwstr(text: &[u16]) -> PCWSTR {
        PCWSTR(text.as_ptr())
    }

    fn bytes(ptr: *mut u8, len: u32) -> Vec<u8> {
        if ptr.is_null() || len == 0 {
            Vec::new()
        } else {
            // SAFETY: the platform owns `len` bytes at `ptr` until the free call.
            unsafe { std::slice::from_raw_parts(ptr, len as usize).to_vec() }
        }
    }

    fn transports(mask: u32) -> Vec<String> {
        let mut out = Vec::new();
        if mask & WEBAUTHN_CTAP_TRANSPORT_USB != 0 {
            out.push("usb".to_owned());
        }
        if mask & WEBAUTHN_CTAP_TRANSPORT_NFC != 0 {
            out.push("nfc".to_owned());
        }
        if mask & WEBAUTHN_CTAP_TRANSPORT_BLE != 0 {
            out.push("ble".to_owned());
        }
        if mask & WEBAUTHN_CTAP_TRANSPORT_HYBRID != 0 {
            out.push("hybrid".to_owned());
        }
        if mask & WEBAUTHN_CTAP_TRANSPORT_INTERNAL != 0 {
            out.push("internal".to_owned());
        }
        out
    }

    pub(super) fn available() -> bool {
        true
    }

    pub(super) fn register(
        request: &RegistrationRequest,
    ) -> Result<PasskeyRegistration, CeremonyError> {
        let rp_id = wide(&request.rp_id);
        let username = wide(&request.username);
        let display = wide(&request.display_name);
        let rp = WEBAUTHN_RP_ENTITY_INFORMATION {
            dwVersion: WEBAUTHN_RP_ENTITY_INFORMATION_CURRENT_VERSION,
            pwszId: pcwstr(&rp_id),
            pwszName: pcwstr(&rp_id),
            pwszIcon: PCWSTR::null(),
        };
        let mut user_id = request.user_handle.clone();
        let user = WEBAUTHN_USER_ENTITY_INFORMATION {
            dwVersion: WEBAUTHN_USER_ENTITY_INFORMATION_CURRENT_VERSION,
            cbId: user_id.len() as u32,
            pbId: user_id.as_mut_ptr(),
            pwszName: pcwstr(&username),
            pwszIcon: PCWSTR::null(),
            pwszDisplayName: pcwstr(&display),
        };
        let mut algorithms = vec![
            WEBAUTHN_COSE_CREDENTIAL_PARAMETER {
                dwVersion: WEBAUTHN_COSE_CREDENTIAL_PARAMETER_CURRENT_VERSION,
                pwszCredentialType: WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY,
                lAlg: -7,
            },
            WEBAUTHN_COSE_CREDENTIAL_PARAMETER {
                dwVersion: WEBAUTHN_COSE_CREDENTIAL_PARAMETER_CURRENT_VERSION,
                pwszCredentialType: WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY,
                lAlg: -8,
            },
            WEBAUTHN_COSE_CREDENTIAL_PARAMETER {
                dwVersion: WEBAUTHN_COSE_CREDENTIAL_PARAMETER_CURRENT_VERSION,
                pwszCredentialType: WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY,
                lAlg: -257,
            },
        ];
        let params = WEBAUTHN_COSE_CREDENTIAL_PARAMETERS {
            cCredentialParameters: algorithms.len() as u32,
            pCredentialParameters: algorithms.as_mut_ptr(),
        };
        let client_data = client_data_json("webauthn.create", &request.challenge, &request.origin);
        let client = WEBAUTHN_CLIENT_DATA {
            dwVersion: WEBAUTHN_CLIENT_DATA_CURRENT_VERSION,
            cbClientDataJSON: client_data.len() as u32,
            pbClientDataJSON: client_data.as_ptr() as *mut u8,
            pwszHashAlgId: WEBAUTHN_HASH_ALGORITHM_SHA_256,
        };
        let mut excluded: Vec<WEBAUTHN_CREDENTIAL_EX> = request
            .exclude_credentials
            .iter()
            .map(|credential| WEBAUTHN_CREDENTIAL_EX {
                dwVersion: WEBAUTHN_CREDENTIAL_EX_CURRENT_VERSION,
                cbId: credential.len() as u32,
                pbId: credential.as_ptr() as *mut u8,
                pwszCredentialType: WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY,
                dwTransports: 0,
            })
            .collect();
        let mut exclude_pointers: Vec<*mut WEBAUTHN_CREDENTIAL_EX> =
            excluded.iter_mut().map(|c| c as *mut _).collect();
        let mut exclude_list = WEBAUTHN_CREDENTIAL_LIST {
            cCredentials: exclude_pointers.len() as u32,
            ppCredentials: exclude_pointers.as_mut_ptr(),
        };
        let api_version = unsafe { WebAuthNGetApiVersionNumber() };
        let use_prf = request.prf_input.is_some() && api_version >= 8;
        let mut options = WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS {
            dwVersion: if use_prf {
                8
            } else {
                WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS_CURRENT_VERSION
            },
            dwTimeoutMilliseconds: 120_000,
            bRequireResidentKey: true.into(),
            dwUserVerificationRequirement: WEBAUTHN_USER_VERIFICATION_REQUIRED,
            dwAuthenticatorAttachment: WEBAUTHN_AUTHENTICATOR_ATTACHMENT_ANY,
            dwAttestationConveyancePreference: WEBAUTHN_ATTESTATION_CONVEYANCE_PREFERENCE_NONE,
            bEnablePrf: request.prf_input.is_some().into(),
            ..Default::default()
        };
        if !exclude_pointers.is_empty() {
            options.pExcludeCredentialList = &mut exclude_list;
        }
        let mut prf_input = request.prf_input.filter(|_| use_prf).map(Box::new);
        let mut prf_salt = prf_input.as_mut().map(|input| WEBAUTHN_HMAC_SECRET_SALT {
            cbFirst: 32,
            pbFirst: input.as_mut_ptr(),
            cbSecond: 0,
            pbSecond: std::ptr::null_mut(),
        });
        let options_v8 = MakeCredentialOptionsV8 {
            base: options,
            pPRFGlobalEval: prf_salt
                .as_mut()
                .map_or(std::ptr::null_mut(), |salt| salt as *mut _),
            cCredentialHints: 0,
            ppwszCredentialHints: std::ptr::null_mut(),
            bThirdPartyPayment: false.into(),
        };
        // SAFETY: every pointer stays alive for the call; the platform returns
        // an allocation freed below.
        let call_result = unsafe {
            WebAuthNAuthenticatorMakeCredential(
                HWND(request.window as *mut c_void),
                &rp,
                &user,
                &params,
                &client,
                Some(&options_v8.base),
            )
        };
        drop(prf_input);
        let attestation = call_result.map_err(|error| CeremonyError::Failed(error.to_string()))?;
        if attestation.is_null() {
            return Err(CeremonyError::Incomplete);
        }
        // SAFETY: the platform owns the attestation until the free call.
        unsafe {
            let attestation = &*attestation;
            let credential_id = bytes(attestation.pbCredentialId, attestation.cbCredentialId);
            let object = bytes(
                attestation.pbAttestationObject,
                attestation.cbAttestationObject,
            );
            let transports = transports(attestation.dwUsedTransport);
            let prf = if use_prf && attestation.dwVersion >= 7 {
                let extended = &*(attestation as *const _ as *const CredentialAttestationV7);
                if extended.pHmacSecret.is_null() {
                    None
                } else {
                    let secret = &*extended.pHmacSecret;
                    prf_output(&bytes(secret.pbFirst, secret.cbFirst))
                }
            } else {
                None
            };
            WebAuthNFreeCredentialAttestation(Some(attestation as *const _));
            if object.is_empty() || credential_id.is_empty() {
                return Err(CeremonyError::Incomplete);
            }
            Ok(PasskeyRegistration {
                credential_id,
                attestation_object: object,
                client_data_json: client_data.to_vec(),
                transports,
                prf,
            })
        }
    }

    pub(super) fn assert(request: &AssertionRequest) -> Result<PasskeyAssertion, CeremonyError> {
        let rp_id = wide(&request.rp_id);
        let client_data = client_data_json("webauthn.get", &request.challenge, &request.origin);
        let client = WEBAUTHN_CLIENT_DATA {
            dwVersion: WEBAUTHN_CLIENT_DATA_CURRENT_VERSION,
            cbClientDataJSON: client_data.len() as u32,
            pbClientDataJSON: client_data.as_ptr() as *mut u8,
            pwszHashAlgId: WEBAUTHN_HASH_ALGORITHM_SHA_256,
        };
        let mut allowed: Vec<WEBAUTHN_CREDENTIAL_EX> = request
            .allow_credentials
            .iter()
            .map(|credential| WEBAUTHN_CREDENTIAL_EX {
                dwVersion: WEBAUTHN_CREDENTIAL_EX_CURRENT_VERSION,
                cbId: credential.len() as u32,
                pbId: credential.as_ptr() as *mut u8,
                pwszCredentialType: WEBAUTHN_CREDENTIAL_TYPE_PUBLIC_KEY,
                dwTransports: 0,
            })
            .collect();
        let mut pointers: Vec<*mut WEBAUTHN_CREDENTIAL_EX> =
            allowed.iter_mut().map(|c| c as *mut _).collect();
        let mut allow_list = WEBAUTHN_CREDENTIAL_LIST {
            cCredentials: pointers.len() as u32,
            ppCredentials: pointers.as_mut_ptr(),
        };
        // The salt's bytes live in this `Box` until the function returns, on
        // every path, as `register` keeps its PRF input; the API only reads them.
        let mut prf_input: Option<Box<[u8; 32]>> = request.prf_input.map(Box::new);
        let mut salt = prf_input.as_mut().map(|input| WEBAUTHN_HMAC_SECRET_SALT {
            cbFirst: 32,
            pbFirst: input.as_mut_ptr(),
            cbSecond: 0,
            pbSecond: std::ptr::null_mut(),
        });
        let mut salt_values = salt.as_mut().map(|salt| WEBAUTHN_HMAC_SECRET_SALT_VALUES {
            pGlobalHmacSalt: salt as *mut _,
            cCredWithHmacSecretSaltList: 0,
            pCredWithHmacSecretSaltList: std::ptr::null_mut(),
        });
        let options = WEBAUTHN_AUTHENTICATOR_GET_ASSERTION_OPTIONS {
            dwVersion: WEBAUTHN_AUTHENTICATOR_GET_ASSERTION_OPTIONS_CURRENT_VERSION,
            dwTimeoutMilliseconds: 120_000,
            dwUserVerificationRequirement: WEBAUTHN_USER_VERIFICATION_REQUIRED,
            dwAuthenticatorAttachment: WEBAUTHN_AUTHENTICATOR_ATTACHMENT_ANY,
            pAllowCredentialList: if pointers.is_empty() {
                std::ptr::null_mut()
            } else {
                &mut allow_list
            },
            pHmacSecretSaltValues: salt_values
                .as_mut()
                .map_or(std::ptr::null_mut(), |v| v as *mut _),
            ..Default::default()
        };
        // SAFETY: pointers stay alive for the call; the platform returns an
        // allocation freed below.
        let assertion = unsafe {
            WebAuthNAuthenticatorGetAssertion(
                HWND(request.window as *mut c_void),
                pcwstr(&rp_id),
                &client,
                Some(&options),
            )
        }
        .map_err(|error| CeremonyError::Failed(error.to_string()))?;
        if assertion.is_null() {
            return Err(CeremonyError::Incomplete);
        }
        // SAFETY: the platform owns the assertion until the free call.
        let result = unsafe {
            let assertion = &*assertion;
            let authenticator_data =
                bytes(assertion.pbAuthenticatorData, assertion.cbAuthenticatorData);
            let signature = bytes(assertion.pbSignature, assertion.cbSignature);
            let credential_id = bytes(assertion.Credential.pbId, assertion.Credential.cbId);
            let user_handle = if assertion.pbUserId.is_null() {
                None
            } else {
                Some(bytes(assertion.pbUserId, assertion.cbUserId))
            };
            // `pHmacSecret` exists only from version 3 of the structure; an
            // older one ends before it.
            let prf = if assertion.dwVersion < WEBAUTHN_ASSERTION_VERSION_3
                || assertion.pHmacSecret.is_null()
            {
                None
            } else {
                let secret = &*assertion.pHmacSecret;
                let bytes = bytes(secret.pbFirst, secret.cbFirst);
                bytes.try_into().ok()
            };
            WebAuthNFreeAssertion(assertion);
            if authenticator_data.is_empty() || signature.is_empty() || credential_id.is_empty() {
                return Err(CeremonyError::Incomplete);
            }
            Ok(PasskeyAssertion {
                credential_id,
                authenticator_data,
                client_data_json: client_data.to_vec(),
                signature,
                user_handle,
                prf,
            })
        };
        // The salt pointed into `prf_input`, which is freed only now, after
        // the call; an early return frees it the same way.
        drop(prf_input);
        result
    }

    const _: () = {
        let _ = std::mem::size_of::<WEBAUTHN_ASSERTION>();
        let _ = std::mem::size_of::<WEBAUTHN_CREDENTIAL>();
        let _ = std::mem::size_of::<c_void>();
        let _ = w!("");
    };

    #[cfg(test)]
    mod layout_tests {
        use super::*;

        #[test]
        fn abi_extensions_start_after_the_binding_structs() {
            assert_eq!(
                std::mem::offset_of!(MakeCredentialOptionsV8, pPRFGlobalEval),
                std::mem::size_of::<WEBAUTHN_AUTHENTICATOR_MAKE_CREDENTIAL_OPTIONS>()
            );
            assert_eq!(
                std::mem::offset_of!(CredentialAttestationV7, pHmacSecret),
                std::mem::size_of::<
                    windows::Win32::Networking::WindowsWebServices::WEBAUTHN_CREDENTIAL_ATTESTATION,
                >()
            );
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use super::*;

    /// Linux has no platform passkey UI (docs/ACCOUNTS.md › Deferred); its
    /// external security keys go through `kanso-fido` and CTAP2 instead.
    pub(super) fn available() -> bool {
        false
    }

    pub(super) fn register(
        _request: &RegistrationRequest,
    ) -> Result<PasskeyRegistration, CeremonyError> {
        Err(CeremonyError::Unsupported)
    }

    pub(super) fn assert(_request: &AssertionRequest) -> Result<PasskeyAssertion, CeremonyError> {
        Err(CeremonyError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registration_request() -> RegistrationRequest {
        RegistrationRequest {
            rp_id: "sync.kansoapp.app".into(),
            origin: "https://sync.kansoapp.app".into(),
            challenge: vec![1u8; 32],
            user_handle: vec![2u8; 16],
            username: "alice".into(),
            display_name: "Alice".into(),
            exclude_credentials: vec![],
            prf_input: Some(PRF_INPUT),
            window: 0,
        }
    }

    #[test]
    fn invalid_requests_fail_before_the_platform() {
        let mut request = registration_request();
        request.rp_id.clear();
        assert_eq!(check_registration(&request), Err(CeremonyError::Invalid));
        let mut request = registration_request();
        request.challenge.clear();
        assert_eq!(check_registration(&request), Err(CeremonyError::Invalid));
        assert_eq!(check_registration(&registration_request()), Ok(()));
        let assertion = AssertionRequest {
            rp_id: "".into(),
            origin: "https://sync.kansoapp.app".into(),
            challenge: vec![1u8; 32],
            allow_credentials: vec![],
            prf_input: None,
            user_verification: UserVerification::Required,
            window: 0,
        };
        assert_eq!(check_assertion(&assertion), Err(CeremonyError::Invalid));
    }

    /// Off the main thread a macOS ceremony doesn't start: it would have no
    /// sheet to present, and the caller hops to the main thread instead.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_macos_ceremony_starts_only_on_the_main_thread() {
        assert!(matches!(
            begin_register(&registration_request()),
            Err(CeremonyError::NotMainThread)
        ));
        // Cancelling elsewhere, or an unknown ceremony, does nothing.
        cancel(42);
    }

    #[test]
    fn client_data_has_the_three_fields_the_server_checks() {
        let data = client_data_json("webauthn.get", &[1u8; 32], "https://kanso.example");
        let text = String::from_utf8(data).unwrap();
        assert!(text.contains(r#""type":"webauthn.get""#));
        assert!(text.contains(r#""challenge":"AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE""#));
        assert!(text.contains(r#""origin":"https://kanso.example""#));
        assert!(text.contains(r#""crossOrigin":false"#));
    }

    #[test]
    fn user_verification_spells_the_webauthn_words() {
        assert_eq!(UserVerification::Required.as_str(), "required");
        assert_eq!(UserVerification::Preferred.as_str(), "preferred");
        assert_eq!(UserVerification::Discouraged.as_str(), "discouraged");
    }

    #[test]
    fn windows_prf_output_requires_exactly_32_bytes() {
        assert_eq!(prf_output(&[0x5a; 32]), Some([0x5a; 32]));
        assert_eq!(prf_output(&[0x5a; 31]), None);
        assert_eq!(prf_output(&[0x5a; 33]), None);
    }
}
