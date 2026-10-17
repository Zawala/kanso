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
    /// The local copy already had this revision.
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

    /// Records that the server now has revision `rev` of `id`. Does nothing
    /// if the document changed again in the meantime.
    pub fn mark_synced(&self, id: &str, rev: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "UPDATE docs SET synced_rev = ?2 WHERE id = ?1 AND rev = ?2",
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
                tx.execute(
                    "UPDATE docs SET synced_rev = rev WHERE id = ?",
                    [&remote.id],
                )?;
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
