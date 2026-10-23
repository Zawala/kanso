//! Installing a gallery theme below `<data>/gallery/`, apart from a person's
//! own `themes/` and `icon-themes/` folders.
//!
//! Every byte is checked against the signed index before it is parsed, and
//! then written through a staging folder and a rename, so a reader sees the
//! old file or the new one and never a half-written theme. A color theme is
//! one JSON file; an icon theme is one deterministic bundle, unpacked with
//! the updater's hardened extractor, checked against its manifest, held to
//! the icon theme caps, sanitized, and swapped in with the updater's
//! two-rename pattern.
//!
//! Nothing here fetches: the app has the bytes, checked, before calling.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read as _};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::index::{FileRef, Theme, sha256_hex};
use crate::sanitize::sanitize_svg;
use crate::{MAX_BUNDLE_BYTES, MAX_BUNDLE_ENTRIES, MAX_COLOR_BYTES, file_url};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("the download's size doesn't match the catalog")]
    SizeMismatch,
    #[error("the download's checksum doesn't match the catalog")]
    ChecksumMismatch,
    #[error("there isn't enough free space for the theme")]
    NoSpace,
    #[error("the icon theme's bundle isn't as expected: {0}")]
    BadBundle(String),
    #[error("the icon theme's folder isn't as expected: {0}")]
    BadFolder(String),
    #[error("couldn't write the theme: {0}")]
    Io(io::Error),
}

impl From<io::Error> for InstallError {
    fn from(err: io::Error) -> InstallError {
        if kanso_update::stage::is_no_space(&err) {
            InstallError::NoSpace
        } else {
            InstallError::Io(err)
        }
    }
}

/// The caps an icon theme is held to while installing. The app passes the
/// same values its loader uses, so installing and loading agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconCaps {
    pub max_json: u64,
    pub max_svg: u64,
    pub max_entries: usize,
    pub max_theme_bytes: u64,
}

impl Default for IconCaps {
    fn default() -> Self {
        IconCaps {
            max_json: 512 * 1024,
            max_svg: 128 * 1024,
            max_entries: 2_000,
            max_theme_bytes: 16 * 1024 * 1024,
        }
    }
}

/// The SHA-256 a color theme's bytes are recorded under. Pure.
pub fn color_digest(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// The URL a color or icon theme is fetched from, from the fixed repository
/// and the path the index validator checked. Pure.
pub fn theme_url(theme: &Theme) -> Option<String> {
    theme.payload().map(|file| file_url(&file.path))
}

/// Creates the gallery tree and empties `staging/`, which only ever holds
/// installs in progress.
pub fn prepare(root: &Path) -> io::Result<()> {
    for dir in ["themes", "icon-themes", "cache", "staging"] {
        fs::create_dir_all(root.join(dir))?;
    }
    kanso_update::stage::fresh_stage_dir(&root.join("staging"))
}

/// Installs `bytes` as the color theme `theme` describes. The bytes are
/// checked against the signed pin first; the file is written in `staging/`
/// and renamed over the old one, so a failure leaves the installed copy
/// alone. Returns the SHA-256 recorded in `state.json`.
pub fn install_color(root: &Path, theme: &Theme, bytes: &[u8]) -> Result<String, InstallError> {
    let file = theme
        .file
        .as_ref()
        .ok_or_else(|| InstallError::BadFolder("a color theme without a file".into()))?;
    check_pin(file, bytes, MAX_COLOR_BYTES)?;
    prepare(root)?;
    let digest = color_digest(bytes);
    let staged = root.join("staging").join(format!(
        "{}-{}-{}.json",
        theme.id,
        std::process::id(),
        nanos()
    ));
    write_file(&staged, bytes)?;
    let target = root.join("themes").join(format!("{}.json", theme.id));
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&staged, &target)?;
    Ok(digest)
}

