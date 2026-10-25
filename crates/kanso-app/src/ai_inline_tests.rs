//! The inline assistant through the real editor, on both backends: a model
//! with an API key (a fake provider) and an agent on this computer (the
//! fake ACP agent). Plus the properties of applying a suggestion.

use std::ops::Range;
use std::time::Duration;

use gpui_kit::TestAppContext;
use kanso_ai::InlineAction;
use kanso_core::DocType;
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use super::inline_glue::{CHANGED, paragraph_before, splice};
use super::writing_tests::Open;
use crate::ai_feature::Maker;
use crate::ai_feature::testing::{Injected, Recorder, fake_agent, maker_of};
use crate::config::AppConfig;
use crate::workspace::AppState;

const BODY: &str = "We ship on friday.\n\nThe launch needs a banner.";
const SENTENCE: Range<usize> = 0..18;

/// A note open with `body`, AI on, and `maker` as the backend.
fn open(cx: &mut TestAppContext, body: &str, ai: bool, maker: Maker) -> Open {
    // The worker thread wakes the card's task from outside the scheduler.
    cx.executor().allow_parking();
    let open = Open::new(cx, body);
    let store = open.store.clone();
    cx.update(|cx| {
        let mut config = AppConfig::default();
        config.ai.enabled = ai;
        cx.set_global(AppState {
            store,
            data_dir: std::env::temp_dir(),
            config,
        });
        cx.set_global(Injected(maker));
    });
    open
}

fn start(open: &Open, cx: &mut TestAppContext, action: InlineAction) {
    open.with(cx, |editor, _, cx| editor.start_inline(action, cx));
}

/// Waits for the worker thread and the card to settle. The suggestion's
/// text and notice are returned, or `None` when there is no card.
fn settle(open: &Open, cx: &mut TestAppContext) -> Option<(String, Option<String>)> {
    for _ in 0..1000 {
        let state = open.with(cx, |editor, _, _| editor.suggestion_for_test());
        match state {
            Some((_, true, _)) => std::thread::sleep(Duration::from_millis(10)),
            Some((text, false, notice)) => return Some((text, notice)),
            None => return None,
        }
    }
    panic!("the suggestion never settled");
}

fn accept(open: &Open, cx: &mut TestAppContext) {
    open.with(cx, |editor, window, cx| editor.accept_inline(window, cx));
}

