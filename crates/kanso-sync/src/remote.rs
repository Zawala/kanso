//! The engine's plaintext side of sync: documents with local IDs and
//! revisions. `Sealed` (`sealed.rs`) opens v2 documents into [`RemoteDoc`]
//! and [`PushResult`], and `PlainView` uses them to show tests the server's
//! documents in plaintext.

use kanso_core::DocType;
use kanso_core::id::is_valid_id;
use kanso_store::SyncDoc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The outcome of pushing one document, with local IDs and revisions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum PushResult {
    /// The server stored the document under change sequence `seq`.
    Ok { id: String, seq: i64 },
    /// The document's `base_rev` is not the server's current revision.
    /// `current` is the server's version, opened (always `Some` from
    /// `Sealed`; a server that lost the document says [`PushResult::Lost`]).
    Conflict {
        id: String,
        current: Option<SyncDoc>,
    },
    /// The client sent a `base_rev`, but the server has no such
    /// document. The engine pushes it again with no base (M1).
    Lost { id: String },
    /// A conflict whose current version didn't fit the response's
    /// budget. The engine pulls before the next round.
    Behind { id: String },
    /// A conflict whose current version couldn't be opened:
    /// undecryptable (format 1), or a newer format.
    Unreadable { id: String, newer_format: bool },
}

impl PushResult {
    pub fn id(&self) -> &str {
        match self {
            PushResult::Ok { id, .. }
            | PushResult::Conflict { id, .. }
            | PushResult::Lost { id }
            | PushResult::Behind { id }
            | PushResult::Unreadable { id, .. } => id,
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

#[derive(Debug, thiserror::Error)]
pub enum RemoteError {
    #[error("the sync server ended the session")]
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
