//! Settings, inside the main window: the sidebar stays, a column lists the
//! pages (General, Editor, Appearance, Sync, Local API) where the note list
//! was, and the page takes the editor's place, in sections of bordered rows.
//! ⌘, opens it; Escape or Back to notes returns to the notes as they were.
//!
//! Switches and menus apply at once. Addresses, ports, names and URLs are
//! changed in a prompt and applied only when they check out, so the API
//! doesn't restart and sync doesn't run on every keystroke. Passwords and
//! tokens are never shown in full.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::clock::Clock;
use crate::config::{DEFAULT_FONT_SIZE, EditorFont, SyncSettings, ThemePreference};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::Library;
use crate::services::{self, ApiState, Services, SyncState, TokenState};
use crate::sidebar::{SYNC_PROBLEMS, Tone, sync_line, tone_color};
use crate::theme::Colors;
use crate::workspace::{AppState, OpenSettings, Workspace, apply_theme, for_each_workspace};
use crate::{dialogs, inkdrop, launch, paths};

actions!(
    kanso,
    [
        OpenGeneralSettings,
        OpenEditorSettings,
        OpenAppearanceSettings,
        OpenSyncSettings,
        OpenApiSettings,
        CloseSettings,
        NextSettingsPage,
        PreviousSettingsPage,
    ]
);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseSettings, Some("Settings")),
        KeyBinding::new("down", NextSettingsPage, Some("SettingsNav")),
        KeyBinding::new("up", PreviousSettingsPage, Some("SettingsNav")),
    ]);
}

/// The column of pages, where the note list is otherwise.
pub const NAV_WIDTH: f32 = 240.;
/// The page's margin, and how wide its rows grow in a wide window.
const PAGE_INSET: f32 = 32.;
const PAGE_MAX_WIDTH: f32 = 880.;
const NAV_ROW_HEIGHT: f32 = 34.;
const ROW_MIN_HEIGHT: f32 = 52.;
/// How often "Synced 3 min ago" is brought up to date.
const TICK: Duration = Duration::from_secs(30);

/// One page of Settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SettingsPage {
    #[default]
    General,
    Editor,
    Appearance,
    Sync,
    LocalApi,
}

impl SettingsPage {
    /// In the order the column lists them.
    pub const ALL: [SettingsPage; 5] = [
        SettingsPage::General,
        SettingsPage::Editor,
        SettingsPage::Appearance,
        SettingsPage::Sync,
        SettingsPage::LocalApi,
    ];

    pub fn title(self) -> &'static str {
        match self {
            SettingsPage::General => "General",
            SettingsPage::Editor => "Editor",
            SettingsPage::Appearance => "Appearance",
            SettingsPage::Sync => "Sync",
            SettingsPage::LocalApi => "Local API",
        }
    }

    fn icon(self) -> Lucide {
        match self {
            SettingsPage::General => Lucide::Settings,
            SettingsPage::Editor => Lucide::Pencil,
            SettingsPage::Appearance => Lucide::Palette,
            SettingsPage::Sync => Lucide::Cloud,
            SettingsPage::LocalApi => Lucide::Plug,
        }
    }

    fn id(self) -> &'static str {
        match self {
            SettingsPage::General => "settings-general",
            SettingsPage::Editor => "settings-editor",
            SettingsPage::Appearance => "settings-appearance",
            SettingsPage::Sync => "settings-sync",
            SettingsPage::LocalApi => "settings-api",
        }
    }

    /// The page `by` rows down the column (up when negative), stopping at
    /// either end.
    pub fn step(self, by: isize) -> SettingsPage {
        let ix = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        Self::ALL[ix.saturating_add_signed(by).min(Self::ALL.len() - 1)]
    }
}

// MARK: Plain words for values and states

/// What a masked secret always starts with, whatever its length.
pub const MASK: &str = "••••••••";
/// How many of a secret's last characters show, so a saved token or
/// password can be told apart from another.
const MASK_SUFFIX: usize = 4;
/// Shorter secrets show none of their characters.
const MASK_SUFFIX_FROM: usize = 12;

/// A secret as Settings shows it: the same row of dots for every length and,
/// for a long enough secret, its last four characters.
pub fn mask_secret(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    if chars.len() < MASK_SUFFIX_FROM {
        return MASK.to_owned();
    }
    let tail: String = chars[chars.len() - MASK_SUFFIX..].iter().collect();
    format!("{MASK}{tail}")
}

/// A status line for the top of a page: a colored dot with a few words, and
/// a sentence that says what it means or what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub tone: Tone,
    pub label: String,
    pub detail: String,
}

/// What to do about each of the sidebar's sync problems
/// ([`SYNC_PROBLEMS`]), in the same order.
const SYNC_PROBLEM_HELP: [&str; 5] = [
    "Add the Worker URL below, then sync.",
    "Set the access token below. The Worker needs it to accept changes.",
    "The Worker didn't accept the saved token. Change it below.",
    "Kanso couldn't reach the Worker. It tries again at the next sync; your notes are safe on this Mac.",
    "Something went wrong on the way. Try Sync now; if it keeps happening, check the Worker URL and the token.",
];

/// The Sync page's status: the sidebar's words ("Synced just now",
/// "Syncing…", "Offline", "Conflict detected", "Sync off") and what they
/// mean here. A conflict names the note it was in, by `conflicted_title`
/// when that is known. A failure's own reason (from the network or the
/// Keychain) never shows.
pub fn sync_status_line(
    state: &SyncState,
    settings: &SyncSettings,
    now_ms: i64,
    conflicted_title: Option<&str>,
) -> StatusLine {
    let (tone, label) = sync_line(state, now_ms);
    let every = sync_every((settings.interval_secs / 60).max(1));
    let detail = match state {
        SyncState::Off if settings.endpoint.is_none() => {
            "Add a Worker URL and an access token to sync notes between your Macs.".to_owned()
        }
        SyncState::Off if !settings.enabled => {
            "Turn on Sync automatically, or sync once with Sync now.".to_owned()
        }
        SyncState::Syncing => {
            "Sending and fetching changes. Writing carries on as usual.".to_owned()
        }
        SyncState::Off | SyncState::Synced { conflicts: 0, .. }
            if settings.enabled && settings.endpoint.is_some() =>
        {
            format!("Kanso syncs {every} and soon after you edit.")
        }
        SyncState::Synced { conflicts: 0, .. } if settings.endpoint.is_none() => {
            "Add a Worker URL to sync again.".to_owned()
        }
        SyncState::Off | SyncState::Synced { conflicts: 0, .. } => {
            "Automatic sync is off; Sync now syncs again.".to_owned()
        }
        SyncState::Synced { conflicts: 1, .. } => match conflicted_title {
            Some(title) => format!(
                "“{title}” changed on two Macs at once. Both versions are kept: the other one is in its history."
            ),
            None => "A note changed on two Macs at once. Both versions are kept: the other one is in the note's history.".to_owned(),
        },
        SyncState::Synced { conflicts, .. } => match conflicted_title {
            Some(title) => format!(
                "“{title}” and {} other {} changed on two Macs at once. Both versions of each are kept: the others are in their history.",
                conflicts - 1,
                if *conflicts == 2 { "note" } else { "notes" },
            ),
            None => format!(
                "{conflicts} notes changed on two Macs at once. Both versions of each are kept: the others are in their history."
            ),
        },
        SyncState::Failed(_) => {
            let problem = SYNC_PROBLEMS
                .iter()
                .position(|p| *p == label)
                .unwrap_or(SYNC_PROBLEMS.len() - 1);
            SYNC_PROBLEM_HELP[problem].to_owned()
        }
    };
    StatusLine {
        tone,
        label,
        detail,
    }
}

/// The Local API page's status. A failure's own reason only shows in the
/// tooltip.
pub fn api_status_line(state: &ApiState) -> StatusLine {
    let (tone, label, detail) = match state {
        ApiState::Off => (
            Tone::Quiet,
            "Off".to_owned(),
            "Turn it on to let Inkdrop's clients and local agents reach your notes.".to_owned(),
        ),
        ApiState::Running(addr) => (
            Tone::Good,
            "Running".to_owned(),
            format!("Listening on {addr}. Clients set up for Inkdrop work unchanged."),
        ),
        ApiState::PortInUse(port) => (
            Tone::Warning,
            format!("Port {port} is in use"),
            "Quit Inkdrop to free it, or choose another port below. Kanso keeps trying.".to_owned(),
        ),
        ApiState::Failed(reason) => {
            let reason = reason.to_lowercase();
            let detail = if reason.contains("username") || reason.contains("password") {
                "Set a username and password below."
            } else {
                "Check the address and port below."
            };
            (
                Tone::Warning,
                "Couldn't start".to_owned(),
                detail.to_owned(),
            )
        }
    };
    StatusLine {
        tone,
        label,
        detail,
    }
}

