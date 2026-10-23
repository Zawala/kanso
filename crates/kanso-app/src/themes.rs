//! The themes Kanso knows, and the ones in use.
//!
//! Built-in themes are compiled in from `themes/`, the kanso-themes
//! repository checked out as a submodule.
//! A person's own themes are JSON files in the `themes` folder of Kanso's
//! data folder: read at launch, again whenever the folder changes, and on
//! Reload. A file that can't be read is skipped with a notice that says
//! why; it never stops the others or the app.
//!
//! Three themes are in use at once: the UI theme colors the
//! panes and controls (through the component library's theme too, so menus
//! and popovers match), the syntax theme the editor's Markdown and code, and
//! the preview theme the rendered note. The last two default to the UI
//! theme's own. Their colors are fitted to the UI theme's surfaces, so a
//! dark syntax theme under a light UI still reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use gpui_kit::component::highlighter::HighlightTheme;
use gpui_kit::component::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::{App, Global, Window};

use crate::config::{KANSO_DARK, KANSO_LIGHT, ThemeSettings};
use crate::origin::Origin;
use crate::theme::{self, Colors};
use crate::theme_file::{PreviewColors, ResolvedTheme, ThemeFile, fit_preview, fit_syntax};
use crate::workspace::AppState;

/// The built-in themes' files, in the order the Themes page shows them:
/// Kanso's own first, then each family's light theme beside its dark one.
pub const BUILTIN: &[(&str, &str)] = &[
    (
        "kanso-light.json",
        include_str!("../../../themes/kanso-light.json"),
    ),
    (
        "kanso-dark.json",
        include_str!("../../../themes/kanso-dark.json"),
    ),
    (
        "default-light.json",
        include_str!("../../../themes/default-light.json"),
    ),
    (
        "default-dark.json",
        include_str!("../../../themes/default-dark.json"),
    ),
    (
        "github-light.json",
        include_str!("../../../themes/github-light.json"),
    ),
    (
        "github-dark.json",
        include_str!("../../../themes/github-dark.json"),
    ),
    (
        "solarized-light.json",
        include_str!("../../../themes/solarized-light.json"),
    ),
    (
        "solarized-dark.json",
        include_str!("../../../themes/solarized-dark.json"),
    ),
    (
        "gruvbox-light.json",
        include_str!("../../../themes/gruvbox-light.json"),
    ),
    (
        "gruvbox-dark.json",
        include_str!("../../../themes/gruvbox-dark.json"),
    ),
    (
        "catppuccin-latte.json",
        include_str!("../../../themes/catppuccin-latte.json"),
    ),
    (
        "catppuccin-mocha.json",
        include_str!("../../../themes/catppuccin-mocha.json"),
    ),
    (
        "rose-pine-dawn.json",
        include_str!("../../../themes/rose-pine-dawn.json"),
    ),
    (
        "rose-pine.json",
        include_str!("../../../themes/rose-pine.json"),
    ),
    (
        "kanagawa-lotus.json",
        include_str!("../../../themes/kanagawa-lotus.json"),
    ),
    (
        "kanagawa-wave.json",
        include_str!("../../../themes/kanagawa-wave.json"),
    ),
    (
        "kanagawa-dragon.json",
        include_str!("../../../themes/kanagawa-dragon.json"),
    ),
    ("nord.json", include_str!("../../../themes/nord.json")),
    ("dracula.json", include_str!("../../../themes/dracula.json")),
    (
        "tokyo-night.json",
        include_str!("../../../themes/tokyo-night.json"),
    ),
];

/// How often the themes folder is checked for changes.
const WATCH_INTERVAL: Duration = Duration::from_secs(2);

/// One theme Kanso can show.
#[derive(Debug)]
pub struct ThemeEntry {
    /// The file stem (color themes have no other id).
    pub id: String,
    pub file: ThemeFile,
    pub resolved: ResolvedTheme,
    /// Where a theme was read from; `None` for a built-in one.
    pub path: Option<PathBuf>,
    /// Whether it is compiled in, the person's own, or from the gallery.
    pub origin: Origin,
}

