//! Reads: single documents, lists with derived counts, and note search.

use std::collections::HashMap;

use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::query::{Clause, Field};
use kanso_core::{Book, DocType, FileMeta, Note, NoteStatus, Query, Tag};
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, OptionalExtension, Row, params_from_iter};
use serde_json::{Map, Value};

use crate::doc::{Attachment, Doc};
use crate::write::load_blob;
use crate::{Result, Store, StoreError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    UpdatedAt,
    CreatedAt,
    Title,
}

impl Sort {
    pub fn parse(value: &str) -> Option<Sort> {
        match value {
            "updatedAt" => Some(Sort::UpdatedAt),
            "createdAt" => Some(Sort::CreatedAt),
            "title" => Some(Sort::Title),
            _ => None,
        }
    }

    fn sql(self) -> &'static str {
        match self {
            Sort::UpdatedAt => "d.updated_at",
            Sort::CreatedAt => "d.created_at",
            Sort::Title => "d.title COLLATE NOCASE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchParams {
    pub query: Query,
    pub sort: Sort,
    pub descending: bool,
    pub limit: Option<usize>,
    pub skip: usize,
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams {
            query: Query::default(),
            sort: Sort::UpdatedAt,
            descending: true,
            limit: None,
            skip: 0,
        }
    }
}

/// What the note list shows, without the full body.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteSummary {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub book_id: String,
    pub status: NoteStatus,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_at: i64,
    pub created_at: i64,
}

/// Note counts for the sidebar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusCounts {
    pub all: u32,
    pub pinned: u32,
    pub trash: u32,
    pub by_status: HashMap<NoteStatus, u32>,
}

/// A saved earlier version of a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Revision {
    pub rev: String,
    pub saved_at: i64,
    pub origin: String,
    pub fields: Map<String, Value>,
}

const DOC_COLUMNS: &str = "d.id, d.type, d.rev, d.seq, d.deleted, d.json";

fn doc_from_row(row: &Row) -> rusqlite::Result<Doc> {
    let doc_type: String = row.get(1)?;
    let json: String = row.get(5)?;
    let fields = match serde_json::from_str(&json) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    };
    Ok(Doc {
        id: row.get(0)?,
        doc_type: DocType::from_name(&doc_type).unwrap_or(DocType::Note),
        rev: row.get(2)?,
        seq: row.get(3)?,
        deleted: row.get(4)?,
        fields,
    })
}

fn live_doc(conn: &Connection, id: &str) -> Result<Option<Doc>> {
    Ok(conn
        .query_row(
            &format!("SELECT {DOC_COLUMNS} FROM docs d WHERE d.id = ? AND d.deleted = 0"),
            [id],
            doc_from_row,
        )
        .optional()?)
}

fn count_map(conn: &Connection, sql: &str) -> Result<HashMap<String, u32>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

const BOOK_COUNTS: &str = "SELECT book_id, COUNT(*) FROM docs
    WHERE type = 'note' AND deleted = 0 AND book_id IS NOT NULL GROUP BY book_id";
const TAG_COUNTS: &str = "SELECT nt.tag_id, COUNT(*) FROM note_tags nt
    JOIN docs d ON d.id = nt.note_id AND d.deleted = 0 AND d.book_id != 'trash'
    GROUP BY nt.tag_id";

fn with_count(count: u32) -> Map<String, Value> {
    let mut extra = Map::new();
    extra.insert("count".into(), count.into());
    extra
}

/// Builds the `WHERE` clause and parameters for a note search.
fn search_where(query: &Query) -> (String, Vec<SqlValue>) {
    let includes_trash = query.includes_trash();
    let mut sql = String::from("d.type = 'note' AND d.deleted = 0");
    if !includes_trash {
        sql.push_str(" AND IFNULL(d.book_id, '') != 'trash'");
    }
    let mut params = Vec::new();
    for clause in &query.clauses {
        let expr = clause_sql(clause, !includes_trash, &mut params);
        sql.push_str(if clause.negated { " AND NOT " } else { " AND " });
        sql.push_str(&format!("COALESCE(({expr}), 0)"));
    }
    (sql, params)
}

const BODY: &str = "json_extract(d.json, '$.body')";

fn bind(params: &mut Vec<SqlValue>, value: String) -> &'static str {
    params.push(SqlValue::Text(value));
    "?"
}

