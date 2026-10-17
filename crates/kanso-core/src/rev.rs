//! CouchDB-style revisions: `"<generation>-<32 hex digits>"`.
//!
//! Only the current revision of a document is kept. The generation counts
//! writes; the hash is an MD5 of the document's content with object keys
//! sorted, so identical content always gets the same revision.

use std::fmt;
use std::str::FromStr;

use md5::{Digest, Md5};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rev {
    pub generation: u64,
    pub hash: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid revision {0:?}")]
pub struct InvalidRev(pub String);

impl Rev {
    /// The revision that follows `previous` for a document with `content`.
    ///
    /// Fields starting with `_` (`_id`, `_rev`, `_attachments`) are not part
    /// of the hashed content.
    pub fn next(previous: Option<&Rev>, content: &Value) -> Rev {
        let mut hasher = Md5::new();
        hash_canonical(content, &mut hasher, true);
        Rev {
            generation: previous.map_or(1, |p| p.generation.saturating_add(1)),
            hash: hex(&hasher.finalize()),
        }
    }
}

impl fmt::Display for Rev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.generation, self.hash)
    }
}

impl FromStr for Rev {
    type Err = InvalidRev;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidRev(s.to_owned());
        let (generation, hash) = s.split_once('-').ok_or_else(invalid)?;
        if generation.is_empty() || !generation.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid());
        }
        let generation = generation.parse().map_err(|_| invalid())?;
        if hash.is_empty() || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        Ok(Rev {
            generation,
            hash: hash.to_owned(),
        })
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Feeds a canonical encoding of `value` to `hasher`: object keys sorted,
/// numbers and strings in their JSON form.
fn hash_canonical(value: &Value, hasher: &mut Md5, top_level: bool) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map
                .keys()
                .filter(|k| !(top_level && k.starts_with('_')))
                .collect();
            keys.sort();
            hasher.update(b"{");
            for key in keys {
                hasher.update(Value::String(key.clone()).to_string().as_bytes());
                hasher.update(b":");
                hash_canonical(&map[key.as_str()], hasher, false);
                hasher.update(b",");
            }
            hasher.update(b"}");
        }
        Value::Array(items) => {
            hasher.update(b"[");
            for item in items {
                hash_canonical(item, hasher, false);
                hasher.update(b",");
            }
            hasher.update(b"]");
        }
        other => hasher.update(other.to_string().as_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_inkdrop_revisions() {
        let rev: Rev = "36-805e5889052b3461f2741cd5b702a067".parse().unwrap();
        assert_eq!(rev.generation, 36);
        assert_eq!(rev.to_string(), "36-805e5889052b3461f2741cd5b702a067");
        assert!("abc".parse::<Rev>().is_err());
        assert!("-abc".parse::<Rev>().is_err());
        assert!("1-".parse::<Rev>().is_err());
        assert!("1-xyz".parse::<Rev>().is_err());
    }

    #[test]
    fn next_ignores_key_order_and_underscore_fields() {
        let a = json!({"title": "x", "body": "y", "_rev": "1-aa"});
        let b = json!({"body": "y", "title": "x", "_id": "note:abc"});
        assert_eq!(Rev::next(None, &a), Rev::next(None, &b));
        assert_eq!(Rev::next(None, &a).generation, 1);
        let prev = Rev::next(None, &a);
        assert_eq!(Rev::next(Some(&prev), &a).generation, 2);
        assert_ne!(Rev::next(None, &a), Rev::next(None, &json!({"title": "z"})));
    }
}
