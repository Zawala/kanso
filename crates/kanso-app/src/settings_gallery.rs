//! Settings › Themes and › Icon Themes, the Browse side: the catalog in
//! `kanso-themes`, a search and filters, and the install, update, remove
//! and preview actions.
//!
//! Browse state (the tab, the search and the filter) is view state in the
//! [`ThemeGallery`] global and resets when Settings closes. The index and
//! theme files are fetched only by a refresh, by opening Browse with no
//! cached index, or by a card that is shown; nothing is fetched at launch
//! for Browse.
//!
//! Without a compiled-in themes key the gallery says it isn't set up and
//! never reaches the network.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use gpui_kit::component::button::Button;
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_gallery::state::Meta;
use kanso_gallery::{Appearance, GalleryPolicy, Index, Installed, Kind, Theme, check_index};
use kanso_update::VerifyingKey;
use kanso_update::{FetchError, Transport};

use crate::gallery;
use crate::origin::Origin;
use crate::settings::SettingsView;
use crate::settings_themes::{CARD_WIDTH, THUMB_HEIGHT, ui_thumb};
use crate::theme::Colors;
use crate::theme_file::ThemeFile;
use crate::themes::{self, ThemeEntry, Themes};
use crate::zoom::px;

/// Which half of the page shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Installed,
    Browse,
}

/// The Browse grid's filters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Light,
    Dark,
    Installed,
    Updates,
}

impl Filter {
    pub const ALL: [Filter; 5] = [
        Filter::All,
        Filter::Light,
        Filter::Dark,
        Filter::Installed,
        Filter::Updates,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Light => "Light",
            Filter::Dark => "Dark",
            Filter::Installed => "Installed",
            Filter::Updates => "Updates",
        }
    }
}

/// What the catalog fetch is doing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    /// Nothing tried yet (or a cached index was not usable).
    #[default]
    Idle,
    /// This build has no themes key: no fetch, ever.
    NotSetUp,
    Loading,
    Ready,
    /// A manual refresh failed; a background one stays quiet.
    Failed(String),
    /// The catalog is from a newer Kanso.
    NeedsNewerKanso,
}

/// Browse's view state, and the verified catalog it shows.
pub struct ThemeGallery {
    pub color_tab: Tab,
    pub icon_tab: Tab,
    pub search: String,
    pub filter: Filter,
    pub index: Option<Index>,
    pub status: Status,
    pub installed: Vec<Installed>,
    pub transport: Option<Arc<dyn Transport>>,
    pub policy: GalleryPolicy,
    pub keys: Vec<VerifyingKey>,
    /// Files already asked for, so a render never loops.
    pub requested: HashSet<String>,
    /// Fetched, checked color themes by id.
    pub files: HashMap<String, Arc<ThemeEntry>>,
    /// The verified raw bytes each theme was parsed from, so an install
    /// writes exactly what was checked instead of re-serializing it.
    pub file_bytes: HashMap<String, Arc<[u8]>>,
    /// Fetched and sanitized icon previews by id.
    pub previews: HashMap<String, Vec<Arc<[u8]>>>,
    /// Why a card couldn't show its file.
    pub failed: HashSet<String>,
    pub preview: Option<Arc<ThemeEntry>>,
    pub selected: Option<String>,
    pub notice: Option<String>,
    /// The last check's failure, kept quiet until someone opens Settings.
    pub last_error: Option<String>,
    pub last_checked_ms: Option<i64>,
    /// Keeps the automatic check alive.
    pub(crate) _schedule: Option<Task<()>>,
}

impl Global for ThemeGallery {}

/// Browse state, made with its defaults when this build hasn't set it up
/// (tests and snapshots that open Settings without running `init`).
fn gallery_mut(cx: &mut App) -> &mut ThemeGallery {
    if !cx.has_global::<ThemeGallery>() {
        cx.set_global(ThemeGallery::default());
    }
    cx.global_mut::<ThemeGallery>()
}

impl Default for ThemeGallery {
    fn default() -> Self {
        ThemeGallery {
            color_tab: Tab::Installed,
            icon_tab: Tab::Installed,
            search: String::new(),
            filter: Filter::All,
            index: None,
            status: Status::Idle,
            installed: Vec::new(),
            transport: None,
            policy: kanso_gallery::policy::policy(),
            keys: kanso_gallery::trusted_keys(),
            requested: HashSet::new(),
            files: HashMap::new(),
            file_bytes: HashMap::new(),
            previews: HashMap::new(),
            failed: HashSet::new(),
            preview: None,
            selected: None,
            notice: None,
            last_error: None,
            last_checked_ms: None,
            _schedule: None,
        }
    }
}

impl ThemeGallery {
    pub fn tab(&self, kind: Kind) -> Tab {
        match kind {
            Kind::Color => self.color_tab,
            Kind::Icons => self.icon_tab,
        }
    }

    pub fn set_tab(&mut self, kind: Kind, tab: Tab) {
        match kind {
            Kind::Color => self.color_tab = tab,
            Kind::Icons => self.icon_tab = tab,
        }
    }

    pub fn installed_record(&self, id: &str) -> Option<&Installed> {
        self.installed.iter().find(|record| record.id == id)
    }
}

/// Sets up Browse's state, reading the cached catalog and reconciling the
/// installed records against the disk. It never fetches. Before the window
/// opens.
pub fn init(cx: &mut App) {
    let mut gallery = ThemeGallery {
        transport: Some(Arc::new(kanso_update::HttpTransport::new(&format!(
            "Kanso/{}",
            crate::updates::current_version()
        )))),
        ..ThemeGallery::default()
    };
    if gallery.keys.is_empty() {
        gallery.status = Status::NotSetUp;
    } else if let Some(data_dir) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| state.data_dir.clone())
    {
        let root = themes::gallery_dir(&data_dir);
        gallery.installed = crate::gallery::installed(&root);
        let meta = Meta::load(&root.join("meta.json"));
        gallery.last_checked_ms = meta.last_checked_ms;
        let bytes = std::fs::read(root.join("index.json")).ok();
        let signature = std::fs::read(root.join("index.json.sig")).ok();
        if let (Some(bytes), Some(signature)) = (bytes, signature)
            && let Ok(index) = check_index(
                &bytes,
                &signature,
                &gallery.keys,
                &meta.floor(),
                &crate::updates::current_version(),
            )
        {
            gallery.index = Some(index);
            gallery.status = Status::Ready;
        }
    }
    cx.set_global(gallery);
}

/// Opens Browse and, with no cached catalog, fetches it once.
pub fn open(kind: Kind, cx: &mut App) {
    let fetch = {
        let gallery = gallery_mut(cx);
        gallery.set_tab(kind, Tab::Browse);
        gallery.index.is_none() && gallery.status == Status::Idle
    };
    if fetch {
        refresh(false, cx);
    }
}

pub fn close(cx: &mut App) {
    {
        let gallery = gallery_mut(cx);
        gallery.color_tab = Tab::Installed;
        gallery.icon_tab = Tab::Installed;
        gallery.search.clear();
        gallery.filter = Filter::All;
        gallery.selected = None;
    }
    themes::stop_preview(cx);
}

/// Which half of `kind`'s page is showing. Pure.
pub fn tab_for(kind: Kind, cx: &App) -> Tab {
    cx.try_global::<ThemeGallery>()
        .map_or(Tab::Installed, |gallery| gallery.tab(kind))
}

pub fn set_search(text: String, cx: &mut App) {
    gallery_mut(cx).search = text;
    cx.refresh_windows();
}

pub fn set_filter(filter: Filter, cx: &mut App) {
    gallery_mut(cx).filter = filter;
    cx.refresh_windows();
}

