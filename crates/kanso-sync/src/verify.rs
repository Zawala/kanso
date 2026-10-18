//! `kanso-sync verify` and `estimate` (docs/ACCOUNTS.md › Command line).
//! Frozen by step 1; the bodies are step 2b, except the legacy manifest hash,
//! which the admin script must reproduce byte for byte (security re-check 6).

use std::path::Path;

use kanso_store::{Store, StoreError};
use serde::{Deserialize, Serialize};

use crate::account::Session;
use crate::api::Backend;
use crate::engine::SyncError;

/// What `verify` found, comparing the whole feed (opened) with the library.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyReport {
    /// Local documents with `synced_rev == rev` found at that revision with
    /// equal fields, and live files whose bytes match.
    pub matched: usize,
    pub missing: usize,
    /// On the server but not in the library.
    pub extra: usize,
    pub different: usize,
    pub undecryptable: usize,
    /// Local changes not yet synced.
    pub pending: usize,
    /// With `--repair`: undecryptable server copies replaced from this Mac.
    pub repaired: usize,
    /// With `--legacy`: the manifest check.
    pub legacy: Option<LegacyReport>,
}

impl VerifyReport {
    /// Exit 0: nothing missing, different, undecryptable or pending, and the
    /// legacy check (if any) passed.
    pub fn ok(&self) -> bool {
        self.missing == 0
            && self.different == 0
            && self.undecryptable == 0
            && self.pending == 0
            && self.legacy.as_ref().is_none_or(|l| l.ok)
    }
}

/// The `--legacy` result, written as `R.json`: `{"manifest_sha256",
/// "account_id", "ok", "rows", "failed"}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyReport {
    /// [`manifest_sha256`] of the manifest's rows.
    pub manifest_sha256: String,
    pub account_id: String,
    pub ok: bool,
    /// Rows checked.
    pub rows: usize,
    /// IDs of rows that failed, sorted.
    pub failed: Vec<String>,
}

/// One row of `admin legacy-export`'s `manifest.json`: `[id, rev, deleted,
/// md5digest]`, `md5digest` a lowercase hex string for files and null
/// otherwise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    from = "(String, String, bool, Option<String>)",
    into = "(String, String, bool, Option<String>)"
)]
pub struct LegacyRow {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    pub md5digest: Option<String>,
}

impl From<(String, String, bool, Option<String>)> for LegacyRow {
    fn from((id, rev, deleted, md5digest): (String, String, bool, Option<String>)) -> Self {
        LegacyRow {
            id,
            rev,
            deleted,
            md5digest,
        }
    }
}

impl From<LegacyRow> for (String, String, bool, Option<String>) {
    fn from(row: LegacyRow) -> Self {
        (row.id, row.rev, row.deleted, row.md5digest)
    }
}

/// `manifest.json`: `{"exported_at", "rows": [[id, rev, deleted, md5digest]]}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyManifest {
    pub exported_at: i64,
    pub rows: Vec<LegacyRow>,
}

