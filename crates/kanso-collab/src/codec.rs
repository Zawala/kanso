//! The byte encodings shared notes use (docs/ACCOUNTS.md › Encodings and
//! › Collaboration): `lp(x)`, big-endian integers, unpadded base64url, and
//! the AEAD and KDF every construction is built from.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// The format byte of every value this crate writes.
pub const FORMAT_V1: u8 = 0x01;
pub const NONCE_LEN: usize = 24;
pub const TAG_LEN: usize = 16;

/// `lp(x)`: a 2-byte big-endian length, then the bytes. Panics past 65,535
/// bytes, which no caller can reach (IDs and names are short).
pub fn put_lp(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = u16::try_from(bytes.len()).expect("lp() takes at most 65,535 bytes");
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(bytes);
}

/// Reads `lp(x)` at `*at`, moving past it.
pub fn take_lp<'a>(bytes: &'a [u8], at: &mut usize) -> Option<&'a [u8]> {
    let len = u16::from_be_bytes(bytes.get(*at..*at + 2)?.try_into().ok()?) as usize;
    let value = bytes.get(*at + 2..*at + 2 + len)?;
    *at += 2 + len;
    Some(value)
}

pub fn take<const N: usize>(bytes: &[u8], at: &mut usize) -> Option<[u8; N]> {
    let value: [u8; N] = bytes.get(*at..*at + N)?.try_into().ok()?;
    *at += N;
    Some(value)
}

pub fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Canonical unpadded base64url only: re-encoding must give `text` back.
pub fn unb64(text: &str) -> Option<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD.decode(text).ok()?;
    (URL_SAFE_NO_PAD.encode(&bytes) == text).then_some(bytes)
}

/// Exactly `N` bytes of canonical base64url.
pub fn unb64_array<const N: usize>(text: &str) -> Option<[u8; N]> {
    unb64(text)?.try_into().ok()
}

pub fn random<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    getrandom::fill(&mut out).expect("the system's random number generator");
    out
}

/// 16 random bytes as 22 base64url characters: share, link and comment IDs.
pub fn new_id() -> String {
    b64(&random::<16>())
}

/// Whether `id` is 16 bytes of canonical base64url.
pub fn is_id(id: &str) -> bool {
    id.len() == 22 && unb64(id).is_some_and(|bytes| bytes.len() == 16)
}

pub fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part);
    }
    hash.finalize().into()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// HKDF-SHA256, 32 bytes.
pub fn hkdf32(ikm: &[u8], salt: Option<&[u8]>, info: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    hkdf::Hkdf::<Sha256>::new(salt, ikm)
        .expand(info, &mut out[..])
        .expect("HKDF-SHA256 gives 32 bytes");
    out
}

/// `0x01 || nonce(24) || XChaCha20-Poly1305(key, nonce, aad, plaintext)`.
pub fn seal(key: &[u8; 32], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let nonce = random::<NONCE_LEN>();
    let mut out = vec![FORMAT_V1];
    out.extend_from_slice(&nonce);
    out.extend(aead_seal(key, &nonce, aad, plaintext));
    out
}

/// Opens [`seal`]'s output; `None` for anything that isn't exactly that.
pub fn open(key: &[u8; 32], aad: &[u8], sealed: &[u8]) -> Option<Zeroizing<Vec<u8>>> {
    if sealed.len() < 1 + NONCE_LEN + TAG_LEN || sealed[0] != FORMAT_V1 {
        return None;
    }
    let nonce: [u8; NONCE_LEN] = sealed[1..1 + NONCE_LEN].try_into().ok()?;
    aead_open(key, &nonce, aad, &sealed[1 + NONCE_LEN..])
}

pub fn aead_seal(key: &[u8; 32], nonce: &[u8; NONCE_LEN], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    XChaCha20Poly1305::new(key.into())
        .encrypt(
            &XNonce::from(*nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("XChaCha20-Poly1305 seals any length this crate uses")
}

pub fn aead_open(
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    aad: &[u8],
    sealed: &[u8],
) -> Option<Zeroizing<Vec<u8>>> {
    XChaCha20Poly1305::new(key.into())
        .decrypt(&XNonce::from(*nonce), Payload { msg: sealed, aad })
        .ok()
        .map(Zeroizing::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(crate::tests::config(256))]

        /// Round trip: `lp` reads back what it wrote, and nothing more.
        #[test]
        fn prop_lp_round_trips(parts in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 0..6)) {
            let mut out = Vec::new();
            for part in &parts {
                put_lp(&mut out, part);
            }
            let mut at = 0;
            for part in &parts {
                prop_assert_eq!(take_lp(&out, &mut at), Some(&part[..]));
            }
            prop_assert_eq!(at, out.len());
            prop_assert_eq!(take_lp(&out, &mut at), None);
        }

        /// Round trip and integrity: a seal opens under its key and AAD
        /// only, and any flipped byte stops it opening.
        #[test]
        fn prop_seal_opens_only_unchanged(key in any::<[u8; 32]>(), aad in prop::collection::vec(any::<u8>(), 0..32),
                                          plain in prop::collection::vec(any::<u8>(), 0..128), flip in any::<prop::sample::Index>()) {
            let sealed = seal(&key, &aad, &plain);
            prop_assert_eq!(open(&key, &aad, &sealed).map(|p| p.to_vec()), Some(plain.clone()));
            let mut bad = sealed.clone();
            let i = flip.index(bad.len());
            bad[i] ^= 0x01;
            prop_assert!(open(&key, &aad, &bad).is_none());
            let mut other_aad = aad.clone();
            other_aad.push(0);
            prop_assert!(open(&key, &other_aad, &sealed).is_none());
        }

        /// Canonical: only the encoding `b64` writes is read back.
        #[test]
        fn prop_unb64_is_canonical(bytes in prop::collection::vec(any::<u8>(), 0..40)) {
            let text = b64(&bytes);
            prop_assert_eq!(unb64(&text), Some(bytes));
        }
    }
}