/// Fetches the signed catalog and verifies it before anything is parsed or
/// applied. A manual refresh reports a failure; a background one keeps the
/// last good catalog and stays quiet.
pub fn refresh(manual: bool, cx: &mut App) {
    let current = crate::updates::current_version();
    let prepared = {
        let gallery = gallery_mut(cx);
        if gallery.keys.is_empty() {
            gallery.status = Status::NotSetUp;
            None
        } else if gallery.status == Status::Loading {
            None
        } else {
            gallery.status = Status::Loading;
            gallery.notice = None;
            Some((
                gallery.keys.clone(),
                gallery.transport.clone(),
                gallery.policy,
            ))
        }
    };
    let Some((keys, Some(transport), policy)) = prepared else {
        cx.refresh_windows();
        return;
    };
    let Some(root) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| themes::gallery_dir(&state.data_dir))
    else {
        return;
    };
    let meta = Meta::load(&root.join("meta.json"));
    let index_etag = meta.index_etag.clone();
    let floor = meta.floor();
    let index_url = kanso_gallery::policy::index_url();
    let signature_url = format!("{index_url}.sig");
    cx.refresh_windows();
    let (tx, rx) = async_channel::bounded(1);
    let check_root = root.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-gallery-check".into())
        .spawn(move || {
            let result = fetch_and_check(
                transport.as_ref(),
                &policy,
                &index_url,
                &signature_url,
                index_etag.as_deref(),
                &check_root,
                &keys,
                &floor,
                &current,
            );
            tx.try_send(result).ok();
        });
    if spawned.is_err() {
        gallery_mut(cx).status = Status::Failed("The gallery couldn't start a check.".into());
        return;
    }
    cx.spawn(async move |cx| {
        let result = rx.recv().await.ok();
        cx.update(|cx| finish_refresh(manual, root, result, cx));
    })
    .detach();
}

#[derive(Debug)]
enum GalleryError {
    Fetch(FetchError),
    Index(kanso_gallery::IndexError),
}

impl std::fmt::Display for GalleryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GalleryError::Fetch(err) => write!(f, "{err}"),
            GalleryError::Index(err) => write!(f, "{err}"),
        }
    }
}

type CheckedIndex = (Vec<u8>, Vec<u8>, Option<String>, Index);
type CheckResult = Option<Result<CheckedIndex, GalleryError>>;

/// Fetches the catalog and signature, verifies them, and uses the cached
/// copy only after verifying it again when the server answers `304`. A
/// cache that no longer verifies is not a failure: the etag is dropped and
/// the index fetched unconditionally.
#[allow(clippy::too_many_arguments)]
fn fetch_and_check(
    transport: &dyn Transport,
    policy: &GalleryPolicy,
    index_url: &str,
    signature_url: &str,
    etag: Option<&str>,
    root: &std::path::Path,
    keys: &[VerifyingKey],
    floor: &kanso_gallery::Floor,
    current: &semver::Version,
) -> Result<CheckedIndex, GalleryError> {
    let index = kanso_update::fetch::get_bytes_if_none_match(
        transport,
        policy,
        index_url,
        kanso_gallery::MAX_INDEX_BYTES,
        etag,
    )
    .map_err(GalleryError::Fetch)?;
    match index {
        kanso_update::fetch::Fetched::Body { bytes, etag } => {
            let signature = fetch_signature(transport, policy, signature_url)?;
            let index = check_index(&bytes, &signature, keys, floor, current)
                .map_err(GalleryError::Index)?;
            Ok((bytes, signature, etag, index))
        }
        kanso_update::fetch::Fetched::NotModified => {
            if let Some((bytes, signature, index)) = cached_index(root, keys, floor, current) {
                return Ok((bytes, signature, etag.map(str::to_owned), index));
            }
            // The cache is gone or no longer verifies; fetch it fresh with
            // no conditional header so the server cannot answer 304 again.
            let bytes = kanso_update::fetch::get_bytes(
                transport,
                policy,
                index_url,
                kanso_gallery::MAX_INDEX_BYTES,
            )
            .map_err(GalleryError::Fetch)?;
            let signature = fetch_signature(transport, policy, signature_url)?;
            let index = check_index(&bytes, &signature, keys, floor, current)
                .map_err(GalleryError::Index)?;
            Ok((bytes, signature, None, index))
        }
    }
}

fn fetch_signature(
    transport: &dyn Transport,
    policy: &GalleryPolicy,
    signature_url: &str,
) -> Result<Vec<u8>, GalleryError> {
    kanso_update::fetch::get_bytes(
        transport,
        policy,
        signature_url,
        kanso_gallery::MAX_SIGNATURE_BYTES,
    )
    .map_err(GalleryError::Fetch)
}

fn cached_index(
    root: &std::path::Path,
    keys: &[VerifyingKey],
    floor: &kanso_gallery::Floor,
    current: &semver::Version,
) -> Option<(Vec<u8>, Vec<u8>, Index)> {
    let bytes = std::fs::read(root.join("index.json")).ok()?;
    let signature = std::fs::read(root.join("index.json.sig")).ok()?;
    let index = check_index(&bytes, &signature, keys, floor, current).ok()?;
    Some((bytes, signature, index))
}

fn finish_refresh(manual: bool, root: std::path::PathBuf, result: CheckResult, cx: &mut App) {
    let now = crate::clock::Clock::now_ms(cx);
    let reload = {
        let gallery = gallery_mut(cx);
        match result {
            Some(Ok((bytes, signature, etag, index))) => {
                // Each file is written atomically before the floor moves.
                // A crash between the two leaves a pair that doesn't
                // verify; the next start then refetches instead of trusting
                // a mixed pair.
                let written = kanso_gallery::state::write_atomic(&root.join("index.json"), &bytes)
                    .and_then(|()| {
                        kanso_gallery::state::write_atomic(&root.join("index.json.sig"), &signature)
                    });
                if let Err(err) = written {
                    gallery.status = if gallery.index.is_some() {
                        Status::Ready
                    } else {
                        Status::Failed(format!("couldn't store the catalog: {err}"))
                    };
                    gallery.last_error = Some(err.to_string());
                    false
                } else {
                    let mut meta = Meta::load(&root.join("meta.json"));
                    meta.accept(index.serial, kanso_gallery::sha256_hex(&bytes), etag);
                    meta.last_checked_ms = Some(now);
                    meta.last_error = None;
                    meta.save(&root.join("meta.json")).ok();
                    gallery.index = Some(index);
                    gallery.status = Status::Ready;
                    gallery.notice = None;
                    gallery.last_error = None;
                    gallery.last_checked_ms = Some(now);
                    gallery.installed = crate::gallery::installed(&root);
                    gallery.requested.clear();
                    true
                }
            }
            Some(Err(GalleryError::Index(kanso_gallery::IndexError::NewerSchema))) => {
                gallery.status = Status::NeedsNewerKanso;
                if manual {
                    gallery.notice = Some("The theme gallery needs a newer Kanso.".into());
                }
                false
            }
            Some(Err(err)) => {
                gallery.status = if gallery.index.is_some() {
                    Status::Ready
                } else {
                    Status::Failed(err.to_string())
                };
                let mut meta = Meta::load(&root.join("meta.json"));
                meta.last_error = Some(err.to_string());
                meta.last_checked_ms = Some(now);
                meta.save(&root.join("meta.json")).ok();
                gallery.last_checked_ms = Some(now);
                gallery.last_error = Some(err.to_string());
                if manual {
                    gallery.notice = Some(if is_offline(&err) {
                        "Kanso couldn't reach GitHub. It tries again later.".into()
                    } else {
                        format!("The theme gallery couldn't be checked: {err}")
                    });
                }
                false
            }
            None => {
                gallery.status = if gallery.index.is_some() {
                    Status::Ready
                } else {
                    Status::Failed("The check stopped.".into())
                };
                false
            }
        }
    };
    if reload {
        let plan = cx
            .try_global::<ThemeGallery>()
            .and_then(|gallery| gallery.index.clone())
            .map(|index| {
                kanso_gallery::plan(
                    &index,
                    &crate::gallery::installed(&root),
                    &crate::updates::current_version(),
                )
            });
        if let Some(plan) = plan {
            apply_plan(plan, &root, cx);
        }
        crate::themes::reload(cx);
        crate::icon_themes::reload(cx);
    }
    cx.refresh_windows();
}

fn is_offline(err: &GalleryError) -> bool {
    matches!(
        err,
        GalleryError::Fetch(
            FetchError::Offline | FetchError::RateLimited | FetchError::TooManyRedirects
        )
    )
}

