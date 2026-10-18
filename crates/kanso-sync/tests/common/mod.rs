//! Helpers shared by the sync test suites.
#![allow(dead_code)]

pub mod legacy;

use std::collections::BTreeMap;

use base64::Engine as _;
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store, StoreError, WriteResult};
use serde_json::{Map, Value, json};

pub fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

pub fn put(store: &Store, doc_type: DocType, body: Value) -> Result<WriteResult, StoreError> {
    store.put(doc_type, body, Put::Upsert, Origin::Api)
}

pub fn merge(store: &Store, doc_type: DocType, body: Value) -> Result<WriteResult, StoreError> {
    store.put(doc_type, body, Put::Merge, Origin::Api)
}

pub fn book(store: &Store, id: &str, name: &str) -> WriteResult {
    put(
        store,
        DocType::Book,
        json!({"_id": id, "name": name, "createdAt": 1, "updatedAt": 1}),
    )
    .unwrap()
}

pub fn note(store: &Store, id: &str, book: &str, body: &str, updated: i64) -> WriteResult {
    put(
        store,
        DocType::Note,
        json!({"_id": id, "bookId": book, "title": id, "body": body, "createdAt": 1, "updatedAt": updated}),
    )
    .unwrap()
}

/// Replaces a note's body (full-document update with the current `_rev`).
pub fn edit_note(store: &Store, id: &str, body: &str, updated: Option<i64>) -> WriteResult {
    let mut doc = store.get_api(id, None, false).unwrap();
    doc["body"] = body.into();
    if let Some(updated) = updated {
        doc["updatedAt"] = updated.into();
    }
    put(store, DocType::Note, doc).unwrap()
}

pub fn file_body(id: &str, name: &str, bytes: &[u8]) -> Value {
    json!({
        "_id": id,
        "name": name,
        "contentType": "image/png",
        "createdAt": 1,
        "_attachments": {"index": {
            "content_type": "image/png",
            "data": base64::engine::general_purpose::STANDARD.encode(bytes),
        }},
    })
}

pub fn file(store: &Store, id: &str, bytes: &[u8]) -> WriteResult {
    put(store, DocType::File, file_body(id, "image.png", bytes)).unwrap()
}

pub fn body(store: &Store, id: &str) -> Option<String> {
    store.note(id).unwrap().map(|n| n.body)
}

/// Every document, tombstones included: id -> (rev, deleted).
pub fn all_docs(store: &Store) -> BTreeMap<String, (String, bool)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted)))
        .collect()
}

/// Live documents: id -> (rev, fields).
pub fn live_docs(store: &Store) -> BTreeMap<String, (String, Map<String, Value>)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .filter(|d| !d.deleted)
        .map(|d| (d.id, (d.rev, d.fields)))
        .collect()
}

pub fn has_revision(store: &Store, id: &str, rev: &str) -> bool {
    store.revisions(id).unwrap().iter().any(|r| r.rev == rev)
}

pub fn pending(store: &Store) -> usize {
    store.pending_push(usize::MAX).unwrap().len()
}

// ---------------------------------------------------------------------------
// Accounts on a MemoryServer (v2)
// ---------------------------------------------------------------------------

use std::sync::Mutex;

use kanso_sync::wire::{WirePage, WirePushDoc, WirePushResult, WireRemote};
use kanso_sync::{
    Api, ApiError, Backend, Choice, DeviceIdentity, KdfParams, MemoryServer, Password, Session,
    SyncError, SyncOptions, SyncReport,
};

pub const SERVER: &str = "http://127.0.0.1:8787";
pub const PASSWORD: &str = "correct horse battery";

