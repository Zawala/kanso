//! The icon pack gallery: open-source icon sets to browse and install from
//! inside Kanso, as an editor's extension view lists themes.
//!
//! The sets come from [Iconify](https://iconify.design): its list of sets
//! (name, author, licence, how many icons, a few samples), the samples
//! themselves, and each set whole as one `icons.json` from its npm package
//! on jsDelivr. Only one-color sets are listed, since Kanso draws icons in
//! the theme's color. Installing writes the set into the icon packs folder
//! (`<data>/icon-packs/<prefix>/`, with a `pack.json` saying where it came
//! from), where `icon_packs` reads it like any other pack; uninstalling
//! removes only packs the gallery installed.
//!
//! Nothing is fetched until the gallery opens, and nothing about the
//! library is sent: only the names of the sets and icons asked for.

use std::collections::{HashMap, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use serde::Deserialize;
use serde_json::json;

use crate::book_icons::matches;
use crate::icon_packs::{self, ICONIFY_FILE, IconifyJson, MANIFEST, MAX_JSON_BYTES};
use crate::theme::Colors;
use crate::workspace::AppState;

actions!(kanso, [BrowseIconPacks]);

/// Iconify's list of icon sets.
pub const COLLECTIONS_URL: &str = "https://api.iconify.design/collections";
/// Largest list of sets read.
const MAX_LIST_BYTES: u64 = 8 * 1024 * 1024;
/// Largest answer for a set's samples.
const MAX_SAMPLE_BYTES: u64 = 512 * 1024;
/// How many sets' samples are fetched at once.
const SAMPLE_WORKERS: usize = 4;
pub const GALLERY_WIDTH: f32 = 640.;
const LIST_HEIGHT: f32 = 460.;

/// The samples of set `prefix`, from Iconify's API.
pub fn samples_url(prefix: &str, names: &[String]) -> String {
    format!(
        "https://api.iconify.design/{prefix}.json?icons={}",
        names.join(",")
    )
}

/// Set `prefix` whole, from its npm package.
pub fn set_url(prefix: &str) -> String {
    format!("https://cdn.jsdelivr.net/npm/@iconify-json/{prefix}/icons.json")
}

/// One GET, its body up to `limit` bytes: [`HttpSource`] in the app, an
/// answer from memory in tests.
pub trait Source: Send + Sync {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, String>;
}

/// HTTPS only, every hop of a redirect too, with no cookies or credentials
/// and no identifying headers beyond a plain user agent.
pub struct HttpSource {
    agent: ureq::Agent,
}

impl Default for HttpSource {
    fn default() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(5)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_body(Some(Duration::from_secs(120)))
            .user_agent("Kanso")
            .build()
            .into();
        HttpSource { agent }
    }
}

impl Source for HttpSource {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, String> {
        let response = self.agent.get(url).call().map_err(|_| {
            "Couldn't reach the icon gallery. Check the connection and try again.".to_owned()
        })?;
        match response.status().as_u16() {
            200 => {}
            404 => return Err("The icon gallery doesn't have that set any more.".into()),
            code => {
                return Err(format!(
                    "The icon gallery answered {code}. Try again later."
                ));
            }
        }
        let mut body = Vec::new();
        response
            .into_body()
            .into_reader()
            .take(limit + 1)
            .read_to_end(&mut body)
            .map_err(|_| "The download stopped. Try again.".to_owned())?;
        if body.len() as u64 > limit {
            return Err("The download is larger than Kanso takes.".into());
        }
        Ok(body)
    }
}

/// Where the gallery fetches from.
pub struct Gallery(pub Arc<dyn Source>);

impl Global for Gallery {}

/// One icon set the gallery lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconSet {
    /// Iconify's name for it, and the pack's ID once installed.
    pub prefix: String,
    pub name: String,
    pub total: u32,
    pub author: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    /// A few of its icons' names.
    pub samples: Vec<String>,
}

#[derive(Deserialize)]
struct RawSet {
    name: String,
    #[serde(default)]
    total: u32,
    #[serde(default)]
    author: Option<RawNamed>,
    #[serde(default)]
    license: Option<RawLicense>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    samples: Vec<String>,
    #[serde(default)]
    palette: bool,
    #[serde(default)]
    hidden: bool,
}

