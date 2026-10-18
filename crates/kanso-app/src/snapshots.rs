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
use crate::commands::Picker;
use crate::config::{AppConfig, EditorFont, ThemePreference, ViewMode};
use crate::editor::EditorEvent;
use crate::layout::{BOTTOM_BAND_HEIGHT, NOTE_LIST_WIDTH, SIDEBAR_WIDTH, rail_width};
use crate::library::{Filter, has_notes_inside};
use crate::palette::{Mode, PANEL_HEIGHT, PANEL_WIDTH};
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
        // The app's keymap, so shortcuts show as bound.
        crate::workspace::bind_keys(cx);
        crate::note_list::bind_keys(cx);
        if let Some(clock) = clock {
            cx.set_global(clock);
        }
    });
    cx
}

/// Puts the workspace in the shot's state before drawing.
type Prepare = fn(&mut HeadlessAppContext, AnyWindowHandle, &Entity<Workspace>, &Store);

/// What is left of the editor, for the surface checks.
#[derive(Clone, Copy, PartialEq)]
enum Left {
    /// The sidebar and the note list.
    Sidebar,
    /// The icon rail and the note list.
    Rail,
    /// Nothing: focus mode.
    Nothing,
}

struct Shot {
    name: &'static str,
    theme: ThemePreference,
    view_mode: ViewMode,
    /// The window's size in points; the PNG is twice that.
    size: Size<Pixels>,
    prepare: Prepare,
    left: Left,
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

/// The sidebar collapsed to its rail, the traffic lights in its top band.
fn collapse_sidebar(
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

/// The rail's notebooks button: the palette lists the notebooks to go to.
fn rail_notebooks(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    collapse_sidebar(cx, window, workspace, store);
    cx.update_window(window, |_, _, cx| {
        let sidebar = workspace.read(cx).sidebar.clone();
        sidebar.update(cx, |_, cx| {
            cx.emit(crate::sidebar::SidebarEvent::GoToNotebook)
        });
    })
    .unwrap();
}

/// Focus mode, as after ⌘⇧D: the note alone, its header out of sight.
fn focus_mode(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| {
            workspace.set_focus_mode(true, window, cx)
        });
    })
    .unwrap();
}

/// Focus mode with the pointer near the top: the header shows.
fn focus_mode_hover(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    focus_mode(cx, window, workspace, store);
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.reveal_chrome = true;
            cx.notify();
        });
    });
}

/// The divider left at 38% of the editor, as a drag in an earlier session
/// keeps it.
fn split_dragged(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.keep_split_ratio(0.38, cx));
    });
}

/// The source scrolled about halfway: the preview beside it follows.
fn split_scrolled(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    // The editor knows how tall its lines are once it has been drawn.
    cx.update_window(window, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    cx.update_window(window, |_, _, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| editor.scroll_source_for_test(0.5, cx));
    })
    .unwrap();
}

/// A library with no notes at all, as on first run.
fn first_run(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    for filter in [Filter::All, Filter::Trash] {
        let params = crate::library::search_params(&filter, "", Default::default(), true);
        for note in store.search_summaries(&params).unwrap() {
            store.delete(&note.id, Origin::Ui).unwrap();
        }
    }
    cx.run_until_parked();
}

/// All notes with none open: what the editor says, with the real shortcut.
fn no_note_open(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| library.update(cx, |library, cx| library.select(None, cx)));
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

