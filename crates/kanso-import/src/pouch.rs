//! PouchDB's SQLite layout (the websql / node-websql adapter Inkdrop uses).
//!
//! - `document-store(id, json, winningseq, max_seq)`: one row per document;
//!   `winningseq` points at the current revision in `by-sequence`.
//! - `by-sequence(seq, json, deleted, doc_id, rev)`: every stored revision.
//!   `json` holds the fields without `_id` and `_rev`.
//! - `attach-store(digest, escaped, body)`: attachment bytes as a PouchDB
//!   "binary string", one character per byte.

use std::fmt;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, bail};
use rusqlite::backup::{Backup, StepResult};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde_json::{Map, Value};

/// A string that is not a PouchDB binary string: it holds a character above U+00FF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBinaryString {
    /// Character index of the first offending character.
    pub index: usize,
    pub code_point: u32,
}

impl fmt::Display for NotBinaryString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "not a binary string: U+{:04X} at character {}",
            self.code_point, self.index
        )
    }
}

impl std::error::Error for NotBinaryString {}

/// Decodes a PouchDB binary string: each character's code point is one byte.
pub fn decode_binary_string(text: &str) -> Result<Vec<u8>, NotBinaryString> {
    text.chars()
        .enumerate()
        .map(|(index, c)| {
            u8::try_from(u32::from(c)).map_err(|_| NotBinaryString {
                index,
                code_point: u32::from(c),
            })
        })
        .collect()
}

/// Encodes bytes as a PouchDB binary string (the inverse of [`decode_binary_string`]).
pub fn encode_binary_string(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

/// PouchDB's websql `escapeBlob`, applied to bodies stored with `escaped = 1`
/// so they contain no NUL characters.
pub fn escape_blob(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{0}' => out.push_str("\u{1}\u{1}"),
            '\u{1}' => out.push_str("\u{1}\u{2}"),
            '\u{2}' => out.push_str("\u{2}\u{2}"),
            other => out.push(other),
        }
    }
    out
}

/// PouchDB's websql `unescapeBlob`, the inverse of [`escape_blob`].
pub fn unescape_blob(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let decoded = match (c, chars.peek()) {
            ('\u{1}', Some('\u{1}')) => Some('\u{0}'),
            ('\u{1}', Some('\u{2}')) => Some('\u{1}'),
            ('\u{2}', Some('\u{2}')) => Some('\u{2}'),
            _ => None,
        };
        match decoded {
            Some(d) => {
                chars.next();
                out.push(d);
            }
            None => out.push(c),
        }
    }
    out
}

/// A `file:` URI that opens `path` read-only.
fn read_only_uri(path: &Path) -> anyhow::Result<String> {
    let path =
        std::path::absolute(path).with_context(|| format!("cannot resolve {}", path.display()))?;
    let text = path
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", path.display()))?;
    let mut uri = String::from("file:");
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.' | b'~') {
            uri.push(char::from(b));
        } else {
            uri.push_str(&format!("%{b:02X}"));
        }
    }
    uri.push_str("?mode=ro");
    Ok(uri)
}

/// Copies a consistent snapshot of the database at `source` into a new
/// database at `dest`, using SQLite's online backup API on a read-only
/// connection. Works while another process (Inkdrop) has the source open.
pub fn snapshot(source: &Path, dest: &Path) -> anyhow::Result<()> {
    if !source.is_file() {
        bail!("{} does not exist", source.display());
    }
    let src = Connection::open_with_flags(
        read_only_uri(source)?,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("cannot open {} read-only", source.display()))?;
    src.busy_timeout(Duration::from_secs(5))?;
    let mut dst = Connection::open(dest)
        .with_context(|| format!("cannot create the snapshot {}", dest.display()))?;
    let backup = Backup::new(&src, &mut dst)?;
    // Copying every page in one step holds a single read transaction, so the
    // snapshot is consistent even if Inkdrop writes meanwhile.
    for _ in 0..200 {
        match backup
            .step(-1)
            .context("snapshot of the Inkdrop database failed")?
        {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            _ => std::thread::sleep(Duration::from_millis(50)),
        }
    }
    bail!("the Inkdrop database stayed locked; try again or quit Inkdrop")
}

/// Fails unless `conn` has PouchDB's tables.
pub fn check_layout(conn: &Connection) -> anyhow::Result<()> {
    for table in ["document-store", "by-sequence", "attach-store"] {
        let found = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?",
                [table],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !found {
            bail!("not an Inkdrop database: the `{table}` table is missing");
        }
    }
    Ok(())
}

/// The winning revision of one document.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceDoc {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    /// Fields without `_id` and `_rev`.
    pub fields: Map<String, Value>,
}