/// Mac number `n`: its library ID, name and device secret.
pub fn device(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

pub fn password(username: &str) -> Password {
    Password::new(PASSWORD, username).unwrap()
}

/// Signs `username` up from Mac `n`.
pub fn sign_up(server: &MemoryServer, username: &str, n: u8) -> Session {
    kanso_sync::sign_up(
        server,
        SERVER,
        username,
        &password(username),
        &device(n),
        &|_| {},
    )
    .expect("sign up")
    .session
}

/// Signs `username` in from Mac `n`.
pub fn sign_in(server: &MemoryServer, username: &str, n: u8) -> Session {
    kanso_sync::sign_in(
        server,
        SERVER,
        username,
        &password(username),
        &device(n),
        &|_| {},
    )
    .expect("sign in")
    .session
}

/// A library bound to one account.
pub struct Mac {
    pub store: Store,
    pub session: Session,
    pub dir: tempfile::TempDir,
}

impl Mac {
    /// A new library, bound fresh to `session`'s account.
    pub fn new(session: Session) -> Mac {
        let (dir, store) = temp_store();
        kanso_sync::bind(
            &store,
            &session,
            Choice::Add,
            &dir.path().join("backups"),
            0,
        )
        .expect("bind");
        Mac {
            store,
            session,
            dir,
        }
    }

    pub fn try_sync(&self, b: &dyn Backend) -> Result<SyncReport, SyncError> {
        kanso_sync::sync(&self.store, b, &self.session, &SyncOptions::default())
    }

    /// Syncs; the sync must succeed.
    pub fn sync(&self, b: &dyn Backend) -> SyncReport {
        self.try_sync(b).expect("sync")
    }

    pub fn sync_with(
        &self,
        b: &dyn Backend,
        options: &SyncOptions,
    ) -> Result<SyncReport, SyncError> {
        kanso_sync::sync(&self.store, b, &self.session, options)
    }

    pub fn cursor(&self) -> i64 {
        self.store
            .meta(kanso_sync::meta_keys::LAST_SEQ)
            .unwrap()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }
}

/// The server's documents of `session`'s account, opened: id -> (rev, deleted).
pub fn server_docs(server: &MemoryServer, session: &Session) -> BTreeMap<String, (String, bool)> {
    server
        .plain_view(session)
        .docs()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted)))
        .collect()
}

/// Live server documents of the account, opened: id -> (rev, fields).
pub fn server_live(
    server: &MemoryServer,
    session: &Session,
) -> BTreeMap<String, (String, Map<String, Value>)> {
    server
        .plain_view(session)
        .docs()
        .into_iter()
        .filter(|d| !d.deleted)
        .map(|d| (d.id, (d.rev, d.fields)))
        .collect()
}

