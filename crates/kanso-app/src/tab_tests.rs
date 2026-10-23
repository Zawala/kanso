//! Tabs in a headless window: ⌘-click and the tab bar, ⌘W, Ctrl+Tab, each
//! tab's own back and forward, a trashed note's tab going, and the tabs kept
//! in the settings coming back. The model's rules are properties in
//! `tabs.rs`; these check the window wires them up.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Entity, Focusable as _, Modifiers, MouseButton, Pixels,
    Point, TestAppContext, VisualTestContext,
};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::AppConfig;
use crate::editor::NoteEditor;
use crate::library::Filter;
use crate::note_menu::NoteCommand;
use crate::tab_bar::TabBarEvent;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

#[cfg(target_os = "macos")]
const CLOSE_TAB: &str = "cmd-w";
#[cfg(not(target_os = "macos"))]
const CLOSE_TAB: &str = "ctrl-w";

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    book: String,
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    setup_with(cx, AppConfig::default(), |_| {})
}

/// A workspace over a fresh store, with `before` run on the store first so
/// the settings can name notes it holds.
fn setup_with(
    cx: &mut TestAppContext,
    mut config: AppConfig,
    before: impl FnOnce(&Store),
) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    before(&store);
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    config.last_note = None;
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        crate::workspace::bind_keys(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        book,
    }
}

