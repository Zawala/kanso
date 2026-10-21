//! The icon themes Kanso knows, and the one in use.
//!
//! Built-in icon themes are compiled in from the repository's
//! `icon-themes/` folder: Lucide, Kanso's own look, and Phosphor. A
//! person's own are folders in the `icon-themes` folder of Kanso's data
//! folder (`icon_theme_file` reads one): read at launch, again whenever the
//! folder changes, and on Reload, each time off the UI thread. A theme that
//! can't be read is skipped with a notice that says why; it never stops the
//! others or the app.
//!
//! The theme in use draws every icon it has, in every window, at once:
//! GPUI Kit asks the hook set here for the path of each icon it draws
//! (`gpui_kit::component::set_icon_resolver`, added by Kanso's patch in
//! `vendor/`), and an icon the theme draws gets a path of its own,
//! `icons/@<key>/<name>.svg`, which [`crate::assets::KansoAssets`] loads
//! from the theme. A window keeps what it has drawn by path, so a new path
//! is drawn afresh and an unchanged one isn't: switching themes changes
//! the paths of the icons either theme draws and no others. The key holds
//! a fingerprint of the theme's SVGs, so a theme edited on disk draws anew
//! too.
//!
//! The library's own icons (a notebook, the Inbox, a pinned note) are
//! drawn through [`library_icon`], whose paths (`icons/library/<slot>.svg`)
//! the hook turns into the theme's icon for the slot, or the icon the slot
//! falls back to.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, SystemTime};

use gpui_kit::component::Icon;
use gpui_kit::{App, Global, SharedString, Task};

use crate::icon_theme_file::{self, Files, SlotSource, slot_source};
pub use crate::models::icon_theme::{IconTheme, IconThemeFile, Slot};
use crate::themes::Notice;
use crate::workspace::AppState;

/// A built-in icon theme as compiled in: its ID, its JSON and its SVGs by path.
type BuiltinTheme = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static [u8])],
);

include!(concat!(env!("OUT_DIR"), "/builtin_icon_themes.rs"));

/// The ID of the theme drawn when none is chosen: Kanso's own icons.
pub const DEFAULT_ID: &str = "lucide";
/// The start of every path a theme's icon is drawn under.
pub const THEMED_PREFIX: &str = "icons/@";
/// The start of the path of a library slot.
const SLOT_PREFIX: &str = "icons/library/";
/// The start of a path that draws Kanso's own icon whatever the theme in
/// use: `icons/~/search.svg`. Settings' previews draw each theme's icons
/// side by side with it.
pub const BUILTIN_PREFIX: &str = "icons/~/";
/// Most theme folders read.
pub const MAX_THEMES: usize = 64;
const WATCH_INTERVAL: Duration = Duration::from_secs(2);

/// The folder a person's own icon themes go in.
pub fn icon_themes_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("icon-themes")
}

// MARK: Paths

/// The path a library slot is drawn under, whatever the theme. Pure.
pub fn slot_path(slot: Slot) -> SharedString {
    format!("{SLOT_PREFIX}{}.svg", slot.key()).into()
}

/// An icon for library slot `slot`, drawn by the theme in use.
pub fn library_icon(slot: Slot) -> Icon {
    Icon::default().path(slot_path(slot))
}

/// The path `theme` draws its UI icon `name` under. Pure.
pub fn themed_path(theme: &IconTheme, name: &str) -> SharedString {
    format!("{THEMED_PREFIX}{}/{name}.svg", theme.key).into()
}

/// The path `theme` draws its icon for `slot` under. Pure.
pub fn themed_slot_path(theme: &IconTheme, slot: Slot) -> SharedString {
    format!("{THEMED_PREFIX}{}/library/{}.svg", theme.key, slot.key()).into()
}

/// The path `theme` draws its notebook icon `name` under. Pure.
pub fn extra_path(theme: &IconTheme, name: &str) -> SharedString {
    format!("{THEMED_PREFIX}{}/extra/{name}.svg", theme.key).into()
}