/// SQL for one clause, ignoring negation. For ASCII text of 3+ characters
/// the trigram index narrows the candidates; `kanso_contains` then decides,
/// so results match `Query::matches` exactly.
fn clause_sql(clause: &Clause, fts_allowed: bool, params: &mut Vec<SqlValue>) -> String {
    let v = &clause.value;
    let use_fts = fts_allowed && !clause.negated && v.is_ascii() && v.chars().count() >= 3;
    let prefilter = |columns: &str, params: &mut Vec<SqlValue>| -> String {
        if !use_fts {
            return String::new();
        }
        let q = bind(
            params,
            format!("{columns} : \"{}\"", v.replace('"', "\"\"")),
        );
        format!("d.id IN (SELECT note_id FROM notes_fts WHERE notes_fts MATCH {q}) AND ")
    };
    match clause.field {
        None => {
            let pre = prefilter("{title body}", params);
            let a = bind(params, v.clone());
            let b = bind(params, v.clone());
            format!("{pre}(kanso_contains(d.title, {a}) OR kanso_contains({BODY}, {b}))")
        }
        Some(Field::Title) => {
            let pre = prefilter("title", params);
            let a = bind(params, v.clone());
            format!("{pre}kanso_contains(d.title, {a})")
        }
        Some(Field::Body) => {
            let pre = prefilter("body", params);
            let a = bind(params, v.clone());
            format!("{pre}kanso_contains({BODY}, {a})")
        }
        Some(Field::Book) => {
            let a = bind(params, v.clone());
            let b = bind(params, v.clone());
            format!(
                "d.book_id IN (SELECT id FROM docs WHERE type = 'book' AND deleted = 0 \
                 AND lower(name) = lower({a})) OR (d.book_id = '{TRASH_BOOK_ID}' AND lower({b}) = '{TRASH_BOOK_ID}')"
            )
        }
        Some(Field::BookId) => {
            let id = bind(params, clause.book_id());
            format!("d.book_id = {id}")
        }
        Some(Field::Tag) => {
            let name = bind(params, v.clone());
            format!(
                "EXISTS (SELECT 1 FROM note_tags nt JOIN docs t ON t.id = nt.tag_id AND t.deleted = 0 \
                 WHERE nt.note_id = d.id AND lower(t.name) = lower({name}))"
            )
        }
        Some(Field::TagId) => {
            let id = bind(params, kanso_core::id::with_prefix(DocType::Tag, v));
            format!("EXISTS (SELECT 1 FROM note_tags WHERE note_id = d.id AND tag_id = {id})")
        }
        Some(Field::Status) => {
            let status = bind(params, v.clone());
            format!("lower(d.status) = lower({status})")
        }
    }
}

fn search_sql(columns: &str, params: &SearchParams) -> (String, Vec<SqlValue>) {
    let (where_sql, mut values) = search_where(&params.query);
    let direction = if params.descending { "DESC" } else { "ASC" };
    let sql = format!(
        "SELECT {columns} FROM docs d WHERE {where_sql} ORDER BY {} {direction}, d.id {direction} LIMIT ? OFFSET ?",
        params.sort.sql()
    );
    values.push(SqlValue::Integer(params.limit.map_or(-1, |n| n as i64)));
    values.push(SqlValue::Integer(params.skip as i64));
    (sql, values)
}

impl Store {
    /// The live document with `id`.
    pub fn get(&self, id: &str) -> Result<Option<Doc>> {
        self.read(|conn| live_doc(conn, id))
    }

    /// A document as the HTTP API returns it. With `rev`, only that exact
    /// current revision is found. With `attachments`, files carry their bytes.
    pub fn get_api(&self, id: &str, rev: Option<&str>, attachments: bool) -> Result<Value> {
        self.read(|conn| {
            let doc = live_doc(conn, id)?.ok_or(StoreError::NotFound)?;
            if rev.is_some_and(|rev| rev != doc.rev) {
                return Err(StoreError::NotFound);
            }
            Ok(match doc.doc_type {
                DocType::Note => doc.to_api(Map::new()),
                DocType::Book => {
                    let counts = count_map(conn, BOOK_COUNTS)?;
                    doc.to_api(with_count(counts.get(id).copied().unwrap_or(0)))
                }
                DocType::Tag => {
                    let counts = count_map(conn, TAG_COUNTS)?;
                    doc.to_api(with_count(counts.get(id).copied().unwrap_or(0)))
                }
                DocType::File => {
                    let blob = load_blob(conn, id)?.unwrap_or_else(|| Attachment::new(Vec::new()));
                    let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
                    doc.file_to_api(&content_type, &blob, attachments)
                }
            })
        })
    }

