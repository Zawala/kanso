//! An account's identity keys (docs/ACCOUNTS.md › Collaboration › Identity
//! keys): Ed25519 to sign what it writes to shared notes, X25519 to receive
//! note keys and notifications. Made on the device, sealed under
//! `identity_key` for the account's other devices, and published with a
//! binding signature that ties the box key to the sign key, the account and
//! the username.

use std::fmt;

use curve25519_dalek::montgomery::MontgomeryPoint;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use zeroize::Zeroizing;

use crate::codec::{FORMAT_V1, b64, open, put_lp, random, seal, sha256, unb64, unb64_array};
use crate::model::PublicIdentity;

/// Base64url lengths on the wire.
pub const PUBLIC_KEY_CHARS: usize = 43;
pub const SIGNATURE_CHARS: usize = 86;
pub const SEALED_IDENTITY_CHARS: usize = 140;

/// An account's private identity keys. Zeroized on drop; `Debug` shows only
/// the public halves.
pub struct Identity {
    sign: SigningKey,
    box_secret: Zeroizing<[u8; 32]>,
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity")
            .field("sign_pub", &b64(&self.sign_pub()))
            .field("box_pub", &b64(&self.box_pub()))
            .finish_non_exhaustive()
    }
}

/// What a device publishes with `PUT /v2/account/identity`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Published {
    pub sign_pub: String,
    pub box_pub: String,
    pub binding: String,
    pub sealed: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum IdentityError {
    #[error("the account's identity keys don't open with this account's key")]
    Undecryptable,
    #[error("a public key or signature isn't well-formed")]
    Malformed,
    #[error("the identity's binding signature doesn't verify")]
    BadBinding,
}

impl Identity {
    /// New keys from the system's random source.
    pub fn generate() -> Identity {
        Identity::from_secrets(random::<32>(), random::<32>())
    }

    /// Keys from their 32-byte secrets.
    pub fn from_secrets(sign_seed: [u8; 32], box_secret: [u8; 32]) -> Identity {
        Identity {
            sign: SigningKey::from_bytes(&sign_seed),
            box_secret: Zeroizing::new(box_secret),
        }
    }

    pub fn sign_pub(&self) -> [u8; 32] {
        self.sign.verifying_key().to_bytes()
    }

    pub fn box_pub(&self) -> [u8; 32] {
        MontgomeryPoint::mul_base_clamped(*self.box_secret).to_bytes()
    }

    /// Ed25519 over `message`.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.sign.sign(message).to_bytes()
    }

    /// X25519 with `their_pub`; `None` when the result is all zero (a
    /// low-order point).
    pub fn agree(&self, their_pub: &[u8; 32]) -> Option<Zeroizing<[u8; 32]>> {
        x25519(&self.box_secret, their_pub)
    }

    /// The identity sealed for the account's devices, with its binding.
    pub fn publish(&self, identity_key: &[u8; 32], account_id: &str, username: &str) -> Published {
        let sign_pub = self.sign_pub();
        let box_pub = self.box_pub();
        let mut secrets = Zeroizing::new(Vec::with_capacity(64));
        secrets.extend_from_slice(self.sign.as_bytes());
        secrets.extend_from_slice(&self.box_secret[..]);
        let sealed = seal(
            identity_key,
            &sealed_aad(account_id, &sign_pub, &box_pub),
            &secrets,
        );
        Published {
            sign_pub: b64(&sign_pub),
            box_pub: b64(&box_pub),
            binding: b64(&self.sign(&binding_message(account_id, username, &box_pub))),
            sealed: b64(&sealed),
        }
    }

    /// Opens what [`Identity::publish`] sealed, checking that the keys inside
    /// are the published ones.
    pub fn open(
        identity_key: &[u8; 32],
        account_id: &str,
        published: &Published,
    ) -> Result<Identity, IdentityError> {
        let sign_pub = unb64_array::<32>(&published.sign_pub).ok_or(IdentityError::Malformed)?;
        let box_pub = unb64_array::<32>(&published.box_pub).ok_or(IdentityError::Malformed)?;
        let sealed = unb64(&published.sealed).ok_or(IdentityError::Malformed)?;
        let secrets = open(
            identity_key,
            &sealed_aad(account_id, &sign_pub, &box_pub),
            &sealed,
        )
        .ok_or(IdentityError::Undecryptable)?;
        if secrets.len() != 64 {
            return Err(IdentityError::Undecryptable);
        }
        let identity = Identity::from_secrets(
            secrets[..32].try_into().expect("32 bytes"),
            secrets[32..].try_into().expect("32 bytes"),
        );
        if identity.sign_pub() != sign_pub || identity.box_pub() != box_pub {
            return Err(IdentityError::Undecryptable);
        }
        Ok(identity)
    }

    /// The fingerprint of this identity's public keys.
    pub fn fingerprint(&self) -> String {
        fingerprint(&self.sign_pub(), &self.box_pub())
    }
}

