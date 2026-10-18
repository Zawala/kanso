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

/// A condition every sync write checks inside its own write transaction:
/// every `(meta key, value)` pair in `expect` must hold. The engine passes the
/// library's binding, `[("sync.v2.origin", origin), ("sync.v2.account_id",
/// account_id)]`, so a sync that started under one account can never write
/// after the library was re-bound to another account or server
/// (docs/ACCOUNTS.md › Sync with encryption › The guard; fix 2 / M6; security
/// re-check 3). A mismatch fails with [`StoreError::GuardFailed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncGuard<'a> {
    pub expect: &'a [(&'a str, &'a str)],
}

/// What [`Store::apply_remote`] did with a server document
/// (docs/ACCOUNTS.md › Library binding › kanso-store APIs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The remote version replaced the local one.
    Applied,
    /// The local copy already had this revision, or its pending changes are
    /// based on it.
    Unchanged,
    /// Both sides changed; the remote version won and the local one was kept
    /// as a snapshot.
    RemoteWon,
    /// Both sides changed; the local version won and will be pushed with
    /// `base_rev` = the remote revision. The remote one was kept as a snapshot.
    ///
    /// `new_rev` is the local version's revision after the conflict. Under M2
    /// (docs/ACCOUNTS.md › Revision generations only grow) it is a new
    /// revision with generation `max(local, remote) + 1` and the same content
    /// hash. Until step 2a adds that bump it is the unchanged local revision.
    LocalWon { new_rev: String },
    /// A replayed older version (M2, security re-check 4): the remote
    /// revision's generation is at or below the generation of the local
    /// `synced_rev`, and it isn't that same revision. It never reverts the
    /// Mac. Instead the local version is rebased onto it like a local win:
    /// it gets `new_rev` (generation `max(local, remote) + 1`, same content
    /// hash), `synced_rev` becomes the remote revision, the remote version is
    /// kept as a snapshot when its content differs, and the local version is
    /// pushed again. Step 2a returns it; until then it is never produced.
    Stale { new_rev: String },
}

