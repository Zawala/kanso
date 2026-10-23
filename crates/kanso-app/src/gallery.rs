//! The theme gallery's work away from the window: installing a checked
//! catalog entry under `<data>/gallery/`, reconciling what is on disk with
//! `state.json`, removing a theme, and turning a crash between a swap and a
//! state write into a recorded or discarded pending entry.
//!
//! Fetching and deciding what to update live in `settings_gallery` and
//! `updates`; this module never touches the network.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use kanso_gallery::state::{Pending, Record, now_rfc3339};
use kanso_gallery::{IconCaps, Kind, State, Theme};

use crate::icon_theme_file;
use crate::origin::Origin;
use crate::theme_file::ThemeFile;
use crate::theme_import;

/// Installs run on background threads, but they read and write one
/// `state.json`; one at a time keeps two halves of a pair from losing each
/// other's record.
static INSTALL_LOCK: Mutex<()> = Mutex::new(());

/// The digest of an installed record's bytes now: a color file's SHA-256, or
/// an icon folder's tree digest. `None` when it is missing.
pub fn current_digest(root: &Path, record: &Record) -> Option<String> {
    match record.kind {
        Kind::Color => {
            let bytes =
                std::fs::read(root.join("themes").join(format!("{}.json", record.id))).ok()?;
            Some(kanso_gallery::color_digest(&bytes))
        }
        Kind::Icons => {
            let folder = root.join("icon-themes").join(&record.id);
            kanso_update::stage::payload_digest(&folder).ok()
        }
    }
}

/// Resolves a pending record from an earlier run and returns the state
/// reconciled against the bytes on disk. A swap that completed is recorded;
/// one that didn't is forgotten, so the old copy stays installed.
pub fn reconcile(root: &Path) -> State {
    let path = root.join("state.json");
    let mut state = State::load(&path);
    let known = state.installed.clone();
    let pending = state.pending.clone();
    let before = state.clone();
    state.reconcile(|id| {
        if let Some(record) = known.iter().find(|record| record.id == id) {
            return current_digest(root, record);
        }
        let swap = pending.iter().find(|swap| swap.id == id)?;
        let record = Record {
            id: swap.id.clone(),
            kind: swap.kind,
            version: swap.version,
            name: swap.name.clone(),
            family: swap.family.clone(),
            appearance: swap.appearance,
            digest: swap.digest.clone(),
            installed_at: swap.installed_at.clone(),
        };
        current_digest(root, &record)
    });
    if state != before {
        state.save(&path).ok();
    }
    state
}

/// The installed gallery themes, each with the digest of what is on disk
/// now, for the catalog's plan.
pub fn installed(root: &Path) -> Vec<kanso_gallery::Installed> {
    let state = reconcile(root);
    state.plan_records(|record| current_digest(root, record))
}

/// The display name `state.json` recorded for an installed theme, if any.
/// The color settings select by name, not by id.
pub fn installed_name(root: &Path, id: &str) -> Option<String> {
    State::load(&root.join("state.json"))
        .record(id)
        .map(|record| record.name.clone())
}

/// Keeps a revoked, edited theme on disk but out of the loaders, and tells
/// the person why through its Browse card. Returns whether it was newly
/// disabled, so a check that finds it again doesn't announce it again.
pub fn disable(root: &Path, id: &str) -> bool {
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let path = root.join("state.json");
    let mut state = State::load(&path);
    if !state.disabled.iter().any(|other| other == id) {
        state.disabled.push(id.to_owned());
        state.disabled.sort();
        state.save(&path).ok();
        return true;
    }
    false
}