fn sealed_aad(account_id: &str, sign_pub: &[u8; 32], box_pub: &[u8; 32]) -> Vec<u8> {
    let mut aad = b"kanso-identity".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, account_id.as_bytes());
    aad.extend_from_slice(sign_pub);
    aad.extend_from_slice(box_pub);
    aad
}

fn binding_message(account_id: &str, username: &str, box_pub: &[u8; 32]) -> Vec<u8> {
    let mut message = b"kanso-identity-bind".to_vec();
    message.push(FORMAT_V1);
    put_lp(&mut message, account_id.as_bytes());
    put_lp(&mut message, username.as_bytes());
    message.extend_from_slice(box_pub);
    message
}

/// X25519 of a secret and a public key; `None` for an all-zero result.
pub(crate) fn x25519(secret: &[u8; 32], public: &[u8; 32]) -> Option<Zeroizing<[u8; 32]>> {
    let shared = Zeroizing::new(MontgomeryPoint(*public).mul_clamped(*secret).to_bytes());
    (shared.iter().any(|&b| b != 0)).then_some(shared)
}

/// Verifies an Ed25519 signature (strictly: no malleable signatures, no
/// weak keys).
pub fn verify(sign_pub: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(sign_pub) else {
        return false;
    };
    key.verify_strict(message, &Signature::from_bytes(signature))
        .is_ok()
}

