//! Renders the real views to PNG files with GPUI's Metal renderer, so the
//! UI can be reviewed without capturing the screen. Run with
//!
//! ```text
//! KANSO_SNAPSHOT_DIR=/tmp/shots [KANSO_SNAPSHOT_DB=copy-of-kanso.db] \
//!     [KANSO_SNAPSHOT_ONLY=dark-split,light-edit] \
//!     cargo test -p kanso-app snapshots -- --ignored --nocapture
//! ```
//!
//! Without `KANSO_SNAPSHOT_DB` a demo library is used, with the clock fixed
//! so every run draws the same picture. The database is copied first, so the
//! original is never written. Each shot also checks that the panes are
//! painted in their theme's surface colors.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, HeadlessAppContext, Pixels, Size,
    WindowBounds, WindowOptions, px, size,
};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::assets::KansoAssets;
use crate::clock::Clock;
use crate::config::{AppConfig, EditorFont, ThemePreference, ViewMode};
use crate::layout::{BOTTOM_BAND_HEIGHT, NOTE_LIST_WIDTH, SIDEBAR_WIDTH, TOP_BAND_HEIGHT};
use crate::library::Filter;
use crate::services::{ApiState, Services, SyncState};
use crate::theme::{Colors, DARK, LIGHT, Rgb};
use crate::workspace::{AppState, Workspace, apply_theme};

/// The demo's "now": Thursday 28 May 2026, 09:45 in a zone two hours ahead
/// of UTC.
const DEMO_NOW_MS: i64 = 1_779_954_300_000;
const DEMO_UTC_OFFSET_SECS: i32 = 2 * 3600;
const MINUTE: i64 = 60_000;
const DAY: i64 = 24 * 60 * MINUTE;

const ROADMAP: &str = "# MCP support model\n\nKanso acts as an MCP host. Clients connect to Kanso, discover the tools it offers, and ask before they act. See the [protocol overview](https://modelcontextprotocol.io) for the wire format.\n\n## Goals\n\n- Enable secure, discoverable access to notes through MCP.\n- A first-party experience that is *simple* and **reliable**.\n- Protect notes with clear permissions and an activity log.\n- Work offline when possible; degrade gracefully.\n\n## Core components\n\n1. MCP server (in Kanso)\n2. Tool registry\n3. Permission manager\n\n## Example tool definition\n\n```json\n{\n  \"name\": \"search_notes\",\n  \"description\": \"Search notes by keyword\",\n  \"inputSchema\": {\n    \"type\": \"object\",\n    \"properties\": { \"query\": { \"type\": \"string\" }, \"limit\": { \"type\": \"integer\", \"default\": 20 } }\n  }\n}\n```\n\nEach tool declares its `inputSchema`; the registry validates calls against it.\n\n## Permissions\n\n- [x] User approves each new client\n- [x] Scopes are explicit\n- [ ] Always ask for write operations\n\n> Destructive actions need review unless a client is explicitly trusted.\n\n| Permission | Default |\n|---|---|\n| Read notes | Allowed |\n| Update notes | Ask |\n| Delete notes | Ask |\n";

/// The demo note other notes link to, so its header shows backlinks.
const LINKED_NOTE: &str = "note:mcp-support";