/// What a [`Tap`] shows its hook, in call order. The hook may rewrite the
/// requests and answers, or fail the call.
pub enum Event<'e> {
    /// Before `GET /v2/changes?since=`.
    Changes { since: i64 },
    /// The page the server answered.
    Page(&'e mut WirePage),
    /// Before `POST /v2/push`.
    Push(&'e mut Vec<WirePushDoc>),
    /// The push results the server answered.
    Pushed(&'e mut Vec<WirePushResult>),
    /// Before `PUT /v2/files/<wire_id>`.
    PutFile {
        wire_id: &'e str,
        sealed: &'e mut Vec<u8>,
    },
    /// The sealed bytes `GET /v2/files/<wire_id>` answered.
    File(&'e mut Option<Vec<u8>>),
}

/// A backend in front of a [`MemoryServer`] whose sync routes go through a
/// hook: hostile servers, races and transport failures in tests.
pub struct Tap<'a, F> {
    pub inner: &'a MemoryServer,
    pub hook: Mutex<F>,
}

impl<'a, F> Tap<'a, F>
where
    F: FnMut(Event) -> Result<(), ApiError> + Send,
{
    pub fn new(inner: &'a MemoryServer, hook: F) -> Tap<'a, F> {
        Tap {
            inner,
            hook: Mutex::new(hook),
        }
    }
}

struct TapRemote<'t, F> {
    inner: Box<dyn WireRemote + 't>,
    hook: &'t Mutex<F>,
}

impl<F> TapRemote<'_, F>
where
    F: FnMut(Event) -> Result<(), ApiError> + Send,
{
    fn call(&self, event: Event) -> Result<(), ApiError> {
        (self.hook.lock().unwrap())(event)
    }
}

impl<F> WireRemote for TapRemote<'_, F>
where
    F: FnMut(Event) -> Result<(), ApiError> + Send,
{
    fn push(&self, docs: &[WirePushDoc]) -> Result<Vec<WirePushResult>, ApiError> {
        let mut docs = docs.to_vec();
        self.call(Event::Push(&mut docs))?;
        let mut results = self.inner.push(&docs)?;
        self.call(Event::Pushed(&mut results))?;
        Ok(results)
    }

    fn changes(&self, since: i64, limit: usize) -> Result<WirePage, ApiError> {
        self.call(Event::Changes { since })?;
        let mut page = self.inner.changes(since, limit)?;
        self.call(Event::Page(&mut page))?;
        Ok(page)
    }

    fn put_file(&self, wire_id: &str, sealed: &[u8]) -> Result<(), ApiError> {
        let mut sealed = sealed.to_vec();
        self.call(Event::PutFile {
            wire_id,
            sealed: &mut sealed,
        })?;
        self.inner.put_file(wire_id, &sealed)
    }

    fn get_file(&self, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        let mut bytes = self.inner.get_file(wire_id)?;
        self.call(Event::File(&mut bytes))?;
        Ok(bytes)
    }
}

impl<F> Backend for Tap<'_, F>
where
    F: FnMut(Event) -> Result<(), ApiError> + Send,
{
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        self.inner.api(server)
    }

    fn remote(&self, session: &Session) -> Box<dyn WireRemote + '_> {
        Box::new(TapRemote {
            inner: self.inner.remote(session),
            hook: &self.hook,
        })
    }

    fn kdf_floor(&self) -> KdfParams {
        self.inner.kdf_floor()
    }

    fn kdf_default(&self) -> KdfParams {
        self.inner.kdf_default()
    }
}

/// Every meta entry the sync bookkeeping uses, for before/after checks.
pub fn sync_meta(store: &Store) -> Vec<(String, Option<String>)> {
    use kanso_sync::meta_keys as k;
    [
        k::ORIGIN,
        k::ACCOUNT_ID,
        k::USERNAME,
        k::SERVER,
        k::LAST_SEQ,
        k::EPOCH,
        k::LEGACY_LAST_SEQ,
    ]
    .into_iter()
    .map(|k| (k.to_owned(), store.meta(k).unwrap()))
    .collect()
}

/// The whole sync state of a library: every document with its revision,
/// tombstone flag, `synced_rev` and fields, and the sync meta entries.
pub fn snapshot(store: &Store) -> (Vec<kanso_store::SyncDoc>, Vec<(String, Option<String>)>) {
    (store.sync_docs().unwrap(), sync_meta(store))
}

// ---------------------------------------------------------------------------
// Plaintext canaries
// ---------------------------------------------------------------------------

/// Every way `secret` could show up in captured bytes: raw, lowercase and
/// uppercase hex, and the stable middle of its standard and URL-safe base64
/// at each of the three alignments.
pub fn encodings(secret: &[u8]) -> Vec<Vec<u8>> {
    let hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
    let mut out = vec![
        secret.to_vec(),
        hex.clone().into_bytes(),
        hex.to_uppercase().into_bytes(),
    ];
    for engine in [
        base64::engine::general_purpose::STANDARD_NO_PAD,
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ] {
        for offset in 0..3usize {
            let mut input = vec![0u8; offset];
            input.extend_from_slice(secret);
            let encoded = base64::Engine::encode(&engine, &input);
            // Only the groups made entirely of the secret's bytes.
            let first = offset.div_ceil(3) * 3;
            let last = (offset + secret.len()) / 3 * 3;
            if last > first {
                out.push(encoded.as_bytes()[first / 3 * 4..last / 3 * 4].to_vec());
            }
        }
    }
    out
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.len() >= 4 && haystack.windows(needle.len()).any(|w| w == needle)
}
