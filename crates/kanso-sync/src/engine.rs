//! One sync pass: pull the server's changes, then push local ones; and what
//! a sync reports (docs/ACCOUNTS.md › Sync with encryption, › Problems).
//!
//! `sync_once`/`sync_with` are today's engine over the plaintext [`Remote`]
//! (v1 rules, legacy cursor). Step 2b runs it over `Sealed<W>` and adds the
//! v2 rules; [`sync`] is the app's entry point. [`SyncReport`],
//! [`SyncOptions`], [`SyncError`], [`SyncProblem`], [`problem`] and
//! [`report_problem`] are frozen by step 1.

use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_store::{ApplyOutcome, Attachment, Store, StoreError, SyncDoc};
use serde_json::Value;

use crate::account::Session;
use crate::api::Backend;
use crate::binding::meta_keys;
use crate::remote::{MAX_FIELDS_BYTES, PushResult, Remote, RemoteError, is_file_id, md5_hex};
use crate::wire::{ApiError, QuotaKind, SignOutReason};

/// The `meta` key holding the last change sequence pulled from a v1 server.
/// `sync_once`/`sync_with` use it until step 2b; v2 uses
/// [`SyncOptions::cursor_key`].
pub const CURSOR_KEY: &str = meta_keys::LEGACY_LAST_SEQ;

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
    /// The `meta` key of the cursor: `sync.v2.last_seq`. Only the rollback
    /// test sets the legacy `sync.last_seq`, to run the old rules.
    pub cursor_key: &'static str,
}

impl Default for SyncOptions {
    fn default() -> Self {
        SyncOptions {
            page_size: 100,
            batch_docs: 100,
            batch_bytes: 1024 * 1024,
            max_rounds: 4,
            cancel: None,
            cursor_key: meta_keys::LAST_SEQ,
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
    /// Documents too large for the server (more than [`MAX_FIELDS_BYTES`]
    /// today; `fields` JSON over 1,400,000 bytes under v2).
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
    /// From the plaintext [`Remote`] (today's v1 servers and test remotes).
    #[error(transparent)]
    Remote(#[from] RemoteError),
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

/// The app's sync: one pass of `session`'s account over `b`, with every v2
/// rule (docs/ACCOUNTS.md › Sync with encryption). Step 2b.
pub fn sync(
    store: &Store,
    b: &dyn Backend,
    session: &Session,
    options: &SyncOptions,
) -> Result<SyncReport, SyncError> {
    crate::stub!("2b", store, b, session, options)
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
        SyncError::Remote(RemoteError::Unauthorized) => {
            SyncProblem::SignedOut(SignOutReason::Unknown)
        }
        SyncError::Remote(RemoteError::Transport(_)) => SyncProblem::Offline,
        SyncError::Remote(_) | SyncError::Store(_) => SyncProblem::Other,
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

/// Pulls, then pushes, with the default options.
///
/// Legacy (v1 rules, plaintext, one shared token): kept until step 3 of
/// docs/ACCOUNTS.md › Build order; accounts use [`sync()`].
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
///
/// Legacy (v1): see [`sync_once`].
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
    let outcome = match store.apply_remote_legacy(doc, attachment) {
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
        ApplyOutcome::LocalWon { .. } => {
            report.conflicts_local += 1;
            report.note_conflict(&doc.id);
        }
        ApplyOutcome::Stale { .. } => report.stale.push(doc.id.clone()),
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
    let mut lost = BTreeSet::new();
    for _ in 0..options.max_rounds.max(1) {
        let mut docs = Vec::new();
        let mut sizes = Vec::new();
        report.too_large.clear();
        for doc in store.pending_push(usize::MAX)? {
            if lost.contains(&doc.id) {
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
                        store.mark_synced(&doc.id, &doc.rev, None)?;
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
                        // The server lost it. Under v2 this is `Lost`: the
                        // engine marks it unsynced and pushes it again with
                        // no base (step 2b). Today it stays pending.
                        lost.insert(doc.id.clone());
                    }
                }
            }
        }
        if conflicts == 0 {
            break;
        }
    }
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
            problem(&SyncError::Remote(RemoteError::Transport("dns".into()))),
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
