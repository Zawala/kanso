//! Icon packs: folders of SVG icons a person adds, as they add themes.
//!
//! A pack is a folder in the `icon-packs` folder of Kanso's data folder,
//! named with its ID, holding `.svg` files and, optionally, a `pack.json`
//! that names it and its icons:
//!
//! ```json
//! { "name": "Phosphor", "author": "…", "license": "MIT",
//!   "icons": { "rocket": { "label": "Rocket", "keywords": "launch space" } } }
//! ```
//!
//! Instead of SVG files, a pack can hold its icons in one `icons.json` in
//! Iconify's format (`{"icons": {"name": {"body": "<path …/>"}}, "width":
//! 24, "height": 24}`), as the gallery installs them (`icon_gallery`).
//!
//! Its icons join the notebook icon picker (a notebook keeps one as
//! `<pack>/<icon>`). Chosen in Settings › Themes, a pack also draws the app's
//! own icons: an SVG named like one of them (`pin.svg`, `chevron-right.svg`)
//! takes its place everywhere. A window keeps the icons it has drawn, so
//! choosing a pack opens the main window again in its place to show it at
//! once (`reopen_main_window`). Icons are drawn in one color, the theme's, as the
//! built-in ones are.
//!
//! Packs are read at launch, again whenever the folder changes, and on
//! Reload, each time off the UI thread. A pack or file that can't be used is
//! skipped with a notice that says why; it never stops the others or the
//! app. However a pack is made, what it costs is capped: so many icons, so
//! many bytes each and in all, and an icon's other names share its SVG.

use std::collections::HashMap;
use std::hash::{Hash as _, Hasher as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use gpui_kit::{App, Global, Task};

pub use crate::models::icon_pack::{IconPack, PackIcon};
use crate::models::icon_pack::{IconWords, Manifest};
use crate::models::iconify::{IconifyIcon, IconifyJson};
use crate::themes::Notice;
use crate::workspace::AppState;

/// Largest SVG a pack may hold.
pub const MAX_SVG_BYTES: u64 = 256 * 1024;
/// Most icons one pack may hold; the rest are skipped.
pub const MAX_ICONS: usize = 40_000;
/// Most bytes of SVG one pack may hold in all; the rest are skipped.
pub const MAX_PACK_BYTES: u64 = 64 * 1024 * 1024;
/// Largest `icons.json` a pack may hold.
pub const MAX_JSON_BYTES: u64 = 48 * 1024 * 1024;
/// The file holding a pack's icons in Iconify's format.
pub const ICONIFY_FILE: &str = "icons.json";
/// Most packs read.
pub const MAX_PACKS: usize = 64;
const WATCH_INTERVAL: Duration = Duration::from_secs(2);
pub const MANIFEST: &str = "pack.json";
/// Room an Iconify body needs for the `<svg …>` around it: the tag and four
/// numbers, however long.
const SVG_WRAPPER_BYTES: usize = 256;

/// The folder icon packs go in.
pub fn packs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("icon-packs")
}

/// SVGs by icon name; an SVG shared by several names is one allocation.
pub type NamedSvgs = Vec<(String, Arc<[u8]>)>;

