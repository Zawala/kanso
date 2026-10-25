//! The review queue in a headless window: a suggestion opens the sheet,
//! the Ask panel counts it, Apply writes once as the agent's edit, a note
//! that changed refuses, and clearing the conversation keeps it.

use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_server::{AgentScope, NewChange, ReviewQueue, TokenStore};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::AppConfig;
use crate::review_queue::{self, ReviewSheet};
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
    queue: ReviewQueue,
    tokens: TokenStore,
    note: String,
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Notes"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let note = store
        .put(
            DocType::Note,
            json!({"title": "Pricing", "body": "The yearly plan costs 40 dollars.\n", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let mut config = AppConfig::default();
    config.ai.enabled = true;
    let data_dir = dir.path().to_path_buf();
    let services = Services::default();
    let tokens = services.agent_tokens.clone();
    let queue = tokens.reviews().clone();
    let shared = store.clone();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: shared,
            data_dir,
            config,
        });
        cx.set_global(services);
        crate::workspace::bind_keys(cx);
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
        queue,
        tokens,
        note,
    }
}

impl Fixture {
    fn draw(&self, cx: &mut TestAppContext) {
        self.window
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// An agent's suggestion to change the note's text.
    fn suggest(&self, body: &str) -> kanso_server::ChangeId {
        let note = self.store.note(&self.note).unwrap().unwrap();
        let minted = self.tokens.mint(AgentScope::ask(None));
        self.queue
            .stage(NewChange {
                note_id: Some(note.id.clone()),
                base_revision: note.rev.clone(),
                book_id: None,
                old_title: note.title.clone(),
                old_body: note.body.clone(),
                new_title: note.title.clone(),
                new_body: body.to_owned(),
                agent: "Claude Agent".to_owned(),
                token: minted.id,
                scope: Arc::new(AgentScope::ask_and_suggest(None, "Claude Agent")),
            })
            .unwrap()
    }

    fn body(&self) -> String {
        self.store.note(&self.note).unwrap().unwrap().body
    }

    fn sheet(&self, cx: &mut TestAppContext) -> Entity<ReviewSheet> {
        self.window
            .update(cx, |_, window, cx| review_queue::open_review(window, cx))
            .unwrap()
            .expect("the review sheet opens")
    }

    fn ask(&self, cx: &mut TestAppContext) -> Entity<crate::ask_panel::AskPanel> {
        cx.read(|cx| {
            self.workspace
                .read(cx)
                .inspector
                .read(cx)
                .ask_panel_for_test()
        })
    }
}

#[gpui_kit::test]
fn a_suggestion_opens_the_sheet_and_the_ask_panel_counts_it(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.draw(cx);
    assert!(!cx.read(review_queue::is_open));
    // A window in the background doesn't take the sheet; a suggestion waits
    // and the Ask panel counts it.
    f.suggest("The yearly plan costs 44 dollars.\n");
    f.draw(cx);
    assert!(!cx.read(review_queue::is_open));
    assert_eq!(
        f.ask(cx).read_with(cx, |ask, _| ask.suggested_for_test()),
        1
    );
    cx.update(|cx| cx.set_global(review_queue::AssumeFocused));
    f.suggest("The yearly plan costs 45 dollars.\n");
    f.draw(cx);
    // Nothing was written.
    assert_eq!(f.body(), "The yearly plan costs 40 dollars.\n");
    assert!(cx.read(review_queue::is_open), "the sheet opens");
    assert_eq!(
        f.ask(cx).read_with(cx, |ask, _| ask.suggested_for_test()),
        2
    );
}

#[gpui_kit::test]
fn apply_writes_once_and_a_second_click_says_nothing(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.suggest("The yearly plan costs 45 dollars.\n");
    let sheet = f.sheet(cx);
    f.draw(cx);
    sheet.update(cx, |sheet, cx| {
        sheet.apply(id, cx);
        sheet.apply(id, cx);
    });
    assert_eq!(f.body(), "The yearly plan costs 45 dollars.\n");
    let notice = sheet.read_with(cx, |sheet, _| sheet.notice_for_test());
    assert_eq!(notice.as_deref(), Some("Applied to “Pricing”."));
    let versions = f.store.revisions(&f.note).unwrap();
    assert_eq!(versions.iter().filter(|v| v.origin == "agent").count(), 1);
    assert_eq!(versions[0].agent.as_deref(), Some("Claude Agent"));
    assert!(f.queue.is_empty());
    f.draw(cx);
}

#[gpui_kit::test]
fn a_note_that_changed_refuses_and_shows_both_versions(cx: &mut TestAppContext) {
    let f = setup(cx);
    let id = f.suggest("The agent's text.\n");
    let sheet = f.sheet(cx);
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.note, "body": "The person's text.\n"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    sheet.update(cx, |sheet, cx| sheet.apply(id, cx));
    assert_eq!(f.body(), "The person's text.\n");
    assert!(sheet.read_with(cx, |sheet, _| sheet.is_conflict_for_test(id)));
    assert_eq!(f.queue.len(), 1, "it stays, to compare or discard");
    // Drawn with both versions on show.
    f.draw(cx);
    sheet.update(cx, |sheet, cx| sheet.discard(id, cx));
    assert!(f.queue.is_empty());
    assert_eq!(f.body(), "The person's text.\n");
}

#[gpui_kit::test]
fn clearing_the_conversation_keeps_the_suggestions(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.suggest("Something else.\n");
    f.draw(cx);
    let ask = f.ask(cx);
    ask.update(cx, |ask, cx| ask.end_conversation(cx));
    f.draw(cx);
    assert_eq!(f.queue.len(), 1);
    assert_eq!(ask.read_with(cx, |ask, _| ask.suggested_for_test()), 1);
}

#[gpui_kit::test]
fn locking_the_note_drops_the_suggestion(cx: &mut TestAppContext) {
    let f = setup(cx);
    f.suggest("Something else.\n");
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.note, "bookId": "trash"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    f.draw(cx);
    assert!(f.queue.is_empty(), "a trashed note can't take it");
    assert_eq!(
        f.ask(cx).read_with(cx, |ask, _| ask.suggested_for_test()),
        0
    );
}