/// Fetches a color theme file (or an icon theme's previews) for a card that
/// is on screen. Once per id.
pub fn request(theme: Theme, cx: &mut App) {
    let prepared = {
        let gallery = gallery_mut(cx);
        let already = !gallery.requested.insert(theme.id.clone())
            || gallery.failed.contains(&theme.id)
            || gallery.files.contains_key(&theme.id)
            || gallery.previews.contains_key(&theme.id);
        if already {
            None
        } else {
            gallery.transport.clone().map(|transport| {
                (
                    transport,
                    gallery.policy,
                    theme.payload().cloned(),
                    theme.preview.clone(),
                )
            })
        }
    };
    let Some((transport, policy, Some(payload), preview)) = prepared else {
        return;
    };
    let id = theme.id.clone();
    let kind = theme.kind;
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-gallery-file".into())
        .spawn(move || {
            // A card only needs the color file, or an icon theme's six
            // previews; the bundle waits for Install.
            let result = if kind == Kind::Color {
                kanso_update::fetch::get_bytes(
                    transport.as_ref(),
                    &policy,
                    &kanso_gallery::file_url(&payload.path),
                    kanso_gallery::MAX_COLOR_BYTES,
                )
                .map_err(|err| err.to_string())
            } else {
                Ok(Vec::new())
            };
            let mut previews = Vec::new();
            if kind == Kind::Icons {
                for file in &preview {
                    match kanso_update::fetch::get_bytes(
                        transport.as_ref(),
                        &policy,
                        &kanso_gallery::file_url(&file.path),
                        kanso_gallery::MAX_PREVIEW_BYTES,
                    ) {
                        Ok(bytes)
                            if kanso_gallery::check_pin(
                                file,
                                &bytes,
                                kanso_gallery::MAX_PREVIEW_BYTES,
                            )
                            .is_ok() =>
                        {
                            if let Ok(svg) = kanso_gallery::sanitize_svg(&bytes) {
                                previews.push(Arc::<[u8]>::from(svg));
                            }
                        }
                        _ => break,
                    }
                }
            }
            tx.try_send((id, result, previews)).ok();
        });
    if spawned.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let received = rx.recv().await.ok();
        cx.update(|cx| finish_request(received, cx));
    })
    .detach();
}

type FileAnswer = (String, Result<Vec<u8>, String>, Vec<Arc<[u8]>>);

fn finish_request(received: Option<FileAnswer>, cx: &mut App) {
    let Some((id, result, previews)) = received else {
        return;
    };
    let theme = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.index.as_ref())
        .and_then(|index| index.theme(&id))
        .cloned();
    {
        let gallery = gallery_mut(cx);
        match (result, theme) {
            (Ok(bytes), Some(theme)) if theme.kind == Kind::Color => {
                // The pin and the legibility checks run before the file is
                // kept, so nothing unverified is ever previewed or installed.
                let pinned = theme.file.as_ref().is_some_and(|file| {
                    kanso_gallery::check_pin(file, &bytes, kanso_gallery::MAX_COLOR_BYTES).is_ok()
                });
                let parsed = pinned
                    .then(|| std::str::from_utf8(&bytes).ok())
                    .flatten()
                    .and_then(|text| ThemeFile::parse(text).ok())
                    .filter(|file| file.name == theme.name)
                    .filter(|file| crate::theme_import::problems(file).is_empty());
                match parsed {
                    Some(file) => {
                        gallery.files.insert(
                            id.clone(),
                            Arc::new(ThemeEntry::new(file, id.clone(), None, Origin::Gallery)),
                        );
                        gallery.file_bytes.insert(id, Arc::from(bytes));
                    }
                    None => {
                        gallery.failed.insert(id);
                    }
                }
            }
            (Ok(_), Some(theme)) if theme.kind == Kind::Icons => {
                if previews.is_empty() && !theme.preview.is_empty() {
                    gallery.failed.insert(id);
                } else {
                    gallery.previews.insert(id, previews);
                }
            }
            (Err(_), _) | (Ok(_), None) => {
                gallery.failed.insert(id);
            }
            (Ok(_), Some(_)) => {}
        }
    }
    cx.refresh_windows();
}

/// Installs `theme` from the fetched, hash-checked bytes. If its card hasn't
/// fetched it yet, fetches it first.
pub fn install(theme: Theme, cx: &mut App) {
    install_with(theme, false, cx);
}

/// An automatic update: no toast on success, and a failure only in the log
/// and the quiet Settings line.
fn install_quiet(theme: Theme, cx: &mut App) {
    install_with(theme, true, cx);
}

fn install_with(theme: Theme, quiet: bool, cx: &mut App) {
    let cached = {
        let gallery = gallery_mut(cx);
        gallery
            .file_bytes
            .get(&theme.id)
            .cloned()
            .filter(|_| theme.kind == Kind::Color)
            .map(|bytes| bytes.to_vec())
    };
    if let Some(bytes) = cached {
        install_bytes(theme, bytes, quiet, cx);
        return;
    }
    let prepared = {
        let gallery = gallery_mut(cx);
        gallery
            .transport
            .clone()
            .map(|transport| (transport, gallery.policy))
    };
    let Some((transport, policy)) = prepared else {
        return;
    };
    let Some(payload) = theme.payload().cloned() else {
        return;
    };
    let cap = if theme.kind == Kind::Color {
        kanso_gallery::MAX_COLOR_BYTES
    } else {
        kanso_gallery::MAX_BUNDLE_BYTES
    };
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-gallery-install".into())
        .spawn(move || {
            let result = kanso_update::fetch::get_bytes(
                transport.as_ref(),
                &policy,
                &kanso_gallery::file_url(&payload.path),
                cap,
            );
            tx.try_send(result.map_err(|e| e.to_string())).ok();
        });
    if spawned.is_err() {
        return;
    }
    cx.spawn(async move |cx| {
        let result = rx.recv().await.ok();
        cx.update(|cx| match result {
            Some(Ok(bytes)) => install_bytes(theme, bytes, quiet, cx),
            Some(Err(err)) => {
                if quiet {
                    log_line(&format!("installing {} failed: {err}", theme.name));
                } else {
                    notice(cx, format!("Couldn't install it: {err}"));
                }
            }
            None => {}
        });
    })
    .detach();
}

fn install_bytes(theme: Theme, bytes: Vec<u8>, quiet: bool, cx: &mut App) {
    let Some(data_dir) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| state.data_dir.clone())
    else {
        return;
    };
    let root = themes::gallery_dir(&data_dir);
    let reserved = reserved_color_names(cx);
    let (ids, names) = reserved_icon_names(cx);
    let worker_theme = theme.clone();
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-gallery-install".into())
        .spawn(move || {
            // Unpacking a bundle and hashing a color file are file work,
            // not window work: keep them off the UI thread.
            let result = match worker_theme.kind {
                Kind::Color => {
                    gallery::install_color(&root, &worker_theme, &bytes, &reserved, quiet)
                }
                Kind::Icons => {
                    gallery::install_icons(&root, &worker_theme, &bytes, &ids, &names, quiet)
                }
            };
            tx.try_send(result).ok();
        });
    if spawned.is_err() {
        notice(cx, format!("Couldn't install {}.", theme.name));
        return;
    }
    cx.spawn(async move |cx| {
        let result = rx.recv().await.ok();
        cx.update(|cx| finish_install(&theme, quiet, result, cx));
    })
    .detach();
}

fn finish_install(theme: &Theme, quiet: bool, result: Option<Result<(), String>>, cx: &mut App) {
    match result {
        Some(Ok(())) => {
            refresh_installed(cx);
            if quiet {
                log_line(&format!(
                    "updated {} to version {}",
                    theme.name, theme.version
                ));
            } else {
                notice(cx, format!("Installed {}.", theme.name));
            }
        }
        Some(Err(err)) => {
            if quiet {
                log_line(&format!("updating {} failed: {err}", theme.name));
            } else {
                notice(cx, format!("Couldn't install {}: {err}", theme.name));
            }
        }
        None => {}
    }
}

pub fn remove(theme: Theme, cx: &mut App) {
    let Some(data_dir) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| state.data_dir.clone())
    else {
        return;
    };
    let root = themes::gallery_dir(&data_dir);
    match gallery::remove(&root, theme.kind, &theme.id) {
        Ok(()) => {
            refresh_installed(cx);
            notice(cx, format!("Removed {}.", theme.name));
        }
        Err(err) => notice(cx, format!("Couldn't remove {}: {err}", theme.name)),
    }
}

/// Selects an installed gallery theme: for colors by name, for icons by id.
pub fn use_theme(theme: Theme, cx: &mut App) {
    match theme.kind {
        Kind::Color => {
            let file = cx
                .try_global::<Themes>()
                .and_then(|themes| themes.get(&theme.name))
                .cloned();
            if let Some(entry) = file {
                let dark = entry.dark();
                crate::settings_themes::select_installed(entry.name(), dark, cx);
            }
        }
        Kind::Icons => {
            if cx
                .try_global::<crate::icon_themes::IconThemes>()
                .is_some_and(|themes| themes.get(&theme.id).is_some())
            {
                crate::icon_themes::choose(Some(theme.id.clone()), cx);
            }
        }
    }
}

