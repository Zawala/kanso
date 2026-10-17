//! Helpers shared by the sync test suites.
#![allow(dead_code)]

use std::collections::BTreeMap;

use base64::Engine as _;
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store, StoreError, WriteResult};
use serde_json::{Map, Value, json};

pub fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

pub fn put(store: &Store, doc_type: DocType, body: Value) -> Result<WriteResult, StoreError> {
    store.put(doc_type, body, Put::Upsert, Origin::Api)
}

pub fn merge(store: &Store, doc_type: DocType, body: Value) -> Result<WriteResult, StoreError> {
    store.put(doc_type, body, Put::Merge, Origin::Api)
}

pub fn book(store: &Store, id: &str, name: &str) -> WriteResult {
    put(
        store,
        DocType::Book,
        json!({"_id": id, "name": name, "createdAt": 1, "updatedAt": 1}),
    )
    .unwrap()
}

pub fn note(store: &Store, id: &str, book: &str, body: &str, updated: i64) -> WriteResult {
    put(
        store,
        DocType::Note,
        json!({"_id": id, "bookId": book, "title": id, "body": body, "createdAt": 1, "updatedAt": updated}),
    )
    .unwrap()
}

/// Replaces a note's body (full-document update with the current `_rev`).
pub fn edit_note(store: &Store, id: &str, body: &str, updated: Option<i64>) -> WriteResult {
    let mut doc = store.get_api(id, None, false).unwrap();
    doc["body"] = body.into();
    if let Some(updated) = updated {
        doc["updatedAt"] = updated.into();
    }
    put(store, DocType::Note, doc).unwrap()
}

pub fn file_body(id: &str, name: &str, bytes: &[u8]) -> Value {
    json!({
        "_id": id,
        "name": name,
        "contentType": "image/png",
        "createdAt": 1,
        "_attachments": {"index": {
            "content_type": "image/png",
            "data": base64::engine::general_purpose::STANDARD.encode(bytes),
        }},
    })
}

pub fn file(store: &Store, id: &str, bytes: &[u8]) -> WriteResult {
    put(store, DocType::File, file_body(id, "image.png", bytes)).unwrap()
}

pub fn body(store: &Store, id: &str) -> Option<String> {
    store.note(id).unwrap().map(|n| n.body)
}

/// Every document, tombstones included: id -> (rev, deleted).
pub fn all_docs(store: &Store) -> BTreeMap<String, (String, bool)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted)))
        .collect()
}

/// Live documents: id -> (rev, fields).
pub fn live_docs(store: &Store) -> BTreeMap<String, (String, Map<String, Value>)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .filter(|d| !d.deleted)
        .map(|d| (d.id, (d.rev, d.fields)))
        .collect()
}

pub fn has_revision(store: &Store, id: &str, rev: &str) -> bool {
    store.revisions(id).unwrap().iter().any(|r| r.rev == rev)
}

pub fn pending(store: &Store) -> usize {
    store.pending_push(usize::MAX).unwrap().len()
}
