//! An agent on this computer as the source of AI answers: what the settings
//! mean for `kanso-ai`, and the plain lines Settings and the Ask panel show
//! about what the agent sees and who it sends questions to.

use std::path::{Path, PathBuf};

use kanso_ai::AgentKind;
use kanso_ai::agent::AgentSettings;

use crate::config::AgentChoice;

/// The empty private folder agents start in, inside Kanso's data folder.
pub fn workspace(data_dir: &Path) -> PathBuf {
    data_dir.join("agents").join("workspace")
}

/// The agent `choice` names, as `kanso-ai` runs it.
pub fn agent_settings(choice: &AgentChoice, data_dir: &Path) -> AgentSettings {
    AgentSettings {
        kind: choice.kind,
        command: choice.command.clone(),
        args: choice.args.clone(),
        env: choice
            .env
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        workspace: workspace(data_dir),
    }
}

/// What the person calls the chosen agent.
pub fn label(choice: &AgentChoice) -> String {
    agent_settings(choice, Path::new("")).label()
}

/// Where a question goes when an agent answers. Settings and the Ask panel
/// show the same line; it never says notes stay on this computer.
pub fn requests_line(choice: &AgentChoice) -> String {
    let label = label(choice);
    let vendor = choice.kind.vendor();
    format!(
        "Questions and the note excerpts they use go to {label} on this computer, which sends them to {vendor} under your own account. Kanso doesn't proxy or keep them. Locked notes and notes tagged #ai-exclude are never sent. {label} may keep its own history of the conversation."
    )
}

/// The trust line: what Kanso refuses, and what it can't control.
pub fn trust_line(choice: &AgentChoice) -> String {
    let label = label(choice);
    format!(
        "{label} runs as a program on this computer, as you. Kanso gives it no access to files or a terminal and refuses its requests for them, but it can still read files and run commands with its own tools. Notes are data, not instructions: Kanso asks the agent to ignore instructions inside them, but a note can still steer an answer. Check the citations."
    )
}

/// A one-line status for a known agent from its detection. How to install
/// it is the Install row's.
pub fn status_line(kind: AgentKind, detection: &kanso_acp::Detection) -> String {
    use kanso_acp::Detection;
    match detection {
        Detection::Found(path) => format!(
            "Found at {}. Test checks that it's signed in.",
            path.display()
        ),
        Detection::NotInstalled => {
            let node = kind
                .node()
                .map(|major| format!(" It needs Node {major} or newer."))
                .unwrap_or_default();
            format!("Not installed on this computer.{node}")
        }
        Detection::OldClaudeAdapter(_) => format!(
            "Only the old Claude Code adapter is installed, which Kanso doesn't start. Install the new one with: {}",
            kind.install().unwrap_or_default()
        ),
    }
}

/// A short status for a menu: Found, or Not installed.
pub fn status_word(detection: &kanso_acp::Detection) -> &'static str {
    match detection {
        kanso_acp::Detection::Found(_) => "Found",
        kanso_acp::Detection::NotInstalled => "Not installed",
        kanso_acp::Detection::OldClaudeAdapter(_) => "Update needed",
    }
}

#[cfg(test)]
mod tests {
    use super::{agent_settings, label, requests_line, status_line, trust_line, workspace};
    use crate::config::AgentChoice;
    use kanso_acp::Detection;
    use kanso_ai::AgentKind;
    use proptest::prelude::*;
    use std::path::Path;

    #[test]
    fn the_workspace_is_inside_the_data_folder_and_not_the_notes() {
        let dir = Path::new("/data/Kanso");
        assert_eq!(workspace(dir), dir.join("agents").join("workspace"));
        let settings = agent_settings(&AgentChoice::default(), dir);
        assert_eq!(settings.workspace, workspace(dir));
        assert_eq!(settings.kind, AgentKind::ClaudeAgent);
    }

    #[test]
    fn a_missing_agent_says_what_it_needs() {
        let line = status_line(AgentKind::ClaudeAgent, &Detection::NotInstalled);
        assert!(line.starts_with("Not installed"), "{line}");
        assert!(line.contains("Node 22"), "{line}");
        let line = status_line(
            AgentKind::ClaudeAgent,
            &Detection::OldClaudeAdapter("/usr/local/bin/claude-code-acp".into()),
        );
        assert!(line.contains("old Claude Code adapter"), "{line}");
    }

    fn gen_choice() -> impl Strategy<Value = AgentChoice> {
        (
            prop::sample::select(AgentKind::ALL.to_vec()),
            prop::option::of("[a-z/-]{1,20}"),
        )
            .prop_map(|(kind, command)| AgentChoice {
                kind,
                command,
                ..AgentChoice::default()
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Whatever the agent, the lines name it and who it sends to, say
        /// locked and excluded notes are never sent, never promise the notes
        /// stay on this computer, and say Kanso doesn't sandbox it.
        #[test]
        fn prop_the_agent_lines_never_overpromise(choice in gen_choice()) {
            let line = requests_line(&choice);
            let label = label(&choice);
            prop_assert!(line.contains(&label));
            prop_assert!(line.contains(choice.kind.vendor()));
            prop_assert!(line.contains("#ai-exclude"));
            prop_assert!(line.contains("Locked notes"));
            prop_assert!(!line.to_lowercase().contains("never leave"));
            prop_assert!(!line.contains('\n'));
            let trust = trust_line(&choice);
            prop_assert!(trust.contains("its own tools"));
            prop_assert!(trust.contains("as you"));
        }
    }
}
