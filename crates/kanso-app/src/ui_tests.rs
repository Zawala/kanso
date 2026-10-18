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
