//! Native platform passkeys for the account flows (docs/ACCOUNTS.md ›
//! Passkeys › Where the ceremony runs).
//!
//! The ceremony itself is `kanso-webauthn`'s: AuthenticationServices on macOS
//! 13 and later (PRF from macOS 15, checked there at run time) and the Win32
//! WebAuthn API on Windows. This module decides whether the app offers a
//! platform passkey at all, maps the server's options into the platform's
//! request and the platform's answer into `kanso-sync`'s
//! [`PasskeyRegistration`] and [`PasskeyAssertion`], and implements
//! `kanso-sync`'s [`PasskeyCeremony`] so an account flow never sees which
//! platform ran it.
//!
//! There is no web page anywhere here. If the platform call fails, passkeys
//! aren't available and the page says so; nothing falls back to a browser.

use gpui_kit::App;
use kanso_sync::passkey::{
    PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError, PasskeyRegistrationChallenge,
};
use kanso_sync::{PasskeyAssertion, PasskeyMethods, PasskeyRegistration};

/// The relying party the official app's Associated Domains entitlement names
/// (`webcredentials:sync.kansoapp.app`). It is fixed when the app is signed.
pub const OFFICIAL_RP_ID: &str = "sync.kansoapp.app";

/// The platform the app runs on, as the method rules see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    MacOs,
    Windows,
    Linux,
}

impl Platform {
    pub const fn current() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }
}

/// Whether the signed-out page offers [Continue with a passkey] and [Create
/// with a passkey]: the server advertises passkeys, this platform has a
/// native ceremony that hasn't failed, and on macOS the relying party is the
/// one in the app's entitlement. A self-hosted server with its own domain
/// doesn't get platform passkeys from the official Mac app even when it
/// advertises them. Linux never shows a platform-passkey button.
pub fn offers_platform_passkeys(
    platform: Platform,
    passkeys: Option<&PasskeyMethods>,
    ceremony_available: bool,
) -> bool {
    let Some(passkeys) = passkeys else {
        return false;
    };
    if !ceremony_available || !valid_rp_id(&passkeys.rp_id) {
        return false;
    }
    match platform {
        Platform::MacOs => passkeys.rp_id == OFFICIAL_RP_ID,
        Platform::Windows => true,
        Platform::Linux => false,
    }
}

/// A relying-party ID WebAuthn accepts: a lowercase DNS name with a dot, or
/// `localhost` for development. No scheme, port, path or IP literal.
pub fn valid_rp_id(rp_id: &str) -> bool {
    if rp_id == "localhost" {
        return true;
    }
    !rp_id.is_empty()
        && rp_id.len() <= 253
        && rp_id.contains('.')
        && !rp_id.starts_with('.')
        && !rp_id.ends_with('.')
        && rp_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
        && !rp_id.split('.').any(str::is_empty)
        && !rp_id
            .split('.')
            .all(|label| label.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `rp_id` is the server's host or a parent of it, which the app
/// checks before a ceremony that isn't bound by an entitlement (Windows and
/// Linux security keys).
pub fn rp_id_fits_server(rp_id: &str, server: &str) -> bool {
    let host = crate::account_words::host_of(server);
    let host = host.split(['/', ':']).next().unwrap_or_default();
    valid_rp_id(rp_id) && (host == rp_id || host.ends_with(&format!(".{rp_id}")))
}

/// Why a native ceremony ended, in `kanso-sync`'s words. Native messages
/// can carry authenticator details, so they are dropped here.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn ceremony_error(error: kanso_webauthn::CeremonyError) -> PasskeyCeremonyError {
    use kanso_webauthn::CeremonyError;
    match error {
        CeremonyError::Unsupported | CeremonyError::NotMainThread | CeremonyError::Invalid => {
            PasskeyCeremonyError::Unavailable
        }
        CeremonyError::Failed(_) | CeremonyError::Incomplete => PasskeyCeremonyError::Failed,
    }
}

/// The platform request for a registration the server started. PRF is asked
/// for with the fixed Kanso input; `kanso-webauthn` drops it where the
/// platform has none (macOS 13 and 14).
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn registration_request(
    challenge: &PasskeyRegistrationChallenge,
) -> kanso_webauthn::RegistrationRequest {
    kanso_webauthn::RegistrationRequest {
        rp_id: challenge.rp_id.clone(),
        origin: challenge.origin.clone(),
        challenge: challenge.challenge.clone(),
        user_handle: challenge.user_handle.clone(),
        username: challenge.username.clone(),
        display_name: challenge.display_name.clone(),
        exclude_credentials: challenge.exclude_credentials.clone(),
        prf_input: challenge.request_prf.then_some(kanso_webauthn::PRF_INPUT),
    }
}

/// The platform request for an assertion. User verification is always
/// required (docs/ACCOUNTS.md › Passkeys).
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn assertion_request(
    challenge: &PasskeyAssertionChallenge,
) -> kanso_webauthn::AssertionRequest {
    kanso_webauthn::AssertionRequest {
        rp_id: challenge.rp_id.clone(),
        origin: challenge.origin.clone(),
        challenge: challenge.challenge.clone(),
        allow_credentials: challenge.allow_credentials.clone(),
        prf_input: challenge.request_prf.then_some(kanso_webauthn::PRF_INPUT),
        user_verification: kanso_webauthn::UserVerification::Required,
    }
}

/// The platform's registration as the wire carries it. The PRF output moves
/// into the zeroizing `kanso-sync` value and is wiped from the platform's.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn registration_from_native(
    mut native: kanso_webauthn::PasskeyRegistration,
) -> PasskeyRegistration {
    use zeroize::Zeroize as _;
    let out = PasskeyRegistration {
        credential_id: std::mem::take(&mut native.credential_id),
        attestation_object: std::mem::take(&mut native.attestation_object),
        client_data_json: std::mem::take(&mut native.client_data_json),
        transports: std::mem::take(&mut native.transports),
        prf: native.prf,
    };
    if let Some(prf) = native.prf.as_mut() {
        prf.zeroize();
    }
    out
}