    /// All live notebooks, tags or files in API form.
    pub fn list_api(
        &self,
        doc_type: DocType,
        limit: Option<usize>,
        skip: usize,
    ) -> Result<Vec<Value>> {
        self.read(|conn| {
            let order = match doc_type {
                DocType::Book | DocType::Tag => "d.name COLLATE NOCASE, d.id",
                DocType::Note => "d.updated_at DESC, d.id",
                DocType::File => "d.created_at, d.id",
            };
            let mut stmt = conn.prepare(&format!(
                "SELECT {DOC_COLUMNS} FROM docs d WHERE d.type = ? AND d.deleted = 0 ORDER BY {order} LIMIT ? OFFSET ?"
            ))?;
            let docs: Vec<Doc> = stmt
                .query_map(
                    rusqlite::params![doc_type.as_str(), limit.map_or(-1, |n| n as i64), skip as i64],
                    doc_from_row,
                )?
                .collect::<rusqlite::Result<_>>()?;
            let counts = match doc_type {
                DocType::Book => count_map(conn, BOOK_COUNTS)?,
                DocType::Tag => count_map(conn, TAG_COUNTS)?,
                _ => HashMap::new(),
            };
            docs.into_iter()
                .map(|doc| match doc.doc_type {
                    DocType::Book | DocType::Tag => {
                        Ok(doc.to_api(with_count(counts.get(&doc.id).copied().unwrap_or(0))))
                    }
                    DocType::File => {
                        let blob = load_blob(conn, &doc.id)?.unwrap_or_else(|| Attachment::new(Vec::new()));
                        let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
                        Ok(doc.file_to_api(&content_type, &blob, false))
                    }
                    DocType::Note => Ok(doc.to_api(Map::new())),
                })
                .collect()
        })
    }

    /// Notes matching a search, with full bodies.
    pub fn search(&self, params: &SearchParams) -> Result<Vec<Doc>> {
        self.read(|conn| {
            let (sql, values) = search_sql(DOC_COLUMNS, params);
            let mut stmt = conn.prepare(&sql)?;
            let docs = stmt
                .query_map(params_from_iter(values), doc_from_row)?
                .collect::<rusqlite::Result<_>>()?;
            Ok(docs)
        })
    }

    /// Notes matching a search, in API form.
    pub fn search_api(&self, params: &SearchParams) -> Result<Vec<Value>> {
        Ok(self
            .search(params)?
            .into_iter()
            .map(|doc| doc.to_api(Map::new()))
            .collect())
    }