/// Kanso's MCP endpoint for clients on this Mac: the API's address, or the
/// loopback address when it listens on all of them.
pub fn mcp_endpoint(bind_address: &str, port: u16) -> String {
    let ip = bind_address
        .trim()
        .parse::<IpAddr>()
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let ip = match ip {
        IpAddr::V4(v4) if v4.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(v6) if v6.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        ip => ip,
    };
    format!("http://{}/mcp", SocketAddr::new(ip, port))
}

/// The choices for how often sync runs, in minutes.
pub const SYNC_INTERVALS: [u64; 6] = [1, 5, 15, 30, 60, 240];

/// "1 minute", "15 minutes", "1 hour", "4 hours".
pub fn interval_label(minutes: u64) -> String {
    match minutes {
        1 => "1 minute".to_owned(),
        60 => "1 hour".to_owned(),
        m if m > 0 && m % 60 == 0 => format!("{} hours", m / 60),
        m => format!("{m} minutes"),
    }
}

/// "every minute", "every 15 minutes", "every hour".
fn sync_every(minutes: u64) -> String {
    match minutes {
        1 => "every minute".to_owned(),
        60 => "every hour".to_owned(),
        m => format!("every {}", interval_label(m)),
    }
}

/// How small and large the editor's text can be.
pub const FONT_SIZES: (f32, f32) = (10., 28.);

/// The editor's text size `step` points larger (smaller when negative), in
/// whole points within [`FONT_SIZES`].
pub fn step_font_size(size: f32, step: i32) -> f32 {
    let size = if size.is_finite() {
        size.round()
    } else {
        DEFAULT_FONT_SIZE
    };
    (size + step as f32).clamp(FONT_SIZES.0, FONT_SIZES.1)
}

/// An error message as a sentence: a capital letter first and a full stop.
pub fn sentence(message: &str) -> String {
    let message = message.trim();
    let mut chars = message.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out: String = first.to_uppercase().chain(chars).collect();
    if !out.ends_with(['.', '!', '?']) {
        out.push('.');
    }
    out
}

/// A path with the home folder written as `~`.
pub fn display_path(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if !home.as_os_str().is_empty() && home != Path::new("/") => {
            if rest.as_os_str().is_empty() {
                "~".to_owned()
            } else {
                format!("~/{}", rest.display())
            }
        }
        _ => path.display().to_string(),
    }
}

fn font_detail(font: EditorFont) -> &'static str {
    match font {
        EditorFont::Auto => "Proportional for writing; monospace beside the preview.",
        EditorFont::Sans => "Proportional in every view.",
        EditorFont::Mono => "Monospace in every view, where Markdown tables line up.",
    }
}

// MARK: Changing settings

fn config(cx: &App) -> &crate::config::AppConfig {
    &cx.global::<AppState>().config
}

/// Brings every editor up to the editor settings. Settings are drawn in the
/// window being updated, which can't be reached until that update ends.
fn update_editors(cx: &mut App) {
    cx.defer(|cx| {
        let appearance = config(cx).appearance.clone();
        for_each_workspace(cx, |workspace, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.editor_font = appearance.editor_font;
                editor.font_size = appearance.editor_font_size;
                editor.allow_line_numbers = appearance.line_numbers;
                editor.readable_width = appearance.readable_width;
                cx.notify();
            });
        });
    });
}

fn set_editor_font(font: EditorFont, cx: &mut App) {
    AppState::update_config(cx, |config| config.appearance.editor_font = font);
    update_editors(cx);
}

fn step_editor_size(step: i32, cx: &mut App) {
    let size = step_font_size(config(cx).appearance.editor_font_size, step);
    AppState::update_config(cx, |config| config.appearance.editor_font_size = size);
    update_editors(cx);
}

fn set_line_numbers(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.appearance.line_numbers = on);
    update_editors(cx);
}

fn set_readable_width(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.appearance.readable_width = on);
    update_editors(cx);
}

fn set_theme(theme: ThemePreference, window: &mut Window, cx: &mut App) {
    AppState::update_config(cx, |config| config.appearance.theme = theme);
    apply_theme(theme, Some(window), cx);
    cx.refresh_windows();
}

fn set_launch_at_login(on: bool, window: &mut Window, cx: &mut App) {
    let data_dir = cx.global::<AppState>().data_dir.clone();
    match launch::set_enabled(&paths::launch_agent_path(), on, &data_dir) {
        Ok(()) => AppState::update_config(cx, |config| config.launch_at_login = on),
        Err(err) => {
            eprintln!("kanso: launch at login: {err:#}");
            window.push_notification(
                Notification::error("Couldn't change whether Kanso opens at login."),
                cx,
            );
        }
    }
}

/// The Worker URL from what was typed, saved and put to use.
fn apply_endpoint(value: String, cx: &mut App) -> Result<(), String> {
    let endpoint = kanso_sync::check_endpoint(&value).map_err(|err| sentence(&err.to_string()))?;
    AppState::update_config(cx, |config| config.sync.endpoint = Some(endpoint));
    services::apply_sync_settings(cx);
    Ok(())
}

fn apply_address(value: String, cx: &mut App) -> Result<(), String> {
    let ip: IpAddr = value
        .trim()
        .parse()
        .map_err(|_| "Enter an IP address, such as 127.0.0.1.".to_owned())?;
    AppState::update_config(cx, |config| config.server.bind_address = ip.to_string());
    services::apply_server_settings(cx);
    Ok(())
}

fn apply_port(value: String, cx: &mut App) -> Result<(), String> {
    let port = parse_port(&value)?;
    AppState::update_config(cx, |config| config.server.port = port);
    services::apply_server_settings(cx);
    Ok(())
}

fn apply_username(value: String, cx: &mut App) -> Result<(), String> {
    let username = parse_username(&value)?;
    AppState::update_config(cx, |config| config.server.username = username);
    services::apply_server_settings(cx);
    Ok(())
}

/// The API port from what was typed.
pub fn parse_port(value: &str) -> Result<u16, String> {
    match value.trim().parse::<u16>() {
        Ok(port) if port >= 1024 => Ok(port),
        _ => Err("Use a port from 1024 to 65535.".into()),
    }
}

/// The API username from what was typed. Basic authentication can't carry
/// a colon in the username.
pub fn parse_username(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.contains(':') || value.chars().any(char::is_control) {
        Err("The username can't be empty or contain a colon.".into())
    } else {
        Ok(value.to_owned())
    }
}

fn copy_to_clipboard(text: String, message: &'static str, window: &mut Window, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(text));
    window.push_notification(Notification::info(message), cx);
}

fn change_token(window: &mut Window, cx: &mut App) {
    dialogs::prompt_secret(
        "Sync Access Token",
        "Save",
        window,
        cx,
        |token, window, cx| match services::save_sync_token(&token, cx) {
            Ok(()) => {
                window.push_notification(Notification::success("Saved to the Keychain."), cx);
                services::apply_sync_settings(cx);
                Ok(())
            }
            Err(err) => {
                eprintln!("kanso: saving the sync token: {err:#}");
                Err("Couldn't save the token in the Keychain.".to_owned())
            }
        },
    );
}

fn remove_token(window: &mut Window, cx: &mut App) {
    dialogs::confirm(
        "Remove the sync access token?",
        "Sync stops until a token is set again. The token itself stays valid on the Worker.",
        "Remove",
        window,
        cx,
        |window, cx| match services::remove_sync_token(cx) {
            Ok(()) => services::apply_sync_settings(cx),
            Err(err) => {
                eprintln!("kanso: removing the sync token: {err:#}");
                window.push_notification(
                    Notification::error("Couldn't remove the token from the Keychain."),
                    cx,
                );
            }
        },
    );
}

fn change_password(window: &mut Window, cx: &mut App) {
    dialogs::prompt_secret(
        "New API Password",
        "Change",
        window,
        cx,
        |password, _, cx| {
            AppState::update_config(cx, |config| config.server.password = password);
            services::apply_server_settings(cx);
            Ok(())
        },
    );
}

