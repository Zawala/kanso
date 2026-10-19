//! Connection setup and schema migrations.

use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

pub(crate) const MIGRATIONS: &[&str] = &[
    r#"
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
"#,
    // The replay floor (docs/ACCOUNTS.md › Sync with encryption › What a Mac
    // remembers per document): the highest generation this Mac has seen on
    // its account for the document. A server-reported loss and a re-merge
    // clear `synced_rev` but never this. Existing rows start at the
    // generation of their `synced_rev`.
    r#"
ALTER TABLE docs ADD COLUMN synced_gen INTEGER NOT NULL DEFAULT 0;
UPDATE docs SET synced_gen = CAST(substr(synced_rev, 1, instr(synced_rev, '-') - 1) AS INTEGER)
    WHERE synced_rev IS NOT NULL AND instr(synced_rev, '-') > 1;
"#,
    // The revision in a push whose answer hasn't been processed: its echo
    // is our own, never a conflict.
    r#"
ALTER TABLE docs ADD COLUMN sent_rev TEXT;
"#,
    // Tables derived from note bodies (`derived.rs`): local to this device,
    // filled from Rust when the store opens and kept current by every write.
    r#"
CREATE TABLE note_links (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    -- `note:…` for an `inkdrop://note/` link.
    target_id TEXT,
    -- The normalized title for a wiki-link.
    target_title TEXT,
    heading TEXT,
    line INTEGER NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX note_links_target_id ON note_links (target_id) WHERE target_id IS NOT NULL;
CREATE INDEX note_links_target_title ON note_links (target_title) WHERE target_title IS NOT NULL;

CREATE TABLE note_headings (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    level INTEGER NOT NULL,
    text TEXT NOT NULL,
    line INTEGER NOT NULL,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;

CREATE TABLE note_tasks (
    note_id TEXT NOT NULL,
    ord INTEGER NOT NULL,
    line INTEGER NOT NULL,
    checked INTEGER NOT NULL,
    text TEXT NOT NULL,
    -- `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, from an `@date` in the task.
    due TEXT,
    PRIMARY KEY (note_id, ord)
) WITHOUT ROWID;
CREATE INDEX note_tasks_due ON note_tasks (due) WHERE due IS NOT NULL;
"#,
];

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
    // A title as wiki-links match it.
    conn.create_scalar_function(
        "kanso_norm_title",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let title = ctx.get::<Option<String>>(0)?.unwrap_or_default();
            Ok(kanso_core::links::normalize_title(&title))
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

/// The `meta` key holding the highest change sequence handed out before
/// [`crate::Store::replace_library`] emptied `docs`, so sequences never
/// repeat and API change feeds keep moving forward.
pub const SEQ_FLOOR_KEY: &str = "store.seq_floor";

/// The highest change sequence handed out so far.
pub fn last_seq(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT max(COALESCE((SELECT MAX(seq) FROM docs), 0),
                    COALESCE((SELECT CAST(value AS INTEGER) FROM meta WHERE key = ?1), 0))",
        [SEQ_FLOOR_KEY],
        |row| row.get(0),
    )?)
}

/// The next value of the store-wide change sequence.
pub fn next_seq(conn: &Connection) -> Result<i64> {
    Ok(last_seq(conn)? + 1)
}
