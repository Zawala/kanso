//! WebAuthn checks, pure, mirroring the Worker's `cloud/src/webauthn.ts`
//! (docs/ACCOUNTS.md › Passkeys): authenticator data, COSE public keys
//! (ES256, EdDSA, RS256), and the registration and assertion checks a
//! relying party makes. [`crate::MemoryServer`] uses them as its relying
//! party, and the Linux security-key path ([`crate::security_key`]) checks
//! its local assertion with them before it sends a wrap.

use ring::signature::{
    ECDSA_P256_SHA256_ASN1, ED25519, RSA_PKCS1_2048_8192_SHA256, RsaPublicKeyComponents,
    UnparsedPublicKey,
};
use sha2::{Digest as _, Sha256};

use crate::crypto::b64;

/// User present.
pub const FLAG_UP: u8 = 0x01;
/// User verified.
pub const FLAG_UV: u8 = 0x04;
/// Attested credential data included.
pub const FLAG_AT: u8 = 0x40;
/// Extension data included.
pub const FLAG_ED: u8 = 0x80;

/// The COSE algorithms Kanso accepts: ES256, EdDSA and RS256.
pub const COSE_ALGORITHMS: [i32; 3] = [-7, -8, -257];
pub const MIN_CREDENTIAL_ID: usize = 16;
pub const MAX_CREDENTIAL_ID: usize = 1023;

// --------------------------------------------------------------------- CBOR

/// A decoded CBOR item: the subset WebAuthn uses.
#[derive(Clone, Debug, PartialEq)]
pub enum Cbor {
    Int(i128),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Cbor>),
    Map(Vec<(Cbor, Cbor)>),
    Bool(bool),
    Null,
}

impl Cbor {
    /// The value under an integer or text key of a map.
    fn get(&self, key: &Cbor) -> Option<&Cbor> {
        match self {
            Cbor::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn int(&self, label: i128) -> Option<&Cbor> {
        self.get(&Cbor::Int(label))
    }

    fn bytes(&self) -> Option<&[u8]> {
        match self {
            Cbor::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }
}

const MAX_DEPTH: usize = 8;

/// Decodes one item at `at`; returns it and where it ends. Definite lengths
/// only, as WebAuthn's canonical CBOR.
pub fn decode_cbor(bytes: &[u8], at: usize) -> Option<(Cbor, usize)> {
    decode(bytes, at, 0)
}

/// Decodes exactly one item filling `bytes`.
pub fn decode_cbor_exactly(bytes: &[u8]) -> Option<Cbor> {
    let (value, end) = decode(bytes, 0, 0)?;
    (end == bytes.len()).then_some(value)
}

fn decode(bytes: &[u8], at: usize, depth: usize) -> Option<(Cbor, usize)> {
    if depth > MAX_DEPTH {
        return None;
    }
    let initial = *bytes.get(at)?;
    let major = initial >> 5;
    let info = initial & 0x1f;
    let mut at = at + 1;
    let argument = match info {
        n if n < 24 => u64::from(n),
        24..=27 => {
            let len = 1usize << (info - 24);
            let slice = bytes.get(at..at.checked_add(len)?)?;
            at += len;
            slice.iter().fold(0u64, |acc, b| (acc << 8) | u64::from(*b))
        }
        _ => return None,
    };
    let length = |arg: u64| usize::try_from(arg).ok();
    Some(match major {
        0 => (Cbor::Int(i128::from(argument)), at),
        1 => (Cbor::Int(-1 - i128::from(argument)), at),
        2 => {
            let end = at.checked_add(length(argument)?)?;
            (Cbor::Bytes(bytes.get(at..end)?.to_vec()), end)
        }
        3 => {
            let end = at.checked_add(length(argument)?)?;
            let text = std::str::from_utf8(bytes.get(at..end)?).ok()?;
            (Cbor::Text(text.to_owned()), end)
        }
        4 => {
            let count = length(argument)?;
            if count > bytes.len() {
                return None;
            }
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let (item, end) = decode(bytes, at, depth + 1)?;
                items.push(item);
                at = end;
            }
            (Cbor::Array(items), at)
        }
        5 => {
            let count = length(argument)?;
            if count > bytes.len() {
                return None;
            }
            let mut entries = Vec::with_capacity(count);
            for _ in 0..count {
                let (key, end) = decode(bytes, at, depth + 1)?;
                let (value, end) = decode(bytes, end, depth + 1)?;
                if entries.iter().any(|(k, _)| *k == key) {
                    return None;
                }
                entries.push((key, value));
                at = end;
            }
            (Cbor::Map(entries), at)
        }
        7 => match info {
            20 => (Cbor::Bool(false), at),
            21 => (Cbor::Bool(true), at),
            22 => (Cbor::Null, at),
            _ => return None,
        },
        _ => return None,
    })
}

// ------------------------------------------------------ authenticator data

/// Authenticator data (WebAuthn §6.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatorData {
    pub rp_id_hash: [u8; 32],
    pub flags: u8,
    pub sign_count: u32,
    /// Present when AT is set (a registration).
    pub attested: Option<AttestedCredential>,
}

/// The attested credential data of a registration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestedCredential {
    pub aaguid: [u8; 16],
    pub credential_id: Vec<u8>,
    /// The COSE public key's exact bytes.
    pub public_key: Vec<u8>,
}

