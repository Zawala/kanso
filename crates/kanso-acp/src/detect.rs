//! Which agents are on this computer: a table of the agents Kanso knows and
//! a search of the places they install to. Nothing here starts an agent.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The agents Settings › AI lists, and the person's own command.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentKind {
    #[default]
    ClaudeAgent,
    Codex,
    GeminiCli,
    OpenCode,
    /// Any other program that speaks ACP, at a command the person gives.
    Custom,
}

impl AgentKind {
    pub const ALL: [AgentKind; 5] = [
        AgentKind::ClaudeAgent,
        AgentKind::Codex,
        AgentKind::GeminiCli,
        AgentKind::OpenCode,
        AgentKind::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AgentKind::ClaudeAgent => "Claude Agent",
            AgentKind::Codex => "Codex",
            AgentKind::GeminiCli => "Gemini CLI",
            AgentKind::OpenCode => "OpenCode",
            AgentKind::Custom => "Another ACP agent",
        }
    }

    /// Who the agent sends questions to, for the line that says so.
    pub fn vendor(self) -> &'static str {
        match self {
            AgentKind::ClaudeAgent => "Anthropic",
            AgentKind::Codex => "OpenAI",
            AgentKind::GeminiCli => "Google",
            AgentKind::OpenCode => "the provider you set up in OpenCode",
            AgentKind::Custom => "whoever that agent uses",
        }
    }

    /// The command Kanso looks for.
    pub fn command(self) -> Option<&'static str> {
        match self {
            AgentKind::ClaudeAgent => Some("claude-agent-acp"),
            AgentKind::Codex => Some("codex-acp"),
            AgentKind::GeminiCli => Some("gemini"),
            AgentKind::OpenCode => Some("opencode"),
            AgentKind::Custom => None,
        }
    }

    /// The arguments that put the command in ACP mode.
    pub fn args(self) -> &'static [&'static str] {
        match self {
            AgentKind::GeminiCli => &["--acp"],
            AgentKind::OpenCode => &["acp"],
            _ => &[],
        }
    }

    /// Variables the agent documents for a safer start. Codex's adapter
    /// reads `INITIAL_AGENT_MODE`; the others pick a mode over ACP.
    pub fn env(self) -> &'static [(&'static str, &'static str)] {
        match self {
            AgentKind::Codex => &[("INITIAL_AGENT_MODE", "read-only")],
            _ => &[],
        }
    }

    /// Its npm package, for the agents Kanso can install with one click.
    pub fn npm_package(self) -> Option<&'static str> {
        match self {
            AgentKind::ClaudeAgent => Some("@agentclientprotocol/claude-agent-acp"),
            AgentKind::Codex => Some("@agentclientprotocol/codex-acp"),
            AgentKind::GeminiCli => Some("@google/gemini-cli"),
            AgentKind::OpenCode | AgentKind::Custom => None,
        }
    }

    /// How to install it, shown when it isn't found.
    pub fn install(self) -> Option<&'static str> {
        match self {
            AgentKind::ClaudeAgent => Some("npm install -g @agentclientprotocol/claude-agent-acp"),
            AgentKind::Codex => Some("npm install -g @agentclientprotocol/codex-acp"),
            AgentKind::GeminiCli => Some("npm install -g @google/gemini-cli"),
            AgentKind::OpenCode => Some("curl -fsSL https://opencode.ai/install | bash"),
            AgentKind::Custom => None,
        }
    }

    /// The Node.js it needs, if any.
    pub fn node(self) -> Option<u32> {
        match self {
            AgentKind::ClaudeAgent => Some(22),
            AgentKind::Codex | AgentKind::GeminiCli => Some(20),
            AgentKind::OpenCode | AgentKind::Custom => None,
        }
    }

    /// What signs the person in, in their own terminal, when the agent can't
    /// do it itself.
    pub fn sign_in(self) -> Option<&'static str> {
        match self {
            AgentKind::ClaudeAgent => Some("claude, then /login"),
            AgentKind::Codex => Some("codex login"),
            AgentKind::GeminiCli => Some("gemini"),
            AgentKind::OpenCode => Some("opencode auth login"),
            AgentKind::Custom => None,
        }
    }
}

