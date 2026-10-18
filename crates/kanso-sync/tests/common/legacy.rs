//! The plaintext v1 server and the old app's sync, for the rollback test
//! (docs/ACCOUNTS.md › Rollback, fix 1 / H5). The app has one sync mode, v2;
//! this is a stand-in for the old app (`Kanso-old.app`) and the old Worker,
//! so the rollback test never needs either.
//!
//! [`LegacyServer`] is the v1 Worker's rules (the `MemoryRemote` the old
//! client was tested against), and `tests/rollback.rs` checks it against
//! the v1 Worker's own cases (`cloud/test/fixtures/conformance-v1.json`).
//! [`legacy_sync`] is the old app's `sync_once`: plaintext documents, the
//! cursor under the legacy `sync.last_seq`, and the old store rules
//! (`Store::apply_remote_legacy`, test support behind kanso-store's
//! `legacy-v1-rules` feature).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Mutex;

use kanso_store::{ApplyOutcome, Attachment, Store, StoreError, SyncDoc};
use kanso_sync::{PushResult, RemoteDoc, RemoteError, SyncReport, batches, md5_hex, meta_keys};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The v1 Worker's limits.
pub const MAX_PUSH_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PUSH_DOCS: usize = 200;
pub const MAX_FIELDS_BYTES: usize = 1_900_000;
pub const MAX_CHANGES_LIMIT: usize = 500;
const MAX_FILE_BYTES: usize = kanso_core::model::MAX_ATTACHMENT_SIZE;
/// The old app's page size, batch sizes and push rounds.
const PAGE_SIZE: usize = 100;
const BATCH_DOCS: usize = 100;
const BATCH_BYTES: usize = 1024 * 1024;
const MAX_ROUNDS: usize = 4;

/// One page of `GET /v1/changes`: documents with `seq > since`, in order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegacyPage {
    pub docs: Vec<RemoteDoc>,
    /// The `seq` of the last document, or `since` for an empty page.
    pub last_seq: i64,
}

/// Attachment bytes as the v1 server stored them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyFile {
    pub md5_hex: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Stored {
    rev: String,
    seq: i64,
    deleted: bool,
    fields: Map<String, Value>,
}

#[derive(Default)]
struct State {
    docs: BTreeMap<String, Stored>,
    counter: i64,
    files: HashMap<String, LegacyFile>,
}

/// The v1 server in memory: a pushed document is accepted iff its
/// `base_rev` is the server's current revision (or the server has none and
/// `base_rev` is null); every accepted write takes the next value of one
/// global sequence; documents in a push apply in order; accepting a file
/// tombstone drops the file's bytes.
#[derive(Default)]
pub struct LegacyServer {
    state: Mutex<State>,
}

fn status(status: u16, message: impl Into<String>) -> RemoteError {
    RemoteError::Status {
        status,
        message: message.into(),
    }
}

