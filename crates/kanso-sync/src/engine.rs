//! One sync pass: pull the server's changes, then push local ones; and what
//! a sync reports (docs/ACCOUNTS.md › Sync with encryption, › Problems).
//!
//! [`sync`] is the only engine: the v2 rules over `Sealed` (sealed
//! documents, opaque IDs, the guard, re-merge, repair, quotas and pauses).
//! [`SyncReport`], [`SyncOptions`], [`SyncError`], [`SyncProblem`],
//! [`problem`] and [`report_problem`] are frozen by step 1.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_store::{ApplyOutcome, Attachment, Store, StoreError, SyncDoc, SyncGuard};
use serde_json::Value;

use crate::account::Session;
use crate::api::Backend;
use crate::binding::meta_keys;
use crate::remote::{is_file_id, md5_hex};
use crate::sealed::{Entry, FileBytes, Pushed, Sealed};
use crate::wire::{ApiError, QuotaKind, SignOutReason, WireChange};

#[derive(Clone, Debug)]
pub struct SyncOptions {
    /// Documents per `changes` page.
    pub page_size: usize,
    /// Most documents per push.
    pub batch_docs: usize,
    /// Target serialized size of one push; a larger document goes alone.
    pub batch_bytes: usize,
    /// Push rounds per sync. A round resolves the conflicts of the one before.
    pub max_rounds: usize,
    /// Set to stop the sync: the engine checks it between pages, batches and
    /// files and returns [`SyncError::Cancelled`]. Every re-bind and
    /// sign-out cancels and joins the running sync first (fix 2).
    pub cancel: Option<Arc<AtomicBool>>,
    /// Most live files whose bytes this Mac lacks that one sync fetches
    /// again, after the pull (docs/ACCOUNTS.md › Sync with encryption ›
    /// Files). 0 turns it off.
    pub refetch_files: usize,
}