/// Where a known agent was found, or why it wasn't.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Detection {
    Found(PathBuf),
    NotInstalled,
    /// Only the old, deprecated Claude Code adapter is installed.
    OldClaudeAdapter(PathBuf),
}

/// The old name of Claude's adapter, which Kanso finds but doesn't start.
pub const OLD_CLAUDE_ADAPTER: &str = "claude-code-acp";

/// Looks for `kind` in `dirs`, in order.
pub fn detect_in(kind: AgentKind, dirs: &[PathBuf]) -> Detection {
    let Some(command) = kind.command() else {
        return Detection::NotInstalled;
    };
    if let Some(path) = which_in(command, dirs) {
        return Detection::Found(path);
    }
    if kind == AgentKind::ClaudeAgent
        && let Some(path) = which_in(OLD_CLAUDE_ADAPTER, dirs)
    {
        return Detection::OldClaudeAdapter(path);
    }
    Detection::NotInstalled
}

/// Looks for `kind` everywhere Kanso searches.
pub fn detect(kind: AgentKind) -> Detection {
    detect_in(kind, &search_dirs())
}

/// The first executable called `name` in `dirs`. On Windows `.exe` and
/// `.cmd` are tried too.
pub fn which_in(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let names: Vec<String> = if cfg!(windows) && Path::new(name).extension().is_none() {
        ["exe", "cmd", "bat"]
            .iter()
            .map(|ext| format!("{name}.{ext}"))
            .collect()
    } else {
        vec![name.to_owned()]
    };
    dirs.iter()
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|path| is_executable(path))
}

fn is_executable(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Where agents install, beyond `PATH`: a GUI app on macOS doesn't get the
/// shell's `PATH`, and nvm, Volta and npm's global folder are often not on
/// it on Linux either.
pub fn extra_dirs(home: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(windows) {
        for var in ["APPDATA", "LOCALAPPDATA", "USERPROFILE"] {
            if let Some(base) = std::env::var_os(var) {
                let base = PathBuf::from(base);
                match var {
                    "APPDATA" => dirs.push(base.join("npm")),
                    "LOCALAPPDATA" => dirs.push(base.join("Programs")),
                    _ => dirs.push(base.join(".local").join("bin")),
                }
            }
        }
        return dirs;
    }
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"].map(PathBuf::from));
    if let Some(home) = home {
        for rel in [
            ".local/bin",
            ".npm-global/bin",
            ".volta/bin",
            ".bun/bin",
            ".cargo/bin",
            ".opencode/bin",
            ".asdf/shims",
        ] {
            dirs.push(home.join(rel));
        }
        // nvm keeps one folder per Node version; the newest goes first.
        if let Ok(entries) = std::fs::read_dir(home.join(".nvm/versions/node")) {
            let mut versions: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path().join("bin"))
                .collect();
            versions.sort_by_key(|dir| std::cmp::Reverse(node_dir_version(dir)));
            dirs.extend(versions);
        }
    }
    dirs
}

/// `…/v22.11.0/bin` as (22, 11, 0), for ordering nvm's folders.
fn node_dir_version(bin: &Path) -> (u32, u32, u32) {
    let name = bin
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("");
    parse_version(name).unwrap_or((0, 0, 0))
}

/// `v22.11.0` or `22.11.0` as numbers.
pub fn parse_version(text: &str) -> Option<(u32, u32, u32)> {
    let text = text.trim().trim_start_matches('v');
    let mut parts = text.split(['.', '-', ' ']).map(|part| part.parse::<u32>());
    let major = parts.next()?.ok()?;
    let minor = parts.next().and_then(Result::ok).unwrap_or(0);
    let patch = parts.next().and_then(Result::ok).unwrap_or(0);
    Some((major, minor, patch))
}