#[derive(Deserialize)]
struct RawNamed {
    name: String,
}

#[derive(Deserialize)]
struct RawLicense {
    title: String,
}

/// The sets in Iconify's list that Kanso can draw: one color, not hidden,
/// with a prefix that can name a pack, by name. Pure.
pub fn parse_collections(bytes: &[u8]) -> Result<Vec<IconSet>, String> {
    let raw: HashMap<String, serde_json::Value> = serde_json::from_slice(bytes)
        .map_err(|_| "The icon gallery's list couldn't be read. Try again later.".to_owned())?;
    let mut sets: Vec<IconSet> = raw
        .into_iter()
        .filter(|(prefix, _)| icon_packs::is_valid_name(prefix))
        .filter_map(|(prefix, value)| {
            let set: RawSet = serde_json::from_value(value).ok()?;
            (!set.palette && !set.hidden).then(|| IconSet {
                prefix,
                name: set.name,
                total: set.total,
                author: set.author.map(|a| a.name),
                license: set.license.map(|l| l.title),
                category: set.category,
                samples: set
                    .samples
                    .into_iter()
                    .filter(|name| icon_packs::is_valid_name(name))
                    .take(6)
                    .collect(),
            })
        })
        .collect();
    sets.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.prefix.cmp(&b.prefix))
    });
    Ok(sets)
}

/// Whether a search for `query` finds `set`: by its name, prefix, author,
/// licence or category, as the icon picker's search finds icons. Pure.
pub fn set_matches(set: &IconSet, query: &str) -> bool {
    let words: Vec<String> = query
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let more = [
        set.author.as_deref(),
        set.license.as_deref(),
        set.category.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    matches(&words, &set.name, &set.prefix, &more)
}

/// The samples of `set`, drawn as SVGs.
pub fn fetch_samples(source: &dyn Source, set: &IconSet) -> Result<Vec<Arc<[u8]>>, String> {
    if set.samples.is_empty() {
        return Ok(Vec::new());
    }
    let bytes = source.get(&samples_url(&set.prefix, &set.samples), MAX_SAMPLE_BYTES)?;
    let found: IconifyJson =
        serde_json::from_slice(&bytes).map_err(|_| "The samples couldn't be read.".to_owned())?;
    let svgs: HashMap<String, String> = found.svgs().into_iter().collect();
    Ok(set
        .samples
        .iter()
        .filter_map(|name| svgs.get(name))
        .map(|svg| Arc::from(svg.as_bytes()))
        .collect())
}

/// What the pack folder `dir` holds: nothing, a pack the gallery installed
/// from `prefix`, or someone's own pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Installed {
    No,
    FromGallery,
    OwnPack,
}

pub fn installed(dir: &Path, prefix: &str) -> Installed {
    if !dir.exists() {
        return Installed::No;
    }
    let source = std::fs::read(dir.join(MANIFEST))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|manifest| {
            manifest
                .pointer("/source/iconify")
                .and_then(|p| p.as_str())
                .map(str::to_owned)
        });
    if source.as_deref() == Some(prefix) {
        Installed::FromGallery
    } else {
        Installed::OwnPack
    }
}

