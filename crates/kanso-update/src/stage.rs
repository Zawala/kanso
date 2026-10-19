//! Downloading an update, checking it against the signed feed, and
//! unpacking it beside the install, ready to swap in.
//!
//! The staging folder sits next to what it replaces (on the same volume, so
//! the swap is two renames). It holds the unpacked update and
//! `pending.json`, which says what replaces what; nothing else is kept.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::feed::Update;
use crate::fetch::{self, FetchError, Transport};
use crate::install::Install;
use crate::platform::{ArchiveKind, Platform};
use crate::policy::UrlPolicy;

/// The most an archive may unpack to: well above Kanso's size, well below
/// a disk.
const MAX_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const PENDING: &str = "pending.json";

#[derive(Debug, thiserror::Error)]
pub enum StageError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("the download's size doesn't match the release")]
    SizeMismatch,
    #[error("the download's checksum doesn't match the release")]
    ChecksumMismatch,
    #[error("there isn't enough free space for the update")]
    NoSpace,
    #[error("the update's archive isn't as expected: {0}")]
    BadArchive(String),
    #[error("the update's signature check failed: {0}")]
    CodeSignature(String),
    #[error("stopped")]
    Cancelled,
    #[error("couldn't write the update: {0}")]
    Io(io::Error),
}

impl From<io::Error> for StageError {
    fn from(err: io::Error) -> StageError {
        if is_no_space(&err) {
            StageError::NoSpace
        } else {
            StageError::Io(err)
        }
    }
}

/// Whether an I/O error means the disk is full.
pub fn is_no_space(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded
    )
}

/// A staged update: what `install::apply` swaps in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    pub version: String,
    /// The unpacked `.app` or executable, in the staging folder.
    pub payload: PathBuf,
    /// What it replaces.
    pub target: PathBuf,
    pub pub_date: String,
    pub release_url: String,
    pub notes: String,
}

impl Pending {
    pub fn save(&self, stage_dir: &Path) -> io::Result<()> {
        let tmp = stage_dir.join("pending.json.tmp");
        fs::write(
            &tmp,
            serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )?;
        fs::rename(tmp, stage_dir.join(PENDING))
    }

    /// The update staged in `stage_dir`, if one is and it is still there.
    pub fn load(stage_dir: &Path) -> Option<Pending> {
        let bytes = fs::read(stage_dir.join(PENDING)).ok()?;
        let pending: Pending = serde_json::from_slice(&bytes).ok()?;
        // Only ever a path inside the staging folder.
        (pending.payload.starts_with(stage_dir) && pending.payload.exists()).then_some(pending)
    }
}

/// Downloads `update` into `install`'s staging folder, checks it, unpacks
/// it, runs `verify` on what will be swapped in (the code signature on
/// macOS), and records it as pending. `progress` gets the bytes so far and
/// the total.
pub fn prepare(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    update: &Update,
    install: &Install,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
    verify: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<Pending, StageError> {
    let dir = &install.stage_dir;
    clear_dir(dir)?;
    fs::create_dir_all(dir)?;
    let archive = dir.join(install.platform.asset_name(&update.version));
    download(transport, policy, &update.asset, &archive, progress, cancel)?;
    let unpacked = dir.join(format!("unpacked-{}", update.version));
    let result = (|| {
        extract(&archive, install.platform.archive(), &unpacked)?;
        let payload = find_payload(&unpacked, install.platform)?;
        verify(&payload).map_err(StageError::CodeSignature)?;
        Ok(payload)
    })();
    fs::remove_file(&archive).ok();
    let payload = match result {
        Ok(payload) => payload,
        Err(err) => {
            fs::remove_dir_all(&unpacked).ok();
            return Err(err);
        }
    };
    let pending = Pending {
        version: update.version.clone(),
        payload,
        target: install.target.clone(),
        pub_date: update.pub_date.clone(),
        release_url: update.release_url.clone(),
        notes: update.notes.clone(),
    };
    pending.save(dir)?;
    Ok(pending)
}

/// Empties the staging folder, keeping the folder.
pub fn clear_dir(dir: &Path) -> io::Result<()> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if entry.file_type().is_ok_and(|t| t.is_dir()) {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            }
            Ok(())
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// Streams `asset` into `to`, hashing as it goes, and keeps it only when
/// its size and SHA-256 are the feed's.
pub fn download(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    asset: &crate::feed::Asset,
    to: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<(), StageError> {
    let part = to.with_extension("part");
    let result = (|| {
        let mut body = fetch::open(transport, policy, &asset.url)?;
        let mut file = File::create(&part)?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut total = 0u64;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(StageError::Cancelled);
            }
            let n = body.read(&mut buf).map_err(FetchError::Io)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > asset.size {
                return Err(StageError::SizeMismatch);
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n])?;
            progress(total, asset.size);
        }
        if total != asset.size {
            return Err(StageError::SizeMismatch);
        }
        file.sync_all()?;
        let digest = hasher.finalize();
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        if hex != asset.sha256 {
            return Err(StageError::ChecksumMismatch);
        }
        fs::rename(&part, to)?;
        Ok(())
    })();
    if result.is_err() {
        fs::remove_file(&part).ok();
    }
    result
}

