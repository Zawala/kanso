//! Published notes (docs/ACCOUNTS.md › Published notes): a note's web page,
//! sealed on this device so that anyone with its link can read it and the
//! server can't.
//!
//! Each share has a random 128-bit ID and its own random 256-bit key. The
//! page is sealed as `0x01 || nonce(12) || AES-256-GCM(key, nonce, aad,
//! html)`, with a fresh random nonce for every upload and `aad = "kanso-share"
//! || 0x01 || share_id`, and uploaded to `PUT /v2/shares/<id>`. The link is
//! `<server>/p/<id>#<key>`: browsers never send the #fragment, so the key
//! never reaches the server, and the viewer page opens the page with
//! WebCrypto.
//!
//! What a note publishes is kept in the note, as `note.extra.kansoPublish`
//! ([`KansoPublish`]), so it syncs end to end encrypted with the note and
//! the account's other devices can update or unpublish the same link. Inkdrop's
//! own `share` field is left alone.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::time::Duration;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use ureq::Agent;
use zeroize::Zeroizing;

use crate::account::Session;
use crate::crypto::{b64, random, sha256_hex, unb64};
use crate::remote::encode_segment;
use crate::wire::{ApiError, MAX_AUTH_RESPONSE, SessionToken};

/// Where a note keeps what it published, beside Inkdrop's own fields.
pub const PUBLISH_KEY: &str = kanso_core::model::PUBLISH_FIELD;
/// The only format so far.
pub const SHARE_FORMAT_V1: u8 = 0x01;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
/// What sealing adds: the format byte, the nonce and the tag.
pub const SHARE_OVERHEAD: usize = 1 + NONCE_LEN + TAG_LEN;
/// The largest page that can be published: 10 MiB (the Worker's cap less
/// [`SHARE_OVERHEAD`]).
pub const MAX_SHARE_PLAINTEXT: usize = 10 * 1024 * 1024;
const AAD_PREFIX: &[u8] = b"kanso-share";

/// How long a published note stays readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Expiry {
    Never,
    Day,
    Week,
    Month,
}

impl Expiry {
    pub const ALL: [Expiry; 4] = [Expiry::Never, Expiry::Day, Expiry::Week, Expiry::Month];

    /// "Never", "1 day", "7 days", "30 days".
    pub fn label(self) -> &'static str {
        match self {
            Expiry::Never => "Never",
            Expiry::Day => "1 day",
            Expiry::Week => "7 days",
            Expiry::Month => "30 days",
        }
    }

    /// When a note published at `now_ms` stops being served. Pure.
    pub fn at(self, now_ms: i64) -> Option<i64> {
        const DAY: i64 = 86_400_000;
        let days = match self {
            Expiry::Never => return None,
            Expiry::Day => 1,
            Expiry::Week => 7,
            Expiry::Month => 30,
        };
        Some(now_ms + days * DAY)
    }
}

// ---------------------------------------------------------------------------
// Keys and IDs
// ---------------------------------------------------------------------------

/// A share's 256-bit key. Zeroized on drop; its `Debug` is redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct ShareKey(Zeroizing<[u8; 32]>);

impl ShareKey {
    /// A fresh key from the system's random source.
    pub fn generate() -> ShareKey {
        ShareKey(Zeroizing::new(random::<32>()))
    }

    pub fn from_bytes(bytes: [u8; 32]) -> ShareKey {
        ShareKey(Zeroizing::new(bytes))
    }

    /// The key as it travels in the link and the note: 43 characters of
    /// canonical unpadded base64url.
    pub fn parse(text: &str) -> Option<ShareKey> {
        let bytes = unb64(text)?;
        let bytes: [u8; 32] = bytes.try_into().ok()?;
        Some(ShareKey::from_bytes(bytes))
    }

    pub fn to_text(&self) -> String {
        b64(&self.0[..])
    }
}

impl fmt::Debug for ShareKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ShareKey(<redacted>)")
    }
}

/// A fresh share ID: 16 random bytes as 22 characters of unpadded base64url.
pub fn new_share_id() -> String {
    b64(&random::<16>())
}

