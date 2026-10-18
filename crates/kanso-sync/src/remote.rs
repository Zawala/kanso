//! The sync protocol: wire types, limits and the [`Remote`] trait.
//!
//! The Cloudflare Worker in `cloud/` and [`crate::MemoryRemote`] implement
//! the same rules; `cloud/test/fixtures/conformance-v1.json` holds the cases
//! both test suites run.

use kanso_core::DocType;
use kanso_core::id::is_valid_id;
use kanso_store::SyncDoc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Largest accepted push request body (bytes).
pub const MAX_PUSH_BYTES: usize = 16 * 1024 * 1024;
/// Most documents in one push. The Worker runs three D1 statements per
/// document in one batch and an invocation may run 1,000.
pub const MAX_PUSH_DOCS: usize = 200;
/// Largest serialized `fields` of one document (bytes). D1 rows top out at 2 MB.
pub const MAX_FIELDS_BYTES: usize = 1_900_000;
/// Largest attachment (bytes), the store's own limit.
pub const MAX_FILE_BYTES: usize = kanso_core::model::MAX_ATTACHMENT_SIZE;
/// Page size of `changes` when none is given, and the largest allowed.
pub const DEFAULT_CHANGES_LIMIT: usize = 100;
pub const MAX_CHANGES_LIMIT: usize = 500;

/// The outcome of pushing one document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum PushResult {
    /// The server stored the document under change sequence `seq`.
    Ok { id: String, seq: i64 },
    /// The document's `base_rev` is not the server's current revision.
    /// `current` is the server's version, or `None` when the server has no
    /// copy although the client sent a `base_rev`.
    Conflict {
        id: String,
        current: Option<SyncDoc>,
    },
}

impl PushResult {
    pub fn id(&self) -> &str {
        match self {
            PushResult::Ok { id, .. } | PushResult::Conflict { id, .. } => id,
        }
    }
}

/// A document in the change feed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoteDoc {
    pub id: String,
    pub rev: String,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub fields: Map<String, Value>,
    pub seq: i64,
}

impl RemoteDoc {
    pub fn to_sync_doc(&self) -> SyncDoc {
        SyncDoc {
            id: self.id.clone(),
            rev: self.rev.clone(),
            deleted: self.deleted,
            fields: self.fields.clone(),
            base_rev: None,
        }
    }
}

/// One page of `GET /v1/changes`: documents with `seq > since`, in order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangesPage {
    pub docs: Vec<RemoteDoc>,
    /// The `seq` of the last document, or `since` for an empty page.
    pub last_seq: i64,
}

/// Attachment bytes as the server stores them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteFile {
    pub md5_hex: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum RemoteError {
    #[error("the sync server rejected the token")]
    Unauthorized,
    #[error("the sync server answered {status}: {message}")]
    Status { status: u16, message: String },
    #[error("could not reach the sync server: {0}")]
    Transport(String),
    #[error("unexpected answer from the sync server: {0}")]
    Protocol(String),
}

impl RemoteError {
    pub fn status(&self) -> Option<u16> {
        match self {
            RemoteError::Unauthorized => Some(401),
            RemoteError::Status { status, .. } => Some(*status),
            _ => None,
        }
    }
}

/// A sync server.
pub trait Remote {
    /// `POST /v1/push`: one result per document, in order. Documents are
    /// applied one after another, so a later one may build on an earlier one.
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError>;
    /// `GET /v1/changes?since=&limit=`.
    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError>;
    /// `PUT /v1/files/<id>`; the server checks the bytes against `md5_hex`.
    fn put_file(
        &self,
        id: &str,
        md5_hex: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError>;
    /// `GET /v1/files/<id>`; `None` when the server has no bytes for `id`.
    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError>;
}

impl<R: Remote + ?Sized> Remote for &R {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        (**self).push(docs)
    }
    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        (**self).changes(since, limit)
    }
    fn put_file(
        &self,
        id: &str,
        md5_hex: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError> {
        (**self).put_file(id, md5_hex, content_type, bytes)
    }
    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        (**self).get_file(id)
    }
}