fn is_rev(rev: &str) -> bool {
    let Some((generation, hash)) = rev.split_once('-') else {
        return false;
    };
    (1..=16).contains(&generation.len())
        && generation.bytes().all(|b| b.is_ascii_digit())
        && (1..=64).contains(&hash.len())
        && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_md5_hex(md5: &str) -> bool {
    md5.len() == 32 && md5.bytes().all(|b| b.is_ascii_hexdigit())
}

fn remote_doc(id: &str, d: &Stored) -> RemoteDoc {
    RemoteDoc {
        id: id.to_owned(),
        rev: d.rev.clone(),
        deleted: d.deleted,
        fields: d.fields.clone(),
        seq: d.seq,
    }
}

impl LegacyServer {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("legacy server poisoned")
    }

    /// Every document the server has, tombstones included, in `seq` order.
    pub fn docs(&self) -> Vec<RemoteDoc> {
        let state = self.state();
        let mut docs: Vec<RemoteDoc> = state.docs.iter().map(|(id, d)| remote_doc(id, d)).collect();
        docs.sort_by_key(|d| d.seq);
        docs
    }

    /// `POST /v1/push`.
    pub fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let body = serde_json::to_vec(&serde_json::json!({ "docs": docs }))
            .map_err(|e| RemoteError::Protocol(e.to_string()))?;
        if body.len() > MAX_PUSH_BYTES {
            return Err(status(413, "The push is too large."));
        }
        if docs.len() > MAX_PUSH_DOCS {
            return Err(status(413, "Too many documents."));
        }
        for doc in docs {
            if !kanso_sync::is_doc_id(&doc.id)
                || !is_rev(&doc.rev)
                || doc.base_rev.as_deref().is_some_and(|b| !is_rev(b))
            {
                return Err(status(400, "A malformed document."));
            }
            let size = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
            if size > MAX_FIELDS_BYTES {
                return Err(status(413, "A document is too large."));
            }
        }
        let mut state = self.state();
        let mut results = Vec::with_capacity(docs.len());
        for doc in docs {
            let current = state.docs.get(&doc.id);
            let accepted = match current {
                None => doc.base_rev.is_none(),
                Some(current) => doc.base_rev.as_deref() == Some(current.rev.as_str()),
            };
            if !accepted {
                results.push(PushResult::Conflict {
                    id: doc.id.clone(),
                    current: current.map(|c| SyncDoc {
                        id: doc.id.clone(),
                        rev: c.rev.clone(),
                        deleted: c.deleted,
                        fields: c.fields.clone(),
                        base_rev: None,
                    }),
                });
                continue;
            }
            state.counter += 1;
            let seq = state.counter;
            state.docs.insert(
                doc.id.clone(),
                Stored {
                    rev: doc.rev.clone(),
                    seq,
                    deleted: doc.deleted,
                    fields: doc.fields.clone(),
                },
            );
            if doc.deleted && kanso_sync::is_file_id(&doc.id) {
                state.files.remove(&doc.id);
            }
            results.push(PushResult::Ok {
                id: doc.id.clone(),
                seq,
            });
        }
        Ok(results)
    }

    /// `GET /v1/changes?since=&limit=`.
    pub fn changes(&self, since: i64, limit: usize) -> Result<LegacyPage, RemoteError> {
        if since < 0 {
            return Err(status(400, "since must be a non-negative integer."));
        }
        if !(1..=MAX_CHANGES_LIMIT).contains(&limit) {
            return Err(status(400, "limit is out of range."));
        }
        let state = self.state();
        let mut docs: Vec<RemoteDoc> = state
            .docs
            .iter()
            .filter(|(_, d)| d.seq > since)
            .map(|(id, d)| remote_doc(id, d))
            .collect();
        docs.sort_by_key(|d| d.seq);
        docs.truncate(limit);
        let last_seq = docs.last().map_or(since, |d| d.seq);
        Ok(LegacyPage { docs, last_seq })
    }

    /// `PUT /v1/files/<id>`; the server checks the bytes against `md5`.
    pub fn put_file(
        &self,
        id: &str,
        claimed_md5: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError> {
        if !kanso_sync::is_file_id(id) {
            return Err(status(400, "Not a file ID."));
        }
        if !is_md5_hex(claimed_md5) {
            return Err(status(400, "x-kanso-md5 must be 32 hex digits."));
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(status(413, "The file is too large."));
        }
        let actual = md5_hex(bytes);
        if actual != claimed_md5.to_ascii_lowercase() {
            return Err(status(400, "The bytes do not match x-kanso-md5."));
        }
        let content_type = if content_type.is_empty() {
            "application/octet-stream".to_owned()
        } else {
            content_type.to_owned()
        };
        self.state().files.insert(
            id.to_owned(),
            LegacyFile {
                md5_hex: actual,
                content_type,
                bytes: bytes.to_vec(),
            },
        );
        Ok(())
    }

    /// `GET /v1/files/<id>`; `None` when the server has no bytes for `id`.
    pub fn get_file(&self, id: &str) -> Result<Option<LegacyFile>, RemoteError> {
        if !kanso_sync::is_file_id(id) {
            return Err(status(400, "Not a file ID."));
        }
        Ok(self.state().files.get(id).cloned())
    }
}

/// One sync of the old app against the v1 server: pull after the legacy
/// cursor, then push every pending document.
pub fn legacy_sync(store: &Store, server: &LegacyServer) -> SyncReport {
    let mut report = SyncReport::default();
    pull(store, server, &mut report);
    push(store, server, &mut report);
    report
}

fn cursor(store: &Store) -> i64 {
    store
        .meta(meta_keys::LEGACY_LAST_SEQ)
        .unwrap()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(0)
}