/// Whether `id` is a share ID: 16 bytes of canonical unpadded base64url.
pub fn is_share_id(id: &str) -> bool {
    id.len() == 22 && unb64(id).is_some_and(|bytes| bytes.len() == 16)
}

/// `"kanso-share" || 0x01 || share_id`: binds the ciphertext to its ID, so
/// the server can't serve one share's bytes under another's link.
fn aad(share_id: &str) -> Vec<u8> {
    let mut aad = Vec::with_capacity(AAD_PREFIX.len() + 1 + share_id.len());
    aad.extend_from_slice(AAD_PREFIX);
    aad.push(SHARE_FORMAT_V1);
    aad.extend_from_slice(share_id.as_bytes());
    aad
}

// ---------------------------------------------------------------------------
// Sealing
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ShareError {
    /// The page is over [`MAX_SHARE_PLAINTEXT`].
    #[error("the note is too large to publish ({0} bytes; the limit is 10 MB)")]
    TooLarge(usize),
    /// The bytes aren't a sealed share this key opens.
    #[error("the published note couldn't be opened")]
    Undecryptable,
    /// The server refused or couldn't be reached.
    #[error(transparent)]
    Api(#[from] ApiError),
}

/// Seals `plaintext` for `share_id` with a fresh random nonce.
pub fn seal_share(key: &ShareKey, share_id: &str, plaintext: &[u8]) -> Result<Vec<u8>, ShareError> {
    if plaintext.len() > MAX_SHARE_PLAINTEXT {
        return Err(ShareError::TooLarge(plaintext.len()));
    }
    Ok(seal_share_with_nonce(
        key,
        share_id,
        random::<NONCE_LEN>(),
        plaintext,
    ))
}

/// [`seal_share`] with a given nonce: only for fixed test vectors. A nonce
/// must never be used twice with one key.
#[doc(hidden)]
pub fn seal_share_with_nonce(
    key: &ShareKey,
    share_id: &str,
    nonce: [u8; NONCE_LEN],
    plaintext: &[u8],
) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(&key.0[..]).expect("a 32-byte key");
    let aad = aad(share_id);
    let ciphertext = cipher
        .encrypt(
            &Nonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .expect("AES-GCM encrypts any message under 64 GiB");
    let mut sealed = Vec::with_capacity(SHARE_OVERHEAD + plaintext.len());
    sealed.push(SHARE_FORMAT_V1);
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&ciphertext);
    sealed
}

/// Opens what [`seal_share`] sealed, as the viewer page does.
pub fn open_share(key: &ShareKey, share_id: &str, sealed: &[u8]) -> Result<Vec<u8>, ShareError> {
    if sealed.len() < SHARE_OVERHEAD || sealed[0] != SHARE_FORMAT_V1 {
        return Err(ShareError::Undecryptable);
    }
    let nonce: [u8; NONCE_LEN] = sealed[1..1 + NONCE_LEN].try_into().expect("12 bytes");
    let cipher = Aes256Gcm::new_from_slice(&key.0[..]).expect("a 32-byte key");
    let aad = aad(share_id);
    cipher
        .decrypt(
            &Nonce::from(nonce),
            Payload {
                msg: &sealed[1 + NONCE_LEN..],
                aad: &aad,
            },
        )
        .map_err(|_| ShareError::Undecryptable)
}

// ---------------------------------------------------------------------------
// What the note keeps
// ---------------------------------------------------------------------------

/// `note.extra.kansoPublish`: the link's ID and key, and what was published
/// when. It travels inside the sealed note, so only the account's devices see
/// the key. Unknown keys are kept, for later versions.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KansoPublish {
    pub share_id: String,
    /// The share's key as base64url ([`ShareKey::to_text`]).
    pub key: String,
    /// Unix ms of the last upload.
    pub published_at: i64,
    /// The note's revision the published page was made from.
    pub published_rev: String,
    /// Unix ms when the link stops working; absent for never.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// [`content_hash`] of the published title and body, so any device can
    /// tell the note changed since (the revision moves with every save,
    /// this one included).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