/// An empty notebook, "Research" when there is one: no notes of its own
/// and none in notebooks inside it. Else the first, preferring a name no
/// other notebook has, so the shot can't be read as its namesake.
fn empty_notebook(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books = store.books().unwrap();
    let is_empty = |b: &&kanso_core::Book| b.count == 0 && !has_notes_inside(&b.id, &books);
    let unique = |b: &&kanso_core::Book| books.iter().filter(|o| o.name == b.name).count() == 1;
    let empty = books
        .iter()
        .filter(is_empty)
        .find(|b| b.name == "Research")
        .or_else(|| books.iter().filter(is_empty).find(unique))
        .or_else(|| books.iter().find(is_empty));
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

/// A notebook that holds only notebooks, its notes a level down. The demo
/// has none, so it gets one: "Clients", with "Acme" and a note inside.
fn notebook_of_notebooks(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books = store.books().unwrap();
    let found = books
        .iter()
        .find(|b| b.count == 0 && has_notes_inside(&b.id, &books))
        .map(|b| b.id.clone());
    let id = found.unwrap_or_else(|| {
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
        let clients = book("Clients", None);
        let acme = book("Acme", Some(&clients));
        let now = cx.update(|cx| Clock::now_ms(cx));
        store
            .put(
                DocType::Note,
                json!({"title": "Kickoff notes", "body": "Scope, people and dates.",
                    "bookId": acme, "createdAt": now - DAY, "updatedAt": now - DAY}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        clients
    });
    cx.run_until_parked();
    let library = cx.update(|cx| workspace.read(cx).library.clone());
    cx.update(|cx| {
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(id), cx);
            library.select(None, cx);
        })
    });
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

/// Searches for something no note has: the zero-results state.
fn search_nothing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let search = workspace.read(cx).note_list.read(cx).search.clone();
        search.update(cx, |search, cx| {
            search.replace_all("quarterly okr", window, cx)
        });
    })
    .unwrap();
}

/// The empty search box with focus, which shows the search syntax.
fn focus_search(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update_window(window, |_, window, cx| {
        let list = workspace.read(cx).note_list.clone();
        list.update(cx, |list, cx| list.focus_search(window, cx));
    })
    .unwrap();
}

/// The pointer over the third note, revealing its actions.
fn hover_third(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.run_until_parked();
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        let third = library.read(cx).notes.get(2).map(|n| n.id.clone());
        let list = workspace.read(cx).note_list.clone();
        list.update(cx, |list, cx| {
            list.hovered = third;
            cx.notify();
        });
    });
}

/// Sorted by title: no date headers.
fn sort_by_title(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        library.update(cx, |library, cx| {
            library.set_sort(kanso_store::Sort::Title, false, cx)
        });
    });
}

/// Narrowed to completed notes from the list's filter menu.
fn only_completed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let library = workspace.read(cx).library.clone();
        library.update(cx, |library, cx| {
            library.set_status_only(Some(kanso_core::NoteStatus::Completed), cx)
        });
    });
}

/// Notes whose titles and excerpts run long, edited just now.
fn long_titles(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    _: &Entity<Workspace>,
    store: &Store,
) {
    let book = store.books().unwrap()[0].id.clone();
    let now = cx.update(|cx| Clock::now_ms(cx));
    let notes = [
        (
            "A very long note title that keeps going well past the width of the note list column",
            "An excerpt long enough to need two lines in the list and then some more words so that it has to stop with an ellipsis at the end of the second line.",
            "active",
            true,
        ),
        (
            "Überlange Überschrift mit Umlauten, die ebenfalls abgeschnitten werden muss",
            "Kurz.",
            "onHold",
            false,
        ),
        (
            "Supercalifragilisticexpialidocious_without_any_spaces_at_all_in_the_title",
            "Averyveryverylongwordwithoutspacesthatmustnotpushtherowwiderthanthecolumn",
            "none",
            false,
        ),
    ];
    for (ix, (title, body, status, pinned)) in notes.into_iter().enumerate() {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book, "status": status,
                    "pinned": pinned, "createdAt": now - DAY, "updatedAt": now - ix as i64 * MINUTE}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
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

/// Opens the palette in `mode` and types `query`, as from the keyboard.
fn open_palette(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    mode: Mode,
    query: &str,
) {
    cx.update_window(window, |_, window, cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| {
            palette.open(mode, window, cx);
            if !query.is_empty() {
                palette.set_query(query, window, cx);
            }
        });
    })
    .unwrap();
}

