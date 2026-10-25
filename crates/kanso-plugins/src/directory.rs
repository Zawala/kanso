//! Building the plugin directory: the next `directory/` from the plugin
//! folders, the revocation list and the directory published before.
//!
//! [`build`] is pure. It never signs; a separate, protected step signs the
//! index it makes. The disk helpers ([`read_plugins`], [`write_built`],
//! [`verify_dir`]) are thin wrappers for the `kanso-plugin` tool.
//!
//! How the pieces fit together:
//!
//! - Every previous entry stays, byte for byte. Older versions stay listed
//!   so installed copies keep verifying and revocations can name them.
//! - `index.revoked` is exactly the revocation list. `Entry.revoked` is the
//!   per-version flag for a hand-made withdrawal; the builder keeps it as
//!   it was and sets it to `false` on new entries. The app reads both, so
//!   the revocation list is the one place the builder withdraws versions,
//!   and the two can't disagree.
//! - A plugin's author key is fixed by its first entry. The app pins that
//!   key on first install and shows a change in red; the directory is
//!   stricter and refuses a change outright, so a stolen account can't
//!   publish an update under another key. Moving to a new key means a new
//!   plugin ID.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ed25519_dalek::VerifyingKey;
use semver::Version;

use crate::bundle::{self, BundleError};
use crate::manifest::{self, ManifestError};
use crate::registry::{RegistryError, check_index, validate_index, verify_bundle};
use crate::registry_model::{Bundle, Entry, INDEX_SCHEMA, Index, Revocation};

/// The folder inside a plugin's folder that holds its files.
pub const SOURCE_DIR: &str = "source";
/// The file in a plugin's folder that holds the author's signature.
pub const SIGNATURE_FILE: &str = "author-signature.txt";
/// The index's file name in a built directory.
pub const INDEX_FILE: &str = "index.json";
/// The folder of bundles in a built directory.
pub const BUNDLES_DIR: &str = "bundles";

/// Why a directory couldn't be built or checked.
#[derive(Debug, thiserror::Error)]
pub enum DirectoryError {
    #[error("{plugin}: {source}")]
    Bundle {
        plugin: String,
        #[source]
        source: BundleError,
    },
    #[error("{plugin}: the manifest isn't valid: {source}")]
    Manifest {
        plugin: String,
        #[source]
        source: ManifestError,
    },
    #[error("{plugin}: the folder must be named after the manifest's ID, {id}")]
    FolderName { plugin: String, id: String },
    #[error("{plugin}: the author signature isn't the manifest's author's, over this bundle")]
    AuthorSignature { plugin: String },
    #[error("{plugin}: the version {version} isn't a plain semver version")]
    Version { plugin: String, version: String },
    #[error("{plugin} is given twice")]
    Duplicate { plugin: String },
    #[error("{id} {version} is already published with other contents: raise its version")]
    Republished { id: String, version: String },
    #[error("{id} {version} isn't newer than {newest}, which is already published")]
    NotNewer {
        id: String,
        version: String,
        newest: String,
    },
    #[error(
        "{id} is signed with another key than the one it was first published with; the directory never accepts a key change, so publish it under a new ID"
    )]
    KeyChanged { id: String },
    #[error("the revocation of {id} needs a reason")]
    NoReason { id: String },
    #[error("the previous directory isn't valid: {0}")]
    Previous(RegistryError),
    #[error("the new directory isn't valid: {0}")]
    Invalid(RegistryError),
    #[error("the serial can't be raised any further")]
    SerialOverflow,
    #[error("{id} {version}: {source}")]
    Listed {
        id: String,
        version: String,
        #[source]
        source: RegistryError,
    },
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("{0}")]
    Io(String),
}

/// One plugin folder, read: what the builder needs to judge it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The folder's name under `plugins/`, which must be the plugin's ID.
    pub folder_name: String,
    /// The packed `source/` folder, as `kanso-plugin pack` makes it.
    pub bundle_bytes: Vec<u8>,
    /// The author's base64 signature over the bundle's SHA-256.
    pub author_signature: String,
}

/// A built directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Built {
    /// The next `index.json`, not yet signed.
    pub index: Index,
    /// The bundles to add, as `(id, version, bytes)`, in index order.
    pub new_bundles: Vec<(String, String, Vec<u8>)>,
    /// Whether anything differs from the previous index.
    pub changed: bool,
}