/// The platform's assertion as the wire carries it.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn assertion_from_native(mut native: kanso_webauthn::PasskeyAssertion) -> PasskeyAssertion {
    use zeroize::Zeroize as _;
    let out = PasskeyAssertion {
        credential_id: std::mem::take(&mut native.credential_id),
        authenticator_data: std::mem::take(&mut native.authenticator_data),
        client_data_json: std::mem::take(&mut native.client_data_json),
        signature: std::mem::take(&mut native.signature),
        user_handle: native.user_handle.take(),
        prf: native.prf,
    };
    if let Some(prf) = native.prf.as_mut() {
        prf.zeroize();
    }
    out
}

/// Whether this build can run a platform passkey ceremony now. On macOS it
/// must be asked on the main thread, which is where the page renders.
pub fn platform_available() -> bool {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        kanso_webauthn::available()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

/// The platform's own ceremony, run on the calling thread. On macOS that
/// must be the main thread: [`MainThreadCeremony`] takes an account task's
/// call there.
pub struct NativeCeremony;

impl PasskeyCeremony for NativeCeremony {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            kanso_webauthn::register(&registration_request(challenge))
                .map(registration_from_native)
                .map_err(ceremony_error)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = challenge;
            Err(PasskeyCeremonyError::Unavailable)
        }
    }

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            kanso_webauthn::assert(&assertion_request(challenge))
                .map(assertion_from_native)
                .map_err(ceremony_error)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = challenge;
            Err(PasskeyCeremonyError::Unavailable)
        }
    }
}

/// One ceremony an account task asks the main thread for.
enum Job {
    Register(
        PasskeyRegistrationChallenge,
        std::sync::mpsc::Sender<Result<PasskeyRegistration, PasskeyCeremonyError>>,
    ),
    Assert(
        PasskeyAssertionChallenge,
        std::sync::mpsc::Sender<Result<PasskeyAssertion, PasskeyCeremonyError>>,
    ),
}

/// A [`PasskeyCeremony`] an account task's thread can call: each call is
/// run by [`NativeCeremony`] on the main thread, where the system sheet can
/// be presented, and the thread waits for its answer. When the main-thread
/// side is gone (the task that serves it was dropped), a call fails as
/// unavailable instead of waiting forever.
#[derive(Clone)]
pub struct MainThreadCeremony {
    jobs: async_channel::Sender<Job>,
}

impl MainThreadCeremony {
    /// Starts serving ceremonies on the main thread. The returned task
    /// serves them while it lives; drop it when the account task ends.
    pub fn serve(cx: &mut App) -> (MainThreadCeremony, gpui_kit::Task<()>) {
        let (jobs, incoming) = async_channel::unbounded::<Job>();
        let task = cx.spawn(async move |_cx| {
            // Outside any `update`: the ceremony pumps the run loop, and
            // other UI work may run while the sheet is up.
            while let Ok(job) = incoming.recv().await {
                match job {
                    Job::Register(challenge, reply) => {
                        reply.send(NativeCeremony.register(&challenge)).ok();
                    }
                    Job::Assert(challenge, reply) => {
                        reply.send(NativeCeremony.assert(&challenge)).ok();
                    }
                }
            }
        });
        (MainThreadCeremony { jobs }, task)
    }
}