/// Installs a color theme the catalog pinned, after holding it to the same
/// parser and legibility checks as an imported theme. `reserved` holds the
/// names of built-in and own themes, which a gallery install must not
/// shadow.
pub fn install_color(
    root: &Path,
    theme: &Theme,
    bytes: &[u8],
    reserved: &HashSet<String>,
    automatic: bool,
) -> Result<(), String> {
    if theme.kind != Kind::Color {
        return Err("that entry isn't a color theme".into());
    }
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let text = std::str::from_utf8(bytes).map_err(|_| "the theme isn't UTF-8".to_owned())?;
    let file = ThemeFile::parse(text).map_err(|err| format!("the theme {err}"))?;
    if file.name != theme.name {
        return Err(format!(
            "the catalog calls it \"{}\", but the file calls itself \"{}\"",
            theme.name, file.name
        ));
    }
    if reserved.contains(&file.name.to_lowercase()) {
        return Err(format!(
            "\"{}\" is already a built-in or your own theme; it can't be installed",
            file.name
        ));
    }
    let problems = theme_import::problems(&file);
    if let Some(problem) = problems.first() {
        return Err(format!("it doesn't read well: {problem}"));
    }
    let path = root.join("state.json");
    let mut state = State::load(&path);
    check_replacement(root, &state, theme.kind, &theme.id, automatic)?;
    check_install_cap(&state, theme.kind, &theme.id)?;
    let pending = Pending {
        id: theme.id.clone(),
        kind: Kind::Color,
        version: theme.version,
        name: theme.name.clone(),
        family: theme.family.clone(),
        appearance: theme.appearance,
        digest: kanso_gallery::color_digest(bytes),
        installed_at: now_rfc3339(),
    };
    state.pending.push(pending.clone());
    state.save(&path).map_err(|err| err.to_string())?;
    if let Err(err) = kanso_gallery::install_color(root, theme, bytes) {
        forget_pending(&mut state, &theme.id);
        state.save(&path).ok();
        return Err(err.to_string());
    }
    record(&mut state, pending);
    state.save(&path).map_err(|err| err.to_string())
}

/// Installs an icon theme the catalog pinned, checks the unpacked folder
/// with exactly the code that reads a person's icon theme, and records it.
/// `reserved_ids` are the built-ins' and own folders'; `reserved_names` the
/// same themes' display names.
#[allow(dead_code)] // Called by the Browse view in #49 step 4.
pub fn install_icons(
    root: &Path,
    theme: &Theme,
    bundle: &[u8],
    reserved_ids: &HashSet<String>,
    reserved_names: &HashSet<String>,
    automatic: bool,
) -> Result<(), String> {
    if theme.kind != Kind::Icons {
        return Err("that entry isn't an icon theme".into());
    }
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if reserved_ids.contains(&theme.id) {
        return Err(format!(
            "\"{}\" is already a built-in or your own icon theme",
            theme.id
        ));
    }
    if reserved_names.contains(&theme.name.to_lowercase()) {
        return Err(format!(
            "\"{}\" is already a built-in or your own icon theme",
            theme.name
        ));
    }
    let path = root.join("state.json");
    let mut state = State::load(&path);
    check_replacement(root, &state, theme.kind, &theme.id, automatic)?;
    check_install_cap(&state, theme.kind, &theme.id)?;
    let mut pending: Option<Pending> = None;
    let installed_at = now_rfc3339();
    let result = kanso_gallery::install_icons_with(
        root,
        theme,
        bundle,
        IconCaps {
            max_json: icon_theme_file::MAX_JSON_BYTES,
            max_svg: icon_theme_file::MAX_SVG_BYTES,
            max_entries: icon_theme_file::MAX_ENTRIES,
            max_theme_bytes: icon_theme_file::MAX_THEME_BYTES,
        },
        &mut |folder, digest| {
            // Hold the staged folder to exactly the reader that will load
            // it. A failure here is before the swap, so a bad update leaves
            // the working older copy installed.
            let mut notices = Vec::new();
            if icon_theme_file::read_folder(folder, Origin::Gallery, &mut notices).is_none() {
                return Err(std::io::Error::other(notices.first().map_or_else(
                    || "the icon theme doesn't read".to_owned(),
                    |n| n.sentence(),
                )));
            }
            let record = Pending {
                id: theme.id.clone(),
                kind: Kind::Icons,
                version: theme.version,
                name: theme.name.clone(),
                family: theme.family.clone(),
                appearance: theme.appearance,
                digest: digest.to_owned(),
                installed_at: installed_at.clone(),
            };
            let mut with_pending = state.clone();
            with_pending.pending.push(record.clone());
            pending = Some(record);
            with_pending.save(&path)
        },
    );
    let digest = match result {
        Ok(digest) => digest,
        Err(err) => {
            if let Some(pending) = pending {
                state.pending.retain(|other| other.id != pending.id);
                state.save(&path).ok();
            }
            return Err(err.to_string());
        }
    };
    let Some(pending) = pending else {
        return Err("the icon theme wasn't staged".into());
    };
    debug_assert_eq!(pending.digest, digest);
    record(&mut state, pending);
    state.save(&path).map_err(|err| err.to_string())
}