fn copy_from_inkdrop(window: &mut Window, cx: &mut App) {
    dialogs::confirm(
        "Use Inkdrop's port, username and password?",
        "Inkdrop keeps its password encrypted with a key in your Keychain. macOS will ask you to allow Kanso to read “inkdrop Safe Storage”.",
        "Continue",
        window,
        cx,
        |window, cx| {
            inkdrop::start_copy_credentials(window, cx, |_, cx| {
                services::apply_server_settings(cx)
            });
        },
    );
}

/// A value with a button that asks for a new one. `apply` says why a value
/// can't be used, and the prompt stays open until it can.
fn change_button(
    id: &'static str,
    title: &'static str,
    current: String,
    apply: fn(String, &mut App) -> Result<(), String>,
) -> Button {
    let label = if current.is_empty() {
        "Set…"
    } else {
        "Change…"
    };
    Button::new(id)
        .outline()
        .small()
        .label(label)
        .on_click(move |_, window, cx| {
            dialogs::prompt_checked(title, &current, "Save", window, cx, move |value, _, cx| {
                apply(value, cx)
            });
        })
}

// MARK: The view

/// Where the notes were when Settings opened, to go back to.
pub struct SettingsReturn {
    focus: Option<FocusHandle>,
    /// Focus mode was on: Settings leave it, and going back brings it back.
    focus_mode: bool,
}

pub struct SettingsView {
    /// Where a conflicted note is opened from the Sync page.
    library: Entity<Library>,
    page: SettingsPage,
    /// The page column's: up and down move between pages there.
    nav_focus: FocusHandle,
    /// The whole view's, to tell whether the keyboard is somewhere in it.
    root_focus: FocusHandle,
    scroll: ScrollHandle,
    /// Room kept at the left of the top band, for the window's traffic
    /// lights when the sidebar is hidden.
    leading_inset: Pixels,
    backing_up: bool,
    /// Shown instead of the data folder, whose temporary path in a snapshot
    /// changes from run to run.
    #[cfg(test)]
    pub(crate) data_dir_label: Option<String>,
    _tick: Task<()>,
}

impl SettingsView {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        SettingsView {
            library,
            page: SettingsPage::default(),
            nav_focus: cx.focus_handle(),
            root_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            leading_inset: px(0.),
            backing_up: false,
            #[cfg(test)]
            data_dir_label: None,
            _tick: tick,
        }
    }

    pub fn page(&self) -> SettingsPage {
        self.page
    }

    /// Shows `page` from its top. The Sync page looks up whether a token
    /// is saved, the first time it shows.
    pub fn show(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        if page == SettingsPage::Sync {
            services::ensure_sync_token(cx);
        }
        if self.page != page {
            self.page = page;
            self.scroll.set_offset(Point::default());
            cx.notify();
        }
    }

    /// Gives the keyboard to the page column.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.nav_focus, cx);
    }

    pub fn contains_focus(&self, window: &Window, cx: &App) -> bool {
        self.root_focus.contains_focused(window, cx)
    }

    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    /// Copies the database to the backups folder, off the main thread.
    fn back_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.backing_up {
            return;
        }
        let state = cx.global::<AppState>();
        let store = state.store.clone();
        let dir = paths::backups_dir(&state.data_dir);
        let path = dir.join(format!("kanso-{}.db", kanso_core::now_ms()));
        let backup = cx.background_spawn({
            let path = path.clone();
            async move {
                std::fs::create_dir_all(&dir)?;
                store.backup_to(&path)?;
                anyhow::Ok(())
            }
        });
        self.backing_up = true;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = backup.await;
            this.update_in(cx, |this, window, cx| {
                this.backing_up = false;
                let notification = match result {
                    Ok(()) => Notification::success("Backed up."),
                    Err(err) => {
                        eprintln!("kanso: backing up to {}: {err:#}", path.display());
                        Notification::error(
                            "Couldn't back up your notes. Check there is free space on this Mac, then try again.",
                        )
                    }
                };
                window.push_notification(notification, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_nav(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // Escape as the palette writes it, rather than the ⎋ glyph.
        let back_key = Kbd::binding_for_action(&CloseSettings, Some("Settings"), window)
            .map(|_| crate::keys::keycap("esc", cx).into_any_element());
        // The page shown wears a ring while the column has the keyboard,
        // so it is clear the arrows move there.
        let nav_focused = self.nav_focus.is_focused(window);
        let mut items = Vec::new();
        for page in SettingsPage::ALL {
            items.push(self.render_nav_item(page, nav_focused, cx));
        }
        v_flex()
            .id("settings-nav")
            .w(px(NAV_WIDTH))
            .flex_none()
            .h_full()
            .bg(colors.list)
            .key_context("SettingsNav")
            .track_focus(&self.nav_focus)
            .on_action(cx.listener(|this, _: &NextSettingsPage, _, cx| {
                this.show(this.page.step(1), cx);
            }))
            .on_action(cx.listener(|this, _: &PreviousSettingsPage, _, cx| {
                this.show(this.page.step(-1), cx);
            }))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(TOP_BAND_HEIGHT))
                    .pl(px(18.) + self.leading_inset)
                    .pr(px(10.))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child("Settings"),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .px_2()
                    .pt(px(4.))
                    .gap(px(2.))
                    .children(items),
            )
            .child(
                h_flex()
                    .flex_none()
                    .h(px(BOTTOM_BAND_HEIGHT))
                    .px(px(8.))
                    .gap_1()
                    .border_t_1()
                    .border_color(colors.divider)
                    .child(
                        Button::new("settings-back")
                            .ghost()
                            .small()
                            .icon(Lucide::ArrowLeft)
                            .label("Back to notes")
                            .text_color(colors.text_secondary)
                            .debug_selector(|| "settings-back".into())
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(CloseSettings), cx)
                            }),
                    )
                    .child(div().flex_1())
                    .children(back_key),
            )
    }

    fn render_nav_item(
        &self,
        page: SettingsPage,
        nav_focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let selected = self.page == page;
        let color = if selected {
            colors.text
        } else {
            colors.text_secondary
        };
        h_flex()
            .id(page.id())
            .flex_none()
            .h(px(NAV_ROW_HEIGHT))
            // The ring's width comes out of the padding.
            .px(px(9.))
            .gap(px(10.))
            .rounded(px(6.))
            .border_1()
            .border_color(if selected && nav_focused {
                Hsla::from(colors.accent)
            } else {
                transparent_black()
            })
            .cursor_pointer()
            .text_size(px(14.))
            .text_color(color)
            .map(|el| {
                if selected {
                    el.bg(colors.row_active)
                } else {
                    el.hover(|el| el.bg(colors.row_hover))
                }
            })
            .child(Icon::new(page.icon()).size(px(16.)).text_color(color))
            .child(page.title())
            .debug_selector(move || page.id().into())
            .on_click(cx.listener(move |this, _, window, cx| {
                this.show(page, cx);
                window.focus(&this.nav_focus, cx);
            }))
            .into_any_element()
    }

    fn render_page(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        match self.page {
            SettingsPage::General => self.general_page(cx),
            SettingsPage::Editor => editor_page(cx),
            SettingsPage::Appearance => appearance_page(cx),
            SettingsPage::Sync => sync_page(self.library.clone(), cx),
            SettingsPage::LocalApi => api_page(cx),
        }
    }

    fn general_page(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = Colors::get(cx);
        let state = cx.global::<AppState>();
        let launch_at_login = state.config.launch_at_login;
        let dir = state.data_dir.clone();
        let shown = display_path(&dir, &paths::home());
        #[cfg(test)]
        let shown = self.data_dir_label.clone().unwrap_or(shown);
        vec![
            section(
                "Startup",
                vec![
                    Row::new(Lucide::Power, "Open Kanso at login")
                        .detail("Kanso starts when you log in to this Mac, with the local API.")
                        .trailing(
                            Switch::new("launch-at-login")
                                .checked(launch_at_login)
                                .on_click(|on, window, cx| set_launch_at_login(*on, window, cx)),
                        ),
                ],
                colors,
            ),
            section(
                "Data",
                vec![
                    Row::new(Lucide::FolderOpen, "Data folder")
                        .value(shown)
                        .trailing(
                            Button::new("reveal-data")
                                .outline()
                                .small()
                                .label("Show in Finder")
                                .on_click(move |_, _, cx| cx.reveal_path(&dir)),
                        ),
                    Row::new(Lucide::DatabaseBackup, "Backups")
                        .detail("Kanso backs up its database once a day and keeps the last 14.")
                        .trailing(
                            Button::new("backup-now")
                                .outline()
                                .small()
                                .label("Back up now")
                                .loading(self.backing_up)
                                .on_click(cx.listener(|this, _, window, cx| this.back_up(window, cx))),
                        ),
                ],
                colors,
            ),
            section(
                "Import",
                vec![
                    Row::new(Lucide::Upload, "Import from Inkdrop")
                        .detail("Copies notes, notebooks, tags and attachments. Inkdrop's files are only read.")
                        .trailing(
                            Button::new("import-inkdrop")
                                .outline()
                                .small()
                                .label("Import…")
                                .on_click(|_, window, cx| inkdrop::start_import(window, cx)),
                        ),
                ],
                colors,
            ),
        ]
    }
}