impl Default for SyncOptions {
    fn default() -> Self {
        SyncOptions {
            page_size: 100,
            batch_docs: 100,
            batch_bytes: 1024 * 1024,
            max_rounds: 4,
            cancel: None,
            refetch_files: 8,
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
    /// Documents too large for the server: `fields` JSON over 1,400,000
    /// bytes (`wire::MAX_FIELDS_JSON`).
    pub too_large: Vec<String>,
    /// Remote documents the store could not accept, with the reason.
    pub invalid: Vec<(String, String)>,
    /// Wire IDs of feed documents (format 1) that couldn't be opened and
    /// weren't repaired; the cursor moved past them. `kanso-sync verify
    /// --repair` handles them (security re-check 10).
    pub skipped: Vec<String>,
    /// Local IDs whose server revision was older than this Mac's synced one
    /// (M2): the local version was rebased onto it and pushed again.
    pub stale: Vec<String>,
    /// Local IDs whose undecryptable server copy is being replaced from this
    /// Mac (M3): only when the broken revision is `wire_rev(id, synced_rev)`.
    pub repaired: Vec<String>,
    /// A feed document uses a newer format: the pull stopped before it.
    pub too_new: bool,
    /// The server's copy was restored or rewound (epoch or head changed), and
    /// this Mac re-merged (M1).
    pub remerged: bool,
    /// A push or upload hit this quota (413); deletions still went through.
    pub quota: Option<QuotaKind>,
    /// A 429 on a push or upload: pushing stopped until this time (ms).
    pub paused_until: Option<i64>,
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
    /// From a v2 route. A 401 ends the session; a 404 means the server isn't
    /// v2 (`ServerOutdated`).
    #[error(transparent)]
    Api(#[from] ApiError),
    /// The server's key ID differs from this Mac's: nothing was written.
    #[error("the server's account keys don't match this Mac's")]
    WrongKey,
    /// [`SyncOptions::cancel`] was set, or the library was re-bound while
    /// the sync ran (the store's guard failed).
    #[error("sync cancelled")]
    Cancelled,
}

/// The app's sync: one pass of `session`'s account over `b`, with every v2
/// rule (docs/ACCOUNTS.md › Sync with encryption, the order below).
///
/// 1. **Pull.** Pages through the feed after the cursor
///    (`sync.v2.last_seq`), opening each document. The first page
///    is checked before anything is written: a key ID other than this
///    Mac's is [`SyncError::WrongKey`]; an epoch other than the saved one,
///    or a `head_seq` below the cursor, means the server's copy was
///    restored or rewound, and the Mac re-merges (M1): the cursor goes to
///    0, the whole feed is applied through `apply_remote` (so replayed
///    revisions come back `Stale` and are rebased), and the documents the
///    feed lacked are marked unsynced and pushed again with no base.
///    Undecryptable documents never stop the pull; one is repaired only
///    when it is the version this Mac last synced (M3), else skipped. A
///    newer format stops the pull before it. Then up to
///    [`SyncOptions::refetch_files`] live files whose bytes this Mac lacks
///    are fetched again.
/// 2. **Push.** Sends pending documents in batches, sealed, uploading file
///    bytes first; up to [`SyncOptions::max_rounds`] rounds resolve
///    conflicts, lost documents and `behind` answers (which pull first). A
///    413 retries the batch with its tombstones only and pushes nothing but
///    tombstones afterwards; a second 413, or any 429, stops pushing.
///
/// Every store write carries the session's guard
/// ([`crate::binding::guard_pairs`]); if the library was re-bound
/// meanwhile, or [`SyncOptions::cancel`] is set, the sync stops with
/// [`SyncError::Cancelled`]. Nothing is pushed unless the library is bound
/// to the session's account.
pub fn sync(
    store: &Store,
    b: &dyn Backend,
    session: &Session,
    options: &SyncOptions,
) -> Result<SyncReport, SyncError> {
    let pairs = crate::binding::guard_pairs(session);
    let sealed = Sealed::new(b.remote(session), session);
    let mut run = Run {
        store,
        sealed: &sealed,
        guard: SyncGuard { expect: &pairs },
        options,
        report: SyncReport::default(),
        epoch: None,
        remerging: false,
        present: HashSet::new(),
        wire_ids: None,
        fetched: HashSet::new(),
        stopped: false,
        tombstones_only: false,
    };
    run.check_cancel()?;
    run.check_bound()?;
    run.pull()?;
    run.refetch()?;
    run.push()?;
    run.report.pending = store.pending_count()?;
    Ok(run.report)
}

/// The value of [`meta_keys::EPOCH`] while a re-merge hasn't finished. It is
/// never a server's epoch (16 hex digits), so an interrupted re-merge runs
/// again from the start at the next sync.
const REMERGING: &str = "remerging";

/// Turns the store's guard failure into [`SyncError::Cancelled`].
fn guarded(err: StoreError) -> SyncError {
    match err {
        StoreError::GuardFailed => SyncError::Cancelled,
        err => SyncError::Store(err),
    }
}

/// The content hash of a local revision (`{generation}-{hash}`).
fn content_hash(rev: &str) -> &str {
    rev.split_once('-').map_or(rev, |(_, hash)| hash)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// An upper bound of a document's size on the wire, sealed, from its
/// `fields` JSON length: the plaintext adds the ID, the revision and 30
/// bytes of framing; sealing adds 41 bytes and base64url a third; the
/// push entry adds its wire ID, revisions and keys.
fn wire_size(doc: &SyncDoc, fields_len: usize) -> usize {
    let plaintext = fields_len + doc.id.len() + doc.rev.len() + 30;
    (plaintext + 41).div_ceil(3) * 4 + 160
}

/// One run of [`sync`].
struct Run<'r> {
    store: &'r Store,
    sealed: &'r Sealed<'r>,
    guard: SyncGuard<'r>,
    options: &'r SyncOptions,
    report: SyncReport,
    /// The epoch of this sync's first page.
    epoch: Option<String>,
    /// A re-merge is running: `present` collects the local IDs the feed
    /// carries.
    remerging: bool,
    present: HashSet<String>,
    /// Wire ID → (local ID, `synced_rev`) of every local document, built
    /// when the first undecryptable document of a pull shows up.
    wire_ids: Option<HashMap<String, (String, Option<String>)>>,
    /// File IDs whose bytes this sync already asked the server for.
    fetched: HashSet<String>,
    /// Pushing stopped for this sync (a second 413, or a 429).
    stopped: bool,
    /// After a 413: push only tombstones.
    tombstones_only: bool,
}

impl Run<'_> {
    fn check_cancel(&self) -> Result<(), SyncError> {
        match &self.options.cancel {
            Some(flag) if flag.load(std::sync::atomic::Ordering::SeqCst) => {
                Err(SyncError::Cancelled)
            }
            _ => Ok(()),
        }
    }

    /// The library must still be bound to the session's account before
    /// anything of it is sent there.
    fn check_bound(&self) -> Result<(), SyncError> {
        for (key, value) in self.guard.expect {
            if self.store.meta(key)?.as_deref() != Some(*value) {
                return Err(SyncError::Cancelled);
            }
        }
        Ok(())
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<(), SyncError> {
        self.store
            .set_meta_if(key, value, &self.guard)
            .map_err(guarded)
    }

    fn cursor(&self) -> Result<i64, SyncError> {
        Ok(self
            .store
            .meta(meta_keys::LAST_SEQ)?
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v >= 0)
            .unwrap_or(0))
    }

    /// Pages through the feed from the cursor (see [`sync`]). Runs again
    /// before a push round after a `behind` answer.
    fn pull(&mut self) -> Result<(), SyncError> {
        let page_size = self
            .options
            .page_size
            .clamp(1, crate::wire::MAX_CHANGES_LIMIT);
        let mut since = self.cursor()?;
        self.wire_ids = None;
        loop {
            self.check_cancel()?;
            let page = self.sealed.changes(since, page_size)?;
            if page.key_id != self.sealed.key_id() {
                return Err(SyncError::WrongKey);
            }
            match &self.epoch {
                None => {
                    self.epoch = Some(page.epoch.clone());
                    let saved = self.store.meta(meta_keys::EPOCH)?;
                    let restored =
                        saved.as_deref().is_some_and(|e| e != page.epoch) || page.head_seq < since;
                    if restored {
                        self.set_meta(meta_keys::LAST_SEQ, "0")?;
                        self.set_meta(meta_keys::EPOCH, REMERGING)?;
                        self.remerging = true;
                        self.present.clear();
                        self.report.remerged = true;
                        if since != 0 {
                            since = 0;
                            continue;
                        }
                    } else if saved.is_none() {
                        // The first sync after binding: adopt the epoch.
                        self.set_meta(meta_keys::EPOCH, &page.epoch)?;
                    }
                }
                Some(epoch) if *epoch != page.epoch => {
                    return Err(SyncError::Api(ApiError::Protocol(
                        "the epoch changed during the sync".into(),
                    )));
                }
                Some(_) => {}
            }
            let mut last = since;
            let mut too_new = false;
            for entry in page.entries {
                let seq = entry.seq();
                match entry {
                    Entry::TooNew { .. } => {
                        too_new = true;
                        break;
                    }
                    Entry::Opened { doc, .. } => {
                        if self.remerging {
                            self.present.insert(doc.id.clone());
                        }
                        self.apply(&doc)?;
                    }
                    Entry::Undecryptable { wire, .. } => self.undecryptable(&wire)?,
                }
                last = seq;
            }
            if last > since {
                self.set_meta(meta_keys::LAST_SEQ, &last.to_string())?;
                since = last;
            }
            if too_new {
                // Stop before it, keeping the cursor; pushing continues.
                self.report.too_new = true;
                return Ok(());
            }
            if !page.more {
                break;
            }
        }
        if self.remerging {
            self.store
                .mark_missing_unsynced(&self.present, Some(&self.guard))
                .map_err(guarded)?;
            let epoch = self.epoch.clone().unwrap_or_default();
            self.set_meta(meta_keys::EPOCH, &epoch)?;
            self.remerging = false;
            self.present.clear();
        }
        Ok(())
    }

    /// Applies one opened server document, fetching its file bytes when the
    /// store lacks them.
    fn apply(&mut self, doc: &SyncDoc) -> Result<(), SyncError> {
        let attachment = if !doc.deleted && is_file_id(&doc.id) {
            self.fetch_blob(doc)?
        } else {
            None
        };
        // During a re-merge the Mac can't tell whether a server version
        // descends from the one it synced (the server lost history), so a
        // remote version that replaces different live content is flagged.
        let replaced = if self.remerging {
            self.store
                .get(&doc.id)?
                .map(|d| d.rev)
                .filter(|rev| content_hash(rev) != content_hash(&doc.rev))
        } else {
            None
        };
        let outcome = match self.store.apply_remote(doc, attachment, Some(&self.guard)) {
            Ok(outcome) => outcome,
            Err(StoreError::Invalid(reason)) => {
                self.report.invalid.push((doc.id.clone(), reason));
                return Ok(());
            }
            Err(err) => return Err(guarded(err)),
        };
        let report = &mut self.report;
        match outcome {
            ApplyOutcome::Applied => {
                report.pulled += 1;
                if replaced.is_some() {
                    // The local version is in history (`apply_remote`
                    // snapshots what it replaces).
                    report.conflicts_remote += 1;
                    report.note_conflict(&doc.id);
                }
            }
            ApplyOutcome::Unchanged => {}
            ApplyOutcome::RemoteWon => {
                report.pulled += 1;
                report.conflicts_remote += 1;
                report.note_conflict(&doc.id);
            }
            ApplyOutcome::LocalWon { .. } => {
                report.conflicts_local += 1;
                report.note_conflict(&doc.id);
            }
            ApplyOutcome::Stale { conflict, .. } => {
                if !report.stale.contains(&doc.id) {
                    report.stale.push(doc.id.clone());
                }
                if conflict {
                    // A live version below the floor that this Mac never
                    // held: kept in history and flagged, never silent.
                    report.conflicts_local += 1;
                    report.note_conflict(&doc.id);
                }
            }
        }
        Ok(())
    }

    /// The opened bytes of a live file document, when they match its
    /// `md5digest`; otherwise they count as missing.
    fn fetch_blob(&mut self, doc: &SyncDoc) -> Result<Option<Attachment>, SyncError> {
        let md5 = doc
            .fields
            .get("md5digest")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase);
        if let Some(md5) = &md5
            && self.store.has_blob(&doc.id, md5)?
        {
            return Ok(None);
        }
        self.check_cancel()?;
        self.fetched.insert(doc.id.clone());
        match self.sealed.get_file(&doc.id)? {
            FileBytes::Opened(bytes) if md5.as_deref() == Some(md5_hex(&bytes).as_str()) => {
                self.report.files_down += 1;
                Ok(Some(Attachment::new(bytes)))
            }
            FileBytes::Opened(_) | FileBytes::Missing | FileBytes::Unreadable => {
                self.report.files_missing += 1;
                Ok(None)
            }
        }
    }

    /// Fetches again the bytes of up to [`SyncOptions::refetch_files`] live
    /// files this Mac lacks (review-sync 5), going on in ID order from where
    /// the last sync stopped and skipping files this sync already asked for.
    /// Bytes are kept only when they open, match the document's `md5digest`
    /// and the document still has the revision they were fetched for
    /// (`Store::fill_blob`, guarded).
    fn refetch(&mut self) -> Result<(), SyncError> {
        let limit = self.options.refetch_files;
        if limit == 0 {
            return Ok(());
        }
        let after = self
            .store
            .meta(meta_keys::REFETCH_AFTER)?
            .unwrap_or_default();
        // Enough rows that skipping the ones this sync fetched still leaves
        // `limit`, from where the last sync stopped, then from the start.
        let window = limit + self.fetched.len();
        let mut todo = self.store.files_without_bytes(&after, window)?;
        if todo.len() < window && !after.is_empty() {
            todo.extend(
                self.store
                    .files_without_bytes("", window)?
                    .into_iter()
                    .filter(|(id, ..)| *id <= after),
            );
        }
        todo.retain(|(id, ..)| !self.fetched.contains(id));
        todo.truncate(limit);
        let mut last = None;
        for (id, rev, md5) in todo {
            self.check_cancel()?;
            self.fetched.insert(id.clone());
            match self.sealed.get_file(&id)? {
                FileBytes::Opened(bytes) if md5_hex(&bytes) == md5 => {
                    if self
                        .store
                        .fill_blob(&id, &rev, Attachment::new(bytes), Some(&self.guard))
                        .map_err(guarded)?
                    {
                        self.report.files_down += 1;
                    }
                }
                FileBytes::Opened(_) | FileBytes::Missing | FileBytes::Unreadable => {
                    self.report.files_missing += 1;
                }
            }
            last = Some(id);
        }
        if let Some(last) = last {
            self.set_meta(meta_keys::REFETCH_AFTER, &last)?;
        }
        Ok(())
    }

    /// A feed document that didn't open (M3, security re-check 10).
    fn undecryptable(&mut self, wire: &WireChange) -> Result<(), SyncError> {
        if self.wire_ids.is_none() {
            let map = self
                .store
                .sync_docs()?
                .into_iter()
                .map(|d| (self.sealed.wire_id(&d.id), (d.id, d.base_rev)))
                .collect();
            self.wire_ids = Some(map);
        }
        let Some((local_id, synced)) = self.wire_ids.as_ref().and_then(|m| m.get(&wire.id)) else {
            self.report.skipped.push(wire.id.clone());
            return Ok(());
        };
        let (local_id, synced) = (local_id.clone(), synced.clone());
        if self.remerging {
            // The server has it, broken: it isn't missing.
            self.present.insert(local_id.clone());
        }
        let ours = synced.map(|s| self.sealed.wire_rev(&local_id, &s));
        if ours.as_deref() == Some(wire.rev.as_str())
            && self
                .store
                .bump_revision(&local_id, Some(&self.guard))
                .map_err(guarded)?
                .is_some()
        {
            // Pushed one generation higher over the broken revision, which
            // is `wire_rev(id, synced_rev)`, the base the push sends.
            if !self.report.repaired.contains(&local_id) {
                self.report.repaired.push(local_id);
            }
        } else {
            self.report.skipped.push(wire.id.clone());
        }
        Ok(())
    }

    fn pause(&mut self, retry_after: u64) {
        let until = now_ms().saturating_add((retry_after as i64).saturating_mul(1000));
        self.report.paused_until = Some(self.report.paused_until.map_or(until, |u| u.max(until)));
        self.stopped = true;
    }

    /// Sends pending documents (see [`sync`]).
    fn push(&mut self) -> Result<(), SyncError> {
        // Documents this sync can't push: their server copy is too new or
        // unreadable.
        let mut held: HashSet<String> = HashSet::new();
        let max_docs = self.options.batch_docs.clamp(1, crate::wire::MAX_PUSH_DOCS);
        let max_bytes = self
            .options
            .batch_bytes
            .clamp(1, crate::wire::MAX_PUSH_BODY - 1024);
        for _ in 0..self.options.max_rounds.max(1) {
            self.check_cancel()?;
            let mut docs = Vec::new();
            let mut sizes = Vec::new();
            self.report.too_large.clear();
            for doc in self.store.pending_push(usize::MAX)? {
                if held.contains(&doc.id) || (self.tombstones_only && !doc.deleted) {
                    continue;
                }
                let fields = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
                if !doc.deleted && fields > crate::wire::MAX_FIELDS_JSON {
                    self.report.too_large.push(doc.id);
                    continue;
                }
                sizes.push(wire_size(&doc, fields));
                docs.push(doc);
            }
            if docs.is_empty() {
                break;
            }
            let mut again = false;
            let mut need_pull = false;
            for range in batches(&sizes, max_docs, max_bytes) {
                self.check_cancel()?;
                let batch: Vec<SyncDoc> = docs[range]
                    .iter()
                    .filter(|d| !self.tombstones_only || d.deleted)
                    .cloned()
                    .collect();
                let batch = self.upload_files(batch)?;
                if self.stopped {
                    break;
                }
                if batch.is_empty() {
                    continue;
                }
                let Some((batch, results)) = self.push_batch(batch)? else {
                    if self.stopped {
                        break;
                    }
                    continue;
                };
                for (doc, result) in batch.iter().zip(results) {
                    match result {
                        Pushed::Ok => {
                            self.store
                                .mark_synced(&doc.id, &doc.rev, Some(&self.guard))
                                .map_err(guarded)?;
                            self.report.pushed += 1;
                        }
                        Pushed::Conflict(current) => {
                            again = true;
                            self.apply(&current)?;
                        }
                        Pushed::Lost => {
                            // The server lost it (M1): push it again with no base.
                            self.store
                                .mark_unsynced(&doc.id, Some(&self.guard))
                                .map_err(guarded)?;
                            again = true;
                        }
                        Pushed::Behind => {
                            need_pull = true;
                            again = true;
                        }
                        Pushed::Unreadable {
                            newer_format: true, ..
                        } => {
                            self.report.too_new = true;
                            held.insert(doc.id.clone());
                        }
                        Pushed::Unreadable {
                            wire_rev,
                            newer_format: false,
                        } => {
                            let ours = doc
                                .base_rev
                                .as_deref()
                                .map(|s| self.sealed.wire_rev(&doc.id, s));
                            if ours.as_deref() == Some(wire_rev.as_str())
                                && self
                                    .store
                                    .bump_revision(&doc.id, Some(&self.guard))
                                    .map_err(guarded)?
                                    .is_some()
                            {
                                if !self.report.repaired.contains(&doc.id) {
                                    self.report.repaired.push(doc.id.clone());
                                }
                                again = true;
                            } else {
                                let wire_id = self.sealed.wire_id(&doc.id);
                                if !self.report.skipped.contains(&wire_id) {
                                    self.report.skipped.push(wire_id);
                                }
                                held.insert(doc.id.clone());
                            }
                        }
                    }
                }
            }
            if self.stopped || !again {
                break;
            }
            if need_pull {
                self.pull()?;
            }
        }
        Ok(())
    }

    /// Pushes one batch, handling a quota (retry with the tombstones only;
    /// a second 413 stops pushing) and a pause. `None` when nothing was
    /// pushed.
    #[allow(clippy::type_complexity)]
    fn push_batch(
        &mut self,
        batch: Vec<SyncDoc>,
    ) -> Result<Option<(Vec<SyncDoc>, Vec<Pushed>)>, SyncError> {
        // Right before the request: an upload just before it writes nothing
        // locally, so no guarded write would have noticed a re-bind. The
        // guarded `mark_sent` checks the binding again in its transaction,
        // and records what is in flight so that, if the answer is lost, its
        // echo is recognised as ours.
        self.check_bound()?;
        let sent: Vec<(&str, &str)> = batch
            .iter()
            .map(|d| (d.id.as_str(), d.rev.as_str()))
            .collect();
        self.store
            .mark_sent(&sent, Some(&self.guard))
            .map_err(guarded)?;
        match self.sealed.push(&batch) {
            Ok(results) => return Ok(Some((batch, results))),
            Err(ApiError::QuotaExceeded { quota, .. }) => {
                self.report.quota = Some(quota);
                if self.tombstones_only {
                    self.stopped = true;
                    return Ok(None);
                }
                self.tombstones_only = true;
            }
            Err(ApiError::RateLimited { retry_after, .. }) => {
                self.pause(retry_after);
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        }
        let tombstones: Vec<SyncDoc> = batch.into_iter().filter(|d| d.deleted).collect();
        if tombstones.is_empty() {
            return Ok(None);
        }
        self.check_cancel()?;
        self.check_bound()?;
        match self.sealed.push(&tombstones) {
            Ok(results) => Ok(Some((tombstones, results))),
            Err(ApiError::QuotaExceeded { quota, .. }) => {
                self.report.quota = Some(quota);
                self.stopped = true;
                Ok(None)
            }
            Err(ApiError::RateLimited { retry_after, .. }) => {
                self.pause(retry_after);
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Uploads the bytes of the live files in `batch`, so they are on the
    /// server before any device can see the documents. A file whose upload
    /// hits a quota leaves the batch and stays pending; a 429 stops pushing.
    fn upload_files(&mut self, batch: Vec<SyncDoc>) -> Result<Vec<SyncDoc>, SyncError> {
        let mut kept = Vec::with_capacity(batch.len());
        for doc in batch {
            if doc.deleted || !is_file_id(&doc.id) {
                kept.push(doc);
                continue;
            }
            match self.store.attachment(&doc.id)? {
                Some((_, blob)) => {
                    self.check_cancel()?;
                    self.check_bound()?;
                    match self.sealed.put_file(&doc.id, &blob.data) {
                        Ok(()) => {
                            self.report.files_up += 1;
                            kept.push(doc);
                        }
                        Err(ApiError::QuotaExceeded { quota, .. }) => {
                            self.report.quota = Some(quota);
                        }
                        Err(ApiError::RateLimited { retry_after, .. }) => {
                            self.pause(retry_after);
                            return Ok(Vec::new());
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
                None => {
                    self.report.files_missing += 1;
                    kept.push(doc);
                }
            }
        }
        Ok(kept)
    }
}

/// What the sidebar and Settings › Account show for a sync that failed or
/// finished with problems (docs/ACCOUNTS.md › Problems).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SyncProblem {
    /// 401: "Signed out". Stops.
    SignedOut(SignOutReason),
    /// 403: "Account turned off". Stops; Sync now retries.
    AccountDisabled,
    /// A 429 on a push or upload: "Sync paused" until `until` (ms).
    Paused { until: i64 },
    /// A 413 quota (never `Server`): "Account full". Keeps running.
    QuotaExceeded(QuotaKind),
    /// A 413 on a server-wide cap: "Server full". Keeps running.
    ServerFull,
    /// Feed documents skipped unrepaired: "Some items skipped". Keeps running.
    Skipped { count: usize },
    /// A newer format in the feed: "Update Kanso". Stops; Sync now retries.
    UpdateKanso,
    /// The key ID differs: "Keys don't match". Stops.
    WrongKey,
    /// A 404 on a v2 route: "Server needs an update". Stops; Sync now retries.
    ServerOutdated,
    /// No answer: "Offline". Keeps running.
    Offline,
    /// Anything else: "Couldn't sync". Keeps running.
    Other,
}

/// What the sync schedule does after a problem (the table's last column).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Schedule {
    KeepsRunning,
    /// Waits until this time (ms), then runs again.
    WaitsUntil(i64),
    /// Stops; "Sync now" tries again.
    StopsUntilSyncNow,
    /// Stops until the user signs in again (or out and in).
    Stops,
}

impl SyncProblem {
    /// Whether the schedule keeps running after this problem.
    pub fn schedule(&self) -> Schedule {
        match self {
            SyncProblem::SignedOut(_) | SyncProblem::WrongKey => Schedule::Stops,
            SyncProblem::AccountDisabled
            | SyncProblem::UpdateKanso
            | SyncProblem::ServerOutdated => Schedule::StopsUntilSyncNow,
            SyncProblem::Paused { until } => Schedule::WaitsUntil(*until),
            SyncProblem::QuotaExceeded(_)
            | SyncProblem::ServerFull
            | SyncProblem::Skipped { .. }
            | SyncProblem::Offline
            | SyncProblem::Other => Schedule::KeepsRunning,
        }
    }
}

/// The problem a failed sync shows; `None` for [`SyncError::Cancelled`].
/// Pure; implemented in step 1.
pub fn problem(err: &SyncError) -> Option<SyncProblem> {
    Some(match err {
        SyncError::Cancelled => return None,
        SyncError::WrongKey => SyncProblem::WrongKey,
        SyncError::Api(api) => match api {
            ApiError::Unauthorized { reason } => SyncProblem::SignedOut(*reason),
            ApiError::AccountDisabled => SyncProblem::AccountDisabled,
            ApiError::NotFound => SyncProblem::ServerOutdated,
            ApiError::QuotaExceeded {
                quota: QuotaKind::Server,
                ..
            } => SyncProblem::ServerFull,
            ApiError::QuotaExceeded { quota, .. } => SyncProblem::QuotaExceeded(*quota),
            ApiError::Transport(_) => SyncProblem::Offline,
            _ => SyncProblem::Other,
        },
        SyncError::Store(_) => SyncProblem::Other,
    })
}

/// The problem a finished sync shows, in this order of precedence:
/// `UpdateKanso`, `Paused`, `ServerFull`, `QuotaExceeded`, `Skipped`.
/// Rebased stale documents and repairs aren't problems. Pure; implemented
/// in step 1.
pub fn report_problem(report: &SyncReport) -> Option<SyncProblem> {
    if report.too_new {
        return Some(SyncProblem::UpdateKanso);
    }
    if let Some(until) = report.paused_until {
        return Some(SyncProblem::Paused { until });
    }
    match report.quota {
        Some(QuotaKind::Server) => return Some(SyncProblem::ServerFull),
        Some(kind) => return Some(SyncProblem::QuotaExceeded(kind)),
        None => {}
    }
    if !report.skipped.is_empty() {
        return Some(SyncProblem::Skipped {
            count: report.skipped.len(),
        });
    }
    None
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

#[cfg(test)]
mod problem_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn errors_map_to_problems() {
        assert_eq!(problem(&SyncError::Cancelled), None);
        assert_eq!(problem(&SyncError::WrongKey), Some(SyncProblem::WrongKey));
        assert_eq!(
            problem(&SyncError::Api(ApiError::Unauthorized {
                reason: SignOutReason::Evicted
            })),
            Some(SyncProblem::SignedOut(SignOutReason::Evicted))
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::AccountDisabled)),
            Some(SyncProblem::AccountDisabled)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::NotFound)),
            Some(SyncProblem::ServerOutdated)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::Transport("dns".into()))),
            Some(SyncProblem::Offline)
        );
        assert_eq!(
            problem(&SyncError::Api(ApiError::Internal)),
            Some(SyncProblem::Other)
        );
        assert_eq!(
            problem(&SyncError::Store(StoreError::GuardFailed)),
            Some(SyncProblem::Other)
        );
    }