/// Previews a color theme across the window from the fetched, checked file.
pub fn preview(theme: Theme, cx: &mut App) {
    let entry = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.files.get(&theme.id).cloned());
    let Some(entry) = entry else {
        request(theme, cx);
        return;
    };
    if cx
        .try_global::<ThemeGallery>()
        .is_some_and(|gallery| gallery.preview.as_ref().is_some_and(|p| p.id == entry.id))
    {
        return;
    }
    {
        let gallery = gallery_mut(cx);
        gallery.preview = Some(entry.clone());
    }
    themes::preview_theme(entry, cx);
}

pub fn stop_preview(cx: &mut App) {
    gallery_mut(cx).preview = None;
    themes::stop_preview(cx);
}

pub fn install_previewed(cx: &mut App) {
    let Some(entry) = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.preview.clone())
    else {
        return;
    };
    let theme = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.index.as_ref())
        .and_then(|index| index.theme_by_name(entry.name()).cloned());
    stop_preview(cx);
    if let Some(theme) = theme {
        install(theme, cx);
    }
}

fn refresh_installed(cx: &mut App) {
    if let Some(data_dir) = cx
        .try_global::<crate::workspace::AppState>()
        .map(|state| state.data_dir.clone())
    {
        let installed = gallery::installed(&themes::gallery_dir(&data_dir));
        {
            gallery_mut(cx).installed = installed;
        }
        crate::themes::reload(cx);
        crate::icon_themes::reload(cx);
    }
    cx.refresh_windows();
}

fn notice(cx: &mut App, message: String) {
    {
        gallery_mut(cx).notice = Some(message);
    }
    cx.refresh_windows();
}

fn reserved_color_names(cx: &App) -> HashSet<String> {
    let mut reserved = HashSet::new();
    if let Some(themes) = cx.try_global::<Themes>() {
        for entry in &themes.entries {
            if entry.origin != Origin::Gallery {
                reserved.insert(entry.name().to_lowercase());
            }
        }
    }
    reserved
}

fn reserved_icon_names(cx: &App) -> (HashSet<String>, HashSet<String>) {
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    if let Some(themes) = cx.try_global::<crate::icon_themes::IconThemes>() {
        for theme in &themes.entries {
            if theme.origin != Origin::Gallery {
                ids.insert(theme.id.clone());
                names.insert(theme.name.to_lowercase());
            }
        }
    }
    (ids, names)
}

/// The catalog's themes that a search and filter leave, paired where a
/// family holds exactly one light and one dark theme. Pure.
pub fn shown<'a>(
    index: &'a Index,
    search: &str,
    filter: Filter,
    installed: &[Installed],
) -> Vec<Item<'a>> {
    let words: Vec<String> = search
        .to_lowercase()
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    let mut keeps: Vec<&Theme> = index
        .themes
        .iter()
        .filter(|theme| matches_words(theme, &words) && matches_filter(theme, filter, installed))
        .collect();
    keeps.sort_by_key(|theme| {
        (
            theme.family.clone().unwrap_or_else(|| theme.name.clone()),
            theme.appearance.map(Appearance::is_dark).unwrap_or(false),
        )
    });
    let mut out = Vec::new();
    let mut used = HashSet::new();
    for (ix, theme) in keeps.iter().enumerate() {
        if used.contains(&ix) {
            continue;
        }
        if let Some(family) = &theme.family
            && let Some((other_ix, other)) = keeps.iter().enumerate().find(|(other_ix, other)| {
                *other_ix != ix
                    && !used.contains(other_ix)
                    && other.family.as_deref() == Some(family.as_str())
            })
        {
            // Only a light and a dark make a pair; a third variant stays a
            // card of its own beside them.
            let pair = theme.appearance.is_some()
                && other.appearance.is_some()
                && theme.appearance != other.appearance
                && keeps
                    .iter()
                    .enumerate()
                    .filter(|(third_ix, third)| {
                        *third_ix != ix
                            && *third_ix != other_ix
                            && third.family.as_deref() == Some(family.as_str())
                    })
                    .count()
                    == 0;
            if pair {
                used.insert(ix);
                used.insert(other_ix);
                let (light, dark) = if theme.appearance == Some(Appearance::Dark) {
                    (*other, *theme)
                } else {
                    (*theme, *other)
                };
                out.push(Item::Pair(light, dark));
                continue;
            }
        }
        used.insert(ix);
        out.push(Item::Single(theme));
    }
    out
}

/// One Browse card, or a paired one.
#[derive(Clone, Copy)]
pub enum Item<'a> {
    Single(&'a Theme),
    /// A family's one light and one dark theme.
    Pair(&'a Theme, &'a Theme),
}

pub fn matches_words(theme: &Theme, words: &[String]) -> bool {
    if words.is_empty() {
        return true;
    }
    let haystack = [
        theme.name.as_str(),
        theme.family.as_deref().unwrap_or_default(),
        theme.author.as_str(),
        theme.license.as_str(),
        theme.description.as_str(),
    ]
    .join(" ")
    .to_lowercase();
    words.iter().all(|word| haystack.contains(word.as_str()))
}

pub fn matches_filter(theme: &Theme, filter: Filter, installed: &[Installed]) -> bool {
    let record = installed.iter().find(|record| record.id == theme.id);
    match filter {
        Filter::All => true,
        Filter::Light => theme.appearance == Some(Appearance::Light),
        Filter::Dark => theme.appearance == Some(Appearance::Dark),
        Filter::Installed => record.is_some(),
        Filter::Updates => record.is_some_and(|record| theme.version > record.version),
    }
}

// MARK: The view

/// The Browse body for one page: cards, the footer, and the empty,
/// offline and not-set-up states. The switch and the search field are
/// drawn by Settings above this.
pub fn browse(kind: Kind, cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let Some((status, notice, index, installed, search, filter, last_checked)) = ({
        let gallery = cx.try_global::<ThemeGallery>();
        gallery.map(|gallery| {
            (
                gallery.status.clone(),
                gallery.notice.clone(),
                gallery.index.clone(),
                gallery.installed.clone(),
                gallery.search.clone(),
                gallery.filter,
                gallery.last_checked_ms,
            )
        })
    }) else {
        return Vec::new();
    };
    let Some(index) = index else {
        return vec![empty_state(status, notice, cx)];
    };
    let items = shown(&index, &search, filter, &installed);

    let mut cards = Vec::new();
    for item in items {
        match item {
            Item::Single(theme) => {
                // Asking for the card's file is what fetches it; nothing is
                // fetched before Browse shows a card.
                if theme.kind == kind {
                    request(theme.clone(), cx);
                }
                cards.push(single_card(theme, colors, cx));
            }
            Item::Pair(light, dark) => {
                if light.kind == kind {
                    request(light.clone(), cx);
                    request(dark.clone(), cx);
                }
                cards.push(pair_card(light, dark, colors, cx));
            }
        }
    }
    if cards.is_empty() {
        cards.push(
            v_flex()
                .py(px(20.))
                .text_size(px(13.))
                .text_color(colors.text_muted)
                .child("No theme matches.")
                .into_any_element(),
        );
    }

    let mut rows = Vec::new();
    rows.push(
        v_flex()
            .gap(px(4.))
            .child(
                div()
                    .text_size(px(14.))
                    .font_medium()
                    .text_color(colors.text_secondary)
                    .child("Browse"),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child("Reviewed and signed in kanso-themes."),
            )
            .into_any_element(),
    );
    if let Some(notice) = notice {
        rows.push(
            v_flex()
                .mt(px(10.))
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(notice)
                .into_any_element(),
        );
    }
    rows.push(
        h_flex()
            .mt(px(14.))
            .flex_wrap()
            .gap(px(16.))
            .children(cards)
            .into_any_element(),
    );
    rows.push(footer(last_checked, colors, cx));
    rows
}