fn editor_page(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let appearance = config(cx).appearance.clone();
    let size = appearance.editor_font_size;
    let stepper = h_flex()
        .gap(px(6.))
        .child(
            Button::new("font-smaller")
                .debug_selector(|| "font-smaller".into())
                .outline()
                .small()
                .icon(Lucide::Minus)
                .tooltip("Smaller")
                .disabled(size <= FONT_SIZES.0)
                .on_click(|_, _, cx| step_editor_size(-1, cx)),
        )
        .child(
            div()
                .w(px(44.))
                .text_center()
                .text_size(px(13.5))
                .text_color(colors.text)
                .child(format!("{size:.0} pt")),
        )
        .child(
            Button::new("font-larger")
                .debug_selector(|| "font-larger".into())
                .outline()
                .small()
                .icon(Lucide::Plus)
                .tooltip("Larger")
                .disabled(size >= FONT_SIZES.1)
                .on_click(|_, _, cx| step_editor_size(1, cx)),
        );
    vec![section(
        "Text",
        vec![
            Row::new(Lucide::Type, "Font")
                .detail(font_detail(appearance.editor_font))
                .trailing(segmented(
                    "editor-font",
                    [
                        ("Auto", EditorFont::Auto),
                        ("Sans", EditorFont::Sans),
                        ("Mono", EditorFont::Mono),
                    ],
                    appearance.editor_font,
                    |font, _, cx| set_editor_font(font, cx),
                    colors,
                )),
            Row::new(Lucide::ALargeSmall, "Text size")
                .detail("The Markdown source, in every view.")
                .trailing(stepper),
            Row::new(Lucide::ListOrdered, "Line numbers")
                .detail("Beside monospace text.")
                .trailing(
                    div().debug_selector(|| "line-numbers".into()).child(
                        Switch::new("line-numbers")
                            .checked(appearance.line_numbers)
                            .on_click(|on, _, cx| set_line_numbers(*on, cx)),
                    ),
                ),
            Row::new(Lucide::Ruler, "Readable width")
                .detail("Keeps lines to a comfortable length; off lets text fill the pane.")
                .trailing(
                    div().debug_selector(|| "readable-width".into()).child(
                        Switch::new("readable-width")
                            .checked(appearance.readable_width)
                            .on_click(|on, _, cx| set_readable_width(*on, cx)),
                    ),
                ),
        ],
        colors,
    )]
}

fn appearance_page(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let theme = config(cx).appearance.theme;
    vec![section(
        "Theme",
        vec![
            Row::new(Lucide::SunMoon, "Light and dark")
                .detail(
                    "System follows the macOS setting. Dark is warm graphite; light is soft paper.",
                )
                .trailing(segmented(
                    "theme",
                    [
                        ("System", ThemePreference::System),
                        ("Light", ThemePreference::Light),
                        ("Dark", ThemePreference::Dark),
                    ],
                    theme,
                    set_theme,
                    colors,
                )),
        ],
        colors,
    )]
}

fn sync_page(library: Entity<Library>, cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let settings = config(cx).sync.clone();
    let (state, token, tip) = match cx.try_global::<Services>() {
        Some(services) => (
            services.sync.clone(),
            services.sync_token.clone(),
            services.sync_status(),
        ),
        None => (SyncState::Off, TokenState::Unknown, "Sync off".to_owned()),
    };
    // The first conflicted note that is still there, to name and open.
    let conflicted = match &state {
        SyncState::Synced { conflicted, .. } => {
            let store = &cx.global::<AppState>().store;
            conflicted
                .iter()
                .filter(|id| id.starts_with("note:"))
                .find_map(|id| store.note(id).ok().flatten())
        }
        _ => None,
    };
    let conflicted_title = conflicted.as_ref().map(|note| {
        if note.title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            note.title.clone()
        }
    });
    let status = sync_status_line(
        &state,
        &settings,
        Clock::now_ms(cx),
        conflicted_title.as_deref(),
    );
    let open_note = conflicted.map(|note| {
        let id = note.id;
        Button::new("open-conflicted-note")
            .outline()
            .label("Open note")
            .debug_selector(|| "open-conflicted-note".into())
            .on_click(move |_, _, cx| {
                let id = id.clone();
                library.update(cx, |library, cx| library.select(Some(id), cx));
            })
    });
    let sync_now = Button::new("sync-now")
        .outline()
        .icon(Lucide::RefreshCcw)
        .label("Sync now")
        .loading(state == SyncState::Syncing)
        // Nothing to sync with until there is a Worker.
        .disabled(settings.endpoint.is_none())
        .debug_selector(|| "sync-now".into())
        .on_click(|_, _, cx| services::sync_now(cx));
    let endpoint = settings.endpoint.clone().unwrap_or_default();
    let minutes = (settings.interval_secs / 60).max(1);
    let interval = Button::new("sync-interval")
        .outline()
        .small()
        .label(interval_label(minutes))
        .dropdown_caret(true)
        .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, _| {
            SYNC_INTERVALS.iter().fold(menu, |menu, &option| {
                menu.item(
                    PopupMenuItem::new(interval_label(option))
                        .checked(option == minutes)
                        .on_click(move |_, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.sync.interval_secs = option * 60
                            });
                            services::apply_sync_settings(cx);
                        }),
                )
            })
        });
    let set_token = || {
        Button::new("set-token")
            .outline()
            .small()
            .label("Set…")
            .debug_selector(|| "set-token".into())
            .on_click(|_, window, cx| change_token(window, cx))
            .into_any_element()
    };
    // A token can always be set, whatever the Keychain said or hasn't yet.
    let token_controls = match token {
        TokenState::Unknown => vec![
            value_text("Checking…", colors).into_any_element(),
            set_token(),
        ],
        TokenState::Missing => vec![
            value_text("Not set", colors).into_any_element(),
            set_token(),
        ],
        TokenState::Unreadable => vec![
            value_text("Couldn't read", colors).into_any_element(),
            set_token(),
            Button::new("remove-token")
                .outline()
                .small()
                .label("Remove")
                .on_click(|_, window, cx| remove_token(window, cx))
                .into_any_element(),
        ],
        TokenState::Saved(masked) => vec![
            value_text(masked, colors).into_any_element(),
            Button::new("change-token")
                .outline()
                .small()
                .label("Change…")
                .on_click(|_, window, cx| change_token(window, cx))
                .into_any_element(),
            Button::new("remove-token")
                .outline()
                .small()
                .label("Remove")
                .on_click(|_, window, cx| remove_token(window, cx))
                .into_any_element(),
        ],
    };
    let actions = h_flex()
        .flex_none()
        .gap_2()
        .children(open_note)
        .child(sync_now);
    // Until one is set, the row says what a Worker is, and "Not set" sits
    // with the button, as the token's does.
    let worker = Row::new(Lucide::Cloud, "Worker URL");
    let worker = if endpoint.is_empty() {
        worker
            .detail("The address of your own Cloudflare Worker (cloud/README.md sets one up).")
            .trailing(value_text("Not set", colors))
    } else {
        worker.value(endpoint.clone())
    };
    vec![
        status_row(&status, tip, Some(actions.into_any_element()), colors),
        section(
            "Server",
            vec![
                worker.trailing(change_button(
                    "change-endpoint",
                    "Worker URL",
                    endpoint,
                    apply_endpoint,
                )),
                Row::new(Lucide::KeyRound, "Access token")
                    .detail("Kept in your Keychain, never in Kanso's settings file.")
                    .trailing_all(token_controls),
            ],
            colors,
        ),
        section(
            "Schedule",
            vec![
                Row::new(Lucide::RefreshCcw, "Sync automatically")
                    .detail("In the background, and soon after you edit.")
                    .trailing(Switch::new("sync-enabled").checked(settings.enabled).on_click(
                        |on, _, cx| {
                            let on = *on;
                            AppState::update_config(cx, |config| config.sync.enabled = on);
                            services::apply_sync_settings(cx);
                        },
                    )),
                Row::new(Lucide::Clock, "Sync every").trailing(interval),
            ],
            colors,
        ),
        section(
            "Conflict handling",
            vec![
                Row::new(Lucide::Split, "Keep both versions")
                    .detail("When a note changes on two Macs at once, the newer version shows and the other is kept in the note's history."),
            ],
            colors,
        ),
    ]
}