/// What each icon path is drawn as under `theme`: the theme's own path for
/// every UI icon it draws, and for every slot the path of what it shows.
/// A path not in it is drawn as it is. Pure.
pub fn themed_paths(theme: &IconTheme) -> HashMap<String, SharedString> {
    let mut paths: HashMap<String, SharedString> = theme
        .icons
        .keys()
        .map(|name| (format!("icons/{name}.svg"), themed_path(theme, name)))
        .collect();
    for slot in Slot::ALL {
        let drawn = match slot_source(theme, slot) {
            SlotSource::Library(at) => themed_slot_path(theme, at),
            SlotSource::Icon(name) => paths
                .get(&format!("icons/{name}.svg"))
                .cloned()
                .unwrap_or_else(|| format!("icons/{name}.svg").into()),
        };
        paths.insert(slot_path(slot).to_string(), drawn);
    }
    paths
}

/// `path`, as [`themed_paths`] gives it, made to draw as it is whatever
/// theme is in use: a theme's path already does; Kanso's own icon gets
/// [`BUILTIN_PREFIX`]. Pure.
pub fn unresolved(path: SharedString) -> SharedString {
    match path.strip_prefix("icons/") {
        Some(_) if path.starts_with(THEMED_PREFIX) => path,
        Some(rest) => format!("{BUILTIN_PREFIX}{rest}").into(),
        None => path,
    }
}

/// For a slot's path, the path of the built-in icon it falls back to,
/// for drawing it with no theme in use. Pure.
pub fn unthemed_slot_path(path: &str) -> Option<String> {
    let key = path.strip_prefix(SLOT_PREFIX)?.strip_suffix(".svg")?;
    let slot = Slot::parse(key)?;
    match slot_source(default_theme(), slot) {
        SlotSource::Icon(name) => Some(format!("icons/{name}.svg")),
        SlotSource::Library(_) => None,
    }
}

// MARK: Every theme read, by key

/// The themes whose icons can be loaded, by key, held weakly: a theme no
/// longer read is let go. Keys hold a fingerprint of the theme's SVGs, so
/// two apps (or tests) reading the same theme share an entry harmlessly.
fn registry() -> &'static Mutex<HashMap<String, Weak<IconTheme>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Weak<IconTheme>>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}

/// Makes `theme`'s icons loadable by their paths.
pub fn register(theme: &Arc<IconTheme>) {
    let mut registry = registry().lock().unwrap_or_else(|e| e.into_inner());
    registry.retain(|_, theme| theme.strong_count() > 0);
    registry.insert(theme.key.clone(), Arc::downgrade(theme));
}

/// The SVG at `path`, a path [`themed_path`], [`themed_slot_path`] or
/// [`extra_path`] made, while its theme is read.
pub fn themed_svg(path: &str) -> Option<Arc<[u8]>> {
    let rest = path.strip_prefix(THEMED_PREFIX)?;
    let (key, rest) = rest.split_once('/')?;
    let theme = registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(key)?
        .upgrade()?;
    let rest = rest.strip_suffix(".svg")?;
    if let Some(slot) = rest.strip_prefix("library/") {
        return theme.library.get(&Slot::parse(slot)?).cloned();
    }
    if let Some(name) = rest.strip_prefix("extra/") {
        return theme.extra(name).map(|icon| icon.svg.clone());
    }
    theme.icons.get(rest).cloned()
}

// MARK: The themes

/// The built-in themes, read once and kept for the life of the app.
// no-prop-test: fixed files; `builtin_icon_themes_load` reads every one.
pub fn builtins() -> &'static [Arc<IconTheme>] {
    static BUILT: OnceLock<Vec<Arc<IconTheme>>> = OnceLock::new();
    BUILT.get_or_init(|| {
        let mut themes: Vec<Arc<IconTheme>> = BUILTIN_FILES
            .iter()
            .filter_map(|(id, json, files)| {
                let file = match IconThemeFile::parse(json) {
                    Ok(file) => file,
                    Err(err) => {
                        // A test fails on this; a release never ships it.
                        eprintln!("kanso: the built-in icon theme {id} {err}");
                        return None;
                    }
                };
                let mut notices = Vec::new();
                let theme = icon_theme_file::load(id, &file, &Files(files), None, &mut notices);
                for notice in notices {
                    eprintln!("kanso: built-in icon theme {}", notice.sentence());
                }
                Some(Arc::new(theme))
            })
            .collect();
        // The default first.
        themes.sort_by_key(|theme| (theme.id != DEFAULT_ID, theme.name.to_lowercase()));
        for theme in &themes {
            register(theme);
        }
        themes
    })
}

