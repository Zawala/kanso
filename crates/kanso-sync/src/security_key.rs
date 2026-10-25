//! The Linux security-key path's pure half (docs/ACCOUNTS.md › Passkeys ›
//! Where the ceremony runs › Linux).
//!
//! Linux has no platform passkey UI: the app is the CTAP2 client and uses
//! [libfido2](https://developers.yubico.com/libfido2/) for USB HID and NFC.
//! This module only maps what libfido2 returns to the same WebAuthn shapes the
//! macOS and Windows ceremonies produce. Enumeration and every libfido2 call
//! stay behind the app's platform module, so the server needs no Linux route
//! and this logic is testable without hardware.
//!
//! CTAP2's `hmac-secret` extension returns the same credential-scoped secret
//! the WebAuthn PRF extension does. The app computes the same salt WebAuthn
//! would (`SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`) and passes it as
//! `salt1`; the authenticator's `HMAC-SHA256(credential_secret, salt)` is
//! `prf_out` exactly, used without another hash.

use std::sync::Mutex;

use sha2::{Digest as _, Sha256};
use zeroize::{Zeroize as _, Zeroizing};

use crate::crypto::{PRF_INPUT, b64};
use crate::passkey::{
    PasskeyAssertion, PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError,
    PasskeyRegistration, PasskeyRegistrationChallenge, attestation_object_none,
};
use crate::webauthn::{Expected, cose_algorithm, parse_authenticator_data, verify_assertion};

/// The label WebAuthn hashes into the salt before the authenticator sees it.
pub const CTAP_PRF_LABEL: &[u8] = b"WebAuthn PRF";

/// `ctap_salt = SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`: the same salt
/// WebAuthn's PRF extension computes, passed to `fido_assert_set_hmac_salt`.
pub fn ctap_salt() -> [u8; 32] {
    let mut input = Vec::with_capacity(CTAP_PRF_LABEL.len() + 1 + PRF_INPUT.len());
    input.extend_from_slice(CTAP_PRF_LABEL);
    input.push(0x00);
    input.extend_from_slice(&PRF_INPUT);
    Sha256::digest(&input).into()
}

/// What `fido_dev_make_cred` gave the app. `auth_data` is libfido2's
/// `fido_cred_authdata_ptr`, already in WebAuthn's authenticator-data layout.
#[derive(Clone, PartialEq, Eq)]
pub struct SecurityKeyRegistration {
    pub credential_id: Vec<u8>,
    pub auth_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub transports: Vec<String>,
    /// The `hmac-secret` assertion that follows registration for a key that
    /// advertises it. libfido2's make-credential binding does not expose
    /// `hmac-secret-mc`, so the app runs one immediate local assertion.
    pub prf: Option<[u8; 32]>,
}

/// What `fido_dev_get_assert` gave the app.
#[derive(Clone, PartialEq, Eq)]
pub struct SecurityKeyAssertion {
    pub credential_id: Vec<u8>,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
    pub prf: Option<[u8; 32]>,
}

// The PRF output is key material: wiped when its holder goes, as
// `PasskeyRegistration` and `PasskeyAssertion` wipe theirs.
impl Drop for SecurityKeyRegistration {
    fn drop(&mut self) {
        if let Some(prf_out) = self.prf.as_mut() {
            prf_out.zeroize();
        }
    }
}

impl Drop for SecurityKeyAssertion {
    fn drop(&mut self) {
        if let Some(prf_out) = self.prf.as_mut() {
            prf_out.zeroize();
        }
    }
}

impl SecurityKeyRegistration {
    /// The standard registration message the Worker checks: attestation
    /// `"none"` and the authenticator data libfido2 produced.
    pub fn to_passkey(&self) -> PasskeyRegistration {
        PasskeyRegistration {
            credential_id: self.credential_id.clone(),
            attestation_object: attestation_object_none(&self.auth_data),
            client_data_json: self.client_data_json.clone(),
            transports: self.transports.clone(),
            prf: self.prf,
        }
    }
}