impl ThemeEntry {
    pub fn new(
        file: ThemeFile,
        id: impl Into<String>,
        path: Option<PathBuf>,
        origin: Origin,
    ) -> ThemeEntry {
        let resolved = file.resolve();
        ThemeEntry {
            id: id.into(),
            file,
            resolved,
            path,
            origin,
        }
    }

    pub fn name(&self) -> &str {
        &self.file.name
    }

    pub fn dark(&self) -> bool {
        self.resolved.dark
    }
}

/// A theme file that couldn't be used, and why, in words for the Themes
/// page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub file: String,
    pub message: String,
}

impl Notice {
    pub fn sentence(&self) -> String {
        format!("{} {}.", self.file, self.message.trim_end_matches('.'))
    }
}

/// The built-in themes, read once.
// no-prop-test: fixed files; `builtin_themes_load` and the contrast tests
// read every one.
pub fn builtins() -> &'static [Arc<ThemeEntry>] {
    static BUILT: OnceLock<Vec<Arc<ThemeEntry>>> = OnceLock::new();
    BUILT.get_or_init(|| {
        BUILTIN
            .iter()
            .filter_map(|(file, text)| match ThemeFile::parse(text) {
                Ok(theme) => Some(Arc::new(ThemeEntry::new(
                    theme,
                    file.trim_end_matches(".json"),
                    None,
                    Origin::Builtin,
                ))),
                Err(err) => {
                    // A test fails on this; a release never ships it.
                    eprintln!("kanso: the built-in theme {file} {err}");
                    None
                }
            })
            .collect()
    })
}

/// Every theme: the built-in ones, then a person's own by name.
#[derive(Default)]
pub struct Themes {
    pub entries: Vec<Arc<ThemeEntry>>,
    pub notices: Vec<Notice>,
    /// The folder's files as last read, to tell when it changes.
    signature: Vec<(String, u64, Option<SystemTime>)>,
    /// Counts reloads, so a view can tell its picture is stale.
    pub generation: u64,
}

impl Global for Themes {}

/// The folder a person's own themes go in.
pub fn themes_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("themes")
}

/// The gallery tree, apart from the person's own folders.
pub fn gallery_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("gallery")
}

/// The `.json` files in `dir` with their sizes and times, sorted by name.
fn signature(dir: &Path) -> Vec<(String, u64, Option<SystemTime>)> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = read
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            meta.is_file().then(|| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    meta.len(),
                    meta.modified().ok(),
                )
            })
        })
        .collect();
    files.sort();
    files
}

/// The signatures of several folders at once, each prefixed so an edit in
/// one can't mask an edit in another. The gallery tree is included, so an
/// install, a manual edit or a removal hot-reloads within two seconds.
fn tree_signature(dirs: &[(&Path, &str)]) -> Vec<(String, u64, Option<SystemTime>)> {
    let mut files = Vec::new();
    for (dir, prefix) in dirs {
        for (name, len, time) in signature(dir) {
            files.push((format!("{prefix}{name}"), len, time));
        }
    }
    files.sort();
    files
}