/// A winning revision that cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub id: String,
    pub reason: String,
}

fn text(value: ValueRef<'_>) -> Option<String> {
    match value {
        ValueRef::Text(bytes) | ValueRef::Blob(bytes) => {
            std::str::from_utf8(bytes).ok().map(str::to_owned)
        }
        ValueRef::Integer(n) => Some(n.to_string()),
        _ => None,
    }
}

/// The winning revision of every document, in ID order.
pub fn winning_docs(conn: &Connection) -> anyhow::Result<Vec<Result<SourceDoc, Unreadable>>> {
    let mut stmt = conn.prepare(
        r#"SELECT d.id, s.doc_id, s.rev, s.deleted, s.json
           FROM "document-store" d
           LEFT JOIN "by-sequence" s ON s.seq = d.winningseq
           ORDER BY d.id"#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            text(row.get_ref(0)?),
            text(row.get_ref(1)?),
            text(row.get_ref(2)?),
            row.get::<_, Option<i64>>(3)?.unwrap_or(0) != 0,
            text(row.get_ref(4)?),
        ))
    })?;
    let mut docs = Vec::new();
    for row in rows {
        let (id, doc_id, rev, deleted, json) = row?;
        let Some(id) = id else { continue };
        let unreadable = |reason: &str| {
            Err(Unreadable {
                id: id.clone(),
                reason: reason.to_owned(),
            })
        };
        let doc = match (doc_id, rev, json) {
            (None, ..) => unreadable("its winning revision is missing"),
            (Some(doc_id), ..) if doc_id != id => {
                unreadable("its winning revision belongs to another document")
            }
            (_, None, _) => unreadable("its winning revision has no rev"),
            (_, _, None) => unreadable("its winning revision has no JSON"),
            (_, Some(rev), Some(json)) => match serde_json::from_str::<Value>(&json) {
                Ok(Value::Object(fields)) => {
                    let deleted = deleted || fields.get("_deleted") == Some(&Value::Bool(true));
                    Ok(SourceDoc {
                        id: id.clone(),
                        rev,
                        deleted,
                        fields,
                    })
                }
                Ok(_) => unreadable("its JSON is not an object"),
                Err(err) => unreadable(&format!("its JSON does not parse: {err}")),
            },
        };
        docs.push(doc);
    }
    Ok(docs)
}

/// The bytes stored under `digest` in `attach-store`, decoded.
/// `Ok(None)` when no body is stored under that digest.
pub fn attachment_bytes(
    conn: &Connection,
    digest: &str,
) -> anyhow::Result<Option<Result<Vec<u8>, String>>> {
    let row = conn
        .query_row(
            r#"SELECT escaped, body FROM "attach-store" WHERE digest = ?"#,
            [digest],
            |row| {
                let escaped = row.get::<_, Option<i64>>(0)?.unwrap_or(0) != 0;
                let body = match row.get_ref(1)? {
                    ValueRef::Text(bytes) => Some(Ok(String::from_utf8_lossy(bytes).into_owned())),
                    ValueRef::Blob(bytes) => Some(Err(bytes.to_vec())),
                    ValueRef::Null => None,
                    _ => None,
                };
                Ok((escaped, body))
            },
        )
        .optional()?;
    let Some((escaped, body)) = row else {
        return Ok(None);
    };
    Ok(Some(match body {
        None => Err("its attachment body is empty or not text".to_owned()),
        // Raw bytes (other adapters) are taken as they are.
        Some(Err(raw)) => Ok(raw),
        Some(Ok(text)) => {
            let text = if escaped { unescape_blob(&text) } else { text };
            decode_binary_string(&text).map_err(|e| e.to_string())
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_escapes_spaces_and_query_characters() {
        let uri = read_only_uri(Path::new("/a b/c?d#e%.sqlite")).unwrap();
        assert_eq!(uri, "file:/a%20b/c%3Fd%23e%25.sqlite?mode=ro");
    }

    #[test]
    fn decodes_latin1_range_and_rejects_wider_characters() {
        assert_eq!(
            decode_binary_string("\u{0}A\u{ff}").unwrap(),
            vec![0, 65, 255]
        );
        let err = decode_binary_string("ab\u{100}").unwrap_err();
        assert_eq!(
            err,
            NotBinaryString {
                index: 2,
                code_point: 0x100
            }
        );
    }

    #[test]
    fn unescape_inverts_escape() {
        let raw = "\u{0}\u{1}\u{2}\u{1}\u{1}x\u{2}\u{0}";
        assert_eq!(unescape_blob(&escape_blob(raw)), raw);
        assert!(!escape_blob(raw).contains('\u{0}'));
    }
}
