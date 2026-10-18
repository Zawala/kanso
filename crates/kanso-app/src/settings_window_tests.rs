//! Settings in a real window: opening and leaving them, the page actions,
//! and controls that reach the editor. Nothing here starts the API or
//! touches the Keychain or the login items.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Entity, FocusHandle, Modifiers, TestAppContext,
    VisualTestContext, point, px,
};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::{AppConfig, EditorFont};
use crate::library::Filter;
use crate::palette::Mode;
use crate::services::{Services, SyncState, TokenState};
use crate::settings::{OpenEditorSettings, OpenSyncSettings, SettingsPage, register_app_actions};
use crate::workspace::{AppState, OpenSettings, Workspace, for_each_workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    /// The open note.
    note: String,
    /// What had the keyboard while writing it.
    body: FocusHandle,
}

fn globals(cx: &mut App, store: Arc<Store>, data_dir: std::path::PathBuf) {
    gpui_kit::init(cx);
    cx.set_global(AppState {
        store,
        data_dir,
        config: AppConfig::default(),
    });
    cx.set_global(Services::default());
    crate::workspace::bind_keys(cx);
}

/// A notebook with enough notes to scroll, the last one open with a long
/// body and the keyboard in it.
fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Work"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let body: String = (0..300).map(|n| format!("Line {n}\n")).collect();
    let mut note = String::new();
    for n in 0..40 {
        note = store
            .put(
                DocType::Note,
                json!({"title": format!("Note {n}"), "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
    }
    let (app_store, data_dir) = (store.clone(), dir.path().to_path_buf());
    let (window, workspace) = cx.update(|cx| {
        globals(cx, app_store, data_dir);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    // Active, as the window in front is: focus changes report blurs.
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    let library = cx.read(|cx| workspace.read(cx).library.clone());
    let id = note.clone();
    library.update(cx, |library, cx| library.select(Some(id), cx));
    cx.run_until_parked();
    let body = window
        .update(cx, |_, window, cx| {
            let editor = workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.focus_body(window, cx));
            window.focused(cx).expect("the body takes the keyboard")
        })
        .unwrap();
    let f = Fixture {
        _dir: dir,
        window,
        workspace,
        note,
        body,
    };
    f.draw(cx);
    f
}

impl Fixture {
    /// Draws, so focus changes reach their listeners and actions find what
    /// is on screen.
    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn keys(&self, cx: &mut TestAppContext, keys: &str) {
        cx.simulate_keystrokes(self.window, keys);
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        self.draw(cx);
    }

    fn open(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| self.workspace.read(cx).settings_open())
    }

    fn page(&self, cx: &mut TestAppContext) -> SettingsPage {
        cx.read(|cx| self.workspace.read(cx).settings.read(cx).page())
    }

    fn focused(&self, cx: &mut TestAppContext, handle: &FocusHandle) -> bool {
        let handle = handle.clone();
        self.window
            .update(cx, |_, window, _| handle.is_focused(window))
            .unwrap()
    }

    /// Clicks the middle of what is drawn with `selector`.
    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
        self.draw(cx);
    }

    /// Sends `action` from the workspace, as the palette does.
    fn dispatch(&self, cx: &mut TestAppContext, action: Box<dyn gpui_kit::Action>) {
        let focus = cx.read(|cx| self.workspace.read(cx).focus.clone());
        self.window
            .update(cx, |_, window, cx| {
                focus.dispatch_action(action.as_ref(), window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        self.draw(cx);
    }

    fn gear_active(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| {
            self.workspace
                .read(cx)
                .sidebar
                .read(cx)
                .settings_open_for_test()
        })
    }
}

/// ⌘, puts Settings where the notes were, and Escape brings the notes back
/// exactly: the same note, the list and the text scrolled as they were, and
/// the keyboard in the text.
#[gpui_kit::test]
fn escape_returns_to_the_notes_as_they_were(cx: &mut TestAppContext) {
    let f = setup(cx);
    let (list, editor) = cx.read(|cx| {
        let workspace = f.workspace.read(cx);
        (workspace.note_list.clone(), workspace.editor.clone())
    });
    cx.update(|cx| {
        list.read(cx).set_scroll_offset(point(px(0.), px(-400.)));
        editor.update(cx, |editor, cx| {
            editor.scroll_body_for_test(point(px(0.), px(-900.)), cx)
        });
    });
    f.draw(cx);
    let scrolled = |cx: &mut TestAppContext| {
        cx.read(|cx| {
            (
                list.read(cx).scroll_offset(),
                editor.read(cx).body_scroll_for_test(cx),
            )
        })
    };
    let before = scrolled(cx);
    assert!(before.0.y < px(0.) && before.1.y < px(0.), "{before:?}");

    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    assert!(f.gear_active(cx));
    assert!(!f.focused(cx, &f.body));
    assert_eq!(f.page(cx), SettingsPage::General);
    // The page column takes the arrow keys.
    f.keys(cx, "down down");
    assert_eq!(f.page(cx), SettingsPage::Appearance);

    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(!f.gear_active(cx));
    assert!(f.focused(cx, &f.body), "the keyboard is back in the text");
    let open = cx.read(|cx| editor.read(cx).note_id().map(str::to_owned));
    assert_eq!(open.as_deref(), Some(f.note.as_str()));
    assert_eq!(scrolled(cx), before);

    // ⌘, again shows the page left last; Back to notes goes back too.
    f.keys(cx, "cmd-,");
    assert_eq!(f.page(cx), SettingsPage::Appearance);
    f.click(cx, "settings-back");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));
}

/// Each page has its command, and the palette runs it.
#[gpui_kit::test]
fn the_palette_opens_each_page(cx: &mut TestAppContext) {
    let f = setup(cx);
    for page in SettingsPage::ALL {
        let label = format!("Settings: {}", page.title());
        let command = crate::commands::all()
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("a command for {page:?}"));
        f.dispatch(cx, command.action().unwrap());
        assert!(f.open(cx));
        assert_eq!(f.page(cx), page);
    }
    f.keys(cx, "escape");
    assert!(!f.open(cx));

    // Typed into the palette, from the notes.
    f.keys(cx, "cmd-k");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| {
                palette.set_query("settings: sync", window, cx)
            })
        })
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(200));
    cx.run_until_parked();
    f.keys(cx, "enter");
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), None);
    assert!(f.open(cx));
    assert_eq!(f.page(cx), SettingsPage::Sync);
    // Escape closes Settings, and the keyboard is back where the palette
    // found it.
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));
}