/// Gallery themes that are disabled: a revoked theme whose copy was edited,
/// kept on disk but not loaded until the owner decides that case.
fn disabled(gallery: Option<&Path>) -> HashSet<String> {
    gallery
        .map(|root| kanso_gallery::State::load(&root.join("state.json")).disabled)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

impl Themes {
    /// The built-in themes, the ones in `dir` (the person's own), and the
    /// gallery's under `gallery`, if given.
    pub fn load(dir: Option<&Path>, gallery: Option<&Path>) -> Themes {
        let mut entries: Vec<Arc<ThemeEntry>> = builtins().to_vec();
        let mut notices = Vec::new();
        let gallery_themes = gallery.map(|root| root.join("themes"));
        let files_signature = tree_signature(&[
            (dir.unwrap_or(Path::new("")), ""),
            (
                gallery_themes.as_deref().unwrap_or(Path::new("")),
                "gallery/",
            ),
        ]);
        let disabled = disabled(gallery);
        let own_signature = dir.map(signature).unwrap_or_default();
        let mut own = Vec::new();
        if let Some(dir) = dir {
            for (name, _, _) in &own_signature {
                let path = dir.join(name);
                let parsed = std::fs::read_to_string(&path)
                    .map_err(|err| format!("couldn't be read: {err}"))
                    .and_then(|text| ThemeFile::parse(&text));
                match parsed {
                    Ok(theme) => {
                        let taken = entries
                            .iter()
                            .chain(own.iter())
                            .any(|e: &Arc<ThemeEntry>| e.name().eq_ignore_ascii_case(&theme.name));
                        if taken {
                            notices.push(Notice {
                                file: name.clone(),
                                message: format!(
                                    "is named \"{}\", like another theme; give it a name of its own",
                                    theme.name
                                ),
                            });
                        } else {
                            own.push(Arc::new(ThemeEntry::new(
                                theme,
                                name.trim_end_matches(".json"),
                                Some(path),
                                Origin::Own,
                            )));
                        }
                    }
                    Err(message) => notices.push(Notice {
                        file: name.clone(),
                        message,
                    }),
                }
            }
        }
        own.sort_by_key(|e| e.name().to_lowercase());
        entries.extend(own);
        // The gallery's themes load after the person's own, so a file of
        // their own with a gallery theme's name wins.
        if let Some(dir) = &gallery_themes {
            for (name, _, _) in signature(dir) {
                let id = name.trim_end_matches(".json").to_owned();
                if disabled.contains(&id) || !kanso_gallery::is_valid_name(&id) {
                    continue;
                }
                let path = dir.join(&name);
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(theme) = ThemeFile::parse(&text) else {
                    continue;
                };
                let taken = entries
                    .iter()
                    .any(|e: &Arc<ThemeEntry>| e.name().eq_ignore_ascii_case(&theme.name));
                if taken {
                    continue;
                }
                entries.push(Arc::new(ThemeEntry::new(
                    theme,
                    id,
                    Some(path),
                    Origin::Gallery,
                )));
            }
        }
        Themes {
            entries,
            notices,
            signature: files_signature,
            generation: 0,
        }
    }

    pub fn get(&self, name: &str) -> Option<&Arc<ThemeEntry>> {
        self.entries.iter().find(|e| e.name() == name)
    }

    /// The theme named `name`, or Kanso's own light or dark one when there is
    /// none by that name (a theme file removed, say).
    pub fn find(&self, name: &str, dark: bool) -> Arc<ThemeEntry> {
        self.get(name)
            .or_else(|| self.get(if dark { KANSO_DARK } else { KANSO_LIGHT }))
            .or_else(|| self.entries.first())
            .cloned()
            .unwrap_or_else(|| {
                let file = ThemeFile::parse(BUILTIN[usize::from(dark)].1).expect("Kanso's theme");
                Arc::new(ThemeEntry::new(
                    file,
                    BUILTIN[usize::from(dark)].0.trim_end_matches(".json"),
                    None,
                    Origin::Builtin,
                ))
            })
    }
}

/// A color theme being previewed across the whole window, before it is
/// installed. It is view state, never saved; `None` clears it.
#[derive(Default)]
pub struct PreviewTheme(pub Option<Arc<ThemeEntry>>);

impl Global for PreviewTheme {}

/// Shows `entry` across the window without writing a file or a setting.
/// Escape or [`stop_preview`] puts back the settings in force.
pub fn preview_theme(entry: Arc<ThemeEntry>, cx: &mut App) {
    cx.set_global(PreviewTheme(Some(entry)));
    apply(None, cx);
}

/// Ends a preview and shows the saved themes again.
pub fn stop_preview(cx: &mut App) {
    if cx
        .try_global::<PreviewTheme>()
        .is_some_and(|p| p.0.is_some())
    {
        cx.set_global(PreviewTheme(None));
        apply(None, cx);
    }
}

/// The themes in use, resolved for drawing.
pub struct ActiveTheme {
    pub ui: Arc<ThemeEntry>,
    pub colors: &'static Colors,
    /// The preview theme's colors, fitted to the UI's editor surface.
    pub preview: PreviewColors,
    /// The colors a preview's code blocks are drawn in, for exports too.
    pub code: Option<Arc<HighlightTheme>>,
}

impl Global for ActiveTheme {}

/// `colors` for the life of the app. Each distinct palette is kept once, so
/// switching back and forth doesn't grow memory.
fn intern(colors: Colors) -> &'static Colors {
    static KEPT: Mutex<Vec<&'static Colors>> = Mutex::new(Vec::new());
    let mut kept = KEPT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(found) = kept.iter().find(|c| ***c == colors) {
        return found;
    }
    let leaked: &'static Colors = Box::leak(Box::new(colors));
    kept.push(leaked);
    leaked
}