/// Unpacks `archive` into `dest`, which must not exist yet. Only plain
/// files and folders are taken, each inside `dest`: links, devices and
/// paths that climb out are refused, and so is an archive that unpacks to
/// more than [`MAX_UNPACKED_BYTES`].
pub fn extract(archive: &Path, kind: ArchiveKind, dest: &Path) -> Result<(), StageError> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    match kind {
        ArchiveKind::TarGz => extract_tar_gz(archive, dest),
        ArchiveKind::Zip => extract_zip(archive, dest),
    }
}

/// `name` as a path inside the destination: relative, with only normal
/// parts.
fn safe_relative(name: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in name.components() {
        match part {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), StageError> {
    let file = File::open(archive)?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let mut total = 0u64;
    let entries = tar
        .entries()
        .map_err(|e| StageError::BadArchive(e.to_string()))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| StageError::BadArchive(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| StageError::BadArchive(e.to_string()))?
            .into_owned();
        let Some(rel) = safe_relative(&path) else {
            return Err(StageError::BadArchive(format!(
                "{} leaves the folder",
                path.display()
            )));
        };
        let out = dest.join(&rel);
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        // GNU and PAX headers that carry long names arrive as entries of
        // their own; `entries` folds them in, so anything else here is a
        // link or a device.
        if !kind.is_file() {
            return Err(StageError::BadArchive(format!(
                "{} isn't a plain file",
                rel.display()
            )));
        }
        total += entry.size();
        if total > MAX_UNPACKED_BYTES {
            return Err(StageError::BadArchive("it unpacks too large".into()));
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&out)?;
        io::copy(&mut entry, &mut file).map_err(|e| {
            if is_no_space(&e) {
                StageError::NoSpace
            } else {
                StageError::BadArchive(e.to_string())
            }
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = entry.header().mode().unwrap_or(0o644) & 0o755;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), StageError> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| StageError::BadArchive(e.to_string()))?;
    let mut total = 0u64;
    for ix in 0..zip.len() {
        let mut entry = zip
            .by_index(ix)
            .map_err(|e| StageError::BadArchive(e.to_string()))?;
        let Some(rel) = entry.enclosed_name().and_then(|p| safe_relative(&p)) else {
            return Err(StageError::BadArchive(format!(
                "{} leaves the folder",
                entry.name()
            )));
        };
        let out = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        if entry.is_symlink() {
            return Err(StageError::BadArchive(format!(
                "{} isn't a plain file",
                rel.display()
            )));
        }
        total += entry.size();
        if total > MAX_UNPACKED_BYTES {
            return Err(StageError::BadArchive("it unpacks too large".into()));
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&out)?;
        io::copy(&mut entry, &mut file).map_err(|e| {
            if is_no_space(&e) {
                StageError::NoSpace
            } else {
                StageError::BadArchive(e.to_string())
            }
        })?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode & 0o755))?;
        }
    }
    Ok(())
}