/// The manifest hash `verify --legacy` writes and `purge-legacy` recomputes
/// from a fresh export (security re-check 6). Rows only, never
/// `exported_at`: the rows sorted by `id` (bytewise), each written as
/// `id TAB rev TAB ("1" | "0") TAB (md5digest | "") LF` in UTF-8, and the
/// SHA-256 of those bytes as 64 lowercase hex digits. Pure; implemented in
/// step 1.
pub fn manifest_sha256(rows: &[LegacyRow]) -> String {
    use sha2::Digest as _;
    let mut sorted: Vec<&LegacyRow> = rows.iter().collect();
    sorted.sort_by(|a, b| a.id.as_bytes().cmp(b.id.as_bytes()));
    let mut canonical = String::new();
    for row in sorted {
        canonical.push_str(&row.id);
        canonical.push('\t');
        canonical.push_str(&row.rev);
        canonical.push('\t');
        canonical.push(if row.deleted { '1' } else { '0' });
        canonical.push('\t');
        canonical.push_str(row.md5digest.as_deref().unwrap_or(""));
        canonical.push('\n');
    }
    sha2::Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The charged bytes a library needs on a v2 server (`kanso-sync estimate`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    /// Live documents.
    pub docs: u64,
    /// `length(sealed) + 512` over every document, tombstones included.
    pub doc_bytes: u64,
    /// Live files with bytes.
    pub files: u64,
    /// `size + 41 + 1024` over those files.
    pub file_bytes: u64,
}

/// Pulls the whole feed into memory without applying it, opens every
/// document and live file, and compares them with the library. With
/// `legacy`, also checks each manifest row; with `repair`, pushes this
/// Mac's version over undecryptable server copies (step 2b).
pub fn verify(
    store: &Store,
    b: &dyn Backend,
    session: &Session,
    legacy: Option<&Path>,
    repair: bool,
) -> Result<VerifyReport, SyncError> {
    crate::stub!("2b", store, b, session, legacy, repair)
}

/// The charged bytes the library needs (step 2b).
pub fn estimate(store: &Store) -> Result<Estimate, StoreError> {
    crate::stub!("2b", store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn row(id: &str, rev: &str, deleted: bool, md5: Option<&str>) -> LegacyRow {
        LegacyRow {
            id: id.into(),
            rev: rev.into(),
            deleted,
            md5digest: md5.map(Into::into),
        }
    }

    /// The pinned vector the admin script must reproduce (also in
    /// docs/ACCOUNTS.md › Contract notes).
    #[test]
    fn manifest_hash_vector() {
        let rows = [
            row("note:b", "2-bb", false, None),
            row(
                "file:a",
                "1-aa",
                false,
                Some("d41d8cd98f00b204e9800998ecf8427e"),
            ),
            row("tag:c", "3-cc", true, None),
        ];
        let canonical = "file:a\t1-aa\t0\td41d8cd98f00b204e9800998ecf8427e\nnote:b\t2-bb\t0\t\ntag:c\t3-cc\t1\t\n";
        use sha2::Digest as _;
        let expected: String = sha2::Sha256::digest(canonical.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(manifest_sha256(&rows), expected);
        assert_eq!(
            expected,
            "a0dbdfeedc42d0d264500b4faad3718aa823528946c653f34e62b2be42575b92"
        );
        assert_eq!(
            manifest_sha256(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn manifest_rows_use_the_array_shape() {
        let manifest: LegacyManifest = serde_json::from_str(
            r#"{"exported_at": 5, "rows": [["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"], ["note:b", "2-bb", true, null]]}"#,
        )
        .unwrap();
        assert_eq!(manifest.rows[1], row("note:b", "2-bb", true, None));
        let back = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            back["rows"][0],
            serde_json::json!(["file:a", "1-aa", false, "d41d8cd98f00b204e9800998ecf8427e"])
        );
    }

    #[test]
    fn verify_report_ok() {
        assert!(VerifyReport::default().ok());
        assert!(
            !VerifyReport {
                pending: 1,
                ..VerifyReport::default()
            }
            .ok()
        );
        assert!(
            VerifyReport {
                extra: 3,
                repaired: 1,
                ..VerifyReport::default()
            }
            .ok()
        );
        let failed = LegacyReport {
            ok: false,
            ..LegacyReport::default()
        };
        assert!(
            !VerifyReport {
                legacy: Some(failed),
                ..VerifyReport::default()
            }
            .ok()
        );
    }

    fn gen_rows() -> impl Strategy<Value = Vec<LegacyRow>> {
        prop::collection::vec(
            (
                "(note|book|tag|file):[a-z0-9]{1,4}",
                "[1-9]-[0-9a-f]{2}",
                any::<bool>(),
                prop::option::of("[0-9a-f]{32}"),
            )
                .prop_map(|(id, rev, deleted, md5digest)| LegacyRow {
                    id,
                    rev,
                    deleted,
                    md5digest,
                }),
            0..8,
        )
        .prop_map(|mut rows| {
            rows.sort_by(|a, b| a.id.cmp(&b.id));
            rows.dedup_by(|a, b| a.id == b.id);
            rows
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Metamorphic: the hash ignores row order and `exported_at`, and any
        /// change to a row's revision, deleted flag or digest changes it.
        #[test]
        fn prop_manifest_hash_is_order_free_and_sensitive(rows in gen_rows(), seed in any::<u64>(), at in any::<prop::sample::Index>()) {
            let mut shuffled = rows.clone();
            let n = shuffled.len();
            for i in (1..n).rev() {
                shuffled.swap(i, (seed as usize).wrapping_mul(31).wrapping_add(i) % (i + 1));
            }
            prop_assert_eq!(manifest_sha256(&rows), manifest_sha256(&shuffled));
            if !rows.is_empty() {
                let i = at.index(rows.len());
                let mut changed = rows.clone();
                changed[i].deleted = !changed[i].deleted;
                prop_assert_ne!(manifest_sha256(&rows), manifest_sha256(&changed));
                let mut changed = rows.clone();
                changed[i].md5digest = match changed[i].md5digest { Some(_) => None, None => Some("0".repeat(32)) };
                prop_assert_ne!(manifest_sha256(&rows), manifest_sha256(&changed));
            }
        }
    }
}