/// The index entry for a bundle, built from its manifest. The author's
/// signature must verify over the bundle's hash with the manifest's key.
pub fn entry_for_bundle(
    bytes: &[u8],
    opened: &Bundle,
    author_signature: &str,
    published_at: &str,
) -> Result<Entry, BundleError> {
    let m = &opened.manifest;
    let signature = author_signature.trim();
    bundle::verify_hash(&opened.sha256, signature, &m.publisher.key)?;
    Ok(Entry {
        id: m.id.clone(),
        name: m.name.clone(),
        publisher: m.publisher.name.clone(),
        description: m.description.clone().unwrap_or_default(),
        version: m.version.clone(),
        sha256: opened.sha256.clone(),
        size: bytes.len() as u64,
        signer: m.publisher.key.clone(),
        author_signature: signature.to_owned(),
        min_app_version: m.min_app_version.clone(),
        permissions: m.permissions.clone(),
        published_at: published_at.to_owned(),
        revoked: false,
    })
}

/// A listed version, parsed. Previous entries have passed `validate_index`,
/// so this only fails for a version that was never checked.
fn parsed(entry: &Entry) -> Version {
    Version::parse(&entry.version).unwrap_or_else(|_| Version::new(0, 0, 0))
}

/// Makes the next directory. Pure.
///
/// `previous` is the index published before, `plugins` the folders under
/// `plugins/`, `revocations` the whole current list and `published_at` the
/// time stamped on entries added now. The rules are in the module notes;
/// the same inputs always give the same [`Index`].
pub fn build(
    previous: Option<Index>,
    plugins: Vec<Candidate>,
    revocations: Vec<Revocation>,
    published_at: &str,
) -> Result<Built, DirectoryError> {
    if let Some(previous) = &previous {
        validate_index(previous).map_err(DirectoryError::Previous)?;
    }
    if let Some(r) = revocations.iter().find(|r| r.reason.is_empty()) {
        return Err(DirectoryError::NoReason { id: r.id.clone() });
    }
    let first = previous.is_none();
    let (mut entries, old_serial, old_revoked) = match previous {
        Some(p) => (p.plugins, p.serial, p.revoked),
        None => (Vec::new(), 0, Vec::new()),
    };

    let mut plugins = plugins;
    plugins.sort_by(|a, b| a.folder_name.cmp(&b.folder_name));
    let mut folders = BTreeSet::new();
    let mut new_bundles = Vec::new();
    for candidate in plugins {
        let name = candidate.folder_name.clone();
        if !folders.insert(name.clone()) {
            return Err(DirectoryError::Duplicate { plugin: name });
        }
        let opened =
            bundle::open(&candidate.bundle_bytes).map_err(|source| DirectoryError::Bundle {
                plugin: name.clone(),
                source,
            })?;
        manifest::validate(&opened.manifest).map_err(|source| DirectoryError::Manifest {
            plugin: name.clone(),
            source,
        })?;
        let m = &opened.manifest;
        if m.id != name {
            return Err(DirectoryError::FolderName {
                plugin: name,
                id: m.id.clone(),
            });
        }
        let version = Version::parse(&m.version)
            .ok()
            .filter(|v| v.build.is_empty() && v.to_string() == m.version)
            .ok_or_else(|| DirectoryError::Version {
                plugin: name.clone(),
                version: m.version.clone(),
            })?;
        let entry = entry_for_bundle(
            &candidate.bundle_bytes,
            &opened,
            &candidate.author_signature,
            published_at,
        )
        .map_err(|_| DirectoryError::AuthorSignature {
            plugin: name.clone(),
        })?;

        if let Some(listed) = entries
            .iter()
            .find(|e| e.id == entry.id && e.version == entry.version)
        {
            if listed.sha256 == entry.sha256 && listed.size == entry.size {
                continue;
            }
            return Err(DirectoryError::Republished {
                id: entry.id,
                version: entry.version,
            });
        }
        let earlier: Vec<&Entry> = entries.iter().filter(|e| e.id == entry.id).collect();
        if earlier.iter().any(|e| e.signer != entry.signer) {
            return Err(DirectoryError::KeyChanged { id: entry.id });
        }
        if let Some(newest) = earlier.iter().map(|e| parsed(e)).max()
            && version <= newest
        {
            return Err(DirectoryError::NotNewer {
                id: entry.id,
                version: entry.version,
                newest: newest.to_string(),
            });
        }
        new_bundles.push((
            entry.id.clone(),
            entry.version.clone(),
            candidate.bundle_bytes,
        ));
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| parsed(a).cmp(&parsed(b))));
    new_bundles.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let changed = first || !new_bundles.is_empty() || old_revoked != revocations;
    let serial = if first {
        1
    } else if changed {
        old_serial
            .checked_add(1)
            .ok_or(DirectoryError::SerialOverflow)?
    } else {
        old_serial
    };
    let index = Index {
        schema: INDEX_SCHEMA,
        serial,
        plugins: entries,
        revoked: revocations,
    };
    validate_index(&index).map_err(DirectoryError::Invalid)?;
    Ok(Built {
        index,
        new_bundles,
        changed,
    })
}