/// The settings in force: the config's, or the defaults before there is one.
fn settings(cx: &App) -> ThemeSettings {
    cx.try_global::<AppState>()
        .map(|state| state.config.appearance.themes.clone())
        .unwrap_or_default()
}

/// Reads the themes folder the first time it is needed.
fn ensure_loaded(cx: &mut App) {
    if cx.has_global::<Themes>() {
        return;
    }
    let (dir, gallery) = cx
        .try_global::<AppState>()
        .map(|state| (themes_dir(&state.data_dir), gallery_dir(&state.data_dir)))
        .unzip();
    let themes = Themes::load(dir.as_deref(), gallery.as_deref());
    for notice in &themes.notices {
        eprintln!("kanso: theme {}", notice.sentence());
    }
    cx.set_global(themes);
}

/// Shows the themes the settings choose, in every window: the panes, the
/// component library's controls, the editor's highlighting and the
/// preview. `window` says whether the system is light or dark; without one,
/// the app's appearance does.
pub fn apply(window: Option<&mut Window>, cx: &mut App) {
    ensure_loaded(cx);
    crate::markdown_source::ensure_registered();
    crate::code_lang::ensure_registered();
    let appearance = window
        .as_ref()
        .map(|window| window.appearance())
        .unwrap_or_else(|| cx.window_appearance());
    let system_dark = ThemeMode::from(appearance).is_dark();
    let settings = settings(cx);
    let themes = cx.global::<Themes>();
    // A preview colors the whole window with one theme's own UI, syntax and
    // preview colors, whatever the settings say, until it stops.
    let previewing = cx
        .try_global::<PreviewTheme>()
        .and_then(|preview| preview.0.clone());
    let (ui, syntax, preview_theme) = match previewing {
        Some(entry) => (entry.clone(), entry.clone(), entry),
        None => {
            let ui = themes.find(settings.ui_for(system_dark), system_dark);
            let syntax = themes.find(settings.syntax_for(system_dark), ui.dark());
            let preview = themes.find(settings.preview_for(system_dark), ui.dark());
            (ui, syntax, preview)
        }
    };

    let colors = intern(ui.resolved.colors);
    let fitted = fit_syntax(&syntax.resolved.syntax, colors.editor, colors.text);
    let config = theme::theme_config(&ui.resolved, colors, &fitted, theme::mono_font(cx));
    let config: Rc<ThemeConfig> = match serde_json::from_value(config) {
        Ok(config) => Rc::new(config),
        Err(err) => {
            eprintln!("kanso: couldn't build the theme {}: {err}", ui.name());
            return;
        }
    };
    let mode = if ui.dark() {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    {
        let theme = Theme::global_mut(cx);
        if ui.dark() {
            theme.dark_theme = config;
        } else {
            theme.light_theme = config;
        }
    }
    Theme::change(mode, window, cx);
    crate::zoom::apply_to_theme(cx);

    let preview = fit_preview(&preview_theme.resolved.preview, colors);
    let code = if preview_theme.name() != syntax.name() {
        let code = fit_syntax(
            &preview_theme.resolved.syntax,
            preview.code_background,
            colors.text,
        );
        let highlight = preview_code::build(&preview_theme.resolved, &code);
        if let Some(highlight) = &highlight {
            preview_code::install(highlight.clone(), cx);
        }
        highlight
    } else {
        preview_code::build(&syntax.resolved, &fitted)
    };
    cx.set_global(ActiveTheme {
        ui,
        colors,
        preview,
        code,
    });
    cx.refresh_windows();
}

/// The preview's colors in use; Kanso's own before a theme is applied.
pub fn preview_colors(cx: &App) -> PreviewColors {
    match cx.try_global::<ActiveTheme>() {
        Some(active) => active.preview,
        None => {
            let dark = Theme::global(cx).is_dark();
            builtins()[usize::from(dark)].resolved.preview
        }
    }
}

/// Reads the themes folder again and shows the result. Says whether any
/// file couldn't be used.
pub fn reload(cx: &mut App) -> Vec<Notice> {
    let (dir, gallery) = cx
        .try_global::<AppState>()
        .map(|state| (themes_dir(&state.data_dir), gallery_dir(&state.data_dir)))
        .unzip();
    let generation = cx
        .try_global::<Themes>()
        .map_or(0, |themes| themes.generation + 1);
    let mut themes = Themes::load(dir.as_deref(), gallery.as_deref());
    themes.generation = generation;
    let notices = themes.notices.clone();
    cx.set_global(themes);
    apply(None, cx);
    notices
}

/// Checks the themes folder every couple of seconds and reloads when a file
/// is added, changed or removed.
pub fn watch(cx: &mut App) {
    cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(WATCH_INTERVAL).await;
            let changed = cx.update(|cx| {
                let Some(state) = cx.try_global::<AppState>() else {
                    return false;
                };
                let dir = themes_dir(&state.data_dir);
                let gallery = gallery_dir(&state.data_dir).join("themes");
                let now = tree_signature(&[(&dir, ""), (&gallery, "gallery/")]);
                cx.try_global::<Themes>()
                    .is_some_and(|themes| themes.signature != now)
            });
            if changed {
                cx.update(|cx| {
                    let before: Vec<Notice> = cx
                        .try_global::<Themes>()
                        .map(|t| t.notices.clone())
                        .unwrap_or_default();
                    let notices = reload(cx);
                    for notice in notices.iter().filter(|n| !before.contains(n)) {
                        eprintln!("kanso: theme {}", notice.sentence());
                    }
                });
            }
        }
    })
    .detach();
}