impl Fixture {
    fn book(&self, name: &str, parent: Option<&str>) -> String {
        self.store
            .put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn note(&self, title: &str, body: &str) -> String {
        self.store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": self.book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn draw(&self, cx: &mut TestAppContext) {
        cx.run_until_parked();
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.executor().advance_clock(Duration::from_millis(700));
        self.draw(cx);
    }

    fn editor(&self, cx: &mut TestAppContext) -> Entity<NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    fn with_editor<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut NoteEditor, &mut gpui_kit::Window, &mut gpui_kit::Context<NoteEditor>) -> R,
    ) -> R {
        let editor = self.editor(cx);
        let out = self
            .window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| f(editor, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        out
    }

    fn open_note(&self, cx: &mut TestAppContext) -> Option<String> {
        let editor = self.editor(cx);
        cx.read(|cx| editor.read(cx).note_id().map(str::to_owned))
    }

    fn note_in(&self, book: &str, title: &str, body: &str) -> String {
        self.store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn type_body(&self, cx: &mut TestAppContext, body: &str) {
        self.with_editor(cx, |editor, window, cx| {
            editor.edit_for_test(body, window, cx)
        });
    }

    fn store_body(&self, id: &str) -> String {
        self.store.note(id).unwrap().unwrap().body
    }

    fn tabs(&self, cx: &mut TestAppContext) -> Vec<String> {
        cx.read(|cx| self.workspace.read(cx).tabs.ids())
    }

    fn active_tab(&self, cx: &mut TestAppContext) -> Option<String> {
        cx.read(|cx| self.workspace.read(cx).tabs.active_id().map(str::to_owned))
    }

    fn keys(&self, cx: &mut TestAppContext, keys: &str) {
        cx.simulate_keystrokes(self.window, keys);
        self.draw(cx);
    }

    fn middle_of(&self, cx: &mut TestAppContext, selector: &str) -> Point<Pixels> {
        self.draw(cx);
        let selector: &'static str = Box::leak(selector.to_owned().into_boxed_str());
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        vcx.debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"))
            .center()
    }

    fn secondary_click(&self, cx: &mut TestAppContext, id: &str) {
        let at = self.middle_of(cx, &format!("note-{id}"));
        VisualTestContext::from_window(self.window, cx)
            .simulate_click(at, Modifiers::secondary_key());
        self.draw(cx);
    }

    fn middle_click(&self, cx: &mut TestAppContext, id: &str) {
        let at = self.middle_of(cx, &format!("note-{id}"));
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        vcx.simulate_mouse_down(at, MouseButton::Middle, Modifiers::none());
        vcx.simulate_mouse_up(at, MouseButton::Middle, Modifiers::none());
        self.draw(cx);
    }

    fn selected(&self, cx: &mut TestAppContext) -> Vec<String> {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        cx.read(|cx| library.read(cx).selection.ids().to_vec())
    }

    fn run(&self, cx: &mut TestAppContext, id: &str, command: NoteCommand) {
        let workspace = self.workspace.clone();
        let id = id.to_owned();
        self.window
            .update(cx, |_, window, cx| {
                workspace.update(cx, |workspace, cx| {
                    workspace.run_note_command(&id, command, window, cx)
                })
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn dialog_open(&self, cx: &mut TestAppContext) -> bool {
        self.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    fn bar_is_showing(&self, cx: &mut TestAppContext) -> bool {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window, cx);
        vcx.debug_bounds("tab-bar").is_some()
    }
}

/// A middle click opens the note after the showing tab and the bar appears;
/// ⌘W closes the showing tab, then the other one.
// no-prop-test: drives one window through clicks and keys; the model's
// rules are properties in tabs.rs.
#[gpui_kit::test]
fn middle_click_opens_a_tab_and_cmd_w_closes_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    assert_eq!(f.tabs(cx), vec![a.clone()]);
    assert!(!f.bar_is_showing(cx), "one note has no bar");

    f.middle_click(cx, &b);
    assert_eq!(f.tabs(cx), [a.clone(), b.clone()]);
    assert_eq!(f.open_note(cx), Some(b.clone()));
    assert!(f.bar_is_showing(cx), "two notes show the bar");

    f.keys(cx, CLOSE_TAB);
    assert_eq!(f.tabs(cx), vec![a.clone()], "⌘W closes the showing tab");
    assert_eq!(f.open_note(cx), Some(a.clone()));
    assert!(!f.bar_is_showing(cx));
}

/// ⌘-click in the list still adds and takes away rows, as on master: only a
/// middle click, the menu or ⌘↩ open a tab.
// no-prop-test: one window, two clicks; the selection's rules are
// properties in selection.rs.
#[gpui_kit::test]
fn cmd_click_selects_several_notes_and_opens_no_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.secondary_click(cx, &b);
    assert_eq!(f.selected(cx), [a.clone(), b.clone()]);
    assert_eq!(f.tabs(cx), vec![b.clone()], "no second tab: the note shows");
    assert_eq!(f.open_note(cx), Some(b.clone()));

    f.secondary_click(cx, &b);
    assert_eq!(f.selected(cx), [a.clone()], "⌘-click takes it away");
    assert_eq!(f.open_note(cx), Some(a.clone()));
}

/// ⌘W with one note open saves its last typing before the window goes:
/// closing through `remove_window` skips the window's own save hook.
// no-prop-test: one window and one key; the bug was a missing save.
#[gpui_kit::test]
fn cmd_w_at_one_tab_saves_the_note_before_closing(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "saved");
    f.select(cx, &a);
    f.type_body(cx, "typed and not yet autosaved");
    assert_eq!(f.store_body(&a), "saved", "the autosave waits");

    // No clock advance: the typing is still unsaved when ⌘W arrives.
    cx.simulate_keystrokes(f.window, CLOSE_TAB);
    cx.run_until_parked();
    assert_eq!(f.store_body(&a), "typed and not yet autosaved");
}

/// Clicking a notebook while two notes are open shows its first note in a
/// new tab, not in the tab someone is reading.
// no-prop-test: one automatic list pick in one window.
#[gpui_kit::test]
fn a_notebook_pick_does_not_replace_an_open_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let c = f.note("C", "text");
    f.select(cx, &a);
    f.middle_click(cx, &c);
    assert_eq!(f.tabs(cx), [a.clone(), c.clone()]);

    let book = f.book("Other", None);
    let b = f.note_in(&book, "B", "text");
    let library = cx.read(|cx| f.workspace.read(cx).library.clone());
    library.update(cx, |library, cx| {
        library.set_filter(Filter::Book(book.clone()), cx)
    });
    cx.run_until_parked();
    f.draw(cx);
    assert_eq!(
        f.tabs(cx),
        [a.clone(), c.clone(), b.clone()],
        "the reading tab is left as it was"
    );
    assert_eq!(f.open_note(cx), Some(b));
    assert!(
        f.tabs(cx).contains(&a) && f.tabs(cx).contains(&c),
        "both earlier notes stay open"
    );
}

/// While a Replace holds unsaved writes the editor refuses to switch; the
/// tab bar then keeps showing the note the editor still has.
// no-prop-test: one hold in one window; the editor's rule is in editor.rs.
#[gpui_kit::test]
fn the_tab_does_not_change_when_the_editor_refuses_to_switch(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.type_body(cx, "saved");
    cx.update(crate::services::hold_writes);
    f.type_body(cx, "waiting while writes are held");

    f.select(cx, &b);
    assert_eq!(f.open_note(cx), Some(a.clone()), "the editor stays");
    assert_eq!(f.tabs(cx), vec![a.clone()], "and so does its tab");
    assert_eq!(f.active_tab(cx), Some(a.clone()));

    cx.update(|cx| crate::services::release_writes(false, cx));
    cx.run_until_parked();
    assert_eq!(f.store_body(&a), "waiting while writes are held");
    f.select(cx, &b);
    assert_eq!(f.tabs(cx), vec![b.clone()], "the switch lands once saved");
    assert_eq!(f.open_note(cx), Some(b));
}

