//! Saved versions the write path would not snapshot on its own.

use kanso_core::now_ms;
use rusqlite::params;
use serde_json::{Map, Value};

use crate::write::SNAPSHOTS_PER_DOC;
use crate::{Origin, Result, Store};

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
        self.write(|tx, _| {
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