impl SecurityKeyAssertion {
    /// The standard assertion message the Worker checks.
    pub fn to_passkey(&self) -> PasskeyAssertion {
        PasskeyAssertion {
            credential_id: self.credential_id.clone(),
            authenticator_data: self.authenticator_data.clone(),
            client_data_json: self.client_data_json.clone(),
            signature: self.signature.clone(),
            user_handle: self.user_handle.clone(),
            prf: self.prf,
        }
    }
}

/// Whether the credential in `auth_data` is the one the key said it made.
/// The app checks this locally before trusting an immediate `hmac-secret`
/// assertion over the new credential.
pub fn auth_data_has_credential_id(auth_data: &[u8], credential_id: &[u8]) -> bool {
    crate::passkey::attested_credential_id(auth_data) == Some(credential_id)
}

// ------------------------------------------- the registration's PRF check

/// The label of the local assertion's challenge.
pub const LOCAL_CHECK_LABEL: &[u8] = b"kanso/v1/security-key-prf-check";

/// The challenge of the one local assertion that follows a Linux
/// registration (docs/ACCOUNTS.md › Where the ceremony runs › Linux):
/// `SHA-256("kanso/v1/security-key-prf-check" || 0x01 || registration
/// challenge)`. Derived, so it is fresh per registration and can never be a
/// server challenge replayed.
pub fn local_check_challenge(registration_challenge: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(LOCAL_CHECK_LABEL);
    hasher.update([0x01]);
    hasher.update(registration_challenge);
    hasher.finalize().into()
}

/// What the local assertion's ceremony needs: the new credential only, the
/// derived challenge, and `hmac-secret` with [`ctap_salt`].
pub fn local_check_request(
    registration: &PasskeyRegistrationChallenge,
    credential_id: &[u8],
) -> PasskeyAssertionChallenge {
    PasskeyAssertionChallenge {
        rp_id: registration.rp_id.clone(),
        origin: registration.origin.clone(),
        challenge: local_check_challenge(&registration.challenge).to_vec(),
        allow_credentials: vec![credential_id.to_vec()],
        request_prf: true,
    }
}

/// Why a registration's local PRF check failed. Any of them means no wrap
/// is sent: the credential still registers and signs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LocalCheckError {
    #[error("the registration's authenticator data is malformed")]
    MalformedRegistration,
    #[error("the assertion is from another credential")]
    OtherCredential,
    #[error("the assertion doesn't verify against the new credential")]
    BadAssertion,
    #[error("the key returned no hmac-secret output")]
    NoPrf,
    #[error("the second touch didn't complete")]
    Ceremony(PasskeyCeremonyError),
}

/// Checks the local assertion after a registration and returns its PRF
/// output: the assertion names the new credential, its client data is a
/// `webauthn.get` over [`local_check_challenge`] at the registration's
/// origin, the RP ID hash matches, UP and UV are set, its signature verifies
/// against the public key the registration attested, and `hmac-secret`
/// answered. Pure; the app sends the wrap only when this is `Ok`.
pub fn verify_local_check(
    challenge: &PasskeyRegistrationChallenge,
    registration: &SecurityKeyRegistration,
    assertion: &SecurityKeyAssertion,
) -> Result<Zeroizing<[u8; 32]>, LocalCheckError> {
    let attested = parse_authenticator_data(&registration.auth_data)
        .and_then(|data| data.attested)
        .filter(|attested| attested.credential_id == registration.credential_id)
        .ok_or(LocalCheckError::MalformedRegistration)?;
    let alg = cose_algorithm(&attested.public_key).ok_or(LocalCheckError::MalformedRegistration)?;
    if assertion.credential_id != registration.credential_id {
        return Err(LocalCheckError::OtherCredential);
    }
    let origins = [challenge.origin.clone()];
    let expected_challenge = b64(&local_check_challenge(&challenge.challenge));
    verify_assertion(
        &assertion.authenticator_data,
        &assertion.client_data_json,
        &assertion.signature,
        &attested.public_key,
        alg,
        &Expected {
            rp_id: &challenge.rp_id,
            origins: &origins,
            challenge: &expected_challenge,
        },
    )
    .ok_or(LocalCheckError::BadAssertion)?;
    assertion
        .prf
        .map(Zeroizing::new)
        .ok_or(LocalCheckError::NoPrf)
}