/// Code blocks in the preview, colored by the preview theme when it isn't
/// the syntax theme. The component library colors them with the syntax
/// theme it was given; this replaces that highlighter in the text views'
/// defaults, once per theme, so code blocks keep their cached highlights
/// between frames.
mod preview_code {
    use std::cell::RefCell;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    use gpui_kit::App;
    use gpui_kit::base::TextViewDefaults;
    use gpui_kit::base::input::Rope;
    use gpui_kit::component::ThemeMode;
    use gpui_kit::component::highlighter::{
        HighlightTheme, HighlightThemeStyle, SyntaxHighlighter,
    };

    use crate::theme_file::{ResolvedTheme, TokenStyle};

    /// The highlight theme for `theme`'s syntax in `code`, or `None` when
    /// the colors can't be written out.
    // no-prop-test: maps theme colors onto the library's style; the
    // snapshots and `code_colors` show it.
    pub fn build(
        theme: &ResolvedTheme,
        code: &BTreeMap<&'static str, TokenStyle>,
    ) -> Option<Arc<HighlightTheme>> {
        let style: HighlightThemeStyle =
            match serde_json::from_value(crate::theme::highlight_style(&theme.colors, code)) {
                Ok(style) => style,
                Err(err) => {
                    eprintln!("kanso: couldn't build the preview's code colors: {err}");
                    return None;
                }
            };
        Some(Arc::new(HighlightTheme {
            name: theme.name.clone(),
            appearance: if theme.dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            style,
        }))
    }