/// A demo library for when no real database is given, with edits spread
/// over today, yesterday and older days.
fn demo_store(path: &Path) -> Store {
    let store = Store::open(path).unwrap();
    let book = |name: &str, parent: Option<&str>| {
        store
            .put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let engineering = book("Engineering", None);
    let platform = book("Platform", Some(&engineering));
    let product = book("Product", None);
    book("Research", None);
    let design = book("Design", None);
    let personal = book("Personal", None);
    let tag = |name: &str, color: &str| {
        store
            .put(
                DocType::Tag,
                json!({"name": name, "color": color}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let mcp = tag("mcp", "blue");
    let agents = tag("agents", "green");
    let security = tag("security", "red");
    let roadmap = tag("roadmap", "orange");
    let rust = tag("rust", "brown");
    let today = DEMO_NOW_MS;
    let notes = [
        (
            &engineering,
            "MCP support model",
            ROADMAP.to_owned(),
            vec![mcp.clone(), agents.clone(), security.clone()],
            "active",
            false,
            today - 4 * MINUTE,
        ),
        (
            &engineering,
            "Agent integration plan",
            "# Agent integration plan\n\nHow Kanso works with local agents and external tools; see the [MCP support model](inkdrop://note/mcp-support).\n\n- Read-only by default\n- Writes go through review\n".to_owned(),
            vec![agents.clone()],
            "active",
            false,
            today - 33 * MINUTE,
        ),
        (
            &platform,
            "Tool registry schema",
            "Schema and capabilities for discoverability.\n\n```rust\npub struct Tool {\n    pub name: String,\n    pub schema: serde_json::Value,\n}\n```\n".to_owned(),
            vec![mcp, rust.clone()],
            "onHold",
            false,
            today - 51 * MINUTE,
        ),
        (
            &product,
            "Kanso roadmap",
            "# Kanso roadmap\n\nA calm, local-first notes app.\n\n## Next\n\n- [x] Import from Inkdrop\n- [x] Local API on port 19840\n- [ ] Command palette\n- [ ] Focus mode\n".to_owned(),
            vec![roadmap.clone()],
            "active",
            true,
            today - DAY - 3 * 60 * MINUTE,
        ),
        (
            &engineering,
            "Weekly sync notes",
            "# Weekly sync\n\n| Topic | Owner | Status | Due | Risk | Next step |\n|---|---|---|---|---|---|\n| Release | Ngoni | On track | Jun 4 | Notarization queue | Submit the build |\n| Docs | Team | Drafting | Jun 11 | Screenshots out of date | Retake them after the theme lands |\n\nAction items live in the tracker.".to_owned(),
            vec![],
            "none",
            false,
            today - DAY - 7 * 60 * MINUTE,
        ),
        (
            &product,
            "Release checklist",
            "---\ntitle: Release checklist\ndate: 2026-05-25\n---\n\n# Release checklist\n\n- [x] Bump the version\n- [ ] Notarize the build\n- [ ] Write the release notes\n".to_owned(),
            vec![roadmap],
            "completed",
            false,
            today - 3 * DAY,
        ),
        (
            &platform,
            "Sync design",
            "# Sync design\n\nDocuments live in D1, attachments in R2. Last writer wins by `updatedAt`; the loser stays in history. Clients reach it through the [MCP support model](inkdrop://note/mcp-support).".to_owned(),
            vec![rust, security],
            "completed",
            false,
            today - 4 * DAY,
        ),
        (
            &design,
            "Notes on a deliberately long title that has to truncate cleanly in the list",
            "Rows, not cards. The active row gets a subtle filled state.".to_owned(),
            vec![],
            "dropped",
            false,
            today - 8 * DAY,
        ),
        (
            &personal,
            "Reading list",
            "# Reading list\n\n1. *The Design of Everyday Things*\n2. **Shape Up**\n3. Crafting Interpreters\n".to_owned(),
            vec![],
            "none",
            false,
            today - 176 * DAY,
        ),
        (
            &TRASH_BOOK_ID.to_owned(),
            "Old meeting notes",
            "Superseded by the weekly sync.".to_owned(),
            vec![],
            "none",
            false,
            today - 30 * DAY,
        ),
    ];
    for (book, title, body, tags, status, pinned, updated) in notes {
        let mut note = json!({"title": title, "body": body, "bookId": book, "tags": tags,
            "status": status, "pinned": pinned,
            "createdAt": updated - 2 * DAY, "updatedAt": updated});
        if title == "MCP support model" {
            note["_id"] = json!(LINKED_NOTE);
        }
        store
            .put(DocType::Note, note, Put::Upsert, Origin::Ui)
            .unwrap();
    }
    store
}

fn context(clock: Option<Clock>) -> HeadlessAppContext {
    let mut cx = HeadlessAppContext::with_platform(
        Arc::new(gpui_apple::AppleTextSystem::new()),
        Arc::new(KansoAssets),
        gpui_kit::platform::current_headless_renderer,
    );
    cx.update(|cx| {
        gpui_kit::init(cx);
        if let Some(clock) = clock {
            cx.set_global(clock);
        }
    });
    cx
}

/// Puts the workspace in the shot's state before drawing.
type Prepare = fn(&mut HeadlessAppContext, AnyWindowHandle, &Entity<Workspace>, &Store);

struct Shot {
    name: &'static str,
    theme: ThemePreference,
    view_mode: ViewMode,
    /// The window's size in points; the PNG is twice that.
    size: Size<Pixels>,
    prepare: Prepare,
    /// Whether the sidebar shows, for the surface checks.
    sidebar: bool,
}

/// The most recently edited note, as after a restart.
fn open_latest(_: &mut HeadlessAppContext, _: AnyWindowHandle, _: &Entity<Workspace>, _: &Store) {}

fn set_filter(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>, filter: Filter) {
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.set_filter(filter, cx)));
}

/// The Status group open on Active, while the API's port is taken (by
/// Inkdrop, say) and sync can't reach its server: the footer's warnings.
fn status_and_warnings(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_filter(
        cx,
        workspace,
        Filter::Status(kanso_core::NoteStatus::Active),
    );
    cx.update(|cx| {
        let services = cx.global_mut::<Services>();
        services.api = ApiState::PortInUse(crate::config::DEFAULT_PORT);
        services.sync = SyncState::Failed("error sending request: connection refused".into());
    });
}

/// Recent, with every tag showing and the Status group closed again.
fn recent_all_tags(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_filter(cx, workspace, Filter::Recent);
    cx.update(|cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |sidebar, cx| sidebar.show_all_tags(cx));
    });
}