/// Downloads `set` whole into the packs folder `packs`, as a pack named
/// with its prefix, replacing one the gallery installed before. A partial
/// download never takes its place: the set is written beside it and moved
/// in once whole.
pub fn install(source: &dyn Source, packs: &Path, set: &IconSet) -> Result<(), String> {
    let target = packs.join(&set.prefix);
    if installed(&target, &set.prefix) == Installed::OwnPack {
        return Err(format!(
            "The icon packs folder already has a pack named {}; rename or remove it first.",
            set.prefix
        ));
    }
    let bytes = source.get(&set_url(&set.prefix), MAX_JSON_BYTES)?;
    let parsed: IconifyJson = serde_json::from_slice(&bytes)
        .map_err(|_| format!("{} didn't download whole. Try again.", set.name))?;
    if parsed.icons.is_empty() {
        return Err(format!("{} has no icons Kanso can use.", set.name));
    }
    let manifest = json!({
        "name": set.name,
        "author": set.author,
        "license": set.license,
        "source": {"iconify": set.prefix},
    });
    let fail = |err: std::io::Error| format!("Couldn't save {}: {err}", set.name);
    // Folders starting with a dot are passed over when packs are read.
    let staging: PathBuf = packs.join(format!(".installing-{}", set.prefix));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(fail)?;
    std::fs::write(staging.join(ICONIFY_FILE), &bytes).map_err(fail)?;
    std::fs::write(
        staging.join(MANIFEST),
        serde_json::to_vec_pretty(&manifest).expect("JSON serializes"),
    )
    .map_err(fail)?;
    if target.exists() {
        std::fs::remove_dir_all(&target).map_err(fail)?;
    }
    std::fs::rename(&staging, &target).map_err(fail)
}

/// Removes the pack the gallery installed from `prefix`; someone's own pack
/// of that name is left alone.
pub fn uninstall(packs: &Path, prefix: &str) -> Result<(), String> {
    let target = packs.join(prefix);
    match installed(&target, prefix) {
        Installed::No => Ok(()),
        Installed::OwnPack => Err(format!(
            "{prefix} wasn't installed from the gallery; remove it from the icon packs folder instead."
        )),
        Installed::FromGallery => std::fs::remove_dir_all(&target)
            .map_err(|err| format!("Couldn't remove {prefix}: {err}")),
    }
}

// MARK: The view

/// What the gallery shows.
enum List {
    Loading,
    Failed(String),
    Ready(Vec<IconSet>),
}

/// A set being installed or removed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Busy {
    Installing,
    Removing,
}

/// The gallery: a search over the sets, each with its samples and an
/// Install or Uninstall button.
pub struct IconGallery {
    search: Entity<InputState>,
    list: List,
    samples: HashMap<String, Vec<Arc<[u8]>>>,
    busy: HashMap<String, Busy>,
    scroll: ScrollHandle,
    _subscription: Subscription,
    _tasks: Vec<Task<()>>,
}

/// Fetches nothing: without a [`Gallery`] (tests, snapshots) the network is
/// never reached.
struct Offline;

impl Source for Offline {
    fn get(&self, _: &str, _: u64) -> Result<Vec<u8>, String> {
        Err("Couldn't reach the icon gallery. Check the connection and try again.".into())
    }
}

fn source(cx: &App) -> Arc<dyn Source> {
    cx.try_global::<Gallery>().map_or_else(
        || Arc::new(Offline) as Arc<dyn Source>,
        |gallery| gallery.0.clone(),
    )
}

fn packs_dir(cx: &App) -> PathBuf {
    icon_packs::packs_dir(&cx.global::<AppState>().data_dir)
}

/// Runs `work` on a thread of its own and hands its result to `done` on
/// the gallery.
fn in_background<T: Send + 'static>(
    cx: &mut Context<IconGallery>,
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(&mut IconGallery, T, &mut Context<IconGallery>) + 'static,
) -> Task<()> {
    let (tx, rx) = async_channel::bounded(1);
    std::thread::Builder::new()
        .name("kanso-icons".into())
        .spawn(move || {
            tx.try_send(work()).ok();
        })
        .ok();
    cx.spawn(async move |this, cx| {
        if let Ok(value) = rx.recv().await {
            this.update(cx, |this, cx| {
                done(this, value, cx);
                cx.notify();
            })
            .ok();
        }
    })
}