/// Parses authenticator data; every byte must be accounted for.
pub fn parse_authenticator_data(bytes: &[u8]) -> Option<AuthenticatorData> {
    if bytes.len() < 37 {
        return None;
    }
    let flags = bytes[32];
    let sign_count = u32::from_be_bytes(bytes[33..37].try_into().ok()?);
    let mut at = 37;
    let mut attested = None;
    if flags & FLAG_AT != 0 {
        let aaguid: [u8; 16] = bytes.get(at..at + 16)?.try_into().ok()?;
        let length = usize::from(u16::from_be_bytes(
            bytes.get(at + 16..at + 18)?.try_into().ok()?,
        ));
        at += 18;
        if !(MIN_CREDENTIAL_ID..=MAX_CREDENTIAL_ID).contains(&length) {
            return None;
        }
        let credential_id = bytes.get(at..at + length)?.to_vec();
        at += length;
        let (key, end) = decode_cbor(bytes, at)?;
        if !matches!(key, Cbor::Map(_)) {
            return None;
        }
        attested = Some(AttestedCredential {
            aaguid,
            credential_id,
            public_key: bytes[at..end].to_vec(),
        });
        at = end;
    }
    if flags & FLAG_ED != 0 {
        let (extensions, end) = decode_cbor(bytes, at)?;
        if !matches!(extensions, Cbor::Map(_)) {
            return None;
        }
        at = end;
    }
    (at == bytes.len()).then(|| AuthenticatorData {
        rp_id_hash: bytes[..32].try_into().expect("32 bytes"),
        flags,
        sign_count,
        attested,
    })
}

// ---------------------------------------------------------------- COSE keys

/// The algorithm a COSE public key declares, when it is one Kanso accepts
/// and well formed.
pub fn cose_algorithm(cose: &[u8]) -> Option<i32> {
    let key = decode_cbor_exactly(cose)?;
    let alg = key.int(3)?;
    let kty = key.int(1)?;
    let bytes_of = |label: i128, len: Option<usize>| {
        key.int(label)
            .and_then(Cbor::bytes)
            .is_some_and(|b| len.is_none_or(|len| b.len() == len))
    };
    match alg {
        Cbor::Int(-7) => (kty == &Cbor::Int(2)
            && key.int(-1) == Some(&Cbor::Int(1))
            && bytes_of(-2, Some(32))
            && bytes_of(-3, Some(32)))
        .then_some(-7),
        Cbor::Int(-8) => {
            (kty == &Cbor::Int(1) && key.int(-1) == Some(&Cbor::Int(6)) && bytes_of(-2, Some(32)))
                .then_some(-8)
        }
        Cbor::Int(-257) => {
            let n = key.int(-1).and_then(Cbor::bytes)?;
            (kty == &Cbor::Int(3) && (256..=512).contains(&n.len()) && bytes_of(-2, None))
                .then_some(-257)
        }
        _ => None,
    }
}

