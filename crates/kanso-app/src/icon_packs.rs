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
//! Its icons join the notebook icon picker (a notebook keeps one as
//! `<pack>/<icon>`). Chosen in Settings › Themes, a pack also draws the app's
//! own icons: an SVG named like one of them (`pin.svg`, `chevron-right.svg`)
//! takes its place everywhere, from the next launch, since the window keeps
//! the icons it has drawn. Icons are drawn in one color, the theme's, as the
//! built-in ones are.
//!
//! Packs are read at launch, again whenever the folder changes, and on
//! Reload. A pack or file that can't be used is skipped with a notice that
//! says why; it never stops the others or the app.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use gpui_kit::{App, Global};
use serde::Deserialize;

use crate::themes::Notice;
use crate::workspace::AppState;

/// Largest SVG a pack may hold.
pub const MAX_SVG_BYTES: u64 = 256 * 1024;
/// Most icons one pack may hold; the rest are skipped.
pub const MAX_ICONS: usize = 4000;
/// Most packs read.
pub const MAX_PACKS: usize = 64;
const WATCH_INTERVAL: Duration = Duration::from_secs(2);
const MANIFEST: &str = "pack.json";

/// The folder icon packs go in.
pub fn packs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("icon-packs")
}

/// One icon of a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackIcon {
    /// The file's name without `.svg`: lowercase letters, digits and dashes.
    pub name: String,
    pub label: String,
    /// More words the search finds it by, spaced.
    pub keywords: String,
    pub svg: Arc<[u8]>,
}

/// A pack of icons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconPack {
    /// The folder's name.
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub license: Option<String>,
    /// In name order.
    pub icons: Vec<PackIcon>,
}

impl IconPack {
    pub fn icon(&self, name: &str) -> Option<&PackIcon> {
        self.icons
            .binary_search_by(|icon| icon.name.as_str().cmp(name))
            .ok()
            .map(|ix| &self.icons[ix])
    }