/// Open in New Tab from a note's menu opens a tab without disturbing the
/// note showing.
// no-prop-test: one command in one window.
#[gpui_kit::test]
fn open_in_new_tab_from_the_menu_adds_a_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.run(cx, &b, NoteCommand::OpenInNewTab);
    assert_eq!(f.tabs(cx), [a, b.clone()]);
    assert_eq!(f.open_note(cx), Some(b));
}

/// ⌘↩ in the focused list opens its note in a new tab (which, for the note
/// already showing, just shows that tab) and loses nothing.
// no-prop-test: one key in one window.
#[gpui_kit::test]
fn cmd_enter_in_the_list_opens_the_focused_note_in_a_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    f.select(cx, &a);
    let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
    f.window
        .update(cx, |_, window, cx| window.focus(&list.focus_handle(cx), cx))
        .unwrap();
    f.keys(cx, "cmd-enter");
    assert_eq!(f.tabs(cx), [a.clone()], "already open: its tab shows");
    assert_eq!(f.open_note(cx), Some(a));
}

/// Dragging a tab to a new place keeps the same note showing.
// no-prop-test: one drop in one window; the model's rule is a property in
// tabs.rs.
#[gpui_kit::test]
fn dragging_a_tab_reorders_the_row(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    let c = f.note("C", "text");
    f.select(cx, &a);
    f.middle_click(cx, &b);
    f.middle_click(cx, &c);
    let bar = cx.read(|cx| f.workspace.read(cx).tab_bar.clone());
    bar.update(cx, |_, cx| cx.emit(TabBarEvent::Reorder { from: 2, to: 0 }));
    cx.run_until_parked();
    assert_eq!(f.tabs(cx), [c.clone(), a, b]);
    assert_eq!(f.open_note(cx), Some(c));
}

/// ⇧⌘T brings back the last closed tab.
// no-prop-test: one close and one key in one window.
#[gpui_kit::test]
fn reopen_closed_tab_brings_it_back(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.middle_click(cx, &b);
    f.keys(cx, CLOSE_TAB);
    assert_eq!(f.tabs(cx), [a.clone()]);
    f.keys(cx, "cmd-shift-t");
    assert_eq!(f.tabs(cx), [a, b.clone()]);
    assert_eq!(f.open_note(cx), Some(b));
}

/// ⇧⌘1…⇧⌘9 show a tab by its place at the start of the row.
// no-prop-test: two keys in one window.
#[gpui_kit::test]
fn go_to_tab_shows_the_numbered_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    let c = f.note("C", "text");
    f.select(cx, &a);
    f.middle_click(cx, &b);
    f.middle_click(cx, &c);
    f.keys(cx, "cmd-shift-1");
    assert_eq!(f.active_tab(cx), Some(a.clone()));
    f.keys(cx, "cmd-shift-2");
    assert_eq!(f.active_tab(cx), Some(b.clone()));
    assert_eq!(f.open_note(cx), Some(b));
    f.keys(cx, "cmd-shift-9");
    assert_eq!(f.active_tab(cx), Some(c));
}

/// A menu command that works on the open note (Lock, History, Publish,
/// Share) opens a note that isn't showing first, as it did on master.
// no-prop-test: two commands in one window.
#[gpui_kit::test]
fn commands_open_a_note_that_is_not_showing_first(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);

    f.run(cx, &b, NoteCommand::Lock);
    assert_eq!(f.open_note(cx), Some(b.clone()));
    assert!(f.dialog_open(cx), "locking asks for the password");
    f.window
        .update(cx, |_, window, cx| window.close_dialog(cx))
        .unwrap();
    cx.run_until_parked();

    f.select(cx, &a);
    f.run(cx, &b, NoteCommand::ShowHistory);
    assert_eq!(f.open_note(cx), Some(b.clone()));
    assert!(f.dialog_open(cx), "history shows its dialog");
}

/// Ctrl+Tab and Ctrl+Shift+Tab walk the row, wrapping at both ends.
// no-prop-test: three keys in one window; wrapping is a property in
// tabs.rs.
#[gpui_kit::test]
fn ctrl_tab_walks_the_row(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    let c = f.note("C", "text");
    f.select(cx, &a);
    f.middle_click(cx, &b);
    f.middle_click(cx, &c);
    assert_eq!(f.active_tab(cx), Some(c.clone()));

    f.keys(cx, "ctrl-tab");
    assert_eq!(f.active_tab(cx), Some(a.clone()));
    assert_eq!(f.open_note(cx), Some(a.clone()));

    f.keys(cx, "ctrl-shift-tab");
    assert_eq!(f.active_tab(cx), Some(c.clone()));
}

