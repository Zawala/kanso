//! Connection setup and schema migrations.

use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

const MIGRATIONS: &[&str] = &[r#"
CREATE TABLE docs (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    rev TEXT NOT NULL,
    seq INTEGER NOT NULL UNIQUE,
    deleted INTEGER NOT NULL DEFAULT 0,
    -- The document without `_id`, `_rev` and derived fields.
    json TEXT NOT NULL,
    -- The revision last known to match the sync server; NULL if never synced.
    synced_rev TEXT,
    book_id TEXT GENERATED ALWAYS AS (json_extract(json, '$.bookId')) VIRTUAL,
    parent_book_id TEXT GENERATED ALWAYS AS (json_extract(json, '$.parentBookId')) VIRTUAL,
    status TEXT GENERATED ALWAYS AS (json_extract(json, '$.status')) VIRTUAL,
    title TEXT GENERATED ALWAYS AS (json_extract(json, '$.title')) VIRTUAL,
    name TEXT GENERATED ALWAYS AS (json_extract(json, '$.name')) VIRTUAL,
    updated_at INTEGER GENERATED ALWAYS AS (json_extract(json, '$.updatedAt')) VIRTUAL,
    created_at INTEGER GENERATED ALWAYS AS (json_extract(json, '$.createdAt')) VIRTUAL,
    pinned INTEGER GENERATED ALWAYS AS (json_extract(json, '$.pinned')) VIRTUAL
);
CREATE INDEX docs_type_updated ON docs (type, deleted, updated_at);
CREATE INDEX docs_book ON docs (type, book_id);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (note_id, tag_id)
) WITHOUT ROWID;
CREATE INDEX note_tags_tag ON note_tags (tag_id);

CREATE TABLE blobs (
    file_id TEXT PRIMARY KEY NOT NULL,
    md5 TEXT NOT NULL,
    data BLOB NOT NULL
);

CREATE TABLE revisions (
    id INTEGER PRIMARY KEY,
    doc_id TEXT NOT NULL,
    rev TEXT NOT NULL,
    json TEXT NOT NULL,
    saved_at INTEGER NOT NULL,
    origin TEXT NOT NULL
);
CREATE INDEX revisions_doc ON revisions (doc_id, saved_at);

CREATE TABLE meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Imported documents Kanso doesn't use (Inkdrop's activity log), kept verbatim.
CREATE TABLE archive (
    id TEXT PRIMARY KEY NOT NULL,
    json TEXT NOT NULL
);

CREATE VIRTUAL TABLE notes_fts USING fts5(note_id UNINDEXED, title, body, tokenize = 'trigram');
"#];

pub fn connect(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.create_scalar_function(
        "kanso_contains",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let haystack = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            let needle = ctx.get::<Option<String>>(1)?.unwrap_or_default();
            Ok(kanso_core::query::contains_ci(&haystack, &needle))
        },
    )?;
    // The part of a body around a search term's first match, when its
    // excerpt can't reach it from the start.
    conn.create_scalar_function(
        "kanso_window",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let body = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            let term = ctx.get::<Option<String>>(1)?.unwrap_or_default();
            Ok(kanso_core::excerpt::match_window(&body, &term).map(str::to_owned))
        },
    )?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        conn.execute_batch(&format!(
            "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
            index + 1
        ))?;
    }
    Ok(())
}

pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// The next value of the store-wide change sequence.
pub fn next_seq(conn: &Connection) -> Result<i64> {
    Ok(
        conn.query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM docs", [], |row| {
            row.get(0)
        })?,
    )
}