/// Asking for notes leaves Settings: a place in the sidebar (even the one
/// already showing), ⌘N, or a note from ⌘P.
#[gpui_kit::test]
fn asking_for_notes_leaves_settings(cx: &mut TestAppContext) {
    let f = setup(cx);
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    let filter = cx.read(|cx| library.read(cx).filter.clone());

    f.keys(cx, "cmd-,");
    library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
    cx.run_until_parked();
    assert!(!f.open(cx), "the place already showing");

    f.keys(cx, "cmd-,");
    library.update(cx, |library, cx| library.set_filter(Filter::Pinned, cx));
    cx.run_until_parked();
    assert!(!f.open(cx), "another place");
    f.draw(cx);

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-n");
    assert!(!f.open(cx), "a new note");
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    let in_title = f
        .window
        .update(cx, |_, window, cx| {
            editor.read(cx).title_focused(window, cx)
        })
        .unwrap();
    assert!(in_title, "typing goes into the new note's title");

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-p");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    assert_eq!(cx.read(|cx| palette.read(cx).mode()), Some(Mode::Notes));
    f.keys(cx, "enter");
    assert!(!f.open(cx), "a note from ⌘P");
}

/// The Editor page's controls change the open editor at once and are kept
/// in the settings file.
#[gpui_kit::test]
fn editor_controls_reach_the_editor(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    f.click(cx, "editor-font-2");
    f.click(cx, "font-larger");
    f.click(cx, "font-larger");
    f.click(cx, "line-numbers");
    let (font, size, numbers) = cx.read(|cx| {
        let editor = editor.read(cx);
        (
            editor.editor_font,
            editor.font_size,
            editor.allow_line_numbers,
        )
    });
    assert_eq!((font, size, numbers), (EditorFont::Mono, 18., false));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert_eq!(saved.appearance.editor_font, EditorFont::Mono);
    assert_eq!(saved.appearance.editor_font_size, 18.);
    assert!(!saved.appearance.line_numbers);
    // A click on a page in the column shows it.
    f.click(cx, "settings-sync");
    assert_eq!(f.page(cx), SettingsPage::Sync);
}

