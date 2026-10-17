//! Document IDs: a type prefix and a short random suffix, e.g. `note:07HmFZwo`.

use std::fmt;

/// The kinds of document Kanso stores, named by their ID prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocType {
    Note,
    Book,
    Tag,
    File,
}

impl DocType {
    pub const ALL: [DocType; 4] = [DocType::Note, DocType::Book, DocType::Tag, DocType::File];

    /// The ID prefix without the colon, also used as the type name in storage.
    pub fn as_str(self) -> &'static str {
        match self {
            DocType::Note => "note",
            DocType::Book => "book",
            DocType::Tag => "tag",
            DocType::File => "file",
        }
    }

    /// The plural collection name used by the HTTP API (`/notes`, `/books`, …).
    pub fn collection(self) -> &'static str {
        match self {
            DocType::Note => "notes",
            DocType::Book => "books",
            DocType::Tag => "tags",
            DocType::File => "files",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == name)
    }

    pub fn from_collection(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.collection() == name)
    }

    /// The type an ID belongs to, judged by its prefix.
    pub fn of_id(id: &str) -> Option<Self> {
        let (prefix, rest) = id.split_once(':')?;
        if rest.is_empty() {
            return None;
        }
        Self::from_name(prefix)
    }
}

impl fmt::Display for DocType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

const ID_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const ID_LEN: usize = 8;

/// A fresh random ID in Inkdrop's format: the type prefix and 8 URL-safe characters.
pub fn new_id(doc_type: DocType) -> String {
    let suffix: String = (0..ID_LEN)
        .map(|_| ID_ALPHABET[fastrand::usize(..ID_ALPHABET.len())] as char)
        .collect();
    format!("{}:{suffix}", doc_type.as_str())
}

/// Adds `prefix:` to `id` unless it already starts with it.
pub fn with_prefix(doc_type: DocType, id: &str) -> String {
    let prefix = doc_type.as_str();
    match id
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix(':'))
    {
        Some(_) => id.to_owned(),
        None => format!("{prefix}:{id}"),
    }
}

/// Whether `id` is a well-formed ID of `doc_type`: the prefix, then 1–120
/// characters that are safe in URLs and in `inkdrop://` links.
pub fn is_valid_id(doc_type: DocType, id: &str) -> bool {
    let Some(suffix) = id
        .strip_prefix(doc_type.as_str())
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return false;
    };
    (1..=120).contains(&suffix.len())
        && suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_have_the_inkdrop_shape() {
        let id = new_id(DocType::Note);
        assert!(id.starts_with("note:"));
        assert_eq!(id.len(), "note:".len() + ID_LEN);
        assert!(is_valid_id(DocType::Note, &id));
        assert_eq!(DocType::of_id(&id), Some(DocType::Note));
    }

    #[test]
    fn real_inkdrop_ids_are_valid() {
        for id in ["note:07HmFZwo", "note:rZmL6-B0", "note:-7d3_V-P"] {
            assert!(is_valid_id(DocType::Note, id), "{id}");
        }
        assert!(is_valid_id(DocType::Book, "book:inbox"));
        assert!(!is_valid_id(DocType::Book, "note:abc"));
        assert!(!is_valid_id(DocType::Note, "note:"));
        assert!(!is_valid_id(DocType::Note, "note:a/b"));
    }

    #[test]
    fn prefixing_is_idempotent() {
        assert_eq!(with_prefix(DocType::Tag, "abc"), "tag:abc");
        assert_eq!(with_prefix(DocType::Tag, "tag:abc"), "tag:abc");
    }
}
