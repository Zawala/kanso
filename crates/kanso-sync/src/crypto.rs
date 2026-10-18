//! Key derivation, wrapping and sealing (docs/ACCOUNTS.md › Constructions).
//! Pure: no I/O, no clock; randomness only from `getrandom`.
//!
//! Step 1 implements the pure validators the app's forms need: [`Password`]
//! and [`RecoveryKey`] parsing and display, and [`KdfParams`] checks.
//! Everything else is the Rust builder's step 2a, pinned by
//! `tests/fixtures/crypto-v1.json`. The signatures below are the contract;
//! items the app never calls (everything but `Password`, `RecoveryKey` and
//! `KdfParams`) may be refined by the Rust builder within the spec.

use std::fmt;

use unicode_normalization::UnicodeNormalization as _;
use zeroize::Zeroizing;

use crate::wire::WireDoc;
use kanso_store::SyncDoc;

// ---------------------------------------------------------------------------
// KDF parameters
// ---------------------------------------------------------------------------

/// Argon2id parameters (version 0x13, 32-byte output). The client takes its
/// floor and default from its backend ([`crate::Backend::kdf_floor`],
/// [`crate::Backend::kdf_default`]); there is no cargo feature that lowers
/// the production floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl KdfParams {
    /// The production floor (`HttpBackend`, and the Worker regardless).
    pub const FLOOR: KdfParams = KdfParams {
        m_kib: 1_048_576,
        t: 3,
        p: 1,
    };
    /// What new accounts use in production: 1 GiB, t = 3, p = 4.
    pub const DEFAULT: KdfParams = KdfParams {
        m_kib: 1_048_576,
        t: 3,
        p: 4,
    };
    /// `MemoryServer`'s floor and default, so tests derive in milliseconds.
    pub const TEST: KdfParams = KdfParams {
        m_kib: 64,
        t: 1,
        p: 1,
    };
    /// The most any server may demand: about 2 s and 2 GiB on an M4 Pro.
    pub const CEILING: KdfParams = KdfParams {
        m_kib: 2_097_152,
        t: 8,
        p: 8,
    };

    /// Whether every parameter lies between `floor` and [`KdfParams::CEILING`]
    /// (inclusive). A prelogin answer outside is refused as `UnsafeKdf`
    /// before any key is derived or sent.
    pub fn is_within(&self, floor: &KdfParams) -> bool {
        (floor.m_kib..=Self::CEILING.m_kib).contains(&self.m_kib)
            && (floor.t..=Self::CEILING.t).contains(&self.t)
            && (floor.p.max(1)..=Self::CEILING.p).contains(&self.p)
    }
}

// ---------------------------------------------------------------------------
// Password
// ---------------------------------------------------------------------------

/// Shortest and longest password, in Unicode scalar values after NFC.
pub const MIN_PASSWORD_CHARS: usize = 12;
pub const MAX_PASSWORD_CHARS: usize = 256;

/// Why a password was refused. Only the client checks these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PasswordError {
    #[error("use at least {MIN_PASSWORD_CHARS} characters")]
    TooShort,
    #[error("use at most {MAX_PASSWORD_CHARS} characters")]
    TooLong,
    #[error("the password is the username")]
    SameAsUsername,
}

/// A password, normalized to Unicode NFC. Zeroized on drop; never printed.
pub struct Password(Zeroizing<String>);

impl Password {
    /// Normalizes `text` to NFC and checks it: 12–256 Unicode scalar values,
    /// and not equal to the username ignoring case (`username` is compared
    /// in its normalized form when it is a valid username, else as given,
    /// both lowercased). Pure (docs/ACCOUNTS.md › Password and KDF).
    pub fn new(text: &str, username: &str) -> Result<Password, PasswordError> {
        let normalized = Zeroizing::new(text.nfc().collect::<String>());
        let chars = normalized.chars().count();
        if chars < MIN_PASSWORD_CHARS {
            return Err(PasswordError::TooShort);
        }
        if chars > MAX_PASSWORD_CHARS {
            return Err(PasswordError::TooLong);
        }
        let name = crate::username::normalize_username(username)
            .unwrap_or_else(|_| username.nfc().collect());
        if *Zeroizing::new(normalized.to_lowercase()) == name.to_lowercase() {
            return Err(PasswordError::SameAsUsername);
        }
        Ok(Password(normalized))
    }

