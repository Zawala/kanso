//! Locked notes: a note whose text only opens with the lock password.
//!
//! The password gives a key through Argon2id over a random 16-byte salt. A
//! locked note's text is sealed as `0x01 || nonce(24) ||
//! XChaCha20-Poly1305(key, nonce, aad, {"body": …})`, with a fresh random
//! nonce for every save and `aad = "kanso/v1/lock" || lp(note_id)`, so a
//! sealed text can't be moved onto another note. The result is kept in the
//! note, as `note.extra.kansoLock` ([`Envelope`]), with the salt and Argon2
//! parameters beside it, and the note's `body` is left empty. It syncs inside
//! the note's own sealed document, so the account's other devices open it
//! with the same password and nothing else has to travel.
//!
//! Notes locked on one device share its salt, so unlocking derives one key;
//! a note locked elsewhere has another salt, and the password derives a key
//! for it too. There is no way back from a forgotten password.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use unicode_normalization::UnicodeNormalization as _;
use zeroize::Zeroizing;

use crate::crypto::{
    FORMAT_V1, KdfParams, NONCE_LEN, Opened, b64, lp, open_raw, random, seal_raw, unb64,
};

/// Where a note keeps its sealed text, beside its other fields.
pub const LOCK_KEY: &str = kanso_core::model::LOCK_FIELD;

/// What new lock keys use: 256 MiB, t = 3, p = 4. Unlocking derives once per
/// salt, so a fraction of a second, while guessing stays costly.
pub const LOCK_KDF: KdfParams = KdfParams {
    m_kib: 262_144,
    t: 3,
    p: 4,
};

/// Shortest and longest lock password, in Unicode scalar values after NFC.
pub const MIN_LOCK_PASSWORD_CHARS: usize = 8;
pub const MAX_LOCK_PASSWORD_CHARS: usize = 256;

const AAD_PREFIX: &[u8] = b"kanso/v1/lock";

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LockError {
    #[error("use at least {MIN_LOCK_PASSWORD_CHARS} characters")]
    TooShort,
    #[error("use at most {MAX_LOCK_PASSWORD_CHARS} characters")]
    TooLong,
    /// The text didn't open under this key: the wrong password, or a sealed
    /// text that was changed.
    #[error("that isn't the lock password")]
    WrongPassword,
    /// The note's `kansoLock` isn't an envelope this version reads.
    #[error("the note's locked text is damaged")]
    Malformed,
    #[error("the note was locked by a newer Kanso; update Kanso to open it")]
    TooNew,
    #[error("key derivation failed: {0}")]
    Kdf(String),
}

/// A lock password, normalized to Unicode NFC. Zeroized on drop; never
/// printed.
pub struct LockPassword(Zeroizing<String>);

impl LockPassword {
    pub fn new(text: &str) -> Result<LockPassword, LockError> {
        let normalized = Zeroizing::new(text.nfc().collect::<String>());
        let chars = normalized.chars().count();
        if chars < MIN_LOCK_PASSWORD_CHARS {
            return Err(LockError::TooShort);
        }
        if chars > MAX_LOCK_PASSWORD_CHARS {
            return Err(LockError::TooLong);
        }
        Ok(LockPassword(normalized))
    }
}

impl fmt::Debug for LockPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LockPassword(<redacted>)")
    }
}

/// A locked note's `kansoLock`: how its key is derived and its sealed text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// The format of `data`: 1.
    pub v: u8,
    /// The Argon2id salt, base64url.
    pub salt: String,
    pub m: u32,
    pub t: u32,
    pub p: u32,
    /// The sealed text, base64url.
    pub data: String,
}

impl Envelope {
    /// The note's envelope, if it is locked; `Err` for a `kansoLock` that
    /// isn't one.
    pub fn from_extra(extra: &Map<String, Value>) -> Option<Result<Envelope, LockError>> {
        let value = extra.get(LOCK_KEY)?;
        Some(match serde_json::from_value::<Envelope>(value.clone()) {
            Ok(envelope) if envelope.v > FORMAT_V1 => Err(LockError::TooNew),
            Ok(envelope) => Ok(envelope),
            Err(_) => Err(LockError::Malformed),
        })
    }

    /// Keeps `envelope` in a note's fields, or takes the lock out.
    pub fn store(envelope: Option<&Envelope>, extra: &mut Map<String, Value>) {
        match envelope.and_then(|e| serde_json::to_value(e).ok()) {
            Some(value) => extra.insert(LOCK_KEY.to_owned(), value),
            None => extra.remove(LOCK_KEY),
        };
    }

    fn salt(&self) -> Option<[u8; 16]> {
        unb64(&self.salt)?.try_into().ok()
    }

    fn params(&self) -> KdfParams {
        KdfParams {
            m_kib: self.m,
            t: self.t,
            p: self.p,
        }
    }
}

/// The text a locked note seals.
#[derive(Serialize, Deserialize)]
struct Sealed {
    body: String,
}

/// A key derived from the lock password. Zeroized on drop; never printed.
pub struct LockKey {
    key: Zeroizing<[u8; 32]>,
    salt: [u8; 16],
    params: KdfParams,
}