fn empty_state(status: Status, notice: Option<String>, cx: &mut App) -> AnyElement {
    let colors = Colors::get(cx);
    let (title, detail) = match status {
        Status::NotSetUp => (
            "The theme gallery isn't set up in this build",
            "The compiled-in themes work as before; Installed and Import still do.",
        ),
        Status::Loading => ("Checking the gallery…", "This takes a moment."),
        Status::NeedsNewerKanso => (
            "The theme gallery needs a newer Kanso",
            "Installed themes keep working; update Kanso to browse the gallery.",
        ),
        Status::Failed(_) => (
            "Kanso couldn't reach GitHub",
            "Installed themes still work. Refresh to try again.",
        ),
        Status::Ready | Status::Idle => (
            "The gallery couldn't be reached",
            "Installed themes still work; Import is beside the Installed tab.",
        ),
    };
    let mut text = v_flex()
        .gap(px(6.))
        .child(
            div()
                .text_size(px(14.))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(title),
        )
        .child(
            div()
                .text_size(px(12.5))
                .text_color(colors.text_muted)
                .child(detail),
        )
        .child(
            div().mt(px(8.)).child(
                Button::new("gallery-refresh-empty")
                    .outline()
                    .small()
                    .label("Refresh")
                    .on_click(|_, _, cx| refresh(true, cx)),
            ),
        );
    if let Some(notice) = notice {
        text = text.child(
            div()
                .mt(px(6.))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(notice),
        );
    }
    section_box(text.into_any_element(), colors)
}

fn section_box(child: AnyElement, colors: &Colors) -> AnyElement {
    div()
        .mt(px(18.))
        .p(px(16.))
        .rounded(px(8.))
        .border_1()
        .border_color(colors.divider)
        .child(child)
        .into_any_element()
}

fn single_card(theme: &Theme, colors: &Colors, cx: &mut App) -> AnyElement {
    match theme.kind {
        Kind::Color => color_card(theme, colors, cx),
        Kind::Icons => icon_card(theme, colors, cx),
    }
}

fn color_card(theme: &Theme, colors: &Colors, cx: &mut App) -> AnyElement {
    let entry = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.files.get(&theme.id).cloned());
    let thumb = match &entry {
        Some(entry) => ui_thumb(entry),
        None => color_placeholder(theme, colors),
    };
    let record = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.installed_record(&theme.id).cloned());
    let builtin = crate::themes::builtins()
        .iter()
        .any(|builtin| builtin.name() == theme.name);
    let needs = !theme.runs_on(&crate::updates::current_version());
    let mut actions = h_flex().gap(px(6.)).flex_wrap();
    if builtin {
        actions = actions.child(badge("Built in", colors));
    } else if needs {
        actions = actions.child(badge(&format!("Needs Kanso {}", theme.min_kanso), colors));
    } else if let Some(record) = &record {
        actions = actions.child(badge("Installed", colors)).child(
            Button::new(format!("gallery-use-{}", theme.id))
                .outline()
                .small()
                .label("Use")
                .on_click({
                    let theme = theme.clone();
                    move |_, _, cx| use_theme(theme.clone(), cx)
                }),
        );
        if theme.version > record.version && record.is_unedited() {
            actions = actions.child(
                Button::new(format!("gallery-update-{}", theme.id))
                    .small()
                    .label("Update")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| install(theme.clone(), cx)
                    }),
            );
        }
        if record.is_unedited() {
            actions = actions.child(
                Button::new(format!("gallery-remove-{}", theme.id))
                    .outline()
                    .small()
                    .label("Remove")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| remove(theme.clone(), cx)
                    }),
            );
        } else if record.is_missing() {
            actions = actions.child(badge("Missing", colors)).child(
                Button::new(format!("gallery-reinstall-{}", theme.id))
                    .small()
                    .label("Reinstall")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| install(theme.clone(), cx)
                    }),
            );
        } else {
            actions = actions.child(badge("Edited", colors));
        }
    } else if entry.is_some() {
        actions = actions
            .child(
                Button::new(format!("gallery-install-{}", theme.id))
                    .small()
                    .label("Install")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| install(theme.clone(), cx)
                    }),
            )
            .child(
                Button::new(format!("gallery-preview-{}", theme.id))
                    .outline()
                    .small()
                    .label("Preview")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| preview(theme.clone(), cx)
                    }),
            );
    } else {
        actions = actions.child(
            Button::new(format!("gallery-install-{}", theme.id))
                .small()
                .label("Install")
                .on_click({
                    let theme = theme.clone();
                    move |_, _, cx| install(theme.clone(), cx)
                }),
        );
    }
    card_shell(
        theme,
        colors,
        thumb.w_full().h(px(THUMB_HEIGHT)),
        actions.into_any_element(),
    )
}

fn icon_card(theme: &Theme, colors: &Colors, cx: &mut App) -> AnyElement {
    let previews = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.previews.get(&theme.id).cloned())
        .unwrap_or_default();
    let strip = h_flex()
        .h(px(44.))
        .px(px(14.))
        .gap(px(12.))
        .justify_center()
        .rounded(px(6.))
        .bg(colors.editor)
        .border_1()
        .border_color(colors.divider);
    let strip = if previews.is_empty() {
        strip.child(
            div()
                .text_size(px(11.5))
                .text_color(colors.text_muted)
                .child(
                    if cx
                        .try_global::<ThemeGallery>()
                        .is_some_and(|gallery| gallery.failed.contains(&theme.id))
                    {
                        "Connect to see this theme."
                    } else {
                        "Loading icons…"
                    },
                ),
        )
    } else {
        let paths = crate::icon_themes::preview_paths(&theme.id, &previews);
        strip.children(paths.into_iter().take(6).map(|path| {
            Icon::default()
                .path(path)
                .size(px(16.))
                .text_color(colors.text_secondary)
        }))
    };
    let record = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.installed_record(&theme.id).cloned());
    let mut actions = h_flex().gap(px(6.)).flex_wrap();
    if let Some(record) = &record {
        actions = actions.child(badge("Installed", colors)).child(
            Button::new(format!("gallery-use-{}", theme.id))
                .outline()
                .small()
                .label("Use")
                .on_click({
                    let theme = theme.clone();
                    move |_, _, cx| use_theme(theme.clone(), cx)
                }),
        );
        if theme.version > record.version && record.is_unedited() {
            actions = actions.child(
                Button::new(format!("gallery-update-{}", theme.id))
                    .small()
                    .label("Update")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| install(theme.clone(), cx)
                    }),
            );
        }
        if record.is_unedited() {
            actions = actions.child(
                Button::new(format!("gallery-remove-{}", theme.id))
                    .outline()
                    .small()
                    .label("Remove")
                    .on_click({
                        let theme = theme.clone();
                        move |_, _, cx| remove(theme.clone(), cx)
                    }),
            );
        } else {
            actions = actions.child(badge("Edited", colors));
        }
    } else {
        actions = actions.child(
            Button::new(format!("gallery-install-{}", theme.id))
                .small()
                .label("Install")
                .on_click({
                    let theme = theme.clone();
                    move |_, _, cx| install(theme.clone(), cx)
                }),
        );
    }
    card_shell(
        theme,
        colors,
        div().w_full().h(px(THUMB_HEIGHT)).child(strip),
        actions.into_any_element(),
    )
}

