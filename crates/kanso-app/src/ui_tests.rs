//! The workspace in a headless window: autosave, switching notes, and
//! changes made elsewhere while a note is open. These are where an editor
//! loses data, so they run against the real views and store.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    notes: [String; 2],
}

fn note_body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

/// Whether `text` is the body of a live note or of a version in history.
fn persisted(store: &Store, ids: &[String], text: &str) -> bool {
    ids.iter().any(|id| {
        store.note(id).unwrap().is_some_and(|n| n.body == text)
            || store
                .revisions(id)
                .unwrap()
                .iter()
                .any(|r| r.fields.get("body").and_then(|v| v.as_str()) == Some(text))
    })
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let note = |title: &str| {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": "start", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let notes = [note("a"), note("b")];
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        store,
        window,
        workspace,
        notes,
    }
}

impl Fixture {
    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
    }

    fn type_body(&self, cx: &mut TestAppContext, body: &str) {
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.edit_for_test(body, window, cx))
            })
            .unwrap();
    }

    fn editor<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&crate::editor::NoteEditor, &gpui_kit::App) -> R,
    ) -> R {
        cx.read(|cx| {
            let editor = self.workspace.read(cx).editor.clone();
            f(editor.read(cx), cx)
        })
    }

    fn update_editor(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(
            &mut crate::editor::NoteEditor,
            &mut gpui_kit::Window,
            &mut gpui_kit::Context<crate::editor::NoteEditor>,
        ),
    ) {
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| f(editor, window, cx))
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// Draws, so focus, actions and element bounds find the views.
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

    fn workspace<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&Workspace, &gpui_kit::App) -> R,
    ) -> R {
        cx.read(|cx| f(self.workspace.read(cx), cx))
    }

    /// The settings as written to disk.
    fn saved_config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self._dir.path())).unwrap()
    }

    fn open_note(&self, cx: &mut TestAppContext) -> Option<String> {
        self.editor(cx, |e, _| e.note_id().map(str::to_owned))
    }

    fn edit_elsewhere(&self, cx: &mut TestAppContext, id: &str, body: &str) {
        self.store
            .put(
                DocType::Note,
                json!({"_id": id, "body": body}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn autosave_writes_after_a_pause(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed");
    cx.run_until_parked();
    assert_eq!(
        note_body(&f.store, &f.notes[0]),
        "start",
        "saved before the pause"
    );
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed");
}

#[gpui_kit::test]
fn switching_notes_saves_pending_edits(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "unsaved yet");
    f.select(cx, &f.notes[1]);
    assert_eq!(note_body(&f.store, &f.notes[0]), "unsaved yet");
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "start");
}

#[gpui_kit::test]
fn a_change_elsewhere_reloads_a_clean_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.edit_elsewhere(cx, &f.notes[0], "from the api");
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "from the api");
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn keeping_mine_over_a_change_elsewhere_keeps_theirs_in_history(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "mine");
    f.edit_elsewhere(cx, &id, "theirs");
    assert!(f.editor(cx, |e, _| e.has_incoming()), "asks which to keep");
    // The pending autosave must not overwrite theirs while the question is open.
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &id), "theirs");

    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    editor.update(cx, |editor, cx| editor.keep_mine(cx));
    cx.run_until_parked();
    assert_eq!(note_body(&f.store, &id), "mine");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "theirs"),
        "theirs is in history"
    );
}

#[gpui_kit::test]
fn using_theirs_keeps_my_draft_in_history(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "my draft");
    f.edit_elsewhere(cx, &id, "theirs");
    let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
    f.window
        .update(cx, |_, window, cx| {
            editor.update(cx, |editor, cx| editor.take_theirs(window, cx))
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(f.editor(cx, |e, cx| e.body_for_test(cx)), "theirs");
    assert_eq!(note_body(&f.store, &id), "theirs");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "my draft"),
        "my draft is in history"
    );
}

#[gpui_kit::test]
fn quitting_from_a_window_saves_the_open_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        cx.on_action(|_: &crate::workspace::Quit, cx| crate::quit(cx));
    });
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed then quit");
    // ⌘Q arrives while the window is being updated.
    cx.dispatch_action(f.window, crate::workspace::Quit);
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed then quit");
}

#[gpui_kit::test]
fn quitting_the_app_saves_the_open_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.type_body(cx, "typed then logged out");
    // As from the Dock or at logout, without the Quit action.
    cx.quit();
    assert_eq!(note_body(&f.store, &f.notes[0]), "typed then logged out");
}