fn io(what: &Path, e: impl std::fmt::Display) -> DirectoryError {
    DirectoryError::Io(format!("{}: {e}", what.display()))
}

/// Reads every plugin folder under `dir`: its `source/` packed, and its
/// author signature. Hidden entries and loose files are skipped.
pub fn read_plugins(dir: &Path) -> Result<Vec<Candidate>, DirectoryError> {
    let mut names = Vec::new();
    for item in std::fs::read_dir(dir).map_err(|e| io(dir, e))? {
        let item = item.map_err(|e| io(dir, e))?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || !item.path().is_dir() {
            continue;
        }
        names.push(name);
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let folder = dir.join(&name);
            let bundle_bytes = bundle::pack_dir(&folder.join(SOURCE_DIR)).map_err(|source| {
                DirectoryError::Bundle {
                    plugin: name.clone(),
                    source,
                }
            })?;
            let sig_path = folder.join(SIGNATURE_FILE);
            let author_signature = std::fs::read_to_string(&sig_path)
                .map_err(|e| io(&sig_path, e))?
                .trim()
                .to_owned();
            Ok(Candidate {
                folder_name: name,
                bundle_bytes,
                author_signature,
            })
        })
        .collect()
}

/// The path of a bundle in a built directory.
pub fn bundle_path(dir: &Path, id: &str, version: &str) -> PathBuf {
    dir.join(BUNDLES_DIR)
        .join(id)
        .join(format!("{version}.kansoplugin"))
}