/// Removes an installed gallery theme and its record. The person's own
/// themes are never touched.
pub fn remove(root: &Path, kind: Kind, id: &str) -> Result<(), String> {
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    kanso_gallery::remove_installed(root, kind, id).map_err(|err| err.to_string())?;
    let path = root.join("state.json");
    let mut state = State::load(&path);
    state.remove(id);
    state.disabled.retain(|other| other != id);
    state.save(&path).map_err(|err| err.to_string())
}

/// Refuses to write over a copy the person has edited, or over an
/// unrecorded file. A missing copy may be reinstalled by hand, never
/// automatically. Checked here, inside the install, so a change between the
/// plan and the write cannot slip through.
fn check_replacement(
    root: &Path,
    state: &State,
    kind: Kind,
    id: &str,
    automatic: bool,
) -> Result<(), String> {
    let target = installed_path(root, kind, id);
    match state.record(id) {
        Some(record) => match current_digest(root, record) {
            Some(digest) if digest == record.digest => Ok(()),
            None if !automatic => Ok(()),
            None => Err(format!(
                "\"{}\" is missing; install it again from Browse",
                record.name
            )),
            Some(_) => Err(format!(
                "\"{}\" was edited; the gallery won't overwrite your copy",
                record.name
            )),
        },
        None if target.exists() => Err(format!(
            "a file named \"{id}\" is already installed but not by the gallery"
        )),
        None => Ok(()),
    }
}

fn check_install_cap(state: &State, kind: Kind, id: &str) -> Result<(), String> {
    let limit = match kind {
        Kind::Color => kanso_gallery::MAX_GALLERY_COLOR_THEMES,
        Kind::Icons => kanso_gallery::MAX_GALLERY_ICON_THEMES,
    };
    let count = state
        .installed
        .iter()
        .filter(|record| record.kind == kind && record.id != id)
        .count();
    if count >= limit {
        return Err(format!(
            "there are already {limit} gallery {} themes installed; remove one first",
            kind.as_str()
        ));
    }
    Ok(())
}

fn installed_path(root: &Path, kind: Kind, id: &str) -> std::path::PathBuf {
    match kind {
        Kind::Color => root.join("themes").join(format!("{id}.json")),
        Kind::Icons => root.join("icon-themes").join(id),
    }
}

fn record(state: &mut State, pending: Pending) {
    state.pending.retain(|other| other.id != pending.id);
    state.insert(Record {
        id: pending.id,
        kind: pending.kind,
        version: pending.version,
        name: pending.name,
        family: pending.family,
        appearance: pending.appearance,
        digest: pending.digest,
        installed_at: pending.installed_at,
    });
}