    /// The NFC text, as UTF-8 bytes for Argon2.
    #[allow(dead_code, reason = "read by the step 2a bodies")]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// Recovery key
// ---------------------------------------------------------------------------

/// Crockford base32, as the recovery key is shown.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// The checksum modulus: prime and below 32².
const CHECK_MODULUS: u128 = 1021;

/// Why recovery-key text was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RecoveryKeyError {
    /// A character outside Crockford base32 (after dropping spaces and
    /// hyphens, uppercasing, and reading O as 0 and I/L as 1). `U` is refused.
    #[error("a recovery key only has the letters and digits shown")]
    BadCharacter,
    /// Not 28 characters.
    #[error("a recovery key has 28 letters and digits")]
    WrongLength,
    /// The first character above 7, a check value of 1021 or more, or a
    /// checksum that doesn't match: a typo.
    #[error("the recovery key has a typo")]
    Checksum,
}

/// A recovery key: 16 random bytes, shown as 28 Crockford base32 characters
/// in 7 groups of 4 (26 data characters, then `V mod 1021` as 2 characters).
/// Zeroized on drop; never printed by `Debug`. See docs/ACCOUNTS.md ›
/// Recovery key.
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryKey(Zeroizing<[u8; 16]>);

impl RecoveryKey {
    /// A new random key (step 2a).
    pub fn generate() -> RecoveryKey {
        crate::stub!("2a")
    }

    /// The key holding exactly these bytes (golden vectors and tests).
    pub fn from_bytes(bytes: [u8; 16]) -> RecoveryKey {
        RecoveryKey(Zeroizing::new(bytes))
    }

    #[allow(dead_code, reason = "read by the step 2a bodies")]
    pub(crate) fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Lenient parsing: drops spaces and hyphens, uppercases, reads `O` as
    /// `0` and `I`/`L` as `1`, refuses `U` and every other character, then
    /// requires exactly 28 characters, a first character below `8`, a check
    /// value below 1021 and a matching checksum. Pure.
    pub fn parse(text: &str) -> Result<RecoveryKey, RecoveryKeyError> {
        let mut digits = Vec::with_capacity(28);
        for c in text.chars() {
            let c = match c.to_ascii_uppercase() {
                ' ' | '-' => continue,
                'O' => '0',
                'I' | 'L' => '1',
                c => c,
            };
            let digit = ALPHABET
                .iter()
                .position(|&a| c.is_ascii() && a == c as u8)
                .ok_or(RecoveryKeyError::BadCharacter)?;
            digits.push(digit as u128);
        }
        if digits.len() != 28 {
            return Err(RecoveryKeyError::WrongLength);
        }
        if digits[0] >= 8 {
            return Err(RecoveryKeyError::Checksum);
        }
        let value = digits[..26].iter().fold(0u128, |v, d| (v << 5) | d);
        let check = digits[26] * 32 + digits[27];
        if check >= CHECK_MODULUS || check != value % CHECK_MODULUS {
            return Err(RecoveryKeyError::Checksum);
        }
        Ok(RecoveryKey::from_bytes(value.to_be_bytes()))
    }

    /// The key as shown: `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`. Pure.
    pub fn display(&self) -> String {
        let value = u128::from_be_bytes(*self.0);
        let check = value % CHECK_MODULUS;
        let mut digits: Vec<u8> = (0..26)
            .map(|i| ALPHABET[((value >> (5 * (25 - i))) & 31) as usize])
            .collect();
        digits.push(ALPHABET[(check / 32) as usize]);
        digits.push(ALPHABET[(check % 32) as usize]);
        digits
            .chunks(4)
            .map(|group| std::str::from_utf8(group).expect("ASCII"))
            .collect::<Vec<_>>()
            .join("-")
    }
}

