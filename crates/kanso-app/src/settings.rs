//! Settings, inside the main window: the sidebar stays, a column lists the
//! pages (General, Editor, Appearance, Account, Local API) where the note list
//! was, and the page takes the editor's place, in sections of bordered rows.
//! ⌘, opens it; Escape or Back to notes returns to the notes as they were.
//!
//! Switches and menus apply at once. Addresses, ports and names are changed
//! in a prompt and applied only when they check out, so the API doesn't
//! restart on every keystroke. Passwords are never shown. The Account page
//! is its own view, in `account_page.rs`.

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

use kanso_sync::{QuotaKind, SyncProblem};

use crate::account_page::AccountPage;
use crate::account_words::{quota_words, signed_out_sentence};
use crate::clock::Clock;
use crate::config::{DEFAULT_FONT_SIZE, EditorFont, EditorSettings, SyncSettings, Typewriter};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::Library;
use crate::services::{self, AccountState, ApiState, Services, SyncState};
use crate::sidebar::{Tone, sync_line, tone_color};
use crate::text_stats::format_count;
use crate::theme::Colors;
use crate::word_goal::PRESETS;
use crate::workspace::{AppState, OpenSettings, Workspace, for_each_workspace};
use crate::{dialogs, inkdrop, launch, paths};

actions!(
    kanso,
    [
        OpenGeneralSettings,
        OpenEditorSettings,
        OpenAppearanceSettings,
        OpenAccountSettings,
        OpenApiSettings,
        OpenUpdateSettings,
        CloseSettings,
        NextSettingsPage,
        PreviousSettingsPage,
        ReloadThemes,
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
    Account,
    LocalApi,
    Updates,
}

impl SettingsPage {
    /// In the order the column lists them.
    pub const ALL: [SettingsPage; 6] = [
        SettingsPage::General,
        SettingsPage::Editor,
        SettingsPage::Appearance,
        SettingsPage::Account,
        SettingsPage::LocalApi,
        SettingsPage::Updates,
    ];

    pub fn title(self) -> &'static str {
        match self {
            SettingsPage::General => "General",
            SettingsPage::Editor => "Editor",
            SettingsPage::Appearance => "Themes",
            SettingsPage::Account => "Account",
            SettingsPage::LocalApi => "Local API",
            SettingsPage::Updates => "Updates",
        }
    }

    fn icon(self) -> Lucide {
        match self {
            SettingsPage::General => Lucide::Settings,
            SettingsPage::Editor => Lucide::Pencil,
            SettingsPage::Appearance => Lucide::Palette,
            SettingsPage::Account => Lucide::CircleUser,
            SettingsPage::LocalApi => Lucide::Plug,
            SettingsPage::Updates => Lucide::CircleArrowDown,
        }
    }

    fn id(self) -> &'static str {
        match self {
            SettingsPage::General => "settings-general",
            SettingsPage::Editor => "settings-editor",
            SettingsPage::Appearance => "settings-appearance",
            SettingsPage::Account => "settings-account",
            SettingsPage::LocalApi => "settings-api",
            SettingsPage::Updates => "settings-updates",
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

/// What a saved password shows instead of itself, whatever its length.
pub const MASK: &str = "••••••••";

/// A status line for the top of a page: a colored dot with a few words, and
/// a sentence that says what it means or what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub tone: Tone,
    pub label: String,
    pub detail: String,
}

/// What to do about a sync problem, as Settings › Account and the sidebar's
/// tooltip say it (docs/ACCOUNTS.md › Problems). `resume` names when a
/// pause ends, to follow "Sync resumes".
pub fn problem_help(problem: &SyncProblem, resume: impl FnOnce(i64) -> String) -> String {
    match problem {
        SyncProblem::SignedOut(reason) => format!(
            "{} Sign in again to keep syncing.",
            signed_out_sentence(*reason)
        ),
        SyncProblem::AccountDisabled => {
            "This account is turned off on the server. Ask the server's owner.".to_owned()
        }
        SyncProblem::Paused { until } => format!(
            "This account made a lot of changes today. Sync resumes {}.",
            resume(*until)
        ),
        SyncProblem::ServerBusy { until } => format!(
            "The server has taken all the changes it can for today. Sync resumes {}.",
            resume(*until)
        ),
        SyncProblem::BackOff { .. } => {
            "This device sent the server a lot of requests at once. Sync resumes in a moment."
                .to_owned()
        }
        SyncProblem::QuotaExceeded(QuotaKind::Server) | SyncProblem::ServerFull => {
            "The server is out of space. Ask its owner.".to_owned()
        }
        SyncProblem::QuotaExceeded(kind) => format!(
            "Your account is full ({}). Delete something or ask the server's owner for more space.",
            quota_words(*kind)
        ),
        SyncProblem::Skipped { count: 1 } => {
            "1 item on the server couldn't be read. It was skipped; your copy here is unchanged."
                .to_owned()
        }
        SyncProblem::Skipped { count } => format!(
            "{count} items on the server couldn't be read. They were skipped; your copies here are unchanged."
        ),
        SyncProblem::UpdateKanso => {
            "Another device uses a newer Kanso. Update this one to keep syncing.".to_owned()
        }
        SyncProblem::WrongKey => "The server's account keys don't match this device's. Sync is stopped; your notes are safe here. Sign out and in again.".to_owned(),
        SyncProblem::ServerOutdated => {
            "That server doesn't run this version of Kanso sync.".to_owned()
        }
        SyncProblem::Offline => {
            "Can't reach the server. Your notes are safe on this device.".to_owned()
        }
        SyncProblem::Other => "Something went wrong. Try Sync now.".to_owned(),
    }
}

/// What the Account page's status says while signed out.
pub const NOT_SIGNED_IN: &str = "Not signed in";
pub const SIGNED_OUT_DETAIL: &str =
    "Sync your notes between your devices, end-to-end encrypted. Your notes stay on this device either way.";

/// The Account page's status: the sidebar's words ("Synced just now",
/// "Syncing…", a problem's label) and what they mean here, with `when`
/// naming the last sync's time as the sidebar does (see [`sync_line`]) and
/// `resume` the end of a pause (see [`problem_help`]). Signed out, it reads
/// "Not signed in". A conflict names the note it was in, by
/// `conflicted_title` when that is known.
pub fn sync_status_line(
    state: &SyncState,
    account: &AccountState,
    settings: &SyncSettings,
    when: impl FnOnce(i64) -> String,
    resume: impl FnOnce(i64) -> String,
    conflicted_title: Option<&str>,
) -> StatusLine {
    let (tone, label) = sync_line(state, account, when);
    let every = sync_every((settings.interval_secs / 60).max(1));
    let (label, detail) = match account {
        AccountState::SignedOut => (NOT_SIGNED_IN.to_owned(), SIGNED_OUT_DETAIL.to_owned()),
        AccountState::SessionEnded { reason, .. } => (
            label,
            problem_help(&SyncProblem::SignedOut(*reason), resume),
        ),
        AccountState::Busy(_) => (
            label,
            "This takes a few seconds. Your notes stay on this device.".to_owned(),
        ),
        AccountState::SignedIn { .. } => {
            let detail = match state {
                SyncState::Failed(problem)
                | SyncState::Synced {
                    problem: Some(problem),
                    ..
                } => problem_help(problem, resume),
                SyncState::Syncing => {
                    "Sending and fetching changes. Writing carries on as usual.".to_owned()
                }
                SyncState::Off | SyncState::Synced { conflicts: 0, .. } if settings.enabled => {
                    format!("Kanso syncs {every} and soon after you edit.")
                }
                SyncState::Off => {
                    "Turn on Sync automatically, or sync once with Sync now.".to_owned()
                }
                SyncState::Synced { conflicts: 0, .. } => {
                    "Automatic sync is off; Sync now syncs again.".to_owned()
                }
                SyncState::Synced { conflicts: 1, .. } => match conflicted_title {
                    Some(title) => format!(
                        "“{title}” changed on two devices at once. Both versions are kept: the other one is in its history."
                    ),
                    None => "A note changed on two devices at once. Both versions are kept: the other one is in the note's history.".to_owned(),
                },
                SyncState::Synced { conflicts, .. } => match conflicted_title {
                    Some(title) => format!(
                        "“{title}” and {} other {} changed on two devices at once. Both versions of each are kept: the others are in their history.",
                        conflicts - 1,
                        if *conflicts == 2 { "note" } else { "notes" },
                    ),
                    None => format!(
                        "{conflicts} notes changed on two devices at once. Both versions of each are kept: the others are in their history."
                    ),
                },
            };
            (label, detail)
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
            "Turn it on to let local agents and other clients reach your notes.".to_owned(),
        ),
        ApiState::Running(addr) => (
            Tone::Good,
            "Running".to_owned(),
            format!("Listening on {addr}. Compatible clients and MCP servers can connect."),
        ),
        ApiState::PortInUse(port) => (
            Tone::Warning,
            format!("Port {port} is in use"),
            "Quit the app using it, or choose another port below. Kanso keeps trying.".to_owned(),
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
pub(crate) fn sync_every(minutes: u64) -> String {
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
        let writing = config(cx).editor;
        for_each_workspace(cx, |workspace, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.set_settings(writing, cx);
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
    page: SettingsPage,
    /// The Account page, with its forms.
    pub(crate) account: Entity<AccountPage>,
    /// The page column's: up and down move between pages there.
    nav_focus: FocusHandle,
    /// The whole view's, to tell whether the keyboard is somewhere in it.
    root_focus: FocusHandle,
    scroll: ScrollHandle,
    /// Room kept at the left of the top band, for the window's traffic
    /// lights when the sidebar is hidden.
    leading_inset: Pixels,
    backing_up: bool,
    /// The quick capture shortcut's control.
    shortcut: Entity<crate::settings_capture::ShortcutRecorder>,
    /// Shown instead of the data folder, whose temporary path in a snapshot
    /// changes from run to run.
    #[cfg(test)]
    pub(crate) data_dir_label: Option<String>,
    _tick: Task<()>,
}

impl SettingsView {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        let account = cx.new(|cx| AccountPage::new(library, window, cx));
        SettingsView {
            page: SettingsPage::default(),
            account,
            // In the Tab order, so the keyboard can come back to it from
            // a page's controls to switch pages.
            nav_focus: cx.focus_handle().tab_stop(true),
            root_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            leading_inset: px(0.),
            backing_up: false,
            shortcut: cx.new(|_| crate::settings_capture::ShortcutRecorder::new()),
            #[cfg(test)]
            data_dir_label: None,
            _tick: tick,
        }
    }

    pub fn page(&self) -> SettingsPage {
        self.page
    }

    /// Shows `page` from its top. The Account page brings its storage and
    /// devices up to date each time it shows.
    pub fn show(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        if page == SettingsPage::Account {
            self.account.update(cx, |account, cx| account.shown(cx));
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

    /// Whether the page column has the keyboard.
    #[cfg(test)]
    pub fn nav_focused(&self, window: &Window) -> bool {
        self.nav_focus.is_focused(window)
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
        let now = Clock::now_ms(cx);
        // Beside the daily backups but out of their rotation, so a copy
        // made on request never costs an older daily one.
        let backup = cx.background_spawn({
            let dir = dir.clone();
            async move { store.backup_now(&dir, now).map(drop) }
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
                        eprintln!("kanso: backing up to {}: {err:#}", dir.display());
                        Notification::error(
                            "Couldn't back up your notes. Check there is free space on this device, then try again.",
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
            SettingsPage::Appearance => crate::settings_themes::page(cx),
            SettingsPage::Account => vec![self.account.clone().into_any_element()],
            SettingsPage::LocalApi => api_page(cx),
            SettingsPage::Updates => crate::updates::page(cx),
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
        let startup = launch::SUPPORTED.then(|| {
            section(
                "Startup",
                vec![
                    Row::new(Lucide::Power, "Open Kanso at login")
                        .detail(if cfg!(target_os = "macos") {
                            "Kanso starts when you log in to this Mac, with the local API."
                        } else {
                            "Kanso starts when you log in, with the local API."
                        })
                        .trailing(
                            Switch::new("launch-at-login")
                                .checked(launch_at_login)
                                .on_click(|on, window, cx| set_launch_at_login(*on, window, cx)),
                        ),
                ],
                colors,
            )
        });
        startup.into_iter().chain([
            crate::settings_capture::capture_section(&self.shortcut, cx),
            crate::settings_capture::journal_section(cx),
            crate::reminders::reminders_section(cx),
            section(
                "Data",
                vec![
                    Row::new(Lucide::FolderOpen, "Data folder")
                        .value(shown)
                        .trailing(
                            Button::new("reveal-data")
                                .outline()
                                .small()
                                .label(paths::SHOW_IN_FILE_MANAGER)
                                .on_click(move |_, _, cx| cx.reveal_path(&dir)),
                        ),
                    Row::new(Lucide::DatabaseBackup, "Backups")
                        .detail("Kanso backs up its database once a day and keeps the last 14. A backup made with Back up now is kept until you delete it.")
                        .trailing(
                            Button::new("backup-now")
                                .outline()
                                .small()
                                .label("Back up now")
                                .loading(self.backing_up)
                                .debug_selector(|| "backup-now".into())
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
        ])
        .collect()
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
                .detail("Keeps lines to a comfortable length in the editor and the preview; off lets text fill the pane.")
                .trailing(
                    div().debug_selector(|| "readable-width".into()).child(
                        Switch::new("readable-width")
                            .checked(appearance.readable_width)
                            .on_click(|on, _, cx| set_readable_width(*on, cx)),
                    ),
                ),
        ],
        colors,
    ),
    writing_section(cx)]
}

/// Settings › Editor › Writing: the aids that act as you type.
// no-prop-test: layout and clicks; the settings snapshots show it and
// `config` tests the settings' defaults and round trip.
fn writing_section(cx: &mut Context<SettingsView>) -> AnyElement {
    let colors = Colors::get(cx);
    let writing = config(cx).editor;
    let switch = |id: &'static str, on: bool, set: fn(&mut EditorSettings, bool)| {
        div().debug_selector(move || id.into()).child(
            Switch::new(id)
                .checked(on)
                .on_click(move |on, _, cx| set_writing(cx, |editor| set(editor, *on))),
        )
    };
    let goal_label = match writing.word_goal {
        Some(n) => format!("{} words", format_count(n as usize)),
        None => "None".to_owned(),
    };
    let current_goal = writing.word_goal;
    let goal_menu = Button::new("default-goal")
        .debug_selector(|| "default-goal".into())
        .outline()
        .small()
        .label(goal_label)
        .dropdown_caret(true)
        .dropdown_menu(move |mut menu, _, _| {
            menu = menu.item(
                PopupMenuItem::new("None")
                    .checked(current_goal.is_none())
                    .on_click(|_, _, cx| set_writing(cx, |editor| editor.word_goal = None)),
            );
            for n in PRESETS {
                menu = menu.item(
                    PopupMenuItem::new(format!("{} words", format_count(n as usize)))
                        .checked(current_goal == Some(n))
                        .on_click(move |_, _, cx| {
                            set_writing(cx, |editor| editor.word_goal = Some(n))
                        }),
                );
            }
            menu
        });
    section(
        "Writing",
        vec![
            Row::new(Lucide::Brackets, "Close brackets and markers")
                .detail("Typing ( [ { \" * _ or ` adds its closer, and wraps selected text.")
                .trailing(switch("auto-pair", writing.auto_pair, |e, on| e.auto_pair = on)),
            Row::new(Lucide::Table, "Line up tables")
                .detail("Tab and Enter in a table pad its cells so the pipes line up in monospace. Format Table does it any time.")
                .trailing(switch("format-tables", writing.format_tables, |e, on| {
                    e.format_tables = on
                })),
            Row::new(Lucide::ClipboardPaste, "Smart paste")
                .detail(format!(
                    "A link pasted over words links them; formatted text pastes as Markdown. {} pastes plain text.",
                    keys_label("secondary-shift-v")
                ))
                .trailing(switch("smart-paste", writing.smart_paste, |e, on| e.smart_paste = on)),
            Row::new(Lucide::ScrollText, "Typewriter scrolling")
                .detail("Keeps the line you're writing in the middle of the editor.")
                .trailing(segmented(
                    "typewriter",
                    [
                        ("Off", Typewriter::Off),
                        ("Focus mode", Typewriter::FocusMode),
                        ("Always", Typewriter::Always),
                    ],
                    writing.typewriter,
                    |typewriter, _, cx| set_writing(cx, |editor| editor.typewriter = typewriter),
                    colors,
                )),
            Row::new(Lucide::Goal, "Word goal")
                .detail("For notes without their own. Focus mode shows the progress; set a note's own goal from its footer.")
                .trailing(goal_menu),
            Row::new(Lucide::Hash, "Tags from #words")
                .detail("A #word in a note's text becomes one of its tags when the note is saved; #work/clients nests. Not in code, links or headings, and tags added by hand stay.")
                .trailing(switch("inline-tags", writing.inline_tags, |e, on| e.inline_tags = on)),
            Row::new(Lucide::Keyboard, "Vim keys")
                .detail(format!(
                    "Normal, Insert and Visual modes in the note's text, shown in its footer. {} shortcuts work as always.",
                    if cfg!(target_os = "macos") { "⌘" } else { "Ctrl" }
                ))
                .trailing(switch("vim-keys", writing.vim, |e, on| e.vim = on)),
        ],
        colors,
    )
}

/// A keystroke as GPUI writes it (`secondary-shift-v`), shown as this
/// platform shows it: "⇧⌘V" on macOS, "Ctrl+Shift+V" elsewhere.
fn keys_label(keys: &str) -> String {
    Keystroke::parse(keys).map_or_else(|_| keys.to_owned(), |k| Kbd::format(&k))
}

fn set_writing(cx: &mut App, change: impl FnOnce(&mut EditorSettings)) {
    AppState::update_config(cx, |config| change(&mut config.editor));
    update_editors(cx);
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
                    .detail("REST and MCP for local agents. It speaks the same API as Inkdrop's local server, so clients written for that work too.")
                    .trailing(Switch::new("api-enabled").checked(server.enabled).on_click(
                        |on, _, cx| {
                            let on = *on;
                            AppState::update_config(cx, |config| config.server.enabled = on);
                            services::apply_server_settings(cx);
                        },
                    )),
                Row::new(Lucide::Network, "Address")
                    .detail("127.0.0.1 keeps the API on this device.")
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
                    .detail("19840 by default. Only one app can listen on a port at a time.")
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
                Row::new(Lucide::Copy, "Settings from Inkdrop")
                    .detail("Takes Inkdrop's port, username and password, so clients already set up for it keep working.")
                    .trailing(
                        Button::new("copy-inkdrop")
                            .outline()
                            .small()
                            .label("Use settings from Inkdrop…")
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
                                page_column()
                                    .flex()
                                    .flex_col()
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
                            .border_t_1()
                            .border_color(colors.divider)
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            // In the page's column, so the words start under
                            // its rows at any width.
                            .child(page_column().child("Changes apply right away.")),
                    ),
            )
    }
}

/// The column a page's rows sit in: centred, so a wide window leaves the
/// same room either side, and no wider than rows read well. The footer's
/// words sit in one too, to start where the rows do.
// no-prop-test: a style; the settings snapshots show it at three widths.
fn page_column() -> Div {
    div()
        .w_full()
        .max_w(px(PAGE_MAX_WIDTH + 2. * PAGE_INSET))
        .mx_auto()
        .px(px(PAGE_INSET))
}

// MARK: Rows

/// One row of a section: an icon, a title with a line below it, and the
/// controls on the right.
// no-prop-test: layout; the snapshots show every page.
pub(crate) struct Row {
    icon: Lucide,
    title: SharedString,
    detail: Option<SharedString>,
    /// Whether the line below is a value (one line, cut short) rather than
    /// a description (wraps).
    value: bool,
    trailing: Vec<AnyElement>,
}

impl Row {
    pub(crate) fn new(icon: Lucide, title: impl Into<SharedString>) -> Self {
        Row {
            icon,
            title: title.into(),
            detail: None,
            value: false,
            trailing: Vec::new(),
        }
    }

    pub(crate) fn detail(mut self, text: impl Into<SharedString>) -> Self {
        self.detail = Some(text.into());
        self
    }

    pub(crate) fn value(mut self, text: impl Into<SharedString>) -> Self {
        self.detail = Some(text.into());
        self.value = true;
        self
    }

    pub(crate) fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
    }

    pub(crate) fn trailing_all(mut self, elements: Vec<AnyElement>) -> Self {
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
pub(crate) fn section(title: &'static str, rows: Vec<Row>, colors: &Colors) -> AnyElement {
    v_flex()
        .mt(px(26.))
        .when(!title.is_empty(), |el| {
            el.child(
                div()
                    .mb(px(10.))
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_medium()
                    .text_color(colors.text_secondary)
                    .child(title),
            )
        })
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

/// The status row's line of words, and its dot, centred on that line.
const STATUS_LINE: f32 = 20.;
const STATUS_DOT: f32 = 9.;

/// A page's status: a dot in the state's color, the state in a few words
/// and what it means, and an action on the right. The words carry the
/// state too, so it never rests on the color alone.
// no-prop-test: layout; the words come from `sync_status_line` and
// `api_status_line`, which are tested.
pub(crate) fn status_row(
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
                // The dot belongs to the state's words, on their line, however
                // many lines the meaning below takes.
                .items_start()
                .child(
                    div()
                        .flex_none()
                        .mt(px((STATUS_LINE - STATUS_DOT) / 2.))
                        .size(px(STATUS_DOT))
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
                                .line_height(px(STATUS_LINE))
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
            self.palette
                .update(cx, |palette, _| palette.set_notes_hidden(true));
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
        self.palette
            .update(cx, |palette, _| palette.set_notes_hidden(false));
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
        self.palette
            .update(cx, |palette, _| palette.set_notes_hidden(false));
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
    let account = page(SettingsPage::Account);
    let api = page(SettingsPage::LocalApi);
    let updates = page(SettingsPage::Updates);
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
    .on_action(
        cx.listener(move |this, _: &OpenAccountSettings, window, cx| account(this, window, cx)),
    )
    .on_action(cx.listener(move |this, _: &OpenApiSettings, window, cx| api(this, window, cx)))
    .on_action(
        cx.listener(move |this, _: &OpenUpdateSettings, window, cx| updates(this, window, cx)),
    )
    .on_action(cx.listener(|this, _: &CloseSettings, window, cx| this.close_settings(window, cx)))
    .on_action(
        cx.listener(|_, _: &ReloadThemes, window, cx| crate::settings_themes::reload(window, cx)),
    )
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
    cx.on_action(move |_: &OpenAccountSettings, cx| {
        show(Some(SettingsPage::Account), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenApiSettings, cx| {
        show(Some(SettingsPage::LocalApi), open_main_window, cx)
    });
    cx.on_action(move |_: &OpenUpdateSettings, cx| {
        show(Some(SettingsPage::Updates), open_main_window, cx)
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

    use kanso_sync::{QuotaKind, SignOutReason, Step, SyncProblem};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{
        FONT_SIZES, NOT_SIGNED_IN, SIGNED_OUT_DETAIL, SYNC_INTERVALS, SettingsPage,
        api_status_line, display_path, interval_label, mcp_endpoint, parse_port, parse_username,
        problem_help, step_font_size, sync_every, sync_status_line,
    };
    use crate::config::{DEFAULT_FONT_SIZE, SyncSettings};
    use crate::grouping::utc_label;
    use crate::services::{AccountState, ApiState, SyncState, strategies};
    use crate::sidebar::{Tone, sync_line};

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn sync_settings(enabled: bool, minutes: u64) -> SyncSettings {
        SyncSettings {
            enabled,
            interval_secs: minutes * 60,
            ..SyncSettings::default()
        }
    }

    fn signed_in() -> AccountState {
        AccountState::SignedIn {
            username: "ngoni".into(),
        }
    }

    /// A pause's end as the page names it, for the examples.
    fn resume(at: i64) -> String {
        format!("at {at}")
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
    fn sync_status_examples() {
        let on = sync_settings(true, 5);
        let account = signed_in();
        let line = |state: &SyncState, settings: &SyncSettings, now: i64, title: Option<&str>| {
            sync_status_line(state, &account, settings, utc_label(now), resume, title)
        };
        let synced = |conflicts| SyncState::Synced {
            at_ms: 0,
            conflicts,
            conflicted: Vec::new(),
            problem: None,
        };
        let done = line(&synced(0), &on, 10_000, None);
        assert_eq!(
            (done.tone, done.label.as_str()),
            (Tone::Good, "Synced just now")
        );
        assert_eq!(
            done.detail,
            "Kanso syncs every 5 minutes and soon after you edit."
        );
        assert_eq!(line(&SyncState::Syncing, &on, 0, None).label, "Syncing…");
        let conflict = line(&synced(2), &on, 0, None);
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
        let named = line(&synced(1), &on, 0, Some("Plans"));
        assert!(
            named.detail.starts_with("“Plans” changed on two devices"),
            "{}",
            named.detail
        );
        let named = line(&synced(2), &on, 0, Some("Plans"));
        assert!(
            named.detail.starts_with("“Plans” and 1 other note changed"),
            "{}",
            named.detail
        );
        let named = line(&synced(3), &on, 0, Some("Plans"));
        assert!(
            named
                .detail
                .starts_with("“Plans” and 2 other notes changed"),
            "{}",
            named.detail
        );
        let offline = line(&SyncState::Failed(SyncProblem::Offline), &on, 0, None);
        assert_eq!(offline.label, "Offline");
        assert_eq!(
            offline.detail,
            "Can't reach the server. Your notes are safe on this device."
        );
        let paused = line(
            &SyncState::Failed(SyncProblem::Paused { until: 7 }),
            &on,
            0,
            None,
        );
        assert_eq!(paused.label, "Sync paused");
        assert_eq!(
            paused.detail,
            "This account made a lot of changes today. Sync resumes at 7."
        );
        let manual = line(&synced(0), &sync_settings(false, 5), 0, None);
        assert!(
            manual.detail.contains("Automatic sync is off"),
            "{}",
            manual.detail
        );
        assert_eq!(
            line(&SyncState::Off, &sync_settings(false, 5), 0, None).detail,
            "Turn on Sync automatically, or sync once with Sync now."
        );
        assert_eq!(
            line(&synced(0), &sync_settings(true, 60), 0, None).detail,
            "Kanso syncs every hour and soon after you edit."
        );
        // Signed out: the page invites signing in, whatever sync last did.
        let out = sync_status_line(
            &synced(0),
            &AccountState::SignedOut,
            &on,
            utc_label(0),
            resume,
            None,
        );
        assert_eq!(
            (out.tone, out.label.as_str(), out.detail.as_str()),
            (Tone::Quiet, "Not signed in", SIGNED_OUT_DETAIL)
        );
        // The server ended the session: the reason, and what to do.
        let ended = sync_status_line(
            &SyncState::Off,
            &AccountState::SessionEnded {
                username: "ngoni".into(),
                reason: SignOutReason::PasswordChanged,
            },
            &on,
            utc_label(0),
            resume,
            None,
        );
        assert_eq!(
            (ended.tone, ended.label.as_str()),
            (Tone::Warning, "Signed out")
        );
        assert_eq!(
            ended.detail,
            "Your password was changed on another device. Sign in again to keep syncing."
        );
        let busy = sync_status_line(
            &SyncState::Off,
            &AccountState::Busy(Step::Contacting),
            &on,
            utc_label(0),
            resume,
            None,
        );
        assert_eq!(
            (busy.tone, busy.label.as_str()),
            (Tone::Busy, "Contacting the server…")
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
        assert!(taken.detail.starts_with("Quit the app using it"));
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
        assert_eq!(SettingsPage::General.step(3), SettingsPage::Account);
        assert_eq!(SettingsPage::LocalApi.step(1), SettingsPage::Updates);
        assert_eq!(SettingsPage::Updates.step(1), SettingsPage::Updates);
        assert_eq!(SettingsPage::Account.title(), "Account");
    }

    /// Every problem in the table has its help, as the table words it.
    #[test]
    fn every_sync_problem_has_help() {
        let cases = [
            (
                SyncProblem::SignedOut(SignOutReason::Revoked),
                "This device was signed out from another device. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Recovered),
                "Your account was recovered with its recovery key. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Evicted),
                "Signed out to make room for another device. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Expired),
                "Signed out after 180 days without syncing. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::SignedOut(SignOutReason::Unknown),
                "This device was signed out. Sign in again to keep syncing.",
            ),
            (
                SyncProblem::AccountDisabled,
                "This account is turned off on the server. Ask the server's owner.",
            ),
            (
                SyncProblem::Paused { until: 1 },
                "This account made a lot of changes today. Sync resumes at 1.",
            ),
            (
                SyncProblem::QuotaExceeded(QuotaKind::DocBytes),
                "Your account is full (notes and notebooks). Delete something or ask the server's owner for more space.",
            ),
            (
                SyncProblem::QuotaExceeded(QuotaKind::Files),
                "Your account is full (number of attachments). Delete something or ask the server's owner for more space.",
            ),
            (
                SyncProblem::ServerFull,
                "The server is out of space. Ask its owner.",
            ),
            (
                SyncProblem::Skipped { count: 3 },
                "3 items on the server couldn't be read. They were skipped; your copies here are unchanged.",
            ),
            (
                SyncProblem::UpdateKanso,
                "Another device uses a newer Kanso. Update this one to keep syncing.",
            ),
            (
                SyncProblem::WrongKey,
                "The server's account keys don't match this device's. Sync is stopped; your notes are safe here. Sign out and in again.",
            ),
            (
                SyncProblem::ServerOutdated,
                "That server doesn't run this version of Kanso sync.",
            ),
            (
                SyncProblem::Offline,
                "Can't reach the server. Your notes are safe on this device.",
            ),
            (SyncProblem::Other, "Something went wrong. Try Sync now."),
        ];
        let settings = sync_settings(true, 5);
        for (problem, help) in cases {
            assert_eq!(problem_help(&problem, resume), help);
            let line = sync_status_line(
                &SyncState::Failed(problem),
                &signed_in(),
                &settings,
                utc_label(0),
                resume,
                None,
            );
            assert_eq!(line.detail, help);
        }
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

        /// Every problem's help is a sentence of its own, and a pause (the
        /// account's or the server's) names when it ends.
        #[test]
        fn prop_problem_help_says_what_to_do(problem in strategies::problem()) {
            let help = problem_help(&problem, |at| format!("at zq{at}"));
            prop_assert!(help.ends_with('.'), "{}", help);
            prop_assert_eq!(
                help.contains("zq"),
                matches!(problem, SyncProblem::Paused { .. } | SyncProblem::ServerBusy { .. }),
                "{}",
                help
            );
            if let SyncProblem::SignedOut(_) = problem {
                prop_assert!(help.ends_with("Sign in again to keep syncing."), "{}", help);
            }
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The Account page says what the sidebar says, with the same
        /// color, except that signed out it reads "Not signed in"; and it
        /// promises syncing on its own only when it will.
        #[test]
        fn prop_sync_status_agrees_with_the_sidebar(
            state in strategies::sync_state(),
            account in strategies::account(),
            enabled: bool,
            minutes in 1u64..2000,
            ago in 0i64..100_000_000,
        ) {
            let settings = sync_settings(enabled, minutes);
            let now = match &state { SyncState::Synced { at_ms, .. } => at_ms + ago, _ => ago };
            let line = sync_status_line(&state, &account, &settings, utc_label(now), resume, None);
            let (tone, label) = sync_line(&state, &account, utc_label(now));
            prop_assert_eq!(line.tone, tone);
            if account == AccountState::SignedOut {
                prop_assert_eq!(line.label.as_str(), NOT_SIGNED_IN);
                prop_assert_eq!(line.detail.as_str(), SIGNED_OUT_DETAIL);
            } else {
                prop_assert_eq!(line.label, label);
            }
            prop_assert!(!line.detail.is_empty());
            let promises = line.detail.contains("Kanso syncs");
            let signed_in = matches!(account, AccountState::SignedIn { .. });
            prop_assert!(!promises || (enabled && signed_in), "{}", line.detail);
            let settled = matches!(
                state,
                SyncState::Off | SyncState::Synced { conflicts: 0, problem: None, .. }
            );
            if settled && enabled && signed_in {
                prop_assert!(promises && line.detail.contains(&sync_every(minutes)), "{}", line.detail);
            }
            // A problem's help shows exactly while signed in with it.
            if signed_in
                && let SyncState::Failed(problem) | SyncState::Synced { problem: Some(problem), .. } = &state
            {
                prop_assert_eq!(line.detail, problem_help(problem, resume));
            }
        }

        /// The Local API status names a taken port and the way to free it,
        /// shows a running address, and keeps a failure's reason out of
        /// sight.
        #[test]
        fn prop_api_status_reads_plainly(port in 1024u16.., addr in "http://127\\.0\\.0\\.1:[0-9]{4,5}", reason in "zq[a-z0-9 :._-]{0,40}") {
            let taken = api_status_line(&ApiState::PortInUse(port));
            prop_assert!(taken.label.contains(&port.to_string()));
            prop_assert!(taken.detail.contains("Quit the app using it"));
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
        fn prop_page_steps_stop_at_the_ends(start in 0usize..6, by in -7isize..7) {
            let page = SettingsPage::ALL[start];
            prop_assert_eq!(page.step(0), page);
            let moved = page.step(by);
            let target = start as isize + by;
            if (0..6).contains(&target) {
                prop_assert_eq!(moved, SettingsPage::ALL[target as usize]);
                prop_assert_eq!(moved.step(-by), page);
            } else {
                prop_assert_eq!(moved, if by < 0 { SettingsPage::General } else { SettingsPage::Updates });
            }
        }
    }
}
