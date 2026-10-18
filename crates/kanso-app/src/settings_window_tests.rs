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
use crate::services::{Services, SyncState};
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
