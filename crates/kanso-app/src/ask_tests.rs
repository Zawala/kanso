//! Ask Your Notes in a headless window: the command is there only while AI
//! is on, and it opens the panel beside the note. Nothing here asks a model.

use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::{AppConfig, InspectorTab};
use crate::palette::Mode;
use crate::services::Services;
use crate::workspace::{AppState, AskYourNotes, SummarizeNote, Workspace};

struct Fixture {
    store: Arc<Store>,
    pricing: String,
    _dir: tempfile::TempDir,
    window: AnyWindowHandle,
    workspace: Entity<Workspace>,
}

fn setup(cx: &mut TestAppContext, ai: bool) -> Fixture {
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
    let pricing = store
        .put(
            DocType::Note,
            json!({"title": "Pricing", "body": "The yearly plan costs 40 dollars.",
                "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": "Secret roadmap", "body": "The zebra codename is Orchid.",
                "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let held = store.clone();
    let mut config = AppConfig::default();
    config.ai.enabled = ai;
    let data_dir = dir.path().to_path_buf();
    let (window, workspace) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        crate::workspace::bind_keys(cx);
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Fixture {
        store: held,
        pricing,
        _dir: dir,
        window,
        workspace,
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

    fn palette_labels(&self, cx: &mut TestAppContext) -> Vec<String> {
        let palette = cx.read(|cx| self.workspace.read(cx).palette.clone());
        self.window
            .update(cx, |_, window, cx| {
                palette.update(cx, |palette, cx| palette.open(Mode::Commands, window, cx));
            })
            .unwrap();
        self.draw(cx);
        cx.read(|cx| palette.read(cx).item_labels())
    }

    fn open_ask(&self, cx: &mut TestAppContext) {
        self.draw(cx);
        self.window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(AskYourNotes), cx)
            })
            .unwrap();
        cx.run_until_parked();
        self.draw(cx);
    }
}

#[gpui_kit::test]
fn ai_off_has_no_ask_command_and_opens_nothing(cx: &mut TestAppContext) {
    let f = setup(cx, false);
    let labels = f.palette_labels(cx);
    assert!(
        !labels.iter().any(|label| label.contains("Ask Your Notes")),
        "{labels:?}"
    );
    // The command palette is still open; close it before the action.
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| palette.close(window, cx));
        })
        .unwrap();
    f.open_ask(cx);
    let (open, tab) = cx.read(|cx| {
        let workspace = f.workspace.read(cx);
        (workspace.inspector_open, workspace.inspector.read(cx).tab)
    });
    assert!(!open);
    assert_ne!(tab, InspectorTab::Ask);
    let tabs = cx.read(|cx| {
        f.workspace
            .read(cx)
            .inspector
            .read(cx)
            .visible_tabs_for_test(cx)
    });
    assert!(!tabs.contains(&InspectorTab::Ask), "{tabs:?}");
}

#[gpui_kit::test]
fn ai_on_has_the_command_and_it_opens_the_panel(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let labels = f.palette_labels(cx);
    assert!(
        labels.iter().any(|label| label == "Ask Your Notes…"),
        "{labels:?}"
    );
    f.open_ask(cx);
    let (open, tab) = cx.read(|cx| {
        let workspace = f.workspace.read(cx);
        (workspace.inspector_open, workspace.inspector.read(cx).tab)
    });
    assert!(open);
    assert_eq!(tab, InspectorTab::Ask);
    let tabs = cx.read(|cx| {
        f.workspace
            .read(cx)
            .inspector
            .read(cx)
            .visible_tabs_for_test(cx)
    });
    assert!(tabs.contains(&InspectorTab::Ask), "{tabs:?}");
}

#[gpui_kit::test]
fn a_closed_stream_settles_the_panel(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let ask = cx.read(|cx| f.workspace.read(cx).inspector.read(cx).ask_panel_for_test());
    ask.update(cx, |ask, _| {
        ask.mark_running_for_test();
        let (settled, cut_off) = ask.settle_closed_for_test();
        assert!(settled);
        assert!(cut_off);
        // A second close (or a Done that already settled it) changes nothing.
        assert_eq!(ask.settle_closed_for_test(), (false, true));
    });
}