fn api_page(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let server = config(cx).server.clone();
    let (state, tip) = match cx.try_global::<Services>() {
        Some(services) => (services.api.clone(), services.api_status()),
        None => (ApiState::Off, "API off".to_owned()),
    };
    let status = api_status_line(&state);
    let endpoint = mcp_endpoint(&server.bind_address, server.port);
    let password = server.password.clone();
    let mut password_controls = vec![
        // None of the password shows: it may be one used elsewhere too,
        // and Copy gets it when it's needed.
        value_text(if password.is_empty() { "Not set" } else { MASK }, colors).into_any_element(),
    ];
    if !password.is_empty() {
        password_controls.push(
            Button::new("copy-password")
                .outline()
                .small()
                .label("Copy")
                .on_click(move |_, window, cx| {
                    copy_to_clipboard(password.clone(), "Password copied.", window, cx)
                })
                .into_any_element(),
        );
    }
    password_controls.push(
        Button::new("change-password")
            .outline()
            .small()
            .label("Change…")
            .on_click(|_, window, cx| change_password(window, cx))
            .into_any_element(),
    );
    vec![
        status_row(&status, tip, None, colors),
        section(
            "Server",
            vec![
                Row::new(Lucide::Plug, "Enable the local API")
                    .detail("Inkdrop's MCP server (@inkdropapp/mcp-server) and other Inkdrop API clients work unchanged.")
                    .trailing(Switch::new("api-enabled").checked(server.enabled).on_click(
                        |on, _, cx| {
                            let on = *on;
                            AppState::update_config(cx, |config| config.server.enabled = on);
                            services::apply_server_settings(cx);
                        },
                    )),
                Row::new(Lucide::Network, "Address")
                    .detail("127.0.0.1 keeps the API on this Mac.")
                    .trailing_all(vec![
                        value_text(server.bind_address.clone(), colors).into_any_element(),
                        change_button(
                            "change-address",
                            "API Address",
                            server.bind_address.clone(),
                            apply_address,
                        )
                        .into_any_element(),
                    ]),
                Row::new(Lucide::Hash, "Port")
                    .detail("Inkdrop's is 19840; quit Inkdrop first to use the same one.")
                    .trailing_all(vec![
                        value_text(server.port.to_string(), colors).into_any_element(),
                        change_button(
                            "change-port",
                            "API Port",
                            server.port.to_string(),
                            apply_port,
                        )
                        .into_any_element(),
                    ]),
            ],
            colors,
        ),
        section(
            "Sign-in",
            vec![
                Row::new(Lucide::User, "Username").trailing_all(vec![
                    value_text(
                        if server.username.is_empty() {
                            "Not set".to_owned()
                        } else {
                            server.username.clone()
                        },
                        colors,
                    )
                    .into_any_element(),
                    change_button(
                        "change-username",
                        "API Username",
                        server.username.clone(),
                        apply_username,
                    )
                    .into_any_element(),
                ]),
                Row::new(Lucide::KeyRound, "Password").trailing_all(password_controls),
                Row::new(Lucide::Copy, "Inkdrop's settings")
                    .detail("Lets clients set up for Inkdrop keep their URL and credentials.")
                    .trailing(
                        Button::new("copy-inkdrop")
                            .outline()
                            .small()
                            .label("Copy from Inkdrop…")
                            .on_click(|_, window, cx| copy_from_inkdrop(window, cx)),
                    ),
            ],
            colors,
        ),
        section(
            "MCP",
            vec![
                Row::new(Lucide::Link, "MCP endpoint")
                    .value(endpoint.clone())
                    .trailing(
                        Button::new("copy-mcp")
                            .outline()
                            .small()
                            .label("Copy")
                            .on_click(move |_, window, cx| {
                                copy_to_clipboard(
                                    endpoint.clone(),
                                    "MCP endpoint copied.",
                                    window,
                                    cx,
                                )
                            }),
                    ),
            ],
            colors,
        ),
    ]
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let page = self.render_page(cx);
        h_flex()
            .size_full()
            .key_context("Settings")
            .track_focus(&self.root_focus)
            .text_color(colors.text)
            .child(self.render_nav(window, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .bg(colors.editor)
                    .border_l_1()
                    .border_color(colors.divider)
                    .child(
                        div()
                            .id("settings-page")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .child(
                                v_flex()
                                    .w_full()
                                    .max_w(px(PAGE_MAX_WIDTH + 2. * PAGE_INSET))
                                    // Centred, so a wide window leaves the
                                    // same room either side.
                                    .mx_auto()
                                    .px(px(PAGE_INSET))
                                    // The title sits just below the top band's
                                    // middle, as in the design.
                                    .pt(px(30.))
                                    .pb(px(40.))
                                    .child(
                                        div()
                                            .text_size(px(24.))
                                            .line_height(px(32.))
                                            .font_semibold()
                                            .text_color(colors.text)
                                            .child(self.page.title()),
                                    )
                                    .children(page),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .h(px(BOTTOM_BAND_HEIGHT))
                            .px(px(PAGE_INSET))
                            .border_t_1()
                            .border_color(colors.divider)
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child("Changes apply right away."),
                    ),
            )
    }
}

// MARK: Rows

/// One row of a section: an icon, a title with a line below it, and the
/// controls on the right.
// no-prop-test: layout; the snapshots show every page.
struct Row {
    icon: Lucide,
    title: SharedString,
    detail: Option<SharedString>,
    /// Whether the line below is a value (one line, cut short) rather than
    /// a description (wraps).
    value: bool,
    trailing: Vec<AnyElement>,
}

impl Row {
    fn new(icon: Lucide, title: impl Into<SharedString>) -> Self {
        Row {
            icon,
            title: title.into(),
            detail: None,
            value: false,
            trailing: Vec::new(),
        }
    }

    fn detail(mut self, text: impl Into<SharedString>) -> Self {
        self.detail = Some(text.into());
        self
    }

    fn value(mut self, text: impl Into<SharedString>) -> Self {
        self.detail = Some(text.into());
        self.value = true;
        self
    }

    fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
    }

    fn trailing_all(mut self, elements: Vec<AnyElement>) -> Self {
        self.trailing.extend(elements);
        self
    }

    fn render(self, colors: &Colors) -> AnyElement {
        let Row {
            icon,
            title,
            detail,
            value,
            trailing,
        } = self;
        h_flex()
            .min_h(px(ROW_MIN_HEIGHT))
            .px(px(16.))
            .py(px(9.))
            .gap(px(14.))
            .child(
                Icon::new(icon)
                    .size(px(18.))
                    .flex_none()
                    .text_color(colors.text_secondary),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(1.))
                    .child(
                        div()
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(colors.text)
                            .child(title),
                    )
                    .when_some(detail, |el, detail| {
                        el.child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(17.))
                                .text_color(colors.text_secondary)
                                .when(value, |el| el.truncate())
                                .child(detail),
                        )
                    }),
            )
            .when(!trailing.is_empty(), |el| {
                el.child(
                    h_flex()
                        .flex_none()
                        .gap(px(8.))
                        .items_center()
                        .children(trailing),
                )
            })
            .into_any_element()
    }
}

/// A section: its title, then its rows in one bordered group with dividers
/// between them.
// no-prop-test: layout; the snapshots show every page.
fn section(title: &'static str, rows: Vec<Row>, colors: &Colors) -> AnyElement {
    v_flex()
        .mt(px(26.))
        .child(
            div()
                .mb(px(10.))
                .text_size(px(14.))
                .line_height(px(20.))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(title),
        )
        .child(
            v_flex()
                .border_1()
                .border_color(colors.divider)
                .rounded(px(8.))
                .children(rows.into_iter().enumerate().map(|(ix, row)| {
                    div()
                        .when(ix > 0, |el| el.border_t_1().border_color(colors.divider))
                        .child(row.render(colors))
                })),
        )
        .into_any_element()
}

