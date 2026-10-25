//! Plugin bundles: the `.kansoplugin` zip, opened with hard caps, packed the
//! same way every time, and the author's signature over its hash.
//!
//! A bundle comes from a stranger, so nothing a zip says about itself is
//! believed. The archive's size is capped before it is read; its entry count
//! before any entry is looked at; every path must pass
//! [`is_valid_bundle_path`](crate::manifest::is_valid_bundle_path); links and
//! encrypted entries are refused; and every file is read through a limit that
//! shrinks as the total grows, so a small archive can't unpack into a large
//! one whatever its headers claim. Only then is `manifest.json` read and the
//! file list held to the manifest's rules.
//!
//! [`pack`] is deterministic (sorted entries, a fixed timestamp, no extra
//! fields), so the registry's CI and an author rebuild the same bytes from the
//! same files, and a hash means one thing. The author signs that hash with
//! their own Ed25519 key; the app pins the key at first install.

use std::collections::BTreeSet;
use std::io::{Cursor, Read as _, Write as _};
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::manifest::{self, MAX_BUNDLE_BYTES, MAX_ENTRIES, ManifestError, is_valid_bundle_path};
use crate::registry_model::Bundle;

/// The most bytes a `.kansoplugin` file may be: the files' cap plus room for
/// the zip's own headers.
pub const MAX_ARCHIVE_BYTES: u64 = MAX_BUNDLE_BYTES + 1024 * 1024;

const MANIFEST_PATH: &str = "manifest.json";
/// How deep a developer folder may nest.
const MAX_DEPTH: usize = 16;
/// Unix file type bits in a zip entry's external attributes.
const S_IFMT: u32 = 0o170000;
const S_IFREG: u32 = 0o100000;
const S_IFDIR: u32 = 0o040000;

/// Why a bundle, a folder or a signature was refused.
#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("the bundle is larger than it may be")]
    TooLarge,
    #[error("the bundle has more than {MAX_ENTRIES} entries")]
    TooManyEntries,
    #[error("the bundle isn't a readable zip: {0}")]
    BadZip(String),
    #[error("{0:?} isn't a path a bundle may hold")]
    BadPath(String),
    #[error("{0:?} is a link, which a bundle may not hold")]
    Link(String),
    #[error("{0:?} is encrypted")]
    Encrypted(String),
    #[error("{0:?} appears twice")]
    Duplicate(String),
    #[error("the bundle has no manifest.json")]
    NoManifest,
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("the author's signature doesn't match the bundle")]
    BadSignature,
    #[error("the author's key isn't a valid ed25519 key")]
    BadKey,
    #[error("the folder can't be read: {0}")]
    Io(String),
}

/// The SHA-256 of `bytes`, as lowercase hex. Pure.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

/// Opens a `.kansoplugin`: reads it under the caps, checks every path and
/// the manifest, and returns the manifest, the files and the bundle's hash.
pub fn open(bytes: &[u8]) -> Result<Bundle, BundleError> {
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(BundleError::TooLarge);
    }
    let files = read_zip(bytes)?;
    finish(files, sha256_hex(bytes))
}

/// Opens an unpacked folder for developer mode, under the same caps and
/// checks. The hash is that of the folder's [`pack`], so it matches what
/// packing it for the directory would give. Dotfiles are skipped; a link
/// anywhere is refused.
pub fn open_dir(dir: &Path) -> Result<Bundle, BundleError> {
    let files = read_dir_files(dir)?;
    let hash = sha256_hex(&pack(&files));
    finish(files, hash)
}

/// The deterministic zip of a folder, after the same checks as
/// [`open_dir`].
pub fn pack_dir(dir: &Path) -> Result<Vec<u8>, BundleError> {
    let files = read_dir_files(dir)?;
    finish(files.clone(), String::new())?;
    Ok(pack(&files))
}

/// A deterministic zip: entries sorted by path, deflate, the timestamp
/// 1980-01-01 00:00, permissions 0644, no extra fields and no comments. The
/// same files in any order give the same bytes. A path too long for a zip
/// gives an empty archive, which [`open`] refuses. Pure.
pub fn pack(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, data) in sorted {
        if writer.start_file(path.as_str(), options).is_err() || writer.write_all(data).is_err() {
            return Vec::new();
        }
    }
    writer.finish().map(Cursor::into_inner).unwrap_or_default()
}

