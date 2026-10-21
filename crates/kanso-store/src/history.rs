//! Saved versions the write path would not snapshot on its own.

use kanso_core::model::LOCK_FIELD;
use kanso_core::now_ms;
use rusqlite::{OptionalExtension, params};
use serde_json::{Map, Value};

use crate::write::SNAPSHOTS_PER_DOC;
use crate::{Origin, Result, Store, StoreError};

impl Store {
    /// Keeps `fields` as a saved version of `id`, for example unsaved edits
    /// the user set aside for a newer version made elsewhere.
    pub fn record_revision(
        &self,
        id: &str,
        rev: &str,
        fields: &Map<String, Value>,
        origin: Origin,
    ) -> Result<()> {
        self.record_revision_at(id, rev, fields, origin, now_ms())
    }

    /// [`Store::record_revision`], saved as of `saved_at`: a version kept
    /// from another clock, or a history drawn at a fixed time.
    pub fn record_revision_at(
        &self,
        id: &str,
        rev: &str,
        fields: &Map<String, Value>,
        origin: Origin,
        saved_at: i64,
    ) -> Result<()> {
        let json = serde_json::to_string(fields)?;
        let has_text = fields
            .get("body")
            .and_then(Value::as_str)
            .is_some_and(|body| !body.is_empty());
        self.write(|tx, _| {
            // A locked note's history holds only sealed versions.
            if has_text && !fields.contains_key(LOCK_FIELD) {
                let locked: bool = tx
                    .query_row(
                        "SELECT json_extract(json, '$.kansoLock') IS NOT NULL FROM docs WHERE id = ?",
                        [id],
                        |row| row.get(0),
                    )
                    .optional()?
                    .unwrap_or(false);
                if locked {
                    return Err(StoreError::Invalid(
                        "The note is locked: its versions are kept sealed.".into(),
                    ));
                }
            }
            tx.execute(
                "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, rev, json, saved_at, origin.as_str()],
            )?;
            tx.execute(
                "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
                 (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
                params![id, SNAPSHOTS_PER_DOC],
            )?;
            Ok(())
        })
    }
}
