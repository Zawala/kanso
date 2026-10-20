//! Writes: create, replace, merge, delete and import.

use base64::Engine as _;
use kanso_core::id::is_valid_id;
use kanso_core::model::{MAX_ATTACHMENT_SIZE, TRASH_BOOK_ID};
use kanso_core::{DocType, Rev, new_id, now_ms};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Map, Value, json};

use crate::doc::Attachment;
use crate::validate::{Refs, normalize, strip_reserved};
use crate::{Change, Origin, Result, Store, StoreError, schema};

/// How a POST treats an existing document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Put {
    /// `POST /notes` and friends: create, or replace the whole document when
    /// `_id` exists, which then requires the current `_rev`.
    Upsert,
    /// `POST /<id>`: shallow-merge the given fields into the current document.
    /// `_rev` is optional and checked when given.
    Merge,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WriteResult {
    pub id: String,
    pub rev: String,
}

/// Minimum gap between snapshots of UI autosaves of one document.
const UI_SNAPSHOT_INTERVAL_MS: i64 = 5 * 60 * 1000;
/// Snapshots kept per document.
pub(crate) const SNAPSHOTS_PER_DOC: i64 = 50;

/// A row of `docs` as the write path needs it.
pub(crate) struct Row {
    pub doc_type: DocType,
    pub rev: String,
    pub deleted: bool,
    pub fields: Map<String, Value>,
    pub synced_rev: Option<String>,
    /// The replay floor (`docs.synced_gen`).
    pub synced_gen: u64,
    /// The revision in a push whose answer wasn't processed (`docs.sent_rev`).
    pub sent_rev: Option<String>,
}

/// What happens to `synced_rev` when a row is written.
#[derive(Clone, Copy)]
pub(crate) enum Synced<'a> {
    Keep,
    Set(&'a str),
}