/// ⌘K with nothing typed.
fn palette(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Commands, "");
}

/// ⌘K with "sync" typed: commands and notes together, with the first note
/// selected so the detail pane shows a note.
fn palette_query(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Commands, "sync");
    // The notes arrive after a pause in typing.
    cx.advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| palette.select_first_note(cx));
    });
}

/// What a click on the editor header's notebook path or "Add tag" does:
/// the palette's list opens, and `query` is typed into it.
fn pick_from_header(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    picker: Picker,
    query: &str,
) {
    cx.update_window(window, |_, _, cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |_, cx| cx.emit(EditorEvent::Pick(picker)));
    })
    .unwrap();
    // The workspace opens the palette once the event is delivered, after
    // the update above.
    if !query.is_empty() {
        cx.update_window(window, |_, window, cx| {
            let palette = workspace.read(cx).palette.clone();
            palette.update(cx, |palette, cx| palette.set_query(query, window, cx));
        })
        .unwrap();
    }
}

/// Move to notebook, from the header's notebook path.
fn palette_move(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Notebook, "");
}

/// The tag list, from the header's "Add tag": the note's tags checked.
fn palette_tag(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Tag, "");
}

/// A name no tag has, typed into the tag list: it offers to make it.
fn palette_new_tag(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    pick_from_header(cx, window, workspace, Picker::Tag, "ru");
}