impl fmt::Debug for RecoveryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryKey(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

macro_rules! secret32 {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        /// 32 bytes, zeroized on drop, redacted in `Debug`.
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name(Zeroizing<[u8; 32]>);

        impl $name {
            /// The key holding exactly these bytes (golden vectors, Keychain).
            pub fn from_bytes(bytes: [u8; 32]) -> $name {
                $name(Zeroizing::new(bytes))
            }

            #[allow(dead_code, reason = "read by the step 2a bodies")]
            pub(crate) fn bytes(&self) -> &[u8; 32] {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

secret32!(
    /// The auth key, HKDF(master, "kanso/v1/auth"), or the recovery auth key,
    /// HKDF(recovery secret, salt, "kanso/v1/recovery-auth"). Sent as 43
    /// base64url characters; the server keeps only its SHA-256.
    AuthKey
);
secret32!(
    /// A key-encryption key: HKDF(master, "kanso/v1/kek") or the recovery
    /// KEK. Never leaves the Mac.
    Kek
);
secret32!(
    /// The random data key: root of every content key. Kept in the Keychain.
    DataKey
);
secret32!(
    /// The device proof, HKDF(device secret, info "kanso/v1/device-proof" ||
    /// lp(origin) || lp(account_id)). Sent at sign-up, sign-in and recovery;
    /// the server keeps its SHA-256 per session, and only a matching proof
    /// exempts a sign-in from `login-user` (security re-check 5).
    DeviceProof
);

impl DataKey {
    /// A new random data key (step 2a).
    pub fn generate() -> DataKey {
        crate::stub!("2a")
    }
}

impl AuthKey {
    /// 43 unpadded base64url characters, as the wire carries it.
    pub fn to_base64url(&self) -> String {
        crate::stub!("2a", self)
    }
}

impl DeviceProof {
    /// 43 unpadded base64url characters, as the wire carries it.
    pub fn to_base64url(&self) -> String {
        crate::stub!("2a", self)
    }
}

/// The two keys a password (or a recovery key) gives.
#[derive(Debug)]
pub struct DerivedKeys {
    /// Sent to the server.
    pub auth_key: AuthKey,
    /// Wraps the data key; never sent.
    pub kek: Kek,
}

/// Which key wrapped the data key; the purpose byte of the wrap's AAD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapPurpose {
    /// `0x50` (`P`): under the password KEK.
    Password,
    /// `0x52` (`R`): under the recovery KEK.
    Recovery,
}

/// Why opening keys failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    /// A wrap didn't open under this KEK and account, or the data key's key
    /// ID differs from the server's.
    #[error("the keys don't match")]
    KeyMismatch,
    /// Argon2 refused the parameters or ran out of memory.
    #[error("key derivation failed: {0}")]
    Kdf(String),
}

/// `Argon2id(password, salt_input)` then HKDF into the auth key and the KEK,
/// zeroizing the master key. `salt_input = "kanso/v1/salt" || lp(origin) ||
/// lp(username) || salt16`; `username` is normalized (docs/ACCOUNTS.md ›
/// Password and KDF, › Keys).
pub fn password_keys(
    password: &Password,
    origin: &str,
    username: &str,
    salt: &[u8; 16],
    params: KdfParams,
) -> Result<DerivedKeys, CryptoError> {
    crate::stub!("2a", password, origin, username, salt, params)
}

/// The recovery auth key and recovery KEK: HKDF(ikm = recovery secret, salt
/// = "kanso/v1/recovery" || lp(origin) || lp(username), info
/// "kanso/v1/recovery-auth" / "kanso/v1/recovery-kek").
pub fn recovery_keys(key: &RecoveryKey, origin: &str, username: &str) -> DerivedKeys {
    crate::stub!("2a", key, origin, username)
}

/// `0x01 || nonce(24) || XChaCha20-Poly1305(kek, nonce, aad, data_key)` as 98
/// base64url characters, where `aad = "kanso-wrap" || 0x01 || purpose ||
/// lp(account_id)` (the account ID binding is security re-check 3).
pub fn wrap(kek: &Kek, purpose: WrapPurpose, account_id: &str, data_key: &DataKey) -> String {
    crate::stub!("2a", kek, purpose, account_id, data_key)
}

/// Opens a wrap made by [`wrap`] with the same KEK, purpose and account ID;
/// anything else is [`CryptoError::KeyMismatch`].
pub fn unwrap(
    kek: &Kek,
    purpose: WrapPurpose,
    account_id: &str,
    wrapped: &str,
) -> Result<DataKey, CryptoError> {
    crate::stub!("2a", kek, purpose, account_id, wrapped)
}

/// HKDF(data_key, "kanso/v1/key-id", 8 bytes) as 16 hex digits.
pub fn key_id(data_key: &DataKey) -> String {
    crate::stub!("2a", data_key)
}

/// What opening a sealed value gave (docs/ACCOUNTS.md › Sealed documents).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opened<T> {
    Opened(T),
    /// Format byte `0x01`, but the AEAD, the JSON shape, the wire ID or
    /// revision, the kind, or a tombstone's empty fields didn't check out.
    Undecryptable,
    /// A format byte above `0x01`: "Update Kanso". Never overwrites anything.
    TooNew,
}

/// The content keys, derived from the data key with HKDF.
pub struct ContentKeys {
    #[allow(dead_code, reason = "read by the step 2a bodies")]
    data_key: DataKey,
}

impl ContentKeys {
    /// Derives `doc_key`, `file_key`, `id_key`, `rev_key` and
    /// `device_name_key` from the data key.
    pub fn derive(data_key: &DataKey) -> ContentKeys {
        crate::stub!("2a", data_key)
    }

    /// `kind + ":" + base64url(HMAC-SHA256(id_key, local_id)[0..16])`.
    pub fn wire_id(&self, local_id: &str) -> String {
        crate::stub!("2a", self, local_id)
    }

    /// `hex(HMAC-SHA256(rev_key, lp(local_id) || local_rev)[0..16])`.
    pub fn wire_rev(&self, local_id: &str, local_rev: &str) -> String {
        crate::stub!("2a", self, local_id, local_rev)
    }

    /// Seals a local document for `account_id`: its wire ID and revision, its
    /// deleted flag, and the sealed text with AAD `"kanso-doc" || 0x01 ||
    /// lp(account_id) || lp(wire_id) || lp(wire_rev) || deleted`.
    pub fn seal_doc(&self, account_id: &str, doc: &SyncDoc) -> WireDoc {
        crate::stub!("2a", self, account_id, doc)
    }

    /// Opens a stored document, checking everything [`Opened::Undecryptable`] lists.
    pub fn open_doc(&self, account_id: &str, wire: &WireDoc) -> Opened<SyncDoc> {
        crate::stub!("2a", self, account_id, wire)
    }

    /// `0x01 || nonce(24) || XChaCha20-Poly1305(file_key, nonce, "kanso-file"
    /// || 0x01 || lp(account_id) || lp(wire_id), bytes)`.
    pub fn seal_file(&self, account_id: &str, wire_id: &str, bytes: &[u8]) -> Vec<u8> {
        crate::stub!("2a", self, account_id, wire_id, bytes)
    }

    pub fn open_file(&self, account_id: &str, wire_id: &str, sealed: &[u8]) -> Opened<Vec<u8>> {
        crate::stub!("2a", self, account_id, wire_id, sealed)
    }

    /// Seals a device name with AAD `"kanso-device" || 0x01 ||
    /// lp(account_id) || lp(device_id)`: 55–400 base64url characters.
    pub fn seal_name(&self, account_id: &str, device_id: &str, name: &str) -> String {
        crate::stub!("2a", self, account_id, device_id, name)
    }

    pub fn open_name(&self, account_id: &str, device_id: &str, sealed: &str) -> Opened<String> {
        crate::stub!("2a", self, account_id, device_id, sealed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn password_examples() {
        assert!(Password::new("correct horse battery", "alice").is_ok());
        assert_eq!(
            Password::new("short", "alice").unwrap_err(),
            PasswordError::TooShort
        );
        assert_eq!(
            Password::new(&"x".repeat(257), "alice").unwrap_err(),
            PasswordError::TooLong
        );
        assert!(Password::new(&"x".repeat(256), "alice").is_ok());
        // Twelve characters, counted as scalar values after NFC: "é" composed
        // from e + U+0301 counts once.
        assert_eq!(
            Password::new("abcdefghije\u{301}", "alice").unwrap_err(),
            PasswordError::TooShort
        );
        assert!(Password::new("abcdefghijke\u{301}", "alice").is_ok());
        assert_eq!(
            Password::new("Correct-Horse", "correct-horse").unwrap_err(),
            PasswordError::SameAsUsername
        );
        assert_eq!(
            Password::new("CORRECT-HORSE", " Correct-Horse\t").unwrap_err(),
            PasswordError::SameAsUsername
        );
        assert!(
            format!("{:?}", Password::new("correct horse battery", "a").unwrap())
                .contains("redacted")
        );
    }

    #[test]
    fn recovery_key_examples() {
        let key = RecoveryKey::from_bytes([0; 16]);
        assert_eq!(key.display(), "0000-0000-0000-0000-0000-0000-0000");
        let max = RecoveryKey::from_bytes([0xff; 16]);
        let shown = max.display();
        assert!(shown.starts_with('7'), "{shown}");
        assert_eq!(RecoveryKey::parse(&shown).unwrap(), max);
        // Lenient: lowercase, O for 0, I and L for 1, no hyphens, spaces.
        let key = RecoveryKey::from_bytes(
            *b"\x01\x23\x45\x67\x89\xab\xcd\xef\x10\x32\x54\x76\x98\xba\xdc\xfe",
        );
        let shown = key.display();
        let typed = shown
            .replace('-', " ")
            .to_lowercase()
            .replace('0', "o")
            .replace('1', "l");
        assert_eq!(RecoveryKey::parse(&typed).unwrap(), key);
        assert_eq!(RecoveryKey::parse(&shown.replace('-', "")).unwrap(), key);
        assert_eq!(
            RecoveryKey::parse("0000-0000-0000-0000-0000-0000-000U"),
            Err(RecoveryKeyError::BadCharacter)
        );
        assert_eq!(
            RecoveryKey::parse("0000-0000-0000-0000-0000-0000-000"),
            Err(RecoveryKeyError::WrongLength)
        );
        assert_eq!(
            RecoveryKey::parse("8000-0000-0000-0000-0000-0000-0000"),
            Err(RecoveryKeyError::Checksum)
        );
        // The spec's layout example has no real checksum.
        assert_eq!(
            RecoveryKey::parse("0K7M-QX9P-3J2H-ZD8W-5TNB-1RGV-E6CA"),
            Err(RecoveryKeyError::Checksum)
        );
        assert!(format!("{key:?}").contains("redacted"));
    }

    #[test]
    fn kdf_bounds() {
        assert!(KdfParams::DEFAULT.is_within(&KdfParams::FLOOR));
        assert!(KdfParams::CEILING.is_within(&KdfParams::FLOOR));
        assert!(!KdfParams::TEST.is_within(&KdfParams::FLOOR));
        assert!(KdfParams::TEST.is_within(&KdfParams::TEST));
        assert!(
            !KdfParams {
                m_kib: 2_097_153,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::FLOOR)
        );
        assert!(
            !KdfParams {
                t: 9,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::FLOOR)
        );
        assert!(
            !KdfParams {
                p: 0,
                ..KdfParams::DEFAULT
            }
            .is_within(&KdfParams::TEST)
        );
    }

    fn key_bytes() -> impl Strategy<Value = [u8; 16]> {
        prop_oneof![Just([0u8; 16]), Just([0xffu8; 16]), any::<[u8; 16]>(),]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round trip: a displayed key parses back to the same bytes.
        #[test]
        fn prop_recovery_key_round_trips(bytes in key_bytes()) {
            let key = RecoveryKey::from_bytes(bytes);
            let shown = key.display();
            prop_assert_eq!(shown.len(), 34);
            prop_assert_eq!(RecoveryKey::parse(&shown).unwrap(), key);
        }

        /// Invariant: every single-character substitution is refused.
        #[test]
        fn prop_every_substitution_is_detected(bytes in key_bytes(), at in 0..28usize, with in 0..32usize) {
            let shown: Vec<u8> = RecoveryKey::from_bytes(bytes).display().bytes().filter(|&b| b != b'-').collect();
            prop_assume!(shown[at] != ALPHABET[with]);
            let mut typo = shown.clone();
            typo[at] = ALPHABET[with];
            prop_assert!(RecoveryKey::parse(std::str::from_utf8(&typo).unwrap()).is_err());
        }

        /// Invariant: every adjacent transposition within the 26 data
        /// characters is refused.
        #[test]
        fn prop_every_adjacent_transposition_is_detected(bytes in key_bytes(), at in 0..25usize) {
            let shown: Vec<u8> = RecoveryKey::from_bytes(bytes).display().bytes().filter(|&b| b != b'-').collect();
            prop_assume!(shown[at] != shown[at + 1]);
            let mut typo = shown.clone();
            typo.swap(at, at + 1);
            prop_assert!(RecoveryKey::parse(std::str::from_utf8(&typo).unwrap()).is_err());
        }

        /// Metamorphic: NFC and NFD spellings of a password are the same password.
        #[test]
        fn prop_nfc_and_nfd_passwords_are_equal(text in "[a-zé\u{e9}\u{301}ñÅ ]{12,40}") {
            let nfc: String = text.nfc().collect();
            let nfd: String = text.nfd().collect();
            match (Password::new(&nfc, "someone"), Password::new(&nfd, "someone")) {
                (Ok(a), Ok(b)) => prop_assert_eq!(a.as_bytes(), b.as_bytes()),
                (a, b) => prop_assert_eq!(a.map(|_| ()), b.map(|_| ())),
            }
        }

        /// Refinement: an accepted password has 12–256 scalar values in NFC
        /// and differs from the username ignoring case.
        #[test]
        fn prop_accepted_passwords_are_refined(text in "\\PC{0,300}", username in "[a-zA-Z0-9]{3,20}") {
            if let Ok(password) = Password::new(&text, &username) {
                let stored = std::str::from_utf8(password.as_bytes()).unwrap();
                prop_assert!((MIN_PASSWORD_CHARS..=MAX_PASSWORD_CHARS).contains(&stored.chars().count()));
                prop_assert_eq!(stored.nfc().collect::<String>(), stored);
                prop_assert_ne!(stored.to_lowercase(), username.to_lowercase());
            }
        }
    }
}