/// Each tab keeps its own back and forward trail and cursor.
// no-prop-test: one window, two tabs; the history's rules are properties in
// nav_history.rs.
#[gpui_kit::test]
fn each_tab_keeps_its_own_history_and_place(cx: &mut TestAppContext) {
    let f = setup(cx);
    let long = (0..300)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let a = f.note("A", &long);
    let b = f.note("B", "text");
    let c = f.note("C", "text");
    let d = f.note("D", "text");
    let e = f.note("E", "text");

    f.select(cx, &a);
    f.with_editor(cx, |editor, window, cx| {
        editor.focus_body(window, cx);
        editor.reveal_line(150, window, cx);
    });
    f.draw(cx);
    let left = f
        .with_editor(cx, |editor, _, cx| editor.place(cx))
        .unwrap()
        .1;

    // A second tab, where C and then D are opened.
    f.middle_click(cx, &b);
    f.select(cx, &c);
    f.keys(cx, "ctrl-tab");
    assert_eq!(f.active_tab(cx), Some(a.clone()));
    f.select(cx, &d);
    f.select(cx, &e);

    // The first tab goes back through D to A, where the reader was, and
    // forward again.
    f.keys(cx, "cmd-[");
    assert_eq!(f.open_note(cx), Some(d.clone()));
    f.keys(cx, "cmd-[");
    assert_eq!(f.open_note(cx), Some(a.clone()));
    let back = f
        .with_editor(cx, |editor, _, cx| editor.place(cx))
        .unwrap()
        .1;
    assert_eq!(back.caret_line, 150);
    assert_eq!(back.scroll_y, left.scroll_y);
    f.keys(cx, "cmd-]");
    assert_eq!(f.open_note(cx), Some(d.clone()));

    // The second tab's trail is its own: B, then C.
    f.keys(cx, "ctrl-tab");
    assert_eq!(f.active_tab(cx), Some(c.clone()));
    assert_eq!(f.open_note(cx), Some(c.clone()));
    f.keys(cx, "cmd-[");
    assert_eq!(f.open_note(cx), Some(b.clone()));
}

/// Trashing a note closes its tab, and the neighbour shows.
// no-prop-test: one change in one window; the model's rule is in tabs.rs.
#[gpui_kit::test]
fn trashing_a_note_closes_its_tab(cx: &mut TestAppContext) {
    let f = setup(cx);
    let a = f.note("A", "text");
    let b = f.note("B", "text");
    f.select(cx, &a);
    f.middle_click(cx, &b);
    assert_eq!(f.tabs(cx), [a.clone(), b.clone()]);

    f.store
        .put(
            DocType::Note,
            json!({"_id": b, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    cx.executor().advance_clock(Duration::from_millis(700));
    f.draw(cx);
    assert_eq!(f.tabs(cx), vec![a.clone()], "the trashed note's tab goes");
    assert_eq!(f.open_note(cx), Some(a));
}

/// The tabs and the one showing kept in the settings come back at launch.
// no-prop-test: one launch from one config; the model's restore rules are
// properties in tabs.rs.
#[gpui_kit::test]
fn tabs_restore_from_the_settings(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let made: Vec<String> = [("A", "one"), ("B", "two"), ("C", "three")]
        .into_iter()
        .map(|(title, body)| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        })
        .collect();
    let mut config = AppConfig::default();
    config.layout.open_tabs = vec![made[0].clone(), made[2].clone()];
    config.layout.active_tab = 1;
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::workspace::bind_keys(cx);
        cx.set_global(AppState {
            store: store.clone(),
            data_dir: dir.path().to_path_buf(),
            config,
        });
        cx.set_global(Services::default());
    });
    let (window, workspace) = cx
        .update(|cx| {
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            })
        })
        .unwrap();
    window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    let open = cx.read(|cx| workspace.read(cx).tabs.ids());
    assert_eq!(open, [made[0].clone(), made[2].clone()]);
    let active = cx.read(|cx| workspace.read(cx).tabs.active_id().map(str::to_owned));
    assert_eq!(active, Some(made[2].clone()));
    let editor = cx.read(|cx| workspace.read(cx).editor.clone());
    let note = cx.read(|cx| editor.read(cx).note_id().map(str::to_owned));
    assert_eq!(note, Some(made[2].clone()));
}