/// The app's libfido2 client (kanso-app). `make_credential` runs
/// `authenticatorMakeCredential` (`rk`, UV required) and returns
/// `prf: None`; `get_assertion` runs `authenticatorGetAssertion` with
/// `hmac-secret` over [`ctap_salt`] when `request_prf` is set, UV required.
/// Both build the standard `clientDataJSON`
/// ([`crate::passkey::client_data_json`]).
pub trait SecurityKeyCeremony {
    fn make_credential(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<SecurityKeyRegistration, PasskeyCeremonyError>;

    fn get_assertion(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<SecurityKeyAssertion, PasskeyCeremonyError>;
}

/// A Linux registration, ready for [`crate::sign_up_with_passkey`] or
/// [`crate::add_passkey`]. `registration.prf` is set only when the local
/// check passed; `check` says why it is unset.
pub struct SecurityKeyRegistered {
    pub registration: PasskeyRegistration,
    /// `None` when the key has no `hmac-secret` or the check passed; the
    /// failure otherwise, which the app shows ("This key can sign in, but
    /// can't unlock a new device").
    pub check: Option<LocalCheckError>,
}

/// Registers a security key: the make-credential ceremony, then, for a key
/// that advertises `hmac-secret`, the one local assertion (a second touch),
/// verified by [`verify_local_check`] before its PRF output may wrap anything.
/// A failed registration ceremony is an error; a failed second touch or
/// check only leaves the registration without PRF.
pub fn register_security_key(
    ceremony: &dyn SecurityKeyCeremony,
    challenge: &PasskeyRegistrationChallenge,
    hmac_secret: bool,
) -> Result<SecurityKeyRegistered, PasskeyCeremonyError> {
    let mut made = ceremony.make_credential(challenge)?;
    made.prf = None;
    let check = if hmac_secret && challenge.request_prf {
        let request = local_check_request(challenge, &made.credential_id);
        match ceremony
            .get_assertion(&request)
            .map_err(LocalCheckError::Ceremony)
            .and_then(|assertion| verify_local_check(challenge, &made, &assertion))
        {
            Ok(prf) => {
                made.prf = Some(*prf);
                None
            }
            Err(err) => Some(err),
        }
    } else {
        None
    };
    Ok(SecurityKeyRegistered {
        registration: made.to_passkey(),
        check,
    })
}

/// A [`SecurityKeyCeremony`] as a [`PasskeyCeremony`], so the account flows
/// take a security key as they take a platform passkey. Registration runs
/// [`register_security_key`]; a failed local check registers without PRF,
/// and [`SecurityKeyPasskeys::check`] says why.
pub struct SecurityKeyPasskeys<'a> {
    pub ceremony: &'a dyn SecurityKeyCeremony,
    /// Whether the key advertises `hmac-secret` (`fido_cbor_info_extensions`).
    pub hmac_secret: bool,
    check: Mutex<Option<LocalCheckError>>,
}

impl<'a> SecurityKeyPasskeys<'a> {
    pub fn new(
        ceremony: &'a dyn SecurityKeyCeremony,
        hmac_secret: bool,
    ) -> SecurityKeyPasskeys<'a> {
        SecurityKeyPasskeys {
            ceremony,
            hmac_secret,
            check: Mutex::new(None),
        }
    }

