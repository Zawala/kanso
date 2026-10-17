//! An in-memory sync server with the Worker's semantics, for tests.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use kanso_store::SyncDoc;
use serde_json::{Map, Value};

use crate::remote::{
    ChangesPage, MAX_CHANGES_LIMIT, MAX_FILE_BYTES, MAX_PUSH_BYTES, PushResult, Remote, RemoteDoc,
    RemoteError, RemoteFile, is_file_id, is_md5_hex, md5_hex, validate_push,
};

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
    /// The change-sequence counter: the seq of the last accepted write.
    counter: i64,
    files: HashMap<String, RemoteFile>,
    drop_push_responses: usize,
    stats: MemoryStats,
}

/// Request counts, for tests that check what a sync sent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryStats {
    pub pushes: usize,
    pub pushed_docs: usize,
    pub changes: usize,
    pub file_puts: usize,
    pub file_gets: usize,
}

/// A sync server held in memory. It follows the Worker's rules exactly: a
/// pushed document is accepted iff its `base_rev` is the server's current
/// revision (or the server has none and `base_rev` is null); every accepted
/// write takes the next value of one global sequence; documents in a push are
/// applied in order; accepting a file tombstone drops the file's bytes.
#[derive(Default)]
pub struct MemoryRemote {
    state: Mutex<State>,
}

impl MemoryRemote {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("memory remote poisoned")
    }

    /// Every document the server has, including tombstones, in `seq` order.
    pub fn docs(&self) -> Vec<RemoteDoc> {
        let state = self.state();
        let mut docs: Vec<RemoteDoc> = state.docs.iter().map(|(id, d)| remote_doc(id, d)).collect();
        docs.sort_by_key(|d| d.seq);
        docs
    }

    /// The highest change sequence handed out so far.
    pub fn last_seq(&self) -> i64 {
        self.state().counter
    }

    /// The stored bytes of attachment `id`.
    pub fn file(&self, id: &str) -> Option<RemoteFile> {
        self.state().files.get(id).cloned()
    }

    pub fn stats(&self) -> MemoryStats {
        self.state().stats
    }

    /// The next `n` pushes are applied but answer with a transport error, as
    /// when the connection drops after the server committed.
    pub fn drop_next_push_responses(&self, n: usize) {
        self.state().drop_push_responses = n;
    }
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

fn status(status: u16, message: impl Into<String>) -> RemoteError {
    RemoteError::Status {
        status,
        message: message.into(),
    }
}

impl Remote for MemoryRemote {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let body = serde_json::to_vec(&serde_json::json!({ "docs": docs }))
            .map_err(|e| RemoteError::Protocol(e.to_string()))?;
        if body.len() > MAX_PUSH_BYTES {
            return Err(status(413, "The push is too large."));
        }
        validate_push(docs).map_err(|(code, message)| status(code, message))?;

        let mut state = self.state();
        state.stats.pushes += 1;
        state.stats.pushed_docs += docs.len();
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
            if doc.deleted && is_file_id(&doc.id) {
                state.files.remove(&doc.id);
            }
            results.push(PushResult::Ok {
                id: doc.id.clone(),
                seq,
            });
        }
        if state.drop_push_responses > 0 {
            state.drop_push_responses -= 1;
            return Err(RemoteError::Transport(
                "connection reset (simulated)".into(),
            ));
        }
        Ok(results)
    }

    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        if since < 0 {
            return Err(status(400, "since must be a non-negative integer."));
        }
        if !(1..=MAX_CHANGES_LIMIT).contains(&limit) {
            return Err(status(
                400,
                format!("limit must be between 1 and {MAX_CHANGES_LIMIT}."),
            ));
        }
        let mut state = self.state();
        state.stats.changes += 1;
        let mut docs: Vec<RemoteDoc> = state
            .docs
            .iter()
            .filter(|(_, d)| d.seq > since)
            .map(|(id, d)| remote_doc(id, d))
            .collect();
        docs.sort_by_key(|d| d.seq);
        docs.truncate(limit);
        let last_seq = docs.last().map_or(since, |d| d.seq);
        Ok(ChangesPage { docs, last_seq })
    }

    fn put_file(
        &self,
        id: &str,
        claimed_md5: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError> {
        if !is_file_id(id) {
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
        let mut state = self.state();
        state.stats.file_puts += 1;
        state.files.insert(
            id.to_owned(),
            RemoteFile {
                md5_hex: actual,
                content_type: if content_type.is_empty() {
                    "application/octet-stream".into()
                } else {
                    content_type.to_owned()
                },
                bytes: bytes.to_vec(),
            },
        );
        Ok(())
    }

    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        if !is_file_id(id) {
            return Err(status(400, "Not a file ID."));
        }
        let mut state = self.state();
        state.stats.file_gets += 1;
        Ok(state.files.get(id).cloned())
    }
}