/// The base64 Ed25519 signature over the 32 raw bytes of `sha256_hex`. A
/// string that isn't a SHA-256 gives an empty signature, which never
/// verifies.
pub fn sign_hash(sha256_hex: &str, key: &SigningKey) -> String {
    match hash_bytes(sha256_hex) {
        Some(hash) => B64.encode(key.sign(&hash).to_bytes()),
        None => String::new(),
    }
}

/// Checks an author's signature over a bundle's hash against their
/// `ed25519:<hex>` key.
pub fn verify_hash(
    sha256_hex: &str,
    signature_b64: &str,
    publisher_key: &str,
) -> Result<(), BundleError> {
    let key = parse_publisher_key(publisher_key)?;
    let hash = hash_bytes(sha256_hex).ok_or(BundleError::BadSignature)?;
    let signature = B64
        .decode(signature_b64.trim())
        .map_err(|_| BundleError::BadSignature)?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| BundleError::BadSignature)?;
    key.verify_strict(&hash, &Signature::from_bytes(&signature))
        .map_err(|_| BundleError::BadSignature)
}

/// `ed25519:` and the key's lowercase hex, as a manifest names it. Pure.
pub fn publisher_key_text(key: &VerifyingKey) -> String {
    format!("ed25519:{}", hex_encode(&key.to_bytes()))
}

/// The key in `ed25519:<64 lowercase hex>`.
pub fn parse_publisher_key(text: &str) -> Result<VerifyingKey, BundleError> {
    let hex = text.strip_prefix("ed25519:").ok_or(BundleError::BadKey)?;
    let bytes = hex_decode(hex).ok_or(BundleError::BadKey)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| BundleError::BadKey)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| BundleError::BadKey)
}

fn hash_bytes(hex: &str) -> Option<[u8; 32]> {
    hex_decode(hex)?.try_into().ok()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Lowercase hex only; anything else is `None`.
fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let digit = |b: u8| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    };
    text.as_bytes()
        .chunks(2)
        .map(|pair| Some(digit(pair[0])? << 4 | digit(pair[1])?))
        .collect()
}

/// Parses the manifest and holds the file list to it; returns the bundle
/// with `manifest.json` set aside.
fn finish(files: Vec<(String, Vec<u8>)>, sha256: String) -> Result<Bundle, BundleError> {
    let manifest_bytes = files
        .iter()
        .find(|(path, _)| path == MANIFEST_PATH)
        .map(|(_, data)| data.as_slice())
        .ok_or(BundleError::NoManifest)?;
    let parsed = manifest::parse(manifest_bytes)?;
    let sizes: Vec<(String, u64)> = files
        .iter()
        .map(|(path, data)| (path.clone(), data.len() as u64))
        .collect();
    manifest::check_files(&parsed, &sizes)?;
    let mut rest: Vec<(String, Vec<u8>)> = files
        .into_iter()
        .filter(|(path, _)| path != MANIFEST_PATH)
        .collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(Bundle {
        manifest: parsed,
        files: rest,
        sha256,
    })
}

/// Every file of a zip, read under the caps. Declared sizes are never
/// trusted: each file is read through a limit of what is left of the total.
fn read_zip(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, BundleError> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| BundleError::BadZip(e.to_string()))?;
    if archive.len() > MAX_ENTRIES {
        return Err(BundleError::TooManyEntries);
    }
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    let mut total: u64 = 0;
    for index in 0..archive.len() {
        {
            let entry = archive
                .by_index_raw(index)
                .map_err(|e| BundleError::BadZip(e.to_string()))?;
            let name = entry.name().to_owned();
            if entry.encrypted() {
                return Err(BundleError::Encrypted(name));
            }
            if let Some(mode) = entry.unix_mode() {
                let kind = mode & S_IFMT;
                let allowed = if entry.is_dir() { S_IFDIR } else { S_IFREG };
                if kind != 0 && kind != allowed {
                    return Err(BundleError::Link(name));
                }
            }
            if entry.is_dir() {
                if !is_valid_bundle_path(name.trim_end_matches('/')) {
                    return Err(BundleError::BadPath(name));
                }
                continue;
            }
            if !is_valid_bundle_path(&name) {
                return Err(BundleError::BadPath(name));
            }
            if !seen.insert(name.clone()) {
                return Err(BundleError::Duplicate(name));
            }
        }
        let entry = archive
            .by_index(index)
            .map_err(|e| BundleError::BadZip(e.to_string()))?;
        let name = entry.name().to_owned();
        let left = MAX_BUNDLE_BYTES - total;
        let mut data = Vec::new();
        entry
            .take(left + 1)
            .read_to_end(&mut data)
            .map_err(|e| BundleError::BadZip(e.to_string()))?;
        if data.len() as u64 > left {
            return Err(BundleError::TooLarge);
        }
        total += data.len() as u64;
        files.push((name, data));
    }
    Ok(files)
}