pub(crate) fn load_row(tx: &Transaction, id: &str) -> Result<Option<Row>> {
    let row = tx
        .query_row(
            "SELECT type, rev, deleted, json, synced_rev, synced_gen, sent_rev FROM docs WHERE id = ?",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, bool>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((doc_type, rev, deleted, json, synced_rev, synced_gen, sent_rev)) = row else {
        return Ok(None);
    };
    let doc_type = DocType::from_name(&doc_type)
        .ok_or_else(|| StoreError::Invalid(format!("unknown document type {doc_type}")))?;
    let fields = match serde_json::from_str(&json)? {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    Ok(Some(Row {
        doc_type,
        rev,
        deleted,
        fields,
        synced_rev,
        synced_gen: u64::try_from(synced_gen).unwrap_or(0),
        sent_rev,
    }))
}

/// Writes one row and its side tables, snapshotting the version it replaces.
#[allow(clippy::too_many_arguments)]
pub(crate) fn store_row(
    tx: &Transaction,
    id: &str,
    doc_type: DocType,
    rev: &str,
    deleted: bool,
    fields: &Map<String, Value>,
    previous: Option<&Row>,
    synced: Synced,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<()> {
    if let Some(prev) = previous
        && !prev.deleted
    {
        snapshot(tx, id, prev, origin)?;
    }
    let seq = schema::next_seq(tx)?;
    let json = serde_json::to_string(fields)?;
    let synced_rev = match synced {
        Synced::Keep => previous.and_then(|p| p.synced_rev.clone()),
        Synced::Set(rev) => Some(rev.to_owned()),
    };
    tx.execute(
        "INSERT INTO docs (id, type, rev, seq, deleted, json, synced_rev)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (id) DO UPDATE SET type = excluded.type, rev = excluded.rev,
             seq = excluded.seq, deleted = excluded.deleted, json = excluded.json,
             synced_rev = excluded.synced_rev",
        params![id, doc_type.as_str(), rev, seq, deleted, json, synced_rev],
    )?;
    if doc_type == DocType::Note {
        tx.execute("DELETE FROM note_tags WHERE note_id = ?", [id])?;
        tx.execute("DELETE FROM notes_fts WHERE note_id = ?", [id])?;
        if !deleted {
            if let Some(Value::Array(tags)) = fields.get("tags") {
                for tag in tags.iter().filter_map(Value::as_str) {
                    tx.execute(
                        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                        params![id, tag],
                    )?;
                }
            }
            if fields.get("bookId").and_then(Value::as_str) != Some(TRASH_BOOK_ID) {
                tx.execute(
                    "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, ?3)",
                    params![
                        id,
                        fields.get("title").and_then(Value::as_str).unwrap_or(""),
                        fields.get("body").and_then(Value::as_str).unwrap_or(""),
                    ],
                )?;
            }
        }
    }
    if crate::derived::applies(doc_type) {
        crate::derived::write_note(tx, id, (!deleted).then_some(fields))?;
    }
    if doc_type == DocType::File && deleted {
        tx.execute("DELETE FROM blobs WHERE file_id = ?", [id])?;
    }
    if doc_type == DocType::File {
        crate::ocr::write_file(tx, id, (!deleted).then_some(fields))?;
    }
    if doc_type == DocType::Note && deleted {
        crate::embed::forget_note(tx, id)?;
    }
    changes.push(Change {
        id: id.to_owned(),
        rev: rev.to_owned(),
        seq,
        deleted,
        origin,
    });
    Ok(())
}

fn snapshot(tx: &Transaction, id: &str, prev: &Row, origin: Origin) -> Result<()> {
    if origin == Origin::Ui {
        let last: Option<i64> = tx.query_row(
            "SELECT MAX(saved_at) FROM revisions WHERE doc_id = ?",
            [id],
            |row| row.get(0),
        )?;
        if last.is_some_and(|at| now_ms() - at < UI_SNAPSHOT_INTERVAL_MS) {
            return Ok(());
        }
    }
    keep_snapshot(tx, id, prev, origin)
}

/// Puts `prev` in the document's history, unthrottled, then prunes it.
fn keep_snapshot(tx: &Transaction, id: &str, prev: &Row, origin: Origin) -> Result<()> {
    let now = now_ms();
    tx.execute(
        "INSERT INTO revisions (doc_id, rev, json, saved_at, origin) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            prev.rev,
            serde_json::to_string(&prev.fields)?,
            now,
            origin.as_str()
        ],
    )?;
    tx.execute(
        "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
         (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
        params![id, SNAPSHOTS_PER_DOC],
    )?;
    Ok(())
}

/// Replaces live row `prev` with a tombstone and returns its revision. The
/// version it deletes always goes into history, however recent the last
/// snapshot: sync tells a replay of content this Mac held from another
/// Mac's edit it never saw by that history, and only the edit may beat the
/// deletion (docs/ACCOUNTS.md › Sync with encryption, review-verify 2).
fn delete_row(
    tx: &Transaction,
    id: &str,
    prev: &Row,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    if origin == Origin::Ui {
        // `store_row`'s throttled snapshot then adds nothing.
        keep_snapshot(tx, id, prev, origin)?;
    }
    let rev = tombstone_rev(prev)?;
    store_row(
        tx,
        id,
        prev.doc_type,
        &rev,
        true,
        &Map::new(),
        Some(prev),
        Synced::Keep,
        origin,
        changes,
    )?;
    Ok(rev)
}

pub(crate) fn next_rev(previous: Option<&Row>, content: &Value) -> Result<String> {
    let previous = previous
        .map(|p| p.rev.parse::<Rev>())
        .transpose()
        .map_err(|e| StoreError::Invalid(e.to_string()))?;
    Ok(Rev::next(previous.as_ref(), content).to_string())
}

pub(crate) fn fresh_id(tx: &Transaction, doc_type: DocType) -> Result<String> {
    loop {
        let id = new_id(doc_type);
        let taken = tx
            .query_row("SELECT 1 FROM docs WHERE id = ?", [&id], |_| Ok(()))
            .optional()?
            .is_some();
        if !taken {
            return Ok(id);
        }
    }
}

fn tombstone_rev(previous: &Row) -> Result<String> {
    next_rev(Some(previous), &json!({ "deleted": true }))
}

impl Store {
    /// Creates, replaces or merges a document of `doc_type` from an API body.
    pub fn put(
        &self,
        doc_type: DocType,
        body: Value,
        mode: Put,
        origin: Origin,
    ) -> Result<WriteResult> {
        self.put_keeping(doc_type, body, mode, origin, &[])
    }

    /// [`Store::put`], except that the fields named in `kept` are never
    /// taken from `body`: the document keeps its current values for them (in
    /// the same transaction), and a new document gets none. The local API
    /// uses it for fields only the app may set.
    pub fn put_keeping(
        &self,
        doc_type: DocType,
        body: Value,
        mode: Put,
        origin: Origin,
        kept: &[&str],
    ) -> Result<WriteResult> {
        let Value::Object(mut fields) = body else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        self.write(|tx, changes| {
            let attachment = if doc_type == DocType::File {
                take_attachment(&mut fields)?
            } else {
                None
            };
            let given_id = fields.get("_id").and_then(Value::as_str).map(str::to_owned);
            let given_rev = fields
                .get("_rev")
                .and_then(Value::as_str)
                .map(str::to_owned);
            let id = match given_id {
                Some(id) if is_valid_id(doc_type, &id) => id,
                Some(id) => {
                    return Err(StoreError::Invalid(format!("Invalid document ID '{id}'.")));
                }
                None if mode == Put::Merge => return Err(StoreError::NotFound),
                None => fresh_id(tx, doc_type)?,
            };
            let previous = load_row(tx, &id)?;
            if let Some(prev) = &previous
                && prev.doc_type != doc_type
            {
                return Err(StoreError::Conflict);
            }
            let live = previous.as_ref().filter(|p| !p.deleted);
            let merged = match (mode, live) {
                (Put::Upsert, Some(prev)) => {
                    if given_rev.as_deref() != Some(prev.rev.as_str()) {
                        return Err(StoreError::Conflict);
                    }
                    fields
                }
                (Put::Upsert, None) => fields,
                (Put::Merge, None) => return Err(StoreError::NotFound),
                (Put::Merge, Some(prev)) => {
                    if given_rev.as_ref().is_some_and(|rev| *rev != prev.rev) {
                        return Err(StoreError::Conflict);
                    }
                    strip_reserved(&mut fields);
                    let mut merged = prev.fields.clone();
                    merged.extend(fields);
                    merged
                }
            };
            let mut merged = merged;
            for &key in kept {
                match live.and_then(|p| p.fields.get(key)) {
                    Some(value) => merged.insert(key.to_owned(), value.clone()),
                    None => merged.remove(key),
                };
            }
            let mut normalized = normalize(
                doc_type,
                &id,
                merged,
                live.map(|p| &p.fields),
                now_ms(),
                if origin == Origin::Sync {
                    Refs::Skip
                } else {
                    Refs::Check(tx)
                },
            )?;

            if doc_type == DocType::File {
                let blob = match attachment {
                    Some(blob) => {
                        if blob.data.len() > MAX_ATTACHMENT_SIZE {
                            return Err(StoreError::Invalid(format!(
                                "The file is larger than {MAX_ATTACHMENT_SIZE} bytes."
                            )));
                        }
                        blob
                    }
                    None => match live {
                        Some(_) => load_blob(tx, &id)?.ok_or(StoreError::NotFound)?,
                        None => {
                            return Err(StoreError::Invalid(
                                "The file has no attachment data.".into(),
                            ));
                        }
                    },
                };
                normalized.insert("contentLength".into(), blob.data.len().into());
                normalized.insert("md5digest".into(), blob.md5_hex().into());
                tx.execute(
                    "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                     ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                    params![id, blob.md5_hex(), blob.data],
                )?;
            }

            if let Some(prev) = live
                && prev.fields == normalized
            {
                return Ok(WriteResult {
                    id,
                    rev: prev.rev.clone(),
                });
            }
            let rev = next_rev(previous.as_ref(), &Value::Object(normalized.clone()))?;
            store_row(
                tx,
                &id,
                doc_type,
                &rev,
                false,
                &normalized,
                previous.as_ref(),
                Synced::Keep,
                origin,
                changes,
            )?;
            Ok(WriteResult { id, rev })
        })
    }

    /// Deletes a document with Inkdrop's semantics: a notebook's notes move to
    /// the trash and its sub-notebooks are deleted; a tag is removed from its
    /// notes first; a note or file is deleted outright.
    pub fn delete(&self, id: &str, origin: Origin) -> Result<WriteResult> {
        self.write(|tx, changes| {
            let prev = load_row(tx, id)?
                .filter(|p| !p.deleted)
                .ok_or(StoreError::NotFound)?;
            let now = now_ms();
            match prev.doc_type {
                DocType::Book => {
                    let mut books = vec![id.to_owned()];
                    let mut i = 0;
                    while i < books.len() {
                        let mut stmt = tx.prepare(
                            "SELECT id FROM docs WHERE type = 'book' AND deleted = 0 AND parent_book_id = ?",
                        )?;
                        let children: Vec<String> = stmt
                            .query_map([&books[i]], |row| row.get(0))?
                            .collect::<Result<_, _>>()?;
                        books.extend(children);
                        i += 1;
                    }
                    for book in &books {
                        let notes: Vec<String> = tx
                            .prepare("SELECT id FROM docs WHERE type = 'note' AND deleted = 0 AND book_id = ?")?
                            .query_map([book], |row| row.get(0))?
                            .collect::<Result<_, _>>()?;
                        for note in notes {
                            rewrite_note(tx, &note, origin, changes, |fields| {
                                fields.insert("bookId".into(), TRASH_BOOK_ID.into());
                                fields.insert("updatedAt".into(), now.into());
                            })?;
                        }
                    }
                    for book in books.iter().skip(1) {
                        if let Some(child) = load_row(tx, book)? {
                            delete_row(tx, book, &child, origin, changes)?;
                        }
                    }
                }
                DocType::Tag => {
                    let notes: Vec<String> = tx
                        .prepare("SELECT note_id FROM note_tags WHERE tag_id = ?")?
                        .query_map([id], |row| row.get(0))?
                        .collect::<Result<_, _>>()?;
                    for note in notes {
                        rewrite_note(tx, &note, origin, changes, |fields| {
                            if let Some(Value::Array(tags)) = fields.get_mut("tags") {
                                tags.retain(|t| t.as_str() != Some(id));
                            }
                        })?;
                    }
                }
                DocType::Note | DocType::File => {}
            }
            let rev = delete_row(tx, id, &prev, origin, changes)?;
            Ok(WriteResult {
                id: id.to_owned(),
                rev,
            })
        })
    }

    /// Writes an imported document as-is, keeping its revision. Returns
    /// `false` when the store already has that revision, or a newer one made
    /// in Kanso. A replaced version is kept in the document's history.
    pub fn import_doc(
        &self,
        doc_type: DocType,
        id: &str,
        rev: &str,
        fields: Map<String, Value>,
        attachment: Option<Attachment>,
    ) -> Result<bool> {
        let incoming: Rev = rev
            .parse()
            .map_err(|e: kanso_core::rev::InvalidRev| StoreError::Invalid(e.to_string()))?;
        self.write(|tx, changes| {
            let previous = load_row(tx, id)?;
            if let Some(previous) = &previous {
                if previous.rev == rev && !previous.deleted {
                    return Ok(false);
                }
                // A copy edited or deleted in Kanso since the last import has
                // a generation at least as high; like CouchDB, it wins.
                let local: Option<Rev> = previous.rev.parse().ok();
                if local.is_some_and(|local| local.generation >= incoming.generation) {
                    return Ok(false);
                }
            }
            let created_at = fields
                .get("createdAt")
                .and_then(Value::as_i64)
                .unwrap_or_else(now_ms);
            let mut normalized = normalize(doc_type, id, fields, None, created_at, Refs::Skip)?;
            if let Some(blob) = &attachment {
                normalized.insert("contentLength".into(), blob.data.len().into());
                tx.execute(
                    "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                     ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                    params![id, blob.md5_hex(), blob.data],
                )?;
            }
            store_row(
                tx,
                id,
                doc_type,
                rev,
                false,
                &normalized,
                previous.as_ref(),
                Synced::Keep,
                Origin::Import,
                changes,
            )?;
            Ok(true)
        })
    }

    /// Keeps an imported document Kanso doesn't use, verbatim.
    pub fn archive(&self, id: &str, json: &Value) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO archive (id, json) VALUES (?1, ?2)
                 ON CONFLICT (id) DO UPDATE SET json = excluded.json",
                params![id, json.to_string()],
            )?;
            Ok(())
        })
    }
}