/// What in an unpacked update replaces the install: the one `.app` on
/// macOS; `kanso` or `kanso.exe`, at the top or in the one folder there,
/// elsewhere.
pub fn find_payload(unpacked: &Path, platform: Platform) -> Result<PathBuf, StageError> {
    let top: Vec<PathBuf> = fs::read_dir(unpacked)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| !is_hidden(p))
        .collect();
    let bad = |why: &str| StageError::BadArchive(why.to_owned());
    if platform.is_bundle() {
        let apps: Vec<&PathBuf> = top
            .iter()
            .filter(|p| p.is_dir() && p.extension().is_some_and(|e| e == "app"))
            .collect();
        let [app] = apps.as_slice() else {
            return Err(bad("expected one .app"));
        };
        if !app.join("Contents/MacOS/kanso").is_file() || !app.join("Contents/Info.plist").is_file()
        {
            return Err(bad("the .app has no Kanso inside"));
        }
        return Ok((*app).clone());
    }
    let exe = if platform == Platform::WindowsX64 {
        "kanso.exe"
    } else {
        "kanso"
    };
    let direct = unpacked.join(exe);
    if direct.is_file() {
        return Ok(direct);
    }
    if let [only] = top.as_slice() {
        let nested = only.join(exe);
        if only.is_dir() && nested.is_file() {
            return Ok(nested);
        }
    }
    Err(bad(&format!("no {exe} in it")))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with('.') || n == "__MACOSX")
}

#[cfg(test)]
pub(crate) mod archives {
    //! Builds update archives in tests.

    use std::io::Write as _;
    use std::path::Path;

    /// A gzipped tar of `files` (path, contents, mode).
    pub fn tar_gz(files: &[(&str, &[u8], u32)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (path, bytes, mode) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(*mode);
            header.set_cksum();
            builder.append_data(&mut header, path, *bytes).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    /// A tar whose one entry's raw name is `name`, bypassing the builder's
    /// own checks, as an attacker would write it.
    pub fn tar_gz_raw_name(name: &str, kind: tar::EntryType) -> Vec<u8> {
        let mut header = tar::Header::new_old();
        header.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
        header.set_entry_type(kind);
        header.set_size(if kind.is_file() { 1 } else { 0 });
        header.set_mode(0o644);
        if kind.is_symlink() {
            header.set_link_name("/etc/passwd").unwrap();
        }
        header.set_cksum();
        let mut raw = Vec::new();
        raw.extend_from_slice(header.as_bytes());
        if kind.is_file() {
            raw.push(b'x');
            raw.resize(1024, 0);
        }
        raw.resize(raw.len() + 1024, 0);
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        gz.write_all(&raw).unwrap();
        gz.finish().unwrap()
    }

    pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut out);
            for (path, bytes) in files {
                zip.start_file(*path, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(bytes).unwrap();
            }
            zip.finish().unwrap();
        }
        out.into_inner()
    }