/// `SHA-256("kanso-fingerprint" || 0x01 || sign_pub || box_pub)`, its first
/// 30 bytes as six 5-digit groups.
pub fn fingerprint(sign_pub: &[u8; 32], box_pub: &[u8; 32]) -> String {
    let hash = sha256(&[b"kanso-fingerprint", &[FORMAT_V1], sign_pub, box_pub]);
    hash[..30]
        .chunks(5)
        .map(|chunk| {
            let value = chunk.iter().fold(0u64, |acc, &b| (acc << 8) | u64::from(b));
            format!("{:05}", value % 100_000)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A public identity's keys, once its binding verifies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verified {
    pub account_id: String,
    pub username: String,
    pub sign_pub: [u8; 32],
    pub box_pub: [u8; 32],
}

impl Verified {
    pub fn fingerprint(&self) -> String {
        fingerprint(&self.sign_pub, &self.box_pub)
    }
}

/// Checks a public identity: well-formed keys, and a binding that verifies
/// under its own sign key for its account, username and box key.
pub fn check_public(identity: &PublicIdentity) -> Result<Verified, IdentityError> {
    let sign_pub = unb64_array::<32>(&identity.sign_pub).ok_or(IdentityError::Malformed)?;
    let box_pub = unb64_array::<32>(&identity.box_pub).ok_or(IdentityError::Malformed)?;
    let binding = unb64_array::<64>(&identity.binding).ok_or(IdentityError::Malformed)?;
    let message = binding_message(&identity.account_id, &identity.username, &box_pub);
    if !verify(&sign_pub, &message, &binding) {
        return Err(IdentityError::BadBinding);
    }
    Ok(Verified {
        account_id: identity.account_id.clone(),
        username: identity.username.clone(),
        sign_pub,
        box_pub,
    })
}

/// Whether a published identity is well-formed (the server's shape rules).
pub fn is_published_shape(published: &Published) -> bool {
    published.sign_pub.len() == PUBLIC_KEY_CHARS
        && published.box_pub.len() == PUBLIC_KEY_CHARS
        && published.binding.len() == SIGNATURE_CHARS
        && published.sealed.len() == SEALED_IDENTITY_CHARS
        && unb64(&published.sign_pub).is_some()
        && unb64(&published.box_pub).is_some()
        && unb64(&published.binding).is_some()
        && unb64(&published.sealed).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn public(identity: &Identity, account: &str, username: &str) -> PublicIdentity {
        let published = identity.publish(&[7; 32], account, username);
        PublicIdentity {
            account_id: account.into(),
            username: username.into(),
            sign_pub: published.sign_pub,
            box_pub: published.box_pub,
            binding: published.binding,
        }
    }

    #[test]
    fn published_values_have_the_documented_lengths() {
        let identity = Identity::generate();
        let published = identity.publish(&[1; 32], "a_00", "ana");
        assert!(is_published_shape(&published), "{published:?}");
        let fp = identity.fingerprint();
        assert_eq!(fp.len(), 6 * 5 + 5);
        assert!(
            fp.split(' ')
                .all(|g| g.len() == 5 && g.bytes().all(|b| b.is_ascii_digit()))
        );
    }

    #[test]
    fn debug_shows_no_secret() {
        let identity = Identity::from_secrets([3; 32], [4; 32]);
        let text = format!("{identity:?}");
        assert!(!text.contains(&b64(&[3; 32])) && !text.contains(&b64(&[4; 32])));
    }

    proptest! {
        #![proptest_config(crate::tests::config(64))]

        /// Round trip: the account's devices open the same keys; another
        /// account's key, or another account ID, opens nothing.
        #[test]
        fn prop_sealed_identity_opens_only_for_its_account(seed in any::<[u8; 32]>(), boxk in any::<[u8; 32]>(),
                                                            key in any::<[u8; 32]>(), other in any::<[u8; 32]>()) {
            let identity = Identity::from_secrets(seed, boxk);
            let published = identity.publish(&key, "a_1", "ana");
            let opened = Identity::open(&key, "a_1", &published).unwrap();
            prop_assert_eq!(opened.sign_pub(), identity.sign_pub());
            prop_assert_eq!(opened.box_pub(), identity.box_pub());
            prop_assert!(Identity::open(&key, "a_2", &published).is_err());
            if other != key {
                prop_assert!(Identity::open(&other, "a_1", &published).is_err());
            }
        }

        /// The binding ties the box key to the account and username: change
        /// any of them, or swap in another box key, and it stops verifying.
        #[test]
        fn prop_binding_fails_when_anything_moves(a in any::<[u8; 32]>(), b in any::<[u8; 32]>(), c in any::<[u8; 32]>()) {
            let identity = Identity::from_secrets(a, b);
            let good = public(&identity, "a_1", "ana");
            prop_assert!(check_public(&good).is_ok());
            let mut renamed = good.clone();
            renamed.username = "bob".into();
            prop_assert_eq!(check_public(&renamed), Err(IdentityError::BadBinding));
            let mut moved = good.clone();
            moved.account_id = "a_2".into();
            prop_assert_eq!(check_public(&moved), Err(IdentityError::BadBinding));
            let mut swapped = good.clone();
            swapped.box_pub = b64(&Identity::from_secrets(a, c).box_pub());
            if swapped.box_pub != good.box_pub {
                prop_assert_eq!(check_public(&swapped), Err(IdentityError::BadBinding));
            }
        }

        /// X25519 agrees both ways.
        #[test]
        fn prop_agreement_is_symmetric(a in any::<[u8; 32]>(), b in any::<[u8; 32]>()) {
            let x = Identity::from_secrets([0; 32], a);
            let y = Identity::from_secrets([0; 32], b);
            prop_assert_eq!(x.agree(&y.box_pub()).map(|k| *k), y.agree(&x.box_pub()).map(|k| *k));
        }
    }
}