/// Kanso's own icons: the theme that draws nothing over them.
pub fn default_theme() -> &'static Arc<IconTheme> {
    static EMPTY: OnceLock<Arc<IconTheme>> = OnceLock::new();
    builtins()
        .iter()
        .find(|theme| theme.id == DEFAULT_ID)
        .unwrap_or_else(|| {
            EMPTY.get_or_init(|| {
                let file = IconThemeFile {
                    name: "Lucide".into(),
                    ..Default::default()
                };
                Arc::new(icon_theme_file::load(
                    DEFAULT_ID,
                    &file,
                    &Files(&[]),
                    None,
                    &mut Vec::new(),
                ))
            })
        })
}

/// Every icon theme: the built-in ones, then a person's own by name.
#[derive(Default)]
pub struct IconThemes {
    pub entries: Vec<Arc<IconTheme>>,
    pub notices: Vec<Notice>,
    signature: u64,
    /// Counts reloads, so a view can tell its picture is stale.
    pub generation: u64,
}

impl Global for IconThemes {}

/// A fingerprint of every file in the icon themes folder, its size and
/// time, to tell when one changes.
fn signature(dir: &Path) -> u64 {
    fn walk(dir: &Path, depth: usize, files: &mut Vec<(PathBuf, u64, Option<SystemTime>)>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read.filter_map(Result::ok).take(4096) {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                if depth < 3 {
                    walk(&entry.path(), depth + 1, files);
                }
            } else {
                files.push((entry.path(), meta.len(), meta.modified().ok()));
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, 0, &mut files);
    files.sort();
    let mut print = icon_theme_file::Fingerprint::default();
    for (path, len, time) in files {
        print.field(path.to_string_lossy().as_bytes());
        print.field(&len.to_le_bytes());
        let nanos = time
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos());
        print.field(&nanos.to_le_bytes());
    }
    print.finish()
}

impl IconThemes {
    /// The built-in themes, and the ones in `dir` if given. Slow for large
    /// themes: read off the UI thread (`reload_soon`).
    pub fn load(dir: Option<&Path>) -> IconThemes {
        let mut entries: Vec<Arc<IconTheme>> = builtins().to_vec();
        let mut notices = Vec::new();
        let signature = dir.map(signature).unwrap_or_default();
        let mut own: Vec<Arc<IconTheme>> = Vec::new();
        if let Some(dir) = dir
            && let Ok(read) = std::fs::read_dir(dir)
        {
            let mut folders: Vec<PathBuf> = read
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                // An import in progress, or a system folder.
                .filter(|path| {
                    !path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                })
                .collect();
            folders.sort();
            if folders.len() > MAX_THEMES {
                notices.push(Notice {
                    file: "icon-themes".into(),
                    message: format!("holds more than {MAX_THEMES} themes; the rest are left out"),
                });
            }
            for folder in folders.into_iter().take(MAX_THEMES) {
                let Some(theme) = icon_theme_file::read_folder_safely(&folder, &mut notices) else {
                    continue;
                };
                let taken = entries.iter().chain(own.iter()).find(|other| {
                    other.id == theme.id || other.name.eq_ignore_ascii_case(&theme.name)
                });
                if let Some(other) = taken {
                    notices.push(Notice {
                        file: theme.id.clone(),
                        message: if other.id == theme.id {
                            format!(
                                "has the folder name of the built-in theme {}; rename the folder",
                                other.name
                            )
                        } else {
                            format!(
                                "is named \"{}\", like another icon theme; give it a name of its own",
                                theme.name
                            )
                        },
                    });
                    continue;
                }
                let theme = Arc::new(theme);
                register(&theme);
                own.push(theme);
            }
        }
        own.sort_by_key(|theme| theme.name.to_lowercase());
        entries.extend(own);
        IconThemes {
            entries,
            notices,
            signature,
            generation: 0,
        }
    }