impl fmt::Debug for LockKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LockKey(<redacted>)")
    }
}

impl LockKey {
    /// A key for locking notes on this device, with a new random salt.
    pub fn create(password: &LockPassword, params: KdfParams) -> Result<LockKey, LockError> {
        LockKey::derive(password, random::<16>(), params)
    }

    /// The key `password` gives for `envelope`'s salt and parameters. It may
    /// still be the wrong one: [`LockKey::open`] tells. Parameters outside
    /// [`KdfParams::TEST`]..[`KdfParams::CEILING`] are refused before any
    /// work, so a crafted envelope can't make unlocking take all memory.
    pub fn for_envelope(
        password: &LockPassword,
        envelope: &Envelope,
    ) -> Result<LockKey, LockError> {
        let salt = envelope.salt().ok_or(LockError::Malformed)?;
        let params = envelope.params();
        if !params.is_within(&KdfParams::TEST) {
            return Err(LockError::Malformed);
        }
        LockKey::derive(password, salt, params)
    }

    fn derive(
        password: &LockPassword,
        salt: [u8; 16],
        params: KdfParams,
    ) -> Result<LockKey, LockError> {
        let kdf = |e: argon2::Error| LockError::Kdf(e.to_string());
        let argon = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(params.m_kib, params.t, params.p, Some(32)).map_err(kdf)?,
        );
        let mut key = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(password.0.as_bytes(), &salt, &mut key[..])
            .map_err(kdf)?;
        Ok(LockKey { key, salt, params })
    }

    /// The key with its salt and parameters, as the Keychain keeps it for
    /// Touch ID: `1:<key>:<salt>:<m>:<t>:<p>`, base64url and decimal.
    pub fn to_secret(&self) -> Zeroizing<String> {
        let key = Zeroizing::new(b64(&self.key[..]));
        Zeroizing::new(format!(
            "{FORMAT_V1}:{}:{}:{}:{}:{}",
            key.as_str(),
            b64(&self.salt),
            self.params.m_kib,
            self.params.t,
            self.params.p
        ))
    }

    /// A key [`LockKey::to_secret`] kept.
    pub fn from_secret(text: &str) -> Result<LockKey, LockError> {
        let parts: Vec<&str> = text.split(':').collect();
        let [version, key, salt, m, t, p] = parts[..] else {
            return Err(LockError::Malformed);
        };
        match version.parse::<u8>() {
            Ok(FORMAT_V1) => {}
            Ok(_) => return Err(LockError::TooNew),
            Err(_) => return Err(LockError::Malformed),
        }
        let key = Zeroizing::new(unb64(key).ok_or(LockError::Malformed)?);
        let key =
            Zeroizing::new(<[u8; 32]>::try_from(key.as_slice()).map_err(|_| LockError::Malformed)?);
        let salt = unb64(salt)
            .and_then(|salt| salt.try_into().ok())
            .ok_or(LockError::Malformed)?;
        let number = |text: &str| text.parse::<u32>().map_err(|_| LockError::Malformed);
        Ok(LockKey {
            key,
            salt,
            params: KdfParams {
                m_kib: number(m)?,
                t: number(t)?,
                p: number(p)?,
            },
        })
    }

    /// Whether this key was derived for `envelope`'s salt and parameters,
    /// and so is the one to try on it.
    pub fn fits(&self, envelope: &Envelope) -> bool {
        envelope.salt() == Some(self.salt) && envelope.params() == self.params
    }

    /// Whether `other` was derived with the same salt and parameters: the
    /// same key, if from the same password.
    pub fn same_kdf(&self, other: &LockKey) -> bool {
        self.salt == other.salt && self.params == other.params
    }

    /// Seals `body` for note `note_id`, with a fresh nonce.
    pub fn seal(&self, note_id: &str, body: &str) -> Envelope {
        self.seal_with_nonce(note_id, body, &random::<NONCE_LEN>())
    }

    fn seal_with_nonce(&self, note_id: &str, body: &str, nonce: &[u8; NONCE_LEN]) -> Envelope {
        let plaintext = Zeroizing::new(
            serde_json::to_vec(&Sealed {
                body: body.to_owned(),
            })
            .expect("a string serializes"),
        );
        Envelope {
            v: FORMAT_V1,
            salt: b64(&self.salt),
            m: self.params.m_kib,
            t: self.params.t,
            p: self.params.p,
            data: b64(&seal_raw(&self.key, nonce, &aad(note_id), &plaintext)),
        }
    }

    /// The text `envelope` seals for note `note_id`.
    pub fn open(&self, note_id: &str, envelope: &Envelope) -> Result<Zeroizing<String>, LockError> {
        if envelope.v > FORMAT_V1 {
            return Err(LockError::TooNew);
        }
        let sealed = unb64(&envelope.data).ok_or(LockError::Malformed)?;
        match open_raw(&self.key, &aad(note_id), &sealed) {
            Opened::Opened(plaintext) => {
                let sealed: Sealed =
                    serde_json::from_slice(&plaintext).map_err(|_| LockError::Malformed)?;
                Ok(Zeroizing::new(sealed.body))
            }
            Opened::TooNew => Err(LockError::TooNew),
            Opened::Undecryptable => Err(LockError::WrongPassword),
        }
    }
}

