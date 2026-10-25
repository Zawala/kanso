//! [`TestAuthenticator`]: a software P-256 passkey for tests (docs/ACCOUNTS.md
//! › Modules › `memory.rs`). It registers and asserts exactly as a platform
//! passkey would, signing with a real key so [`crate::MemoryServer`] and the
//! Worker verify it, and returns a deterministic PRF output. Behind the
//! `test-support` feature; the app never builds it.

use std::sync::atomic::{AtomicU32, Ordering};

use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair as _};
use sha2::{Digest as _, Sha256};

use crate::passkey::{
    PasskeyAssertion, PasskeyAssertionChallenge, PasskeyCeremony, PasskeyCeremonyError,
    PasskeyRegistration, PasskeyRegistrationChallenge, attestation_object_none, client_data_json,
};
use crate::webauthn::{FLAG_AT, FLAG_UP, FLAG_UV, cose_es256};

/// One software credential. The same value used on two "devices" behaves
/// like a synced passkey: the same credential ID, key and PRF output.
pub struct TestAuthenticator {
    credential_id: Vec<u8>,
    key: EcdsaKeyPair,
    prf_secret: [u8; 32],
    prf: bool,
    user_verification: bool,
    counts: bool,
    sign_count: AtomicU32,
    rng: SystemRandom,
}

impl TestAuthenticator {
    /// A credential whose ID and PRF output follow from `seed`; its signing
    /// key is fresh. PRF and user verification are on; the counter counts.
    pub fn new(seed: u8) -> TestAuthenticator {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng)
            .expect("a P-256 key");
        let key = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8.as_ref(), &rng)
            .expect("the key just made");
        let derive = |label: &[u8]| -> [u8; 32] {
            let mut hasher = Sha256::new();
            hasher.update(label);
            hasher.update([seed]);
            hasher.finalize().into()
        };
        TestAuthenticator {
            credential_id: derive(b"kanso-test-credential").to_vec(),
            key,
            prf_secret: derive(b"kanso-test-prf"),
            prf: true,
            user_verification: true,
            counts: true,
            sign_count: AtomicU32::new(0),
            rng,
        }
    }

    /// An authenticator without PRF (macOS 13/14, a key without
    /// `hmac-secret`): it signs in but can't unlock.
    pub fn without_prf(mut self) -> TestAuthenticator {
        self.prf = false;
        self
    }

    /// An authenticator that doesn't verify the user (UV clear).
    pub fn without_user_verification(mut self) -> TestAuthenticator {
        self.user_verification = false;
        self
    }

    /// An authenticator whose counter stays at 0, as many platform passkeys.
    pub fn without_counter(mut self) -> TestAuthenticator {
        self.counts = false;
        self
    }

    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }

    /// The PRF output for Kanso's input: `HMAC-SHA256(secret, ctap_salt)`,
    /// what CTAP2 `hmac-secret` returns and WebAuthn's PRF exposes.
    pub fn prf_output(&self) -> [u8; 32] {
        crate::crypto::hmac(&self.prf_secret, &[&crate::security_key::ctap_salt()])
    }

    /// The credential's COSE public key.
    pub fn cose_public_key(&self) -> Vec<u8> {
        cose_es256(self.key.public_key().as_ref()).expect("an uncompressed P-256 point")
    }

    /// Sets the next assertion's counter (to replay a clone).
    pub fn set_sign_count(&self, count: u32) {
        self.sign_count.store(count, Ordering::SeqCst);
    }

    fn flags(&self) -> u8 {
        FLAG_UP | if self.user_verification { FLAG_UV } else { 0 }
    }

    fn next_count(&self) -> u32 {
        if self.counts {
            self.sign_count.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            0
        }
    }

    /// Authenticator data with attested credential data, for a registration.
    pub fn registration_auth_data(&self, rp_id: &str) -> Vec<u8> {
        let mut data = Sha256::digest(rp_id.as_bytes()).to_vec();
        data.push(self.flags() | FLAG_AT);
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&[0u8; 16]);
        data.extend_from_slice(&(self.credential_id.len() as u16).to_be_bytes());
        data.extend_from_slice(&self.credential_id);
        data.extend_from_slice(&self.cose_public_key());
        data
    }

    /// Signs `authenticator_data || SHA-256(client_data_json)` (DER).
    pub fn sign(&self, authenticator_data: &[u8], client_data_json: &[u8]) -> Vec<u8> {
        let mut signed = authenticator_data.to_vec();
        signed.extend_from_slice(&Sha256::digest(client_data_json));
        self.key
            .sign(&self.rng, &signed)
            .expect("signing")
            .as_ref()
            .to_vec()
    }

    /// An assertion over `challenge` for `rp_id` and `origin`, with or
    /// without the PRF output.
    pub fn assertion(
        &self,
        rp_id: &str,
        origin: &str,
        challenge: &[u8],
        request_prf: bool,
    ) -> PasskeyAssertion {
        let mut authenticator_data = Sha256::digest(rp_id.as_bytes()).to_vec();
        authenticator_data.push(self.flags());
        authenticator_data.extend_from_slice(&self.next_count().to_be_bytes());
        let client_data = client_data_json("webauthn.get", challenge, origin).to_vec();
        let signature = self.sign(&authenticator_data, &client_data);
        PasskeyAssertion {
            credential_id: self.credential_id.clone(),
            authenticator_data,
            client_data_json: client_data,
            signature,
            user_handle: None,
            prf: (request_prf && self.prf).then(|| self.prf_output()),
        }
    }
}