fn pair_card(light: &Theme, dark: &Theme, colors: &Colors, cx: &mut App) -> AnyElement {
    let entry = cx
        .try_global::<ThemeGallery>()
        .and_then(|gallery| gallery.files.get(&dark.id).cloned());
    let thumb = match &entry {
        Some(entry) => ui_thumb(entry),
        None => color_placeholder(dark, colors),
    };
    let light_installed = cx
        .try_global::<ThemeGallery>()
        .is_some_and(|gallery| gallery.installed_record(&light.id).is_some());
    let dark_installed = cx
        .try_global::<ThemeGallery>()
        .is_some_and(|gallery| gallery.installed_record(&dark.id).is_some());
    let label: SharedString = format!("{} · {} and {}", light.id, light.name, dark.name).into();
    let actions = h_flex()
        .gap(px(6.))
        .flex_wrap()
        .child(if light_installed && dark_installed {
            badge("Installed", colors).into_any_element()
        } else {
            Button::new(format!("gallery-install-both-{}", dark.id))
                .small()
                .label(if light_installed || dark_installed {
                    "Install the other"
                } else {
                    "Install both"
                })
                .on_click({
                    let light = light.clone();
                    let dark = dark.clone();
                    move |_, _, cx| {
                        // Only the half that is missing: an installed (and
                        // possibly edited) copy is never overwritten by a
                        // pair's install.
                        if !light_installed {
                            install(light.clone(), cx);
                        }
                        if !dark_installed {
                            install(dark.clone(), cx);
                        }
                    }
                })
                .into_any_element()
        });
    v_flex()
        .w(px(CARD_WIDTH))
        .gap(px(6.))
        .child(
            div().p(px(2.)).rounded(px(9.)).child(
                thumb
                    .w_full()
                    .h(px(THUMB_HEIGHT))
                    .rounded(px(6.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(colors.divider),
            ),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(colors.text_secondary)
                .child(label),
        )
        .child(actions)
        .into_any_element()
}

fn color_placeholder(theme: &Theme, colors: &Colors) -> Div {
    let headline = theme.colors.clone().unwrap_or_default();
    let color = |text: &str, fallback: crate::theme::Rgb| {
        crate::theme_file::Color::parse(text).map_or(fallback, |c| c.over(crate::theme::Rgb(0)))
    };
    let editor = color(&headline.editor, colors.editor);
    let sidebar = color(&headline.sidebar, colors.sidebar);
    let text = color(&headline.text, colors.text_secondary);
    let accent = color(&headline.accent, colors.accent);
    v_flex()
        .h_full()
        .bg(editor)
        .p(px(8.))
        .gap(px(4.))
        .child(
            h_flex()
                .gap(px(4.))
                .child(div().w(px(18.)).h_full().bg(sidebar))
                .child(
                    v_flex()
                        .gap(px(4.))
                        .child(div().w(px(70.)).h(px(4.)).rounded(px(2.)).bg(text)),
                )
                .child(div().w(px(3.)).h(px(3.)).rounded_full().bg(accent)),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(colors.text_muted)
                .child("Loading…"),
        )
}

fn card_shell(theme: &Theme, colors: &Colors, thumb: Div, actions: AnyElement) -> AnyElement {
    let record = None::<Installed>;
    let _ = record;
    v_flex()
        .w(px(CARD_WIDTH))
        .gap(px(6.))
        .child(
            div().p(px(2.)).rounded(px(9.)).child(
                thumb
                    .w_full()
                    .h(px(THUMB_HEIGHT))
                    .rounded(px(6.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(colors.divider),
            ),
        )
        .child(
            h_flex()
                .gap(px(6.))
                .child(
                    div()
                        .flex_1()
                        .truncate()
                        .text_size(px(12.))
                        .line_height(px(16.))
                        .text_color(colors.text)
                        .child(theme.name.clone()),
                )
                .child(badge(&format!("v{}", theme.version), colors)),
        )
        .child(
            div()
                .text_size(px(11.))
                .line_height(px(15.))
                .text_color(colors.text_muted)
                .child(format!(
                    "From the gallery · by {} · {}",
                    theme.author, theme.license
                )),
        )
        .child(actions)
        .into_any_element()
}

fn badge(text: &str, colors: &Colors) -> AnyElement {
    div()
        .px(px(6.))
        .py(px(2.))
        .rounded(px(4.))
        .bg(colors.fill)
        .text_size(px(10.5))
        .text_color(colors.text_secondary)
        .child(text.to_owned())
        .into_any_element()
}

fn footer(last_checked: Option<i64>, colors: &Colors, _cx: &mut App) -> AnyElement {
    let checked = last_checked
        .map(|_| "Checked just now.".to_owned())
        .unwrap_or_else(|| "Not checked yet.".to_owned());
    h_flex()
        .mt(px(22.))
        .justify_between()
        .items_center()
        .child(
            div()
                .text_size(px(11.5))
                .text_color(colors.text_muted)
                .child(format!("{checked} Themes come from iamngoni/kanso-themes.")),
        )
        .child(
            Button::new("gallery-refresh")
                .outline()
                .small()
                .label("Refresh")
                .on_click(|_, _, cx| refresh(true, cx)),
        )
        .into_any_element()
}

/// The Installed | Browse switch and, in Browse, the search and filters.
pub fn header(
    kind: Kind,
    search: &Entity<gpui_kit::component::input::InputState>,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let colors = Colors::get(cx);
    let Some(gallery) = cx.try_global::<ThemeGallery>() else {
        return div().into_any_element();
    };
    let tab = gallery.tab(kind);
    let filter = gallery.filter;
    let last_error = gallery.last_error.clone();
    let mut row = h_flex().gap(px(6.)).items_center().child(
        Button::new("gallery-tab-installed")
            .small()
            .when(tab == Tab::Installed, |button| button)
            .label("Installed")
            .on_click(move |_, _, cx| {
                gallery_mut(cx).set_tab(kind, Tab::Installed);
                stop_preview(cx);
                cx.refresh_windows();
            }),
    );
    row = row.child(
        Button::new("gallery-tab-browse")
            .small()
            .label("Browse")
            .on_click(move |_, _, cx| open(kind, cx)),
    );
    if tab == Tab::Browse {
        row = row
            .child(
                div()
                    .flex_1()
                    .child(gpui_kit::component::input::Input::new(search)),
            )
            .children(
                Filter::ALL
                    .into_iter()
                    .filter(|f| *f != Filter::Installed || kind == Kind::Color)
                    .map(|f| {
                        Button::new(format!("gallery-filter-{}", f.label()))
                            .small()
                            .label(f.label())
                            .when(f == filter, |button| button)
                            .on_click(move |_, _, cx| set_filter(f, cx))
                    }),
            );
    }
    v_flex()
        .gap(px(8.))
        .child(row)
        .when_some(last_error, |el, _| {
            el.child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child("Couldn't check for theme updates. Installed themes still work."),
            )
        })
        .when_some(gallery.preview.clone(), |el, preview| {
            el.child(
                h_flex()
                    .gap(px(10.))
                    .items_center()
                    .p(px(8.))
                    .rounded(px(6.))
                    .bg(colors.fill)
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(colors.text)
                            .child(format!("Previewing {}.", preview.name())),
                    )
                    .child(
                        Button::new("gallery-preview-install")
                            .small()
                            .label("Install")
                            .on_click(|_, _, cx| install_previewed(cx)),
                    )
                    .child(
                        Button::new("gallery-preview-stop")
                            .outline()
                            .small()
                            .label("Stop")
                            .on_click(|_, _, cx| stop_preview(cx)),
                    ),
            )
        })
        .into_any_element()
}

/// Starts the automatic check: 20 seconds after launch, so it doesn't
/// compete with the app-update check at 8 seconds, then on the app's
/// half-hour wall clock, five minutes after a failure and within a minute
/// of waking.
pub fn schedule(cx: &mut App) {
    let task = cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_secs(20))
            .await;
        cx.update(auto_check);
        loop {
            cx.background_executor()
                .timer(Duration::from_secs(60))
                .await;
            cx.update(auto_check);
        }
    });
    gallery_mut(cx)._schedule = Some(task);
}

pub(crate) fn auto_check(cx: &mut App) {
    let due = {
        // The switch turns updates off, not the check: a signed revocation
        // must still be applied when automatic updates are paused.
        if cx.try_global::<crate::workspace::AppState>().is_none() {
            return;
        }
        let Some(gallery) = cx.try_global::<ThemeGallery>() else {
            return;
        };
        let failed = gallery.last_error.is_some() || matches!(gallery.status, Status::Failed(_));
        crate::updates::check_due(
            gallery.last_checked_ms,
            failed,
            crate::clock::Clock::now_ms(cx),
        )
    };
    if due {
        refresh(false, cx);
    }
}

/// Whether a planned action runs with automatic updates on (`true`) or off.
/// Only an update is gated; a revocation always applies. Pure.
fn action_runs(action: &kanso_gallery::Action, auto_update: bool) -> bool {
    auto_update || !matches!(action, kanso_gallery::Action::Update(_))
}

