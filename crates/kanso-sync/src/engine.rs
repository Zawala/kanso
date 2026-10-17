//! One sync pass: pull the server's changes, then push local ones.

use std::collections::BTreeSet;
use std::ops::Range;

use kanso_store::{ApplyOutcome, Attachment, Store, StoreError, SyncDoc};
use serde_json::Value;

use crate::remote::{MAX_FIELDS_BYTES, PushResult, Remote, RemoteError, is_file_id, md5_hex};

/// The `meta` key holding the last change sequence pulled from the server.
pub const CURSOR_KEY: &str = "sync.last_seq";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncOptions {
    /// Documents per `changes` page.
    pub page_size: usize,
    /// Most documents per push.
    pub batch_docs: usize,
    /// Target serialized size of one push; a larger document goes alone.
    pub batch_bytes: usize,
    /// Push rounds per sync. A round resolves the conflicts of the one before.
    pub max_rounds: usize,
}

impl Default for SyncOptions {
    fn default() -> Self {
        SyncOptions {
            page_size: 100,
            batch_docs: 100,
            batch_bytes: 1024 * 1024,
            max_rounds: 4,
        }
    }
}

/// What a sync did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncReport {
    /// Remote changes written locally, including conflicts the remote won.
    pub pulled: usize,
    /// Local changes the server accepted.
    pub pushed: usize,
    /// Conflicts the local version won; the remote one is in its revisions.
    pub conflicts_local: usize,
    /// Conflicts the remote version won; the local one is in its revisions.
    pub conflicts_remote: usize,
    pub files_up: usize,
    pub files_down: usize,
    /// Attachments a document refers to whose bytes neither side had.
    pub files_missing: usize,
    /// Local changes still not on the server after this sync: edited during
    /// it, conflicts that kept racing, or listed below.
    pub pending: usize,
    /// Documents that had a conflict this sync, for the UI to flag.
    pub conflicted: Vec<String>,
    /// Documents the server refused because it has no copy of their base
    /// revision (it lost data, or was replaced).
    pub rejected: Vec<String>,
    /// Documents too large for the server (more than [`MAX_FIELDS_BYTES`]).
    pub too_large: Vec<String>,
    /// Remote documents the store could not accept, with the reason.
    pub invalid: Vec<(String, String)>,
}

impl SyncReport {
    pub fn conflicts(&self) -> usize {
        self.conflicts_local + self.conflicts_remote
    }

    fn note_conflict(&mut self, id: &str) {
        if !self.conflicted.iter().any(|c| c == id) {
            self.conflicted.push(id.to_owned());
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Remote(#[from] RemoteError),
}

fn protocol(message: String) -> SyncError {
    SyncError::Remote(RemoteError::Protocol(message))
}

/// The last change sequence this store pulled.
pub fn cursor(store: &Store) -> Result<i64, StoreError> {
    Ok(store
        .meta(CURSOR_KEY)?
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v >= 0)
        .unwrap_or(0))
}

/// Makes the next sync pull every document again. Safe at any time: what the
/// store already has comes back as unchanged, and missing attachment bytes
/// are fetched.
pub fn reset_cursor(store: &Store) -> Result<(), StoreError> {
    store.set_meta(CURSOR_KEY, "0")
}

/// Pulls, then pushes, with the default options.
pub fn sync_once<R: Remote + ?Sized>(store: &Store, remote: &R) -> Result<SyncReport, SyncError> {
    sync_with(store, remote, &SyncOptions::default())
}

/// One sync pass.
///
/// 1. Pull: pages through the server's changes after the stored cursor,
///    fetching the bytes of live files the store lacks, and applies each
///    document (last writer by `updatedAt` wins; the loser is kept in
///    revisions). The cursor is saved after each page.
/// 2. Push: sends every pending document in batches, uploading file bytes
///    first. Accepted documents are marked synced; on a conflict the server's
///    version is applied, and a local version that won goes again in the
///    next round.
pub fn sync_with<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    options: &SyncOptions,
) -> Result<SyncReport, SyncError> {
    let mut report = SyncReport::default();
    pull(store, remote, options, &mut report)?;
    push(store, remote, options, &mut report)?;
    Ok(report)
}

fn pull<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    options: &SyncOptions,
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    let page_size = options.page_size.max(1);
    let mut since = cursor(store)?;
    loop {
        let page = remote.changes(since, page_size)?;
        let mut last = since;
        for doc in &page.docs {
            if doc.seq <= last {
                return Err(protocol(format!(
                    "the change feed went from seq {last} to {}",
                    doc.seq
                )));
            }
            apply(store, remote, &doc.to_sync_doc(), report)?;
            last = doc.seq;
        }
        if last > since {
            store.set_meta(CURSOR_KEY, &last.to_string())?;
            since = last;
        }
        if page.docs.len() < page_size {
            return Ok(());
        }
    }
}