/// Each icon of `set` as a whole SVG, by name, in name order: aliases that
/// only rename an icon included, hidden icons and ones larger than
/// [`MAX_SVG_BYTES`] left out. At most `limit` icons and `budget` bytes of
/// SVG are made; the flag says whether any were left out for that. An
/// icon's SVG is made once and shared by its aliases, so a set of one large
/// icon and many names for it costs one icon. Pure.
pub fn iconify_svgs(set: &IconifyJson, limit: usize, budget: u64) -> (NamedSvgs, bool) {
    let fits = |icon: &IconifyIcon| icon.body.len() + SVG_WRAPPER_BYTES <= MAX_SVG_BYTES as usize;
    // Each name with the icon it draws, before any SVG is made.
    let mut names: Vec<(&str, &str)> = set
        .icons
        .iter()
        .filter(|(_, icon)| !icon.hidden && fits(icon))
        .map(|(name, _)| (name.as_str(), name.as_str()))
        .collect();
    names.extend(set.aliases.iter().filter_map(|(name, alias)| {
        let plain = alias.rotate.unwrap_or(0) == 0 && !alias.h_flip && !alias.v_flip;
        let parent = set.icons.get(&alias.parent)?;
        // An icon of the same name comes first.
        (plain && !alias.hidden && fits(parent) && !set.icons.contains_key(name))
            .then_some((name.as_str(), alias.parent.as_str()))
    }));
    names.sort_unstable();
    let mut more = names.len() > limit;
    names.truncate(limit);
    let mut made: HashMap<&str, Arc<[u8]>> = HashMap::new();
    let mut spent = 0u64;
    let mut out = Vec::with_capacity(names.len());
    for (name, parent) in names {
        let svg = match made.get(parent) {
            Some(svg) => svg.clone(),
            None => {
                let icon = &set.icons[parent];
                let left = icon.left.or(set.left).unwrap_or(0.);
                let top = icon.top.or(set.top).unwrap_or(0.);
                let width = icon.width.or(set.width).unwrap_or(16.);
                let height = icon.height.or(set.height).unwrap_or(16.);
                let svg = format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{left} {top} {width} {height}">{}</svg>"#,
                    icon.body
                );
                spent += svg.len() as u64;
                if spent > budget {
                    more = true;
                    break;
                }
                let svg: Arc<[u8]> = Arc::from(svg.into_bytes());
                made.insert(parent, svg.clone());
                svg
            }
        };
        out.push((name.to_owned(), svg));
    }
    (out, more)
}

/// Whether `id` can name a pack or an icon: lowercase ASCII letters, digits,
/// dashes and underscores, starting with a letter or digit. Pure.
pub fn is_valid_name(id: &str) -> bool {
    id.len() <= 64
        && id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// "rocket-launch" as "Rocket launch". Pure.
pub fn label_from_name(name: &str) -> String {
    let spaced = name.replace(['-', '_'], " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => spaced,
    }
}

/// Whether `bytes` look like an SVG Kanso can draw: UTF-8 with an `<svg`
/// element. Pure.
pub fn looks_like_svg(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok_and(|text| text.contains("<svg"))
}

/// The label and search words `manifest` gives icon `name`.
fn words_for(manifest: &Manifest, name: &str) -> (String, String) {
    let words: Option<&IconWords> = manifest.icons.get(name);
    (
        words
            .and_then(|w| w.label.clone())
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| label_from_name(name)),
        words.and_then(|w| w.keywords.clone()).unwrap_or_default(),
    )
}

fn too_many(id: &str) -> Notice {
    Notice {
        file: id.to_owned(),
        message: format!(
            "has more than {MAX_ICONS} icons or {} MB of them; the rest are left out",
            MAX_PACK_BYTES / (1024 * 1024)
        ),
    }
}

/// Reads the pack in folder `dir`, saying what couldn't be used.
pub fn read_pack(dir: &Path, notices: &mut Vec<Notice>) -> Option<IconPack> {
    let id = dir.file_name()?.to_string_lossy().into_owned();
    if !is_valid_name(&id) {
        notices.push(Notice {
            file: id,
            message: "isn't a pack name Kanso can use; name the folder with lowercase letters, digits and dashes".into(),
        });
        return None;
    }
    let manifest = match std::fs::read_to_string(dir.join(MANIFEST)) {
        Ok(text) => match serde_json::from_str::<Manifest>(&text) {
            Ok(manifest) => manifest,
            Err(err) => {
                notices.push(Notice {
                    file: format!("{id}/{MANIFEST}"),
                    message: format!("couldn't be read: {err}"),
                });
                Manifest::default()
            }
        },
        Err(_) => Manifest::default(),
    };
    let Ok(read) = std::fs::read_dir(dir) else {
        return None;
    };
    let mut files: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("svg"))
        })
        .collect();
    files.sort();
    let mut icons = Vec::new();
    let mut spent = 0u64;
    for path in files {
        let file = path.file_name()?.to_string_lossy().into_owned();
        let where_ = format!("{id}/{file}");
        let name = path.file_stem()?.to_string_lossy().into_owned();
        if !is_valid_name(&name) {
            notices.push(Notice {
                file: where_,
                message: "isn't an icon name Kanso can use; name it with lowercase letters, digits and dashes".into(),
            });
            continue;
        }
        let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        if size > MAX_SVG_BYTES {
            notices.push(Notice {
                file: where_,
                message: format!("is larger than {} KB", MAX_SVG_BYTES / 1024),
            });
            continue;
        }
        if icons.len() == MAX_ICONS || spent + size > MAX_PACK_BYTES {
            notices.push(too_many(&id));
            break;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) if looks_like_svg(&bytes) => bytes,
            Ok(_) => {
                notices.push(Notice {
                    file: where_,
                    message: "isn't an SVG".into(),
                });
                continue;
            }
            Err(err) => {
                notices.push(Notice {
                    file: where_,
                    message: format!("couldn't be read: {err}"),
                });
                continue;
            }
        };
        spent += bytes.len() as u64;
        let (label, keywords) = words_for(&manifest, &name);
        icons.push(PackIcon {
            label,
            keywords,
            name,
            svg: Arc::from(bytes),
        });
    }
    read_iconify(dir, &id, &manifest, spent, &mut icons, notices);
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    icons.dedup_by(|a, b| a.name == b.name);
    if icons.is_empty() {
        notices.push(Notice {
            file: id,
            message: "has no SVG icons".into(),
        });
        return None;
    }
    let replaces = icons
        .iter()
        .filter(|icon| crate::assets::is_app_icon(&icon.name))
        .count();
    Some(IconPack {
        name: manifest
            .name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| label_from_name(&id)),
        author: manifest.author.filter(|a| !a.trim().is_empty()),
        license: manifest.license.filter(|l| !l.trim().is_empty()),
        iconify: manifest
            .source
            .iconify
            .filter(|prefix| is_valid_name(prefix)),
        id,
        icons,
        replaces,
    })
}