    /// Why the last registration through this adapter got no PRF output
    /// ([`SecurityKeyRegistered::check`]): `None` before any, when the key
    /// has no `hmac-secret`, or when the check passed. The app shows a
    /// failure ("This key can sign in, but can't unlock a new device").
    pub fn check(&self) -> Option<LocalCheckError> {
        *self
            .check
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl PasskeyCeremony for SecurityKeyPasskeys<'_> {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
        let registered = register_security_key(self.ceremony, challenge, self.hmac_secret);
        let mut check = self
            .check
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *check = registered
            .as_ref()
            .ok()
            .and_then(|registered| registered.check);
        registered.map(|registered| registered.registration)
    }

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
        let mut request = challenge.clone();
        request.request_prf &= self.hmac_secret;
        self.ceremony
            .get_assertion(&request)
            .map(|assertion| assertion.to_passkey())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctap_salt_matches_the_webauthn_construction() {
        // `SHA-256("WebAuthn PRF" || 0x00 || PRF_INPUT)`, pinned.
        assert_eq!(
            ctap_salt(),
            [
                0x2b, 0x3e, 0xd9, 0x02, 0xb2, 0x5c, 0x43, 0xe8, 0x4d, 0xd6, 0x47, 0xaa, 0x61, 0x18,
                0xa3, 0xc5, 0x30, 0x70, 0xaa, 0xaf, 0x23, 0x04, 0x54, 0x25, 0xf8, 0xa4, 0x1f, 0x1b,
                0xa0, 0xc0, 0x4b, 0x40
            ]
        );
    }

    #[test]
    fn a_registration_becomes_the_same_webauthn_shape() {
        let mut auth_data = vec![0u8; 37];
        auth_data[32] = 0x41;
        let credential = vec![9u8; 32];
        auth_data.extend_from_slice(&[0u8; 16]);
        auth_data.extend_from_slice(&(credential.len() as u16).to_be_bytes());
        auth_data.extend_from_slice(&credential);
        auth_data.extend_from_slice(&[0xa0]); // an empty COSE key map placeholder

        let registration = SecurityKeyRegistration {
            credential_id: credential.clone(),
            auth_data: auth_data.clone(),
            client_data_json: b"{}".to_vec(),
            transports: vec!["usb".into()],
            prf: Some([3u8; 32]),
        };
        let passkey = registration.to_passkey();
        assert_eq!(passkey.credential_id, credential);
        assert_eq!(passkey.prf, Some([3u8; 32]));
        assert_eq!(
            crate::passkey::attestation_auth_data(&passkey.attestation_object),
            Some(auth_data.as_slice())
        );
        assert!(auth_data_has_credential_id(
            &auth_data,
            &passkey.credential_id
        ));
        assert!(!auth_data_has_credential_id(&auth_data, &[8u8; 32]));
    }

    use crate::TestAuthenticator;
    use proptest::prelude::*;
    use std::cell::Cell;