/// The sidebar hidden: the note list is leftmost, below the traffic lights.
fn hide_sidebar(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.show_sidebar = false;
            cx.notify();
        })
    });
}

/// A note in the trash, with its banner.
fn open_trashed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    set_filter(cx, workspace, Filter::Trash);
    let params = crate::library::search_params(&Filter::Trash, "", Default::default(), true);
    if let Some(note) = store.search_summaries(&params).unwrap().into_iter().next() {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| library.update(cx, |library, cx| library.select(Some(note.id), cx)));
    }
}

/// Unsaved typing in the open note while another app saves it: the editor
/// asks which version to keep.
fn conflict(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let editor = cx.update(|cx| workspace.read(cx).editor.clone());
    let id = cx.update(|cx| editor.read(cx).note_id().map(str::to_owned));
    let Some(id) = id else {
        return;
    };
    cx.update_window(window, |_, window, cx| {
        editor.update(cx, |editor, cx| {
            let body = format!("{}\nA line typed here.", editor.body_for_test(cx));
            editor.edit_for_test(&body, window, cx);
        });
    })
    .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "Changed by an agent over the local API."}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
}

/// An empty notebook, "Research" when there is one.
fn empty_notebook(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books = store.books().unwrap();
    let empty = books
        .iter()
        .find(|b| b.count == 0 && b.name == "Research")
        .or_else(|| books.iter().find(|b| b.count == 0));
    if let Some(book) = empty {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| {
            library.update(cx, |library, cx| {
                library.set_filter(crate::library::Filter::Book(book.id.clone()), cx);
                library.select(None, cx);
            })
        });
    }
}

/// The note whose first table of five or more columns comes earliest, so
/// the table sits as high in the preview as the library allows.
fn open_wide_table(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    // The line of the first table head separator with five or more columns.
    let wide_table_line = |body: &str| {
        body.lines().position(|line| {
            let line = line.trim();
            line.starts_with('|')
                && line.matches('|').count() >= 6
                && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
        })
    };
    let notes = store.search_summaries(&Default::default()).unwrap();
    let found = notes
        .into_iter()
        .filter_map(|n| {
            let note = store.note(&n.id).ok().flatten()?;
            Some((wide_table_line(&note.body)?, n))
        })
        .min_by_key(|(line, _)| *line)
        .map(|(_, note)| note);
    if let Some(note) = found {
        let library = cx.update(|cx| workspace.read(cx).library.clone());
        cx.update(|cx| library.update(cx, |library, cx| library.select(Some(note.id), cx)));
    }
}