/// `note:…`, `book:…`, `tag:…` or `file:…` with 1–120 of `[A-Za-z0-9_.-]`.
pub fn is_doc_id(id: &str) -> bool {
    DocType::of_id(id).is_some_and(|t| is_valid_id(t, id))
}

pub fn is_file_id(id: &str) -> bool {
    is_valid_id(DocType::File, id)
}

/// `<1–16 digits>-<1–64 hex digits>`.
pub fn is_rev(rev: &str) -> bool {
    let Some((generation, hash)) = rev.split_once('-') else {
        return false;
    };
    (1..=16).contains(&generation.len())
        && generation.bytes().all(|b| b.is_ascii_digit())
        && (1..=64).contains(&hash.len())
        && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

/// 32 hex digits.
pub fn is_md5_hex(md5: &str) -> bool {
    md5.len() == 32 && md5.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The MD5 of `bytes` as 32 lowercase hex digits.
pub fn md5_hex(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    Md5::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The push validation the Worker applies, as an error message.
pub fn validate_push(docs: &[SyncDoc]) -> Result<(), (u16, String)> {
    if docs.len() > MAX_PUSH_DOCS {
        return Err((
            413,
            format!("A push carries at most {MAX_PUSH_DOCS} documents."),
        ));
    }
    for (i, doc) in docs.iter().enumerate() {
        if !is_doc_id(&doc.id) {
            return Err((400, format!("docs[{i}].id is not a document ID.")));
        }
        if !is_rev(&doc.rev) {
            return Err((400, format!("docs[{i}].rev is not a revision.")));
        }
        if doc.base_rev.as_deref().is_some_and(|b| !is_rev(b)) {
            return Err((400, format!("docs[{i}].base_rev is not a revision.")));
        }
        let size = serde_json::to_string(&doc.fields).map_or(usize::MAX, |s| s.len());
        if size > MAX_FIELDS_BYTES {
            return Err((
                413,
                format!("docs[{i}].fields is larger than {MAX_FIELDS_BYTES} bytes."),
            ));
        }
    }
    Ok(())
}

/// Percent-encodes one URL path segment.
pub(crate) fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for b in segment.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_ids_revs_and_digests() {
        assert!(is_doc_id("note:abc_1-2.x"));
        assert!(is_doc_id("book:inbox"));
        assert!(!is_doc_id("note:"));
        assert!(!is_doc_id("page:abc"));
        assert!(!is_doc_id("note:a/b"));
        assert!(is_file_id("file:x"));
        assert!(!is_file_id("note:x"));
        assert!(is_rev("1-0a"));
        assert!(is_rev("36-805e5889052b3461f2741cd5b702a067"));
        assert!(!is_rev("1-"));
        assert!(!is_rev("-a"));
        assert!(!is_rev("x-a"));
        assert!(!is_rev("1-xyz"));
        assert!(is_md5_hex("d41d8cd98f00b204e9800998ecf8427e"));
        assert!(!is_md5_hex("d41d8cd98f00b204e9800998ecf8427"));
    }

    #[test]
    fn encodes_path_segments() {
        assert_eq!(encode_segment("file:ab_c-1.x~"), "file%3Aab_c-1.x~");
        assert_eq!(encode_segment("a b/ü"), "a%20b%2F%C3%BC");
    }

    #[test]
    fn push_results_use_the_wire_shape() {
        let ok: PushResult =
            serde_json::from_str(r#"{"id":"note:a","status":"ok","seq":3}"#).unwrap();
        assert_eq!(
            ok,
            PushResult::Ok {
                id: "note:a".into(),
                seq: 3
            }
        );
        let conflict: PushResult = serde_json::from_str(
            r#"{"id":"note:a","status":"conflict","current":{"id":"note:a","rev":"2-ab","deleted":false,"fields":{"x":1}}}"#,
        )
        .unwrap();
        let PushResult::Conflict {
            current: Some(current),
            ..
        } = conflict
        else {
            panic!("expected a conflict");
        };
        assert_eq!(current.rev, "2-ab");
        assert_eq!(current.base_rev, None);
        let missing: PushResult =
            serde_json::from_str(r#"{"id":"note:a","status":"conflict","current":null}"#).unwrap();
        assert_eq!(
            missing,
            PushResult::Conflict {
                id: "note:a".into(),
                current: None
            }
        );
    }
}