#[gpui_kit::test]
fn leaving_a_note_before_answering_keeps_mine(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "mine before switch");
    f.edit_elsewhere(cx, &id, "theirs");
    assert!(f.editor(cx, |e, _| e.has_incoming()));
    f.select(cx, &f.notes[1]);
    assert_eq!(note_body(&f.store, &id), "mine before switch");
    let history = f.store.revisions(&id).unwrap();
    assert!(
        history.iter().any(|r| r.fields["body"] == "theirs"),
        "theirs is in history"
    );
}

#[gpui_kit::test]
fn a_note_deleted_elsewhere_asks_before_dropping_edits(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "unsaved when deleted");
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    assert!(f.editor(cx, |e, _| e.has_incoming()), "asks what to do");
    assert_eq!(f.open_note(cx).as_deref(), Some(id.as_str()));
    // The pending autosave must not bring the note back on its own.
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert!(f.store.note(&id).unwrap().is_none());

    f.update_editor(cx, |editor, _, cx| editor.keep_mine(cx));
    assert_eq!(note_body(&f.store, &id), "unsaved when deleted");
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn discarding_a_note_deleted_elsewhere_closes_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.type_body(cx, "discard me");
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    f.update_editor(cx, |editor, window, cx| editor.take_theirs(window, cx));
    assert_eq!(f.open_note(cx), None);
    assert!(f.store.note(&id).unwrap().is_none(), "stays deleted");
}

#[gpui_kit::test]
fn a_clean_note_deleted_elsewhere_closes(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.notes[0].clone();
    f.select(cx, &id);
    f.store.delete(&id, Origin::Sync).unwrap();
    cx.run_until_parked();
    assert_eq!(f.open_note(cx), None);
    assert!(!f.editor(cx, |e, _| e.has_incoming()));
}