/// Every folder Kanso searches, in order, without repeats: `PATH`, the
/// login shell's `PATH` (read once), then the usual install folders.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();
    dirs.extend(login_shell_path().iter().cloned());
    let home = std::env::var_os("HOME").map(PathBuf::from);
    dirs.extend(extra_dirs(home.as_deref()));
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|dir| !dir.as_os_str().is_empty() && seen.insert(dir.clone()));
    dirs
}

/// The `PATH` an agent starts with: where it was found first, then every
/// folder Kanso searches, so a `#!/usr/bin/env node` script finds Node.
pub fn agent_path(found: &Path) -> Option<String> {
    let mut dirs = Vec::new();
    if let Some(parent) = found.parent() {
        dirs.push(parent.to_path_buf());
    }
    for dir in search_dirs() {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    std::env::join_paths(dirs)
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

/// The `PATH` the person's login shell sets, asked once with a short
/// timeout. Empty on Windows, or if the shell doesn't answer.
fn login_shell_path() -> &'static [PathBuf] {
    static PATH: OnceLock<Vec<PathBuf>> = OnceLock::new();
    PATH.get_or_init(|| {
        if cfg!(windows) {
            return Vec::new();
        }
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_owned());
        let Some(output) = run_with_timeout(
            &shell,
            &["-l", "-c", "printf '%s' \"$PATH\""],
            Duration::from_secs(2),
        ) else {
            return Vec::new();
        };
        // A login shell may print a banner first; the PATH is the last line.
        let last = output.lines().last().unwrap_or("").trim().to_owned();
        std::env::split_paths(&last).collect()
    })
}

/// Runs `program` and returns what it printed, or nothing if it fails or
/// takes longer than `timeout` (then it is killed).
fn run_with_timeout(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    use std::io::Read as _;
    let mut child = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let until = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if std::time::Instant::now() >= until => {
                child.kill().ok();
                child.wait().ok();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    Some(out)
}

/// The Node.js version on `path`, asked with a short timeout.
pub fn node_version(path: &str) -> Option<(u32, u32, u32)> {
    let dirs: Vec<PathBuf> = std::env::split_paths(path).collect();
    let node = which_in("node", &dirs)?;
    let out = run_with_timeout(node.to_str()?, &["--version"], Duration::from_secs(2))?;
    parse_version(&out)
}

/// A custom command's arguments, typed on one line: split on spaces, with
/// double quotes keeping spaces in one argument. Never run through a shell.
pub fn split_args(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut any = false;
    for ch in line.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            ch if ch.is_whitespace() && !quoted => {
                if any {
                    args.push(std::mem::take(&mut current));
                    any = false;
                }
            }
            ch => {
                current.push(ch);
                any = true;
            }
        }
    }
    if any {
        args.push(current);
    }
    args
}