/// Adds the icons in the pack's `icons.json`, if it has one, after any SVG
/// files of the same names, within what the pack has left: `spent` bytes
/// of SVG are read already.
fn read_iconify(
    dir: &Path,
    id: &str,
    manifest: &Manifest,
    spent: u64,
    icons: &mut Vec<PackIcon>,
    notices: &mut Vec<Notice>,
) {
    let path = dir.join(ICONIFY_FILE);
    let Ok(meta) = std::fs::metadata(&path) else {
        return;
    };
    let file = format!("{id}/{ICONIFY_FILE}");
    if meta.len() > MAX_JSON_BYTES {
        notices.push(Notice {
            file,
            message: format!("is larger than {} MB", MAX_JSON_BYTES / (1024 * 1024)),
        });
        return;
    }
    let parsed = std::fs::read(&path)
        .map_err(|err| format!("couldn't be read: {err}"))
        .and_then(|bytes| {
            serde_json::from_slice::<IconifyJson>(&bytes)
                .map_err(|err| format!("isn't an Iconify icon set: {err}"))
        });
    let set = match parsed {
        Ok(set) => set,
        Err(message) => {
            notices.push(Notice { file, message });
            return;
        }
    };
    let (svgs, more) = iconify_svgs(
        &set,
        MAX_ICONS.saturating_sub(icons.len()),
        MAX_PACK_BYTES.saturating_sub(spent),
    );
    if more {
        notices.push(too_many(id));
    }
    for (name, svg) in svgs {
        if !is_valid_name(&name) {
            continue;
        }
        let (label, keywords) = words_for(manifest, &name);
        icons.push(PackIcon {
            label,
            keywords,
            name,
            svg,
        });
    }
}

/// A fingerprint of every file in the packs folder, its size and time, to
/// tell when one changes.
fn signature(dir: &Path) -> u64 {
    let mut files = Vec::new();
    if let Ok(packs) = std::fs::read_dir(dir) {
        for pack in packs.filter_map(Result::ok) {
            let Ok(read) = std::fs::read_dir(pack.path()) else {
                continue;
            };
            for entry in read.filter_map(Result::ok) {
                if let Ok(meta) = entry.metadata() {
                    files.push((entry.path(), meta.len(), meta.modified().ok()));
                }
            }
        }
    }
    files.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    files.hash(&mut hasher);
    hasher.finish()
}

/// The icon packs, as last read.
#[derive(Default)]
pub struct IconPacks {
    pub packs: Vec<Arc<IconPack>>,
    pub notices: Vec<Notice>,
    signature: u64,
    /// Counts reloads, so a view can tell its picture is stale.
    pub generation: u64,
}

impl Global for IconPacks {}

