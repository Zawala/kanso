//! Between the engine's plaintext documents and the wire: [`Sealed`] seals
//! outgoing documents, opens incoming ones, maps local IDs and revisions to
//! wire ones, and keeps the repair bases of undecryptable server copies
//! (docs/ACCOUNTS.md › Sealed documents, › Sync with encryption).
//!
//! Step 1 froze only [`PlainView`], which the test suites use instead of
//! reading server state directly (docs/ACCOUNTS.md › Tests). `Sealed` is
//! internal.

use std::cell::RefCell;
use std::collections::HashMap;

use kanso_store::SyncDoc;

use crate::account::Session;
use crate::crypto::{ContentKeys, Opened, key_id};
use crate::memory::AccountHandle;
use crate::remote::{PushResult, RemoteDoc, RemoteError};
use crate::wire::{
    ApiError, WireChange, WireDoc, WirePushDoc, WirePushResult, WireRemote, is_hex16,
};

/// One feed document, opened or not.
#[derive(Clone, Debug)]
pub(crate) enum Entry {
    Opened {
        seq: i64,
        doc: SyncDoc,
    },
    /// Format 1, but it didn't open (docs/ACCOUNTS.md › Sealed documents).
    Undecryptable {
        seq: i64,
        wire: WireChange,
    },
    /// A newer format: the pull stops before it.
    TooNew {
        seq: i64,
        wire: WireChange,
    },
}

impl Entry {
    pub(crate) fn seq(&self) -> i64 {
        match self {
            Entry::Opened { seq, .. }
            | Entry::Undecryptable { seq, .. }
            | Entry::TooNew { seq, .. } => *seq,
        }
    }
}

/// One page of the change feed, opened. `epoch` and `key_id` are always
/// there: the wire page requires both, so a server can't skip the checks.
#[derive(Clone, Debug)]
pub(crate) struct Page {
    pub entries: Vec<Entry>,
    pub last_seq: i64,
    pub more: bool,
    pub head_seq: i64,
    pub epoch: String,
    pub key_id: String,
}

/// The outcome of pushing one document, opened.
#[derive(Clone, Debug)]
pub(crate) enum Pushed {
    Ok,
    Conflict(SyncDoc),
    Lost,
    Behind,
    /// The server's current version didn't open; `wire_rev` is its revision.
    Unreadable {
        wire_rev: String,
        newer_format: bool,
    },
}

/// A file's bytes as the server holds them, opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FileBytes {
    Missing,
    Opened(Vec<u8>),
    Unreadable,
}

fn protocol(message: impl Into<String>) -> ApiError {
    ApiError::Protocol(message.into())
}

/// A session's sync routes with its content keys in between.
pub(crate) struct Sealed<'a> {
    wire: Box<dyn WireRemote + 'a>,
    keys: ContentKeys,
    account_id: String,
    key_id: String,
    /// Local ID → the wire revision to push over instead of
    /// `wire_rev(id, base_rev)`: undecryptable server copies that `verify
    /// --repair` replaces from this Mac.
    repair_bases: RefCell<HashMap<String, String>>,
}