/// A page's status: a dot in the state's color, the state in a few words
/// and what it means, and an action on the right. The words carry the
/// state too, so it never rests on the color alone.
// no-prop-test: layout; the words come from `sync_status_line` and
// `api_status_line`, which are tested.
fn status_row(
    status: &StatusLine,
    tooltip: String,
    action: Option<AnyElement>,
    colors: &Colors,
) -> AnyElement {
    h_flex()
        .mt(px(16.))
        .gap(px(12.))
        .child(
            h_flex()
                .id("settings-status")
                .flex_1()
                .min_w_0()
                .gap(px(12.))
                .child(
                    div()
                        .flex_none()
                        .size(px(9.))
                        .rounded_full()
                        .bg(tone_color(status.tone, colors)),
                )
                .child(
                    v_flex()
                        .min_w_0()
                        .gap(px(1.))
                        .child(
                            div()
                                .text_size(px(15.))
                                .line_height(px(20.))
                                .font_medium()
                                .text_color(colors.text)
                                .child(status.label.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(17.))
                                .text_color(colors.text_secondary)
                                .child(status.detail.clone()),
                        ),
                )
                .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx)),
        )
        .children(action)
        .into_any_element()
}

fn value_text(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(13.5))
        .text_color(colors.text_secondary)
        .child(text.into())
}

/// Three choices side by side, the current one selected.
// no-prop-test: drawing and a click; the UI tests click it.
fn segmented<T: Copy + PartialEq + 'static>(
    id: &'static str,
    options: [(&'static str, T); 3],
    current: T,
    on_pick: fn(T, &mut Window, &mut App),
    colors: &Colors,
) -> ButtonGroup {
    ButtonGroup::new(id)
        .outline()
        .small()
        .children(options.iter().enumerate().map(|(ix, (label, value))| {
            let selected = *value == current;
            Button::new((id, ix))
                .selected(selected)
                .debug_selector(move || format!("{id}-{ix}"))
                .child(
                    div()
                        .px_1()
                        .text_size(px(12.5))
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(*label),
                )
        }))
        .on_click(move |clicked: &Vec<usize>, window, cx| {
            if let Some(&ix) = clicked.first() {
                on_pick(options[ix].1, window, cx);
            }
        })
}

// MARK: Settings in the workspace

impl Workspace {
    pub fn settings_open(&self) -> bool {
        self.settings_return.is_some()
    }

    /// Shows Settings in place of the list and the editor, at `page` or at
    /// the page shown last.
    pub fn open_settings(
        &mut self,
        page: Option<SettingsPage>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // The palette gives the keyboard back first, so going back lands
        // where it was before the palette.
        self.palette
            .update(cx, |palette, cx| palette.close(window, cx));
        if self.settings_return.is_none() {
            self.settings_return = Some(SettingsReturn {
                focus: window.focused(cx),
                focus_mode: self.focus_mode,
            });
            self.set_focus_mode(false, window, cx);
            self.sidebar
                .update(cx, |sidebar, cx| sidebar.set_settings_open(true, cx));
        }
        self.settings.update(cx, |settings, cx| {
            settings.show(page.unwrap_or(settings.page()), cx);
            settings.focus(window, cx);
        });
        cx.notify();
    }

    /// Back to the notes as they were: the same note, scrolled the same, with
    /// the keyboard where it was, in focus mode if it was on.
    pub fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(back) = self.settings_return.take() else {
            return;
        };
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_settings_open(false, cx));
        if back.focus_mode {
            self.set_focus_mode(true, window, cx);
        }
        window.focus(back.focus.as_ref().unwrap_or(&self.focus), cx);
        cx.notify();
    }

    /// Settings give way to notes something asked for (a place in the
    /// sidebar, a note from the palette, a new note). The keyboard stays
    /// where that put it, or goes to the list.
    pub fn leave_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings_return.take().is_none() {
            return;
        }
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_settings_open(false, cx));
        if self.settings.read(cx).contains_focus(window, cx) {
            self.note_list
                .update(cx, |list, cx| list.focus_list(window, cx));
        }
        cx.notify();
    }
}

/// The workspace's handlers for opening and closing Settings.
// no-prop-test: wiring; the UI tests send each action.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    let page = |page: SettingsPage| {
        move |this: &mut Workspace, window: &mut Window, cx: &mut Context<Workspace>| {
            this.open_settings(Some(page), window, cx)
        }
    };
    let general = page(SettingsPage::General);
    let editor = page(SettingsPage::Editor);
    let appearance = page(SettingsPage::Appearance);
    let sync = page(SettingsPage::Sync);
    let api = page(SettingsPage::LocalApi);
    el.on_action(
        cx.listener(|this, _: &OpenSettings, window, cx| this.open_settings(None, window, cx)),
    )
    .on_action(
        cx.listener(move |this, _: &OpenGeneralSettings, window, cx| general(this, window, cx)),
    )
    .on_action(
        cx.listener(move |this, _: &OpenEditorSettings, window, cx| editor(this, window, cx)),
    )
    .on_action(
        cx.listener(move |this, _: &OpenAppearanceSettings, window, cx| {
            appearance(this, window, cx)
        }),
    )
    .on_action(cx.listener(move |this, _: &OpenSyncSettings, window, cx| sync(this, window, cx)))
    .on_action(cx.listener(move |this, _: &OpenApiSettings, window, cx| api(this, window, cx)))
    .on_action(cx.listener(|this, _: &CloseSettings, window, cx| this.close_settings(window, cx)))
}

