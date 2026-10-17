//! Sync bookkeeping. Each document remembers the revision last known to
//! match the server (`synced_rev`); a document whose current revision differs
//! has local changes to push.

use kanso_core::DocType;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::doc::Attachment;
use crate::validate::{Refs, normalize};
use crate::write::{Synced, load_row, store_row};
use crate::{Origin, Result, Store, StoreError};

/// A document as exchanged with the sync server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncDoc {
    pub id: String,
    pub rev: String,
    #[serde(default)]
    pub deleted: bool,
    /// The document's fields; empty for deletions.
    #[serde(default)]
    pub fields: Map<String, Value>,
    /// The server revision this change was made on top of; `None` for a
    /// document the server has never seen.
    #[serde(default)]
    pub base_rev: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The remote version replaced the local one.
    Applied,
    /// The local copy already had this revision, or its pending changes are
    /// based on it.
    Unchanged,
    /// Both sides changed; the remote version won and the local one was kept
    /// as a snapshot.
    RemoteWon,
    /// Both sides changed; the local version won and will be pushed. The
    /// remote one was kept as a snapshot.
    LocalWon,
}

fn updated_at(fields: &Map<String, Value>) -> i64 {
    fields.get("updatedAt").and_then(Value::as_i64).unwrap_or(0)
}