/// Without a Worker there is nothing to sync with: Sync now waits for one.
#[gpui_kit::test]
fn sync_now_needs_a_worker(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenSyncSettings));
    f.click(cx, "sync-now");
    let sync = cx.read(|cx| cx.global::<Services>().sync.clone());
    assert_eq!(sync, SyncState::Off);
}

fn open_test_window(cx: &mut App) {
    gpui_kit::open_window(Default::default(), cx, |window, cx| {
        cx.new(|cx| Workspace::new(window, cx))
    })
    .unwrap();
}

/// Settings from the menu bar with no window open: the main window opens,
/// showing Settings; with it open, it is used rather than another opened.
#[gpui_kit::test]
fn settings_open_a_window_when_none_is_open(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    cx.update(|cx| {
        globals(cx, store, data_dir);
        register_app_actions(cx, open_test_window);
    });
    assert!(cx.read(|cx| cx.windows().is_empty()));
    cx.update(|cx| cx.dispatch_action(&OpenSyncSettings));
    cx.run_until_parked();
    let shown = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            let mut shown = Vec::new();
            for_each_workspace(cx, |workspace, cx| {
                shown.push(
                    workspace
                        .settings_open()
                        .then(|| workspace.settings.read(cx).page()),
                );
            });
            shown
        })
    };
    assert_eq!(shown(cx), [Some(SettingsPage::Sync)]);
    cx.update(|cx| cx.dispatch_action(&OpenSettings));
    cx.run_until_parked();
    assert_eq!(shown(cx), [Some(SettingsPage::Sync)]);
}