fn forget_pending(state: &mut State, id: &str) {
    state.pending.retain(|other| other.id != id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_gallery::install::color_digest;
    use kanso_gallery::{Appearance, FileRef, Theme, sha256_hex};
    use proptest::prelude::*;

    fn theme() -> (Theme, Vec<u8>) {
        let file = crate::themes::builtins()[0].file.to_json_string();
        let bytes = file.clone().into_bytes();
        let name = ThemeFile::parse(&file).unwrap().name;
        (
            Theme {
                id: "gallery-test".into(),
                kind: Kind::Color,
                family: Some("test".into()),
                version: 3,
                name,
                appearance: Some(Appearance::Light),
                author: "Author".into(),
                license: "MIT".into(),
                source: "https://example.com".into(),
                description: "A test.".into(),
                min_kanso: "0.1.0".into(),
                file: Some(FileRef {
                    path: "gallery-test.json".into(),
                    sha256: sha256_hex(&bytes),
                    size: bytes.len() as u64,
                }),
                bundle: None,
                preview: Vec::new(),
                colors: None,
            },
            bytes,
        )
    }

    #[test]
    fn disabling_twice_reports_only_the_first_time() {
        let dir = tempfile::tempdir().unwrap();
        assert!(disable(dir.path(), "nord"));
        assert!(!disable(dir.path(), "nord"));
        let state = State::load(&dir.path().join("state.json"));
        assert_eq!(state.disabled, vec!["nord".to_owned()]);
    }

    #[test]
    fn installing_records_what_was_written_and_spots_an_edit() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        let records = installed(&root);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].version, 3);
        assert_eq!(records[0].digest, color_digest(&bytes));
        assert!(records[0].is_unedited());
        // Editing the file makes the current digest differ.
        let path = root.join("themes/gallery-test.json");
        std::fs::write(&path, b"{}").unwrap();
        let after = installed(&root);
        assert!(!after[0].is_unedited());
    }

    #[test]
    fn a_pending_swap_is_resolved_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        std::fs::create_dir_all(root.join("themes")).unwrap();
        std::fs::write(root.join("themes/gallery-test.json"), &bytes).unwrap();
        let mut state = State::default();
        state.pending.push(Pending {
            id: theme.id.clone(),
            kind: Kind::Color,
            version: 3,
            name: theme.name.clone(),
            family: None,
            appearance: None,
            digest: color_digest(&bytes),
            installed_at: now_rfc3339(),
        });
        state.save(&root.join("state.json")).unwrap();
        let state = reconcile(&root);
        assert!(state.pending.is_empty());
        assert_eq!(state.record("gallery-test").unwrap().version, 3);
    }

    #[test]
    fn removing_a_theme_removes_its_record() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        remove(&root, Kind::Color, "gallery-test").unwrap();
        assert!(!root.join("themes/gallery-test.json").exists());
        assert!(
            State::load(&root.join("state.json"))
                .record("gallery-test")
                .is_none()
        );
    }

    fn icon_bundle(id: &str) -> Vec<u8> {
        icon_bundle_with(id, br#"{"schema_version": 1, "name": "Demo"}"#)
    }

    fn icon_bundle_with(id: &str, json: &[u8]) -> Vec<u8> {
        let json = json.to_vec();
        let svg =
            br#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0h4v4z"/></svg>"#.to_vec();
        let manifest = format!(
            r#"{{"files":[{{"path":"{id}/{id}.json","sha256":"{}","size":{}}},{{"path":"{id}/icons/a.svg","sha256":"{}","size":{}}}]}}"#,
            sha256_hex(&json),
            json.len(),
            sha256_hex(&svg),
            svg.len()
        );
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        ));
        let mut add = |path: &str, bytes: &[u8]| {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, path, bytes).unwrap();
        };
        add("manifest.json", manifest.as_bytes());
        add(&format!("{id}/{id}.json"), &json);
        add(&format!("{id}/icons/a.svg"), &svg);
        let encoder = tar.into_inner().unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn an_icon_bundle_installs_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let bundle = icon_bundle("demo");
        let theme = Theme {
            id: "demo".into(),
            kind: Kind::Icons,
            family: None,
            version: 1,
            name: "Demo".into(),
            appearance: None,
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com".into(),
            description: "A test.".into(),
            min_kanso: "0.1.0".into(),
            file: None,
            bundle: Some(FileRef {
                path: "gallery/icon-themes/demo.tar.gz".into(),
                sha256: sha256_hex(&bundle),
                size: bundle.len() as u64,
            }),
            preview: Vec::new(),
            colors: None,
        };
        install_icons(
            &root,
            &theme,
            &bundle,
            &HashSet::new(),
            &HashSet::new(),
            false,
        )
        .unwrap();
        let records = installed(&root);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].kind, Kind::Icons);
        assert!(root.join("icon-themes/demo/demo.json").is_file());
        // The sanitizer normalized the SVG on disk.
        let svg = std::fs::read_to_string(root.join("icon-themes/demo/icons/a.svg")).unwrap();
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn an_edited_copy_is_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        std::fs::write(root.join("themes/gallery-test.json"), b"{\"edited\":true}").unwrap();
        let mut newer = bytes.clone();
        newer.extend_from_slice(b" ");
        let err = install_color(&root, &theme, &newer, &HashSet::new(), false).unwrap_err();
        assert!(err.contains("edited"), "{err}");
        assert_eq!(
            std::fs::read(root.join("themes/gallery-test.json")).unwrap(),
            b"{\"edited\":true}"
        );
    }

    #[test]
    fn a_file_the_gallery_did_not_write_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        std::fs::create_dir_all(root.join("themes")).unwrap();
        std::fs::write(root.join("themes/gallery-test.json"), b"{}").unwrap();
        let (theme, bytes) = theme();
        let err = install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap_err();
        assert!(err.contains("not by the gallery"), "{err}");
        assert_eq!(
            std::fs::read(root.join("themes/gallery-test.json")).unwrap(),
            b"{}"
        );
    }

    #[test]
    fn a_missing_copy_is_reinstalled_only_by_hand() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        std::fs::remove_file(root.join("themes/gallery-test.json")).unwrap();
        assert!(install_color(&root, &theme, &bytes, &HashSet::new(), true).is_err());
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        assert!(root.join("themes/gallery-test.json").is_file());
    }

    #[test]
    fn the_installed_theme_cap_is_enforced() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        std::fs::create_dir_all(&root).unwrap();
        let mut state = State::default();
        for at in 0..kanso_gallery::MAX_GALLERY_COLOR_THEMES {
            state.installed.push(Record {
                id: format!("fill-{at}"),
                kind: Kind::Color,
                version: 1,
                name: format!("Fill {at}"),
                family: None,
                appearance: None,
                digest: "aa".into(),
                installed_at: now_rfc3339(),
            });
        }
        state.save(&root.join("state.json")).unwrap();
        let (theme, bytes) = theme();
        let err = install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap_err();
        assert!(err.contains("remove one first"), "{err}");
    }

    #[test]
    fn a_bad_update_leaves_the_working_version_installed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let first = icon_bundle("demo");
        let mut theme = icon_theme(&first);
        install_icons(
            &root,
            &theme,
            &first,
            &HashSet::new(),
            &HashSet::new(),
            false,
        )
        .unwrap();
        let good = std::fs::read(root.join("icon-themes/demo/demo.json")).unwrap();

        // It installs and clears its pins, but Kanso's own reader refuses it.
        let bad = icon_bundle_with("demo", br#"{"schema_version": 99}"#);
        theme.version = 2;
        theme.bundle = Some(FileRef {
            path: "gallery/icon-themes/demo.tar.gz".into(),
            sha256: sha256_hex(&bad),
            size: bad.len() as u64,
        });
        assert!(
            install_icons(&root, &theme, &bad, &HashSet::new(), &HashSet::new(), false).is_err()
        );
        assert_eq!(
            std::fs::read(root.join("icon-themes/demo/demo.json")).unwrap(),
            good
        );
    }

    fn icon_theme(bundle: &[u8]) -> Theme {
        Theme {
            id: "demo".into(),
            kind: Kind::Icons,
            family: None,
            version: 1,
            name: "Demo".into(),
            appearance: None,
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com".into(),
            description: "A test.".into(),
            min_kanso: "0.1.0".into(),
            file: None,
            bundle: Some(FileRef {
                path: "gallery/icon-themes/demo.tar.gz".into(),
                sha256: sha256_hex(bundle),
                size: bundle.len() as u64,
            }),
            preview: Vec::new(),
            colors: None,
        }
    }

    #[test]
    fn a_name_taken_by_a_builtin_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = theme();
        let mut reserved = HashSet::new();
        reserved.insert(theme.name.to_lowercase());
        assert!(install_color(&root, &theme, &bytes, &reserved, false).is_err());
    }

    proptest! {
        /// Whatever the edit is, the gallery never writes over the copy on
        /// disk; the file keeps exactly the bytes the person left.
        #[test]
        fn prop_an_edited_copy_is_never_replaced(edit in prop::collection::vec(any::<u8>(), 1..64)) {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("gallery");
            let (theme, bytes) = theme();
            install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
            std::fs::write(root.join("themes/gallery-test.json"), &edit).unwrap();
            prop_assert!(install_color(&root, &theme, &bytes, &HashSet::new(), false).is_err());
            prop_assert_eq!(std::fs::read(root.join("themes/gallery-test.json")).unwrap(), edit);
        }
    }
}