impl IconPacks {
    /// The packs in `dir`, if given, by name. Slow for large packs: read
    /// off the UI thread (`reload_soon`).
    pub fn load(dir: Option<&Path>) -> IconPacks {
        let mut packs = Vec::new();
        let mut notices = Vec::new();
        let signature = dir.map(signature).unwrap_or_default();
        if let Some(dir) = dir
            && let Ok(read) = std::fs::read_dir(dir)
        {
            let mut folders: Vec<PathBuf> = read
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                // A download in progress, or a system folder.
                .filter(|path| {
                    !path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                })
                .collect();
            folders.sort();
            for folder in folders.into_iter().take(MAX_PACKS) {
                if let Some(pack) = read_pack_safely(&folder, &mut notices) {
                    packs.push(Arc::new(pack));
                }
            }
        }
        packs.sort_by_key(|pack| pack.name.to_lowercase());
        IconPacks {
            packs,
            notices,
            signature,
            generation: 0,
        }
    }

    pub fn pack(&self, id: &str) -> Option<&Arc<IconPack>> {
        self.packs.iter().find(|pack| pack.id == id)
    }

    /// The icon `<pack>/<icon>` names, if that pack is here and has it.
    pub fn lookup(&self, key: &str) -> Option<&PackIcon> {
        let (pack, icon) = split_key(key)?;
        self.pack(pack)?.icon(icon)
    }
}

/// [`read_pack`], with a pack that trips it skipped and said so, never
/// taking the app down with it.
fn read_pack_safely(dir: &Path, notices: &mut Vec<Notice>) -> Option<IconPack> {
    let mut theirs = Vec::new();
    let read =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read_pack(dir, &mut theirs)));
    notices.append(&mut theirs);
    read.unwrap_or_else(|_| {
        notices.push(Notice {
            file: dir.file_name()?.to_string_lossy().into_owned(),
            message: "couldn't be read".into(),
        });
        None
    })
}

/// `<pack>/<icon>` as its two names, if it has that shape. Pure.
pub fn split_key(key: &str) -> Option<(&str, &str)> {
    let (pack, icon) = key.split_once('/')?;
    (is_valid_name(pack) && is_valid_name(icon)).then_some((pack, icon))
}

fn folder(cx: &App) -> Option<PathBuf> {
    cx.try_global::<AppState>()
        .map(|state| packs_dir(&state.data_dir))
}

/// Makes `packs` the ones shown, counting the reload, and draws them.
fn show(mut packs: IconPacks, cx: &mut App) -> Vec<Notice> {
    packs.generation = cx
        .try_global::<IconPacks>()
        .map_or(0, |packs| packs.generation + 1);
    let notices = packs.notices.clone();
    cx.set_global(packs);
    cx.refresh_windows();
    notices
}

/// Reads the packs folder now, on this thread, if no read has finished
/// yet. Launch reads it in the background (`watch`), so this is only for a
/// view that needs the packs before that read is done.
pub fn ensure_loaded(cx: &mut App) {
    if cx.has_global::<IconPacks>() {
        return;
    }
    let packs = IconPacks::load(folder(cx).as_deref());
    cx.set_global(packs);
}

/// The packs, read if they haven't been.
pub fn packs(cx: &mut App) -> &IconPacks {
    ensure_loaded(cx);
    cx.global::<IconPacks>()
}

/// Reads the packs folder again off the UI thread, then shows the packs
/// in every window. The task gives what couldn't be used.
pub fn reload_soon(cx: &mut App) -> Task<Vec<Notice>> {
    let dir = folder(cx);
    let read = cx
        .background_executor()
        .spawn(async move { IconPacks::load(dir.as_deref()) });
    cx.spawn(async move |cx| {
        let packs = read.await;
        cx.update(|cx| show(packs, cx))
    })
}

/// Reads the packs folder again, on this thread (tests and snapshots).
#[cfg(test)]
pub fn reload(cx: &mut App) -> Vec<Notice> {
    let packs = IconPacks::load(folder(cx).as_deref());
    show(packs, cx)
}