    pub fn get(&self, id: &str) -> Option<&Arc<IconTheme>> {
        self.entries.iter().find(|theme| theme.id == id)
    }
}

/// The icon theme in use, with what each path it changes is drawn as.
pub struct ActiveIconTheme {
    pub theme: Arc<IconTheme>,
    paths: HashMap<String, SharedString>,
}

impl Global for ActiveIconTheme {}

impl ActiveIconTheme {
    pub fn new(theme: Arc<IconTheme>) -> Self {
        register(&theme);
        ActiveIconTheme {
            paths: themed_paths(&theme),
            theme,
        }
    }

    /// The path to draw for the icon at `path`, when the theme changes it.
    pub fn resolve(&self, path: &str) -> Option<SharedString> {
        self.paths.get(path).cloned()
    }
}

/// The theme in use, or Kanso's own before one is applied.
pub fn active(cx: &App) -> Arc<IconTheme> {
    cx.try_global::<ActiveIconTheme>()
        .map_or_else(|| default_theme().clone(), |active| active.theme.clone())
}

fn folder(cx: &App) -> Option<PathBuf> {
    cx.try_global::<AppState>()
        .map(|state| icon_themes_dir(&state.data_dir))
}

/// The theme the config chooses, by ID.
fn chosen(cx: &App) -> Option<String> {
    cx.try_global::<AppState>()
        .and_then(|state| state.config.appearance.icon_theme.clone())
}

/// Sets GPUI Kit's icon hook to draw with the theme in use, and draws with
/// the theme the config chooses, among the built-in ones until the folder
/// is read (`watch`). Before any window opens.
pub fn init(cx: &mut App) {
    gpui_kit::component::set_icon_resolver(cx, |path, cx| {
        cx.try_global::<ActiveIconTheme>()?.resolve(path)
    });
    if !cx.has_global::<IconThemes>() {
        cx.set_global(IconThemes::load(None));
    }
    apply(cx);
}

/// Draws with the theme the config chooses, or Kanso's own when it isn't
/// read (yet), in every window, now.
pub fn apply(cx: &mut App) {
    let theme = chosen(cx)
        .and_then(|id| cx.try_global::<IconThemes>()?.get(&id).cloned())
        .unwrap_or_else(|| default_theme().clone());
    let unchanged = cx
        .try_global::<ActiveIconTheme>()
        .is_some_and(|active| active.theme.key == theme.key);
    if !unchanged {
        cx.set_global(ActiveIconTheme::new(theme));
        cx.refresh_windows();
    }
}

/// Chooses the theme `id` (`None`: Kanso's own), saves it, and draws with
/// it everywhere.
pub fn choose(id: Option<String>, cx: &mut App) {
    let id = id.filter(|id| id != DEFAULT_ID);
    AppState::update_config(cx, |config| config.appearance.icon_theme = id);
    apply(cx);
}

/// Makes `themes` the ones read, counting the reload, and draws with the
/// chosen one.
fn show(mut themes: IconThemes, cx: &mut App) -> Vec<Notice> {
    themes.generation = cx
        .try_global::<IconThemes>()
        .map_or(0, |themes| themes.generation + 1);
    let notices = themes.notices.clone();
    cx.set_global(themes);
    apply(cx);
    cx.refresh_windows();
    notices
}

/// Reads the icon themes folder again off the UI thread, then draws with
/// the result. The task gives what couldn't be used.
pub fn reload_soon(cx: &mut App) -> Task<Vec<Notice>> {
    let dir = folder(cx);
    let read = cx
        .background_executor()
        .spawn(async move { IconThemes::load(dir.as_deref()) });
    cx.spawn(async move |cx| {
        let themes = read.await;
        cx.update(|cx| show(themes, cx))
    })
}