impl IconGallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icon packs"));
        let subscription = cx.subscribe(&search, |this: &mut IconGallery, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.scroll.set_offset(point(px(0.), px(0.)));
                cx.notify();
            }
        });
        let mut gallery = IconGallery {
            search,
            list: List::Loading,
            samples: HashMap::new(),
            busy: HashMap::new(),
            scroll: ScrollHandle::new(),
            _subscription: subscription,
            _tasks: Vec::new(),
        };
        gallery.load(cx);
        gallery
    }

    /// Fetches the list of sets, then their samples.
    fn load(&mut self, cx: &mut Context<Self>) {
        self.list = List::Loading;
        let source = source(cx);
        let task = in_background(
            cx,
            move || {
                source
                    .get(COLLECTIONS_URL, MAX_LIST_BYTES)
                    .and_then(|bytes| parse_collections(&bytes))
            },
            |this, sets, cx| match sets {
                Ok(sets) => {
                    this.fetch_samples(&sets, cx);
                    this.list = List::Ready(sets);
                }
                Err(words) => this.list = List::Failed(words),
            },
        );
        self._tasks.push(task);
    }

    /// Fetches each set's samples, a few sets at a time, showing them as
    /// they come.
    fn fetch_samples(&mut self, sets: &[IconSet], cx: &mut Context<Self>) {
        let source = source(cx);
        let queue = Arc::new(std::sync::Mutex::new(sets.to_vec()));
        let (tx, rx) = async_channel::unbounded::<(String, Vec<Arc<[u8]>>)>();
        for _ in 0..SAMPLE_WORKERS {
            let (queue, source, tx) = (queue.clone(), source.clone(), tx.clone());
            std::thread::Builder::new()
                .name("kanso-icons".into())
                .spawn(move || {
                    loop {
                        let next = queue.lock().ok().and_then(|mut q| q.pop());
                        let Some(set) = next else { break };
                        if let Ok(svgs) = fetch_samples(source.as_ref(), &set)
                            && tx.send_blocking((set.prefix, svgs)).is_err()
                        {
                            break;
                        }
                    }
                })
                .ok();
        }
        self._tasks.push(cx.spawn(async move |this, cx| {
            while let Ok((prefix, svgs)) = rx.recv().await {
                let kept = this.update(cx, |this, cx| {
                    this.samples.insert(prefix, svgs);
                    cx.notify();
                });
                if kept.is_err() {
                    break;
                }
            }
        }));
    }

    /// The sets the search finds.
    fn shown(&self, cx: &App) -> Vec<IconSet> {
        let query = self.search.read(cx).value();
        match &self.list {
            List::Ready(sets) => sets
                .iter()
                .filter(|set| set_matches(set, &query))
                .cloned()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Installs `set`, then reads the packs again.
    pub fn install(&mut self, set: IconSet, window: &mut Window, cx: &mut Context<Self>) {
        self.run(set, Busy::Installing, window, cx);
    }

    /// Removes the pack installed from `set`, then reads the packs again.
    pub fn uninstall(&mut self, set: IconSet, window: &mut Window, cx: &mut Context<Self>) {
        self.run(set, Busy::Removing, window, cx);
    }

    fn run(&mut self, set: IconSet, busy: Busy, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.contains_key(&set.prefix) {
            return;
        }
        self.busy.insert(set.prefix.clone(), busy);
        let (source, packs) = (source(cx), packs_dir(cx));
        let prefix = set.prefix.clone();
        let name = set.name.clone();
        let handle = window.window_handle();
        let task = in_background(
            cx,
            move || match busy {
                Busy::Installing => install(source.as_ref(), &packs, &set),
                Busy::Removing => uninstall(&packs, &set.prefix),
            },
            move |this, result, cx| {
                this.busy.remove(&prefix);
                icon_packs::reload(cx);
                let notification = match (result, busy) {
                    (Ok(()), Busy::Installing) => Notification::success(format!(
                        "{name} is installed. Its icons are in each notebook's Change Icon…"
                    )),
                    (Ok(()), Busy::Removing) => {
                        Notification::info(format!("{name} is uninstalled."))
                    }
                    (Err(words), _) => Notification::error(words),
                };
                handle
                    .update(cx, |_, window, cx| {
                        window.push_notification(notification, cx)
                    })
                    .ok();
            },
        );
        self._tasks.push(task);
        cx.notify();
    }

    /// Whether the list has loaded, for tests.
    #[cfg(test)]
    pub fn is_ready(&self) -> bool {
        matches!(self.list, List::Ready(_))
    }

    /// The prefixes of the sets showing, for tests.
    #[cfg(test)]
    pub fn shown_prefixes(&self, cx: &App) -> Vec<String> {
        self.shown(cx).into_iter().map(|set| set.prefix).collect()
    }

    /// Types `query` into the search (tests).
    #[cfg(test)]
    pub fn search_for(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        let query = query.to_owned();
        self.search
            .update(cx, |search, cx| search.set_value(query, window, cx));
    }

    /// How many sets' samples have come (snapshots).
    #[cfg(test)]
    pub fn samples_loaded(&self) -> usize {
        self.samples.len()
    }

    /// Whether anything is being installed or removed (tests).
    #[cfg(test)]
    pub fn is_busy(&self) -> bool {
        !self.busy.is_empty()
    }

    /// The set with `prefix`, once listed (tests).
    #[cfg(test)]
    pub fn set(&self, prefix: &str) -> Option<IconSet> {
        match &self.list {
            List::Ready(sets) => sets.iter().find(|s| s.prefix == prefix).cloned(),
            _ => None,
        }
    }

    fn render_row(
        &self,
        set: &IconSet,
        installed_ids: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let samples = self.samples.get(&set.prefix);
        let tiles = (0..6).map(|ix| {
            let svg = samples.and_then(|s| s.get(ix)).cloned();
            div()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .children(svg.map(|svg| {
                    Icon::default()
                        .data(&svg)
                        .size(px(16.))
                        .text_color(colors.text_secondary)
                }))
        });
        let tiles: Vec<_> = tiles.collect();
        let mut tiles = tiles.into_iter();
        let preview = v_flex()
            .flex_none()
            .size(px(64.))
            .rounded(px(10.))
            .bg(colors.sidebar)
            .items_center()
            .justify_center()
            .gap(px(2.))
            .child(h_flex().gap(px(2.)).children(tiles.by_ref().take(3)))
            .child(h_flex().gap(px(2.)).children(tiles));
        let installed = installed_ids.contains(&set.prefix);
        let busy = self.busy.get(&set.prefix).copied();
        let action_set = set.clone();
        let prefix = set.prefix.clone();
        let button = Button::new(SharedString::from(format!("icon-set-{prefix}")))
            .small()
            .debug_selector(move || format!("icon-set-{prefix}"))
            .map(|button| match (busy, installed) {
                (Some(Busy::Installing), _) => button.primary().label("Installing…").loading(true),
                (Some(Busy::Removing), _) => button.label("Removing…").loading(true),
                (None, true) => button.outline().label("Uninstall"),
                (None, false) => button.primary().label("Install"),
            })
            .disabled(busy.is_some())
            .on_click(cx.listener(move |this, _, window, cx| {
                if installed {
                    this.uninstall(action_set.clone(), window, cx);
                } else {
                    this.install(action_set.clone(), window, cx);
                }
            }));
        let byline = [set.author.clone(), set.license.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        let summary = match &set.category {
            Some(category) => format!("{} icons · {category}", group_digits(set.total)),
            None => format!("{} icons", group_digits(set.total)),
        };
        h_flex()
            .gap(px(14.))
            .py(px(10.))
            .border_b_1()
            .border_color(colors.divider)
            .child(preview)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(
                        h_flex()
                            .gap(px(8.))
                            .child(
                                div()
                                    .text_size(px(14.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors.text)
                                    .truncate()
                                    .child(set.name.clone()),
                            )
                            .when(installed, |el| {
                                el.child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(colors.success)
                                        .child("Installed"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(colors.text_secondary)
                            .truncate()
                            .child(summary),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .truncate()
                            .child(byline),
                    ),
            )
            .child(div().flex_none().child(button))
            .into_any_element()
    }
}

/// 6238 as "6,238". Pure.
pub fn group_digits(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (ix, c) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

// no-prop-test: drawing; `parse_collections` and `set_matches` are tested,
// and the snapshot shows the gallery.
impl Render for IconGallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let installed_ids: HashSet<String> = icon_packs::packs(cx)
            .packs
            .iter()
            .filter_map(|pack| pack.iconify.clone())
            .collect();
        let shown = self.shown(cx);
        let count = shown.len();
        let body: AnyElement = match &self.list {
            List::Loading => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(colors.text_muted)
                .child("Loading icon packs…")
                .into_any_element(),
            List::Failed(words) => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_3()
                .text_color(colors.text_muted)
                .child(words.clone())
                .child(
                    Button::new("icon-gallery-retry")
                        .small()
                        .outline()
                        .label("Try Again")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.load(cx);
                            cx.notify();
                        })),
                )
                .into_any_element(),
            List::Ready(_) if shown.is_empty() => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(colors.text_muted)
                .child("No icon packs match. Try another word.")
                .into_any_element(),
            List::Ready(_) => {
                let rows: Vec<AnyElement> = shown
                    .iter()
                    .map(|set| self.render_row(set, &installed_ids, cx))
                    .collect();
                v_flex().children(rows).into_any_element()
            }
        };
        v_flex()
            .gap(px(10.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .child("Open-source icon sets from Iconify. Kanso draws icons in your theme's color, so each set matches it."),
            )
            .child(Input::new(&self.search).small())
            .when(matches!(self.list, List::Ready(_)), |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_muted)
                        .child(match count {
                            1 => "1 icon pack".to_owned(),
                            n => format!("{n} icon packs"),
                        }),
                )
            })
            .child(
                div()
                    .id("icon-gallery-list")
                    .h(px(LIST_HEIGHT))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(body),
            )
    }
}