/// Another note in the library, whose words must never reach a feature.
fn other_note(open: &Open) {
    let book = open
        .store
        .put(
            DocType::Book,
            json!({"name": "Other"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    open.store
        .put(
            DocType::Note,
            json!({"title": "Secret roadmap", "body": "The zebra codename is Orchid.",
                "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

/// Improve on a model with an API key: the card streams in, Accept
/// replaces the selection in one undo step, and the request carried the
/// selection and no tools and no other note.
// no-prop-test: drives the editor; `splice` has the properties.
#[gpui_kit::test]
fn improve_on_the_api_path_replaces_the_selection_in_one_undo_step(cx: &mut TestAppContext) {
    let recorder = Recorder::new("We ship on Friday.");
    let open = open(cx, BODY, true, recorder.maker());
    other_note(&open);
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Improve);
    let (text, _) = settle(&open, cx).expect("a suggestion");
    assert_eq!(text, "We ship on Friday.");
    // Nothing changes until Accept.
    assert_eq!(open.text(cx).0, BODY);

    accept(&open, cx);
    assert_eq!(
        open.text(cx).0,
        "We ship on Friday.\n\nThe launch needs a banner."
    );
    assert!(settle(&open, cx).is_none(), "the card is gone");
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, BODY);

    let sent = recorder.sent();
    assert_eq!(sent.len(), 1);
    let all = format!("{} {}", sent[0].system, sent[0].user);
    assert!(all.contains("We ship on friday."), "{all}");
    assert!(!all.contains("zebra") && !all.contains("Orchid"), "{all}");
    assert_eq!(*recorder.tool_asks.lock().unwrap(), 0);
}

/// The same on an agent: the fake agent answers, Accept applies it, and
/// what it was sent holds the selection and not the other note.
// no-prop-test: drives the editor; `splice` has the properties.
#[gpui_kit::test]
fn improve_on_an_agent_replaces_the_selection(cx: &mut TestAppContext) {
    let agent = fake_agent(
        kanso_acp::fake::Script::Answer("We ship on Friday.".into()),
        "inline-improve",
    );
    let open = open(cx, BODY, true, maker_of(agent));
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Improve);
    let (text, _) = settle(&open, cx).expect("a suggestion");
    assert_eq!(text, "We ship on Friday.");
    accept(&open, cx);
    assert_eq!(
        open.text(cx).0,
        "We ship on Friday.\n\nThe launch needs a banner."
    );
}

/// What an agent is sent for a selection is the instruction and the
/// selection with a little of the note before it: not another note, and
/// the session names no notes server.
// no-prop-test: drives the editor over the fake agent.
#[gpui_kit::test]
fn an_agent_feature_reads_only_what_the_prompt_carries(cx: &mut TestAppContext) {
    let agent = fake_agent(kanso_acp::fake::Script::Echo, "inline-echo");
    let open = open(cx, BODY, true, maker_of(agent));
    other_note(&open);
    open.select(cx, 20..46);
    start(&open, cx, InlineAction::Shorten);
    let (sent, _) = settle(&open, cx).expect("a suggestion");
    assert!(sent.contains("The launch needs a banner."), "{sent}");
    assert!(sent.contains("We ship on friday."), "context before it");
    assert!(
        !sent.contains("zebra") && !sent.contains("Orchid"),
        "{sent}"
    );
    assert!(!sent.contains("Secret roadmap"), "{sent}");

    let notes = fake_agent(kanso_acp::fake::Script::Notes, "inline-no-servers");
    let open = self::open(cx, BODY, true, maker_of(notes));
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Improve);
    let (said, _) = settle(&open, cx).expect("a suggestion");
    assert!(said.contains("no notes server"), "{said}");
}

/// Continue and Summarize go after the selection, as a paragraph.
// no-prop-test: drives the editor; `splice` has the properties.
#[gpui_kit::test]
fn continue_goes_after_the_selection(cx: &mut TestAppContext) {
    let open = open(cx, BODY, true, Recorder::new("Then we party.").maker());
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Continue);
    settle(&open, cx).expect("a suggestion");
    accept(&open, cx);
    assert_eq!(
        open.text(cx).0,
        "We ship on friday.\n\nThen we party.\n\nThe launch needs a banner."
    );
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, BODY);

    // With nothing selected, Continue goes on from the paragraph before the
    // caret; the other actions want a selection.
    let recorder = Recorder::new("More.");
    let open = self::open(cx, BODY, true, recorder.maker());
    open.select(cx, 18..18);
    start(&open, cx, InlineAction::Improve);
    assert!(
        open.with(cx, |editor, _, _| editor.suggestion_for_test())
            .is_none()
    );
    assert!(recorder.sent().is_empty());
    start(&open, cx, InlineAction::Continue);
    settle(&open, cx).expect("a suggestion");
    let sent = recorder.sent();
    assert!(
        sent[0].user.contains("We ship on friday."),
        "{}",
        sent[0].user
    );
    accept(&open, cx);
    assert_eq!(
        open.text(cx).0,
        "We ship on friday.\n\nMore.\n\nThe launch needs a banner."
    );
}

/// Accept applies once, and only to the text it was made for: when the
/// text moved on, the card says so and stays.
// no-prop-test: drives the editor; `splice` has the properties.
#[gpui_kit::test]
fn accept_applies_once_and_only_when_the_text_is_unchanged(cx: &mut TestAppContext) {
    let open = open(cx, BODY, true, Recorder::new("We ship Friday.").maker());
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Improve);
    settle(&open, cx).expect("a suggestion");

    // The sentence is edited meanwhile.
    open.select(cx, 0..0);
    open.input(cx, ">> ");
    let edited = open.text(cx).0;
    assert!(edited.starts_with(">> We ship"));
    accept(&open, cx);
    assert_eq!(open.text(cx).0, edited, "nothing was applied");
    let (_, _, notice) = open
        .with(cx, |editor, _, _| editor.suggestion_for_test())
        .expect("the card stays");
    assert_eq!(notice.as_deref(), Some(CHANGED));

    // Put back, it applies; a second Accept finds no card and does nothing.
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, BODY);
    accept(&open, cx);
    let once = open.text(cx).0;
    assert_eq!(once, "We ship Friday.\n\nThe launch needs a banner.");
    accept(&open, cx);
    assert_eq!(open.text(cx).0, once);
}

