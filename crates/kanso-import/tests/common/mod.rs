//! A synthetic PouchDB library: PouchDB's six SQLite tables with the real
//! column layout (copied from a real library's database), filled with
//! documents, revisions and attachments.

#![allow(dead_code)]

use std::path::Path;

use kanso_store::Attachment;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value, json};

/// The schema of a real library's `db/<hash>.sqlite`, verbatim.
pub const POUCHDB_SCHEMA: &str = r#"
CREATE TABLE 'attach-store' (digest UNIQUE, escaped TINYINT(1), body BLOB);
CREATE TABLE 'local-store' (id UNIQUE, rev, json);
CREATE TABLE 'attach-seq-store' (digest, seq INTEGER);
CREATE INDEX 'attach-seq-seq-idx' ON 'attach-seq-store' (seq);
CREATE UNIQUE INDEX 'attach-seq-digest-idx' ON 'attach-seq-store' (digest, seq);
CREATE TABLE 'document-store' (id unique, json, winningseq, max_seq INTEGER UNIQUE);
CREATE INDEX 'doc-winningseq-idx' ON 'document-store' (winningseq);
CREATE TABLE 'by-sequence' (seq INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, json, deleted TINYINT(1), doc_id, rev);
CREATE INDEX 'by-seq-deleted-idx' ON 'by-sequence' (seq, deleted);
CREATE UNIQUE INDEX 'by-seq-doc-id-rev' ON 'by-sequence' (doc_id, rev);
CREATE TABLE 'metadata-store' (dbid, db_version INTEGER);
"#;

/// One stored revision of a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Revision {
    pub rev: String,
    /// Fields without `_id` and `_rev`, as PouchDB stores them.
    pub fields: Map<String, Value>,
    pub deleted: bool,
}

impl Revision {
    pub fn live(rev: &str, fields: Value) -> Self {
        Revision {
            rev: rev.to_owned(),
            fields: object(fields),
            deleted: false,
        }
    }

    /// A deletion, stored as PouchDB does: `{"_deleted": true}` with `deleted = 1`.
    pub fn deleted(rev: &str) -> Self {
        Revision {
            rev: rev.to_owned(),
            fields: object(json!({ "_deleted": true })),
            deleted: true,
        }
    }
}

pub fn object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => panic!("expected an object, got {other}"),
    }
}

/// Bytes as a PouchDB binary string, one character per byte. Written
/// independently of the importer's encoder.
pub fn binary_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// PouchDB's attachment stub for `bytes`, as stored in a file's `_attachments`.
pub fn attachment_stub(bytes: &[u8], content_type: &str) -> Value {
    json!({
        "index": {
            "content_type": content_type,
            "digest": Attachment::new(bytes.to_vec()).digest(),
            "length": bytes.len(),
            "revpos": 1,
        }
    })
}

pub struct Fixture {
    pub conn: Connection,
}

impl Fixture {
    /// Creates the database at `path` in WAL mode, as the app keeps it.
    pub fn create(path: &Path) -> Fixture {
        let conn = Connection::open(path).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.execute_batch(POUCHDB_SCHEMA).unwrap();
        conn.execute(
            r#"INSERT INTO "metadata-store" (dbid, db_version) VALUES ('9a350432-0000-4000-8000-000000000000', 7)"#,
            [],
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "local-store" (id, rev, json) VALUES ('_local/sync', '0-1', '{"last_seq":0}')"#,
            [],
        )
        .unwrap();
        Fixture { conn }
    }