/// Types "sync" into the search box, so the field and the list agree.
fn search_sync(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let search = workspace.read(cx).note_list.read(cx).search.clone();
        search.update(cx, |search, cx| search.replace_all("sync", window, cx));
    })
    .unwrap();
}

fn mono_font(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.editor_font = EditorFont::Mono;
            cx.notify();
        });
    });
}

fn open(
    cx: &mut HeadlessAppContext,
    store: Arc<Store>,
    data_dir: PathBuf,
    shot: &Shot,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let mut config = AppConfig::default();
    config.appearance.theme = shot.theme;
    config.appearance.view_mode = shot.view_mode;
    config.last_note = store
        .search_summaries(&Default::default())
        .ok()
        .and_then(|notes| notes.into_iter().next())
        .map(|n| n.id);
    let theme = shot.theme;
    let now = cx.update(|cx| Clock::now_ms(cx));
    cx.update(|cx| {
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        // The footer as it usually reads; nothing is listening or syncing.
        let mut services = Services::default();
        services.api = ApiState::Running("http://127.0.0.1:19840".into());
        services.sync = SyncState::Synced {
            at_ms: now - 2 * MINUTE,
            conflicts: 0,
        };
        cx.set_global(services);
        apply_theme(theme, None, cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Default::default(),
                    size: shot.size,
                })),
                focus: false,
                show: false,
                ..Default::default()
            },
            cx,
            |window, cx| {
                apply_theme(theme, Some(window), cx);
                cx.new(|cx| {
                    let mut workspace = Workspace::new(window, cx);
                    workspace.placeholder_traffic_lights = true;
                    workspace
                })
            },
        )
    })
    .unwrap()
}

/// Checks that the pixel at point `at` (in window points) is `expected`,
/// within `tolerance` per channel.
fn assert_surface(
    image: &image::RgbaImage,
    scale: f32,
    at: (f32, f32),
    expected: Rgb,
    tolerance: u8,
    what: &str,
) {
    let (x, y) = ((at.0 * scale) as u32, (at.1 * scale) as u32);
    let pixel = image.get_pixel(x, y).0;
    let want = [16, 8, 0].map(|shift| ((expected.0 >> shift) & 0xFF) as u8);
    let close = (0..3).all(|i| pixel[i].abs_diff(want[i]) <= tolerance);
    assert!(
        close,
        "{what} at {at:?}: got #{:02X}{:02X}{:02X}, want {}",
        pixel[0],
        pixel[1],
        pixel[2],
        expected.hex()
    );
}

/// Each pane is painted in its own surface color. The points sit in
/// padding that no content reaches.
fn check_surfaces(image: &image::RgbaImage, shot: &Shot) {
    let colors: &Colors = match shot.theme {
        ThemePreference::Dark => &DARK,
        _ => &LIGHT,
    };
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    // The bands are part of their panes: one surface from top to bottom.
    let mut checks = vec![
        ((width - 4., 4.), colors.editor, "editor's top band"),
        (
            (width - 12., height / 2.),
            colors.editor,
            "editor or preview",
        ),
        (
            (width - 4., height - BOTTOM_BAND_HEIGHT / 2.),
            colors.editor,
            "editor's bottom band",
        ),
    ];
    if shot.sidebar {
        checks.extend([
            ((4., height / 2.), colors.sidebar, "sidebar"),
            (
                (SIDEBAR_WIDTH - 6., 4.),
                colors.sidebar,
                "sidebar's top band",
            ),
            ((SIDEBAR_WIDTH + 6., 4.), colors.list, "note list"),
        ]);
    } else {
        // Leftmost, the list starts below the traffic lights' band.
        checks.push(((8., TOP_BAND_HEIGHT - 4.), colors.list, "note list"));
        checks.push((
            (NOTE_LIST_WIDTH, TOP_BAND_HEIGHT - 4.),
            colors.list,
            "note list",
        ));
    }
    for (at, expected, what) in checks {
        assert_surface(
            image,
            scale,
            at,
            expected,
            2,
            &format!("{}: {what}", shot.name),
        );
    }
}