/// Fails with [`StoreError::GuardFailed`] unless `guard` (when given) holds.
/// Runs inside the caller's write transaction, so it can't race a re-bind.
fn check_guard(tx: &rusqlite::Transaction, guard: Option<&SyncGuard>) -> Result<()> {
    for (key, value) in guard.map_or(&[][..], |g| g.expect) {
        if crate::schema::get_meta(tx, key)?.as_deref() != Some(*value) {
            return Err(StoreError::GuardFailed);
        }
    }
    Ok(())
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

    /// The number of documents with changes that never reached any server:
    /// the length of [`Store::pending_push`]. `binding::local_library` reports
    /// it as `unsynced`.
    pub fn pending_count(&self) -> Result<usize> {
        self.read(|conn| {
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM docs WHERE synced_rev IS NULL OR synced_rev != rev",
                [],
                |row| row.get(0),
            )?;
            Ok(count as usize)
        })
    }

    /// Records that the server accepted revision `rev` of `id`. Call it right
    /// after the push that sent `rev` succeeded. With a `guard`, nothing is
    /// written unless it holds ([`StoreError::GuardFailed`]).
    ///
    /// If the document was edited locally while the push was in flight, the
    /// edit stays pending, now based on `rev`: local writes only ever build
    /// on the current revision, so the newer one descends from what the
    /// server now has and its next push is accepted.
    pub fn mark_synced(&self, id: &str, rev: &str, guard: Option<&SyncGuard>) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            tx.execute(
                "UPDATE docs SET synced_rev = ?2 WHERE id = ?1",
                params![id, rev],
            )?;
            Ok(())
        })
    }

    /// Forgets that `id` was ever on the server: its `synced_rev` becomes NULL,
    /// so it is pending and its next push carries `base_rev = null`. The
    /// engine calls it for a document the server lost (`Lost`, M1). A repair
    /// of an undecryptable server copy (M3) instead pushes a new generation
    /// on the broken revision (docs/ACCOUNTS.md › Sync with encryption). With
    /// a `guard`, nothing is written unless it holds.
    pub fn mark_unsynced(&self, id: &str, guard: Option<&SyncGuard>) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            tx.execute("UPDATE docs SET synced_rev = NULL WHERE id = ?", [id])?;
            Ok(())
        })
    }

    /// After a re-merge's full pull (M1, security re-check 4): every document
    /// that has a `synced_rev` (tombstones included) but whose ID isn't in
    /// `present`, the local IDs the server's whole feed carried, gets
    /// `synced_rev = NULL`, so it is pushed again with `base_rev = null`.
    /// Returns those IDs, sorted. Nothing is written unless `guard` holds.
    pub fn mark_missing_unsynced(
        &self,
        present: &std::collections::HashSet<String>,
        guard: Option<&SyncGuard>,
    ) -> Result<Vec<String>> {
        self.write(|tx, _| {
            check_guard(tx, guard)?;
            let synced: Vec<String> = tx
                .prepare("SELECT id FROM docs WHERE synced_rev IS NOT NULL ORDER BY id")?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let missing: Vec<String> = synced
                .into_iter()
                .filter(|id| !present.contains(id))
                .collect();
            for id in &missing {
                tx.execute("UPDATE docs SET synced_rev = NULL WHERE id = ?", [id])?;
            }
            Ok(missing)
        })
    }

    /// Writes a `meta` entry only while `guard` holds, checked in the same
    /// transaction. The engine saves the cursor with it after each page.
    pub fn set_meta_if(&self, key: &str, value: &str, guard: &SyncGuard) -> Result<()> {
        self.write(|tx, _| {
            check_guard(tx, Some(guard))?;
            crate::schema::set_meta(tx, key, value)
        })
    }

    /// Prepares the library for a different sync binding, in one transaction
    /// (docs/ACCOUNTS.md › Library binding): live documents get
    /// `synced_rev = NULL`; tombstones get `synced_rev = rev`, so they are
    /// never pushed to an account that never had them; then each meta entry
    /// is written (`Some`) or removed (`None`). Used by bind fresh, merge
    /// (add) and unbind: binding to a different account. A re-merge with the
    /// same account (M1) doesn't use it; it keeps `synced_rev` (security
    /// re-check 4).
    pub fn reset_sync(&self, meta: &[(&str, Option<&str>)]) -> Result<()> {
        let _ = meta;
        todo!("step 2a: docs/ACCOUNTS.md › Library binding › reset_sync")
    }

    /// Empties the library in one transaction (docs/ACCOUNTS.md › Library
    /// binding › Replace, fix 3): removes every document, note-tag row, blob,
    /// revision, FTS row and archive row, then writes (`Some`) or removes
    /// (`None`) the meta entries. Broadcasts a deletion per document
    /// (`Origin::Sync`). Returns the number of documents removed. A crash
    /// leaves either the old library or an empty one bound fresh.
    pub fn replace_library(&self, meta: &[(&str, Option<&str>)]) -> Result<usize> {
        let _ = meta;
        todo!("step 2a: docs/ACCOUNTS.md › Library binding › replace_library")
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
    /// losing version is kept in the document's revision history. With a
    /// `guard`, nothing is written unless it holds ([`StoreError::GuardFailed`]).
    ///
    /// Step 2a adds the M2 rules (docs/ACCOUNTS.md › Revision generations only
    /// grow): a local win gets a new revision, and a replayed older revision
    /// never reverts the Mac: the local version is rebased onto it and pushed
    /// again ([`ApplyOutcome::Stale`]).
    pub fn apply_remote(
        &self,
        remote: &SyncDoc,
        attachment: Option<Attachment>,
        guard: Option<&SyncGuard>,
    ) -> Result<ApplyOutcome> {
        let doc_type = DocType::of_id(&remote.id)
            .ok_or_else(|| StoreError::Invalid(format!("Invalid document ID '{}'.", remote.id)))?;
        remote
            .rev
            .parse::<kanso_core::Rev>()
            .map_err(|e| StoreError::Invalid(e.to_string()))?;
        self.write(|tx, changes| {
            check_guard(tx, guard)?;
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
            Ok(ApplyOutcome::LocalWon { new_rev: local.rev })
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
        store.mark_synced(&note.id, &pushed.rev, None).unwrap();

        let edited = merge_body(&store, &note.id, "v2");
        assert_eq!(store.note(&note.id).unwrap().unwrap().updated_at, 100);

        let echo = SyncDoc {
            base_rev: None,
            ..pushed.clone()
        };
        assert_eq!(
            store.apply_remote(&echo, None, None).unwrap(),
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
        store.mark_synced(&note.id, &pushed.rev, None).unwrap();

        let next = pending(&store, &note.id).unwrap();
        assert_eq!(next.rev, edited.rev);
        assert_eq!(next.base_rev.as_deref(), Some(pushed.rev.as_str()));

        store.mark_synced(&note.id, &edited.rev, None).unwrap();
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
            store.apply_remote(&doc, None, None).unwrap(),
            ApplyOutcome::Applied
        );
        assert!(store.attachment(&doc.id).unwrap().is_none());
        assert!(!store.has_blob(&doc.id, &blob.md5_hex()).unwrap());

        assert_eq!(
            store.apply_remote(&doc, Some(blob.clone()), None).unwrap(),
            ApplyOutcome::Unchanged
        );
        let (content_type, stored) = store.attachment(&doc.id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(stored.data, bytes);
        assert!(store.has_blob(&doc.id, &blob.md5_hex()).unwrap());
        assert!(store.pending_push(10).unwrap().is_empty());
    }

    fn guard_meta(store: &Store, origin: &str, account: &str) {
        store.set_meta("sync.v2.origin", origin).unwrap();
        store.set_meta("sync.v2.account_id", account).unwrap();
    }

    const BOUND: [(&str, &str); 2] = [
        ("sync.v2.origin", "https://a.example"),
        ("sync.v2.account_id", "a_1"),
    ];

    /// A guarded write lands only while every guarded meta entry matches,
    /// and a refused one writes nothing.
    #[test]
    fn guarded_writes_need_the_whole_binding() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:guard1");
        let rev = pending(&store, &note.id).unwrap().rev;
        let guard = SyncGuard { expect: &BOUND };

        assert!(matches!(
            store.mark_synced(&note.id, &rev, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        guard_meta(&store, "https://b.example", "a_1");
        assert!(matches!(
            store.mark_synced(&note.id, &rev, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        assert!(matches!(
            store.set_meta_if("sync.v2.last_seq", "9", &guard),
            Err(StoreError::GuardFailed)
        ));
        assert_eq!(store.meta("sync.v2.last_seq").unwrap(), None);
        assert_eq!(store.pending_count().unwrap(), 1);

        guard_meta(&store, "https://a.example", "a_1");
        store.mark_synced(&note.id, &rev, Some(&guard)).unwrap();
        store.set_meta_if("sync.v2.last_seq", "9", &guard).unwrap();
        assert_eq!(
            store.meta("sync.v2.last_seq").unwrap().as_deref(),
            Some("9")
        );
        assert_eq!(store.pending_count().unwrap(), 0);

        guard_meta(&store, "https://a.example", "a_2");
        let remote = SyncDoc {
            rev: "9-ff".into(),
            ..pending_doc(&note.id)
        };
        assert!(matches!(
            store.apply_remote(&remote, None, Some(&guard)),
            Err(StoreError::GuardFailed)
        ));
        assert_eq!(
            store.get_api(&note.id, None, false).unwrap()["_rev"],
            rev.as_str()
        );
    }

    fn pending_doc(id: &str) -> SyncDoc {
        SyncDoc {
            id: id.into(),
            rev: "1-aa".into(),
            deleted: false,
            fields: json!({"bookId": "trash", "body": "remote", "updatedAt": 1})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        }
    }

    /// A document the server lost goes back to pending with no base.
    #[test]
    fn mark_unsynced_makes_a_document_pending_again() {
        let (_dir, store) = store();
        let note = create_note(&store, "note:lost1");
        let rev = pending(&store, &note.id).unwrap().rev;
        store.mark_synced(&note.id, &rev, None).unwrap();
        assert!(pending(&store, &note.id).is_none());
        store.mark_unsynced(&note.id, None).unwrap();
        let again = pending(&store, &note.id).unwrap();
        assert_eq!((again.rev, again.base_rev), (rev, None));
    }

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        /// Model-based: `pending_count` equals the length of `pending_push`
        /// (the oracle), and `mark_missing_unsynced` turns exactly the synced
        /// documents missing from `present` (tombstones included) back into
        /// pending ones.
        #[test]
        fn prop_missing_documents_become_pending(
            docs in prop::collection::vec((any::<bool>(), any::<bool>(), any::<bool>()), 1..8),
        ) {
            let (_dir, store) = store();
            let mut present = std::collections::HashSet::new();
            let mut expected_missing = Vec::new();
            for (i, (synced, deleted, in_feed)) in docs.iter().enumerate() {
                let id = format!("note:m{i}");
                create_note(&store, &id);
                if *deleted {
                    store.delete(&id, Origin::Api).unwrap();
                }
                if *synced {
                    let rev = pending(&store, &id).unwrap().rev;
                    store.mark_synced(&id, &rev, None).unwrap();
                    if !*in_feed {
                        expected_missing.push(id.clone());
                    }
                }
                if *in_feed {
                    present.insert(id);
                }
                prop_assert_eq!(store.pending_count().unwrap(), store.pending_push(usize::MAX).unwrap().len());
            }
            let before: std::collections::BTreeSet<String> =
                store.pending_push(usize::MAX).unwrap().into_iter().map(|d| d.id).collect();
            expected_missing.sort();
            let missing = store.mark_missing_unsynced(&present, None).unwrap();
            prop_assert_eq!(&missing, &expected_missing);
            let after: std::collections::BTreeSet<String> =
                store.pending_push(usize::MAX).unwrap().into_iter().map(|d| d.id).collect();
            let mut expected = before;
            expected.extend(expected_missing);
            prop_assert_eq!(after, expected);
            prop_assert!(store.pending_push(usize::MAX).unwrap().iter().filter(|d| missing.contains(&d.id)).all(|d| d.base_rev.is_none()));
        }
    }
}
