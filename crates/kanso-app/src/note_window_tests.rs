//! A note in a window of its own beside the main window, headless: opening
//! it, typing in either window and seeing it in the other at once, saving
//! without the two windows fighting, closing it, and the note going to the
//! trash or away for good while it is open.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use crate::config::{AppConfig, NoteWindowFrame};
use crate::editor::{NoteEditor, SaveState};
use crate::note_menu::NoteCommand;
use crate::note_window::{NoteWindow, note_windows};
use crate::services::Services;
use crate::workspace::{AppState, Workspace, for_each_editor};

struct Fixture {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    main: AnyWindowHandle,
    workspace: Entity<Workspace>,
    note: String,
    other: String,
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
    let (note, other) = (note("a"), note("b"));
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let (main, workspace) = cx.update(|cx| {
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
    let f = Fixture {
        dir,
        store,
        main,
        workspace,
        note,
        other,
    };
    f.select(cx, &f.note.clone());
    f
}

impl Fixture {
    fn select(&self, cx: &mut TestAppContext, id: &str) {
        let library = cx.read(|cx| self.workspace.read(cx).library.clone());
        let id = id.to_owned();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
    }

    fn main_editor(&self, cx: &mut TestAppContext) -> Entity<NoteEditor> {
        cx.read(|cx| self.workspace.read(cx).editor.clone())
    }

    /// "Open in New Window" from note `id`'s menu in the main window.
    fn open_window_for(&self, cx: &mut TestAppContext, id: &str) {
        self.run_in_main(cx, id, NoteCommand::OpenInNewWindow);
    }

    /// `command` from note `id`'s menu in the main window.
    fn run_in_main(&self, cx: &mut TestAppContext, id: &str, command: NoteCommand) {
        let (workspace, id) = (self.workspace.clone(), id.to_owned());
        self.main
            .update(cx, |_, window, cx| {
                workspace.update(cx, |workspace, cx| {
                    workspace.run_note_command(&id, command, window, cx)
                })
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn windows(&self, cx: &mut TestAppContext) -> Vec<(AnyWindowHandle, Entity<NoteWindow>)> {
        cx.update(note_windows)
    }

    /// The only note window, opened for the fixture's note.
    fn opened(&self, cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<NoteEditor>) {
        let windows = self.windows(cx);
        assert_eq!(windows.len(), 1, "one note window");
        let (handle, view) = windows[0].clone();
        let editor = cx.read(|cx| view.read(cx).editor.clone());
        (handle, editor)
    }

    fn saved_config(&self) -> AppConfig {
        AppConfig::load(&crate::paths::config_path(self.dir.path())).unwrap()
    }
}

/// Types `text` at the end of `editor`'s body, as keystrokes would.
fn type_at_end(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    editor: &Entity<NoteEditor>,
    text: &str,
) {
    let body = cx.read(|cx| editor.read(cx).body_state());
    window
        .update(cx, |_, window, cx| {
            body.update(cx, |state, cx| {
                let end = state.text().len();
                state.set_selected_range(end..end, cx);
                state.replace(text.to_owned(), window, cx);
            })
        })
        .unwrap();
    cx.run_until_parked();
}

fn body(cx: &mut TestAppContext, editor: &Entity<NoteEditor>) -> String {
    cx.read(|cx| editor.read(cx).body_for_test(cx))
}

fn save_state(cx: &mut TestAppContext, editor: &Entity<NoteEditor>) -> SaveState {
    cx.read(|cx| editor.read(cx).save_state(cx))
}

fn asks(cx: &mut TestAppContext, editor: &Entity<NoteEditor>) -> bool {
    cx.read(|cx| editor.read(cx).asks_about_a_change())
}

fn wait_for_autosave(cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
}

#[gpui_kit::test]
fn a_note_opens_in_one_window_of_its_own(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (_, editor) = f.opened(cx);
    assert_eq!(
        cx.read(|cx| editor.read(cx).note_id().map(str::to_owned)),
        Some(f.note.clone())
    );
    // Asked again, the same window comes forward.
    f.open_window_for(cx, &f.note.clone());
    assert_eq!(f.windows(cx).len(), 1);
    // Another note gets its own.
    f.open_window_for(cx, &f.other.clone());
    assert_eq!(f.windows(cx).len(), 2);
    // Remembered for the next launch.
    cx.executor().advance_clock(Duration::from_secs(3));
    cx.run_until_parked();
    let saved: Vec<String> = f
        .saved_config()
        .layout
        .note_windows
        .into_iter()
        .map(|w| w.note_id)
        .collect();
    assert_eq!(saved, vec![f.note.clone(), f.other.clone()]);
    // Every editor is reached, so settings and voice find the windows.
    let mut open = Vec::new();
    cx.update(|cx| {
        for_each_editor(cx, |editor, _| {
            open.push(editor.note_id().map(str::to_owned));
        })
    });
    assert_eq!(open.len(), 3);
}

#[gpui_kit::test]
fn note_windows_edit_live_together(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (window, theirs) = f.opened(cx);
    let mine = f.main_editor(cx);

    // Typed in the main window: in the note's window at once, before the
    // autosave, which writes it once.
    type_at_end(cx, f.main, &mine, " one");
    assert_eq!(body(cx, &theirs), "start one");
    assert_eq!(f.store.note(&f.note).unwrap().unwrap().body, "start");
    assert_eq!(save_state(cx, &theirs), SaveState::Saving);
    wait_for_autosave(cx);
    assert_eq!(f.store.note(&f.note).unwrap().unwrap().body, "start one");
    assert_eq!(save_state(cx, &mine), SaveState::Saved);
    assert_eq!(save_state(cx, &theirs), SaveState::Saved);

    // And the other way.
    type_at_end(cx, window, &theirs, " two");
    assert_eq!(body(cx, &mine), "start one two");
    wait_for_autosave(cx);
    assert_eq!(
        f.store.note(&f.note).unwrap().unwrap().body,
        "start one two"
    );
    assert!(!asks(cx, &mine) && !asks(cx, &theirs));

    // Typing in both before either saves: one text, no question.
    type_at_end(cx, f.main, &mine, " three");
    type_at_end(cx, window, &theirs, " four");
    assert_eq!(body(cx, &mine), "start one two three four");
    wait_for_autosave(cx);
    assert!(!asks(cx, &mine) && !asks(cx, &theirs));
    assert_eq!(
        f.store.note(&f.note).unwrap().unwrap().body,
        "start one two three four"
    );
    assert_eq!(save_state(cx, &mine), SaveState::Saved);
    assert_eq!(save_state(cx, &theirs), SaveState::Saved);
}

#[gpui_kit::test]
fn the_caret_stays_put_when_the_other_window_types(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (window, theirs) = f.opened(cx);
    let mine = f.main_editor(cx);
    // The note window's caret after "st".
    let state = cx.read(|cx| theirs.read(cx).body_state());
    window
        .update(cx, |_, _, cx| {
            state.update(cx, |state, cx| state.set_selected_range(2..2, cx))
        })
        .unwrap();
    type_at_end(cx, f.main, &mine, "!");
    let range = cx.read(|cx| state.read(cx).selected_range());
    assert_eq!(range, 2..2);
    assert_eq!(body(cx, &theirs), "start!");
}

#[gpui_kit::test]
fn a_main_window_opening_the_note_starts_from_the_typing(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (window, theirs) = f.opened(cx);
    let mine = f.main_editor(cx);
    f.select(cx, &f.other.clone());
    type_at_end(cx, window, &theirs, " typed");
    // Not saved yet; the main window opens the note with it.
    f.select(cx, &f.note.clone());
    assert_eq!(body(cx, &mine), "start typed");
    wait_for_autosave(cx);
    assert!(!asks(cx, &mine) && !asks(cx, &theirs));
    assert_eq!(f.store.note(&f.note).unwrap().unwrap().body, "start typed");
}

#[gpui_kit::test]
fn closing_a_note_window_saves_and_forgets_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (window, theirs) = f.opened(cx);
    type_at_end(cx, window, &theirs, " closing");
    let view = f.windows(cx)[0].1.clone();
    window
        .update(cx, |_, window, cx| {
            window.dispatch_action(Box::new(crate::note_window::CloseNoteWindow), cx);
            let _ = view;
        })
        .unwrap();
    cx.run_until_parked();
    assert!(f.windows(cx).is_empty());
    assert_eq!(
        f.store.note(&f.note).unwrap().unwrap().body,
        "start closing"
    );
    // The main window stays, and the closed one won't open next time.
    assert!(f.main.update(cx, |_, _, _| ()).is_ok());
    assert!(f.saved_config().layout.note_windows.is_empty());
}

#[gpui_kit::test]
fn a_trashed_note_stays_and_a_deleted_one_closes(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (_, theirs) = f.opened(cx);
    // Trashed from the main window: the note window shows it trashed.
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.note, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    cx.run_until_parked();
    assert_eq!(f.windows(cx).len(), 1);
    let trashed = cx.read(|cx| theirs.read(cx).current_note(cx).map(|n| n.is_trashed()));
    assert_eq!(trashed, Some(true));
    // Deleted for good: the window closes, and isn't opened next time.
    f.store.delete(&f.note, Origin::Ui).unwrap();
    cx.run_until_parked();
    assert!(f.windows(cx).is_empty());
    assert!(f.saved_config().layout.note_windows.is_empty());
}

#[gpui_kit::test]
fn note_windows_open_again_where_they_were(cx: &mut TestAppContext) {
    let f = setup(cx);
    let frame = |note_id: &str| NoteWindowFrame {
        note_id: note_id.to_owned(),
        x: 40.,
        y: 60.,
        width: 600.,
        height: 500.,
        sticky: false,
        color: Default::default(),
        closed: false,
    };
    cx.update(|cx| {
        AppState::update_config(cx, |config| {
            config.layout.note_windows = vec![frame(&f.note), frame("note:gone")];
        });
        crate::note_window::restore(cx);
    });
    cx.run_until_parked();
    let (_, editor) = f.opened(cx);
    assert_eq!(
        cx.read(|cx| editor.read(cx).note_id().map(str::to_owned)),
        Some(f.note.clone())
    );
    // The deleted note's window is forgotten.
    let saved: Vec<String> = f
        .saved_config()
        .layout
        .note_windows
        .into_iter()
        .map(|w| w.note_id)
        .collect();
    assert_eq!(saved, vec![f.note.clone()]);
}

/// What can happen to a note open in both windows.
#[derive(Debug, Clone)]
enum Op {
    TypeMain(u8),
    TypeWindow(u8),
    Wait,
    SaveMain,
    SaveWindow,
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0u8..26).prop_map(Op::TypeMain),
        3 => (0u8..26).prop_map(Op::TypeWindow),
        2 => Just(Op::Wait),
        1 => Just(Op::SaveMain),
        1 => Just(Op::SaveWindow),
    ]
}

/// Runs `ops` against the main window and a note window on one note, and
/// checks after each that both show the same text, and that neither ever
/// asks which version to keep. Once typing stops, the store has the text.
fn both_windows_agree(ops: &[Op]) -> Result<(), TestCaseError> {
    let mut cx = TestAppContext::single();
    let cx = &mut cx;
    let f = setup(cx);
    f.open_window_for(cx, &f.note.clone());
    let (window, theirs) = f.opened(cx);
    let mine = f.main_editor(cx);
    let mut expected = "start".to_owned();
    for (step, op) in ops.iter().enumerate() {
        match op {
            Op::TypeMain(c) | Op::TypeWindow(c) => {
                let text = char::from(b'a' + c).to_string();
                let (handle, editor) = match op {
                    Op::TypeMain(_) => (f.main, &mine),
                    _ => (window, &theirs),
                };
                type_at_end(cx, handle, editor, &text);
                expected.push_str(&text);
            }
            Op::Wait => wait_for_autosave(cx),
            Op::SaveMain => {
                mine.update(cx, |e, cx| e.flush(cx));
                cx.run_until_parked();
            }
            Op::SaveWindow => {
                theirs.update(cx, |e, cx| e.flush(cx));
                cx.run_until_parked();
            }
        }
        prop_assert_eq!(
            &body(cx, &mine),
            &expected,
            "main, step {} ({:?})",
            step,
            op
        );
        prop_assert_eq!(
            &body(cx, &theirs),
            &expected,
            "window, step {} ({:?})",
            step,
            op
        );
        prop_assert!(
            !asks(cx, &mine) && !asks(cx, &theirs),
            "asked at step {}",
            step
        );
    }
    wait_for_autosave(cx);
    prop_assert_eq!(f.store.note(&f.note).unwrap().unwrap().body, expected);
    prop_assert_eq!(save_state(cx, &mine), SaveState::Saved);
    prop_assert_eq!(save_state(cx, &theirs), SaveState::Saved);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Model-based: whatever order typing, pauses and saves come in across
    /// the two windows, they show one text, never ask which version to
    /// keep, and the store ends with it.
    #[test]
    fn prop_two_windows_never_fight(ops in prop::collection::vec(op(), 1..20)) {
        both_windows_agree(&ops)?;
    }
}

/// "Open as Sticky": one small window for the note, remembered as a sticky;
/// asked for its ordinary window, the note moves there and the sticky goes.
#[gpui_kit::test]
fn a_note_opens_as_a_sticky_and_back(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.run_in_main(cx, &f.note.clone(), NoteCommand::OpenAsSticky);
    let windows = f.windows(cx);
    assert_eq!(windows.len(), 1, "one sticky");
    assert!(cx.read(|cx| windows[0].1.read(cx).is_sticky()));
    // It opens in Preview, and leaves the view mode new windows use alone.
    let editor = cx.read(|cx| windows[0].1.read(cx).editor.clone());
    assert_eq!(
        cx.read(|cx| editor.read(cx).view_mode),
        crate::config::ViewMode::Preview
    );
    cx.run_until_parked();
    let saved = kept_frames(cx);
    assert_eq!(saved.len(), 1);
    assert!(saved[0].sticky);
    assert_eq!(saved[0].note_id, f.note);
    // Asked again, the same sticky comes forward.
    f.run_in_main(cx, &f.note.clone(), NoteCommand::OpenAsSticky);
    assert_eq!(f.windows(cx).len(), 1);
    // Its ordinary window replaces it.
    f.open_window_for(cx, &f.note.clone());
    let windows = f.windows(cx);
    assert_eq!(windows.len(), 1, "the sticky became a window");
    assert!(!cx.read(|cx| windows[0].1.read(cx).is_sticky()));
    cx.run_until_parked();
    // The window is open; the sticky is only kept, for where it was.
    let saved = kept_frames(cx);
    assert_eq!(saved.len(), 2);
    let open: Vec<_> = saved.iter().filter(|f| !f.closed).collect();
    assert_eq!(open.len(), 1);
    assert!(!open[0].sticky);
    assert!(saved.iter().any(|f| f.sticky && f.closed));
}

/// A closed sticky isn't opened at launch, and opens again in its colour
/// when asked for; closed again, it is kept, not forgotten.
#[gpui_kit::test]
fn a_closed_sticky_keeps_its_colour(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        AppState::update_config(cx, |config| {
            config.layout.note_windows = vec![NoteWindowFrame {
                note_id: f.note.clone(),
                x: 900.,
                y: 40.,
                width: 300.,
                height: 220.,
                sticky: true,
                color: crate::config::StickyColor::Purple,
                closed: true,
            }];
        });
        crate::note_window::restore(cx);
    });
    cx.run_until_parked();
    assert!(f.windows(cx).is_empty());
    f.run_in_main(cx, &f.note.clone(), NoteCommand::OpenAsSticky);
    let windows = f.windows(cx);
    assert_eq!(windows.len(), 1);
    assert!(cx.read(|cx| windows[0].1.read(cx).is_sticky()));
    cx.run_until_parked();
    let saved = kept_frames(cx);
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].color, crate::config::StickyColor::Purple);
    assert!(!saved[0].closed);
    let (handle, view) = windows[0].clone();
    let editor = cx.read(|cx| view.read(cx).editor.clone());
    handle
        .update(cx, |_, window, cx| {
            // Its editor showing and focused, as when the user types.
            editor.update(cx, |editor, cx| {
                editor.set_view_mode(crate::config::ViewMode::Edit, cx);
                editor.focus_body(window, cx);
            });
        })
        .unwrap();
    cx.run_until_parked();
    handle
        .update(cx, |_, window, cx| {
            window.dispatch_action(Box::new(crate::note_window::CloseNoteWindow), cx);
        })
        .unwrap();
    cx.run_until_parked();
    assert!(f.windows(cx).is_empty());
    let saved = kept_frames(cx);
    assert_eq!(saved.len(), 1);
    assert!(saved[0].closed);
    assert_eq!(saved[0].color, crate::config::StickyColor::Purple);
}

/// A sticky open at quit opens again as a sticky of its colour.
#[gpui_kit::test]
fn stickies_open_again_as_stickies(cx: &mut TestAppContext) {
    let f = setup(cx);
    cx.update(|cx| {
        AppState::update_config(cx, |config| {
            config.layout.note_windows = vec![NoteWindowFrame {
                note_id: f.note.clone(),
                x: 900.,
                y: 40.,
                width: 300.,
                height: 220.,
                sticky: true,
                color: crate::config::StickyColor::Blue,
                closed: false,
            }];
        });
        crate::note_window::restore(cx);
    });
    cx.run_until_parked();
    let windows = f.windows(cx);
    assert_eq!(windows.len(), 1);
    assert!(cx.read(|cx| windows[0].1.read(cx).is_sticky()));
    let saved = kept_frames(cx);
    assert_eq!(saved[0].color, crate::config::StickyColor::Blue);
    assert!(saved[0].sticky);
}

/// The note windows the config keeps now, saved or about to be.
fn kept_frames(cx: &mut TestAppContext) -> Vec<NoteWindowFrame> {
    cx.read(|cx| cx.global::<AppState>().config.layout.note_windows.clone())
}
