//! Snapshot states for Settings › AI: an API key, and an agent on this
//! computer found and ready, wanting a sign-in, not installed, failing to
//! start, and a custom command.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_acp::Detection;
use kanso_ai::AgentKind;
use kanso_store::Store;

use crate::config::AiBackend;
use crate::settings::SettingsPage;
use crate::settings_ai::Test;
use crate::workspace::{AppState, Workspace};

/// Every known agent: Claude Agent and OpenCode found, the rest not.
fn detections() -> Vec<(AgentKind, Detection)> {
    vec![
        (
            AgentKind::ClaudeAgent,
            Detection::Found("/opt/homebrew/bin/claude-agent-acp".into()),
        ),
        (AgentKind::Codex, Detection::NotInstalled),
        (AgentKind::GeminiCli, Detection::NotInstalled),
        (
            AgentKind::OpenCode,
            Detection::Found("/Users/kanso/.opencode/bin/opencode".into()),
        ),
    ]
}

fn settings_ai(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    backend: AiBackend,
    kind: AgentKind,
    test: Test,
) {
    cx.update(|cx| {
        let ai = &mut cx.global_mut::<AppState>().config.ai;
        ai.enabled = true;
        ai.backend = backend;
        ai.agent.kind = kind;
        if kind == AgentKind::Custom {
            ai.agent.command = Some("/usr/local/bin/my-agent".into());
            ai.agent.args = vec!["--acp".into()];
        }
    });
    super::show_settings(cx, window, workspace, SettingsPage::Ai);
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            let settings = workspace.settings.read(cx);
            settings.ai.clone().update(cx, |page, cx| {
                page.snapshot_agents(detections(), test);
                cx.notify();
            });
        });
    });
}

pub(super) fn settings_ai_api(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Api,
        AgentKind::ClaudeAgent,
        Test::NotRun,
    );
}

pub(super) fn settings_ai_agent_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::ClaudeAgent,
        Test::Ok(
            "Ready · Claude Agent 0.94.0 said “OK”. Billing is with the agent's provider, not Kanso."
                .into(),
        ),
    );
}

pub(super) fn settings_ai_agent_sign_in(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::OpenCode,
        Test::SignIn(vec![("opencode-login".into(), "Log in".into())]),
    );
}

pub(super) fn settings_ai_agent_not_installed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::Codex,
        Test::NotRun,
    );
}

pub(super) fn settings_ai_agent_failed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::ClaudeAgent,
        Test::Failed(
            "Claude Agent didn't finish starting. Its log may say why. It said: Error: Node 18 is not supported."
                .into(),
        ),
    );
}

pub(super) fn settings_ai_agent_custom(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::Custom,
        Test::NotRun,
    );
}