/// Reads the packs at launch, off the UI thread, and shows them once read,
/// so notebooks with a pack's icon show it without the picker opening
/// first. Then checks the folder every couple of seconds, also off the UI
/// thread, and reads it again when a file is added, changed or removed.
pub fn watch(cx: &mut App) {
    let first = reload_soon(cx);
    cx.spawn(async move |cx| {
        for notice in first.await {
            eprintln!("kanso: icon pack {}", notice.sentence());
        }
        loop {
            cx.background_executor().timer(WATCH_INTERVAL).await;
            let seen = cx.update(|cx| {
                let packs = cx.try_global::<IconPacks>()?;
                Some((folder(cx)?, packs.signature))
            });
            let Some((dir, seen)) = seen else {
                continue;
            };
            let now = cx
                .background_executor()
                .spawn(async move { signature(&dir) })
                .await;
            if now != seen {
                cx.update(reload_soon).await;
            }
        }
    })
    .detach();
}

// MARK: The app's own icons

/// The pack drawing the app's own icons, by icon name. The app keeps it as
/// a global, and its asset loader (`assets::AppAssets`) holds the same one,
/// so each app, and each test, has its own.
#[derive(Clone, Default)]
pub struct AppIcons(Arc<RwLock<Option<SvgsByName>>>);

type SvgsByName = HashMap<String, Arc<[u8]>>;

impl Global for AppIcons {}

impl AppIcons {
    /// Draws the app's own icons with `pack`'s, where it has them, or with
    /// the built-in ones. A window keeps the icons it has drawn: this takes
    /// effect in windows opened after it (see `reopen_main_window`).
    pub fn set(&self, pack: Option<&IconPack>) {
        let icons = pack.map(|pack| {
            pack.icons
                .iter()
                .map(|icon| (icon.name.clone(), icon.svg.clone()))
                .collect()
        });
        *self.0.write().unwrap_or_else(|e| e.into_inner()) = icons;
    }

    /// The chosen pack's SVG for the asset at `path` (`icons/<name>.svg`),
    /// if it draws that icon.
    pub fn get(&self, path: &str) -> Option<Arc<[u8]>> {
        let name = path.strip_prefix("icons/")?.strip_suffix(".svg")?;
        self.0
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()?
            .get(name)
            .cloned()
    }
}

/// Draws the app's own icons with `pack`'s (`None`: the built-in ones), in
/// windows opened from now on.
pub fn use_for_app(pack: Option<&IconPack>, cx: &App) {
    if let Some(icons) = cx.try_global::<AppIcons>() {
        icons.set(pack);
    }
}