#[test]
#[ignore = "writes PNGs; run on demand with KANSO_SNAPSHOT_DIR set"]
fn snapshots() {
    let out = PathBuf::from(std::env::var("KANSO_SNAPSHOT_DIR").expect("set KANSO_SNAPSHOT_DIR"));
    std::fs::create_dir_all(&out).unwrap();
    use ThemePreference::{Dark, Light};
    use ViewMode::{Edit, Preview, Split};
    let full = size(px(1440.), px(900.));
    let shot = |name, theme, view_mode, prepare: Prepare| Shot {
        name,
        theme,
        view_mode,
        size: full,
        prepare,
        sidebar: true,
    };
    let shots = [
        shot("dark-split", Dark, Split, open_latest),
        shot("light-split", Light, Split, open_latest),
        shot("dark-edit", Dark, Edit, open_latest),
        shot("light-edit", Light, Edit, open_latest),
        shot("dark-preview", Dark, Preview, open_latest),
        shot("light-preview", Light, Preview, open_latest),
        shot("dark-edit-mono", Dark, Edit, mono_font),
        shot("dark-empty", Dark, Split, empty_notebook),
        shot("dark-search", Dark, Edit, search_sync),
        // The Status group open and the footer's warnings.
        shot("dark-status-warnings", Dark, Edit, status_and_warnings),
        shot("light-status-warnings", Light, Split, status_and_warnings),
        shot("dark-recent-tags", Dark, Preview, recent_all_tags),
        Shot {
            sidebar: false,
            ..shot("dark-no-sidebar", Dark, Edit, hide_sidebar)
        },
        shot("dark-trash", Dark, Edit, open_trashed),
        shot("dark-conflict", Dark, Edit, conflict),
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-split-min", Dark, Split, open_latest)
        },
        // A table wider than the narrow preview scrolls instead of clipping.
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-split-table", Dark, Split, open_wide_table)
        },
        // Tall enough for the whole demo note.
        Shot {
            size: size(px(1440.), px(1700.)),
            ..shot("dark-preview-long", Dark, Preview, open_latest)
        },
        Shot {
            size: size(px(1440.), px(1700.)),
            ..shot("light-preview-long", Light, Preview, open_latest)
        },
    ];
    let real_db = std::env::var_os("KANSO_SNAPSHOT_DB");
    let only = std::env::var("KANSO_SNAPSHOT_ONLY").ok();
    for shot in shots.iter().filter(|s| {
        only.as_deref()
            .is_none_or(|o| o.split(',').any(|n| n == s.name))
    }) {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kanso.db");
        let (store, clock) = match &real_db {
            Some(source) => {
                std::fs::copy(source, &db).unwrap();
                (Store::open(&db).unwrap(), None)
            }
            None => (
                demo_store(&db),
                Some(Clock::fixed(DEMO_NOW_MS, DEMO_UTC_OFFSET_SECS)),
            ),
        };
        let store = Arc::new(store);
        let mut cx = context(clock);
        let (window, workspace) = open(&mut cx, store.clone(), dir.path().to_path_buf(), shot);
        cx.run_until_parked();
        (shot.prepare)(&mut cx, window, &workspace, &store);
        // Search waits for typing to pause.
        cx.advance_clock(std::time::Duration::from_secs(1));
        // Let background work (backlinks, image decoding) settle, then draw.
        // Each round moves the clock on too, so work a draw schedules after
        // a pause, such as Markdown highlighting that ran out of time, is
        // done before the picture is taken.
        for _ in 0..3 {
            cx.advance_clock(std::time::Duration::from_millis(500));
            cx.run_until_parked();
            cx.update_window(window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        }
        let image = cx.capture_screenshot(window).expect("Metal rendering");
        let path = out.join(format!("{}.png", shot.name));
        image.save(&path).unwrap();
        println!(
            "wrote {} ({}x{})",
            path.display(),
            image.width(),
            image.height()
        );
        check_surfaces(&image, shot);
    }
}