fn pull(store: &Store, server: &LegacyServer, report: &mut SyncReport) {
    let mut since = cursor(store);
    loop {
        let page = server.changes(since, PAGE_SIZE).expect("v1 changes");
        let mut last = since;
        for doc in &page.docs {
            assert!(doc.seq > last, "the v1 feed went backwards");
            apply(store, server, &doc.to_sync_doc(), report);
            last = doc.seq;
        }
        if last > since {
            store
                .set_meta(meta_keys::LEGACY_LAST_SEQ, &last.to_string())
                .unwrap();
            since = last;
        }
        if page.docs.len() < PAGE_SIZE {
            return;
        }
    }
}

fn apply(store: &Store, server: &LegacyServer, doc: &SyncDoc, report: &mut SyncReport) {
    let attachment = if !doc.deleted && kanso_sync::is_file_id(&doc.id) {
        fetch_blob(store, server, doc, report)
    } else {
        None
    };
    let outcome = match store.apply_remote_legacy(doc, attachment) {
        Ok(outcome) => outcome,
        Err(StoreError::Invalid(reason)) => {
            report.invalid.push((doc.id.clone(), reason));
            return;
        }
        Err(err) => panic!("v1 apply: {err}"),
    };
    match outcome {
        ApplyOutcome::Applied => report.pulled += 1,
        ApplyOutcome::Unchanged => {}
        ApplyOutcome::RemoteWon => {
            report.pulled += 1;
            report.conflicts_remote += 1;
        }
        ApplyOutcome::LocalWon { .. } => report.conflicts_local += 1,
        ApplyOutcome::Stale { .. } => panic!("the old rules never answer Stale"),
    }
}

fn fetch_blob(
    store: &Store,
    server: &LegacyServer,
    doc: &SyncDoc,
    report: &mut SyncReport,
) -> Option<Attachment> {
    if let Some(md5) = doc.fields.get("md5digest").and_then(Value::as_str)
        && store.has_blob(&doc.id, &md5.to_ascii_lowercase()).unwrap()
    {
        return None;
    }
    let Some(file) = server.get_file(&doc.id).expect("v1 file") else {
        report.files_missing += 1;
        return None;
    };
    assert_eq!(md5_hex(&file.bytes), file.md5_hex, "v1 file bytes");
    report.files_down += 1;
    Some(Attachment::new(file.bytes))
}

fn push(store: &Store, server: &LegacyServer, report: &mut SyncReport) {
    let mut lost = BTreeSet::new();
    for _ in 0..MAX_ROUNDS {
        let mut docs = Vec::new();
        let mut sizes = Vec::new();
        for doc in store.pending_push(usize::MAX).unwrap() {
            let fields = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
            if lost.contains(&doc.id) || fields > MAX_FIELDS_BYTES {
                continue;
            }
            sizes.push(serde_json::to_vec(&doc).map_or(usize::MAX, |s| s.len()));
            docs.push(doc);
        }
        if docs.is_empty() {
            break;
        }
        let mut conflicts = 0;
        for range in batches(&sizes, BATCH_DOCS, BATCH_BYTES) {
            let batch = &docs[range];
            for doc in batch
                .iter()
                .filter(|d| !d.deleted && kanso_sync::is_file_id(&d.id))
            {
                match store.attachment(&doc.id).unwrap() {
                    Some((content_type, blob)) => {
                        server
                            .put_file(&doc.id, &blob.md5_hex(), &content_type, &blob.data)
                            .expect("v1 upload");
                        report.files_up += 1;
                    }
                    None => report.files_missing += 1,
                }
            }
            let results = server.push(batch).expect("v1 push");
            assert_eq!(results.len(), batch.len());
            for (doc, result) in batch.iter().zip(results) {
                match result {
                    PushResult::Ok { .. } => {
                        store.mark_synced(&doc.id, &doc.rev, None).unwrap();
                        report.pushed += 1;
                    }
                    PushResult::Conflict {
                        current: Some(current),
                        ..
                    } => {
                        conflicts += 1;
                        apply(store, server, &current, report);
                    }
                    // The v1 server lost it: the old app keeps it pending.
                    _ => {
                        lost.insert(doc.id.clone());
                    }
                }
            }
        }
        if conflicts == 0 {
            break;
        }
    }
    report.pending = store.pending_push(usize::MAX).unwrap().len();
}