impl PasskeyCeremony for TestAuthenticator {
    fn register(
        &self,
        challenge: &PasskeyRegistrationChallenge,
    ) -> Result<PasskeyRegistration, PasskeyCeremonyError> {
        if challenge
            .exclude_credentials
            .iter()
            .any(|id| id == &self.credential_id)
        {
            return Err(PasskeyCeremonyError::Failed);
        }
        let auth_data = self.registration_auth_data(&challenge.rp_id);
        Ok(PasskeyRegistration {
            credential_id: self.credential_id.clone(),
            attestation_object: attestation_object_none(&auth_data),
            client_data_json: client_data_json(
                "webauthn.create",
                &challenge.challenge,
                &challenge.origin,
            )
            .to_vec(),
            transports: vec!["internal".into()],
            prf: (challenge.request_prf && self.prf).then(|| self.prf_output()),
        })
    }

    fn assert(
        &self,
        challenge: &PasskeyAssertionChallenge,
    ) -> Result<PasskeyAssertion, PasskeyCeremonyError> {
        if !challenge.allow_credentials.is_empty()
            && !challenge
                .allow_credentials
                .iter()
                .any(|id| id == &self.credential_id)
        {
            return Err(PasskeyCeremonyError::Failed);
        }
        Ok(self.assertion(
            &challenge.rp_id,
            &challenge.origin,
            &challenge.challenge,
            challenge.request_prf,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webauthn::{Expected, verify_assertion, verify_registration};

    #[test]
    fn its_ceremonies_verify_like_a_platform_passkey() {
        let auth = TestAuthenticator::new(1);
        let origins = vec!["https://rp.test".to_owned()];
        let challenge = vec![7u8; 32];
        let registration = auth
            .register(&PasskeyRegistrationChallenge {
                rp_id: "rp.test".into(),
                origin: origins[0].clone(),
                challenge: challenge.clone(),
                user_handle: vec![1; 16],
                username: "alice".into(),
                display_name: "alice".into(),
                exclude_credentials: vec![],
                request_prf: true,
            })
            .unwrap();
        let expected = Expected {
            rp_id: "rp.test",
            origins: &origins,
            challenge: &crate::crypto::b64(&challenge),
        };
        let stored = verify_registration(
            &registration.credential_id,
            &registration.attestation_object,
            &registration.client_data_json,
            &expected,
        )
        .expect("the registration verifies");
        assert_eq!(registration.prf, Some(auth.prf_output()));
        assert_eq!(TestAuthenticator::new(1).prf_output(), auth.prf_output());
        assert_ne!(TestAuthenticator::new(2).prf_output(), auth.prf_output());

        let assertion = auth.assertion("rp.test", &origins[0], &challenge, false);
        assert_eq!(assertion.prf, None);
        assert_eq!(
            verify_assertion(
                &assertion.authenticator_data,
                &assertion.client_data_json,
                &assertion.signature,
                &stored.public_key,
                stored.alg,
                &expected,
            ),
            Some(1)
        );
        let other = TestAuthenticator::new(1).assertion("rp.test", &origins[0], &challenge, false);
        assert_eq!(
            verify_assertion(
                &other.authenticator_data,
                &other.client_data_json,
                &other.signature,
                &stored.public_key,
                stored.alg,
                &expected,
            ),
            None,
            "the same ID with another key doesn't verify"
        );
    }
}