    /// Hands `highlight` to the text views, for code blocks in the preview.
    // no-prop-test: hands the library a highlighter; the preview snapshots
    // show its colors.
    pub fn install(highlight: Arc<HighlightTheme>, cx: &mut App) {
        TextViewDefaults::global(cx)
            .with_code_block_highlighter(move |block| {
                thread_local! {
                    static HIGHLIGHTERS: RefCell<HashMap<&'static str, SyntaxHighlighter>> =
                        RefCell::new(HashMap::new());
                }
                let Some(lang) = block
                    .lang()
                    .and_then(|info| crate::code_lang::grammar(&info))
                else {
                    return Vec::new();
                };
                let code = block.code();
                HIGHLIGHTERS.with(|cache| {
                    let mut cache = cache.borrow_mut();
                    let highlighter = cache
                        .entry(lang)
                        .or_insert_with(|| SyntaxHighlighter::new(lang));
                    highlighter.update(None, &Rope::from_str(code.as_ref()), None);
                    highlighter.styles(&(0..code.len()), highlight.as_ref())
                })
            })
            .install(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::{BUILTIN, Themes, builtins};
    use crate::theme_file::{ThemeFile, contrast, mix};

    /// Every built-in file reads, under a name of its own, and says the
    /// truth about its appearance: a dark theme's text is lighter than its
    /// editor.
    #[test]
    fn builtin_themes_load() {
        assert_eq!(
            builtins().len(),
            BUILTIN.len(),
            "a built-in theme didn't parse"
        );
        for (file, text) in BUILTIN {
            let theme = ThemeFile::parse(text).unwrap_or_else(|e| panic!("{file} {e}"));
            assert!(!theme.license.is_empty(), "{file}: no license");
            assert!(!theme.author.is_empty(), "{file}: no author");
            let c = theme.resolve().colors;
            assert_eq!(
                crate::theme_file::luminance(c.text) > crate::theme_file::luminance(c.editor),
                theme.appearance.is_dark(),
                "{file}: appearance"
            );
            // Kept as written: the file is already in the canonical form.
            assert_eq!(
                theme.to_json_string(),
                *text,
                "{file} isn't formatted; run `kanso theme fmt`"
            );
        }
        let names: std::collections::HashSet<_> =
            builtins().iter().map(|e| e.name().to_lowercase()).collect();
        assert_eq!(
            names.len(),
            builtins().len(),
            "two built-in themes share a name"
        );
    }

    /// Every built-in theme keeps its text legible, as written (before
    /// any fitting): body text at 4.5:1 on every surface, secondary text at
    /// 3:1 and quiet text at 3:1 on the panes, links at 4.5:1, status colors
    /// and the caret at 3:1, button text at 4.5:1, selection visible with
    /// text still reading on it, selected rows visible, and every syntax
    /// and preview color reading on what it is drawn on.
    #[test]
    fn every_builtin_theme_reads() {
        let mut failures = Vec::new();
        for entry in builtins() {
            let name = entry.name();
            let t = &entry.resolved;
            let c = &t.colors;
            let mut want = |what: String, ratio: f64, min: f64| {
                if ratio < min {
                    failures.push(format!("{name}: {what} is {ratio:.2}, wants {min}"));
                }
            };
            let surfaces = [
                ("app", c.app),
                ("sidebar", c.sidebar),
                ("list", c.list),
                ("editor", c.editor),
                ("elevated", c.elevated),
                ("fill", c.fill),
                ("row_active", c.row_active),
                ("row_hover", c.row_hover),
                ("sidebar_active", c.sidebar_active),
                ("sidebar_hover", c.sidebar_hover),
                ("hover", c.hover),
                ("selected", c.selected),
                ("code", c.code),
            ];
            for (surface, bg) in surfaces {
                want(format!("text on {surface}"), contrast(c.text, bg), 4.5);
                want(
                    format!("text_secondary on {surface}"),
                    contrast(c.text_secondary, bg),
                    3.0,
                );
            }
            for (surface, bg) in [
                ("sidebar", c.sidebar),
                ("list", c.list),
                ("editor", c.editor),
                ("elevated", c.elevated),
            ] {
                want(
                    format!("text_muted on {surface}"),
                    contrast(c.text_muted, bg),
                    3.0,
                );
                want(
                    format!("link on {surface}"),
                    contrast(c.link, bg),
                    if surface == "sidebar" { 3.0 } else { 4.5 },
                );
                for (status, fg) in [
                    ("success", c.success),
                    ("warning", c.warning),
                    ("danger", c.danger),
                ] {
                    want(format!("{status} on {surface}"), contrast(fg, bg), 3.0);
                }
            }
            want("caret on editor".into(), contrast(t.caret, c.editor), 3.0);
            want(
                "button_text on button".into(),
                contrast(c.button_text, c.button),
                4.5,
            );
            let selection = t.selection.over(c.editor);
            want(
                "selection on editor".into(),
                contrast(selection, c.editor),
                1.15,
            );
            want("text on selection".into(), contrast(c.text, selection), 4.5);
            want(
                "row_active on list".into(),
                contrast(c.row_active, c.list),
                1.05,
            );
            want(
                "sidebar_active on sidebar".into(),
                contrast(c.sidebar_active, c.sidebar),
                1.05,
            );
            for (key, style) in &t.syntax {
                if let Some(color) = style.color {
                    want(
                        format!("syntax.{key} on editor"),
                        contrast(color, c.editor),
                        3.0,
                    );
                }
            }
            let p = &t.preview;
            want("preview text".into(), contrast(p.text, c.editor), 4.5);
            want("preview link".into(), contrast(p.link, c.editor), 4.5);
            want("preview quote".into(), contrast(p.quote, c.editor), 4.5);
            want(
                "preview finished tasks".into(),
                contrast(p.task_done, c.editor),
                3.0,
            );
            if let Some(heading) = p.heading {
                want("preview heading".into(), contrast(heading, c.editor), 3.0);
            }
            want(
                "preview text on code".into(),
                contrast(p.code_text.unwrap_or(p.text), p.code_background),
                4.5,
            );
            want(
                "preview text on inline code".into(),
                contrast(
                    p.inline_code_text.unwrap_or(p.text),
                    p.inline_code_background,
                ),
                4.5,
            );
            if let Some(head) = p.table_header_background {
                want(
                    "preview text on table header".into(),
                    contrast(p.text, head),
                    4.5,
                );
            }
            let preview_selection = match p.selection {
                Some(s) => s.over(c.editor),
                None => mix(c.accent, c.editor, 0.3),
            };
            want(
                "preview text on selection".into(),
                contrast(p.text, preview_selection),
                4.5,
            );
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Rewrites every file in the repository's `themes/` folder in the
    /// canonical form `builtin_themes_load` expects. Run with
    /// `KANSO_THEME_FMT=1 cargo test -p kanso-app format_theme_files -- --ignored`.
    #[test]
    #[ignore = "rewrites themes/*.json; run on demand"]
    fn format_theme_files() {
        if std::env::var_os("KANSO_THEME_FMT").is_none() {
            return;
        }
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "json") || path.ends_with("index.json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            match ThemeFile::parse(&text) {
                Ok(theme) => std::fs::write(&path, theme.to_json_string()).unwrap(),
                Err(err) => panic!("{} {err}", path.display()),
            }
        }
    }

    /// A folder with a good theme, a broken one, one named like a built-in
    /// and a stray file: the good one loads after the built-ins, and each
    /// bad one gets a notice that names its file and what is wrong.
    #[test]
    fn a_folder_of_themes_loads_with_notices() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, text: &str| std::fs::write(dir.path().join(name), text).unwrap();
        write(
            "mine.json",
            r##"{"name": "Mine", "appearance": "dark", "ui": {"editor": "#101010", "text": "#EEEEEE", "accent": "#88AAFF"}}"##,
        );
        write("broken.json", "{ not json");
        write(
            "typo.json",
            r##"{"name": "Typo", "appearance": "dark", "ui": {"editor": "#101010", "text": "#EEEEEE", "accent": "#88AAFF", "backgrund": "#000000"}}"##,
        );
        write(
            "copy.json",
            r##"{"name": "kanso dark", "appearance": "dark", "ui": {"editor": "#101010", "text": "#EEEEEE", "accent": "#88AAFF"}}"##,
        );
        write("notes.txt", "not a theme");
        let themes = Themes::load(Some(dir.path()), None);
        assert_eq!(themes.entries.len(), builtins().len() + 1);
        assert_eq!(themes.entries.last().unwrap().name(), "Mine");
        assert!(themes.entries.last().unwrap().path.is_some());
        let notices: Vec<_> = themes.notices.iter().map(|n| n.sentence()).collect();
        assert_eq!(notices.len(), 3, "{notices:?}");
        assert!(
            notices[0].starts_with("broken.json isn't valid JSON"),
            "{notices:?}"
        );
        assert!(notices[1].contains("like another theme"), "{notices:?}");
        assert!(notices[2].contains("`ui.backgrund`"), "{notices:?}");
        // A missing folder is no themes of one's own, and no notice.
        let none = Themes::load(Some(&dir.path().join("missing")), None);
        assert_eq!(none.entries.len(), builtins().len());
        assert!(none.notices.is_empty());
        // A theme that is gone falls back to Kanso's own.
        assert_eq!(themes.find("Gone", true).name(), "Kanso Dark");
        assert_eq!(themes.find("Gone", false).name(), "Kanso Light");
        assert_eq!(themes.find("Mine", false).name(), "Mine");
    }

    /// Gallery themes load after the person's own, and a disabled one (a
    /// revoked, edited copy) is kept on disk but never loaded.
    #[test]
    fn gallery_themes_load_after_own_themes_and_can_be_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let own = dir.path().join("themes");
        let gallery = dir.path().join("gallery");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::create_dir_all(gallery.join("themes")).unwrap();
        std::fs::write(
            own.join("mine.json"),
            r##"{"name": "Mine", "appearance": "dark", "ui": {"editor": "#101010", "text": "#EEEEEE", "accent": "#88AAFF"}}"##,
        )
        .unwrap();
        let nord = BUILTIN
            .iter()
            .find(|(file, _)| *file == "nord.json")
            .unwrap()
            .1
            .replace("\"name\": \"Nord\"", "\"name\": \"Gallery Nord\"");
        std::fs::write(gallery.join("themes/nord.json"), &nord).unwrap();

        let themes = Themes::load(Some(&own), Some(&gallery));
        let entry = themes.get("Gallery Nord").expect("the gallery theme loads");
        assert_eq!(entry.origin, crate::origin::Origin::Gallery);
        assert!(themes.get("Mine").is_some());
        // A selection that is removed falls back to Kanso's own.
        std::fs::remove_file(gallery.join("themes/nord.json")).unwrap();
        let reloaded = Themes::load(Some(&own), Some(&gallery));
        assert!(reloaded.get("Gallery Nord").is_none());
        assert_eq!(reloaded.find("Gallery Nord", true).name(), "Kanso Dark");

        std::fs::write(gallery.join("themes/nord.json"), &nord).unwrap();
        std::fs::write(
            gallery.join("state.json"),
            r#"{"installed":[],"pending":[],"disabled":["nord"]}"#,
        )
        .unwrap();
        let disabled = Themes::load(Some(&own), Some(&gallery));
        assert!(disabled.get("Gallery Nord").is_none());
    }
}