/// Writes a changed build into `out`: `index.json` (pretty, with a trailing
/// newline) and each new bundle. A bundle already on disk with other bytes
/// is an error, never overwritten. Does nothing when nothing changed.
pub fn write_built(out: &Path, built: &Built) -> Result<(), DirectoryError> {
    if !built.changed {
        return Ok(());
    }
    for (id, version, bytes) in &built.new_bundles {
        let path = bundle_path(out, id, version);
        if path.exists() {
            if std::fs::read(&path).map_err(|e| io(&path, e))? != *bytes {
                return Err(DirectoryError::Io(format!(
                    "{} already exists with other contents",
                    path.display()
                )));
            }
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        std::fs::write(&path, bytes).map_err(|e| io(&path, e))?;
    }
    let mut text =
        serde_json::to_vec_pretty(&built.index).map_err(|e| DirectoryError::Io(e.to_string()))?;
    text.push(b'\n');
    std::fs::create_dir_all(out).map_err(|e| io(out, e))?;
    let path = out.join(INDEX_FILE);
    std::fs::write(&path, text).map_err(|e| io(&path, e))
}

/// Checks a built, signed directory the way the app would: the signature
/// and every entry of `index.json`, then each listed bundle under
/// `bundles/` against its entry. Returns the index.
pub fn verify_dir(dir: &Path, keys: &[VerifyingKey]) -> Result<Index, DirectoryError> {
    let index_path = dir.join(INDEX_FILE);
    let sig_path = dir.join(format!("{INDEX_FILE}.sig"));
    let bytes = std::fs::read(&index_path).map_err(|e| io(&index_path, e))?;
    let sig = std::fs::read(&sig_path).map_err(|e| io(&sig_path, e))?;
    let index = check_index(&bytes, &sig, keys, None)?;
    for entry in &index.plugins {
        let path = bundle_path(dir, &entry.id, &entry.version);
        let bytes = std::fs::read(&path).map_err(|e| io(&path, e))?;
        verify_bundle(&bytes, entry).map_err(|source| DirectoryError::Listed {
            id: entry.id.clone(),
            version: entry.version.clone(),
            source,
        })?;
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{pack, publisher_key_text, sign_hash};
    use ed25519_dalek::SigningKey;
    use kanso_update::feed::sign_feed;
    use proptest::prelude::*;

    const NOW: &str = "2026-10-20T10:00:00Z";
    const LATER: &str = "2026-10-21T10:00:00Z";

    fn key(seed: [u8; 32]) -> SigningKey {
        SigningKey::from_bytes(&seed)
    }

    fn id_of(n: u8) -> String {
        format!("com.example.p{n}")
    }

    /// A signed candidate: `id` at `version`, `body` as its script.
    fn cand(id: &str, version: &str, seed: [u8; 32], body: &[u8]) -> Candidate {
        let k = key(seed);
        let manifest = format!(
            r#"{{"schema":1,"apiVersion":1,"id":"{id}","name":"Test","version":"{version}",
"entry":"main.js","publisher":{{"name":"Tester","key":"{}"}},"minAppVersion":"0.1.0",
"permissions":{{"commands":true}}}}"#,
            publisher_key_text(&k.verifying_key())
        );
        let bytes = pack(&[
            ("manifest.json".to_owned(), manifest.into_bytes()),
            ("main.js".to_owned(), body.to_vec()),
        ]);
        let sig = sign_hash(&bundle::sha256_hex(&bytes), &k);
        Candidate {
            folder_name: id.to_owned(),
            bundle_bytes: bytes,
            author_signature: sig,
        }
    }

    fn rev(id: &str, versions: &str, reason: &str) -> Revocation {
        Revocation {
            id: id.to_owned(),
            versions: versions.to_owned(),
            reason: reason.to_owned(),
        }
    }

    fn seed(n: u8) -> [u8; 32] {
        [n; 32]
    }

    fn first(candidates: Vec<Candidate>) -> Built {
        build(None, candidates, vec![], NOW).unwrap()
    }

    // --- examples ---------------------------------------------------------

    #[test]
    fn a_first_build_starts_at_serial_one() {
        let built = first(vec![cand(&id_of(1), "1.0.0", seed(1), b"a")]);
        assert!(built.changed);
        assert_eq!(built.index.serial, 1);
        assert_eq!(built.index.schema, INDEX_SCHEMA);
        assert_eq!(built.index.plugins.len(), 1);
        assert_eq!(built.new_bundles.len(), 1);
        assert!(!built.index.plugins[0].revoked);
        assert_eq!(built.index.plugins[0].published_at, NOW);
    }

    #[test]
    fn an_empty_first_build_is_a_valid_empty_directory() {
        let built = first(vec![]);
        assert_eq!(built.index.serial, 1);
        assert!(built.index.plugins.is_empty());
    }

    #[test]
    fn rebuilding_with_the_same_inputs_changes_nothing() {
        let c = cand(&id_of(1), "1.0.0", seed(1), b"a");
        let one = first(vec![c.clone()]);
        let two = build(Some(one.index.clone()), vec![c], vec![], LATER).unwrap();
        assert!(!two.changed);
        assert_eq!(two.index, one.index);
        assert!(two.new_bundles.is_empty());
    }

    #[test]
    fn a_new_version_is_added_beside_the_old_one() {
        let one = first(vec![cand(&id_of(1), "1.0.0", seed(1), b"a")]);
        let next = build(
            Some(one.index.clone()),
            vec![cand(&id_of(1), "1.1.0", seed(1), b"b")],
            vec![],
            LATER,
        )
        .unwrap();
        assert!(next.changed);
        assert_eq!(next.index.serial, 2);
        assert_eq!(next.index.plugins[0], one.index.plugins[0]);
        assert_eq!(next.index.plugins[1].version, "1.1.0");
        assert_eq!(next.index.plugins[1].published_at, LATER);
        assert_eq!(next.new_bundles.len(), 1);
    }

    #[test]
    fn versions_sort_by_semver_not_text() {
        let one = first(vec![cand(&id_of(1), "1.9.0", seed(1), b"a")]);
        let next = build(
            Some(one.index),
            vec![cand(&id_of(1), "1.10.0", seed(1), b"b")],
            vec![],
            LATER,
        )
        .unwrap();
        let versions: Vec<_> = next.index.plugins.iter().map(|e| &*e.version).collect();
        assert_eq!(versions, ["1.9.0", "1.10.0"]);
    }

    #[test]
    fn other_contents_at_a_listed_version_fail() {
        let one = first(vec![cand(&id_of(1), "1.0.0", seed(1), b"a")]);
        let err = build(
            Some(one.index),
            vec![cand(&id_of(1), "1.0.0", seed(1), b"changed")],
            vec![],
            LATER,
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "com.example.p1 1.0.0 is already published with other contents: raise its version"
        );
    }

    #[test]
    fn a_lower_version_fails() {
        let one = first(vec![cand(&id_of(1), "1.2.0", seed(1), b"a")]);
        let err = build(
            Some(one.index),
            vec![cand(&id_of(1), "1.1.0", seed(1), b"b")],
            vec![],
            LATER,
        )
        .unwrap_err();
        assert!(matches!(err, DirectoryError::NotNewer { .. }), "{err}");
    }

    #[test]
    fn a_key_change_fails() {
        let one = first(vec![cand(&id_of(1), "1.0.0", seed(1), b"a")]);
        let err = build(
            Some(one.index),
            vec![cand(&id_of(1), "2.0.0", seed(2), b"b")],
            vec![],
            LATER,
        )
        .unwrap_err();
        assert!(matches!(err, DirectoryError::KeyChanged { .. }), "{err}");
    }

    #[test]
    fn a_bad_signature_names_the_plugin() {
        let mut c = cand(&id_of(1), "1.0.0", seed(1), b"a");
        c.author_signature = cand(&id_of(1), "1.0.0", seed(2), b"a").author_signature;
        let err = build(None, vec![c], vec![], NOW).unwrap_err();
        assert!(err.to_string().starts_with("com.example.p1:"), "{err}");
    }

    #[test]
    fn a_folder_named_otherwise_fails() {
        let mut c = cand(&id_of(1), "1.0.0", seed(1), b"a");
        c.folder_name = "com.example.other".to_owned();
        let err = build(None, vec![c], vec![], NOW).unwrap_err();
        assert!(matches!(err, DirectoryError::FolderName { .. }), "{err}");
    }

    #[test]
    fn a_broken_bundle_fails() {
        let c = Candidate {
            folder_name: id_of(1),
            bundle_bytes: b"not a zip".to_vec(),
            author_signature: String::new(),
        };
        assert!(matches!(
            build(None, vec![c], vec![], NOW),
            Err(DirectoryError::Bundle { .. })
        ));
    }

    #[test]
    fn the_same_folder_twice_fails() {
        let c = cand(&id_of(1), "1.0.0", seed(1), b"a");
        let err = build(None, vec![c.clone(), c], vec![], NOW).unwrap_err();
        assert!(matches!(err, DirectoryError::Duplicate { .. }), "{err}");
    }

    #[test]
    fn revocations_are_validated_and_kept_as_given() {
        let list = vec![
            rev(&id_of(1), "<1.0.0", "Stole notes."),
            rev(&id_of(2), "*", "x"),
        ];
        let built = build(None, vec![], list.clone(), NOW).unwrap();
        assert_eq!(built.index.revoked, list);
        assert!(build(None, vec![], vec![rev(&id_of(1), "<1.0.0", "")], NOW).is_err());
        assert!(build(None, vec![], vec![rev(&id_of(1), "oops", "r")], NOW).is_err());
        assert!(build(None, vec![], vec![rev("bad", "*", "r")], NOW).is_err());
    }

    #[test]
    fn a_revocation_change_raises_the_serial_and_entries_stay_unflagged() {
        let one = first(vec![cand(&id_of(1), "1.0.0", seed(1), b"a")]);
        let list = vec![rev(&id_of(1), "*", "Withdrawn.")];
        let two = build(Some(one.index.clone()), vec![], list.clone(), LATER).unwrap();
        assert!(two.changed);
        assert_eq!(two.index.serial, 2);
        assert_eq!(two.index.plugins, one.index.plugins);
        // The app reads the withdrawal from the list.
        assert!(
            crate::registry::revocation_reason(
                Some(&two.index),
                &BTreeSet::new(),
                &id_of(1),
                "1.0.0"
            )
            .is_some()
        );
        // The same list again changes nothing.
        let three = build(Some(two.index.clone()), vec![], list, LATER).unwrap();
        assert!(!three.changed);
        assert_eq!(three.index, two.index);
    }

    // --- properties -------------------------------------------------------

    fn seed_strategy() -> impl Strategy<Value = [u8; 32]> {
        prop::array::uniform32(any::<u8>())
    }

    fn body_strategy() -> impl Strategy<Value = Vec<u8>> {
        prop::collection::vec(any::<u8>(), 0..24)
    }

    /// A plugin number, a version triple and its bytes.
    type Spec = (u8, (u8, u8, u8), Vec<u8>);

    fn spec_strategy() -> impl Strategy<Value = Spec> {
        (0u8..4, (0u8..4, 0u8..4, 0u8..4), body_strategy())
    }

    fn version_text(v: (u8, u8, u8)) -> String {
        format!("{}.{}.{}", v.0, v.1, v.2)
    }

    /// Builds candidates from specs, one key per plugin number, dropping
    /// repeats of the same plugin and version.
    fn candidates(specs: &[Spec], seeds: &[[u8; 32]]) -> Vec<Candidate> {
        // One candidate per plugin: the first spec for each number.
        let mut seen = BTreeSet::new();
        specs
            .iter()
            .filter(|s| seen.insert(s.0))
            .map(|(n, v, body)| {
                cand(
                    &id_of(*n),
                    &version_text(*v),
                    seeds[*n as usize % seeds.len()],
                    body,
                )
            })
            .collect()
    }

    proptest! {
        /// Invariant: the same inputs always give the same index.
        #[test]
        fn prop_deterministic(
            specs in prop::collection::vec(spec_strategy(), 0..5),
            seeds in prop::collection::vec(seed_strategy(), 4),
        ) {
            let c = candidates(&specs, &seeds);
            let mut shuffled = c.clone();
            shuffled.reverse();
            let a = build(None, c.clone(), vec![], NOW);
            let b = build(None, c, vec![], NOW);
            let r = build(None, shuffled, vec![], NOW);
            prop_assert_eq!(a.as_ref().ok(), b.as_ref().ok());
            prop_assert_eq!(a.as_ref().ok(), r.as_ref().ok());
        }

        /// Invariant: previous entries are never altered or dropped, the
        /// serial grows exactly when something changed, and the output
        /// always passes the app's checks once signed.
        #[test]
        fn prop_next_build_keeps_the_past(
            specs in prop::collection::vec(spec_strategy(), 0..5),
            more in prop::collection::vec(spec_strategy(), 0..5),
            seeds in prop::collection::vec(seed_strategy(), 4),
            registry_seed in seed_strategy(),
            revoke in prop::option::of(0u8..4),
        ) {
            let one = build(None, candidates(&specs, &seeds), vec![], NOW).unwrap();
            let list: Vec<Revocation> =
                revoke.iter().map(|n| rev(&id_of(*n), "<1.0.0", "Old.")).collect();
            if let Ok(next) = build(
                Some(one.index.clone()),
                candidates(&more, &seeds),
                list.clone(),
                LATER,
            ) {
                for old in &one.index.plugins {
                    prop_assert!(next.index.plugins.contains(old));
                }
                prop_assert_eq!(next.index.revoked.clone(), list.clone());
                prop_assert_eq!(next.changed, next.index.serial > one.index.serial);
                prop_assert_eq!(
                    next.changed,
                    !next.new_bundles.is_empty() || !list.is_empty()
                );
                prop_assert!(
                    next.index.serial == one.index.serial || next.index.serial == one.index.serial + 1
                );
                prop_assert!(validate_index(&next.index).is_ok());
                let sorted = next.index.plugins.windows(2).all(|w| {
                    (&w[0].id, parsed(&w[0])) < (&w[1].id, parsed(&w[1]))
                });
                prop_assert!(sorted);
                // Signed with a test key, the app's own check accepts it.
                let k = key(registry_seed);
                let bytes = serde_json::to_vec_pretty(&next.index).unwrap();
                let sig = sign_feed(&bytes, &k);
                let checked =
                    check_index(&bytes, sig.as_bytes(), &[k.verifying_key()], Some(one.index.serial));
                prop_assert_eq!(checked.ok(), Some(next.index.clone()));
            }
        }

        /// Invariant: republishing identical inputs changes nothing.
        #[test]
        fn prop_republishing_is_unchanged(
            specs in prop::collection::vec(spec_strategy(), 0..5),
            seeds in prop::collection::vec(seed_strategy(), 4),
            with_revocation in any::<bool>(),
        ) {
            let c = candidates(&specs, &seeds);
            let list: Vec<Revocation> = if with_revocation {
                vec![rev(&id_of(0), "*", "Gone.")]
            } else {
                vec![]
            };
            let one = build(None, c.clone(), list.clone(), NOW).unwrap();
            let two = build(Some(one.index.clone()), c, list, LATER).unwrap();
            prop_assert!(!two.changed);
            prop_assert!(two.new_bundles.is_empty());
            prop_assert_eq!(two.index, one.index);
        }

        /// Invariant: other bytes at a listed version always fail.
        #[test]
        fn prop_same_version_other_bytes_fails(
            v in (0u8..4, 0u8..4, 0u8..4),
            a in body_strategy(),
            b in body_strategy(),
            s in seed_strategy(),
        ) {
            prop_assume!(a != b);
            let one = build(None, vec![cand(&id_of(0), &version_text(v), s, &a)], vec![], NOW).unwrap();
            let err = build(
                Some(one.index),
                vec![cand(&id_of(0), &version_text(v), s, &b)],
                vec![],
                LATER,
            );
            let is_republished = matches!(err, Err(DirectoryError::Republished { .. }));
            prop_assert!(is_republished);
        }

        /// Invariant: a version below any listed one fails.
        #[test]
        fn prop_lower_version_fails(
            listed in (0u8..4, 0u8..4, 0u8..4),
            new in (0u8..4, 0u8..4, 0u8..4),
            s in seed_strategy(),
        ) {
            prop_assume!(new < listed);
            let one = build(None, vec![cand(&id_of(0), &version_text(listed), s, b"a")], vec![], NOW).unwrap();
            let err = build(
                Some(one.index),
                vec![cand(&id_of(0), &version_text(new), s, b"b")],
                vec![],
                LATER,
            );
            let is_not_newer = matches!(err, Err(DirectoryError::NotNewer { .. }));
            prop_assert!(is_not_newer);
        }

        /// Invariant: a new key for a listed plugin always fails, however
        /// high the version.
        #[test]
        fn prop_key_change_fails(s1 in seed_strategy(), s2 in seed_strategy()) {
            prop_assume!(key(s1).verifying_key() != key(s2).verifying_key());
            let one = build(None, vec![cand(&id_of(0), "1.0.0", s1, b"a")], vec![], NOW).unwrap();
            let err = build(
                Some(one.index),
                vec![cand(&id_of(0), "9.0.0", s2, b"a")],
                vec![],
                LATER,
            );
            let is_key_changed = matches!(err, Err(DirectoryError::KeyChanged { .. }));
            prop_assert!(is_key_changed);
        }

        /// Invariant: revocations come out exactly as given.
        #[test]
        fn prop_revocations_exactly_as_given(
            list in prop::collection::vec(
                (0u8..4, prop::sample::select(vec!["*", "<1.0.0", "=1.2.3", ">=2.0.0, <3.0.0"]), "[A-Za-z][A-Za-z ]{0,40}[A-Za-z]"),
                0..5,
            ),
        ) {
            let list: Vec<Revocation> =
                list.iter().map(|(n, v, r)| rev(&id_of(*n), v, r)).collect();
            let built = build(None, vec![], list.clone(), NOW).unwrap();
            prop_assert_eq!(built.index.revoked, list);
        }
    }

    // --- disk -------------------------------------------------------------

    #[test]
    fn write_then_verify_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let built = first(vec![
            cand(&id_of(1), "1.0.0", seed(1), b"a"),
            cand(&id_of(2), "2.0.0", seed(2), b"b"),
        ]);
        write_built(dir.path(), &built).unwrap();
        let k = key(seed(90));
        let bytes = std::fs::read(dir.path().join(INDEX_FILE)).unwrap();
        assert_eq!(bytes.last(), Some(&b'\n'));
        std::fs::write(dir.path().join("index.json.sig"), sign_feed(&bytes, &k)).unwrap();
        let index = verify_dir(dir.path(), &[k.verifying_key()]).unwrap();
        assert_eq!(index, built.index);
        // A bundle with other bytes is caught, and is never overwritten.
        let path = bundle_path(dir.path(), &id_of(1), "1.0.0");
        std::fs::write(&path, b"tampered").unwrap();
        assert!(verify_dir(dir.path(), &[k.verifying_key()]).is_err());
        assert!(write_built(dir.path(), &built).is_err());
    }
}