/// Reads the icon themes folder again, on this thread (tests, snapshots
/// and the import).
pub fn reload(cx: &mut App) -> Vec<Notice> {
    let themes = IconThemes::load(folder(cx).as_deref());
    show(themes, cx)
}

/// Reads the folder at launch, off the UI thread, and draws with the
/// chosen theme once read. Then checks the folder every couple of seconds,
/// also off the UI thread, and reads it again when a file is added,
/// changed or removed.
pub fn watch(cx: &mut App) {
    let first = reload_soon(cx);
    cx.spawn(async move |cx| {
        for notice in first.await {
            eprintln!("kanso: icon theme {}", notice.sentence());
        }
        loop {
            cx.background_executor().timer(WATCH_INTERVAL).await;
            let seen = cx.update(|cx| {
                let themes = cx.try_global::<IconThemes>()?;
                Some((folder(cx)?, themes.signature))
            });
            let Some((dir, seen)) = seen else {
                continue;
            };
            let now = cx
                .background_executor()
                .spawn(async move { signature(&dir) })
                .await;
            if now != seen {
                let before: Vec<Notice> = cx.update(|cx| {
                    cx.try_global::<IconThemes>()
                        .map(|t| t.notices.clone())
                        .unwrap_or_default()
                });
                let notices = cx.update(reload_soon).await;
                for notice in notices.iter().filter(|n| !before.contains(n)) {
                    eprintln!("kanso: icon theme {}", notice.sentence());
                }
            }
        }
    })
    .detach();
}

/// A few icons that show what a theme looks like, by path: UI icons and
/// library slots. Settings' preview row draws them under each theme.
pub const PREVIEW: &[&str] = &[
    "icons/library/notebook.svg",
    "icons/library/inbox.svg",
    "icons/library/note-pinned.svg",
    "icons/library/tag.svg",
    "icons/search.svg",
    "icons/settings.svg",
    "icons/calendar-days.svg",
    "icons/check.svg",
    "icons/trash.svg",
];