    #[test]
    fn schedules() {
        assert_eq!(
            SyncProblem::SignedOut(SignOutReason::Expired).schedule(),
            Schedule::Stops
        );
        assert_eq!(SyncProblem::WrongKey.schedule(), Schedule::Stops);
        assert_eq!(
            SyncProblem::UpdateKanso.schedule(),
            Schedule::StopsUntilSyncNow
        );
        assert_eq!(
            SyncProblem::Paused { until: 7 }.schedule(),
            Schedule::WaitsUntil(7)
        );
        assert_eq!(SyncProblem::Offline.schedule(), Schedule::KeepsRunning);
        assert_eq!(report_problem(&SyncReport::default()), None);
    }

    fn gen_report() -> impl Strategy<Value = SyncReport> {
        (
            any::<bool>(),
            prop::option::of(0i64..1_000),
            prop::option::of(prop::sample::select(QuotaKind::ALL.to_vec())),
            0..3usize,
            0..3usize,
            0..3usize,
        )
            .prop_map(|(too_new, paused_until, quota, skipped, stale, repaired)| {
                SyncReport {
                    too_new,
                    paused_until,
                    quota,
                    skipped: (0..skipped).map(|i| format!("note:s{i}")).collect(),
                    stale: (0..stale).map(|i| format!("note:t{i}")).collect(),
                    repaired: (0..repaired).map(|i| format!("note:r{i}")).collect(),
                    ..SyncReport::default()
                }
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Model-based: the reported problem is the first that applies, in
        /// the order UpdateKanso, Paused, ServerFull, QuotaExceeded, Skipped;
        /// stale and repaired documents alone are never a problem.
        #[test]
        fn prop_report_problem_follows_the_precedence(report in gen_report()) {
            let mut model = Vec::new();
            if report.too_new { model.push(SyncProblem::UpdateKanso); }
            if let Some(until) = report.paused_until { model.push(SyncProblem::Paused { until }); }
            if report.quota == Some(QuotaKind::Server) { model.push(SyncProblem::ServerFull); }
            if let Some(kind) = report.quota.filter(|k| *k != QuotaKind::Server) { model.push(SyncProblem::QuotaExceeded(kind)); }
            if !report.skipped.is_empty() { model.push(SyncProblem::Skipped { count: report.skipped.len() }); }
            prop_assert_eq!(report_problem(&report), model.first().cloned());
            if let Some(SyncProblem::QuotaExceeded(kind)) = report_problem(&report) {
                prop_assert_ne!(kind, QuotaKind::Server);
            }
        }
    }
}