/// Every file under `dir` as `(relative path, bytes)`, skipping dotfiles and
/// refusing links and anything that isn't a plain file or folder.
fn read_dir_files(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, BundleError> {
    let root = std::fs::symlink_metadata(dir).map_err(|e| BundleError::Io(e.to_string()))?;
    if !root.is_dir() {
        return Err(BundleError::Io(format!("{} isn't a folder", dir.display())));
    }
    let mut files = Vec::new();
    let mut total: u64 = 0;
    walk(dir, "", 0, &mut files, &mut total)?;
    Ok(files)
}

fn walk(
    dir: &Path,
    prefix: &str,
    depth: usize,
    files: &mut Vec<(String, Vec<u8>)>,
    total: &mut u64,
) -> Result<(), BundleError> {
    if depth > MAX_DEPTH {
        return Err(BundleError::BadPath(prefix.to_owned()));
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| BundleError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| BundleError::Io(e.to_string()))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|n| BundleError::BadPath(n.to_string_lossy().into_owned()))?;
        names.push(name);
    }
    names.sort();
    for name in names {
        if name.starts_with('.') {
            continue;
        }
        let path = dir.join(&name);
        let relative = format!("{prefix}{name}");
        let meta = std::fs::symlink_metadata(&path).map_err(|e| BundleError::Io(e.to_string()))?;
        let kind = meta.file_type();
        if kind.is_symlink() {
            return Err(BundleError::Link(relative));
        } else if kind.is_dir() {
            walk(&path, &format!("{relative}/"), depth + 1, files, total)?;
        } else if kind.is_file() {
            if !is_valid_bundle_path(&relative) {
                return Err(BundleError::BadPath(relative));
            }
            if files.len() >= MAX_ENTRIES {
                return Err(BundleError::TooManyEntries);
            }
            let left = MAX_BUNDLE_BYTES - *total;
            let mut data = Vec::new();
            std::fs::File::open(&path)
                .and_then(|file| file.take(left + 1).read_to_end(&mut data))
                .map_err(|e| BundleError::Io(e.to_string()))?;
            if data.len() as u64 > left {
                return Err(BundleError::TooLarge);
            }
            *total += data.len() as u64;
            files.push((relative, data));
        } else {
            return Err(BundleError::BadPath(relative));
        }
    }
    Ok(())
}