/// Opens the gallery, with the keyboard in its search. Returns it.
pub fn open_gallery(window: &mut Window, cx: &mut App) -> Entity<IconGallery> {
    icon_packs::ensure_loaded(cx);
    let gallery = cx.new(|cx| IconGallery::new(window, cx));
    let search = gallery.read(cx).search.clone();
    let first_build = std::cell::Cell::new(true);
    let view = gallery.clone();
    let dir = packs_dir(cx);
    let dir = Rc::new(dir);
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            search.update(cx, |search, cx| search.focus(window, cx));
        }
        let dir = dir.clone();
        alert
            .title("Icon Packs")
            .width(px(GALLERY_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("icon-gallery-folder")
                            .small()
                            .label("Open Folder")
                            .on_click(move |_, _, cx| {
                                std::fs::create_dir_all(dir.as_ref()).ok();
                                cx.open_with_system(dir.as_ref());
                            }),
                    )
                    .child(
                        Button::new("icon-gallery-done")
                            .small()
                            .primary()
                            .label("Done")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
    });
    gallery
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use gpui_kit::{AppContext as _, Context, IntoElement, Render, Window, div};

    use super::{
        COLLECTIONS_URL, Gallery, IconGallery, Installed, Source, fetch_samples, group_digits,
        install, installed, open_gallery, parse_collections, samples_url, set_matches, set_url,
        uninstall,
    };
    use crate::icon_packs;
    use crate::workspace::AppState;

    /// Answers from memory, and remembers what was asked.
    #[derive(Default)]
    pub struct FakeSource {
        pub answers: Mutex<HashMap<String, Result<Vec<u8>, String>>>,
        pub asked: Mutex<Vec<String>>,
    }

    impl FakeSource {
        pub fn answer(&self, url: &str, body: &str) {
            self.answers
                .lock()
                .unwrap()
                .insert(url.to_owned(), Ok(body.as_bytes().to_vec()));
        }
    }

    impl Source for FakeSource {
        fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, String> {
            self.asked.lock().unwrap().push(url.to_owned());
            let answer = self
                .answers
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .unwrap_or_else(|| Err("offline".into()))?;
            if answer.len() as u64 > limit {
                return Err("too large".into());
            }
            Ok(answer)
        }
    }

    pub const COLLECTIONS: &str = r#"{
        "ph": {"name": "Phosphor", "total": 9072, "author": {"name": "Phosphor Icons"},
               "license": {"title": "MIT"}, "samples": ["acorn", "anchor"], "category": "UI Other"},
        "twemoji": {"name": "Twitter Emoji", "total": 3000, "palette": true},
        "old": {"name": "Old Set", "total": 10, "hidden": true},
        "Bad Prefix": {"name": "Bad", "total": 1},
        "tabler": {"name": "Tabler Icons", "total": 6238, "author": {"name": "Paweł Kuna"},
                   "license": {"title": "MIT"}, "category": "UI 24px"}
    }"#;

    pub const PHOSPHOR: &str = r#"{"prefix": "ph", "width": 256, "height": 256,
        "icons": {"acorn": {"body": "<path d='M1 1'/>"}, "anchor": {"body": "<path d='M2 2'/>"}}}"#;

    pub fn fake() -> Arc<FakeSource> {
        let source = FakeSource::default();
        source.answer(COLLECTIONS_URL, COLLECTIONS);
        source.answer(
            &samples_url("ph", &["acorn".into(), "anchor".into()]),
            PHOSPHOR,
        );
        source.answer(&set_url("ph"), PHOSPHOR);
        Arc::new(source)
    }

    #[test]
    fn the_list_keeps_one_color_sets_by_name() {
        let sets = parse_collections(COLLECTIONS.as_bytes()).unwrap();
        let prefixes: Vec<&str> = sets.iter().map(|s| s.prefix.as_str()).collect();
        assert_eq!(prefixes, vec!["ph", "tabler"]);
        assert_eq!(sets[0].author.as_deref(), Some("Phosphor Icons"));
        assert_eq!(sets[0].license.as_deref(), Some("MIT"));
        assert!(parse_collections(b"nope").is_err());
        assert!(set_matches(&sets[0], "phos"));
        assert!(set_matches(&sets[1], "kuna"));
        assert!(set_matches(&sets[1], "24px"));
        assert!(!set_matches(&sets[0], "tabler"));
        assert_eq!(group_digits(6238), "6,238");
        assert_eq!(group_digits(15728), "15,728");
        assert_eq!(group_digits(1234567), "1,234,567");
        assert_eq!(group_digits(42), "42");
    }

    #[test]
    fn installing_writes_a_pack_the_packs_folder_reads_and_uninstalling_removes_it() {
        let source = fake();
        let dir = tempfile::tempdir().unwrap();
        let packs = icon_packs::packs_dir(dir.path());
        let sets = parse_collections(COLLECTIONS.as_bytes()).unwrap();
        let ph = &sets[0];
        assert_eq!(fetch_samples(source.as_ref(), ph).unwrap().len(), 2);

        install(source.as_ref(), &packs, ph).unwrap();
        let loaded = icon_packs::IconPacks::load(Some(&packs));
        assert!(loaded.notices.is_empty(), "{:?}", loaded.notices);
        let pack = loaded.pack("ph").unwrap();
        assert_eq!(pack.name, "Phosphor");
        assert_eq!(pack.iconify.as_deref(), Some("ph"));
        assert_eq!(pack.icons.len(), 2);
        assert_eq!(installed(&packs.join("ph"), "ph"), Installed::FromGallery);
        // Installing again replaces it; nothing is left beside it.
        install(source.as_ref(), &packs, ph).unwrap();
        let folders: Vec<_> = std::fs::read_dir(&packs).unwrap().collect();
        assert_eq!(folders.len(), 1);

        uninstall(&packs, "ph").unwrap();
        assert_eq!(installed(&packs.join("ph"), "ph"), Installed::No);
        uninstall(&packs, "ph").unwrap();
    }

    #[test]
    fn someones_own_pack_is_never_replaced_or_removed() {
        let source = fake();
        let dir = tempfile::tempdir().unwrap();
        let packs = icon_packs::packs_dir(dir.path());
        let own = packs.join("ph");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join("star.svg"), "<svg/>").unwrap();
        let sets = parse_collections(COLLECTIONS.as_bytes()).unwrap();
        assert!(install(source.as_ref(), &packs, &sets[0]).is_err());
        assert!(uninstall(&packs, "ph").is_err());
        assert!(own.join("star.svg").exists());
        // A failed download leaves the folder as it was.
        assert!(install(source.as_ref(), &packs, &sets[1]).is_err());
        assert!(!packs.join("tabler").exists());
        assert!(!packs.join(".installing-tabler").exists());
    }

    struct Blank;

    impl Render for Blank {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    /// In a window: the gallery lists the sets, the search narrows them,
    /// and Install adds the pack to the ones the picker shows, Uninstall
    /// takes it away.
    #[gpui_kit::test]
    fn the_gallery_lists_installs_and_uninstalls(cx: &mut gpui_kit::TestAppContext) {
        cx.executor().allow_parking();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
        let data_dir = dir.path().to_path_buf();
        let (window, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(AppState {
                store,
                data_dir,
                config: crate::config::AppConfig::default(),
            });
            cx.set_global(Gallery(fake()));
            gpui_kit::open_window(Default::default(), cx, |_, cx| cx.new(|_| Blank)).unwrap()
        });
        let gallery = window
            .update(cx, |_, window, cx| open_gallery(window, cx))
            .unwrap();
        let wait = |cx: &mut gpui_kit::TestAppContext, done: &dyn Fn(&IconGallery) -> bool| {
            let deadline = std::time::Instant::now() + Duration::from_secs(30);
            loop {
                cx.run_until_parked();
                if cx.read(|cx| done(gallery.read(cx))) {
                    break;
                }
                assert!(std::time::Instant::now() < deadline, "the gallery settled");
                std::thread::sleep(Duration::from_millis(2));
            }
        };
        wait(cx, &|g| g.is_ready() && g.samples.contains_key("ph"));
        assert_eq!(
            cx.read(|cx| gallery.read(cx).shown_prefixes(cx)),
            vec!["ph".to_owned(), "tabler".to_owned()]
        );
        window
            .update(cx, |_, window, cx| {
                gallery.update(cx, |g, cx| g.search_for("phosphor", window, cx))
            })
            .unwrap();
        assert_eq!(
            cx.read(|cx| gallery.read(cx).shown_prefixes(cx)),
            vec!["ph".to_owned()]
        );

        let ph = cx.read(|cx| gallery.read(cx).set("ph")).unwrap();
        window
            .update(cx, |_, window, cx| {
                gallery.update(cx, |g, cx| g.install(ph.clone(), window, cx))
            })
            .unwrap();
        wait(cx, &|g| !g.is_busy());
        let installed = cx.read(|cx| {
            cx.global::<icon_packs::IconPacks>()
                .pack("ph")
                .map(|p| p.icons.len())
        });
        assert_eq!(installed, Some(2));

        window
            .update(cx, |_, window, cx| {
                gallery.update(cx, |g, cx| g.uninstall(ph.clone(), window, cx))
            })
            .unwrap();
        wait(cx, &|g| !g.is_busy());
        assert!(cx.read(|cx| cx.global::<icon_packs::IconPacks>().pack("ph").is_none()));
    }

    /// Against the real gallery: the list, a set's samples, and a whole set
    /// installed and read. Run on demand.
    #[test]
    #[ignore = "reaches Iconify and jsDelivr"]
    fn the_real_gallery_installs_a_set() {
        let source = super::HttpSource::default();
        let bytes = source.get(COLLECTIONS_URL, 8 * 1024 * 1024).unwrap();
        let sets = parse_collections(&bytes).unwrap();
        println!("{} sets", sets.len());
        let lucide = sets.iter().find(|s| s.prefix == "lucide").unwrap();
        assert_eq!(
            fetch_samples(&source, lucide).unwrap().len(),
            lucide.samples.len()
        );
        let dir = tempfile::tempdir().unwrap();
        let packs = icon_packs::packs_dir(dir.path());
        install(&source, &packs, lucide).unwrap();
        let loaded = icon_packs::IconPacks::load(Some(&packs));
        let pack = loaded.pack("lucide").unwrap();
        println!("{} icons, notices {:?}", pack.icons.len(), loaded.notices);
        assert!(pack.icons.len() > 1000);
        // Lucide's own names: it can draw every one of Kanso's icons.
        assert!(pack.replaces() > 100, "{}", pack.replaces());
    }
}