impl fmt::Debug for KansoPublish {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KansoPublish")
            .field("share_id", &self.share_id)
            .field("key", &"<redacted>")
            .field("published_at", &self.published_at)
            .field("published_rev", &self.published_rev)
            .field("expires_at", &self.expires_at)
            .field("content_hash", &self.content_hash)
            .finish_non_exhaustive()
    }
}

impl KansoPublish {
    /// What a note's extra fields say it published, when that is usable: a
    /// share ID and a key of the right shapes.
    pub fn from_extra(extra: &Map<String, Value>) -> Option<KansoPublish> {
        let publish: KansoPublish = serde_json::from_value(extra.get(PUBLISH_KEY)?.clone()).ok()?;
        (is_share_id(&publish.share_id) && ShareKey::parse(&publish.key).is_some())
            .then_some(publish)
    }

    /// Keeps it in a note's extra fields, or (with `None`) takes it out.
    pub fn store(publish: Option<&KansoPublish>, extra: &mut Map<String, Value>) {
        match publish.and_then(|p| serde_json::to_value(p).ok()) {
            Some(value) => {
                extra.insert(PUBLISH_KEY.into(), value);
            }
            None => {
                extra.remove(PUBLISH_KEY);
            }
        }
    }

    pub fn share_key(&self) -> Option<ShareKey> {
        ShareKey::parse(&self.key)
    }

    /// `<server>/p/<id>#<key>`.
    pub fn link(&self, server: &str) -> String {
        share_link(server, &self.share_id, &self.key)
    }

    /// Whether the note's title or body differ from what was published.
    pub fn is_outdated(&self, title: &str, body: &str) -> bool {
        match &self.content_hash {
            Some(hash) => *hash != content_hash(title, body),
            None => true,
        }
    }

    /// Whether the link has stopped working at `now_ms`.
    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at.is_some_and(|at| at <= now_ms)
    }
}

/// `<server>/p/<id>#<key>`. Pure.
pub fn share_link(server: &str, share_id: &str, key: &str) -> String {
    format!("{}/p/{share_id}#{key}", server.trim_end_matches('/'))
}

/// The first 16 bytes of SHA-256 over the length-prefixed title and the
/// body, as hex. Pure.
pub fn content_hash(title: &str, body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update((title.len() as u64).to_be_bytes());
    hasher.update(title.as_bytes());
    hasher.update(body.as_bytes());
    hasher.finalize()[..16]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---------------------------------------------------------------------------
// The server
// ---------------------------------------------------------------------------

/// The share routes, on the session's server with its token.
pub trait ShareApi: Send + Sync {
    /// `PUT /v2/shares/<id>` with `x-kanso-sha256` and, when it expires,
    /// `x-kanso-expires-at`.
    fn put_share(
        &self,
        session: &Session,
        share_id: &str,
        sealed: &[u8],
        expires_at: Option<i64>,
    ) -> Result<(), ApiError>;

    /// `DELETE /v2/shares/<id>`; [`ApiError::NotFound`] when there is no
    /// such share of this account.
    fn delete_share(&self, session: &Session, share_id: &str) -> Result<(), ApiError>;
}

fn share_url(session: &Session, share_id: &str) -> String {
    format!(
        "{}/v2/shares/{}",
        session.server.trim_end_matches('/'),
        encode_segment(share_id)
    )
}

fn bearer(token: &SessionToken) -> Zeroizing<String> {
    Zeroizing::new(format!("Bearer {}", token.expose()))
}

/// [`ShareApi`] over HTTPS, as [`crate::HttpBackend`] talks to the Worker:
/// no redirects, no `Origin`, capped answers.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpShares;

impl HttpShares {
    fn agent() -> Agent {
        Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_global(Some(Duration::from_secs(300)))
            .user_agent(concat!("kanso-sync/", env!("CARGO_PKG_VERSION")))
            .build()
            .into()
    }

    fn answer(
        response: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    ) -> Result<(), ApiError> {
        let mut response = response.map_err(|err| ApiError::Transport(err.to_string()))?;
        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            return Ok(());
        }
        if (300..400).contains(&status) {
            return Err(ApiError::Protocol(format!(
                "the server answered {status} (a redirect), which the client never follows"
            )));
        }
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_AUTH_RESPONSE)
            .read_to_vec()
            .map_err(|err| ApiError::Transport(err.to_string()))?;
        Err(ApiError::from_response(status, &body))
    }
}

