//! The envelope everything written to a shared note travels in
//! (docs/ACCOUNTS.md › Collaboration › Updates, snapshots, comments and
//! presence):
//!
//! ```text
//! header   = 0x01 || kind || u32(epoch) || lp(author_account) || lp(author_device) || u64(counter)
//! sealed   = XChaCha20-Poly1305(note_key[epoch], nonce(24), aad, plaintext)
//! aad      = "kanso-collab" || 0x01 || lp(share_id) || header   (|| u64(base_seq) for a snapshot)
//! signature= Ed25519(author sign, "kanso-collab-sig" || 0x01 || lp(share_id) || header || nonce || sealed)
//! envelope = header || nonce || sealed || signature
//! ```
//!
//! The server reads the header only ([`parse_header`], which the Worker
//! mirrors and `cloud/test/fixtures/collab.json` pins); members open and
//! verify the rest.

use zeroize::Zeroizing;

use crate::codec::{
    FORMAT_V1, NONCE_LEN, TAG_LEN, aead_open, aead_seal, put_lp, random, take, take_lp,
};
use crate::identity::{Identity, verify};
use crate::keys::NoteKey;
use crate::model::{Header, Kind};

pub const SIGNATURE_LEN: usize = 64;

/// Size limits per kind, as the server enforces them.
pub const MAX_UPDATE: usize = 256 * 1024;
pub const MAX_SNAPSHOT: usize = 4 * 1024 * 1024;
pub const MAX_COMMENT: usize = 16 * 1024;
pub const MAX_PRESENCE: usize = 2 * 1024;

pub fn max_size(kind: Kind) -> usize {
    match kind {
        Kind::Update => MAX_UPDATE,
        Kind::Snapshot => MAX_SNAPSHOT,
        Kind::Comment => MAX_COMMENT,
        Kind::Presence => MAX_PRESENCE,
    }
}

pub fn encode_header(header: &Header) -> Vec<u8> {
    let mut out = vec![FORMAT_V1, header.kind.byte()];
    out.extend_from_slice(&header.epoch.to_be_bytes());
    put_lp(&mut out, header.author_account.as_bytes());
    put_lp(&mut out, header.author_device.as_bytes());
    out.extend_from_slice(&header.counter.to_be_bytes());
    out
}

/// Whether `text` is `<prefix>_` and 32 lowercase hex digits: an account
/// ID (`a`) or a device ID (`d`).
fn is_hex_id(text: &str, prefix: u8) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 34
        && bytes[0] == prefix
        && bytes[1] == b'_'
        && bytes[2..]
            .iter()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The header of an envelope and where it ends; `None` unless the envelope
/// has a well-formed header (epoch 1 or later, the author an account ID and
/// the device a device ID) followed by room for a nonce, a tag and a
/// signature.
pub fn parse_header(envelope: &[u8]) -> Option<(Header, usize)> {
    let mut at = 0;
    let [format, kind] = take::<2>(envelope, &mut at)?;
    if format != FORMAT_V1 {
        return None;
    }
    let kind = Kind::from_byte(kind)?;
    let epoch = u32::from_be_bytes(take::<4>(envelope, &mut at)?);
    let author_account = std::str::from_utf8(take_lp(envelope, &mut at)?)
        .ok()?
        .to_owned();
    let author_device = std::str::from_utf8(take_lp(envelope, &mut at)?)
        .ok()?
        .to_owned();
    let counter = u64::from_be_bytes(take::<8>(envelope, &mut at)?);
    if epoch == 0
        || !is_hex_id(&author_account, b'a')
        || !is_hex_id(&author_device, b'd')
        || envelope.len() < at + NONCE_LEN + TAG_LEN + SIGNATURE_LEN
    {
        return None;
    }
    Some((
        Header {
            kind,
            epoch,
            author_account,
            author_device,
            counter,
        },
        at,
    ))
}