/// The COSE map of a P-256 key from its uncompressed SEC1 point.
pub fn cose_es256(point: &[u8]) -> Option<Vec<u8>> {
    if point.len() != 65 || point[0] != 0x04 {
        return None;
    }
    // map(5) {1: 2, 3: -7, -1: 1, -2: x, -3: y}
    let mut out = vec![0xa5, 0x01, 0x02, 0x03, 0x26, 0x20, 0x01, 0x21, 0x58, 0x20];
    out.extend_from_slice(&point[1..33]);
    out.extend_from_slice(&[0x22, 0x58, 0x20]);
    out.extend_from_slice(&point[33..]);
    Some(out)
}

/// Whether `signature` by the COSE key `cose` (of algorithm `alg`) verifies
/// over `data`. ES256 signatures are ASN.1 DER, as WebAuthn sends them.
pub fn verify_signature(cose: &[u8], alg: i32, data: &[u8], signature: &[u8]) -> bool {
    if cose_algorithm(cose) != Some(alg) {
        return false;
    }
    let Some(key) = decode_cbor_exactly(cose) else {
        return false;
    };
    let bytes = |label| key.int(label).and_then(Cbor::bytes).unwrap_or_default();
    match alg {
        -7 => {
            let mut point = vec![0x04];
            point.extend_from_slice(bytes(-2));
            point.extend_from_slice(bytes(-3));
            UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, point)
                .verify(data, signature)
                .is_ok()
        }
        -8 => UnparsedPublicKey::new(&ED25519, bytes(-2))
            .verify(data, signature)
            .is_ok(),
        -257 => RsaPublicKeyComponents {
            n: bytes(-1),
            e: bytes(-2),
        }
        .verify(&RSA_PKCS1_2048_8192_SHA256, data, signature)
        .is_ok(),
        _ => false,
    }
}

// ---------------------------------------------------------------- ceremonies

/// What a ceremony must match: the relying party, the origins it accepts
/// and the flow's challenge (43 base64url characters, as the start gave it).
#[derive(Clone, Copy, Debug)]
pub struct Expected<'a> {
    pub rp_id: &'a str,
    pub origins: &'a [String],
    pub challenge: &'a str,
}

/// `clientDataJSON`'s type, challenge, an accepted origin, and no
/// cross-origin use (WebAuthn §7.1 steps 5–10, §7.2 steps 10–14).
pub fn check_client_data(client_data_json: &[u8], kind: &str, expected: &Expected) -> bool {
    let Ok(serde_json::Value::Object(data)) =
        serde_json::from_slice::<serde_json::Value>(client_data_json)
    else {
        return false;
    };
    data.get("type").and_then(|v| v.as_str()) == Some(kind)
        && data.get("challenge").and_then(|v| v.as_str()) == Some(expected.challenge)
        && data
            .get("origin")
            .and_then(|v| v.as_str())
            .is_some_and(|origin| expected.origins.iter().any(|o| o == origin))
        && data
            .get("crossOrigin")
            .is_none_or(|v| v == &serde_json::Value::Bool(false))
}

fn check_authenticator(data: &AuthenticatorData, rp_id: &str) -> bool {
    let required = FLAG_UP | FLAG_UV;
    data.rp_id_hash == <[u8; 32]>::from(Sha256::digest(rp_id.as_bytes()))
        && data.flags & required == required
}

/// A verified registration: the credential's key, its algorithm and count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registration {
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub alg: i32,
    pub sign_count: u32,
}

/// The attestation object's `authData`, for attestation `"none"` with an
/// empty statement, in any key order.
pub fn attestation_auth_data(attestation_object: &[u8]) -> Option<Vec<u8>> {
    let object = decode_cbor_exactly(attestation_object)?;
    if object.get(&Cbor::Text("fmt".into())) != Some(&Cbor::Text("none".into())) {
        return None;
    }
    match object.get(&Cbor::Text("attStmt".into())) {
        Some(Cbor::Map(entries)) if entries.is_empty() => {}
        _ => return None,
    }
    object
        .get(&Cbor::Text("authData".into()))
        .and_then(Cbor::bytes)
        .map(<[u8]>::to_vec)
}