/// Installs the icon bundle `bundle` as the icon theme `theme` describes,
/// returning the folder digest recorded in `state.json`. The old folder, if
/// any, is only replaced after the new one has fully checked out.
pub fn install_icons(
    root: &Path,
    theme: &Theme,
    bundle: &[u8],
    caps: IconCaps,
) -> Result<String, InstallError> {
    let pin = theme
        .bundle
        .as_ref()
        .ok_or_else(|| InstallError::BadBundle("an icon theme without a bundle".into()))?;
    check_pin(pin, bundle, MAX_BUNDLE_BYTES)?;
    prepare(root)?;
    let staging = root.join("staging");
    let archive = staging.join(format!(
        "{}-{}-{}.tar.gz",
        theme.id,
        std::process::id(),
        nanos()
    ));
    write_file(&archive, bundle)?;
    let unpacked = staging.join("unpacked");
    if let Err(err) = kanso_update::stage::extract_limited(
        &archive,
        kanso_update::ArchiveKind::TarGz,
        &unpacked,
        kanso_update::stage::ExtractLimits {
            max_bytes: MAX_BUNDLE_BYTES,
            max_entries: MAX_BUNDLE_ENTRIES,
        },
    ) {
        fs::remove_file(&archive).ok();
        kanso_update::stage::remove_entry(&unpacked).ok();
        return Err(match err {
            kanso_update::StageError::NoSpace => InstallError::NoSpace,
            other => InstallError::BadBundle(other.to_string()),
        });
    }
    fs::remove_file(&archive).ok();

    let result = (|| {
        check_manifest(&unpacked)?;
        let folder = unpacked.join(&theme.id);
        if !folder.is_dir() {
            return Err(InstallError::BadBundle(format!(
                "the bundle has no {}/ folder",
                theme.id
            )));
        }
        check_folder(&folder, caps)?;
        let installed = root.join("icon-themes").join(&theme.id);
        if installed.is_dir() {
            let backup = root.join("icon-themes").join(format!("{}.old", theme.id));
            kanso_update::stage::remove_entry(&backup)?;
            kanso_update::install::swap_in(&folder, &installed, &backup)
                .map_err(InstallError::Io)?;
            kanso_update::stage::remove_entry(&backup)?;
        } else {
            fs::rename(&folder, &installed)?;
        }
        kanso_update::stage::payload_digest(&installed).map_err(InstallError::Io)
    })();
    kanso_update::stage::remove_entry(&unpacked).ok();
    result
}

/// Removes an installed gallery theme, whatever is at its path. Only ever
/// called with a path under the gallery tree.
pub fn remove_installed(root: &Path, kind: crate::index::Kind, id: &str) -> io::Result<()> {
    let path = match kind {
        crate::index::Kind::Color => root.join("themes").join(format!("{id}.json")),
        crate::index::Kind::Icons => root.join("icon-themes").join(id),
    };
    kanso_update::stage::remove_entry(&path)
}

/// Checks `bytes` against a signed pin and the cap for its kind. Public so
/// the app can check a card's file before it is previewed or kept, rather
/// than only when it is installed.
pub fn check_pin(pin: &FileRef, bytes: &[u8], cap: u64) -> Result<(), InstallError> {
    if bytes.len() as u64 != pin.size || bytes.len() as u64 > cap {
        return Err(InstallError::SizeMismatch);
    }
    if sha256_hex(bytes) != pin.sha256 {
        return Err(InstallError::ChecksumMismatch);
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    use std::io::Write as _;
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}

/// `manifest.json` at the bundle's root, listing every other file's path,
/// SHA-256 and size.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    files: Vec<FilePin>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FilePin {
    path: String,
    sha256: String,
    size: u64,
}

/// Verifies the manifest against the unpacked bytes: every file present,
/// exact, and nothing extra.
fn check_manifest(unpacked: &Path) -> Result<(), InstallError> {
    let manifest_path = unpacked.join("manifest.json");
    let bytes = fs::read(&manifest_path)
        .map_err(|_| InstallError::BadBundle("it has no manifest.json".into()))?;
    if bytes.len() as u64 > 512 * 1024 {
        return Err(InstallError::BadBundle("the manifest is too large".into()));
    }
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|_| InstallError::BadBundle("the manifest can't be read".into()))?;
    let mut listed = BTreeSet::new();
    for pin in &manifest.files {
        let relative = safe_relative(&pin.path)
            .ok_or_else(|| InstallError::BadBundle(format!("{} leaves the bundle", pin.path)))?;
        if !listed.insert(relative.clone()) {
            return Err(InstallError::BadBundle(format!(
                "{} is listed twice",
                pin.path
            )));
        }
        let full = unpacked.join(&relative);
        let meta = fs::symlink_metadata(&full)
            .map_err(|_| InstallError::BadBundle(format!("{} is missing", pin.path)))?;
        if !meta.is_file() {
            return Err(InstallError::BadBundle(format!(
                "{} isn't a plain file",
                pin.path
            )));
        }
        if meta.len() != pin.size {
            return Err(InstallError::BadBundle(format!(
                "{} is a different size",
                pin.path
            )));
        }
        let bytes = fs::read(&full)?;
        if sha256_hex(&bytes) != pin.sha256 {
            return Err(InstallError::BadBundle(format!(
                "{} is a different file",
                pin.path
            )));
        }
    }
    let mut found = BTreeSet::new();
    collect_files(unpacked, unpacked, &mut found, 0)?;
    found.remove(Path::new("manifest.json"));
    if found != listed {
        return Err(InstallError::BadBundle(
            "the bundle holds files its manifest doesn't list".into(),
        ));
    }
    Ok(())
}