    pub fn write(path: &Path, bytes: &[u8]) {
        std::fs::write(path, bytes).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::archives::*;
    use super::*;
    use crate::feed::Asset;
    use crate::fetch::fake::{Answer, FakeTransport};
    use proptest::prelude::*;

    const URL: &str = "https://github.com/iamngoni/kanso/releases/download/v0.2.0/Kanso-0.2.0-macos-arm64.app.tar.gz";

    fn sha(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn fetch(body: &[u8], asset: &Asset, to: &Path) -> Result<(), StageError> {
        let t = FakeTransport::default().with(URL, Answer::Body(body.to_vec()));
        download(
            &t,
            &UrlPolicy::RELEASE,
            asset,
            to,
            &mut |_, _| {},
            &AtomicBool::new(false),
        )
    }

    #[test]
    fn a_matching_download_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"kanso update bytes";
        let asset = Asset {
            url: URL.into(),
            sha256: sha(body),
            size: body.len() as u64,
        };
        let to = dir.path().join("a.tar.gz");
        fetch(body, &asset, &to).unwrap();
        assert_eq!(fs::read(&to).unwrap(), body);
        assert!(!to.with_extension("part").exists());
    }

    #[test]
    fn a_wrong_size_or_checksum_is_thrown_away() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"kanso update bytes";
        let to = dir.path().join("a.tar.gz");
        for (asset, expect) in [
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(body),
                    size: body.len() as u64 + 1,
                },
                "size",
            ),
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(body),
                    size: body.len() as u64 - 1,
                },
                "size",
            ),
            (
                Asset {
                    url: URL.into(),
                    sha256: sha(b"other"),
                    size: body.len() as u64,
                },
                "checksum",
            ),
        ] {
            let err = fetch(body, &asset, &to).unwrap_err();
            match expect {
                "size" => assert!(matches!(err, StageError::SizeMismatch), "{err:?}"),
                _ => assert!(matches!(err, StageError::ChecksumMismatch), "{err:?}"),
            }
            assert!(!to.exists() && !to.with_extension("part").exists());
        }
    }

    #[test]
    fn a_cancelled_download_stops() {
        let dir = tempfile::tempdir().unwrap();
        let body = b"bytes";
        let asset = Asset {
            url: URL.into(),
            sha256: sha(body),
            size: 5,
        };
        let t = FakeTransport::default().with(URL, Answer::Body(body.to_vec()));
        let err = download(
            &t,
            &UrlPolicy::RELEASE,
            &asset,
            &dir.path().join("a"),
            &mut |_, _| {},
            &AtomicBool::new(true),
        )
        .unwrap_err();
        assert!(matches!(err, StageError::Cancelled));
    }

    #[test]
    fn an_app_bundle_unpacks_with_its_modes() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.tar.gz");
        write(
            &archive,
            &tar_gz(&[
                ("Kanso.app/Contents/Info.plist", b"<plist/>", 0o644),
                ("Kanso.app/Contents/MacOS/kanso", b"#!/bin/sh\n", 0o755),
            ]),
        );
        let dest = dir.path().join("out");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        let app = find_payload(&dest, Platform::MacosArm64).unwrap();
        assert_eq!(app, dest.join("Kanso.app"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(app.join("Contents/MacOS/kanso"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }
    }

    #[test]
    fn linux_and_windows_payloads_are_found() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("l.tar.gz");
        write(
            &archive,
            &tar_gz(&[
                ("Kanso-0.2.0-linux-x64/kanso", b"\x7fELF", 0o755),
                ("Kanso-0.2.0-linux-x64/install.sh", b"#!/bin/sh", 0o755),
            ]),
        );
        let dest = dir.path().join("l");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert_eq!(
            find_payload(&dest, Platform::LinuxX64).unwrap(),
            dest.join("Kanso-0.2.0-linux-x64/kanso")
        );

        let archive = dir.path().join("w.zip");
        write(&archive, &zip(&[("kanso.exe", b"MZ")]));
        let dest = dir.path().join("w");
        extract(&archive, ArchiveKind::Zip, &dest).unwrap();
        assert_eq!(
            find_payload(&dest, Platform::WindowsX64).unwrap(),
            dest.join("kanso.exe")
        );
    }

    #[test]
    fn archives_without_kanso_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.tar.gz");
        write(
            &archive,
            &tar_gz(&[("Other.app/Contents/Info.plist", b"x", 0o644)]),
        );
        let dest = dir.path().join("out");
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert!(matches!(
            find_payload(&dest, Platform::MacosArm64),
            Err(StageError::BadArchive(_))
        ));
        assert!(matches!(
            find_payload(&dest, Platform::LinuxX64),
            Err(StageError::BadArchive(_))
        ));
        // Two apps is one too many.
        write(
            &archive,
            &tar_gz(&[
                ("A.app/Contents/Info.plist", b"x", 0o644),
                ("A.app/Contents/MacOS/kanso", b"x", 0o755),
                ("B.app/Contents/Info.plist", b"x", 0o644),
                ("B.app/Contents/MacOS/kanso", b"x", 0o755),
            ]),
        );
        extract(&archive, ArchiveKind::TarGz, &dest).unwrap();
        assert!(find_payload(&dest, Platform::MacosArm64).is_err());
    }

    #[test]
    fn escaping_paths_and_links_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = dir.path().join("a.tar.gz");
        for (name, kind) in [
            ("../evil", tar::EntryType::Regular),
            ("/abs/evil", tar::EntryType::Regular),
            ("Kanso.app/link", tar::EntryType::Symlink),
            ("Kanso.app/dev", tar::EntryType::Char),
        ] {
            write(&archive, &tar_gz_raw_name(name, kind));
            let err = extract(&archive, ArchiveKind::TarGz, &dest).unwrap_err();
            assert!(matches!(err, StageError::BadArchive(_)), "{name}: {err:?}");
            assert!(!dir.path().join("evil").exists());
        }
        // A zip that climbs out is refused too.
        write(&archive, &zip(&[("../evil.exe", b"MZ")]));
        assert!(extract(&archive, ArchiveKind::Zip, &dest).is_err());
        assert!(!dir.path().join("evil.exe").exists());
    }

    #[test]
    fn pending_updates_round_trip_and_stay_inside() {
        let dir = tempfile::tempdir().unwrap();
        let payload = dir.path().join("unpacked-0.2.0/Kanso.app");
        fs::create_dir_all(&payload).unwrap();
        let pending = Pending {
            version: "0.2.0".into(),
            payload: payload.clone(),
            target: "/Applications/Kanso.app".into(),
            pub_date: "2026-10-20".into(),
            release_url: "https://github.com/iamngoni/kanso/releases/tag/v0.2.0".into(),
            notes: "- New".into(),
        };
        pending.save(dir.path()).unwrap();
        assert_eq!(Pending::load(dir.path()), Some(pending.clone()));
        // A record pointing outside the staging folder is ignored.
        let outside = Pending {
            payload: "/Applications/Other.app".into(),
            ..pending
        };
        outside.save(dir.path()).unwrap();
        assert_eq!(Pending::load(dir.path()), None);
    }

    #[test]
    fn a_full_disk_reads_as_no_space() {
        let err: StageError = io::Error::from(io::ErrorKind::StorageFull).into();
        assert!(matches!(err, StageError::NoSpace));
        let err: StageError = io::Error::from(io::ErrorKind::PermissionDenied).into();
        assert!(matches!(err, StageError::Io(_)));
    }

    proptest! {
        /// Whatever the body, only the bytes the feed names are kept.
        #[test]
        fn prop_only_the_signed_bytes_are_kept(
            body in prop::collection::vec(any::<u8>(), 1..4096),
            other in prop::collection::vec(any::<u8>(), 1..4096),
        ) {
            let dir = tempfile::tempdir().unwrap();
            let to = dir.path().join("a");
            let asset = Asset { url: URL.into(), sha256: sha(&body), size: body.len() as u64 };
            let result = fetch(&other, &asset, &to);
            prop_assert_eq!(result.is_ok(), other == body);
            prop_assert_eq!(to.exists(), other == body);
        }

        /// Paths inside the folder are kept as they are; anything else is
        /// refused.
        #[test]
        fn prop_safe_paths_stay_inside(parts in prop::collection::vec(prop::sample::select(vec!["a", "b.app", ".", "..", "Contents"]), 1..6), absolute in any::<bool>()) {
            let joined = parts.join("/");
            let path = if absolute { format!("/{joined}") } else { joined };
            let safe = safe_relative(Path::new(&path));
            let climbs = absolute || parts.contains(&"..");
            if climbs {
                prop_assert!(safe.is_none());
            } else if let Some(rel) = safe {
                prop_assert!(rel.is_relative());
                prop_assert!(rel.components().all(|c| matches!(c, Component::Normal(_))));
            }
        }
    }
}