    /// Notes matching a search, as list rows with a short snippet.
    pub fn search_summaries(&self, params: &SearchParams) -> Result<Vec<NoteSummary>> {
        self.read(|conn| {
            // Enough of the body to get past front matter, a title heading
            // and a code block or two.
            let columns = "d.id, d.rev, d.title, substr(json_extract(d.json, '$.body'), 1, 2000), \
                           d.book_id, d.status, json_extract(d.json, '$.tags'), d.pinned, d.updated_at, d.created_at";
            let (sql, values) = search_sql(columns, params);
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_from_iter(values), |row| {
                let tags: Option<String> = row.get(6)?;
                let status: Option<String> = row.get(5)?;
                let title = row.get::<_, Option<String>>(2)?.unwrap_or_default();
                let body = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                Ok(NoteSummary {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    snippet: kanso_core::excerpt(&title, &body),
                    title,
                    book_id: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    status: status.as_deref().and_then(NoteStatus::parse).unwrap_or_default(),
                    tags: tags
                        .and_then(|t| serde_json::from_str(&t).ok())
                        .unwrap_or_default(),
                    pinned: row.get::<_, Option<bool>>(7)?.unwrap_or(false),
                    updated_at: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    created_at: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn note(&self, id: &str) -> Result<Option<Note>> {
        self.typed(id)
    }

    pub fn file_meta(&self, id: &str) -> Result<Option<FileMeta>> {
        self.typed(id)
    }

    fn typed<T: serde::de::DeserializeOwned>(&self, id: &str) -> Result<Option<T>> {
        let Some(doc) = self.get(id)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_value(doc.to_api(Map::new()))?))
    }

    /// All live notebooks with note counts, sorted by name.
    pub fn books(&self) -> Result<Vec<Book>> {
        self.list_api(DocType::Book, None, 0)?
            .into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    /// All live tags with note counts, sorted by name.
    pub fn tags(&self) -> Result<Vec<Tag>> {
        self.list_api(DocType::Tag, None, 0)?
            .into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    /// An attachment's content type and bytes.
    pub fn attachment(&self, id: &str) -> Result<Option<(String, Attachment)>> {
        self.read(|conn| {
            let Some(doc) = live_doc(conn, id)?.filter(|d| d.doc_type == DocType::File) else {
                return Ok(None);
            };
            let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
            Ok(load_blob(conn, id)?.map(|blob| (content_type, blob)))
        })
    }

    /// Note counts for the sidebar.
    pub fn status_counts(&self) -> Result<StatusCounts> {
        self.read(|conn| {
            let mut counts = StatusCounts::default();
            let mut stmt = conn.prepare(
                "SELECT book_id = 'trash', status, pinned, COUNT(*) FROM docs
                 WHERE type = 'note' AND deleted = 0 GROUP BY 1, 2, 3",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, Option<bool>>(0)?.unwrap_or(false),
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<bool>>(2)?.unwrap_or(false),
                    row.get::<_, u32>(3)?,
                ))
            })?;
            for row in rows {
                let (trash, status, pinned, n) = row?;
                if trash {
                    counts.trash += n;
                    continue;
                }
                counts.all += n;
                if pinned {
                    counts.pinned += n;
                }
                let status = status
                    .as_deref()
                    .and_then(NoteStatus::parse)
                    .unwrap_or_default();
                *counts.by_status.entry(status).or_default() += n;
            }
            Ok(counts)
        })
    }

    /// Saved earlier versions of a document, newest first.
    pub fn revisions(&self, id: &str) -> Result<Vec<Revision>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rev, saved_at, origin, json FROM revisions WHERE doc_id = ? ORDER BY saved_at DESC, id DESC",
            )?;
            let rows = stmt.query_map([id], |row| {
                let json: String = row.get(3)?;
                Ok(Revision {
                    rev: row.get(0)?,
                    saved_at: row.get(1)?,
                    origin: row.get(2)?,
                    fields: match serde_json::from_str(&json) {
                        Ok(Value::Object(map)) => map,
                        _ => Map::new(),
                    },
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Documents changed after `since`, including deletions, oldest first.
    pub fn changes_since(
        &self,
        since: i64,
        limit: Option<usize>,
        descending: bool,
    ) -> Result<Vec<Doc>> {
        self.read(|conn| {
            let direction = if descending { "DESC" } else { "ASC" };
            let mut stmt = conn.prepare(&format!(
                "SELECT {DOC_COLUMNS} FROM docs d WHERE d.seq > ? ORDER BY d.seq {direction} LIMIT ?"
            ))?;
            let docs = stmt
                .query_map(rusqlite::params![since, limit.map_or(-1, |n| n as i64)], doc_from_row)?
                .collect::<rusqlite::Result<_>>()?;
            Ok(docs)
        })
    }

    /// The highest change sequence number.
    pub fn last_seq(&self) -> Result<i64> {
        self.read(|conn| {
            Ok(conn.query_row("SELECT COALESCE(MAX(seq), 0) FROM docs", [], |r| r.get(0))?)
        })
    }

    /// Live documents per type.
    pub fn counts(&self) -> Result<HashMap<DocType, usize>> {
        self.read(|conn| {
            let mut stmt =
                conn.prepare("SELECT type, COUNT(*) FROM docs WHERE deleted = 0 GROUP BY type")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = HashMap::new();
            for row in rows {
                let (doc_type, n) = row?;
                if let Some(doc_type) = DocType::from_name(&doc_type) {
                    counts.insert(doc_type, n);
                }
            }
            Ok(counts)
        })
    }
}