/// The app's icons at launch, before any window is drawn: the pack the
/// config chooses, read from `data_dir`. A pack that can't be read, or
/// trips the reader, leaves the built-in icons; the caps on a pack keep
/// what it costs to read bounded, so no pack can stop a launch.
pub fn apply_at_launch(data_dir: &Path, chosen: Option<&str>) -> AppIcons {
    let icons = AppIcons::default();
    let Some(chosen) = chosen.filter(|id| is_valid_name(id)) else {
        return icons;
    };
    let mut notices = Vec::new();
    let pack = read_pack_safely(&packs_dir(data_dir).join(chosen), &mut notices);
    if pack.is_none() {
        eprintln!("kanso: the icon pack {chosen} couldn't be used; drawing the built-in icons");
    }
    icons.set(pack.as_ref());
    icons
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 12h16" stroke="currentColor"/></svg>"#;

    fn write(dir: &Path, file: &str, text: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
    }

    #[test]
    fn names_and_labels() {
        assert!(is_valid_name("phosphor"));
        assert!(is_valid_name("rocket-launch_2"));
        for bad in ["", "Rocket", "-x", "a/b", "a b", "é"] {
            assert!(!is_valid_name(bad), "{bad}");
        }
        assert_eq!(label_from_name("rocket-launch"), "Rocket launch");
        assert_eq!(split_key("phosphor/rocket"), Some(("phosphor", "rocket")));
        assert_eq!(split_key("rocket"), None);
        assert_eq!(split_key("../x/y"), None);
        assert!(looks_like_svg(SVG.as_bytes()));
        assert!(!looks_like_svg(b"<html>"));
        assert!(!looks_like_svg(&[0xff, 0xfe]));
    }

    #[test]
    fn a_pack_reads_its_icons_and_words_and_skips_what_it_cant_use() {
        let dir = tempfile::tempdir().unwrap();
        let root = packs_dir(dir.path());
        let pack = root.join("phosphor");
        write(
            &pack,
            MANIFEST,
            r#"{"name": "Phosphor", "author": "Helena", "license": "MIT",
                "icons": {"orbit": {"label": "Launch", "keywords": "space ship"}}}"#,
        );
        write(&pack, "orbit.svg", SVG);
        write(&pack, "pin.svg", SVG);
        write(&pack, "Bad Name.svg", SVG);
        write(&pack, "fake.svg", "not an svg");
        write(&pack, "notes.txt", "ignored");
        std::fs::write(
            pack.join("huge.svg"),
            vec![b' '; MAX_SVG_BYTES as usize + 1],
        )
        .unwrap();
        write(&root.join("Wrong Folder"), "a.svg", SVG);
        write(&root.join("empty"), "readme.md", "nothing");

        let packs = IconPacks::load(Some(&root));
        assert_eq!(packs.packs.len(), 1);
        let p = &packs.packs[0];
        assert_eq!((p.id.as_str(), p.name.as_str()), ("phosphor", "Phosphor"));
        assert_eq!(p.author.as_deref(), Some("Helena"));
        let names: Vec<&str> = p.icons.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["orbit", "pin"]);
        let orbit = packs.lookup("phosphor/orbit").unwrap();
        assert_eq!(
            (orbit.label.as_str(), orbit.keywords.as_str()),
            ("Launch", "space ship")
        );
        assert_eq!(packs.lookup("phosphor/pin").unwrap().label, "Pin");
        assert!(packs.lookup("phosphor/missing").is_none());
        assert!(packs.lookup("other/orbit").is_none());
        // `pin` is one of the app's own icons; `orbit` isn't one Kanso has.
        assert_eq!(p.replaces, 1);
        let said: Vec<String> = packs.notices.iter().map(Notice::sentence).collect();
        for expected in [
            "Bad Name.svg",
            "fake.svg isn't an SVG",
            "huge.svg is larger",
            "Wrong Folder",
            "empty has no SVG icons",
        ] {
            assert!(
                said.iter().any(|s| s.contains(expected)),
                "{expected}: {said:?}"
            );
        }
    }

    #[test]
    fn a_pack_reads_an_iconify_icon_set() {
        let dir = tempfile::tempdir().unwrap();
        let root = packs_dir(dir.path());
        let pack = root.join("ph");
        write(
            &pack,
            MANIFEST,
            r#"{"name": "Phosphor", "source": {"iconify": "ph"}}"#,
        );
        write(
            &pack,
            ICONIFY_FILE,
            r#"{"prefix": "ph", "width": 256, "height": 256,
                "icons": {"acorn": {"body": "<path d='M1 1'/>"},
                          "tall": {"body": "<g/>", "height": 300},
                          "secret": {"body": "<g/>", "hidden": true}},
                "aliases": {"nut": {"parent": "acorn"},
                            "acorn-turned": {"parent": "acorn", "rotate": 1}}}"#,
        );
        let packs = IconPacks::load(Some(&root));
        let p = &packs.packs[0];
        assert_eq!(p.iconify.as_deref(), Some("ph"));
        let names: Vec<&str> = p.icons.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["acorn", "nut", "tall"]);
        let svg = |name: &str| String::from_utf8(p.icon(name).unwrap().svg.to_vec()).unwrap();
        assert!(
            svg("acorn").contains(r#"viewBox="0 0 256 256""#),
            "{}",
            svg("acorn")
        );
        assert!(svg("acorn").contains("<path d='M1 1'/>"));
        assert_eq!(svg("nut"), svg("acorn"));
        assert!(svg("tall").contains(r#"viewBox="0 0 256 300""#));
        // A damaged set is reported, and leaves the pack without icons.
        write(&root.join("bad"), ICONIFY_FILE, "{ nope");
        let packs = IconPacks::load(Some(&root));
        assert!(packs.notices.iter().any(|n| n.file == "bad/icons.json"));
    }

    #[test]
    fn a_pack_without_a_manifest_is_named_after_its_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = packs_dir(dir.path());
        write(&root.join("my-icons"), "star.svg", SVG);
        write(&root.join("broken"), MANIFEST, "{ not json");
        write(&root.join("broken"), "star.svg", SVG);
        let packs = IconPacks::load(Some(&root));
        let names: Vec<&str> = packs.packs.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["Broken", "My icons"]);
        // A manifest that doesn't parse is reported; its icons still come in.
        assert!(packs.notices.iter().any(|n| n.file == "broken/pack.json"));
        assert!(IconPacks::load(None).packs.is_empty());
    }

    #[test]
    fn the_chosen_pack_draws_the_icons_it_has() {
        let dir = tempfile::tempdir().unwrap();
        let root = packs_dir(dir.path());
        write(&root.join("mine"), "pin.svg", SVG);
        let icons = apply_at_launch(dir.path(), Some("mine"));
        assert_eq!(icons.get("icons/pin.svg").as_deref(), Some(SVG.as_bytes()));
        assert_eq!(icons.get("icons/plus.svg"), None);
        assert_eq!(icons.get("pin.svg"), None);
        // The app's asset loader draws it in the built-in icon's place; one
        // made apart from it, as each snapshot's is, still draws Kanso's.
        use gpui_kit::AssetSource as _;
        let app = crate::assets::AppAssets {
            icons: icons.clone(),
        };
        let drawn = app.load("icons/pin.svg").unwrap().unwrap();
        assert_eq!(drawn.as_ref(), SVG.as_bytes());
        let builtin = crate::assets::KansoAssets
            .load("icons/pin.svg")
            .unwrap()
            .unwrap();
        assert_ne!(builtin.as_ref(), SVG.as_bytes());
        assert_eq!(
            crate::assets::AppAssets::default()
                .load("icons/pin.svg")
                .unwrap()
                .unwrap(),
            builtin
        );
        icons.set(None);
        assert_eq!(icons.get("icons/pin.svg"), None);
        // A pack that has gone, or a name that can't be one, leaves Kanso's.
        assert_eq!(
            apply_at_launch(dir.path(), Some("gone")).get("icons/pin.svg"),
            None
        );
        assert_eq!(
            apply_at_launch(dir.path(), Some("../mine")).get("icons/pin.svg"),
            None
        );
        assert_eq!(apply_at_launch(dir.path(), None).get("icons/pin.svg"), None);
    }

    /// A body as large as a pack allows, and very many names for it: the
    /// names stop at the cap, and all of them share the one SVG rather than
    /// a copy each.
    #[test]
    fn aliases_share_their_icon_and_stop_at_the_cap() {
        let body = format!("<path d='{}'/>", "M1 1".repeat(50_000));
        let mut aliases = serde_json::Map::new();
        for n in 0..100_000 {
            aliases.insert(format!("a{n:06}"), serde_json::json!({"parent": "big"}));
        }
        let text = serde_json::json!({
            "icons": {"big": {"body": body}, "huge": {"body": "x".repeat(MAX_SVG_BYTES as usize)}},
            "aliases": aliases,
        })
        .to_string();
        let set: IconifyJson = serde_json::from_str(&text).unwrap();
        let (svgs, more) = iconify_svgs(&set, MAX_ICONS, MAX_PACK_BYTES);
        assert!(more);
        assert_eq!(svgs.len(), MAX_ICONS);
        let first = &svgs[0].1;
        assert!(svgs.iter().all(|(_, svg)| Arc::ptr_eq(svg, first)));
        // The body too large to draw is left out, whatever its name.
        assert!(svgs.iter().all(|(name, _)| name != "huge"));

        // Read as a pack: bounded too, and said so.
        let dir = tempfile::tempdir().unwrap();
        let root = packs_dir(dir.path());
        write(&root.join("many"), ICONIFY_FILE, &text);
        let packs = IconPacks::load(Some(&root));
        assert_eq!(packs.packs[0].icons.len(), MAX_ICONS);
        assert!(
            packs
                .notices
                .iter()
                .any(|n| n.file == "many" && n.message.contains("left out")),
            "{:?}",
            packs.notices
        );
    }

    /// More bytes than a pack may hold in all: the icons stop there.
    #[test]
    fn a_pack_stops_at_its_byte_budget() {
        let mut icons = serde_json::Map::new();
        for n in 0..10 {
            icons.insert(
                format!("i{n}"),
                serde_json::json!({"body": "y".repeat(1000)}),
            );
        }
        let set: IconifyJson =
            serde_json::from_value(serde_json::json!({ "icons": icons })).unwrap();
        let (svgs, more) = iconify_svgs(&set, 100, 3500);
        assert!(more);
        assert_eq!(svgs.len(), 3);
        let names: Vec<&str> = svgs.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, vec!["i0", "i1", "i2"]);
        assert!(!iconify_svgs(&set, 100, MAX_PACK_BYTES).1);
    }

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// An Iconify set of a few icons, some hidden or large, and aliases,
    /// some turned or pointing nowhere.
    fn iconify_set() -> impl Strategy<Value = IconifyJson> {
        let icon = (0usize..4, any::<bool>());
        let alias = (0usize..6, any::<bool>(), any::<bool>());
        (
            prop::collection::btree_map("[a-d]{1,2}", icon, 0..8),
            prop::collection::btree_map("[a-f]{1,2}", alias, 0..12),
        )
            .prop_map(|(icons, aliases)| {
                let parents = ["a", "b", "c", "d", "e", "f"];
                serde_json::from_value(serde_json::json!({
                    "icons": icons.into_iter().map(|(name, (size, hidden))| {
                        let body = "z".repeat([1, 10, 100, MAX_SVG_BYTES as usize][size]);
                        (name, serde_json::json!({"body": body, "hidden": hidden}))
                    }).collect::<serde_json::Map<_, _>>(),
                    "aliases": aliases.into_iter().map(|(name, (parent, turned, hidden))| {
                        (name, serde_json::json!({
                            "parent": parents[parent], "hFlip": turned, "hidden": hidden,
                        }))
                    }).collect::<serde_json::Map<_, _>>(),
                }))
                .unwrap()
            })
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: whatever the caps, the icons made are in name order,
        /// at most `limit` of them, each small enough to draw and within
        /// the byte budget counting each shared SVG once; an alias shares
        /// its icon's SVG; and a tighter cap gives the start of what a
        /// looser one gives, saying so when it leaves any out.
        #[test]
        fn prop_iconify_svgs_stay_within_their_caps(
            set in iconify_set(),
            limit in 0usize..20,
            budget in 0u64..2000,
        ) {
            let (all, none_left) = iconify_svgs(&set, usize::MAX, u64::MAX);
            prop_assert!(!none_left);
            let (svgs, more) = iconify_svgs(&set, limit, budget);
            prop_assert!(svgs.len() <= limit);
            prop_assert!(svgs.windows(2).all(|pair| pair[0].0 < pair[1].0));
            prop_assert_eq!(&all[..svgs.len()], &svgs[..]);
            prop_assert_eq!(more, svgs.len() < all.len());
            let mut seen: Vec<&Arc<[u8]>> = Vec::new();
            for (_, svg) in &svgs {
                prop_assert!(svg.len() as u64 <= MAX_SVG_BYTES);
                if !seen.iter().any(|other| Arc::ptr_eq(other, svg)) {
                    seen.push(svg);
                }
            }
            let spent: u64 = seen.iter().map(|svg| svg.len() as u64).sum();
            prop_assert!(spent <= budget);
            for (name, svg) in &svgs {
                if let Some(alias) = set.aliases.get(name) && !set.icons.contains_key(name) {
                    let parent = svgs.iter().find(|(other, _)| *other == alias.parent);
                    if let Some((_, parent)) = parent {
                        prop_assert!(Arc::ptr_eq(parent, svg));
                    }
                    prop_assert!(!alias.h_flip && !alias.hidden);
                }
            }
        }
    }
}

#[cfg(test)]
mod real_sets {
    /// A real Iconify set at `KANSO_ICONIFY_JSON` loads whole, and quickly.
    /// Run on demand.
    #[test]
    #[ignore = "reads a downloaded Iconify set"]
    fn a_downloaded_set_loads() {
        let source = std::env::var("KANSO_ICONIFY_JSON").expect("set KANSO_ICONIFY_JSON");
        let dir = tempfile::tempdir().unwrap();
        let pack = super::packs_dir(dir.path()).join("real");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::copy(&source, pack.join(super::ICONIFY_FILE)).unwrap();
        let started = std::time::Instant::now();
        let packs = super::IconPacks::load(Some(&super::packs_dir(dir.path())));
        let icons = packs.packs[0].icons.len();
        println!(
            "{icons} icons in {:?}; notices {:?}",
            started.elapsed(),
            packs.notices
        );
        assert!(icons > 1000);
    }
}