/// Open Note (⌘P): the most recently edited notes.
fn note_picker(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    open_palette(cx, window, workspace, Mode::Notes, "");
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

/// The palette's panel is painted in the elevated color, centred in the
/// window. The point sits in its padding, left of the search field.
fn check_palette(image: &image::RgbaImage, shot: &Shot) {
    let colors: &Colors = match shot.theme {
        ThemePreference::Dark => &DARK,
        _ => &LIGHT,
    };
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    let corner = (
        (width - PANEL_WIDTH) / 2.,
        ((height - PANEL_HEIGHT) / 2.).floor(),
    );
    assert_surface(
        image,
        scale,
        (corner.0 + 6., corner.1 + 6.),
        colors.elevated,
        2,
        &format!("{}: palette", shot.name),
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
    let band_top = height - BOTTOM_BAND_HEIGHT;
    // The panes keep their widths whatever the window's size; the sidebar
    // and its rail are the same surface.
    let leading = match shot.left {
        Left::Sidebar => Some(SIDEBAR_WIDTH),
        Left::Rail => Some(rail_width(false)),
        Left::Nothing => None,
    };
    match leading {
        Some(leading) => {
            checks.extend([
                ((4., height / 2.), colors.sidebar, "sidebar"),
                ((leading - 6., 4.), colors.sidebar, "sidebar's top band"),
                ((leading + 6., 4.), colors.list, "note list"),
                (
                    (leading + NOTE_LIST_WIDTH - 6., 4.),
                    colors.list,
                    "note list's right edge",
                ),
                (
                    (leading + NOTE_LIST_WIDTH + 8., 4.),
                    colors.editor,
                    "editor's left edge",
                ),
                // The bottom band's top edge runs across every pane, with a
                // note open or not.
                ((4., band_top), colors.divider, "sidebar's bottom divider"),
                (
                    (leading + 6., band_top),
                    colors.divider,
                    "note list's bottom divider",
                ),
                (
                    (width - 4., band_top),
                    colors.divider,
                    "editor's bottom divider",
                ),
            ]);
        }
        // Focus mode: the editor's surface edge to edge, and a quiet footer.
        None => checks.extend([
            ((4., 4.), colors.editor, "focus mode's top band"),
            ((4., height / 2.), colors.editor, "focus mode's left margin"),
            (
                (4., band_top),
                colors.editor,
                "focus mode's footer, without a divider",
            ),
        ]),
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
        left: Left::Sidebar,
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
        shot("light-empty", Light, Split, empty_notebook),
        shot(
            "dark-notebook-of-notebooks",
            Dark,
            Edit,
            notebook_of_notebooks,
        ),
        shot("dark-search", Dark, Edit, search_sync),
        // The Status group open and the footer's warnings.
        shot("dark-status-warnings", Dark, Edit, status_and_warnings),
        shot("light-status-warnings", Light, Split, status_and_warnings),
        shot("dark-recent-tags", Dark, Preview, recent_all_tags),
        // The sidebar collapsed to its rail.
        Shot {
            left: Left::Rail,
            ..shot("dark-rail", Dark, Edit, collapse_sidebar)
        },
        Shot {
            left: Left::Rail,
            ..shot("light-rail", Light, Split, collapse_sidebar)
        },
        Shot {
            left: Left::Rail,
            ..shot("dark-rail-notebooks", Dark, Edit, rail_notebooks)
        },
        // Focus mode, at rest and with the pointer near the top.
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus", Dark, Edit, focus_mode)
        },
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-hover", Dark, Edit, focus_mode_hover)
        },
        Shot {
            left: Left::Nothing,
            ..shot("light-focus", Light, Edit, focus_mode)
        },
        Shot {
            left: Left::Nothing,
            ..shot("dark-focus-split", Dark, Split, focus_mode_hover)
        },
        // The divider where a drag left it, and the preview following the
        // source's scrolling.
        shot("dark-split-dragged", Dark, Split, split_dragged),
        shot("light-split-dragged", Light, Split, split_dragged),
        shot("dark-split-scrolled", Dark, Split, split_scrolled),
        // An empty library, and a library with no note open.
        shot("dark-first-run", Dark, Split, first_run),
        shot("light-first-run", Light, Split, first_run),
        shot("dark-no-note", Dark, Edit, no_note_open),
        shot("dark-trash", Dark, Edit, open_trashed),
        shot("dark-conflict", Dark, Edit, conflict),
        shot("light-search", Light, Edit, search_sync),
        shot("dark-list-no-results", Dark, Edit, search_nothing),
        shot("dark-list-search-focus", Dark, Edit, focus_search),
        shot("dark-list-hover", Dark, Edit, hover_third),
        shot("dark-list-title-sort", Dark, Edit, sort_by_title),
        shot("dark-list-filtered", Dark, Edit, only_completed),
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("dark-list-long-titles", Dark, Edit, long_titles)
        },
        Shot {
            size: size(px(1100.), px(720.)),
            ..shot("light-list-long-titles", Light, Edit, long_titles)
        },
        // Every date group of the demo at once.
        Shot {
            size: size(px(1440.), px(1100.)),
            ..shot("dark-list-grouped", Dark, Edit, open_latest)
        },
        // A wide window gives the room to the editor, not the panes.
        Shot {
            size: size(px(1800.), px(1000.)),
            ..shot("dark-wide", Dark, Split, open_latest)
        },
        shot("dark-palette", Dark, Edit, palette),
        shot("dark-palette-query", Dark, Edit, palette_query),
        shot("light-palette-query", Light, Edit, palette_query),
        shot("dark-palette-move", Dark, Edit, palette_move),
        shot("dark-palette-tag", Dark, Edit, palette_tag),
        shot("light-palette-new-tag", Light, Edit, palette_new_tag),
        shot("dark-open-note", Dark, Split, note_picker),
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
        if real_db.is_some() {
            // What a reload costs the list on the main thread: grouping the
            // notes by local day and sizing the rows.
            let (rows, took) = cx.update(|cx| {
                let list = workspace.read(cx).note_list.clone();
                list.update(cx, |list, cx| list.time_layout(cx))
            });
            println!("{}: laid out {rows} rows in {took:?}", shot.name);
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
        let palette_open = cx.update(|cx| workspace.read(cx).palette.read(cx).is_open());
        if palette_open {
            check_palette(&image, shot);
        } else {
            check_surfaces(&image, shot);
        }
    }
}