fn aad(share_id: &str, header: &[u8], base_seq: Option<u64>) -> Vec<u8> {
    let mut aad = b"kanso-collab".to_vec();
    aad.push(FORMAT_V1);
    put_lp(&mut aad, share_id.as_bytes());
    aad.extend_from_slice(header);
    if let Some(seq) = base_seq {
        aad.extend_from_slice(&seq.to_be_bytes());
    }
    aad
}

fn signed_message(share_id: &str, header: &[u8], nonce: &[u8], sealed: &[u8]) -> Vec<u8> {
    let mut message = b"kanso-collab-sig".to_vec();
    message.push(FORMAT_V1);
    put_lp(&mut message, share_id.as_bytes());
    message.extend_from_slice(header);
    message.extend_from_slice(nonce);
    message.extend_from_slice(sealed);
    message
}

/// Seals and signs `plaintext`. `base_seq` is given for snapshots only.
pub fn seal(
    author: &Identity,
    key: &NoteKey,
    share_id: &str,
    header: &Header,
    base_seq: Option<u64>,
    plaintext: &[u8],
) -> Vec<u8> {
    let head = encode_header(header);
    let nonce = random::<NONCE_LEN>();
    let sealed = aead_seal(
        key.bytes(),
        &nonce,
        &aad(share_id, &head, base_seq),
        plaintext,
    );
    let signature = author.sign(&signed_message(share_id, &head, &nonce, &sealed));
    let mut out = head;
    out.extend_from_slice(&nonce);
    out.extend(sealed);
    out.extend_from_slice(&signature);
    out
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum OpenError {
    #[error("the envelope isn't well-formed")]
    Malformed,
    #[error("the envelope's signature doesn't verify under its author's key")]
    BadSignature,
    #[error("the envelope doesn't open with the note's key")]
    Undecryptable,
}

/// Opens an envelope: its header, then its signature under `author_sign_pub`
/// (the caller looks the author up from the header in the epoch's member
/// list), then its plaintext.
pub fn open(
    envelope: &[u8],
    share_id: &str,
    base_seq: Option<u64>,
    author_sign_pub: &[u8; 32],
    key: &NoteKey,
) -> Result<(Header, Zeroizing<Vec<u8>>), OpenError> {
    let (header, end) = parse_header(envelope).ok_or(OpenError::Malformed)?;
    let head = &envelope[..end];
    let nonce: [u8; NONCE_LEN] = envelope[end..end + NONCE_LEN]
        .try_into()
        .map_err(|_| OpenError::Malformed)?;
    let sig_at = envelope.len() - SIGNATURE_LEN;
    let sealed = &envelope[end + NONCE_LEN..sig_at];
    let signature: [u8; SIGNATURE_LEN] = envelope[sig_at..]
        .try_into()
        .map_err(|_| OpenError::Malformed)?;
    if !verify(
        author_sign_pub,
        &signed_message(share_id, head, &nonce, sealed),
        &signature,
    ) {
        return Err(OpenError::BadSignature);
    }
    let plain = aead_open(key.bytes(), &nonce, &aad(share_id, head, base_seq), sealed)
        .ok_or(OpenError::Undecryptable)?;
    Ok((header, plain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn header(kind: Kind, epoch: u32, counter: u64) -> Header {
        Header {
            kind,
            epoch,
            author_account: "a_0123456789abcdef0123456789abcdef".into(),
            author_device: "d_fedcba9876543210fedcba9876543210".into(),
            counter,
        }
    }

    fn kinds() -> impl Strategy<Value = Kind> {
        prop::sample::select(vec![
            Kind::Update,
            Kind::Snapshot,
            Kind::Comment,
            Kind::Presence,
        ])
    }

    proptest! {
        #![proptest_config(crate::tests::config(64))]

        /// Round trip: an envelope opens to its plaintext and header under
        /// its author's key and the note's key.
        #[test]
        fn prop_envelopes_round_trip(kind in kinds(), epoch in 1u32..,  counter in any::<u64>(),
                                     plain in prop::collection::vec(any::<u8>(), 0..256), base in any::<u64>()) {
            let author = Identity::generate();
            let key = NoteKey::generate();
            let h = header(kind, epoch, counter);
            let base = (kind == Kind::Snapshot).then_some(base);
            let env = seal(&author, &key, "s", &h, base, &plain);
            prop_assert_eq!(parse_header(&env).map(|(h, _)| h), Some(h.clone()));
            let (opened, text) = open(&env, "s", base, &author.sign_pub(), &key).unwrap();
            prop_assert_eq!(opened, h);
            prop_assert_eq!(text.to_vec(), plain);
        }

        /// Authenticity: any changed byte, another author's key, another
        /// note's key or another note's ID, and it doesn't open.
        #[test]
        fn prop_tampering_is_refused(plain in prop::collection::vec(any::<u8>(), 1..64), flip in any::<prop::sample::Index>()) {
            let author = Identity::generate();
            let key = NoteKey::generate();
            let env = seal(&author, &key, "s", &header(Kind::Update, 3, 9), None, &plain);
            let mut bad = env.clone();
            let i = flip.index(bad.len());
            bad[i] ^= 0x01;
            prop_assert!(open(&bad, "s", None, &author.sign_pub(), &key).is_err());
            prop_assert_eq!(open(&env, "s", None, &Identity::generate().sign_pub(), &key), Err(OpenError::BadSignature));
            prop_assert_eq!(open(&env, "s", None, &author.sign_pub(), &NoteKey::generate()), Err(OpenError::Undecryptable));
            prop_assert!(open(&env, "t", None, &author.sign_pub(), &key).is_err());
        }

        /// The header parser never panics, and accepts only what it can
        /// encode again byte for byte.
        #[test]
        fn prop_header_parser_is_total(bytes in prop::collection::vec(any::<u8>(), 0..160)) {
            if let Some((h, end)) = parse_header(&bytes) {
                prop_assert_eq!(&encode_header(&h)[..], &bytes[..end]);
                prop_assert!(bytes.len() >= end + NONCE_LEN + TAG_LEN + SIGNATURE_LEN);
            }
        }
    }

    /// The Worker's header cases (`cloud/test/fixtures/collab.json`): both
    /// parsers agree on every one.
    #[test]
    fn the_workers_header_cases_parse_alike() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../cloud/test/fixtures/collab.json"
        );
        let fixtures: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let cases = fixtures["cases"].as_array().unwrap();
        assert!(cases.len() >= 20);
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let parsed = parse_header(&bytes).map(|(h, _)| h);
            let name = case["name"].as_str().unwrap();
            if case["ok"].as_bool().unwrap() {
                let h = parsed.unwrap_or_else(|| panic!("{name}"));
                assert_eq!(
                    h.kind.byte(),
                    case["kind"].as_str().unwrap().as_bytes()[0],
                    "{name}"
                );
                assert_eq!(
                    u64::from(h.epoch),
                    case["epoch"].as_u64().unwrap(),
                    "{name}"
                );
                assert_eq!(h.author_account, case["author"].as_str().unwrap(), "{name}");
                assert_eq!(h.author_device, case["device"].as_str().unwrap(), "{name}");
                assert_eq!(
                    h.counter.to_string(),
                    case["counter"].as_str().unwrap(),
                    "{name}"
                );
            } else {
                assert_eq!(parsed, None, "{name}");
            }
        }
    }

    #[test]
    fn a_snapshot_binds_its_base() {
        let author = Identity::generate();
        let key = NoteKey::generate();
        let env = seal(
            &author,
            &key,
            "s",
            &header(Kind::Snapshot, 1, 1),
            Some(7),
            b"state",
        );
        assert!(open(&env, "s", Some(7), &author.sign_pub(), &key).is_ok());
        assert_eq!(
            open(&env, "s", Some(8), &author.sign_pub(), &key),
            Err(OpenError::Undecryptable)
        );
    }
}