impl ShareApi for HttpShares {
    fn put_share(
        &self,
        session: &Session,
        share_id: &str,
        sealed: &[u8],
        expires_at: Option<i64>,
    ) -> Result<(), ApiError> {
        let auth = bearer(session.token());
        let mut request = HttpShares::agent()
            .put(share_url(session, share_id))
            .header("authorization", auth.as_str())
            .header("content-type", "application/octet-stream")
            .header("x-kanso-sha256", sha256_hex(sealed));
        if let Some(at) = expires_at {
            request = request.header("x-kanso-expires-at", at.to_string());
        }
        HttpShares::answer(request.send(sealed))
    }

    fn delete_share(&self, session: &Session, share_id: &str) -> Result<(), ApiError> {
        let auth = bearer(session.token());
        HttpShares::answer(
            HttpShares::agent()
                .delete(share_url(session, share_id))
                .header("authorization", auth.as_str())
                .call(),
        )
    }
}

/// One share as [`MemoryShares`] holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredShare {
    /// The account that published it.
    pub owner: String,
    pub sealed: Vec<u8>,
    pub expires_at: Option<i64>,
}

/// [`ShareApi`] in memory, for tests: the Worker's rules (owner only; an
/// ID that stopped being served is retired for good, for every account)
/// without a server.
#[derive(Debug, Default)]
pub struct MemoryShares {
    shares: Mutex<HashMap<String, StoredShare>>,
    retired: Mutex<Vec<String>>,
    /// The next this many requests fail as if offline.
    offline: Mutex<usize>,
}

impl MemoryShares {
    pub fn new() -> MemoryShares {
        MemoryShares::default()
    }

    pub fn get(&self, share_id: &str) -> Option<StoredShare> {
        self.shares.lock().unwrap().get(share_id).cloned()
    }

    pub fn len(&self) -> usize {
        self.shares.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Stops serving a share as the server does when it expires or the
    /// owner of the server takes it down: the share goes and its ID is
    /// retired for good. False when there was no such share.
    pub fn take_down(&self, share_id: &str) -> bool {
        let gone = self.shares.lock().unwrap().remove(share_id).is_some();
        if gone {
            self.retired.lock().unwrap().push(share_id.to_owned());
        }
        gone
    }

    /// Fails the next `n` requests with a transport error.
    pub fn go_offline(&self, n: usize) {
        *self.offline.lock().unwrap() = n;
    }

    fn offline(&self) -> Result<(), ApiError> {
        let mut left = self.offline.lock().unwrap();
        if *left > 0 {
            *left -= 1;
            return Err(ApiError::Transport("offline".into()));
        }
        Ok(())
    }
}

impl ShareApi for MemoryShares {
    fn put_share(
        &self,
        session: &Session,
        share_id: &str,
        sealed: &[u8],
        expires_at: Option<i64>,
    ) -> Result<(), ApiError> {
        self.offline()?;
        if !is_share_id(share_id) || sealed.len() < SHARE_OVERHEAD || sealed[0] != SHARE_FORMAT_V1 {
            return Err(ApiError::BadRequest);
        }
        if sealed.len() > MAX_SHARE_PLAINTEXT + SHARE_OVERHEAD {
            return Err(ApiError::TooLarge);
        }
        let mut shares = self.shares.lock().unwrap();
        if self.retired.lock().unwrap().iter().any(|id| id == share_id) {
            return Err(ApiError::Unrecognized {
                status: 409,
                code: "share_retired".into(),
            });
        }
        if shares
            .get(share_id)
            .is_some_and(|s| s.owner != session.account_id)
        {
            return Err(ApiError::Unrecognized {
                status: 409,
                code: "share_id_taken".into(),
            });
        }
        shares.insert(
            share_id.to_owned(),
            StoredShare {
                owner: session.account_id.clone(),
                sealed: sealed.to_vec(),
                expires_at,
            },
        );
        Ok(())
    }

