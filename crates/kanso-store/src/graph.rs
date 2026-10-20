//! What the graph view draws: every live note outside the trash, and the
//! links in their bodies, from the `note_links` index. The app resolves
//! wiki-link titles and lays the graph out.

use crate::{Result, Store};

/// A note as the graph shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNote {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub tags: Vec<String>,
}

/// One link in a note's body, as indexed: to a note's ID, or to a title
/// (normalized) for a wiki-link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLink {
    pub from: String,
    pub target_id: Option<String>,
    pub target_title: Option<String>,
}

impl Store {
    /// Every live note outside the trash, first created first (the order
    /// wiki-link titles resolve in), and every link those notes make.
    pub fn graph_rows(&self) -> Result<(Vec<GraphNote>, Vec<GraphLink>)> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, book_id, json_extract(json, '$.tags') FROM docs
                 WHERE type = 'note' AND deleted = 0 AND IFNULL(book_id, '') != 'trash'
                 ORDER BY created_at, id",
            )?;
            let notes = stmt
                .query_map([], |row| {
                    let tags: Option<String> = row.get(3)?;
                    Ok(GraphNote {
                        id: row.get(0)?,
                        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        tags: tags
                            .and_then(|t| serde_json::from_str(&t).ok())
                            .unwrap_or_default(),
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let mut stmt = conn.prepare(
                "SELECT l.note_id, l.target_id, l.target_title FROM note_links l
                 JOIN docs d ON d.id = l.note_id
                 WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                 ORDER BY l.note_id, l.ord",
            )?;
            let links = stmt
                .query_map([], |row| {
                    Ok(GraphLink {
                        from: row.get(0)?,
                        target_id: row.get(1)?,
                        target_title: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok((notes, links))
        })
    }
}