    /// How many of the app's own icons this pack draws in their place.
    pub fn replaces(&self) -> usize {
        self.icons
            .iter()
            .filter(|icon| crate::assets::is_app_icon(&icon.name))
            .count()
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Manifest {
    name: Option<String>,
    author: Option<String>,
    license: Option<String>,
    icons: HashMap<String, IconWords>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct IconWords {
    label: Option<String>,
    keywords: Option<String>,
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
        if icons.len() == MAX_ICONS {
            notices.push(Notice {
                file: id.clone(),
                message: format!("has more than {MAX_ICONS} icons; the rest are left out"),
            });
            break;
        }
        let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        if size > MAX_SVG_BYTES {
            notices.push(Notice {
                file: where_,
                message: format!("is larger than {} KB", MAX_SVG_BYTES / 1024),
            });
            continue;
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
        let words = manifest.icons.get(&name);
        icons.push(PackIcon {
            label: words
                .and_then(|w| w.label.clone())
                .filter(|label| !label.trim().is_empty())
                .unwrap_or_else(|| label_from_name(&name)),
            keywords: words.and_then(|w| w.keywords.clone()).unwrap_or_default(),
            name,
            svg: Arc::from(bytes),
        });
    }
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    if icons.is_empty() {
        notices.push(Notice {
            file: id,
            message: "has no SVG icons".into(),
        });
        return None;
    }
    Some(IconPack {
        name: manifest
            .name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| label_from_name(&id)),
        author: manifest.author.filter(|a| !a.trim().is_empty()),
        license: manifest.license.filter(|l| !l.trim().is_empty()),
        id,
        icons,
    })
}

/// Every file in the packs folder with its size and time, to tell when it
/// changes.
fn signature(dir: &Path) -> Vec<(String, u64, Option<SystemTime>)> {
    let mut files = Vec::new();
    let Ok(packs) = std::fs::read_dir(dir) else {
        return files;
    };
    for pack in packs.filter_map(Result::ok) {
        let Ok(read) = std::fs::read_dir(pack.path()) else {
            continue;
        };
        for entry in read.filter_map(Result::ok) {
            if let Ok(meta) = entry.metadata() {
                files.push((
                    entry.path().to_string_lossy().into_owned(),
                    meta.len(),
                    meta.modified().ok(),
                ));
            }
        }
    }
    files.sort();
    files
}

/// The icon packs, as last read.
#[derive(Default)]
pub struct IconPacks {
    pub packs: Vec<Arc<IconPack>>,
    pub notices: Vec<Notice>,
    signature: Vec<(String, u64, Option<SystemTime>)>,
    /// Counts reloads, so a view can tell its picture is stale.
    pub generation: u64,
}

impl Global for IconPacks {}

impl IconPacks {
    /// The packs in `dir`, if given, by name.
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
                .collect();
            folders.sort();
            for folder in folders.into_iter().take(MAX_PACKS) {
                if let Some(pack) = read_pack(&folder, &mut notices) {
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

/// `<pack>/<icon>` as its two names, if it has that shape. Pure.
pub fn split_key(key: &str) -> Option<(&str, &str)> {
    let (pack, icon) = key.split_once('/')?;
    (is_valid_name(pack) && is_valid_name(icon)).then_some((pack, icon))
}

/// Reads the packs folder the first time it is needed.
pub fn ensure_loaded(cx: &mut App) {
    if cx.has_global::<IconPacks>() {
        return;
    }
    let dir = cx
        .try_global::<AppState>()
        .map(|state| packs_dir(&state.data_dir));
    let packs = IconPacks::load(dir.as_deref());
    for notice in &packs.notices {
        eprintln!("kanso: icon pack {}", notice.sentence());
    }
    cx.set_global(packs);
}

/// The packs, read if they haven't been.
pub fn packs(cx: &mut App) -> &IconPacks {
    ensure_loaded(cx);
    cx.global::<IconPacks>()
}

/// Reads the packs folder again, and says what couldn't be used.
pub fn reload(cx: &mut App) -> Vec<Notice> {
    let dir = cx
        .try_global::<AppState>()
        .map(|state| packs_dir(&state.data_dir));
    let generation = cx
        .try_global::<IconPacks>()
        .map_or(0, |packs| packs.generation + 1);
    let mut packs = IconPacks::load(dir.as_deref());
    packs.generation = generation;
    let notices = packs.notices.clone();
    cx.set_global(packs);
    cx.refresh_windows();
    notices
}

/// Checks the packs folder every couple of seconds and reloads when a file
/// is added, changed or removed.
pub fn watch(cx: &mut App) {
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(WATCH_INTERVAL).await;
            let changed = cx.update(|cx| {
                let Some(state) = cx.try_global::<AppState>() else {
                    return false;
                };
                let dir = packs_dir(&state.data_dir);
                cx.try_global::<IconPacks>()
                    .is_some_and(|packs| packs.signature != signature(&dir))
            });
            if changed {
                cx.update(|cx| {
                    reload(cx);
                });
            }
        }
    })
    .detach();
}

// MARK: The app's own icons

/// The pack drawing the app's own icons, by icon name, set once at launch.
static APP_ICONS: RwLock<Option<HashMap<String, Arc<[u8]>>>> = RwLock::new(None);
/// The ID of the pack drawing them since launch.
static IN_USE: RwLock<Option<String>> = RwLock::new(None);

/// The pack drawing the app's own icons since launch, by ID.
pub fn in_use() -> Option<String> {
    IN_USE.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Draws the app's own icons with `pack`'s, where it has them, or with the
/// built-in ones. Takes effect for icons not drawn yet: call it at launch.
pub fn use_for_app(pack: Option<&IconPack>) {
    let icons = pack.map(|pack| {
        pack.icons
            .iter()
            .map(|icon| (icon.name.clone(), icon.svg.clone()))
            .collect()
    });
    *APP_ICONS.write().unwrap_or_else(|e| e.into_inner()) = icons;
    *IN_USE.write().unwrap_or_else(|e| e.into_inner()) = pack.map(|pack| pack.id.clone());
}

/// The chosen pack's SVG for the asset at `path` (`icons/<name>.svg`), if it
/// draws that icon.
pub fn app_icon(path: &str) -> Option<Arc<[u8]>> {
    let name = path.strip_prefix("icons/")?.strip_suffix(".svg")?;
    APP_ICONS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()?
        .get(name)
        .cloned()
}

/// The pack the config chooses for the app's icons, read from `data_dir`,
/// for launch, before any window is drawn.
pub fn apply_at_launch(data_dir: &Path, chosen: Option<&str>) {
    let Some(chosen) = chosen else {
        return;
    };
    let mut notices = Vec::new();
    let pack = read_pack(&packs_dir(data_dir).join(chosen), &mut notices);
    if pack.is_none() {
        eprintln!("kanso: the icon pack {chosen} couldn't be used; drawing the built-in icons");
    }
    use_for_app(pack.as_ref());
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
        assert_eq!(p.replaces(), 1);
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
        apply_at_launch(dir.path(), Some("mine"));
        assert_eq!(app_icon("icons/pin.svg").as_deref(), Some(SVG.as_bytes()));
        assert_eq!(app_icon("icons/plus.svg"), None);
        // The asset loader draws it in the built-in icon's place.
        use gpui_kit::AssetSource as _;
        let drawn = crate::assets::KansoAssets
            .load("icons/pin.svg")
            .unwrap()
            .unwrap();
        assert_eq!(drawn.as_ref(), SVG.as_bytes());
        assert_eq!(app_icon("pin.svg"), None);
        use_for_app(None);
        assert_eq!(app_icon("icons/pin.svg"), None);
    }
}