impl<'a> Sealed<'a> {
    pub(crate) fn new(wire: Box<dyn WireRemote + 'a>, session: &Session) -> Sealed<'a> {
        Sealed {
            wire,
            keys: ContentKeys::derive(session.data_key()),
            account_id: session.account_id.clone(),
            key_id: key_id(session.data_key()),
            repair_bases: RefCell::default(),
        }
    }

    /// This Mac's key ID for the account.
    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    pub(crate) fn wire_id(&self, local_id: &str) -> String {
        self.keys.wire_id(local_id)
    }

    pub(crate) fn wire_rev(&self, local_id: &str, local_rev: &str) -> String {
        self.keys.wire_rev(local_id, local_rev)
    }

    /// Pushes `local_id` over `wire_rev` next time, whatever its `base_rev`.
    pub(crate) fn set_repair_base(&self, local_id: &str, wire_rev: &str) {
        self.repair_bases
            .borrow_mut()
            .insert(local_id.to_owned(), wire_rev.to_owned());
    }

    /// `GET /v2/changes`, opened. Refuses a page that breaks the feed's
    /// shape: sequences not increasing past `since`, a `last_seq` that
    /// isn't the last document's, more documents than asked for, `more`
    /// on an empty page, a document past `head_seq`, or an epoch or key ID
    /// that isn't 16 hex digits.
    pub(crate) fn changes(&self, since: i64, limit: usize) -> Result<Page, ApiError> {
        let page = self.wire.changes(since, limit)?;
        if !is_hex16(&page.epoch) || !is_hex16(&page.key_id) {
            return Err(protocol("the page's epoch or key ID isn't 16 hex digits"));
        }
        if page.docs.len() > limit {
            return Err(protocol(format!(
                "asked for {limit} documents and got {}",
                page.docs.len()
            )));
        }
        let mut last = since;
        let mut entries = Vec::with_capacity(page.docs.len());
        for change in page.docs {
            if change.seq <= last {
                return Err(protocol(format!(
                    "the change feed went from seq {last} to {}",
                    change.seq
                )));
            }
            last = change.seq;
            let wire = WireDoc {
                id: change.id.clone(),
                rev: change.rev.clone(),
                deleted: change.deleted,
                sealed: change.sealed.clone(),
            };
            entries.push(match self.keys.open_doc(&self.account_id, &wire) {
                Opened::Opened(doc) => Entry::Opened {
                    seq: change.seq,
                    doc,
                },
                Opened::Undecryptable => Entry::Undecryptable {
                    seq: change.seq,
                    wire: change,
                },
                Opened::TooNew => Entry::TooNew {
                    seq: change.seq,
                    wire: change,
                },
            });
        }
        if page.last_seq != last {
            return Err(protocol(format!(
                "the page ends at seq {last} but says {}",
                page.last_seq
            )));
        }
        if page.more && entries.is_empty() {
            return Err(protocol("an empty page says more follow"));
        }
        // An empty page ends at `since`, which a rewound account's head
        // can be below; a document can't be past the head.
        if !entries.is_empty() && page.head_seq < page.last_seq {
            return Err(protocol(format!(
                "the head seq {} is below the page's {}",
                page.head_seq, page.last_seq
            )));
        }
        Ok(Page {
            entries,
            last_seq: page.last_seq,
            more: page.more,
            head_seq: page.head_seq,
            epoch: page.epoch,
            key_id: page.key_id,
        })
    }

    /// Seals `docs` and pushes them: one result per document, in order. A
    /// document's `base_rev` (local) goes as `wire_rev(id, base_rev)`, or as
    /// its repair base.
    pub(crate) fn push(&self, docs: &[SyncDoc]) -> Result<Vec<Pushed>, ApiError> {
        let sealed: Vec<WirePushDoc> = docs.iter().map(|doc| self.seal(doc)).collect();
        let results = self.wire.push(&sealed)?;
        if results.len() != docs.len() {
            return Err(protocol(format!(
                "a push of {} documents got {} results",
                docs.len(),
                results.len()
            )));
        }
        let mut out = Vec::with_capacity(results.len());
        for ((doc, wire), result) in docs.iter().zip(&sealed).zip(results) {
            if result.id() != wire.id {
                return Err(protocol(format!(
                    "the result for {} names {}",
                    wire.id,
                    result.id()
                )));
            }
            out.push(match result {
                WirePushResult::Ok { .. } => {
                    self.repair_bases.borrow_mut().remove(&doc.id);
                    Pushed::Ok
                }
                WirePushResult::Conflict { current, .. } => {
                    if current.id != wire.id {
                        return Err(protocol(format!(
                            "the conflict for {} carries {}",
                            wire.id, current.id
                        )));
                    }
                    match self.keys.open_doc(&self.account_id, &current) {
                        Opened::Opened(current) if current.id == doc.id => {
                            Pushed::Conflict(current)
                        }
                        Opened::Opened(_) | Opened::Undecryptable => Pushed::Unreadable {
                            wire_rev: current.rev,
                            newer_format: false,
                        },
                        Opened::TooNew => Pushed::Unreadable {
                            wire_rev: current.rev,
                            newer_format: true,
                        },
                    }
                }
                WirePushResult::Lost { .. } => Pushed::Lost,
                WirePushResult::Behind { .. } => Pushed::Behind,
            });
        }
        Ok(out)
    }

    fn seal(&self, doc: &SyncDoc) -> WirePushDoc {
        let wire = self.keys.seal_doc(&self.account_id, doc);
        let base_rev = match self.repair_bases.borrow().get(&doc.id) {
            Some(base) => Some(base.clone()),
            None => doc
                .base_rev
                .as_deref()
                .map(|base| self.keys.wire_rev(&doc.id, base)),
        };
        WirePushDoc {
            id: wire.id,
            rev: wire.rev,
            deleted: wire.deleted,
            sealed: wire.sealed,
            base_rev,
        }
    }

    /// Seals and uploads the bytes of file document `local_id`.
    pub(crate) fn put_file(&self, local_id: &str, bytes: &[u8]) -> Result<(), ApiError> {
        let wire_id = self.keys.wire_id(local_id);
        let sealed = self.keys.seal_file(&self.account_id, &wire_id, bytes);
        self.wire.put_file(&wire_id, &sealed)
    }

    /// Fetches and opens the bytes of file document `local_id`.
    pub(crate) fn get_file(&self, local_id: &str) -> Result<FileBytes, ApiError> {
        let wire_id = self.keys.wire_id(local_id);
        Ok(match self.wire.get_file(&wire_id)? {
            None => FileBytes::Missing,
            Some(sealed) => match self.keys.open_file(&self.account_id, &wire_id, &sealed) {
                Opened::Opened(bytes) => FileBytes::Opened(bytes),
                Opened::Undecryptable | Opened::TooNew => FileBytes::Unreadable,
            },
        })
    }
}

/// The decrypted view of one account on a [`crate::MemoryServer`], under a
/// test session: what the server holds, opened, and a way to push plaintext
/// as another client of the same account. Tests only.
pub struct PlainView {
    handle: AccountHandle,
    keys: ContentKeys,
    account_id: String,
}

impl PlainView {
    pub(crate) fn new(handle: AccountHandle, session: &Session) -> PlainView {
        PlainView {
            handle,
            keys: ContentKeys::derive(session.data_key()),
            account_id: session.account_id.clone(),
        }
    }