impl PasskeyCeremony for MainThreadCeremony {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
        let (reply, answer) = std::sync::mpsc::channel();
        self.jobs
            .send_blocking(Job::Register(challenge.clone(), reply))
            .map_err(|_| PasskeyCeremonyError::Unavailable)?;
        answer
            .recv()
            .unwrap_or(Err(PasskeyCeremonyError::Unavailable))
    }

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
        let (reply, answer) = std::sync::mpsc::channel();
        self.jobs
            .send_blocking(Job::Assert(challenge.clone(), reply))
            .map_err(|_| PasskeyCeremonyError::Unavailable)?;
        answer
            .recv()
            .unwrap_or(Err(PasskeyCeremonyError::Unavailable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn methods(rp_id: &str) -> PasskeyMethods {
        PasskeyMethods {
            rp_id: rp_id.to_owned(),
            security_key: true,
        }
    }

    #[test]
    fn macos_offers_platform_passkeys_only_for_the_entitled_relying_party() {
        let official = methods(OFFICIAL_RP_ID);
        assert!(offers_platform_passkeys(
            Platform::MacOs,
            Some(&official),
            true
        ));
        // A self-hosted server's own domain isn't in the entitlement.
        assert!(!offers_platform_passkeys(
            Platform::MacOs,
            Some(&methods("sync.example.com")),
            true
        ));
        // No advertisement, or a ceremony that failed: no button.
        assert!(!offers_platform_passkeys(Platform::MacOs, None, true));
        assert!(!offers_platform_passkeys(
            Platform::MacOs,
            Some(&official),
            false
        ));
    }

    #[test]
    fn windows_offers_any_valid_relying_party_and_linux_none() {
        assert!(offers_platform_passkeys(
            Platform::Windows,
            Some(&methods("sync.example.com")),
            true
        ));
        assert!(!offers_platform_passkeys(
            Platform::Windows,
            Some(&methods("10.0.0.1")),
            true
        ));
        assert!(!offers_platform_passkeys(
            Platform::Linux,
            Some(&methods(OFFICIAL_RP_ID)),
            true
        ));
    }

    #[test]
    fn relying_party_ids_are_hosts_without_extras() {
        for good in [
            OFFICIAL_RP_ID,
            "kansoapp.app",
            "localhost",
            "a-b.example.org",
        ] {
            assert!(valid_rp_id(good), "{good}");
        }
        for bad in [
            "",
            "https://sync.kansoapp.app",
            "sync.kansoapp.app:443",
            "sync.kansoapp.app/",
            "Sync.Kansoapp.App",
            "192.168.1.2",
            "nodot",
            ".kansoapp.app",
            "kansoapp.app.",
            "a..b",
        ] {
            assert!(!valid_rp_id(bad), "{bad}");
        }
    }

    #[test]
    fn the_relying_party_must_be_the_server_or_a_parent() {
        assert!(rp_id_fits_server(
            OFFICIAL_RP_ID,
            "https://sync.kansoapp.app"
        ));
        assert!(rp_id_fits_server(
            "kansoapp.app",
            "https://sync.kansoapp.app"
        ));
        assert!(rp_id_fits_server(
            "example.com",
            "https://sync.example.com:8443/"
        ));
        assert!(!rp_id_fits_server(
            OFFICIAL_RP_ID,
            "https://sync.example.com"
        ));
        // A suffix that isn't a whole label doesn't count.
        assert!(!rp_id_fits_server("ample.com", "https://sync.example.com"));
    }

    /// `kanso-fido` and `kanso-webauthn` keep their own copies of the PRF
    /// input, the CTAP salt and the client data, so they stay free of the
    /// sync client; these keep them equal to `kanso-sync`'s.
    #[test]
    fn the_platform_crates_share_kanso_syncs_constants() {
        assert_eq!(kanso_fido::PRF_INPUT, kanso_sync::PRF_INPUT);
        assert_eq!(
            kanso_fido::CTAP_PRF_LABEL,
            kanso_sync::security_key::CTAP_PRF_LABEL
        );
        assert_eq!(
            kanso_fido::ctap_salt(),
            kanso_sync::security_key::ctap_salt()
        );
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        assert_eq!(kanso_webauthn::PRF_INPUT, kanso_sync::PRF_INPUT);
    }

    proptest! {
        /// Every crate writes the same `clientDataJSON` for any challenge
        /// and origin, escapes included.
        #[test]
        fn prop_every_crate_writes_the_same_client_data(
            kind in prop::sample::select(vec!["webauthn.create", "webauthn.get"]),
            challenge in prop::collection::vec(any::<u8>(), 0..64),
            origin in "[ -~\\t\\n\\r\"\\\\\u{1}\u{1f}é]{0,40}",
        ) {
            let sync = kanso_sync::passkey::client_data_json(kind, &challenge, &origin);
            prop_assert_eq!(
                kanso_fido::client_data_json(kind, &challenge, &origin),
                sync.to_vec()
            );
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            prop_assert_eq!(
                kanso_webauthn::client_data_json(kind, &challenge, &origin),
                sync.to_vec()
            );
        }

        /// A parent of the host fits; a host that isn't under the RP ID
        /// never does.
        #[test]
        fn prop_rp_id_fits_exactly_its_subdomains(
            labels in prop::collection::vec("[a-z][a-z0-9]{0,6}", 2..5),
            extra in "[a-z][a-z0-9]{0,6}",
        ) {
            let rp_id = labels.join(".");
            let under = format!("https://{extra}.{rp_id}");
            let itself = format!("https://{rp_id}");
            let beside = format!("https://{extra}{rp_id}");
            prop_assert!(rp_id_fits_server(&rp_id, &under));
            prop_assert!(rp_id_fits_server(&rp_id, &itself));
            prop_assert!(!rp_id_fits_server(&rp_id, &beside));
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    mod native {
        use super::super::*;

        #[test]
        fn the_platform_request_carries_the_servers_options_and_the_prf_input() {
            let challenge = PasskeyRegistrationChallenge {
                rp_id: OFFICIAL_RP_ID.into(),
                origin: "https://sync.kansoapp.app".into(),
                challenge: vec![1; 32],
                user_handle: vec![2; 16],
                username: "quiet-otter-4821".into(),
                display_name: "quiet-otter-4821".into(),
                exclude_credentials: vec![vec![3; 16]],
                request_prf: true,
            };
            let request = registration_request(&challenge);
            assert_eq!(request.rp_id, challenge.rp_id);
            assert_eq!(request.challenge, challenge.challenge);
            assert_eq!(request.user_handle, challenge.user_handle);
            assert_eq!(request.exclude_credentials, challenge.exclude_credentials);
            assert_eq!(request.prf_input, Some(kanso_sync::PRF_INPUT));
            let without = registration_request(&PasskeyRegistrationChallenge {
                request_prf: false,
                ..challenge
            });
            assert_eq!(without.prf_input, None);

            let assertion = assertion_request(&PasskeyAssertionChallenge {
                rp_id: OFFICIAL_RP_ID.into(),
                origin: "https://sync.kansoapp.app".into(),
                challenge: vec![4; 32],
                allow_credentials: vec![],
                request_prf: true,
            });
            assert_eq!(
                assertion.user_verification,
                kanso_webauthn::UserVerification::Required
            );
            assert_eq!(assertion.prf_input, Some(kanso_sync::PRF_INPUT));
            assert!(assertion.allow_credentials.is_empty());
        }

        #[test]
        fn the_platform_answer_becomes_the_wire_shape() {
            let registration = registration_from_native(kanso_webauthn::PasskeyRegistration {
                credential_id: vec![9; 16],
                attestation_object: vec![0xa3],
                client_data_json: b"{}".to_vec(),
                transports: vec!["internal".into(), "hybrid".into()],
                prf: Some([7; 32]),
            });
            assert_eq!(registration.credential_id, vec![9; 16]);
            assert_eq!(registration.attestation_object, vec![0xa3]);
            assert_eq!(registration.transports, ["internal", "hybrid"]);
            assert_eq!(registration.prf, Some([7; 32]));

            let assertion = assertion_from_native(kanso_webauthn::PasskeyAssertion {
                credential_id: vec![9; 16],
                authenticator_data: vec![1; 37],
                client_data_json: b"{}".to_vec(),
                signature: vec![2; 64],
                user_handle: Some(vec![3; 16]),
                prf: None,
            });
            assert_eq!(assertion.authenticator_data, vec![1; 37]);
            assert_eq!(assertion.signature, vec![2; 64]);
            assert_eq!(assertion.user_handle, Some(vec![3; 16]));
            assert_eq!(assertion.prf, None);
        }

        #[test]
        fn native_errors_lose_their_messages() {
            use kanso_webauthn::CeremonyError;
            assert_eq!(
                ceremony_error(CeremonyError::Failed("credential 1234".into())),
                PasskeyCeremonyError::Failed
            );
            assert_eq!(
                ceremony_error(CeremonyError::Unsupported),
                PasskeyCeremonyError::Unavailable
            );
            assert_eq!(
                ceremony_error(CeremonyError::NotMainThread),
                PasskeyCeremonyError::Unavailable
            );
        }
    }
}