/// Settings from the menu bar or the Dock with no main window to take the
/// action, or with another window in front: the main window comes forward,
/// opened first if need be, showing Settings.
// no-prop-test: window management; the UI tests open Settings in a window.
pub fn register_app_actions(cx: &mut App, open_main_window: fn(&mut App)) {
    fn show(page: Option<SettingsPage>, open_main_window: fn(&mut App), cx: &mut App) {
        // The action may arrive while a window is being updated, and a
        // window can't be reached from inside its own update.
        cx.defer(move |cx| {
            if !show_in_open_window(page, cx) {
                open_main_window(cx);
                show_in_open_window(page, cx);
            }
        });
    }
    cx.on_action(move |_: &OpenSettings, cx| show(None, open_main_window, cx));
    cx.on_action(move |_: &OpenGeneralSettings, cx| {
        show(Some(SettingsPage::General), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenEditorSettings, cx| {
        show(Some(SettingsPage::Editor), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenAppearanceSettings, cx| {
        show(Some(SettingsPage::Appearance), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenSyncSettings, cx| {
        show(Some(SettingsPage::Sync), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenApiSettings, cx| {
        show(Some(SettingsPage::LocalApi), open_main_window, cx)
    });
}

/// Shows Settings in the first main window and brings it forward; false
/// when there is none.
fn show_in_open_window(page: Option<SettingsPage>, cx: &mut App) -> bool {
    for handle in cx.windows() {
        let shown = handle
            .update(cx, |root, window, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return false;
                };
                let view = root.read(cx).view().clone();
                let Ok(workspace) = view.downcast::<Workspace>() else {
                    return false;
                };
                workspace.update(cx, |workspace, cx| {
                    workspace.open_settings(page, window, cx)
                });
                window.activate_window();
                true
            })
            .unwrap_or(false);
        if shown {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, SocketAddr};
    use std::path::Path;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{
        FONT_SIZES, MASK, SYNC_INTERVALS, SYNC_PROBLEM_HELP, SettingsPage, api_status_line,
        display_path, interval_label, mask_secret, mcp_endpoint, parse_port, parse_username,
        sentence, step_font_size, sync_every, sync_status_line,
    };
    use crate::config::{DEFAULT_FONT_SIZE, SyncSettings};
    use crate::services::{ApiState, SyncState};
    use crate::sidebar::{SYNC_PROBLEMS, Tone, sync_line};

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn sync_settings(endpoint: bool, enabled: bool, minutes: u64) -> SyncSettings {
        SyncSettings {
            enabled,
            endpoint: endpoint.then(|| "https://sync.example.workers.dev".to_owned()),
            interval_secs: minutes * 60,
        }
    }

    #[test]
    fn examples() {
        assert_eq!(parse_port(" 19840 "), Ok(19840));
        assert!(parse_port("1023").is_err());
        assert!(parse_port("1984o").is_err());
        assert!(parse_port("65536").is_err());
        assert_eq!(parse_username(" bob ").as_deref(), Ok("bob"));
        assert!(parse_username("bob:x").is_err());
        assert!(parse_username("  ").is_err());
    }

    #[test]
    fn mask_examples() {
        assert_eq!(mask_secret("abcdefghijkl3f9a"), "••••••••3f9a");
        assert_eq!(mask_secret("x".repeat(64).as_str()), "••••••••xxxx");
        assert_eq!(mask_secret("short"), MASK);
        assert_eq!(mask_secret(""), MASK);
        // Eleven characters: still nothing of it.
        assert_eq!(mask_secret("abcdefghijk"), MASK);
        assert_eq!(mask_secret("ünïcödé-tökén-ÿ"), "••••••••én-ÿ");
    }

    #[test]
    fn sync_status_examples() {
        let on = sync_settings(true, true, 5);
        let synced = |conflicts| SyncState::Synced {
            at_ms: 0,
            conflicts,
            conflicted: Vec::new(),
        };
        let line = sync_status_line(&synced(0), &on, 10_000, None);
        assert_eq!(
            (line.tone, line.label.as_str()),
            (Tone::Good, "Synced just now")
        );
        assert_eq!(
            line.detail,
            "Kanso syncs every 5 minutes and soon after you edit."
        );
        assert_eq!(
            sync_status_line(&SyncState::Syncing, &on, 0, None).label,
            "Syncing…"
        );
        let conflict = sync_status_line(&synced(2), &on, 0, None);
        assert_eq!(
            (conflict.tone, conflict.label.as_str()),
            (Tone::Warning, "Conflict detected")
        );
        assert!(
            conflict.detail.starts_with("2 notes"),
            "{}",
            conflict.detail
        );
        // A conflict names its note when the note is known.
        let named = sync_status_line(&synced(1), &on, 0, Some("Plans"));
        assert!(
            named.detail.starts_with("“Plans” changed on two Macs"),
            "{}",
            named.detail
        );
        let named = sync_status_line(&synced(2), &on, 0, Some("Plans"));
        assert!(
            named.detail.starts_with("“Plans” and 1 other note changed"),
            "{}",
            named.detail
        );
        let named = sync_status_line(&synced(3), &on, 0, Some("Plans"));
        assert!(
            named
                .detail
                .starts_with("“Plans” and 2 other notes changed"),
            "{}",
            named.detail
        );
        let offline = sync_status_line(
            &SyncState::Failed("error sending request: connection refused".into()),
            &on,
            0,
            None,
        );
        assert_eq!(offline.label, "Offline");
        assert!(
            offline.detail.contains("couldn't reach"),
            "{}",
            offline.detail
        );
        let off = sync_status_line(&SyncState::Off, &sync_settings(false, false, 5), 0, None);
        assert_eq!((off.tone, off.label.as_str()), (Tone::Quiet, "Sync off"));
        assert!(off.detail.contains("Worker URL"), "{}", off.detail);
        let paused = sync_status_line(&synced(0), &sync_settings(true, false, 5), 0, None);
        assert!(
            paused.detail.contains("Automatic sync is off"),
            "{}",
            paused.detail
        );
        // Synced once, then the Worker URL was taken away.
        let unset = sync_status_line(&synced(0), &sync_settings(false, true, 5), 0, None);
        assert_eq!(unset.detail, "Add a Worker URL to sync again.");
        assert_eq!(
            sync_status_line(&synced(0), &sync_settings(true, true, 60), 0, None).detail,
            "Kanso syncs every hour and soon after you edit."
        );
    }

    #[test]
    fn api_status_examples() {
        let running = api_status_line(&ApiState::Running("http://127.0.0.1:19840".into()));
        assert_eq!(
            (running.tone, running.label.as_str()),
            (Tone::Good, "Running")
        );
        assert!(running.detail.contains("http://127.0.0.1:19840"));
        let taken = api_status_line(&ApiState::PortInUse(19840));
        assert_eq!(taken.label, "Port 19840 is in use");
        assert!(taken.detail.starts_with("Quit Inkdrop to free it"));
        assert_eq!(api_status_line(&ApiState::Off).tone, Tone::Quiet);
        let unset = api_status_line(&ApiState::Failed(
            "set a username and password first".into(),
        ));
        assert_eq!(unset.detail, "Set a username and password below.");
    }

    #[test]
    fn endpoint_examples() {
        assert_eq!(
            mcp_endpoint("127.0.0.1", 19840),
            "http://127.0.0.1:19840/mcp"
        );
        assert_eq!(mcp_endpoint("0.0.0.0", 4000), "http://127.0.0.1:4000/mcp");
        assert_eq!(mcp_endpoint("::", 4000), "http://[::1]:4000/mcp");
        assert_eq!(mcp_endpoint("fe80::1", 4000), "http://[fe80::1]:4000/mcp");
        assert_eq!(
            mcp_endpoint("not an ip", 19840),
            "http://127.0.0.1:19840/mcp"
        );
    }

    #[test]
    fn small_examples() {
        assert_eq!(
            SYNC_INTERVALS.map(interval_label),
            [
                "1 minute",
                "5 minutes",
                "15 minutes",
                "30 minutes",
                "1 hour",
                "4 hours"
            ]
        );
        assert_eq!(sync_every(1), "every minute");
        assert_eq!(sync_every(15), "every 15 minutes");
        assert_eq!(step_font_size(16., 1), 17.);
        assert_eq!(step_font_size(28., 1), 28.);
        assert_eq!(step_font_size(10., -1), 10.);
        assert_eq!(step_font_size(15.6, 0), 16.);
        assert_eq!(step_font_size(f32::NAN, 0), DEFAULT_FONT_SIZE);
        assert_eq!(
            sentence("the URL must start with https://"),
            "The URL must start with https://."
        );
        assert_eq!(sentence("done!"), "Done!");
        assert_eq!(sentence("éclair"), "Éclair.");
        assert_eq!(sentence("  "), "");
        assert_eq!(
            display_path(Path::new("/Users/me/Library/Kanso"), Path::new("/Users/me")),
            "~/Library/Kanso"
        );
        assert_eq!(
            display_path(Path::new("/Users/me"), Path::new("/Users/me")),
            "~"
        );
        assert_eq!(
            display_path(Path::new("/tmp/Kanso"), Path::new("/Users/me")),
            "/tmp/Kanso"
        );
        // A name that only starts like the home folder isn't inside it.
        assert_eq!(
            display_path(Path::new("/Users/meg/Kanso"), Path::new("/Users/me")),
            "/Users/meg/Kanso"
        );
        assert_eq!(SettingsPage::General.step(-1), SettingsPage::General);
        assert_eq!(SettingsPage::General.step(3), SettingsPage::Sync);
        assert_eq!(SettingsPage::LocalApi.step(1), SettingsPage::LocalApi);
    }

    /// Every sidebar problem has its own help, so none falls through to
    /// another's.
    #[test]
    fn every_sync_problem_has_help() {
        assert_eq!(SYNC_PROBLEMS.len(), SYNC_PROBLEM_HELP.len());
        let reasons = [
            "set the Worker URL in Settings › Sync",
            "no access token; set one in Settings › Sync",
            "the sync server rejected the token",
            "could not reach the sync server: io: failed to lookup address information: nodename nor servname provided, or not known",
            "the server said 500",
        ];
        let settings = sync_settings(true, true, 5);
        for (ix, reason) in reasons.into_iter().enumerate() {
            let line = sync_status_line(&SyncState::Failed(reason.into()), &settings, 0, None);
            assert_eq!(line.label, SYNC_PROBLEMS[ix]);
            assert_eq!(line.detail, SYNC_PROBLEM_HELP[ix]);
        }
    }

    fn sync_state() -> impl Strategy<Value = SyncState> {
        prop_oneof![
            Just(SyncState::Off),
            Just(SyncState::Syncing),
            (0i64..1_000_000_000, 0usize..4).prop_map(|(at_ms, conflicts)| SyncState::Synced {
                at_ms,
                conflicts,
                conflicted: Vec::new()
            }),
            // Reasons start with letters no fixed sentence has, so one in
            // the output can only have come from the reason.
            "zq[a-z0-9 :;/._-]{4,40}".prop_map(SyncState::Failed),
            (
                "zq[a-z ]{0,10}",
                prop::sample::select(vec!["token", "connect", "worker url", "500"])
            )
                .prop_map(|(a, b)| SyncState::Failed(format!("{a}{b}"))),
        ]
    }

    proptest! {
        #![proptest_config(config())]

        /// Round-trip: every usable port is accepted as typed, and nothing
        /// outside 1024..=65535 is.
        #[test]
        fn prop_ports_round_trip(port in any::<u16>()) {
            let parsed = parse_port(&port.to_string());
            prop_assert_eq!(parsed.is_ok(), port >= 1024);
            if let Ok(parsed) = parsed {
                prop_assert_eq!(parsed, port);
            }
        }

        /// Refinement: an accepted username is trimmed, non-empty and
        /// usable in Basic authentication.
        #[test]
        fn prop_usernames_fit_basic_auth(value in "\\PC{0,12}|[a-z:]{0,6}") {
            if let Ok(name) = parse_username(&value) {
                prop_assert!(!name.is_empty() && !name.contains(':'));
                prop_assert_eq!(name.trim(), name.as_str());
            }
        }

        /// The mask: the same dots whatever the length, then at most the
        /// last four characters, and only of a secret of twelve or more. Two
        /// secrets that end alike look alike, however they begin or how
        /// long they are: nothing else of a secret gets through.
        #[test]
        fn prop_masks_show_only_a_short_tail(
            head_a in "\\PC{0,40}",
            head_b in "\\PC{0,40}",
            tail in "\\PC{4}",
        ) {
            let (a, b) = (format!("{head_a}{tail}"), format!("{head_b}{tail}"));
            for secret in [&a, &b] {
                let masked = mask_secret(secret);
                let shown = masked.strip_prefix(MASK).expect("starts with the dots");
                prop_assert!(shown.chars().count() <= 4);
                prop_assert!(secret.ends_with(shown));
                if secret.chars().count() < 12 {
                    prop_assert_eq!(shown, "");
                }
            }
            if a.chars().count() >= 12 && b.chars().count() >= 12 {
                prop_assert_eq!(mask_secret(&a), mask_secret(&b));
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The Sync page says what the sidebar says, with the same color,
        /// and never repeats a failure's raw reason.
        #[test]
        fn prop_sync_status_agrees_with_the_sidebar(
            state in sync_state(),
            endpoint: bool,
            enabled: bool,
            minutes in 1u64..2000,
            ago in 0i64..100_000_000,
        ) {
            let settings = sync_settings(endpoint, enabled, minutes);
            let now = match &state { SyncState::Synced { at_ms, .. } => at_ms + ago, _ => ago };
            let line = sync_status_line(&state, &settings, now, None);
            prop_assert_eq!((line.tone, line.label.clone()), sync_line(&state, now));
            prop_assert!(!line.detail.is_empty());
            prop_assert!(!line.detail.contains("zq"), "{}", line.detail);
            // It promises syncing on its own only when it will, and then
            // says how often.
            let promises = line.detail.contains("Kanso syncs");
            prop_assert!(!promises || (endpoint && enabled), "{}", line.detail);
            let settled = matches!(state, SyncState::Off | SyncState::Synced { conflicts: 0, .. });
            if settled && endpoint && enabled {
                prop_assert!(promises && line.detail.contains(&sync_every(minutes)), "{}", line.detail);
            }
            // With nothing to sync with, it says what is missing.
            if state == SyncState::Off && !endpoint {
                prop_assert!(line.detail.contains("Worker URL"), "{}", line.detail);
            }
        }

        /// The Local API status names a taken port and the way to free it,
        /// shows a running address, and keeps a failure's reason out of
        /// sight.
        #[test]
        fn prop_api_status_reads_plainly(port in 1024u16.., addr in "http://127\\.0\\.0\\.1:[0-9]{4,5}", reason in "zq[a-z0-9 :._-]{0,40}") {
            let taken = api_status_line(&ApiState::PortInUse(port));
            prop_assert!(taken.label.contains(&port.to_string()));
            prop_assert!(taken.detail.contains("Quit Inkdrop"));
            prop_assert_eq!(taken.tone, Tone::Warning);
            let running = api_status_line(&ApiState::Running(addr.clone()));
            prop_assert!(running.detail.contains(&addr));
            let failed = api_status_line(&ApiState::Failed(reason));
            prop_assert!(!failed.label.contains("zq") && !failed.detail.contains("zq"));
            prop_assert_eq!(failed.tone, Tone::Warning);
        }

        /// The MCP endpoint is an http URL ending in /mcp whose host and
        /// port read back as the API's own, or loopback for "any address".
        #[test]
        fn prop_endpoint_reaches_the_api(ip in any::<IpAddr>(), port: u16) {
            let url = mcp_endpoint(&ip.to_string(), port);
            let authority = url
                .strip_prefix("http://")
                .and_then(|rest| rest.strip_suffix("/mcp"))
                .expect("http://…/mcp");
            let addr: SocketAddr = authority.parse().expect("host and port");
            prop_assert_eq!(addr.port(), port);
            if ip.is_unspecified() {
                prop_assert!(addr.ip().is_loopback());
                prop_assert_eq!(addr.is_ipv6(), ip.is_ipv6());
            } else {
                prop_assert_eq!(addr.ip(), ip);
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Round-trip: an interval's label reads back as the same number of
        /// minutes, singular exactly for one.
        #[test]
        fn prop_interval_labels_read_back(minutes in 1u64..100_000) {
            let label = interval_label(minutes);
            let (count, unit) = label.split_once(' ').expect("a count and a unit");
            let count: u64 = count.parse().expect("a number");
            let back = match unit {
                "minute" | "minutes" => count,
                "hour" | "hours" => count * 60,
                other => panic!("unit {other}"),
            };
            prop_assert_eq!(back, minutes);
            prop_assert_eq!(unit.ends_with('s'), count != 1);
        }

        /// Text size steps stay whole points in range, move the way asked,
        /// and undo each other away from the ends.
        #[test]
        fn prop_font_steps_stay_in_range(size in 0f32..40., step in -30i32..30, other in -30i32..30) {
            let stepped = step_font_size(size, step);
            prop_assert!((FONT_SIZES.0..=FONT_SIZES.1).contains(&stepped));
            prop_assert_eq!(stepped, stepped.round());
            if step <= other {
                prop_assert!(stepped <= step_font_size(size, other));
            }
            let start = size.round();
            if (FONT_SIZES.0..=FONT_SIZES.1).contains(&start)
                && (FONT_SIZES.0..=FONT_SIZES.1).contains(&(start + step as f32))
            {
                prop_assert_eq!(step_font_size(stepped, -step), start);
            }
        }

        /// A message becomes one sentence: its first letter capitalized,
        /// the rest as it was, ending in a stop.
        #[test]
        fn prop_sentences_keep_the_words(message in "\\PC{0,40}") {
            let out = sentence(&message);
            let trimmed = message.trim();
            if trimmed.is_empty() {
                prop_assert_eq!(out, "");
            } else {
                let mut chars = trimmed.chars();
                let first = chars.next().unwrap();
                let rest: String = chars.collect();
                prop_assert!(out.starts_with(&first.to_uppercase().collect::<String>()));
                prop_assert!(out.contains(rest.as_str()));
                prop_assert!(out.ends_with(['.', '!', '?']));
                prop_assert!(out.chars().count() <= trimmed.chars().count() + 3);
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Paths in the home folder read with `~` and expand back to the
        /// same path; others show in full.
        #[test]
        fn prop_display_paths_expand_back(home in "(/[a-z]{1,6}){1,3}", rest in "(/[a-zA-Z .]{1,8}){0,3}", outside: bool) {
            let home_path = Path::new(&home);
            let path = if outside { format!("/elsewhere{rest}") } else { format!("{home}{rest}") };
            let shown = display_path(Path::new(&path), home_path);
            let expanded = match shown.strip_prefix('~') {
                Some(tail) => format!("{home}{tail}"),
                None => shown.clone(),
            };
            prop_assert_eq!(Path::new(&expanded), Path::new(&path));
            prop_assert_eq!(shown.starts_with('~'), Path::new(&path).starts_with(home_path));
        }

        /// The page column: a step stays on a page, standing still is a
        /// no-op, and a step there and back returns unless an end stopped it.
        #[test]
        fn prop_page_steps_stop_at_the_ends(start in 0usize..5, by in -6isize..6) {
            let page = SettingsPage::ALL[start];
            prop_assert_eq!(page.step(0), page);
            let moved = page.step(by);
            let target = start as isize + by;
            if (0..5).contains(&target) {
                prop_assert_eq!(moved, SettingsPage::ALL[target as usize]);
                prop_assert_eq!(moved.step(-by), page);
            } else {
                prop_assert_eq!(moved, if by < 0 { SettingsPage::General } else { SettingsPage::LocalApi });
            }
        }
    }
}