/// Applies a verified plan: updates only unedited copies, removes a revoked
/// theme and falls back when it was in use, and disables (never deletes) a
/// revoked copy that was edited until the owner decides that case.
fn apply_plan(plan: kanso_gallery::Plan, root: &std::path::Path, cx: &mut App) {
    let auto = cx
        .try_global::<crate::workspace::AppState>()
        .is_some_and(|state| state.config.gallery.auto_update);
    for action in plan.actions {
        if !action_runs(&action, auto) {
            continue;
        }
        match action {
            kanso_gallery::Action::Update(theme) => install_quiet(*theme, cx),
            kanso_gallery::Action::Remove { id, kind, reason } => {
                fall_back_if_selected(kind, &id, root, cx);
                if let Err(err) = crate::gallery::remove(root, kind, &id) {
                    log_line(&format!("removing revoked {id} failed: {err}"));
                    continue;
                }
                log_line(&format!("removed revoked {id}: {reason}"));
                notice(cx, format!("The gallery withdrew a theme: {reason}"));
            }
            kanso_gallery::Action::Disable { id, kind, reason } => {
                fall_back_if_selected(kind, &id, root, cx);
                if !crate::gallery::disable(root, &id) {
                    continue;
                }
                log_line(&format!("disabled revoked, edited {id}: {reason}"));
                notice(
                    cx,
                    format!(
                        "The gallery withdrew a theme you had edited; it stays on disk but is no longer drawn. {reason}"
                    ),
                );
            }
        }
    }
    refresh_installed(cx);
}

/// Switches away from a revoked theme that is in use, before its files go.
/// `state.json` records the theme's display name, and the theme settings
/// store names, so every slot naming it is cleared; comparing the id alone
/// missed an edited copy, whose record still carries the name.
fn fall_back_if_selected(kind: Kind, id: &str, root: &std::path::Path, cx: &mut App) {
    match kind {
        Kind::Color => {
            let Some(name) = crate::gallery::installed_name(root, id) else {
                return;
            };
            let dark = cx
                .try_global::<crate::themes::ActiveTheme>()
                .is_none_or(|active| active.ui.dark());
            let fallback = if dark {
                crate::config::KANSO_DARK
            } else {
                crate::config::KANSO_LIGHT
            };
            let mut changed = false;
            crate::workspace::AppState::update_config(cx, |config| {
                changed = clear_theme_name(&mut config.appearance.themes, &name, fallback);
            });
            if changed {
                crate::themes::apply(None, cx);
            }
        }
        Kind::Icons => {
            let chosen = cx
                .try_global::<crate::workspace::AppState>()
                .and_then(|state| state.config.appearance.icon_theme.clone());
            if chosen.as_deref() == Some(id) {
                crate::icon_themes::choose(None, cx);
            }
        }
    }
}

/// Clears every theme setting that names `name`, replacing the UI slots
/// with `fallback`. Returns whether anything changed. Pure.
fn clear_theme_name(themes: &mut crate::config::ThemeSettings, name: &str, fallback: &str) -> bool {
    let mut changed = false;
    for slot in [&mut themes.ui, &mut themes.light, &mut themes.dark] {
        if *slot == name {
            *slot = fallback.to_owned();
            changed = true;
        }
    }
    if themes.syntax.as_deref() == Some(name) {
        themes.syntax = None;
        changed = true;
    }
    if themes.preview.as_deref() == Some(name) {
        themes.preview = None;
        changed = true;
    }
    changed
}

static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Where the gallery log writes: `<data folder>/logs/themes.log`.
pub fn set_log_dir(data_dir: &std::path::Path) {
    let dir = data_dir.join("logs");
    if std::fs::create_dir_all(&dir).is_ok() {
        LOG_DIR.set(dir).ok();
    }
}

/// Appends a line to the gallery's log (kept under 256 KB, rotated like
/// `updates.log`) and to standard error. A quiet failure shows here.
fn log_line(line: &str) {
    eprintln!("kanso: themes: {line}");
    let Some(dir) = LOG_DIR.get() else {
        return;
    };
    let path = dir.join("themes.log");
    if std::fs::metadata(&path).is_ok_and(|meta| meta.len() > 256 * 1024) {
        std::fs::rename(&path, dir.join("themes.log.1")).ok();
    }
    use std::io::Write as _;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let now = kanso_core::now_ms();
        writeln!(file, "{now} {line}").ok();
    }
}

#[cfg(test)]
mod tests {
    use crate::settings_gallery::{
        Filter, Installed, Item, Kind, action_runs, clear_theme_name, matches_filter,
        matches_words, shown,
    };
    use kanso_gallery::index::{Index, SCHEMA};
    use kanso_gallery::{Appearance, FileRef, GalleryPolicy, Theme, check_index};
    use proptest::prelude::*;

    fn theme(id: &str, family: &str, appearance: Appearance) -> Theme {
        Theme {
            id: id.into(),
            kind: Kind::Color,
            family: Some(family.into()),
            version: 1,
            name: id.into(),
            appearance: Some(appearance),
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com".into(),
            description: "A calm theme.".into(),
            min_kanso: "0.1.0".into(),
            file: Some(FileRef {
                path: format!("{id}.json"),
                sha256: "ab".repeat(32),
                size: 10,
            }),
            bundle: None,
            preview: Vec::new(),
            colors: None,
        }
    }

    fn index(themes: Vec<Theme>) -> Index {
        Index {
            schema: SCHEMA,
            serial: 1,
            generated_at: "2026-10-23T12:00:00Z".into(),
            source_commit: "8d96298839a6176210cd1e3b376b82ffefc70ab3".into(),
            signing_key_id: "x".into(),
            themes,
            revoked: Vec::new(),
        }
    }

    fn record(id: &str, version: u64) -> Installed {
        Installed {
            id: id.into(),
            kind: Kind::Color,
            version,
            name: id.into(),
            family: None,
            digest: "aa".into(),
            current_digest: Some("aa".into()),
            installed_at: "2026-10-23T12:00:00Z".into(),
        }
    }

    #[test]
    fn clearing_a_revoked_theme_uses_its_name_not_its_id() {
        let mut settings = crate::config::ThemeSettings {
            ui: "Nord".into(),
            syntax: Some("Nord".into()),
            preview: Some("Nord".into()),
            follow_system: true,
            light: "Nord".into(),
            dark: "Kanso Dark".into(),
        };
        assert!(clear_theme_name(&mut settings, "Nord", "Kanso Light"));
        assert_eq!(settings.ui, "Kanso Light");
        assert_eq!(settings.light, "Kanso Light");
        assert_eq!(settings.dark, "Kanso Dark");
        assert_eq!(settings.syntax, None);
        assert_eq!(settings.preview, None);
        // The id never matches a name, and nothing else is touched.
        let mut settings = crate::config::ThemeSettings::default();
        assert!(!clear_theme_name(&mut settings, "nord", "Kanso Light"));
    }

    #[test]
    fn a_revocation_always_runs_even_with_automatic_updates_off() {
        let theme = theme("nord", "nord", Appearance::Dark);
        let update = kanso_gallery::Action::Update(Box::new(theme));
        let remove = kanso_gallery::Action::Remove {
            id: "nord".into(),
            kind: Kind::Color,
            reason: "Withdrawn.".into(),
        };
        let disable = kanso_gallery::Action::Disable {
            id: "nord".into(),
            kind: Kind::Color,
            reason: "Withdrawn.".into(),
        };
        assert!(action_runs(&update, true));
        assert!(!action_runs(&update, false));
        assert!(action_runs(&remove, false));
        assert!(action_runs(&disable, false));
    }

    struct ScriptedIndex {
        index: Vec<u8>,
        signature: Vec<u8>,
        calls: std::sync::atomic::AtomicUsize,
        signature_calls: std::sync::atomic::AtomicUsize,
        etag: Option<String>,
        first: u16,
    }

    impl kanso_update::Transport for ScriptedIndex {
        fn get(
            &self,
            url: &str,
        ) -> Result<kanso_update::fetch::Response, kanso_update::FetchError> {
            use std::sync::atomic::Ordering;
            let body = if url.ends_with(".sig") {
                self.signature_calls.fetch_add(1, Ordering::SeqCst);
                self.signature.clone()
            } else {
                let call = self.calls.fetch_add(1, Ordering::SeqCst);
                if call == 0 {
                    return Ok(kanso_update::fetch::Response {
                        status: self.first,
                        location: None,
                        rate_limit_remaining: None,
                        etag: self.etag.clone(),
                        body: Box::new(std::io::empty()),
                    });
                }
                self.index.clone()
            };
            Ok(kanso_update::fetch::Response {
                status: 200,
                location: None,
                rate_limit_remaining: None,
                etag: self.etag.clone(),
                body: Box::new(std::io::Cursor::new(body)),
            })
        }
    }

    fn signed_bytes(serial: u64) -> (Vec<u8>, Vec<u8>, kanso_update::VerifyingKey) {
        let (key, _) = kanso_update::feed::generate_signing_key();
        let mut index = index(vec![theme("nord", "nord", Appearance::Dark)]);
        index.serial = serial;
        let bytes = serde_json::to_vec(&index).unwrap();
        let signature = kanso_update::sign_feed(&bytes, &key).into_bytes();
        (bytes, signature, key.verifying_key())
    }