    fn delete_share(&self, session: &Session, share_id: &str) -> Result<(), ApiError> {
        self.offline()?;
        let mut shares = self.shares.lock().unwrap();
        match shares.get(share_id) {
            Some(share) if share.owner == session.account_id => {
                shares.remove(share_id);
                self.retired.lock().unwrap().push(share_id.to_owned());
                Ok(())
            }
            _ => Err(ApiError::NotFound),
        }
    }
}

// ---------------------------------------------------------------------------
// Publishing
// ---------------------------------------------------------------------------

/// A page to publish: the note's exported HTML, and what it was made from.
#[derive(Clone, Debug)]
pub struct Page<'a> {
    pub html: &'a [u8],
    pub title: &'a str,
    pub body: &'a str,
    /// The note's revision the page was made from.
    pub rev: &'a str,
}

/// Publishes `page`, or updates the note's existing link: the same ID and
/// key with new ciphertext under a fresh nonce.
///
/// A link that has stopped working is never revived. One past its expiry is
/// left alone and the note is published under a new ID and key. The server
/// retires every ID it stops serving (unpublished from another device,
/// expired, taken down) and answers 409 for it, as for another account's
/// ID; then too the note gets a new ID and key, so whoever holds the old
/// link can never read the note's later versions. Returns what the note
/// should now keep.
pub fn publish(
    api: &dyn ShareApi,
    session: &Session,
    existing: Option<&KansoPublish>,
    page: &Page,
    expiry: Option<i64>,
    now_ms: i64,
) -> Result<KansoPublish, ShareError> {
    if page.html.len() > MAX_SHARE_PLAINTEXT {
        return Err(ShareError::TooLarge(page.html.len()));
    }
    let live = existing.filter(|p| !p.is_expired(now_ms));
    let reuse = live.and_then(|p| Some((p.share_id.clone(), p.share_key()?, p.other.clone())));
    let (mut share_id, mut key, other) = match reuse {
        Some((id, key, other)) if is_share_id(&id) => (id, key, other),
        _ => (new_share_id(), ShareKey::generate(), Map::new()),
    };
    let mut fresh = existing.is_none_or(|p| p.share_id != share_id);
    loop {
        let sealed = seal_share(&key, &share_id, page.html)?;
        match api.put_share(session, &share_id, &sealed, expiry) {
            Ok(()) => break,
            Err(ApiError::Unrecognized { status: 409, .. }) if !fresh => {
                share_id = new_share_id();
                key = ShareKey::generate();
                fresh = true;
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(KansoPublish {
        share_id,
        key: key.to_text(),
        published_at: now_ms,
        published_rev: page.rev.to_owned(),
        expires_at: expiry,
        content_hash: Some(content_hash(page.title, page.body)),
        other,
    })
}

/// Unpublishes the note's link. A link the server no longer has (already
/// unpublished, expired, or the account was deleted) counts as done.
pub fn unpublish(
    api: &dyn ShareApi,
    session: &Session,
    publish: &KansoPublish,
) -> Result<(), ShareError> {
    match api.delete_share(session, &publish.share_id) {
        Ok(()) | Err(ApiError::NotFound) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use std::collections::HashSet;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn key_of(bytes: [u8; 32]) -> ShareKey {
        ShareKey::from_bytes(bytes)
    }

    fn share_id_of(bytes: [u8; 16]) -> String {
        b64(&bytes)
    }

    /// The vector the Worker's tests open with the viewer's WebCrypto code.
    #[test]
    fn matches_the_shared_fixture() {
        let text = std::fs::read_to_string(format!(
            "{}/../../cloud/test/fixtures/share.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let fixture: Value = serde_json::from_str(&text).unwrap();
        let s = |k: &str| fixture[k].as_str().unwrap().to_owned();
        let unhex = |h: String| -> Vec<u8> {
            (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect()
        };
        let key = ShareKey::parse(&s("key")).unwrap();
        let nonce: [u8; 12] = unhex(s("nonce")).try_into().unwrap();
        let sealed = seal_share_with_nonce(&key, &s("share_id"), nonce, s("plaintext").as_bytes());
        assert_eq!(sealed, unhex(s("sealed")));
        assert_eq!(
            open_share(&key, &s("share_id"), &sealed).unwrap(),
            s("plaintext").as_bytes()
        );
        assert!(is_share_id(&s("share_id")));
    }

    #[test]
    fn examples() {
        let key = ShareKey::generate();
        let id = new_share_id();
        assert!(is_share_id(&id));
        assert_eq!(ShareKey::parse(&key.to_text()), Some(key.clone()));
        assert_eq!(key.to_text().len(), 43);
        assert_eq!(format!("{key:?}"), "ShareKey(<redacted>)");
        assert!(!is_share_id("short"));
        assert!(!is_share_id("AAAAAAAAAAAAAAAAAAAAAB"), "non-canonical");
        assert!(!is_share_id("AAAAAAAAAAAAAAAAAAAA+A"));
        let sealed = seal_share(&key, &id, b"<p>Hi</p>").unwrap();
        assert_eq!(sealed.len(), SHARE_OVERHEAD + 9);
        assert_eq!(sealed[0], SHARE_FORMAT_V1);
        assert!(!sealed.windows(9).any(|w| w == b"<p>Hi</p>"));
        assert!(matches!(
            seal_share(&key, &id, &vec![0; MAX_SHARE_PLAINTEXT + 1]),
            Err(ShareError::TooLarge(_))
        ));
        assert!(matches!(
            open_share(&key, &id, &sealed[..SHARE_OVERHEAD - 1]),
            Err(ShareError::Undecryptable)
        ));
        assert_eq!(
            share_link("https://kanso.example/", "ID", "KEY"),
            "https://kanso.example/p/ID#KEY"
        );
        assert_eq!(Expiry::Never.at(5), None);
        assert_eq!(Expiry::Week.at(0), Some(7 * 86_400_000));
        assert_eq!(
            Expiry::ALL.map(Expiry::label),
            ["Never", "1 day", "7 days", "30 days"]
        );
    }

    #[test]
    fn a_note_keeps_its_publish_beside_other_fields() {
        let publish = KansoPublish {
            share_id: new_share_id(),
            key: ShareKey::generate().to_text(),
            published_at: 1,
            published_rev: "3-abc".into(),
            expires_at: None,
            content_hash: Some(content_hash("T", "B")),
            other: Map::new(),
        };
        let mut extra = Map::new();
        extra.insert("share".into(), "public".into());
        KansoPublish::store(Some(&publish), &mut extra);
        assert_eq!(KansoPublish::from_extra(&extra), Some(publish.clone()));
        assert_eq!(
            extra["share"], "public",
            "Inkdrop's share field is untouched"
        );
        let value = &extra[PUBLISH_KEY];
        assert!(value.get("shareId").is_some() && value.get("publishedRev").is_some());
        assert!(value.get("expiresAt").is_none());
        assert!(!publish.is_outdated("T", "B"));
        assert!(publish.is_outdated("T", "B2"));
        assert!(!format!("{publish:?}").contains(&publish.key));
        KansoPublish::store(None, &mut extra);
        assert!(!extra.contains_key(PUBLISH_KEY));
        // Malformed IDs or keys don't count as published.
        let mut bad = serde_json::to_value(&publish).unwrap();
        bad["key"] = "not a key".into();
        let mut extra = Map::new();
        extra.insert(PUBLISH_KEY.into(), bad);
        assert_eq!(KansoPublish::from_extra(&extra), None);
    }

    fn gen_publish() -> impl Strategy<Value = KansoPublish> {
        (
            any::<[u8; 16]>(),
            any::<[u8; 32]>(),
            any::<i64>(),
            "[0-9]{1,4}-[0-9a-f]{1,32}",
            prop::option::of(any::<i64>()),
            prop::option::of("[0-9a-f]{32}"),
            prop::collection::btree_map("[a-z]{1,8}", any::<i32>(), 0..3),
        )
            .prop_map(|(id, key, at, rev, expires, hash, other)| KansoPublish {
                share_id: share_id_of(id),
                key: key_of(key).to_text(),
                published_at: at,
                published_rev: rev,
                expires_at: expires,
                content_hash: hash,
                other: other
                    .into_iter()
                    // Unknown keys only: a known one would be read as its field.
                    .map(|(k, v)| (format!("x{k}"), Value::from(v)))
                    .collect(),
            })
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Round trip: what a key seals for an ID, that key opens for that ID.
        #[test]
        fn prop_sealed_notes_open(key in any::<[u8; 32]>(), id in any::<[u8; 16]>(), page in prop::collection::vec(any::<u8>(), 0..2048)) {
            let (key, id) = (key_of(key), share_id_of(id));
            let sealed = seal_share(&key, &id, &page).unwrap();
            prop_assert_eq!(sealed.len(), page.len() + SHARE_OVERHEAD);
            prop_assert_eq!(open_share(&key, &id, &sealed).unwrap(), page);
        }

        /// Any other key, any other ID, or any flipped bit fails to open.
        #[test]
        fn prop_wrong_keys_ids_and_tampering_fail(
            key in any::<[u8; 32]>(), other in any::<[u8; 32]>(),
            id in any::<[u8; 16]>(), other_id in any::<[u8; 16]>(),
            page in prop::collection::vec(any::<u8>(), 0..512), bit in any::<usize>(),
        ) {
            let sealed = seal_share(&key_of(key), &share_id_of(id), &page).unwrap();
            if other != key {
                prop_assert!(open_share(&key_of(other), &share_id_of(id), &sealed).is_err());
            }
            if other_id != id {
                prop_assert!(open_share(&key_of(key), &share_id_of(other_id), &sealed).is_err());
            }
            let mut flipped = sealed.clone();
            let at = bit % (flipped.len() * 8);
            flipped[at / 8] ^= 1 << (at % 8);
            prop_assert!(open_share(&key_of(key), &share_id_of(id), &flipped).is_err());
        }

        /// Every upload gets its own nonce: sealing the same page with the
        /// same key many times never repeats one.
        #[test]
        fn prop_nonces_are_unique(key in any::<[u8; 32]>(), id in any::<[u8; 16]>(), page in prop::collection::vec(any::<u8>(), 0..64)) {
            let (key, id) = (key_of(key), share_id_of(id));
            let mut nonces = HashSet::new();
            for _ in 0..64 {
                let sealed = seal_share(&key, &id, &page).unwrap();
                prop_assert!(nonces.insert(sealed[1..13].to_vec()), "a nonce repeated");
            }
        }

        /// `kansoPublish` survives JSON, and a note's extra fields, unchanged,
        /// unknown keys included.
        #[test]
        fn prop_publish_round_trips(publish in gen_publish()) {
            let json = serde_json::to_string(&publish).unwrap();
            let back: KansoPublish = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(&back, &publish);
            let mut extra = Map::new();
            KansoPublish::store(Some(&publish), &mut extra);
            prop_assert_eq!(KansoPublish::from_extra(&extra), Some(publish));
        }

        /// IDs: fresh ones are valid; a valid ID is 16 bytes re-encoded the same.
        #[test]
        fn prop_share_ids_are_128_bits(text in "[A-Za-z0-9_-]{20,24}") {
            prop_assert!(is_share_id(&new_share_id()));
            if is_share_id(&text) {
                prop_assert_eq!(unb64(&text).unwrap().len(), 16);
                prop_assert_eq!(b64(&unb64(&text).unwrap()), text);
            }
        }

        /// The content hash changes with the title or the body, and the
        /// boundary between them can't be moved.
        #[test]
        fn prop_content_hash_tells_edits_apart(title in "\\PC{0,20}", body in "\\PC{0,40}", cut in 0usize..20) {
            prop_assert_eq!(content_hash(&title, &body), content_hash(&title, &body));
            let joined = format!("{title}{body}");
            let split = joined.char_indices().nth(cut).map_or(joined.len(), |(i, _)| i);
            let (t2, b2) = joined.split_at(split);
            if t2 != title {
                prop_assert_ne!(content_hash(t2, b2), content_hash(&title, &body));
            }
        }
    }
}