/// Applies one server document, fetching its attachment bytes when needed.
fn apply<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    doc: &SyncDoc,
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    let attachment = if !doc.deleted && is_file_id(&doc.id) {
        fetch_blob(store, remote, doc, report)?
    } else {
        None
    };
    let outcome = match store.apply_remote(doc, attachment) {
        Ok(outcome) => outcome,
        Err(StoreError::Invalid(reason)) => {
            report.invalid.push((doc.id.clone(), reason));
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };
    match outcome {
        ApplyOutcome::Applied => report.pulled += 1,
        ApplyOutcome::Unchanged => {}
        ApplyOutcome::RemoteWon => {
            report.pulled += 1;
            report.conflicts_remote += 1;
            report.note_conflict(&doc.id);
        }
        ApplyOutcome::LocalWon => {
            report.conflicts_local += 1;
            report.note_conflict(&doc.id);
        }
    }
    Ok(())
}

fn fetch_blob<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    doc: &SyncDoc,
    report: &mut SyncReport,
) -> Result<Option<Attachment>, SyncError> {
    if let Some(md5) = doc.fields.get("md5digest").and_then(Value::as_str)
        && store.has_blob(&doc.id, &md5.to_ascii_lowercase())?
    {
        return Ok(None);
    }
    let Some(file) = remote.get_file(&doc.id)? else {
        report.files_missing += 1;
        return Ok(None);
    };
    if md5_hex(&file.bytes) != file.md5_hex.to_ascii_lowercase() {
        return Err(protocol(format!(
            "the bytes of {} do not match their MD5",
            doc.id
        )));
    }
    report.files_down += 1;
    Ok(Some(Attachment::new(file.bytes)))
}

fn push<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    options: &SyncOptions,
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    let mut rejected = BTreeSet::new();
    for _ in 0..options.max_rounds.max(1) {
        let mut docs = Vec::new();
        let mut sizes = Vec::new();
        report.too_large.clear();
        for doc in store.pending_push(usize::MAX)? {
            if rejected.contains(&doc.id) {
                continue;
            }
            let fields = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
            if fields > MAX_FIELDS_BYTES {
                report.too_large.push(doc.id);
                continue;
            }
            sizes.push(serde_json::to_vec(&doc).map_or(usize::MAX, |s| s.len()));
            docs.push(doc);
        }
        if docs.is_empty() {
            break;
        }
        let mut conflicts = 0;
        for range in batches(&sizes, options.batch_docs, options.batch_bytes) {
            let batch = &docs[range];
            upload_files(store, remote, batch, report)?;
            let results = remote.push(batch)?;
            if results.len() != batch.len() {
                return Err(protocol(format!(
                    "a push of {} documents got {} results",
                    batch.len(),
                    results.len()
                )));
            }
            for (doc, result) in batch.iter().zip(results) {
                if result.id() != doc.id {
                    return Err(protocol(format!(
                        "the result for {} names {}",
                        doc.id,
                        result.id()
                    )));
                }
                match result {
                    PushResult::Ok { .. } => {
                        store.mark_synced(&doc.id, &doc.rev)?;
                        report.pushed += 1;
                    }
                    PushResult::Conflict {
                        current: Some(current),
                        ..
                    } => {
                        if current.id != doc.id {
                            return Err(protocol(format!(
                                "the conflict for {} carries {}",
                                doc.id, current.id
                            )));
                        }
                        conflicts += 1;
                        apply(store, remote, &current, report)?;
                    }
                    PushResult::Conflict { current: None, .. } => {
                        rejected.insert(doc.id.clone());
                    }
                }
            }
        }
        if conflicts == 0 {
            break;
        }
    }
    report.rejected = rejected.into_iter().collect();
    report.pending = store.pending_push(usize::MAX)?.len();
    Ok(())
}

/// Uploads the bytes of the live files in `batch` so they are on the server
/// before any device can see the documents.
fn upload_files<R: Remote + ?Sized>(
    store: &Store,
    remote: &R,
    batch: &[SyncDoc],
    report: &mut SyncReport,
) -> Result<(), SyncError> {
    for doc in batch.iter().filter(|d| !d.deleted && is_file_id(&d.id)) {
        match store.attachment(&doc.id)? {
            Some((content_type, blob)) => {
                remote.put_file(&doc.id, &blob.md5_hex(), &content_type, &blob.data)?;
                report.files_up += 1;
            }
            None => report.files_missing += 1,
        }
    }
    Ok(())
}

/// Splits items of the given sizes into consecutive batches of at most
/// `max_docs` items and `max_bytes` bytes, filling each batch greedily. An
/// item larger than `max_bytes` goes alone.
pub fn batches(sizes: &[usize], max_docs: usize, max_bytes: usize) -> Vec<Range<usize>> {
    let max_docs = max_docs.max(1);
    let mut out = Vec::new();
    let mut start = 0;
    let mut bytes = 0usize;
    for (i, &size) in sizes.iter().enumerate() {
        let len = i - start;
        if len > 0 && (len >= max_docs || bytes.saturating_add(size) > max_bytes) {
            out.push(start..i);
            start = i;
            bytes = 0;
        }
        bytes = bytes.saturating_add(size);
    }
    if start < sizes.len() {
        out.push(start..sizes.len());
    }
    out
}