fn collect_files(
    root: &Path,
    dir: &Path,
    out: &mut BTreeSet<PathBuf>,
    depth: usize,
) -> Result<(), InstallError> {
    if depth > 16 {
        return Err(InstallError::BadBundle(
            "the folders nest too deeply".into(),
        ));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err(InstallError::BadBundle(format!(
                "{} is a link",
                path.display()
            )));
        }
        if meta.is_dir() {
            collect_files(root, &path, out, depth + 1)?;
        } else if meta.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| InstallError::BadBundle("a file leaves the bundle".into()))?;
            out.insert(relative.to_path_buf());
        } else {
            return Err(InstallError::BadBundle(format!(
                "{} isn't a plain file",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Checks the unpacked icon theme folder against the caps, then sanitizes
/// every SVG in place. A failure rejects the whole theme.
fn check_folder(folder: &Path, caps: IconCaps) -> Result<(), InstallError> {
    let mut json = Vec::new();
    let mut files = BTreeSet::new();
    collect_files(folder, folder, &mut files, 0)?;
    if files.len() > 4_096 {
        return Err(InstallError::BadFolder("it holds too many files".into()));
    }
    let mut total = 0u64;
    for relative in &files {
        let name = relative.to_string_lossy().replace('\\', "/");
        if name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part.starts_with('.'))
            || name.len() > 200
        {
            return Err(InstallError::BadFolder(format!("{name} isn't a safe path")));
        }
        let path = folder.join(relative);
        let meta = fs::symlink_metadata(&path)?;
        if !meta.is_file() {
            return Err(InstallError::BadFolder(format!("{name} isn't a file")));
        }
        total += meta.len();
        if total > caps.max_theme_bytes {
            return Err(InstallError::BadFolder("it holds too many bytes".into()));
        }
        if name.ends_with(".json") {
            json.push(path.clone());
            if meta.len() > caps.max_json {
                return Err(InstallError::BadFolder(format!(
                    "{name} is larger than {} KB",
                    caps.max_json / 1024
                )));
            }
        } else if name.to_ascii_lowercase().ends_with(".svg") {
            if meta.len() > caps.max_svg {
                return Err(InstallError::BadFolder(format!(
                    "{name} is larger than {} KB",
                    caps.max_svg / 1024
                )));
            }
            let bytes = fs::read(&path)?;
            let clean = sanitize_svg(&bytes)
                .map_err(|err| InstallError::BadFolder(format!("{name} {err}")))?;
            // The installed tree holds the normalized SVG, so a later load
            // sanitizes it to the same bytes.
            if clean != bytes {
                write_file(&path, &clean)?;
            }
        } else {
            // A licence or readme is harmless; anything else has no role in
            // an icon theme folder.
            let lower = name.to_ascii_lowercase();
            let allowed = lower.ends_with(".md")
                || lower.ends_with(".txt")
                || lower.contains("license")
                || lower.contains("licence")
                || lower.contains("copying");
            if !allowed {
                return Err(InstallError::BadFolder(format!(
                    "{name} isn't an icon theme file"
                )));
            }
        }
    }
    if json.len() != 1 {
        return Err(InstallError::BadFolder(
            "it must hold exactly one .json theme file".into(),
        ));
    }
    Ok(())
}

/// `path` as a relative path with only normal parts. Pure.
fn safe_relative(path: &str) -> Option<PathBuf> {
    if path.is_empty() || path.contains('\\') || path.starts_with('/') {
        return None;
    }
    let mut out = PathBuf::new();
    for part in Path::new(path).components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// Reads a file at most `limit` bytes, for the app's icon loader.
pub fn read_capped(path: &Path, limit: u64) -> io::Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "isn't a file of the expected size",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "is too large"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{Appearance, Kind};
    use proptest::prelude::*;

    fn theme(bytes: &[u8], kind: Kind) -> Theme {
        let path = match kind {
            Kind::Color => "nord.json".to_owned(),
            Kind::Icons => "gallery/icon-themes/nord.tar.gz".to_owned(),
        };
        let pin = FileRef {
            path: path.clone(),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        };
        Theme {
            id: "nord".into(),
            kind,
            family: None,
            version: 1,
            name: "Nord".into(),
            appearance: (kind == Kind::Color).then_some(Appearance::Dark),
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com".into(),
            description: "A theme.".into(),
            min_kanso: "0.1.0".into(),
            file: (kind == Kind::Color).then(|| pin.clone()),
            bundle: (kind == Kind::Icons).then_some(pin),
            preview: Vec::new(),
            colors: None,
        }
    }

    #[test]
    fn a_color_theme_is_written_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let bytes = br#"{"schema_version":1}"#;
        let theme = theme(bytes, Kind::Color);
        let digest = install_color(&root, &theme, bytes).unwrap();
        assert_eq!(digest, sha256_hex(bytes));
        assert_eq!(fs::read(root.join("themes/nord.json")).unwrap(), bytes);
    }

    #[test]
    fn a_wrong_hash_is_never_written() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let good = br#"{"schema_version":1}"#;
        let mut theme = theme(good, Kind::Color);
        theme.file.as_mut().unwrap().sha256 = "ab".repeat(32);
        assert!(matches!(
            install_color(&root, &theme, good),
            Err(InstallError::ChecksumMismatch)
        ));
        assert!(!root.join("themes/nord.json").exists());
    }

    fn bundle(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        ));
        for (path, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, path, *bytes).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn a_bundle_that_unpacks_too_large_is_refused_before_the_swap() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        // A small gzip that unpacks past the gallery's 16 MiB total.
        let huge = vec![0u8; (crate::MAX_BUNDLE_BYTES as usize) + 1];
        let bytes = bundle(&[("nord/nord.json", &huge)]);
        let theme = theme(&bytes, Kind::Icons);
        let err = install_icons(&root, &theme, &bytes, IconCaps::default()).unwrap_err();
        assert!(matches!(err, InstallError::BadBundle(_)), "{err}");
        assert!(!root.join("icon-themes/nord").exists());
    }

    #[test]
    fn a_bundle_with_too_many_entries_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let many: Vec<(&str, &[u8])> = (0..crate::MAX_BUNDLE_ENTRIES + 1)
            .map(|_| ("nord/icons/a.svg", b"<svg/>".as_slice()))
            .collect();
        let bytes = bundle(&many);
        let theme = theme(&bytes, Kind::Icons);
        assert!(install_icons(&root, &theme, &bytes, IconCaps::default()).is_err());
        assert!(!root.join("icon-themes/nord").exists());
    }

    #[test]
    fn every_svg_in_the_icon_themes_passes_the_sanitizer() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes/icon-themes");
        let mut count = 0;
        walk_svgs(&root, &mut |path| {
            let bytes = fs::read(path).unwrap();
            let out =
                sanitize_svg(&bytes).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            let again = sanitize_svg(&out).unwrap();
            assert_eq!(out, again, "{}", path.display());
            count += 1;
        });
        assert!(count > 300, "only {count} SVGs found");
    }

    fn walk_svgs(dir: &Path, at: &mut impl FnMut(&Path)) {
        let Ok(read) = fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_svgs(&path, at);
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
            {
                at(&path);
            }
        }
    }

    proptest! {
        /// Bytes that don't hash to the pin are never written.
        #[test]
        fn prop_unpinned_bytes_are_never_written(
            pinned in prop::collection::vec(any::<u8>(), 1..64),
            given in prop::collection::vec(any::<u8>(), 0..64),
        ) {
            prop_assume!(pinned != given);
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("gallery");
            let theme = theme(&pinned, Kind::Color);
            prop_assert!(install_color(&root, &theme, &given).is_err());
            prop_assert!(!root.join("themes/nord.json").exists());
        }

        /// What was pinned is what is on disk, byte for byte.
        #[test]
        fn prop_the_installed_file_is_the_checked_one(bytes in prop::collection::vec(any::<u8>(), 1..512)) {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("gallery");
            let theme = theme(&bytes, Kind::Color);
            install_color(&root, &theme, &bytes).unwrap();
            prop_assert_eq!(fs::read(root.join("themes/nord.json")).unwrap(), bytes);
        }
    }
}
