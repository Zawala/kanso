//! The permission sheet in a headless window: no window rejects, Enter and
//! Escape reject, only "Allow once" allows, a second request waits for the
//! first, and a request that ended behind its sheet closes it. The sheet's
//! choices are properties in `agent_permission.rs`; these check the window
//! wires them up, against the fake agent over the real connection.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Modifiers, TestAppContext, VisualTestContext};
use kanso_acp::fake::{self, Script};
use kanso_acp::{AcpEvent, AgentProcess, Limits, PermissionPolicy};

use crate::agent_permission;
use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    _dir: tempfile::TempDir,
    window: Option<AnyWindowHandle>,
    policy: PermissionPolicy,
}

/// A headless app with the sheet installed, with a main window or without.
fn setup(cx: &mut TestAppContext, with_window: bool) -> Fixture {
    // The agent's threads wake the sheet's task from outside the test.
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(kanso_store::Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    let (window, policy) = cx.update(|cx| {
        gpui_kit::init(cx);
        crate::workspace::bind_keys(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        let window = with_window.then(|| {
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            })
            .unwrap()
            .0
        });
        (window, agent_permission::start(cx))
    });
    if let Some(window) = window {
        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
    }
    cx.run_until_parked();
    Fixture {
        _dir: dir,
        window,
        policy,
    }
}

/// A fake agent asking `script`'s questions, and what its turn said.
struct Asking {
    _agent: AgentProcess,
    _session: kanso_acp::Session,
    said: Receiver<String>,
}

fn ask(script: Script, label: &str, limits: Limits, policy: &PermissionPolicy) -> Asking {
    let (from_agent, to_agent) = fake::in_process(script).unwrap();
    let agent = AgentProcess::connect(label, from_agent, to_agent, limits, policy.clone()).unwrap();
    let session = agent.new_session(&std::env::temp_dir()).unwrap();
    let turn = session.prompt("x", None).unwrap();
    let (tx, said) = channel();
    std::thread::spawn(move || {
        let text: String = turn
            .filter_map(|event| match event {
                AcpEvent::Text(text) => Some(text),
                _ => None,
            })
            .collect();
        tx.send(text).ok();
    });
    Asking {
        _agent: agent,
        _session: session,
        said,
    }
}

impl Asking {
    /// What the agent heard, once its turn is over.
    fn heard(&self, cx: &mut TestAppContext) -> String {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            cx.run_until_parked();
            if let Ok(text) = self.said.try_recv() {
                return text;
            }
            assert!(Instant::now() < deadline, "waited for the agent's answer");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn still_waiting(&self, cx: &mut TestAppContext) -> bool {
        for _ in 0..30 {
            cx.run_until_parked();
            if self.said.try_recv().is_ok() {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        true
    }
}

impl Fixture {
    fn window(&self) -> AnyWindowHandle {
        self.window.expect("the fixture has a window")
    }

    fn sheet_open(&self, cx: &mut TestAppContext) -> bool {
        self.window()
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    }

    fn draw(&self, cx: &mut TestAppContext) {
        self.window()
            .update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        cx.run_until_parked();
    }

    /// Waits for a sheet to be up.
    fn wait_for_sheet(&self, cx: &mut TestAppContext) {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            cx.run_until_parked();
            if self.sheet_open(cx) {
                self.draw(cx);
                return;
            }
            assert!(Instant::now() < deadline, "waited for the sheet");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window(), cx);
        let bounds = vcx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is drawn"));
        vcx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
    }

    fn shows(&self, cx: &mut TestAppContext, selector: &'static str) -> bool {
        self.draw(cx);
        let mut vcx = VisualTestContext::from_window(self.window(), cx);
        vcx.debug_bounds(selector).is_some()
    }

    fn press(&self, cx: &mut TestAppContext, keys: &str) {
        cx.simulate_keystrokes(self.window(), keys);
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn with_no_window_a_request_is_rejected(cx: &mut TestAppContext) {
    let f = setup(cx, false);
    let asking = ask(
        Script::Permission,
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    assert_eq!(asking.heard(cx), "permission=no");
}

#[gpui_kit::test]
fn the_sheet_shows_the_tool_and_its_kind(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let asking = ask(
        Script::Permission,
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    f.wait_for_sheet(cx);
    assert!(f.shows(cx, "permission-tool"));
    assert!(f.shows(cx, "permission-kind"));
    assert!(f.shows(cx, "permission-allow"));
    assert!(f.shows(cx, "permission-reject"));
    f.press(cx, "escape");
    assert_eq!(asking.heard(cx), "permission=no");
}

#[gpui_kit::test]
fn enter_rejects(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let asking = ask(
        Script::Permission,
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    f.wait_for_sheet(cx);
    f.press(cx, "enter");
    assert_eq!(asking.heard(cx), "permission=no");
    assert!(!f.sheet_open(cx));
}

#[gpui_kit::test]
fn escape_rejects(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let asking = ask(
        Script::Permission,
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    f.wait_for_sheet(cx);
    f.press(cx, "escape");
    assert_eq!(asking.heard(cx), "permission=no");
    assert!(!f.sheet_open(cx));
}

#[gpui_kit::test]
fn the_reject_button_rejects(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let asking = ask(
        Script::Permission,
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    f.wait_for_sheet(cx);
    f.click(cx, "permission-reject");
    assert_eq!(asking.heard(cx), "permission=no");
    assert!(!f.sheet_open(cx));
}

#[gpui_kit::test]
fn allow_once_allows_this_request_only(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    // Two questions in a row: allowing the first does not allow the second.
    let asking = ask(
        Script::Permissions(2),
        "Fake Agent",
        Limits::default(),
        &f.policy,
    );
    f.wait_for_sheet(cx);
    f.click(cx, "permission-allow");
    f.wait_for_sheet(cx);
    f.press(cx, "enter");
    assert_eq!(asking.heard(cx), "before permissions=yes,no");
}

#[gpui_kit::test]
fn a_second_request_waits_for_the_first(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let first = ask(Script::Permission, "Agent A", Limits::default(), &f.policy);
    f.wait_for_sheet(cx);
    let second = ask(Script::Permission, "Agent B", Limits::default(), &f.policy);
    assert!(second.still_waiting(cx));
    assert!(first.still_waiting(cx));
    f.click(cx, "permission-allow");
    assert_eq!(first.heard(cx), "permission=yes");
    // The second sheet takes its place; the first answer didn't reach it.
    f.wait_for_sheet(cx);
    assert!(second.still_waiting(cx));
    f.press(cx, "enter");
    assert_eq!(second.heard(cx), "permission=no");
}

#[gpui_kit::test]
fn a_request_that_times_out_behind_its_sheet_closes_it(cx: &mut TestAppContext) {
    let f = setup(cx, true);
    let limits = Limits {
        permission: Duration::from_millis(200),
        ..Limits::default()
    };
    let asking = ask(Script::Permission, "Fake Agent", limits, &f.policy);
    f.wait_for_sheet(cx);
    // The agent has been told no; the sheet notices on its next look.
    assert_eq!(asking.heard(cx), "permission=no");
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    assert!(!f.sheet_open(cx));
}
