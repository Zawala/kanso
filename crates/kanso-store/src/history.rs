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
        let json = serde_json::to_string(fields)?;
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, rev, json, now_ms(), origin.as_str()],
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