    /// Every document of the account, opened, tombstones included, in `seq`
    /// order, with local IDs and revisions. Rows that don't open under the
    /// session's keys are left out ([`PlainView::unreadable`] counts them).
    pub fn docs(&self) -> Vec<RemoteDoc> {
        self.handle
            .docs()
            .into_iter()
            .filter_map(|change| {
                let wire = WireDoc {
                    id: change.id,
                    rev: change.rev,
                    deleted: change.deleted,
                    sealed: change.sealed,
                };
                match self.keys.open_doc(&self.account_id, &wire) {
                    Opened::Opened(doc) => Some(RemoteDoc {
                        id: doc.id,
                        rev: doc.rev,
                        deleted: doc.deleted,
                        fields: doc.fields,
                        seq: change.seq,
                    }),
                    Opened::Undecryptable | Opened::TooNew => None,
                }
            })
            .collect()
    }

    /// The account's rows as the server stores them, sealed, in `seq` order.
    pub fn wire_docs(&self) -> Vec<WireChange> {
        self.handle.docs()
    }

    /// How many of the account's rows don't open under the session's keys.
    pub fn unreadable(&self) -> usize {
        self.handle
            .docs()
            .into_iter()
            .filter(|change| {
                let wire = WireDoc {
                    id: change.id.clone(),
                    rev: change.rev.clone(),
                    deleted: change.deleted,
                    sealed: change.sealed.clone(),
                };
                !matches!(
                    self.keys.open_doc(&self.account_id, &wire),
                    Opened::Opened(_)
                )
            })
            .count()
    }

    /// The opened bytes of the file document `local_id`, if the server has them.
    pub fn file(&self, local_id: &str) -> Option<Vec<u8>> {
        let wire_id = self.keys.wire_id(local_id);
        let sealed = self.handle.file(&wire_id)?;
        match self.keys.open_file(&self.account_id, &wire_id, &sealed) {
            Opened::Opened(bytes) => Some(bytes),
            Opened::Undecryptable | Opened::TooNew => None,
        }
    }

    /// Seals and uploads file bytes for `local_id` as another device of the
    /// account.
    pub fn put_file(&self, local_id: &str, bytes: &[u8]) -> Result<(), RemoteError> {
        let wire_id = self.keys.wire_id(local_id);
        let sealed = self.keys.seal_file(&self.account_id, &wire_id, bytes);
        self.handle
            .put_file(&wire_id, &sealed)
            .map_err(remote_error)
    }

    /// Seals and pushes plaintext documents as another device of the account.
    pub fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let sealed: Vec<WirePushDoc> = docs
            .iter()
            .map(|doc| {
                let wire = self.keys.seal_doc(&self.account_id, doc);
                WirePushDoc {
                    id: wire.id,
                    rev: wire.rev,
                    deleted: wire.deleted,
                    sealed: wire.sealed,
                    base_rev: doc
                        .base_rev
                        .as_deref()
                        .map(|base| self.keys.wire_rev(&doc.id, base)),
                }
            })
            .collect();
        let results = self.handle.push(&sealed).map_err(remote_error)?;
        Ok(docs
            .iter()
            .zip(results)
            .map(|(doc, result)| {
                let id = doc.id.clone();
                match result {
                    WirePushResult::Ok { seq, .. } => PushResult::Ok { id, seq },
                    WirePushResult::Conflict { current, .. } => {
                        match self.keys.open_doc(&self.account_id, &current) {
                            Opened::Opened(current) => PushResult::Conflict {
                                id,
                                current: Some(current),
                            },
                            Opened::Undecryptable => PushResult::Unreadable {
                                id,
                                newer_format: false,
                            },
                            Opened::TooNew => PushResult::Unreadable {
                                id,
                                newer_format: true,
                            },
                        }
                    }
                    WirePushResult::Lost { .. } => PushResult::Lost { id },
                    WirePushResult::Behind { .. } => PushResult::Behind { id },
                }
            })
            .collect())
    }

    /// The wire ID of a local ID under the account's keys.
    pub fn wire_id(&self, local_id: &str) -> String {
        self.keys.wire_id(local_id)
    }

    /// The wire revision of a local revision under the account's keys.
    pub fn wire_rev(&self, local_id: &str, local_rev: &str) -> String {
        self.keys.wire_rev(local_id, local_rev)
    }
}

/// A v2 error in the plaintext [`RemoteError`] shape.
fn remote_error(err: ApiError) -> RemoteError {
    match err {
        ApiError::Unauthorized { .. } => RemoteError::Unauthorized,
        ApiError::Transport(message) => RemoteError::Transport(message),
        ApiError::Protocol(message) => RemoteError::Protocol(message),
        other => RemoteError::Status {
            status: other.status().unwrap_or(0),
            message: other.code().unwrap_or_default().to_owned(),
        },
    }
}