    /// Stores `revisions` of document `id` in order and points
    /// `document-store.winningseq` at `revisions[winner]`.
    pub fn put_doc(&self, id: &str, revisions: &[Revision], winner: usize) {
        assert!(winner < revisions.len());
        let mut seqs = Vec::new();
        for revision in revisions {
            self.conn
                .execute(
                    r#"INSERT INTO "by-sequence" (json, deleted, doc_id, rev) VALUES (?1, ?2, ?3, ?4)"#,
                    params![
                        Value::Object(revision.fields.clone()).to_string(),
                        i64::from(revision.deleted),
                        id,
                        revision.rev
                    ],
                )
                .unwrap();
            seqs.push(self.conn.last_insert_rowid());
        }
        let winning = &revisions[winner];
        let meta = json!({
            "id": id,
            "rev_tree": [],
            "winningRev": winning.rev,
            "deleted": winning.deleted,
            "seq": seqs[winner],
        });
        self.conn
            .execute(
                r#"INSERT INTO "document-store" (id, json, winningseq, max_seq) VALUES (?1, ?2, ?3, ?4)"#,
                params![id, meta.to_string(), seqs[winner], seqs.last().unwrap()],
            )
            .unwrap();
    }

    /// A document with one live revision.
    pub fn put_live(&self, id: &str, rev: &str, fields: Value) {
        self.put_doc(id, &[Revision::live(rev, fields)], 0);
    }

    /// Stores attachment bytes under their digest as a binary string.
    pub fn put_attachment(&self, bytes: &[u8]) {
        self.put_attachment_body(
            &Attachment::new(bytes.to_vec()).digest(),
            &binary_string(bytes),
            false,
        );
    }

    /// Stores an attachment body as given (possibly corrupt or escaped).
    pub fn put_attachment_body(&self, digest: &str, body: &str, escaped: bool) {
        self.conn
            .execute(
                r#"INSERT INTO "attach-store" (digest, escaped, body) VALUES (?1, ?2, ?3)
                   ON CONFLICT (digest) DO UPDATE SET escaped = excluded.escaped, body = excluded.body"#,
                params![digest, i64::from(escaped), body],
            )
            .unwrap();
        let seq: i64 = self
            .conn
            .query_row(
                r#"SELECT COALESCE(MAX(seq), 0) FROM "by-sequence""#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        self.conn
            .execute(
                r#"INSERT OR IGNORE INTO "attach-seq-store" (digest, seq) VALUES (?1, ?2)"#,
                params![digest, seq],
            )
            .unwrap();
    }

    /// The body stored under `digest`.
    pub fn attachment_body(&self, digest: &str) -> Option<String> {
        self.conn
            .query_row(
                r#"SELECT body FROM "attach-store" WHERE digest = ?"#,
                [digest],
                |r| r.get(0),
            )
            .optional()
            .unwrap()
    }
}

/// A complete, canonical note as the app writes it.
pub fn note_fields(title: &str, body: &str, book_id: &str) -> Value {
    let tasks = kanso_core::tasks::count_tasks(body);
    json!({
        "doctype": "markdown",
        "bookId": book_id,
        "tags": [],
        "updatedAt": 1_700_000_100_000i64,
        "createdAt": 1_700_000_000_000i64,
        "status": "none",
        "share": "private",
        "pinned": false,
        "timestamp": 1_700_000_100_000i64,
        "title": title,
        "body": body,
        "numOfTasks": tasks.total,
        "numOfCheckedTasks": tasks.checked,
    })
}

pub fn book_fields(name: &str, parent: Option<&str>) -> Value {
    json!({
        "updatedAt": 1_700_000_000_000i64,
        "createdAt": 1_700_000_000_000i64,
        "count": 3,
        "name": name,
        "parentBookId": parent,
        "order": 1,
    })
}

pub fn tag_fields(name: &str, color: &str) -> Value {
    json!({
        "color": color,
        "updatedAt": 1_700_000_000_000i64,
        "createdAt": 1_700_000_000_000i64,
        "count": 2,
        "name": name,
    })
}

pub fn file_fields(name: &str, bytes: &[u8], content_type: &str) -> Value {
    json!({
        "createdAt": 1_700_000_000_000i64,
        "name": name,
        "contentType": content_type,
        "contentLength": bytes.len(),
        "publicIn": [],
        "_attachments": attachment_stub(bytes, content_type),
    })
}

/// A 32-hex-digit revision hash derived from `seed`.
pub fn rev(generation: u64, seed: u64) -> String {
    format!(
        "{generation}-{:032x}",
        u128::from(seed).wrapping_mul(0x9e37_79b9_7f4a_7c15)
    )
}