// Note summary: the Ask panel's "Summarize this note", and the palette
// command that runs it. Both backends are fakes (a recording provider, the
// in-process ACP agent); nothing leaves the process.

use crate::ai_feature::testing::{Injected, Recorder, fake_agent, maker_of};
use crate::ai_feature::{KEPT_OUT, Maker};
use crate::config::InspectorTab as Tab;

impl Fixture {
    /// The note "Pricing" open in the editor, and `maker` as the backend.
    fn with_backend(&self, cx: &mut TestAppContext, maker: Maker) {
        // The worker thread wakes the panel's task from outside the scheduler.
        cx.executor().allow_parking();
        cx.update(|cx| cx.set_global(Injected(maker)));
        let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
        let id = self.pricing.clone();
        self.window
            .update(cx, |_, window, cx| {
                editor.update(cx, |editor, cx| editor.open(Some(&id), window, cx))
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn summarize(&self, cx: &mut TestAppContext) {
        self.draw(cx);
        self.window
            .update(cx, |_, window, cx| {
                window.dispatch_action(Box::new(SummarizeNote), cx)
            })
            .unwrap();
        cx.run_until_parked();
    }

    fn ask_panel(&self, cx: &mut TestAppContext) -> Entity<crate::ask_panel::AskPanel> {
        cx.read(|cx| {
            self.workspace
                .read(cx)
                .inspector
                .read(cx)
                .ask_panel_for_test()
        })
    }

    /// Waits for the summary worker to finish.
    fn finish(&self, cx: &mut TestAppContext) -> (bool, Option<String>, Option<String>) {
        let ask = self.ask_panel(cx);
        for _ in 0..1000 {
            cx.run_until_parked();
            let state = ask.read_with(cx, |ask, _| ask.state_for_test());
            if !state.0 {
                return state;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the summary never finished");
    }

    fn turns(&self, cx: &mut TestAppContext) -> Vec<(String, String)> {
        let ask = self.ask_panel(cx);
        ask.read_with(cx, |ask, _| ask.turns_for_test())
    }
}

/// A summary on a model with an API key is a turn of the Ask panel,
/// `Summary of “Pricing”`; the request holds that one note, and no tools.
// no-prop-test: drives the window over a fake provider; `features` has the
// properties for the prompts.
#[gpui_kit::test]
fn a_summary_on_the_api_path_is_a_turn_of_the_panel(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let recorder = Recorder::new("- The yearly plan costs 40 dollars.");
    f.with_backend(cx, recorder.maker());
    f.summarize(cx);
    let (running, error, _) = f.finish(cx);
    assert!(!running);
    assert_eq!(error, None);
    assert_eq!(
        f.turns(cx),
        [(
            "Summary of \u{201c}Pricing\u{201d}".to_owned(),
            "- The yearly plan costs 40 dollars.".to_owned()
        )]
    );
    let tab = cx.read(|cx| f.workspace.read(cx).inspector.read(cx).tab);
    assert_eq!(tab, Tab::Ask);

    let sent = recorder.sent();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].user.contains("The yearly plan costs 40 dollars."));
    let all = format!("{} {}", sent[0].system, sent[0].user);
    assert!(!all.contains("zebra") && !all.contains("Orchid"), "{all}");
    assert_eq!(*recorder.tool_asks.lock().unwrap(), 0);
}

/// The same on an agent, whose session names no notes server.
// no-prop-test: drives the window over the in-process fake agent.
#[gpui_kit::test]
fn a_summary_on_an_agent_is_a_turn_of_the_panel(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let agent = fake_agent(
        kanso_acp::fake::Script::Answer("A yearly plan.".into()),
        "summary-answer",
    );
    f.with_backend(cx, maker_of(agent));
    f.summarize(cx);
    f.finish(cx);
    assert_eq!(
        f.turns(cx),
        [(
            "Summary of \u{201c}Pricing\u{201d}".to_owned(),
            "A yearly plan.".to_owned()
        )]
    );

    // What the agent was sent: this note, not the other one.
    let echo = fake_agent(kanso_acp::fake::Script::Echo, "summary-echo");
    f.with_backend(cx, maker_of(echo));
    f.summarize(cx);
    f.finish(cx);
    let turns = f.turns(cx);
    let sent = &turns.last().unwrap().1;
    assert!(sent.contains("The yearly plan costs 40 dollars."), "{sent}");
    assert!(
        !sent.contains("zebra") && !sent.contains("Orchid"),
        "{sent}"
    );

    let notes = fake_agent(kanso_acp::fake::Script::Notes, "summary-no-servers");
    f.with_backend(cx, maker_of(notes));
    f.summarize(cx);
    f.finish(cx);
    assert!(f.turns(cx).last().unwrap().1.contains("no notes server"));
}

/// A note kept out of AI (tagged, or locked) is not summarised: the panel
/// says so and nothing is sent.
// no-prop-test: drives the window over a fake provider.
#[gpui_kit::test]
fn a_note_kept_out_of_ai_is_not_summarised(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let recorder = Recorder::new("x");
    f.with_backend(cx, recorder.maker());
    let tag = f
        .store
        .put(
            DocType::Tag,
            json!({"name": "ai-exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.pricing, "tags": [tag]}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    f.summarize(cx);
    let (running, error, _) = f.finish(cx);
    assert!(!running);
    assert_eq!(error.as_deref(), Some(KEPT_OUT));
    assert!(f.turns(cx).is_empty());
    assert!(recorder.sent().is_empty());

    let f = setup(cx, true);
    let recorder = Recorder::new("x");
    f.with_backend(cx, recorder.maker());
    f.store
        .put(
            DocType::Note,
            json!({"_id": f.pricing, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    f.summarize(cx);
    let (_, error, _) = f.finish(cx);
    assert_eq!(error.as_deref(), Some(KEPT_OUT));
    assert!(recorder.sent().is_empty());
}

/// Stop ends a summary in progress.
// no-prop-test: drives the window over the in-process fake agent.
#[gpui_kit::test]
fn stop_ends_a_summary(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let hang = fake_agent(kanso_acp::fake::Script::Hang, "summary-hang");
    f.with_backend(cx, maker_of(hang));
    f.summarize(cx);
    let ask = f.ask_panel(cx);
    assert!(ask.read_with(cx, |ask, _| ask.state_for_test().0));
    ask.update(cx, |ask, cx| ask.stop(cx));
    assert!(!ask.read_with(cx, |ask, _| ask.state_for_test().0));
    assert!(f.turns(cx).is_empty());
}

/// The commands are in the palette while AI is on and a note is open, and
/// gone with AI off, where the action does nothing.
// no-prop-test: drives the window.
#[gpui_kit::test]
fn the_summary_and_inline_commands_follow_the_ai_setting(cx: &mut TestAppContext) {
    let wanted = [
        "Summarize Note",
        "Improve Writing",
        "Make Shorter",
        "Fix Spelling and Grammar",
        "Continue Writing",
        "Summarize Selection",
    ];
    let f = setup(cx, true);
    f.with_backend(cx, Recorder::new("x").maker());
    let labels = f.palette_labels(cx);
    for want in wanted {
        assert!(
            labels.iter().any(|label| label == want),
            "{want}: {labels:?}"
        );
    }

    let f = setup(cx, false);
    f.with_backend(cx, Recorder::new("x").maker());
    let labels = f.palette_labels(cx);
    for want in wanted {
        assert!(
            !labels.iter().any(|label| label == want),
            "{want}: {labels:?}"
        );
    }
    let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
    f.window
        .update(cx, |_, window, cx| {
            palette.update(cx, |palette, cx| palette.close(window, cx));
        })
        .unwrap();
    f.summarize(cx);
    assert!(f.turns(cx).is_empty());
    assert!(!cx.read(|cx| f.workspace.read(cx).inspector_open));
}