/// What `path` is drawn as under `theme`: the path [`themed_paths`] gives,
/// or `path` itself. Pure.
#[cfg(test)]
pub fn drawn_as(theme: &IconTheme, path: &str) -> SharedString {
    themed_paths(theme)
        .get(path)
        .cloned()
        .unwrap_or_else(|| SharedString::from(path.to_owned()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::icon_theme_file::load;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// Every built-in theme reads without a notice; Lucide draws nothing
    /// over Kanso's icons, and Phosphor draws every UI icon and most slots.
    #[test]
    fn builtin_icon_themes_load() {
        for (id, json, files) in BUILTIN_FILES {
            let file = IconThemeFile::parse(json).unwrap_or_else(|err| panic!("{id} {err}"));
            let mut notices = Vec::new();
            let theme = load(id, &file, &Files(files), None, &mut notices);
            assert!(notices.is_empty(), "{id}: {notices:?}");
            assert_eq!(
                theme.count(),
                file.icons.len() + file.library.len() + file.notebook_icons.len()
            );
            // Every SVG compiled in is used.
            let used: std::collections::HashSet<&str> = file
                .icons
                .values()
                .chain(file.library.values())
                .chain(file.notebook_icons.values().map(|e| &e.file))
                .map(String::as_str)
                .collect();
            for (path, _) in *files {
                assert!(used.contains(path), "{id}/{path} isn't used");
            }
        }
        let ids: Vec<&str> = builtins().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, vec!["lucide", "phosphor"]);
        assert_eq!(default_theme().count(), 0);
        let phosphor = &builtins()[1];
        assert_eq!(phosphor.license, "MIT");
        let all = crate::assets::app_icon_names();
        let missing: Vec<&String> = all
            .iter()
            .filter(|n| !phosphor.icons.contains_key(*n))
            .collect();
        assert!(missing.is_empty(), "Phosphor lacks {missing:?}");
        assert!(phosphor.library.len() >= 20);
        assert!(phosphor.extras.len() >= 20);
    }

    /// With no theme in use, a slot loads the built-in icon it falls back to.
    #[test]
    fn slots_load_without_a_theme() {
        use gpui_kit::AssetSource as _;
        let assets = crate::assets::KansoAssets;
        for slot in Slot::ALL {
            let bytes = assets.load(&slot_path(slot)).unwrap().unwrap();
            assert!(icon_theme_file::looks_like_svg(&bytes), "{slot:?}");
        }
        assert_eq!(
            assets.load("icons/library/inbox.svg").unwrap(),
            assets.load("icons/notebook.svg").unwrap()
        );
        assert!(assets.load("icons/library/nope.svg").is_err());
        assert!(assets.load("icons/@nope-0/search.svg").is_err());
    }

    /// A theme's paths load its SVGs, while it is read.
    #[test]
    fn themed_paths_load_the_themes_svgs() {
        use gpui_kit::AssetSource as _;
        let phosphor = builtins()[1].clone();
        let path = themed_path(&phosphor, "search");
        let loaded = crate::assets::KansoAssets.load(&path).unwrap().unwrap();
        assert_eq!(loaded.as_ref(), phosphor.icons["search"].as_ref());
        let open = themed_slot_path(&phosphor, Slot::NotebookOpen);
        assert_eq!(
            themed_svg(&open).unwrap(),
            phosphor.library[&Slot::NotebookOpen]
        );
        let extra = &phosphor.extras[0];
        assert_eq!(
            themed_svg(&extra_path(&phosphor, &extra.name)).unwrap(),
            extra.svg
        );
        // A theme let go can't be loaded any more.
        let gone = Arc::new(theme(&[("search", 1)], &[]));
        let path = themed_path(&gone, "search");
        register(&gone);
        assert!(themed_svg(&path).is_some());
        drop(gone);
        assert!(themed_svg(&path).is_none());
    }

    const UI: &[&str] = &[
        "search",
        "check",
        "chevron-down",
        "chevron-right",
        "notebook",
        "lock",
        "file-text",
        "minus",
    ];

    /// A theme drawing the UI icons and slots given, each SVG a byte that
    /// tells it apart.
    fn theme(icons: &[(&str, u8)], library: &[(Slot, u8)]) -> IconTheme {
        let svg = |b: u8| -> Arc<[u8]> { Arc::from(format!("<svg>{b}</svg>").into_bytes()) };
        let mut theme = IconTheme {
            id: "t".into(),
            name: "T".into(),
            author: String::new(),
            license: String::new(),
            source: None,
            description: None,
            icons: icons
                .iter()
                .map(|(n, b)| ((*n).to_owned(), svg(*b)))
                .collect(),
            library: library
                .iter()
                .map(|(s, b)| (*s, svg(*b)))
                .collect::<BTreeMap<_, _>>(),
            extras: Vec::new(),
            path: None,
            key: String::new(),
        };
        theme.key = icon_theme_file::theme_key(&theme);
        theme
    }

    fn any_theme(id: &'static str) -> impl Strategy<Value = IconTheme> {
        (
            prop::collection::btree_map(prop::sample::select(UI.to_vec()), 0u8..3, 0..6),
            prop::collection::btree_map(prop::sample::select(Slot::ALL.to_vec()), 0u8..3, 0..8),
        )
            .prop_map(move |(icons, library)| {
                let icons: Vec<(&str, u8)> = icons.into_iter().collect();
                let library: Vec<(Slot, u8)> = library.into_iter().collect();
                let mut t = theme(&icons, &library);
                t.id = id.into();
                t.key = icon_theme_file::theme_key(&t);
                t
            })
    }

    /// Every path the app may draw: the UI icons above and every slot.
    fn every_path() -> Vec<String> {
        UI.iter()
            .map(|n| format!("icons/{n}.svg"))
            .chain(Slot::ALL.iter().map(|s| slot_path(*s).to_string()))
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Resolution falls back icon by icon: a UI icon the theme draws is
        /// drawn from the theme, one it doesn't is drawn as it is; a slot is
        /// drawn from the theme's slot, else from what it falls back to,
        /// which the theme may draw in turn, else Kanso's own icon.
        #[test]
        fn prop_resolution_falls_back_per_icon(t in any_theme("a")) {
            let paths = themed_paths(&t);
            for name in UI {
                let drawn = drawn_as(&t, &format!("icons/{name}.svg"));
                if t.icons.contains_key(*name) {
                    prop_assert_eq!(drawn, themed_path(&t, name));
                } else {
                    prop_assert_eq!(drawn.as_ref(), format!("icons/{name}.svg"));
                }
            }
            for slot in Slot::ALL {
                let drawn = paths[slot_path(slot).as_ref()].clone();
                match slot_source(&t, slot) {
                    SlotSource::Library(at) => {
                        prop_assert!(t.library.contains_key(&at));
                        prop_assert_eq!(drawn, themed_slot_path(&t, at));
                    }
                    SlotSource::Icon(name) => {
                        prop_assert_eq!(drawn, drawn_as(&t, &format!("icons/{name}.svg")));
                    }
                }
                if t.library.contains_key(&slot) {
                    prop_assert_eq!(slot_source(&t, slot), SlotSource::Library(slot));
                }
            }
        }

        /// Switching from one theme to another changes the drawn path of
        /// every icon either theme draws (so no window keeps an old one),
        /// and of nothing else (so nothing else is drawn again).
        #[test]
        fn prop_switching_changes_every_themed_icon_and_nothing_else(a in any_theme("a"), b in any_theme("b")) {
            let draws = |t: &IconTheme, path: &str| drawn_as(t, path).contains(&format!("@{}/", t.key));
            for path in every_path() {
                let changed = drawn_as(&a, &path) != drawn_as(&b, &path);
                let themed = draws(&a, &path) || draws(&b, &path);
                prop_assert_eq!(changed, themed, "{}", path);
            }
            // Drawing with the same theme again changes nothing.
            for path in every_path() {
                prop_assert_eq!(drawn_as(&a, &path), drawn_as(&a.clone(), &path));
            }
        }

        /// A theme edited on disk gets a new key, so its icons draw anew.
        #[test]
        fn prop_a_changed_svg_changes_the_key(t in any_theme("a"), byte in 3u8..9) {
            prop_assume!(!t.icons.is_empty());
            let mut edited = t.clone();
            let first = edited.icons.keys().next().unwrap().clone();
            edited.icons.insert(first, Arc::from(format!("<svg>{byte}</svg>").into_bytes()));
            edited.key = icon_theme_file::theme_key(&edited);
            prop_assert_ne!(edited.key, t.key);
        }
    }

    /// A person's own themes come after the built-in ones by name; one
    /// that takes a built-in's folder name or another's name is skipped
    /// and said so.
    #[test]
    fn own_themes_load_after_the_builtins() {
        let dir = tempfile::tempdir().unwrap();
        let svg = r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/></svg>"#;
        for (folder, name) in [
            ("zest", "Zest"),
            ("apple", "Apple"),
            ("phosphor", "Mine"),
            ("zz-copy", "zest"),
        ] {
            let path = dir.path().join(folder);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                path.join("theme.json"),
                format!(r#"{{"name": "{name}", "icons": {{"search": "s.svg"}}}}"#),
            )
            .unwrap();
            std::fs::write(path.join("s.svg"), svg).unwrap();
        }
        let themes = IconThemes::load(Some(dir.path()));
        let names: Vec<&str> = themes.entries.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["Lucide", "Phosphor", "Apple", "Zest"]);
        let said: Vec<String> = themes.notices.iter().map(Notice::sentence).collect();
        assert!(
            said.iter()
                .any(|s| s.starts_with("phosphor has the folder name")),
            "{said:?}"
        );
        assert!(
            said.iter().any(|s| s.starts_with("zz-copy is named")),
            "{said:?}"
        );
        assert!(IconThemes::load(None).entries.len() == 2);
    }
}