impl Store {
    /// Local changes the server doesn't have yet, oldest first.
    pub fn pending_push(&self, limit: usize) -> Result<Vec<SyncDoc>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, rev, deleted, json, synced_rev FROM docs
                 WHERE synced_rev IS NULL OR synced_rev != rev ORDER BY seq LIMIT ?",
            )?;
            let rows = stmt.query_map([limit as i64], |row| {
                let json: String = row.get(3)?;
                Ok(SyncDoc {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    deleted: row.get(2)?,
                    fields: match serde_json::from_str(&json) {
                        Ok(Value::Object(map)) => map,
                        _ => Map::new(),
                    },
                    base_rev: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Records that the server accepted revision `rev` of `id`. Call it right
    /// after the push that sent `rev` succeeded.
    ///
    /// If the document was edited locally while the push was in flight, the
    /// edit stays pending, now based on `rev`: local writes only ever build
    /// on the current revision, so the newer one descends from what the
    /// server now has and its next push is accepted.
    pub fn mark_synced(&self, id: &str, rev: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE docs SET synced_rev = ?2 WHERE id = ?1",
                params![id, rev],
            )?;
            Ok(())
        })
    }

    /// Whether the store has the bytes of attachment `id` with this MD5 (hex).
    pub fn has_blob(&self, id: &str, md5_hex: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM blobs WHERE file_id = ? AND md5 = ?",
                    [id, md5_hex],
                    |_| Ok(()),
                )
                .map(|_| true)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(false),
                    e => Err(e),
                })?)
        })
    }

    /// Applies a document received from the server. When both sides changed
    /// it, the newer `updatedAt` wins (an edit always beats a deletion) and the
    /// losing version is kept in the document's revision history.
    pub fn apply_remote(
        &self,
        remote: &SyncDoc,
        attachment: Option<Attachment>,
    ) -> Result<ApplyOutcome> {
        let doc_type = DocType::of_id(&remote.id)
            .ok_or_else(|| StoreError::Invalid(format!("Invalid document ID '{}'.", remote.id)))?;
        remote
            .rev
            .parse::<kanso_core::Rev>()
            .map_err(|e| StoreError::Invalid(e.to_string()))?;
        self.write(|tx, changes| {
            let local = load_row(tx, &remote.id)?;
            let fields = if remote.deleted {
                Map::new()
            } else {
                let created = remote.fields.get("createdAt").and_then(Value::as_i64).unwrap_or(0);
                normalize(doc_type, &remote.id, remote.fields.clone(), None, created, Refs::Skip)?
            };
            let store_blob = |tx: &rusqlite::Transaction| -> Result<()> {
                if let Some(blob) = &attachment {
                    tx.execute(
                        "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                         ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                        params![remote.id, blob.md5_hex(), blob.data],
                    )?;
                }
                Ok(())
            };
            let Some(local) = local else {
                store_blob(tx)?;
                store_row(tx, &remote.id, doc_type, &remote.rev, remote.deleted, &fields, None, Synced::Set(&remote.rev), Origin::Sync, changes)?;
                return Ok(ApplyOutcome::Applied);
            };
            if local.rev == remote.rev {
                // Same revision, same content: bytes that arrive with it are
                // this document's (e.g. fetched after an earlier miss).
                if !remote.deleted {
                    store_blob(tx)?;
                }
                tx.execute(
                    "UPDATE docs SET synced_rev = rev WHERE id = ?",
                    [&remote.id],
                )?;
                return Ok(ApplyOutcome::Unchanged);
            }
            // The server's version is the one the local edits are based on
            // (typically our own push coming back through the change feed):
            // nothing new on the remote side, so it is not a conflict.
            if local.synced_rev.as_deref() == Some(remote.rev.as_str()) {
                return Ok(ApplyOutcome::Unchanged);
            }
            let local_dirty = local.synced_rev.as_deref() != Some(local.rev.as_str());
            let remote_wins = !local_dirty
                || match (local.deleted, remote.deleted) {
                    (true, false) => true,
                    (false, true) => false,
                    (true, true) => true,
                    (false, false) => updated_at(&remote.fields) >= updated_at(&local.fields),
                };
            if remote_wins {
                store_blob(tx)?;
                store_row(tx, &remote.id, doc_type, &remote.rev, remote.deleted, &fields, Some(&local), Synced::Set(&remote.rev), Origin::Sync, changes)?;
                return Ok(if local_dirty { ApplyOutcome::RemoteWon } else { ApplyOutcome::Applied });
            }
            // Keep the local version; base it on the server's revision so the
            // next push is accepted, and keep the remote version as history.
            if !remote.deleted {
                tx.execute(
                    "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, 'sync')",
                    params![remote.id, remote.rev, serde_json::to_string(&fields)?, kanso_core::now_ms()],
                )?;
            }
            tx.execute(
                "UPDATE docs SET synced_rev = ?2 WHERE id = ?1",
                params![remote.id, remote.rev],
            )?;
            Ok(ApplyOutcome::LocalWon)
        })
    }
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use serde_json::json;

    use super::*;
    use crate::Put;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn pending(store: &Store, id: &str) -> Option<SyncDoc> {
        store
            .pending_push(usize::MAX)
            .unwrap()
            .into_iter()
            .find(|d| d.id == id)
    }

    fn create_note(store: &Store, id: &str) -> crate::WriteResult {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "bookId": "trash", "body": "v1", "updatedAt": 100}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
    }

    fn merge_body(store: &Store, id: &str, body: &str) -> crate::WriteResult {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": body}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap()
    }

    /// Pushing R1, editing to R2 and then pulling R1 back through the change
    /// feed must keep R2, even when both share `updatedAt` (merges keep it).
    #[test]
    fn own_push_echo_is_not_a_conflict() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:echo1");
        let pushed = pending(&store, &note.id).unwrap();
        store.mark_synced(&note.id, &pushed.rev).unwrap();

        let edited = merge_body(&store, &note.id, "v2");
        assert_eq!(store.note(&note.id).unwrap().unwrap().updated_at, 100);

        let echo = SyncDoc {
            base_rev: None,
            ..pushed.clone()
        };
        assert_eq!(
            store.apply_remote(&echo, None).unwrap(),
            ApplyOutcome::Unchanged
        );
        assert_eq!(store.note(&note.id).unwrap().unwrap().body, "v2");
        let still = pending(&store, &note.id).unwrap();
        assert_eq!(still.rev, edited.rev);
        assert_eq!(still.base_rev.as_deref(), Some(pushed.rev.as_str()));
        assert!(
            store
                .revisions(&note.id)
                .unwrap()
                .iter()
                .all(|r| r.origin != "sync"),
            "an echo must not be recorded as a conflicting version"
        );
    }

    /// An edit made while a push was in flight stays pending, based on the
    /// revision the server accepted.
    #[test]
    fn mark_synced_rebases_an_edit_made_during_the_push() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:race1");
        let pushed = pending(&store, &note.id).unwrap();
        assert_eq!(pushed.base_rev, None);

        let edited = merge_body(&store, &note.id, "v2");
        store.mark_synced(&note.id, &pushed.rev).unwrap();

        let next = pending(&store, &note.id).unwrap();
        assert_eq!(next.rev, edited.rev);
        assert_eq!(next.base_rev.as_deref(), Some(pushed.rev.as_str()));

        store.mark_synced(&note.id, &edited.rev).unwrap();
        assert!(pending(&store, &note.id).is_none());
    }

    /// A file first applied without its bytes (the server had none yet)
    /// takes them when the same revision arrives again with them.
    #[test]
    fn same_revision_with_bytes_fills_a_missing_blob() {
        let (_dir, store) = store();
        let bytes = b"\x89PNG fake".to_vec();
        let blob = Attachment::new(bytes.clone());
        let doc = SyncDoc {
            id: "file:img1".into(),
            rev: "1-aa".into(),
            deleted: false,
            fields: json!({"name": "a.png", "contentType": "image/png", "md5digest": blob.md5_hex(), "createdAt": 1})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        };
        assert_eq!(
            store.apply_remote(&doc, None).unwrap(),
            ApplyOutcome::Applied
        );
        assert!(store.attachment(&doc.id).unwrap().is_none());
        assert!(!store.has_blob(&doc.id, &blob.md5_hex()).unwrap());

        assert_eq!(
            store.apply_remote(&doc, Some(blob.clone())).unwrap(),
            ApplyOutcome::Unchanged
        );
        let (content_type, stored) = store.attachment(&doc.id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(stored.data, bytes);
        assert!(store.has_blob(&doc.id, &blob.md5_hex()).unwrap());
        assert!(store.pending_push(10).unwrap().is_empty());
    }
}