    #[test]
    fn a_304_with_a_good_cache_is_success_without_refetching() {
        let dir = tempfile::tempdir().unwrap();
        let (bytes, signature, key) = signed_bytes(1);
        std::fs::write(dir.path().join("index.json"), &bytes).unwrap();
        std::fs::write(dir.path().join("index.json.sig"), &signature).unwrap();
        let transport = ScriptedIndex {
            index: bytes.clone(),
            signature: signature.clone(),
            calls: Default::default(),
            signature_calls: Default::default(),
            etag: Some("\"v1\"".into()),
            first: 304,
        };
        let current = crate::updates::current_version();
        let answer = super::fetch_and_check(
            &transport,
            &GalleryPolicy::GALLERY,
            kanso_gallery::INDEX_URL,
            kanso_gallery::SIGNATURE_URL,
            Some("\"v1\""),
            dir.path(),
            &[key],
            &kanso_gallery::Floor::default(),
            &current,
        )
        .unwrap();
        assert_eq!(answer.0, bytes);
        assert_eq!(answer.2.as_deref(), Some("\"v1\""));
        assert_eq!(transport.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(
            transport
                .signature_calls
                .load(std::sync::atomic::Ordering::SeqCst),
            0,
            "a 304 re-verifies the cached signature instead of fetching it"
        );
    }

    #[test]
    fn a_304_with_a_corrupt_cache_refetches_unconditionally() {
        let dir = tempfile::tempdir().unwrap();
        let (bytes, signature, key) = signed_bytes(2);
        // The cache holds an index with no signature beside it.
        std::fs::write(dir.path().join("index.json"), &bytes).unwrap();
        let transport = ScriptedIndex {
            index: bytes.clone(),
            signature,
            calls: Default::default(),
            signature_calls: Default::default(),
            etag: Some("\"v2\"".into()),
            first: 304,
        };
        let current = crate::updates::current_version();
        let answer = super::fetch_and_check(
            &transport,
            &GalleryPolicy::GALLERY,
            kanso_gallery::INDEX_URL,
            kanso_gallery::SIGNATURE_URL,
            Some("\"v2\""),
            dir.path(),
            &[key],
            &kanso_gallery::Floor::default(),
            &current,
        )
        .unwrap();
        assert_eq!(answer.0, bytes);
        assert_eq!(
            transport.calls.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "the second request drops If-None-Match"
        );
    }

    #[test]
    fn search_requires_every_word() {
        let mut theme = theme("x", "fam", Appearance::Dark);
        theme.name = "nord".into();
        theme.author = "arctic".into();
        theme.description = "bluish".into();
        assert!(matches_words(
            &theme,
            &["nord".to_owned(), "arctic".to_owned()]
        ));
        assert!(!matches_words(
            &theme,
            &["nord".to_owned(), "solarized".to_owned()]
        ));
    }

    proptest! {
        /// Filters are exactly what they say: light, dark, installed, and a
        /// newer catalog version than the installed one.
        #[test]
        fn prop_filters_partition_the_catalog(
            light in any::<bool>(),
            installed in any::<bool>(),
            version in 1u64..5,
        ) {
            let mut theme = theme("x", "fam", if light { Appearance::Light } else { Appearance::Dark });
            theme.version = version;
            let records = if installed { vec![record("x", 1)] } else { Vec::new() };
            prop_assert_eq!(matches_filter(&theme, Filter::All, &records), true);
            prop_assert_eq!(matches_filter(&theme, Filter::Light, &records), light);
            prop_assert_eq!(matches_filter(&theme, Filter::Dark, &records), !light);
            prop_assert_eq!(matches_filter(&theme, Filter::Installed, &records), installed);
            prop_assert_eq!(
                matches_filter(&theme, Filter::Updates, &records),
                installed && version > 1
            );
        }

        /// At the refresh boundary, a catalog whose signature or serial
        /// fails is never returned, so no plan is ever built from it.
        #[test]
        fn prop_no_failing_catalog_is_ever_applied(
            at in any::<prop::sample::Index>(),
            bit in 0u8..8,
        ) {
            let (key, _) = kanso_update::feed::generate_signing_key();
            let catalog = index(vec![theme("nord", "nord", Appearance::Dark)]);
            let mut bytes = serde_json::to_vec(&catalog).unwrap();
            let signature = kanso_update::sign_feed(&bytes, &key).into_bytes();
            let ix = at.index(bytes.len());
            bytes[ix] ^= 1 << bit;
            let got = check_index(
                &bytes,
                &signature,
                &[key.verifying_key()],
                &kanso_gallery::Floor::default(),
                &crate::updates::current_version(),
            );
            prop_assert!(got.is_err());
        }

        /// Clearing a revoked theme's name leaves no slot pointing at it,
        /// whichever slots held it, and leaves the others alone.
        #[test]
        fn prop_clearing_a_name_clears_every_slot_that_held_it(
            name in "[A-Za-z ]{1,12}",
            fallback in "[A-Za-z ]{1,12}",
            in_ui in any::<bool>(),
            in_light in any::<bool>(),
            in_dark in any::<bool>(),
            in_syntax in any::<bool>(),
            in_preview in any::<bool>(),
        ) {
            let mut settings = crate::config::ThemeSettings {
                ui: if in_ui { name.clone() } else { "Other Ui".into() },
                light: if in_light { name.clone() } else { "Other Light".into() },
                dark: if in_dark { name.clone() } else { "Other Dark".into() },
                syntax: Some(if in_syntax { name.clone() } else { "Other Syntax".into() }),
                preview: Some(if in_preview { name.clone() } else { "Other Preview".into() }),
                follow_system: true,
            };
            let changed = clear_theme_name(&mut settings, &name, &fallback);
            prop_assert_eq!(changed, in_ui || in_light || in_dark || in_syntax || in_preview);
            prop_assert_ne!(&settings.ui, &name);
            prop_assert_ne!(&settings.light, &name);
            prop_assert_ne!(&settings.dark, &name);
            prop_assert_ne!(settings.syntax.as_deref(), Some(name.as_str()));
            prop_assert_ne!(settings.preview.as_deref(), Some(name.as_str()));
        }

        /// A search finds a theme by every word in its name, family, author,
        /// licence or description only when all its words are there.
        #[test]
        fn prop_search_needs_every_word(
            name in "[a-z]{1,8}",
            author in "[a-z]{1,8}",
            word in "[a-z]{1,6}",
        ) {
            let mut theme = theme("x", "fam", Appearance::Dark);
            theme.name = name;
            theme.author = author;
            theme.license = "MIT".into();
            theme.description = word.clone();
            prop_assert!(matches_words(&theme, std::slice::from_ref(&word)));
            prop_assert!(!matches_words(
                &theme,
                &["a-word-that-is-not-there".to_owned(), word]
            ));
        }
    }

    #[test]
    fn a_family_of_one_light_and_one_dark_is_one_paired_card() {
        let index = index(vec![
            theme("nord-light", "nord", Appearance::Light),
            theme("nord-dark", "nord", Appearance::Dark),
            theme("solo", "solo", Appearance::Dark),
        ]);
        let items = shown(&index, "", Filter::All, &[]);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], Item::Pair(..)) || matches!(items[1], Item::Pair(..)));
    }

    #[test]
    fn a_family_with_three_variants_stays_separate_cards() {
        let index = index(vec![
            theme("kanagawa-lotus", "kanagawa", Appearance::Light),
            theme("kanagawa-wave", "kanagawa", Appearance::Dark),
            theme("kanagawa-dragon", "kanagawa", Appearance::Dark),
        ]);
        let items = shown(&index, "", Filter::All, &[]);
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|item| matches!(item, Item::Single(_))));
    }

    #[test]
    fn search_filters_name_family_author_licence_and_description() {
        let mut theme = theme("nord", "arctic", Appearance::Dark);
        theme.name = "Nord".into();
        theme.author = "Arctic Ice Studio".into();
        theme.license = "MIT".into();
        theme.description = "A north-bluish palette.".into();
        for query in ["nord", "arctic", "studio", "mit", "bluish"] {
            assert!(matches_words(&theme, &[query.to_owned()]), "{query}");
        }
        assert!(!matches_words(&theme, &["solarized".to_owned()]));
    }
}