/// The same with notes, the last one kept from before: the window opens on
/// Settings and stays there, with the last note open behind them. The app
/// picking that note itself doesn't take Settings away.
#[gpui_kit::test]
fn settings_stay_when_the_window_opens_on_a_library(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let mut last = String::new();
    for n in 0..3 {
        last = store
            .put(
                DocType::Note,
                json!({"title": format!("Note {n}"), "body": "x", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
    }
    let data_dir = dir.path().to_path_buf();
    let kept = last.clone();
    cx.update(|cx| {
        globals(cx, store, data_dir);
        cx.global_mut::<AppState>().config.last_note = Some(kept);
        register_app_actions(cx, open_test_window);
    });
    cx.update(|cx| cx.dispatch_action(&OpenSyncSettings));
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    let shown = cx.update(|cx| {
        let mut shown = Vec::new();
        for_each_workspace(cx, |workspace, cx| {
            shown.push((
                workspace
                    .settings_open()
                    .then(|| workspace.settings.read(cx).page()),
                workspace.editor.read(cx).note_id().map(str::to_owned),
            ));
        });
        shown
    });
    assert_eq!(shown, [(Some(SettingsPage::Sync), Some(last))]);
}

/// What acts on the open note brings it back from behind Settings first,
/// so the note it changes is in view; the palette over Settings offers no
/// note actions at all.
#[gpui_kit::test]
fn note_actions_leave_settings_first(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    f.dispatch(cx, Box::new(crate::workspace::TrashNote));
    assert!(!f.open(cx), "Settings closed first");
    let store = cx.read(|cx| cx.global::<AppState>().store.clone());
    let note = store.note(&f.note).unwrap().unwrap();
    assert!(note.is_trashed());
    let open = cx.read(|cx| {
        let editor = f.workspace.read(cx).editor.clone();
        editor.read(cx).note_id().map(str::to_owned)
    });
    assert_eq!(open.as_deref(), Some(f.note.as_str()), "in view");

    for action in [
        Box::new(crate::workspace::TogglePinned) as Box<dyn gpui_kit::Action>,
        Box::new(crate::workspace::ShowPreview),
    ] {
        f.keys(cx, "cmd-,");
        f.dispatch(cx, action);
        assert!(!f.open(cx));
    }

    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-k");
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    let labels = cx.read(|cx| palette.read(cx).item_labels());
    assert!(labels.iter().any(|l| l == "Settings: Sync"), "{labels:?}");
    let note_commands: Vec<_> = crate::commands::all()
        .iter()
        .filter(|c| c.needs != crate::commands::Needs::Nothing)
        .map(|c| c.label)
        .collect();
    assert!(!note_commands.is_empty());
    for label in note_commands {
        assert!(!labels.iter().any(|l| l == label), "{label} over Settings");
    }
    f.keys(cx, "escape");
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    // Back with the notes, the palette offers them again.
    f.keys(cx, "cmd-k");
    let labels = cx.read(|cx| palette.read(cx).item_labels());
    assert!(labels.iter().any(|l| l == "Show history…"), "{labels:?}");
}

/// The token can always be set: while the Keychain lookup is pending or
/// never answered, and when the Keychain wouldn't say.
#[gpui_kit::test]
fn a_token_can_always_be_set(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenSyncSettings));
    assert_eq!(
        cx.read(|cx| cx.global::<Services>().sync_token.clone()),
        TokenState::Unknown,
        "tests never read the Keychain"
    );
    assert!(f.drawn(cx, "set-token"), "unknown");
    cx.update(|cx| cx.global_mut::<Services>().sync_token = TokenState::Unreadable);
    assert!(f.drawn(cx, "set-token"), "unreadable");
    cx.update(|cx| cx.global_mut::<Services>().sync_token = TokenState::Missing);
    assert!(f.drawn(cx, "set-token"), "missing");
}

/// A conflict names its note on the Sync page, and Open note goes to it.
#[gpui_kit::test]
fn a_conflicted_note_opens_from_the_sync_page(cx: &mut TestAppContext) {
    let f = setup(cx);
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    let other = cx.read(|cx| {
        library
            .read(cx)
            .notes
            .iter()
            .map(|n| n.id.clone())
            .find(|id| *id != f.note)
            .unwrap()
    });
    cx.update(|cx| {
        cx.global_mut::<Services>().sync = SyncState::Synced {
            at_ms: 0,
            conflicts: 1,
            conflicted: vec!["book:gone".into(), other.clone()],
        };
    });
    f.dispatch(cx, Box::new(OpenSyncSettings));
    f.click(cx, "open-conflicted-note");
    assert!(!f.open(cx));
    let open = cx.read(|cx| {
        let editor = f.workspace.read(cx).editor.clone();
        editor.read(cx).note_id().map(str::to_owned)
    });
    assert_eq!(open, Some(other));
}

/// The readable width can be turned off, for the editor at once and in the
/// settings file.
#[gpui_kit::test]
fn the_readable_width_can_be_turned_off(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.dispatch(cx, Box::new(OpenEditorSettings));
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    assert!(cx.read(|cx| editor.read(cx).readable_width));
    f.click(cx, "readable-width");
    assert!(!cx.read(|cx| editor.read(cx).readable_width));
    let saved = cx.read(|cx| {
        let state = cx.global::<AppState>();
        AppConfig::load(&crate::paths::config_path(&state.data_dir)).unwrap()
    });
    assert!(!saved.appearance.readable_width);
}

impl Fixture {
    fn focus_mode(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| self.workspace.read(cx).focus_mode)
    }

    fn drawn(&self, cx: &mut TestAppContext, selector: &'static str) -> bool {
        self.draw(cx);
        VisualTestContext::from_window(self.window, cx)
            .debug_bounds(selector)
            .is_some()
    }

    fn saved_focus_mode(&self, cx: &mut TestAppContext) -> bool {
        cx.read(|cx| {
            let state = cx.global::<AppState>();
            AppConfig::load(&crate::paths::config_path(&state.data_dir))
                .unwrap()
                .layout
                .focus_mode
        })
    }
}