/// Applies `edit` to a live note and stores it under a new revision.
pub(crate) fn rewrite_note(
    tx: &Transaction,
    id: &str,
    origin: Origin,
    changes: &mut Vec<Change>,
    edit: impl FnOnce(&mut Map<String, Value>),
) -> Result<()> {
    let Some(prev) = load_row(tx, id)?.filter(|p| !p.deleted) else {
        return Ok(());
    };
    let mut fields = prev.fields.clone();
    edit(&mut fields);
    let fields = normalize(
        DocType::Note,
        id,
        fields,
        Some(&prev.fields),
        now_ms(),
        Refs::Skip,
    )?;
    let rev = next_rev(Some(&prev), &Value::Object(fields.clone()))?;
    store_row(
        tx,
        id,
        DocType::Note,
        &rev,
        false,
        &fields,
        Some(&prev),
        Synced::Keep,
        origin,
        changes,
    )
}

pub(crate) fn load_blob(tx: &rusqlite::Connection, id: &str) -> Result<Option<Attachment>> {
    Ok(tx
        .query_row("SELECT data FROM blobs WHERE file_id = ?", [id], |row| {
            row.get::<_, Vec<u8>>(0)
        })
        .optional()?
        .map(Attachment::new))
}

/// Removes `_attachments` from a file body, returning the decoded bytes when
/// `index.data` is present.
fn take_attachment(fields: &mut Map<String, Value>) -> Result<Option<Attachment>> {
    let Some(attachments) = fields.remove("_attachments") else {
        return Ok(None);
    };
    let data = attachments
        .get("index")
        .and_then(|index| index.get("data"))
        .and_then(Value::as_str);
    let Some(data) = data else { return Ok(None) };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.trim())
        .map_err(|_| StoreError::Invalid("The attachment data is not valid base64.".into()))?;
    if let Some(content_type) = attachments["index"]
        .get("content_type")
        .and_then(Value::as_str)
        && !fields.contains_key("contentType")
    {
        fields.insert("contentType".into(), content_type.into());
    }
    Ok(Some(Attachment::new(bytes)))
}