    /// A libfido2 stand-in over [`TestAuthenticator`]; `signer` signs the
    /// assertions (another key with the same ID forges them).
    struct FakeKey<'a> {
        key: &'a TestAuthenticator,
        signer: &'a TestAuthenticator,
        fail_assertion: bool,
        touches: Cell<usize>,
    }

    impl<'a> FakeKey<'a> {
        fn new(key: &'a TestAuthenticator) -> FakeKey<'a> {
            FakeKey {
                key,
                signer: key,
                fail_assertion: false,
                touches: Cell::new(0),
            }
        }
    }

    impl SecurityKeyCeremony for FakeKey<'_> {
        fn make_credential(
            &self,
            challenge: &PasskeyRegistrationChallenge,
        ) -> Result<SecurityKeyRegistration, PasskeyCeremonyError> {
            self.touches.set(self.touches.get() + 1);
            Ok(SecurityKeyRegistration {
                credential_id: self.key.credential_id().to_vec(),
                auth_data: self.key.registration_auth_data(&challenge.rp_id),
                client_data_json: crate::passkey::client_data_json(
                    "webauthn.create",
                    &challenge.challenge,
                    &challenge.origin,
                )
                .to_vec(),
                transports: vec!["usb".into()],
                prf: Some([0xee; 32]),
            })
        }

        fn get_assertion(
            &self,
            challenge: &PasskeyAssertionChallenge,
        ) -> Result<SecurityKeyAssertion, PasskeyCeremonyError> {
            self.touches.set(self.touches.get() + 1);
            if self.fail_assertion {
                return Err(PasskeyCeremonyError::Cancelled);
            }
            let a = self.signer.assertion(
                &challenge.rp_id,
                &challenge.origin,
                &challenge.challenge,
                challenge.request_prf,
            );
            Ok(SecurityKeyAssertion {
                credential_id: a.credential_id.clone(),
                authenticator_data: a.authenticator_data.clone(),
                client_data_json: a.client_data_json.clone(),
                signature: a.signature.clone(),
                user_handle: None,
                prf: a.prf,
            })
        }
    }

    fn registration_challenge(seed: u8) -> PasskeyRegistrationChallenge {
        PasskeyRegistrationChallenge {
            rp_id: "rp.test".into(),
            origin: "https://rp.test".into(),
            challenge: vec![seed; 32],
            user_handle: vec![1; 16],
            username: "alice".into(),
            display_name: "alice".into(),
            exclude_credentials: vec![],
            request_prf: true,
        }
    }

    #[test]
    fn a_key_with_hmac_secret_is_touched_twice_and_gets_its_prf() {
        let key = TestAuthenticator::new(1);
        let fake = FakeKey::new(&key);
        let registered = register_security_key(&fake, &registration_challenge(1), true).unwrap();
        assert_eq!(registered.check, None);
        assert_eq!(registered.registration.prf, Some(key.prf_output()));
        assert_eq!(fake.touches.get(), 2);
    }

    #[test]
    fn a_key_without_hmac_secret_is_touched_once_and_gets_none() {
        let key = TestAuthenticator::new(2);
        let fake = FakeKey::new(&key);
        let registered = register_security_key(&fake, &registration_challenge(2), false).unwrap();
        assert_eq!(
            (registered.check, registered.registration.prf),
            (None, None)
        );
        assert_eq!(fake.touches.get(), 1);
    }

    #[test]
    fn a_failed_or_forged_second_touch_sends_no_prf() {
        let key = TestAuthenticator::new(3);
        let mut fake = FakeKey::new(&key);
        fake.fail_assertion = true;
        let registered = register_security_key(&fake, &registration_challenge(3), true).unwrap();
        assert_eq!(
            registered.check,
            Some(LocalCheckError::Ceremony(PasskeyCeremonyError::Cancelled))
        );
        assert_eq!(registered.registration.prf, None);

        // The same credential ID, signed by another key.
        let impostor = TestAuthenticator::new(3);
        let mut fake = FakeKey::new(&key);
        fake.signer = &impostor;
        let registered = register_security_key(&fake, &registration_challenge(3), true).unwrap();
        assert_eq!(registered.check, Some(LocalCheckError::BadAssertion));
        assert_eq!(registered.registration.prf, None);

        // A key that didn't verify the user.
        let no_uv = TestAuthenticator::new(4).without_user_verification();
        let registered =
            register_security_key(&FakeKey::new(&no_uv), &registration_challenge(4), true).unwrap();
        assert_eq!(registered.check, Some(LocalCheckError::BadAssertion));

        // A key that answered without hmac-secret.
        let no_prf = TestAuthenticator::new(5).without_prf();
        let registered =
            register_security_key(&FakeKey::new(&no_prf), &registration_challenge(5), true)
                .unwrap();
        assert_eq!(registered.check, Some(LocalCheckError::NoPrf));
    }

    #[test]
    fn an_assertion_over_the_registration_challenge_itself_is_refused() {
        let key = TestAuthenticator::new(6);
        let challenge = registration_challenge(6);
        let fake = FakeKey::new(&key);
        let made = fake.make_credential(&challenge).unwrap();
        let mut request = local_check_request(&challenge, &made.credential_id);
        request.challenge = challenge.challenge.clone();
        let assertion = fake.get_assertion(&request).unwrap();
        assert_eq!(
            verify_local_check(&challenge, &made, &assertion),
            Err(LocalCheckError::BadAssertion)
        );
        let other = TestAuthenticator::new(7);
        let mut assertion = fake
            .get_assertion(&local_check_request(&challenge, &made.credential_id))
            .unwrap();
        assertion.credential_id = other.credential_id().to_vec();
        assert_eq!(
            verify_local_check(&challenge, &made, &assertion),
            Err(LocalCheckError::OtherCredential)
        );
    }

    /// The adapter keeps why its last registration got no PRF, so the app
    /// can say "This key can sign in, but can't unlock a new device".
    #[test]
    fn the_adapter_keeps_the_local_check_result() {
        let key = TestAuthenticator::new(9);
        let impostor = TestAuthenticator::new(9);
        let mut forged = FakeKey::new(&key);
        forged.signer = &impostor;
        let passkeys = SecurityKeyPasskeys::new(&forged, true);
        let registration = passkeys.register(&registration_challenge(9)).unwrap();
        assert_eq!(registration.prf, None);
        assert_eq!(passkeys.check(), Some(LocalCheckError::BadAssertion));
        // The next registration's result replaces it.
        let honest = FakeKey::new(&key);
        let passkeys = SecurityKeyPasskeys::new(&honest, true);
        passkeys.register(&registration_challenge(9)).unwrap();
        assert_eq!(passkeys.check(), None);
    }

    /// Reads `prf` from a value whose destructor just ran in place.
    fn prf_after_drop<T>(
        value: T,
        prf: impl Fn(&T) -> *const Option<[u8; 32]>,
    ) -> Option<[u8; 32]> {
        let mut value = std::mem::ManuallyDrop::new(value);
        let at = prf(&value);
        // SAFETY: the value stays allocated in the ManuallyDrop; only its
        // plain `Option<[u8; 32]>` field is read after its destructor ran.
        unsafe {
            std::ptr::drop_in_place(&mut *value);
            at.read()
        }
    }

    #[test]
    fn security_key_prf_outputs_are_wiped_on_drop() {
        let registration = SecurityKeyRegistration {
            credential_id: vec![1; 16],
            auth_data: vec![2; 37],
            client_data_json: vec![3],
            transports: vec!["usb".into()],
            prf: Some([7; 32]),
        };
        assert_eq!(
            prf_after_drop(registration, |r| std::ptr::addr_of!(r.prf)),
            Some([0; 32])
        );
        let assertion = SecurityKeyAssertion {
            credential_id: vec![1; 16],
            authenticator_data: vec![2; 37],
            client_data_json: vec![3],
            signature: vec![4],
            user_handle: None,
            prf: Some([7; 32]),
        };
        assert_eq!(
            prf_after_drop(assertion, |a| std::ptr::addr_of!(a.prf)),
            Some([0; 32])
        );
    }

    #[test]
    fn the_adapter_is_a_passkey_ceremony() {
        let key = TestAuthenticator::new(8);
        let fake = FakeKey::new(&key);
        let passkeys = SecurityKeyPasskeys::new(&fake, true);
        assert_eq!(passkeys.check(), None);
        let registration = passkeys.register(&registration_challenge(8)).unwrap();
        assert_eq!(registration.prf, Some(key.prf_output()));
        assert_eq!(passkeys.check(), None);
        let no_hmac = SecurityKeyPasskeys::new(&fake, false);
        let assertion = no_hmac
            .assert(&local_check_request(
                &registration_challenge(8),
                key.credential_id(),
            ))
            .unwrap();
        assert_eq!(
            assertion.prf, None,
            "no hmac-secret is asked of a key without it"
        );
    }

    proptest! {
        /// The local challenge is a function of the registration challenge,
        /// distinct per challenge, and never the challenge itself.
        #[test]
        fn prop_local_check_challenge_is_derived(
            a in proptest::collection::vec(any::<u8>(), 16..64),
            b in proptest::collection::vec(any::<u8>(), 16..64),
        ) {
            prop_assert_eq!(local_check_challenge(&a), local_check_challenge(&a));
            prop_assert_ne!(local_check_challenge(&a).to_vec(), a.clone());
            if a != b {
                prop_assert_ne!(local_check_challenge(&a), local_check_challenge(&b));
            }
        }
    }
}