/// Focus mode and Settings never show together: ⌘, from focus mode shows
/// Settings beside the sidebar, and going back brings focus mode back with
/// the keyboard in the text. ⌘⇧D from Settings goes to the note, alone.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn settings_and_focus_mode_take_turns(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.keys(cx, "cmd-shift-d");
    assert!(f.focus_mode(cx));
    assert!(!f.drawn(cx, "sidebar-pane"));

    f.keys(cx, "cmd-,");
    assert!(f.open(cx));
    assert!(!f.focus_mode(cx), "Settings leave focus mode");
    assert!(!f.saved_focus_mode(cx));
    assert!(f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "settings-pane"));
    assert!(!f.drawn(cx, "editor-pane"));
    assert!(!f.drawn(cx, "list-pane"));

    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focus_mode(cx), "going back brings focus mode back");
    assert!(f.saved_focus_mode(cx));
    assert!(f.focused(cx, &f.body));
    assert!(!f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "editor-pane"));

    // ⌘⇧D in Settings: back to the note, in focus mode.
    f.keys(cx, "cmd-shift-d");
    assert!(!f.focus_mode(cx));
    f.keys(cx, "cmd-, cmd-shift-d");
    assert!(!f.open(cx));
    assert!(f.focus_mode(cx));
    assert!(f.focused(cx, &f.body));

    // Leaving Settings for a place in the sidebar shows the list, not focus
    // mode.
    f.keys(cx, "cmd-,");
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| library.set_filter(Filter::Pinned, cx));
    cx.run_until_parked();
    assert!(!f.open(cx));
    assert!(!f.focus_mode(cx));
    assert!(f.drawn(cx, "list-pane"));
}

/// ⌘\ folds the sidebar to its rail beside Settings too, and the rail's gear
/// is the place shown. Escape goes first to the palette over Settings, then
/// to Settings, wherever the keyboard is in the window.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn the_rail_palette_and_escape_in_settings(cx: &mut TestAppContext) {
    let f = setup(cx);
    let sidebar = cx.read(|cx| f.workspace.read(cx).sidebar.clone());
    f.keys(cx, "cmd-,");
    f.keys(cx, "cmd-\\");
    assert!(f.open(cx), "Settings stay");
    assert!(cx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert!(f.gear_active(cx));
    assert!(f.drawn(cx, "sidebar-pane"));
    assert!(f.drawn(cx, "settings-pane"));
    f.draw(cx);
    let (rail, page) = {
        let mut vcx = VisualTestContext::from_window(f.window, cx);
        (
            vcx.debug_bounds("sidebar-pane").unwrap(),
            vcx.debug_bounds("settings-pane").unwrap(),
        )
    };
    assert_eq!(f32::from(rail.size.width), crate::layout::rail_width(false));
    assert_eq!(
        page.left(),
        rail.right(),
        "Settings start where the rail ends"
    );

    // The palette takes the first Escape, Settings the next.
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.keys(cx, "cmd-k");
    assert!(cx.read(|cx| palette.read(cx).is_open()));
    f.keys(cx, "escape");
    assert!(!cx.read(|cx| palette.read(cx).is_open()));
    assert!(f.open(cx));
    f.keys(cx, "escape");
    assert!(!f.open(cx));
    assert!(f.focused(cx, &f.body));

    // With the keyboard on the window itself rather than in Settings.
    f.keys(cx, "cmd-,");
    let focus = cx.read(|cx| f.workspace.read(cx).focus.clone());
    f.window
        .update(cx, |_, window, cx| window.focus(&focus, cx))
        .unwrap();
    f.keys(cx, "escape");
    assert!(!f.open(cx));

    f.keys(cx, "cmd-\\");
    assert!(!cx.read(|cx| sidebar.read(cx).is_collapsed()));
}

/// In focus mode the list is out of sight, so asking for its search box
/// brings it back rather than typing into nothing.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn search_leaves_focus_mode(cx: &mut TestAppContext) {
    let f = setup(cx);
    let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
    let search = cx.read(|cx| gpui_kit::Focusable::focus_handle(list.read(cx).search.read(cx), cx));
    for keys in ["cmd-l", "cmd-shift-f"] {
        f.keys(cx, "cmd-shift-d");
        assert!(f.focus_mode(cx));
        f.keys(cx, keys);
        assert!(!f.focus_mode(cx), "{keys}");
        assert!(f.drawn(cx, "list-pane"), "{keys}");
        assert!(f.focused(cx, &search), "{keys}");
    }
}