/// Bundles and keys the tests here and in [`crate::registry`] share.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    /// A valid `manifest.json` for `id` at `version`, signed-by `seed`'s key.
    pub fn manifest_json(id: &str, version: &str, seed: u8) -> String {
        format!(
            r#"{{"schema":1,"apiVersion":1,"id":"{id}","name":"Test","version":"{version}",
"entry":"main.js","publisher":{{"name":"Tester","key":"{}"}},"minAppVersion":"0.1.0",
"permissions":{{"commands":true}}}}"#,
            publisher_key_text(&key(seed).verifying_key())
        )
    }

    /// The files of a small valid bundle.
    pub fn files(id: &str, version: &str, seed: u8) -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "manifest.json".to_owned(),
                manifest_json(id, version, seed).into_bytes(),
            ),
            ("main.js".to_owned(), b"kanso.commands.register();".to_vec()),
        ]
    }

    /// A signed-ready bundle for `id`.
    pub fn bundle_bytes(id: &str, version: &str, seed: u8) -> Vec<u8> {
        pack(&files(id, version, seed))
    }

    /// A raw zip with the given entries, for building hostile archives.
    pub fn raw_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, data) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use proptest::prelude::*;

    const ID: &str = "com.example.words";

    fn good() -> Vec<(String, Vec<u8>)> {
        files(ID, "1.0.0", 1)
    }

    fn with_extra(path: &str, data: &[u8]) -> Vec<u8> {
        let manifest = manifest_json(ID, "1.0.0", 1);
        raw_zip(&[
            ("manifest.json", manifest.as_bytes()),
            ("main.js", b"x"),
            (path, data),
        ])
    }

    #[test]
    fn a_good_bundle_opens_and_hashes_its_bytes() {
        let bytes = pack(&good());
        let bundle = open(&bytes).unwrap();
        assert_eq!(bundle.manifest.id, ID);
        assert_eq!(bundle.sha256, sha256_hex(&bytes));
        assert_eq!(bundle.files.len(), 1);
        assert_eq!(bundle.files[0].0, "main.js");
    }

    #[test]
    fn pack_is_deterministic_with_a_fixed_timestamp() {
        let bytes = pack(&good());
        assert_eq!(bytes, pack(&good()));
        let mut archive = zip::ZipArchive::new(Cursor::new(&bytes[..])).unwrap();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            let t = entry.last_modified().unwrap();
            assert_eq!((t.year(), t.month(), t.day(), t.hour()), (1980, 1, 1, 0));
            assert_eq!(entry.unix_mode(), Some(0o100644));
            assert!(entry.extra_data().is_none_or(<[u8]>::is_empty));
        }
    }

    #[test]
    fn a_missing_manifest_is_refused() {
        let bytes = raw_zip(&[("main.js", b"x")]);
        assert!(matches!(open(&bytes), Err(BundleError::NoManifest)));
    }

    #[test]
    fn not_a_zip_is_refused() {
        assert!(open(b"hello").is_err());
        assert!(open(&[]).is_err());
    }

    #[test]
    fn a_symlink_entry_is_refused() {
        let manifest = manifest_json(ID, "1.0.0", 1);
        let options = zip::write::SimpleFileOptions::default();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(manifest.as_bytes()).unwrap();
        writer.start_file("main.js", options).unwrap();
        writer.write_all(b"x").unwrap();
        writer.add_symlink("link.js", "main.js", options).unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        assert!(matches!(open(&bytes), Err(BundleError::Link(_))));
    }

    #[test]
    fn more_than_the_entry_cap_is_refused() {
        let manifest = manifest_json(ID, "1.0.0", 1);
        let names: Vec<String> = (0..MAX_ENTRIES).map(|i| format!("f{i}.txt")).collect();
        let mut entries: Vec<(&str, &[u8])> =
            vec![("manifest.json", manifest.as_bytes()), ("main.js", b"x")];
        entries.extend(names.iter().map(|n| (n.as_str(), &b"y"[..])));
        assert!(matches!(
            open(&raw_zip(&entries)),
            Err(BundleError::TooManyEntries)
        ));
    }

    #[test]
    fn a_zip_bomb_is_refused_however_small_it_is() {
        // 20 MiB of zeros deflates to about 20 KiB.
        let bomb = vec![0u8; 20 * 1024 * 1024];
        let bytes = raw_zip(&[
            ("manifest.json", manifest_json(ID, "1.0.0", 1).as_bytes()),
            ("main.js", &bomb),
        ]);
        assert!(
            (bytes.len() as u64) < MAX_ARCHIVE_BYTES,
            "small enough to be read"
        );
        assert!(matches!(open(&bytes), Err(BundleError::TooLarge)));
    }

    #[test]
    fn a_bomb_whose_headers_lie_is_refused_too() {
        let bomb = vec![0u8; 20 * 1024 * 1024];
        let mut bytes = raw_zip(&[
            ("manifest.json", manifest_json(ID, "1.0.0", 1).as_bytes()),
            ("main.js", &bomb),
        ]);
        // Rewrite every central directory entry's uncompressed size to 10.
        let mut at = 0;
        while at + 28 < bytes.len() {
            if bytes[at..at + 4] == [0x50, 0x4b, 0x01, 0x02] {
                bytes[at + 24..at + 28].copy_from_slice(&10u32.to_le_bytes());
            }
            at += 1;
        }
        assert!(open(&bytes).is_err());
    }

    #[test]
    fn an_oversized_archive_is_refused_unread() {
        let big = vec![0u8; MAX_ARCHIVE_BYTES as usize + 1];
        assert!(matches!(open(&big), Err(BundleError::TooLarge)));
    }

    #[test]
    fn a_folder_opens_and_matches_its_pack() {
        let dir = tempfile::tempdir().unwrap();
        for (path, data) in good() {
            std::fs::write(dir.path().join(path), data).unwrap();
        }
        std::fs::write(dir.path().join(".DS_Store"), b"junk").unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git/config"), b"junk").unwrap();
        let bundle = open_dir(dir.path()).unwrap();
        assert_eq!(bundle.sha256, sha256_hex(&pack(&good())));
        assert_eq!(pack_dir(dir.path()).unwrap(), pack(&good()));
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_with_a_link_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        for (path, data) in good() {
            std::fs::write(dir.path().join(path), data).unwrap();
        }
        std::os::unix::fs::symlink("/etc/hosts", dir.path().join("hosts.txt")).unwrap();
        assert!(matches!(open_dir(dir.path()), Err(BundleError::Link(_))));
    }

    #[test]
    fn a_folder_missing_its_manifest_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.js"), b"x").unwrap();
        assert!(open_dir(dir.path()).is_err());
    }

    #[test]
    fn author_signatures_round_trip() {
        let key = key(7);
        let hash = sha256_hex(b"bundle");
        let sig = sign_hash(&hash, &key);
        let text = publisher_key_text(&key.verifying_key());
        assert!(verify_hash(&hash, &sig, &text).is_ok());
        assert!(verify_hash(&sha256_hex(b"other"), &sig, &text).is_err());
        let other = publisher_key_text(&super::test_support::key(8).verifying_key());
        assert!(verify_hash(&hash, &sig, &other).is_err());
        assert!(verify_hash(&hash, "", &text).is_err());
        assert!(verify_hash(&hash, &sig, "ed25519:zz").is_err());
        assert!(verify_hash(&hash, &sig, &text.to_uppercase()).is_err());
        assert_eq!(sign_hash("nothex", &key), "");
    }

    fn file_set() -> impl Strategy<Value = Vec<(String, Vec<u8>)>> {
        prop::collection::btree_map(
            "[a-z]{1,6}(/[a-z]{1,6})?\\.(txt|md|json)",
            prop::collection::vec(any::<u8>(), 0..64),
            0..8,
        )
        .prop_map(|m| {
            // A file can't also be the folder of another.
            m.into_iter()
                .filter(|(p, _)| p != "manifest.json")
                .collect::<Vec<_>>()
        })
    }

    proptest! {
        /// Invariant: the same files in any order pack to the same bytes, and
        /// opening the pack gives the files back.
        #[test]
        fn prop_pack_is_order_independent_and_lossless(
            extra in file_set(),
            seed in any::<u64>(),
        ) {
            let mut all = good();
            all.extend(extra.into_iter().filter(|(p, _)| p != "main.js"));
            let mut shuffled = all.clone();
            let mut state = seed;
            for i in (1..shuffled.len()).rev() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                shuffled.swap(i, (state >> 33) as usize % (i + 1));
            }
            let bytes = pack(&all);
            prop_assert_eq!(&bytes, &pack(&shuffled));
            let bundle = open(&bytes).unwrap();
            let mut want: Vec<_> = all.into_iter().filter(|(p, _)| p != "manifest.json").collect();
            want.sort();
            prop_assert_eq!(bundle.files, want);
        }

        /// Invariant: an archive with an entry whose path climbs out, is
        /// absolute or uses a backslash is never opened.
        #[test]
        fn prop_hostile_paths_are_refused(
            head in prop::sample::select(vec!["../", "/", "a/../", "..\\", "a\\", "./", "//"]),
            tail in "[a-z]{1,8}\\.js",
        ) {
            let path = format!("{head}{tail}");
            prop_assert!(open(&with_extra(&path, b"x")).is_err(), "{}", path);
        }

        /// Invariant: no byte string that isn't a valid archive opens, and
        /// opening never panics.
        #[test]
        fn prop_garbage_never_opens(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
            prop_assert!(open(&bytes).is_err());
        }

        /// Invariant: a signature verifies only for the hash and key it was
        /// made for.
        #[test]
        fn prop_signatures_bind_hash_and_key(
            a in prop::collection::vec(any::<u8>(), 0..32),
            b in prop::collection::vec(any::<u8>(), 0..32),
            seed in 1u8..200,
        ) {
            let key = super::test_support::key(seed);
            let text = publisher_key_text(&key.verifying_key());
            let ha = sha256_hex(&a);
            let sig = sign_hash(&ha, &key);
            prop_assert!(verify_hash(&ha, &sig, &text).is_ok());
            if a != b {
                prop_assert!(verify_hash(&sha256_hex(&b), &sig, &text).is_err());
            }
            let other = publisher_key_text(&super::test_support::key(seed + 1).verifying_key());
            prop_assert!(verify_hash(&ha, &sig, &other).is_err());
        }
    }
}