/// Verifies a registration (WebAuthn §7.1) with attestation `"none"`: the
/// client data, the RP ID hash, UP and UV, attested credential data naming
/// `credential_id`, and a public key in an accepted algorithm.
pub fn verify_registration(
    credential_id: &[u8],
    attestation_object: &[u8],
    client_data_json: &[u8],
    expected: &Expected,
) -> Option<Registration> {
    if !check_client_data(client_data_json, "webauthn.create", expected) {
        return None;
    }
    let auth_data = attestation_auth_data(attestation_object)?;
    let data = parse_authenticator_data(&auth_data)?;
    if !check_authenticator(&data, expected.rp_id) {
        return None;
    }
    let attested = data.attested?;
    if attested.credential_id != credential_id {
        return None;
    }
    let alg = cose_algorithm(&attested.public_key)?;
    Some(Registration {
        credential_id: attested.credential_id,
        public_key: attested.public_key,
        alg,
        sign_count: data.sign_count,
    })
}

/// Verifies an assertion (WebAuthn §7.2) against a stored key: the client
/// data, the RP ID hash, UP and UV, no attested data, then the signature
/// over `authenticator_data || SHA-256(client_data_json)`. Returns the new
/// sign count; the caller applies [`sign_count_accepted`].
pub fn verify_assertion(
    authenticator_data: &[u8],
    client_data_json: &[u8],
    signature: &[u8],
    public_key: &[u8],
    alg: i32,
    expected: &Expected,
) -> Option<u32> {
    if !check_client_data(client_data_json, "webauthn.get", expected) {
        return None;
    }
    let data = parse_authenticator_data(authenticator_data)?;
    if data.attested.is_some() || !check_authenticator(&data, expected.rp_id) {
        return None;
    }
    let mut signed = authenticator_data.to_vec();
    signed.extend_from_slice(&Sha256::digest(client_data_json));
    verify_signature(public_key, alg, &signed, signature).then_some(data.sign_count)
}

/// The sign-count rule: a stored count of 0 means the authenticator doesn't
/// count and is never enforced; when both are above 0, a new count at or
/// below the stored one is a possible clone.
pub fn sign_count_accepted(stored: u32, next: u32) -> bool {
    stored == 0 || next == 0 || next > stored
}

/// The challenge as `clientDataJSON` carries it.
pub fn challenge_text(challenge: &[u8]) -> String {
    b64(challenge)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use ring::rand::SystemRandom;
    use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair as _};

    #[test]
    fn cbor_reads_what_webauthn_sends() {
        let cose = cose_es256(&[[4u8].as_slice(), &[1; 64]].concat()).unwrap();
        assert_eq!(cose_algorithm(&cose), Some(-7));
        assert_eq!(
            decode_cbor_exactly(&[0x39, 0x01, 0x00]),
            Some(Cbor::Int(-257))
        );
        // A duplicate key, a truncated item and trailing bytes are refused.
        assert_eq!(decode_cbor_exactly(&[0xa2, 0x01, 0x01, 0x01, 0x02]), None);
        assert_eq!(decode_cbor_exactly(&[0x58, 0x20, 0x00]), None);
        assert_eq!(decode_cbor_exactly(&[0x01, 0x01]), None);
    }

    #[test]
    fn an_es256_signature_verifies_and_a_flipped_one_does_not() {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8.as_ref(), &rng)
            .unwrap();
        let cose = cose_es256(pair.public_key().as_ref()).unwrap();
        let signature = pair.sign(&rng, b"data").unwrap();
        assert!(verify_signature(&cose, -7, b"data", signature.as_ref()));
        assert!(!verify_signature(&cose, -7, b"date", signature.as_ref()));
        assert!(!verify_signature(&cose, -8, b"data", signature.as_ref()));
    }

    proptest! {
        /// Arbitrary bytes never panic the parsers.
        #[test]
        fn prop_parsers_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..200)) {
            let _ = decode_cbor_exactly(&bytes);
            let _ = parse_authenticator_data(&bytes);
            let _ = cose_algorithm(&bytes);
            let _ = attestation_auth_data(&bytes);
        }

        #[test]
        fn prop_sign_count_refuses_only_a_non_growing_nonzero_count(stored: u32, next: u32) {
            let accepted = sign_count_accepted(stored, next);
            prop_assert_eq!(accepted, stored == 0 || next == 0 || next > stored);
        }
    }
}
