//! The workspace in a headless window: autosave, switching notes, and
//! changes made elsewhere while a note is open. These are where an editor
//! loses data, so they run against the real views and store.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
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