fn aad(note_id: &str) -> Vec<u8> {
    let mut aad = AAD_PREFIX.to_vec();
    lp(&mut aad, note_id);
    aad
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn password(text: &str) -> LockPassword {
        LockPassword::new(text).unwrap()
    }

    fn key(text: &str) -> LockKey {
        LockKey::create(&password(text), KdfParams::TEST).unwrap()
    }

    #[test]
    fn passwords_need_eight_characters_after_nfc() {
        assert_eq!(
            LockPassword::new("seven77").unwrap_err(),
            LockError::TooShort
        );
        assert!(LockPassword::new("eight888").is_ok());
        // "é" as e + combining acute is one character once composed.
        assert_eq!(
            LockPassword::new("cafe\u{301}xyz").unwrap_err(),
            LockError::TooShort
        );
        assert_eq!(
            LockPassword::new(&"x".repeat(257)).unwrap_err(),
            LockError::TooLong
        );
    }

    #[test]
    fn the_password_opens_on_another_device_with_the_envelopes_salt() {
        let mine = key("correct horse");
        let envelope = mine.seal("note:a", "secret plans");
        let theirs = LockKey::for_envelope(&password("correct horse"), &envelope).unwrap();
        assert!(theirs.fits(&envelope));
        assert_eq!(
            theirs.open("note:a", &envelope).unwrap().as_str(),
            "secret plans"
        );
    }

    #[test]
    fn a_wrong_password_or_another_note_does_not_open() {
        let envelope = key("correct horse").seal("note:a", "secret plans");
        let wrong = LockKey::for_envelope(&password("battery staple"), &envelope).unwrap();
        assert_eq!(
            wrong.open("note:a", &envelope).unwrap_err(),
            LockError::WrongPassword
        );
        let right = LockKey::for_envelope(&password("correct horse"), &envelope).unwrap();
        assert_eq!(
            right.open("note:b", &envelope).unwrap_err(),
            LockError::WrongPassword
        );
    }

    #[test]
    fn an_envelope_says_nothing_of_its_text_and_keys_fit_only_their_salt() {
        let first = key("correct horse");
        let second = key("correct horse");
        let envelope = first.seal("note:a", "secret plans");
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(!json.contains("secret"));
        assert!(first.fits(&envelope));
        assert!(!second.fits(&envelope));
        // Each save gets its own nonce.
        assert_ne!(
            first.seal("note:a", "x").data,
            first.seal("note:a", "x").data
        );
    }

    #[test]
    fn damaged_too_new_and_unsafe_envelopes_are_refused() {
        let k = key("correct horse");
        let mut envelope = k.seal("note:a", "secret");
        let mut bad = envelope.clone();
        bad.data.push('!');
        assert_eq!(k.open("note:a", &bad).unwrap_err(), LockError::Malformed);
        let mut huge = envelope.clone();
        huge.m = KdfParams::CEILING.m_kib + 1;
        assert_eq!(
            LockKey::for_envelope(&password("correct horse"), &huge).unwrap_err(),
            LockError::Malformed
        );
        envelope.v = 2;
        let mut extra = Map::new();
        Envelope::store(Some(&envelope), &mut extra);
        assert_eq!(Envelope::from_extra(&extra), Some(Err(LockError::TooNew)));
        extra.insert(LOCK_KEY.into(), Value::String("nonsense".into()));
        assert_eq!(
            Envelope::from_extra(&extra),
            Some(Err(LockError::Malformed))
        );
        Envelope::store(None, &mut extra);
        assert_eq!(Envelope::from_extra(&extra), None);
    }

    #[test]
    fn a_kept_key_opens_what_the_original_sealed() {
        let k = key("correct horse");
        let envelope = k.seal("note:a", "secret");
        let kept = LockKey::from_secret(&k.to_secret()).unwrap();
        assert!(kept.fits(&envelope) && kept.same_kdf(&k));
        assert_eq!(kept.open("note:a", &envelope).unwrap().as_str(), "secret");
        assert_eq!(
            LockKey::from_secret("{}").unwrap_err(),
            LockError::Malformed
        );
        assert!(!format!("{kept:?}").contains(&k.to_secret()[..10]));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]
        /// Invariant: whatever the text, sealing then opening gives it back,
        /// and the envelope round-trips through a note's fields.
        #[test]
        fn prop_seal_then_open_round_trips(body in "\\PC{0,200}", nonce in any::<[u8; 24]>()) {
            let k = key("correct horse");
            let envelope = k.seal_with_nonce("note:a", &body, &nonce);
            let mut extra = Map::new();
            Envelope::store(Some(&envelope), &mut extra);
            let back = Envelope::from_extra(&extra).unwrap().unwrap();
            let opened = k.open("note:a", &back).unwrap();
            prop_assert_eq!(opened.as_str(), body.as_str());
        }
    }
}