#[gpui_kit::test]
fn keeping_a_note_whose_notebook_is_gone_moves_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    let other = f
        .store
        .put(
            DocType::Book,
            json!({"name": "Gone"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let id = f
        .store
        .put(
            DocType::Note,
            json!({"title": "c", "body": "start", "bookId": other}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    cx.run_until_parked();
    f.select(cx, &id);
    f.type_body(cx, "outlived its notebook");
    f.store.delete(&id, Origin::Sync).unwrap();
    f.store.delete(&other, Origin::Sync).unwrap();
    cx.run_until_parked();
    f.update_editor(cx, |editor, _, cx| editor.keep_mine(cx));
    let note = f.store.note(&id).unwrap().expect("brought back");
    assert_eq!(note.body, "outlived its notebook");
    assert_ne!(note.book_id, other);
}

/// Things a user, another app or another Mac can do while a note is open.
#[derive(Debug, Clone)]
enum Op {
    Type,
    Wait,
    EditElsewhere,
    DeleteElsewhere,
    Switch,
    KeepMine,
    TakeTheirs,
    Save,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => Just(Op::Type),
        2 => Just(Op::Wait),
        2 => Just(Op::EditElsewhere),
        1 => Just(Op::DeleteElsewhere),
        2 => Just(Op::Switch),
        1 => Just(Op::KeepMine),
        1 => Just(Op::TakeTheirs),
        1 => Just(Op::Save),
    ]
}

/// Runs `ops` and checks after each one that every text typed into the
/// editor is still there or has reached the store (as a live note or in
/// history), and that quitting at the end leaves nothing unsaved. Text
/// replaced by more typing before anything else happened is only keystrokes
/// and may go.
fn typed_text_survives(ops: &[Op]) -> Result<(), TestCaseError> {
    let mut cx = TestAppContext::single();
    let cx = &mut cx;
    let f = setup(cx);
    let ids = f.notes.to_vec();
    let mut current = 0;
    f.select(cx, &ids[current]);
    let mut outstanding: Vec<String> = Vec::new();
    for (step, op) in ops.iter().enumerate() {
        let open = f.open_note(cx);
        match op {
            Op::Type => {
                if open.is_none() {
                    continue;
                }
                let replaced = f.editor(cx, |e, cx| e.body_for_test(cx));
                outstanding.retain(|t| *t != replaced);
                let text = format!("typed {step}");
                f.type_body(cx, &text);
                outstanding.push(text);
            }
            Op::Wait => cx.executor().advance_clock(Duration::from_secs(1)),
            Op::EditElsewhere => {
                if let Some(id) = open {
                    f.store
                        .put(
                            DocType::Note,
                            json!({"_id": id, "body": format!("elsewhere {step}")}),
                            Put::Merge,
                            Origin::Sync,
                        )
                        .ok();
                }
            }
            Op::DeleteElsewhere => {
                if let Some(id) = open {
                    f.store.delete(&id, Origin::Sync).ok();
                }
            }
            Op::Switch => {
                current = 1 - current;
                f.select(cx, &ids[current]);
            }
            Op::KeepMine => f.update_editor(cx, |e, _, cx| e.keep_mine(cx)),
            Op::TakeTheirs => f.update_editor(cx, |e, window, cx| e.take_theirs(window, cx)),
            Op::Save => f.update_editor(cx, |e, _, cx| e.flush(cx)),
        }
        cx.run_until_parked();
        let in_editor = f.editor(cx, |e, cx| e.body_for_test(cx));
        outstanding.retain(|t| !persisted(&f.store, &ids, t));
        for text in &outstanding {
            prop_assert_eq!(
                text,
                &in_editor,
                "'{}' was lost after step {} ({:?})",
                text,
                step,
                op
            );
        }
    }
    cx.quit();
    outstanding.retain(|t| !persisted(&f.store, &ids, t));
    prop_assert!(outstanding.is_empty(), "lost on quit: {:?}", outstanding);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Model-based: no sequence of edits, changes and deletions elsewhere,
    /// note switches, answers and quitting loses typed text.
    #[test]
    fn prop_typed_text_is_never_lost(ops in prop::collection::vec(op(), 1..24)) {
        typed_text_survives(&ops)?;
    }
}

fn saved_last_note(f: &Fixture) -> Option<String> {
    let path = crate::paths::config_path(f._dir.path());
    AppConfig::load(&path).unwrap().last_note
}

#[gpui_kit::test]
fn the_selected_note_is_saved_once_selection_settles(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.select(cx, &f.notes[0]);
    f.select(cx, &f.notes[1]);
    assert_eq!(saved_last_note(&f), None, "written on every click");
    cx.executor().advance_clock(Duration::from_secs(3));
    cx.run_until_parked();
    assert_eq!(saved_last_note(&f), Some(f.notes[1].clone()));
}

#[gpui_kit::test]
fn quitting_saves_the_selected_note_right_away(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::watch_for_quit);
    f.select(cx, &f.notes[1]);
    cx.quit();
    assert_eq!(saved_last_note(&f), Some(f.notes[1].clone()));
}

/// A press on bare chrome leaves the keyboard in the note: on the editor's
/// top band, where a drag moves the window and a double-click zooms it, and
/// on its footer. The workspace's own focus is only for when nothing else
/// has the keyboard.
#[gpui_kit::test]
fn a_press_on_the_chrome_keeps_the_keyboard(cx: &mut TestAppContext) {
    use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
    use gpui_kit::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, point, px};

    let f = setup(cx);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    let body = f
        .window
        .update(cx, |_, window, cx| {
            let editor = f.workspace.read(cx).editor.clone();
            editor.update(cx, |editor, cx| editor.focus_body(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            window.focused(cx).expect("the body takes the keyboard")
        })
        .unwrap();
    cx.run_until_parked();
    // The middle of the editor pane, between the path and the buttons.
    let (x, height) = f
        .window
        .update(cx, |_, window, cx| {
            let workspace = f.workspace.read(cx);
            let widths = workspace.pane_widths;
            let size = window.viewport_size();
            let left = widths.list
                + if workspace.show_sidebar {
                    widths.sidebar
                } else {
                    0.
                };
            ((left + f32::from(size.width)) / 2., f32::from(size.height))
        })
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let press = |vcx: &mut gpui_kit::VisualTestContext, y: f32, click_count: usize| {
        let position = point(px(x), px(y));
        vcx.simulate_event(MouseDownEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
            first_mouse: false,
        });
        vcx.simulate_event(MouseUpEvent {
            position,
            modifiers: Modifiers::none(),
            button: MouseButton::Left,
            click_count,
        });
    };
    let body_focused =
        |vcx: &mut gpui_kit::VisualTestContext| vcx.update(|window, _| body.is_focused(window));
    press(&mut vcx, TOP_BAND_HEIGHT / 2., 1);
    assert!(body_focused(&mut vcx), "a press on the top band");
    press(&mut vcx, TOP_BAND_HEIGHT / 2., 2);
    assert!(body_focused(&mut vcx), "a double-click on the top band");
    press(&mut vcx, height - BOTTOM_BAND_HEIGHT / 2., 1);
    assert!(body_focused(&mut vcx), "a press on the footer");
    // Typing afterwards still reaches the note.
    vcx.simulate_input("!");
    let typed = f.editor(cx, |e, cx| e.body_for_test(cx));
    assert!(typed.contains('!'), "{typed:?}");
}

/// A second window, as on the next launch, reading the settings the first
/// one wrote.
fn reopen(cx: &mut TestAppContext) -> Entity<Workspace> {
    let (_, workspace) = cx.update(|cx| {
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    workspace
}

/// ⌘⇧D leaves the note alone in the window, with the keyboard in its text,
/// and the next launch starts the same way. The palette still opens.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn focus_mode_leaves_the_note_alone_and_is_kept(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    assert!(vcx.debug_bounds("sidebar-pane").is_some());
    assert!(vcx.debug_bounds("list-pane").is_some());

    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(f.saved_config().layout.focus_mode);
    assert_eq!(vcx.debug_bounds("sidebar-pane"), None);
    assert_eq!(vcx.debug_bounds("list-pane"), None);
    let editor = vcx.debug_bounds("editor-pane").unwrap();
    let width = vcx.update(|window, _| window.viewport_size().width);
    assert_eq!(editor.size.width, width, "the editor has the window");
    let editor_view = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(
        vcx.update(|window, cx| editor_view.read(cx).body_focused(window, cx)),
        "the keyboard goes to the text"
    );
    // Typing reaches the note.
    vcx.simulate_input("!");
    assert!(
        f.editor(&mut vcx, |e, cx| e.body_for_test(cx))
            .contains('!')
    );

    // The palette still opens, over the note.
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    vcx.simulate_keystrokes("cmd-k");
    assert!(vcx.read(|cx| palette.read(cx).is_open()));

    // The next launch starts in focus mode.
    let next = reopen(&mut vcx);
    assert!(vcx.read(|cx| next.read(cx).focus_mode));

    vcx.simulate_keystrokes("escape");
    assert!(!vcx.read(|cx| palette.read(cx).is_open()));
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(!f.saved_config().layout.focus_mode);
    assert!(vcx.debug_bounds("sidebar-pane").is_some());
    assert!(vcx.debug_bounds("list-pane").is_some());
}

/// Escape leaves focus mode only once nothing else wants it: the first
/// Escape with the palette open closes the palette; the next, from the
/// text, leaves focus mode.
// no-prop-test: drives one window through keystrokes.
#[gpui_kit::test]
fn escape_leaves_focus_mode_last(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_keystrokes("cmd-shift-d");
    f.draw(&mut vcx);
    let palette = f.workspace(&mut vcx, |w, _| w.palette.clone());
    vcx.simulate_keystrokes("cmd-k");
    f.draw(&mut vcx);
    assert!(vcx.read(|cx| palette.read(cx).is_open()));

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(
        !vcx.read(|cx| palette.read(cx).is_open()),
        "the palette closes"
    );
    assert!(
        f.workspace(&mut vcx, |w, _| w.focus_mode),
        "focus mode stays"
    );
    let editor = f.workspace(&mut vcx, |w, _| w.editor.clone());
    assert!(vcx.update(|window, cx| editor.read(cx).body_focused(window, cx)));

    vcx.simulate_keystrokes("escape");
    f.draw(&mut vcx);
    assert!(
        !f.workspace(&mut vcx, |w, _| w.focus_mode),
        "the next one leaves"
    );
    assert!(!f.saved_config().layout.focus_mode);
    // Out of focus mode, Escape in the text leaves things as they are.
    vcx.simulate_keystrokes("escape");
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
}

/// ⌘\ folds the sidebar to its icon rail rather than hiding it, keeps that
/// for next time, and from focus mode brings everything back.
// no-prop-test: drives one window through keystrokes; the widths' rules
// are properties in layout.rs.
#[gpui_kit::test]
fn the_sidebar_folds_to_its_rail(cx: &mut TestAppContext) {
    use crate::layout::rail_width;

    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.select(cx, &f.notes[0]);
    f.draw(cx);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let sidebar = f.workspace(&mut vcx, |w, _| w.sidebar.clone());
    let full = vcx.debug_bounds("sidebar-pane").unwrap().size.width;

    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert!(!f.saved_config().appearance.show_sidebar);
    assert!(vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    let rail = vcx.debug_bounds("sidebar-pane").expect("the rail stays");
    assert_eq!(f32::from(rail.size.width), rail_width(false));
    assert!(rail.size.width < full);
    // The list starts where the rail ends.
    let list = vcx.debug_bounds("list-pane").unwrap();
    assert_eq!(list.left(), rail.right());

    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert!(f.saved_config().appearance.show_sidebar);
    assert!(!vcx.read(|cx| sidebar.read(cx).is_collapsed()));
    assert_eq!(vcx.debug_bounds("sidebar-pane").unwrap().size.width, full);

    // Folded, then in focus mode: ⌘\ brings the whole sidebar back.
    vcx.simulate_keystrokes("cmd-\\ cmd-shift-d");
    f.draw(&mut vcx);
    assert_eq!(vcx.debug_bounds("sidebar-pane"), None);
    vcx.simulate_keystrokes("cmd-\\");
    f.draw(&mut vcx);
    assert!(!f.workspace(&mut vcx, |w, _| w.focus_mode));
    assert!(f.workspace(&mut vcx, |w, _| w.show_sidebar));
    assert_eq!(vcx.debug_bounds("sidebar-pane").unwrap().size.width, full);
}

/// Opening or closing the Status group is kept for the next launch.
// no-prop-test: one toggle through the settings file; the open rule is a
// property in the sidebar's tests.
#[gpui_kit::test]
fn the_status_group_remembers(cx: &mut TestAppContext) {
    let f = setup(cx);
    let sidebar = f.workspace(cx, |w, _| w.sidebar.clone());
    assert_eq!(f.saved_config().layout.status_open, None);
    let before = cx.read(|cx| sidebar.read(cx).is_status_open(cx));
    sidebar.update(cx, |sidebar, cx| sidebar.toggle_status_group(cx));
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| sidebar.read(cx).is_status_open(cx)), !before);
    assert_eq!(f.saved_config().layout.status_open, Some(!before));
    let next = reopen(cx);
    let next_sidebar = cx.read(|cx| next.read(cx).sidebar.clone());
    assert_eq!(
        cx.read(|cx| next_sidebar.read(cx).is_status_open(cx)),
        !before
    );
    sidebar.update(cx, |sidebar, cx| sidebar.toggle_status_group(cx));
    assert_eq!(f.saved_config().layout.status_open, Some(before));
}

/// Opens the first note in Split in a window wide enough for both sides.
fn split_fixture(cx: &mut TestAppContext, body: &str) -> Fixture {
    let f = setup(cx);
    cx.update(crate::workspace::bind_keys);
    f.edit_elsewhere(cx, &f.notes[0], body);
    f.select(cx, &f.notes[0]);
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_resize(gpui_kit::size(gpui_kit::px(1600.), gpui_kit::px(1000.)));
    vcx.simulate_keystrokes("cmd-2");
    // The preview renders after a pause; the divider is placed once the
    // editor has been measured.
    vcx.executor().advance_clock(Duration::from_secs(1));
    for _ in 0..3 {
        f.draw(&mut vcx);
    }
    f
}

/// Dragging the divider moves it, keeps where it was left as a share of
/// the editor for next time, and neither side gets under its minimum, from
/// a drag or from a kept share.
// no-prop-test: drives the real divider with the pointer; the placement
// rules are properties in split.rs.
#[gpui_kit::test]
fn a_dragged_divider_is_kept_and_both_sides_keep_room(cx: &mut TestAppContext) {
    use crate::split::SPLIT_MIN_SIDE;
    use gpui_kit::{Modifiers, MouseButton, point, px};

    let f = split_fixture(cx, "one\ntwo");
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let pane = vcx.debug_bounds("editor-pane").unwrap();
    let width = f32::from(pane.size.width);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert_eq!(sizes.len(), 2, "{sizes:?}");
    assert!((sizes[0] - width / 2.).abs() <= 1., "{sizes:?} of {width}");
    assert!(
        (sizes[0] + sizes[1] - width).abs() <= 1.,
        "{sizes:?} of {width}"
    );

    let y = pane.top() + pane.size.height / 2.;
    let drag = |vcx: &mut gpui_kit::VisualTestContext, from: f32, to: f32| {
        let from = point(pane.left() + px(from), y);
        vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
        // Past the drag threshold, then in steps to where it is let go.
        for step in 1..=4 {
            let x = from.x + (pane.left() + px(to) - from.x) * (step as f32 / 4.);
            vcx.simulate_mouse_move(point(x, y), MouseButton::Left, Modifiers::none());
            f.draw(vcx);
        }
        vcx.simulate_mouse_up(
            point(pane.left() + px(to), y),
            MouseButton::Left,
            Modifiers::none(),
        );
        f.draw(vcx);
    };
    let left = sizes[0];
    drag(&mut vcx, left, width * 0.3);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert!((sizes[0] - width * 0.3).abs() <= 2., "{sizes:?} of {width}");
    let kept = f.saved_config().layout.split_ratio;
    assert!((kept - 0.3).abs() < 0.01, "kept {kept}");
    assert!((f.editor(&mut vcx, |e, _| e.split_ratio()) - kept).abs() < 1e-6);

    // Dragged nearly to the edge, the source keeps its minimum.
    drag(&mut vcx, sizes[0], 40.);
    let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
    assert!((sizes[0] - SPLIT_MIN_SIDE).abs() <= 1., "{sizes:?}");
    let kept = f.saved_config().layout.split_ratio;
    assert!((kept * width - SPLIT_MIN_SIDE).abs() <= 2., "kept {kept}");

    // Shares kept from a wider window still leave both sides room.
    for (ratio, left) in [
        (0.02, SPLIT_MIN_SIDE),
        (0.98, width - SPLIT_MIN_SIDE),
        (0.6, width * 0.6),
    ] {
        f.update_editor(&mut vcx, |e, _, cx| e.keep_split_ratio(ratio, cx));
        f.draw(&mut vcx);
        f.draw(&mut vcx);
        let sizes = f.editor(&mut vcx, |e, cx| e.split_sizes(cx));
        assert!(
            (sizes[0] - left).abs() <= 1.,
            "{ratio}: {sizes:?} of {width}"
        );
        assert!(sizes[1] >= SPLIT_MIN_SIDE - 1., "{ratio}: {sizes:?}");
    }

    // The next launch puts the divider where it was left.
    let next = reopen(&mut vcx);
    let next_editor = vcx.read(|cx| next.read(cx).editor.clone());
    let saved = f.saved_config().layout.split_ratio;
    assert_eq!(vcx.read(|cx| next_editor.read(cx).split_ratio()), saved);
}

/// Scrolling the source scrolls the preview beside it by the same share:
/// top to top, middle to middle, end to end.
// no-prop-test: the mapping is a property in split.rs; this checks the
// views are wired to it.
#[gpui_kit::test]
fn the_preview_follows_the_source(cx: &mut TestAppContext) {
    use gpui_kit::{Modifiers, ScrollDelta, ScrollWheelEvent, TouchPhase, point, px};

    let body = (1..=300)
        .map(|n| {
            if n % 20 == 1 {
                format!("# Part {n}")
            } else {
                format!("Line {n} of the note.")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let f = split_fixture(cx, &body);
    let (top, max) = f.editor(cx, |e, cx| e.preview_scroll(cx));
    assert_eq!(top, 0.);
    assert!(max > 1000., "the preview can scroll: {max}");
    let scroll = |cx: &mut TestAppContext, share: f32| {
        f.update_editor(cx, |e, _, cx| e.scroll_source_for_test(share, cx));
        for _ in 0..2 {
            f.draw(cx);
        }
        let source = f.editor(cx, |e, cx| e.source_scroll(cx));
        let (top, max) = f.editor(cx, |e, cx| e.preview_scroll(cx));
        (source, top / max)
    };
    // A scroll set from outside the wheel, as by the scrollbar.
    let (source, share) = scroll(cx, 0.5);
    assert!(source > 0., "the source scrolled");
    assert!((share - 0.5).abs() < 0.05, "the preview is {share} down");
    let (_, share) = scroll(cx, 1.);
    assert!(share > 0.95, "the preview is {share} down");
    let (source, share) = scroll(cx, 0.);
    assert_eq!((source, share), (0., 0.));

    // The wheel over the source.
    let pane = gpui_kit::VisualTestContext::from_window(f.window, cx)
        .debug_bounds("editor-pane")
        .unwrap();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    vcx.simulate_event(ScrollWheelEvent {
        position: pane.origin + point(px(150.), pane.size.height / 2.),
        delta: ScrollDelta::Pixels(point(px(0.), px(-600.))),
        modifiers: Modifiers::none(),
        touch_phase: TouchPhase::Moved,
    });
    f.draw(&mut vcx);
    let source = f.editor(&mut vcx, |e, cx| e.source_scroll(cx));
    let (top, max) = f.editor(&mut vcx, |e, cx| e.preview_scroll(cx));
    assert_eq!(source, 600.);
    assert!(top > 0. && top < max / 2., "{top} of {max}");
}
