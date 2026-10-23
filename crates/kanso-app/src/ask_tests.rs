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
use crate::workspace::{AppState, AskYourNotes, Workspace};

struct Fixture {
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
    store
        .put(
            DocType::Note,
            json!({"title": "Pricing", "body": "The yearly plan costs 40 dollars.",
                "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
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