/// Discard, Escape, opening another note and locking drop the card and
/// stop its request.
// no-prop-test: drives the editor.
#[gpui_kit::test]
fn discard_escape_and_leaving_the_note_drop_the_suggestion(cx: &mut TestAppContext) {
    let open = open(cx, BODY, true, Recorder::new("Better.").maker());
    open.select(cx, SENTENCE);
    let card = |open: &Open, cx: &mut TestAppContext| {
        open.with(cx, |editor, _, _| editor.suggestion_for_test().is_some())
    };

    start(&open, cx, InlineAction::Improve);
    settle(&open, cx);
    open.with(cx, |editor, _, cx| editor.discard_inline(cx));
    assert!(!card(&open, cx));
    assert_eq!(open.text(cx).0, BODY);

    start(&open, cx, InlineAction::Improve);
    settle(&open, cx);
    open.keys(cx, "escape");
    assert!(!card(&open, cx), "Escape drops it");

    start(&open, cx, InlineAction::Improve);
    settle(&open, cx);
    let book = open
        .store
        .put(DocType::Book, json!({"name": "C"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let next = open
        .store
        .put(
            DocType::Note,
            json!({"title": "Next", "body": "Another.", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    open.with(cx, |editor, window, cx| {
        editor.open(Some(&next), window, cx)
    });
    assert!(!card(&open, cx), "another note drops it");
}

/// Stop ends the request and keeps what arrived.
// no-prop-test: drives the editor over the fake agent.
#[gpui_kit::test]
fn stop_ends_the_request(cx: &mut TestAppContext) {
    let agent = fake_agent(kanso_acp::fake::Script::Hang, "inline-hang");
    let open = open(cx, BODY, true, maker_of(agent));
    open.select(cx, SENTENCE);
    start(&open, cx, InlineAction::Improve);
    let (_, running, _) = open
        .with(cx, |editor, _, _| editor.suggestion_for_test())
        .unwrap();
    assert!(running);
    open.with(cx, |editor, _, cx| editor.stop_inline(cx));
    let (_, running, notice) = open
        .with(cx, |editor, _, _| editor.suggestion_for_test())
        .unwrap();
    assert!(!running);
    assert_eq!(notice.as_deref(), Some("Stopped."));
}

/// A note kept out of AI, locked or with AI off sends nothing and gets no
/// card and no menu items.
// no-prop-test: drives the editor.
#[gpui_kit::test]
fn a_note_kept_out_of_ai_sends_nothing(cx: &mut TestAppContext) {
    // Tagged #ai-exclude.
    let recorder = Recorder::new("x");
    let open = open(cx, BODY, true, recorder.maker());
    open.select(cx, SENTENCE);
    assert_eq!(menu(&open, cx).len(), 5);
    let tag = open
        .store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    open.store
        .put(
            DocType::Note,
            json!({"_id": open.id, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(menu(&open, cx).is_empty());
    start(&open, cx, InlineAction::Improve);
    assert!(
        open.with(cx, |editor, _, _| editor.suggestion_for_test())
            .is_none()
    );
    assert!(recorder.sent().is_empty());

    // Locked.
    let recorder = Recorder::new("x");
    let open = self::open(cx, BODY, true, recorder.maker());
    open.select(cx, SENTENCE);
    open.store
        .put(
            DocType::Note,
            json!({"_id": open.id, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    assert!(menu(&open, cx).is_empty());
    start(&open, cx, InlineAction::Improve);
    assert!(
        open.with(cx, |editor, _, _| editor.suggestion_for_test())
            .is_none()
    );
    assert!(recorder.sent().is_empty());

    // AI off.
    let recorder = Recorder::new("x");
    let open = self::open(cx, BODY, false, recorder.maker());
    open.select(cx, SENTENCE);
    assert!(menu(&open, cx).is_empty());
    start(&open, cx, InlineAction::Improve);
    assert!(
        open.with(cx, |editor, _, _| editor.suggestion_for_test())
            .is_none()
    );
    assert!(recorder.sent().is_empty());
}

/// The right-click menu's AI items: the five actions with a selection,
/// none without one.
fn menu(open: &Open, cx: &mut TestAppContext) -> Vec<&'static str> {
    open.with(cx, |editor, _, cx| {
        editor
            .inline_menu(cx)
            .into_iter()
            .map(|a| a.label())
            .collect()
    })
}

// no-prop-test: drives the editor.
#[gpui_kit::test]
fn the_menu_offers_the_five_actions_on_a_selection(cx: &mut TestAppContext) {
    let open = open(cx, BODY, true, Recorder::new("x").maker());
    open.select(cx, SENTENCE);
    assert_eq!(
        menu(&open, cx),
        [
            "Improve Writing",
            "Make Shorter",
            "Fix Spelling and Grammar",
            "Continue Writing",
            "Summarize Selection"
        ]
    );
    open.select(cx, 3..3);
    assert!(menu(&open, cx).is_empty());
}

/// The text between `a` and `b` (any order, counted in characters).
fn span(text: &str, a: usize, b: usize) -> Range<usize> {
    let at = |n: usize| {
        text.char_indices()
            .nth(n % (text.chars().count() + 1))
            .map_or(text.len(), |(at, _)| at)
    };
    let (a, b) = (at(a), at(b));
    a.min(b)..a.max(b)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Accepting changes nothing outside the range it was made for: what
    /// comes before is identical, and what comes after is identical (after
    /// the suggestion, or after the selection it follows). A rewrite leaves
    /// the selection's own outer blank space alone too.
    #[test]
    fn prop_accept_leaves_text_outside_the_range_identical(
        text in "[\\PC\n]{0,80}",
        a in 0usize..100,
        b in 0usize..100,
        suggestion in "[\\PC\n]{0,40}",
        action in prop::sample::select(InlineAction::ALL.to_vec()),
    ) {
        let range = span(&text, a, b);
        let original = text[range.clone()].to_owned();
        match splice(&text, &range, &original, action, &suggestion) {
            None => prop_assert!(suggestion.trim().is_empty()),
            Some(change) => {
                prop_assert!(!suggestion.trim().is_empty());
                let after = change.apply(&text);
                let put = suggestion.trim();
                if action.replaces() {
                    prop_assert_eq!(&after[..range.start], &text[..range.start]);
                    prop_assert!(after.ends_with(&text[range.end..]));
                    prop_assert_eq!(after.len(), text.len() - original.len() + change.with.len());
                    let inner = &after[range.start..after.len() - (text.len() - range.end)];
                    prop_assert!(inner.contains(put));
                    prop_assert_eq!(inner.trim(), put.trim());
                } else {
                    prop_assert_eq!(&after[..range.end], &text[..range.end]);
                    prop_assert!(after.ends_with(&text[range.end..]));
                    prop_assert_eq!(&after[range.end..after.len() - (text.len() - range.end)],
                        &format!("\n\n{put}"));
                }
            }
        }
    }

    /// A suggestion is never applied to text other than the text it was
    /// made for.
    #[test]
    fn prop_a_changed_selection_is_never_replaced(
        text in "[\\PC\n]{1,80}",
        a in 0usize..100,
        b in 0usize..100,
        other in "[\\PC\n]{0,20}",
        action in prop::sample::select(InlineAction::ALL.to_vec()),
    ) {
        let range = span(&text, a, b);
        prop_assume!(text[range.clone()] != other);
        prop_assert!(splice(&text, &range, &other, action, "A suggestion.").is_none());
    }

    /// The paragraph before the caret ends at the caret, starts after a blank
    /// line, and is non-blank or empty.
    #[test]
    fn prop_the_paragraph_before_the_caret_ends_at_it(
        text in "[\\PC\n]{0,80}",
        caret in 0usize..100,
    ) {
        let caret = span(&text, caret, caret).start;
        let range = paragraph_before(&text, caret);
        prop_assert_eq!(range.end, caret);
        prop_assert!(range.start <= range.end);
        let part = &text[range.clone()];
        prop_assert!(range.is_empty() || !part.trim().is_empty());
        prop_assert!(!part.contains("\n\n"));
    }
}
