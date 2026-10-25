//! Snapshot states for Settings › AI: an API key, and an agent on this
//! computer found and ready, wanting a sign-in, not installed, failing to
//! start, a custom command, an agent from the ACP registry that isn't
//! installed, and the sheet that picks the notebooks an agent may read.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_acp::Detection;
use kanso_ai::AgentKind;
use kanso_store::Store;

use crate::config::AiBackend;
use crate::settings::SettingsPage;
use crate::settings_ai::{Install, Test};
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

/// Copilot picked from the ACP registry, not installed yet: the command it
/// would be installed with, and Install….
pub(super) fn settings_ai_agent_registry(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    let agents = kanso_acp::registry::parse(
        r#"{"agents": [{"id": "github-copilot-cli", "name": "Copilot", "version": "1.0.103",
            "description": "GitHub's AI pair programmer",
            "distribution": {"npx": {"package": "@github/copilot@1.0.103", "args": ["--acp"]}}}]}"#,
    )
    .unwrap();
    cx.update(|cx| {
        let ai = &mut cx.global_mut::<AppState>().config.ai;
        ai.enabled = true;
        ai.backend = AiBackend::Agent;
        ai.agent.kind = AgentKind::Custom;
        ai.agent.registry = Some("github-copilot-cli".into());
        ai.agent.command = None;
        ai.agent.args = vec!["--acp".into()];
    });
    super::show_settings(cx, window, workspace, SettingsPage::Ai);
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            let settings = workspace.settings.read(cx);
            settings.ai.clone().update(cx, |page, cx| {
                page.snapshot_agents(detections(), Test::NotRun);
                page.snapshot_registry(agents.clone(), Install::Idle);
                cx.notify();
            });
        });
    });
}

/// Notes Choose… clicked: the agent may read Work and the notebooks inside
/// it. The locked notebook isn't offered.
pub(super) fn settings_ai_agent_scope(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    let put = |fields: serde_json::Value| {
        store
            .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    };
    let work = put(serde_json::json!({ "name": "Work" }));
    put(serde_json::json!({ "name": "Clients", "parentBookId": work }));
    put(serde_json::json!({ "name": "Personal vault", "kansoLock": { "salt": "abc" } }));
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.ai.agent.notebooks = vec![work.clone()];
    });
    settings_ai(
        cx,
        window,
        workspace,
        AiBackend::Agent,
        AgentKind::ClaudeAgent,
        Test::NotRun,
    );
    cx.update_window(window, |_, window, cx| {
        crate::agent_scope::open_sheet(window, cx);
    })
    .unwrap();
}

/// The inline assistant's card above the text: a rewrite, ready to accept.
pub(super) fn editor_inline_suggestion(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let editor = workspace.read(cx).editor.clone();
        editor.update(cx, |editor, cx| {
            editor.show_for_snapshot(
                kanso_ai::InlineAction::Improve,
                "Kanso acts as an MCP host. Clients connect to it, discover the tools it offers, and ask before they act.\n\nSee the protocol overview for the wire format.",
                cx,
            )
        });
    });
}