/// `NAME=value` pairs typed on one line, split like [`split_args`]. A pair
/// without `=` or with an empty name is skipped.
pub fn split_env(line: &str) -> Vec<(String, String)> {
    split_args(line)
        .into_iter()
        .filter_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            let name = name.trim();
            (!name.is_empty()).then(|| (name.to_owned(), value.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn executable(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    fn command_file(kind: AgentKind) -> String {
        let command = kind.command().unwrap();
        if cfg!(windows) {
            format!("{command}.cmd")
        } else {
            command.to_owned()
        }
    }

    #[test]
    fn detection_finds_fixture_binaries_in_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let dirs = vec![first.path().to_path_buf(), second.path().to_path_buf()];
        assert_eq!(detect_in(AgentKind::Codex, &dirs), Detection::NotInstalled);
        let later = executable(second.path(), &command_file(AgentKind::Codex));
        assert_eq!(detect_in(AgentKind::Codex, &dirs), Detection::Found(later));
        let earlier = executable(first.path(), &command_file(AgentKind::Codex));
        assert_eq!(
            detect_in(AgentKind::Codex, &dirs),
            Detection::Found(earlier)
        );
        assert_eq!(detect_in(AgentKind::Custom, &dirs), Detection::NotInstalled);
    }

    #[test]
    fn the_old_claude_adapter_is_found_but_named() {
        let dir = tempfile::tempdir().unwrap();
        let dirs = vec![dir.path().to_path_buf()];
        let name = if cfg!(windows) {
            format!("{OLD_CLAUDE_ADAPTER}.cmd")
        } else {
            OLD_CLAUDE_ADAPTER.to_owned()
        };
        let old = executable(dir.path(), &name);
        assert_eq!(
            detect_in(AgentKind::ClaudeAgent, &dirs),
            Detection::OldClaudeAdapter(old)
        );
        let new = executable(dir.path(), &command_file(AgentKind::ClaudeAgent));
        assert_eq!(
            detect_in(AgentKind::ClaudeAgent, &dirs),
            Detection::Found(new)
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_file_without_execute_permission_is_not_an_agent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("codex-acp"), "").unwrap();
        assert_eq!(
            detect_in(AgentKind::Codex, &[dir.path().to_path_buf()]),
            Detection::NotInstalled
        );
    }

    #[cfg(unix)]
    #[test]
    fn nvm_folders_are_searched_newest_first() {
        let home = tempfile::tempdir().unwrap();
        for version in ["v20.1.0", "v22.11.0", "v9.0.0"] {
            std::fs::create_dir_all(
                home.path()
                    .join(".nvm/versions/node")
                    .join(version)
                    .join("bin"),
            )
            .unwrap();
        }
        let dirs = extra_dirs(Some(home.path()));
        let nvm: Vec<String> = dirs
            .iter()
            .filter(|dir| dir.to_string_lossy().contains(".nvm"))
            .map(|dir| {
                dir.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(nvm, vec!["v22.11.0", "v20.1.0", "v9.0.0"]);
        assert!(dirs.contains(&PathBuf::from("/opt/homebrew/bin")));
    }

    #[test]
    fn every_known_agent_says_how_to_install_and_sign_in() {
        for kind in AgentKind::ALL {
            assert!(!kind.label().is_empty());
            assert_eq!(kind.command().is_some(), kind != AgentKind::Custom);
            assert_eq!(kind.install().is_some(), kind != AgentKind::Custom);
            assert_eq!(kind.sign_in().is_some(), kind != AgentKind::Custom);
        }
        assert_eq!(AgentKind::GeminiCli.args(), &["--acp"]);
        assert_eq!(AgentKind::OpenCode.args(), &["acp"]);
        assert_eq!(parse_version("v22.11.0\n"), Some((22, 11, 0)));
        assert_eq!(parse_version("20"), Some((20, 0, 0)));
        assert_eq!(parse_version("node"), None);
    }

    #[test]
    fn arguments_split_on_spaces_and_keep_quotes_together() {
        assert_eq!(
            split_args("  --acp  --model \"big one\" \"\" "),
            vec!["--acp", "--model", "big one", ""]
        );
        assert_eq!(
            split_env("A=1 B=\"two words\" bad =x C="),
            vec![
                ("A".to_owned(), "1".to_owned()),
                ("B".to_owned(), "two words".to_owned()),
                ("C".to_owned(), String::new()),
            ]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Plain words joined by spaces split back into the same words: the
        /// round trip a custom command's arguments rely on.
        #[test]
        fn prop_plain_arguments_round_trip(
            args in prop::collection::vec("[A-Za-z0-9_./:=-]{1,12}", 0..8),
            gaps in prop::collection::vec(1usize..4, 0..8),
        ) {
            let mut line = String::new();
            for (ix, arg) in args.iter().enumerate() {
                line.push_str(&" ".repeat(*gaps.get(ix).unwrap_or(&1)));
                line.push_str(arg);
            }
            prop_assert_eq!(split_args(&line), args.clone());
            let quoted: Vec<String> = args.iter().map(|arg| format!("\"{arg} x\"")).collect();
            let expected: Vec<String> = args.iter().map(|arg| format!("{arg} x")).collect();
            prop_assert_eq!(split_args(&quoted.join(" ")), expected);
        }
    }
}
