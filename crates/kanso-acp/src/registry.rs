//! The ACP registry: the agents that speak the protocol, as Zed lists them.
//! Claude Agent, Codex, Gemini CLI and OpenCode are featured and tested with
//! Kanso; every other entry is "Not tested with Kanso".
//!
//! The app fetches [`URL`] in the background and reads it with [`parse`];
//! nothing here touches the network. An npm agent can be installed with
//! [`npm_install`], only after the person confirms, and only a package name
//! that is a plain npm name (never an option, a path or a URL). Kanso never
//! runs `npx -y`.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::AgentKind;
use crate::detect::which_in;

/// Where the registry is published.
pub const URL: &str = "https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json";

/// The longest an install may run before it is stopped.
pub const INSTALL_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// How an agent is installed and started, as its entry says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Distribution {
    /// An npm package, run with `args` once installed.
    Npm {
        /// The package, with its version: `@scope/name@1.2.3`.
        package: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
    },
    /// A download for each platform, or a Python package: installed from
    /// the agent's own site, not by Kanso.
    Other,
}

/// One agent in the registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryAgent {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    /// Its site, or failing that its repository.
    pub website: Option<String>,
    pub distribution: Distribution,
}

impl RegistryAgent {
    /// The agent Kanso knows by name, if this is one of the four featured
    /// and tested.
    pub fn featured(&self) -> Option<AgentKind> {
        featured(&self.id)
    }

    /// The npm package to install, if it is one Kanso can install.
    pub fn npm_package(&self) -> Option<&str> {
        match &self.distribution {
            Distribution::Npm { package, .. } if is_npm_package(package) => Some(package),
            _ => None,
        }
    }

    /// The command that installs it, as the person sees it.
    pub fn install_command(&self) -> Option<String> {
        self.npm_package()
            .map(|package| format!("npm install -g {package}"))
    }
}

/// The featured agent a registry id stands for.
pub fn featured(id: &str) -> Option<AgentKind> {
    match id {
        "claude-acp" => Some(AgentKind::ClaudeAgent),
        "codex-acp" => Some(AgentKind::Codex),
        "gemini" => Some(AgentKind::GeminiCli),
        "opencode" => Some(AgentKind::OpenCode),
        _ => None,
    }
}

/// Reads the registry's JSON. Entries without an id or a name are skipped;
/// one that can't be read doesn't stop the rest.
pub fn parse(json: &str) -> Result<Vec<RegistryAgent>, String> {
    let value: Value = serde_json::from_str(json).map_err(|err| err.to_string())?;
    let agents = value
        .get("agents")
        .and_then(Value::as_array)
        .ok_or("the registry has no agents list")?;
    let text = |value: &Value, key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<RegistryAgent> = agents
        .iter()
        .filter_map(|agent| {
            let distribution = match agent.pointer("/distribution/npx") {
                Some(npx) => match text(npx, "package") {
                    Some(package) => Distribution::Npm {
                        package,
                        args: npx
                            .get("args")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|arg| arg.as_str().map(str::to_owned))
                            .collect(),
                        env: npx
                            .get("env")
                            .and_then(Value::as_object)
                            .into_iter()
                            .flatten()
                            .filter_map(|(name, value)| {
                                Some((name.clone(), value.as_str()?.to_owned()))
                            })
                            .collect(),
                    },
                    None => Distribution::Other,
                },
                None => Distribution::Other,
            };
            Some(RegistryAgent {
                id: text(agent, "id")?,
                name: text(agent, "name")?,
                version: text(agent, "version").unwrap_or_default(),
                description: text(agent, "description").unwrap_or_default(),
                website: text(agent, "website").or_else(|| text(agent, "repository")),
                distribution,
            })
        })
        .filter(|agent| seen.insert(agent.id.clone()))
        .collect();
    out.sort_by_key(|agent| agent.name.to_lowercase());
    Ok(out)
}

/// Whether `package` is a plain npm package name, optionally with a version:
/// `name`, `@scope/name`, `name@1.2.3`. Never an option (`-g`), a path, a
/// URL, a git address or a tarball, so the install can't be turned into
/// something else.
pub fn is_npm_package(package: &str) -> bool {
    fn name_part(part: &str) -> bool {
        !part.is_empty()
            && part.len() <= 214
            && !part.starts_with(['.', '_', '-'])
            && part
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || "-._~".contains(ch))
    }
    let (name, version) = split_version(package);
    if let Some(version) = version
        && (version.is_empty()
            || !version
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ".+-".contains(ch))
            || version.starts_with('-'))
    {
        return false;
    }
    match name.strip_prefix('@') {
        Some(scoped) => {
            matches!(scoped.split_once('/'), Some((scope, rest)) if name_part(scope) && name_part(rest))
        }
        None => name_part(name),
    }
}

/// `@scope/name@1.2` as `("@scope/name", Some("1.2"))`.
fn split_version(package: &str) -> (&str, Option<&str>) {
    let from = usize::from(package.starts_with('@'));
    match package[from..].find('@') {
        Some(at) => (&package[..from + at], Some(&package[from + at + 1..])),
        None => (package, None),
    }
}

/// The package's name without its version.
pub fn package_name(package: &str) -> &str {
    split_version(package).0
}

/// The folders npm installs global packages into, beside the folders Kanso
/// searches for programs: `<prefix>/lib/node_modules` next to each `npm`
/// (`<prefix>\node_modules` on Windows).
pub fn global_roots(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for dir in dirs {
        if which_in("npm", std::slice::from_ref(dir)).is_none() {
            continue;
        }
        let root = if cfg!(windows) {
            dir.join("node_modules")
        } else {
            match dir.parent() {
                Some(prefix) => prefix.join("lib").join("node_modules"),
                None => continue,
            }
        };
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    roots
}

/// The command a globally installed npm `package` runs as, from its
/// `package.json`'s `bin`, if it is installed under one of `roots`.
pub fn installed_bin(package: &str, roots: &[PathBuf]) -> Option<String> {
    let name = package_name(package);
    roots.iter().find_map(|root| {
        let manifest = std::fs::read_to_string(root.join(name).join("package.json")).ok()?;
        bin_name(name, &serde_json::from_str(&manifest).ok()?)
    })
}

/// The command `package.json` names in `bin`: a string means the package's
/// own name (without its scope); with several, the one named like the
/// package, else the first.
pub fn bin_name(package: &str, manifest: &Value) -> Option<String> {
    let unscoped = package.rsplit('/').next().unwrap_or(package);
    let valid = |name: &str| {
        !name.is_empty()
            && !name.starts_with('-')
            && !name.contains(['/', '\\'])
            && name != "."
            && name != ".."
    };
    let bin = manifest.get("bin")?;
    let name = match bin {
        Value::String(_) => unscoped.to_owned(),
        Value::Object(bins) => {
            if bins.contains_key(unscoped) {
                unscoped.to_owned()
            } else {
                bins.keys().next()?.clone()
            }
        }
        _ => return None,
    };
    valid(&name).then_some(name)
}

/// Runs `npm install -g <package>` with `npm` from `dirs` and `env`, waiting
/// up to [`INSTALL_TIMEOUT`]. On failure, the last of what npm wrote.
pub fn npm_install(
    package: &str,
    dirs: &[PathBuf],
    env: &[(String, String)],
) -> Result<(), String> {
    if !is_npm_package(package) {
        return Err(format!("{package} isn't an npm package Kanso can install."));
    }
    let npm = which_in("npm", dirs).ok_or(
        "npm isn't on this computer. Install Node.js (it comes with npm), then Find again.",
    )?;
    run_install(&npm, &["install", "-g", package], env)
}

fn run_install(program: &Path, args: &[&str], env: &[(String, String)]) -> Result<(), String> {
    let mut child = std::process::Command::new(program)
        .args(args)
        .env_clear()
        .envs(env.iter().map(|(name, value)| (name, value)))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| format!("npm didn't start: {err}"))?;
    // npm can write a lot; read it as it comes so it never blocks.
    let stderr = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        if let Some(mut stderr) = stderr {
            stderr.read_to_end(&mut out).ok();
        }
        let text = String::from_utf8_lossy(&out).into_owned();
        let lines: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        lines[lines.len().saturating_sub(3)..].join(" ")
    });
    let until = Instant::now() + INSTALL_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() >= until => {
                child.kill().ok();
                child.wait().ok();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(err) => return Err(err.to_string()),
        }
    };
    let said = reader.join().unwrap_or_default();
    match status {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err(format!("npm couldn't install it. It said: {said}")),
        None => Err("npm took longer than 10 minutes, so Kanso stopped it.".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const SAMPLE: &str = r#"{"version": "1.0.0", "agents": [
        {"id": "claude-acp", "name": "Claude Agent", "version": "0.95.0", "description": "ACP wrapper",
         "repository": "https://github.com/agentclientprotocol/claude-agent-acp",
         "distribution": {"npx": {"package": "@agentclientprotocol/claude-agent-acp@0.95.0"}}},
        {"id": "github-copilot-cli", "name": "Copilot", "version": "1.0.103", "website": "https://github.com/features/copilot/cli/",
         "distribution": {"npx": {"package": "@github/copilot@1.0.103", "args": ["--acp"], "env": {"X": "1"}}}},
        {"id": "goose", "name": "goose", "version": "1.0", "distribution": {"binary": {"darwin-aarch64": {"archive": "https://x", "cmd": "./goose"}}}},
        {"id": "", "name": "No id"},
        {"id": "bad", "name": "Bad", "distribution": {"npx": {"package": "-g evil"}}}
    ], "extensions": []}"#;

    #[test]
    fn the_registry_reads_featured_npm_and_other_agents() {
        let agents = parse(SAMPLE).unwrap();
        let ids: Vec<&str> = agents.iter().map(|agent| agent.id.as_str()).collect();
        assert_eq!(ids, ["bad", "claude-acp", "github-copilot-cli", "goose"]);
        let claude = &agents[1];
        assert_eq!(claude.featured(), Some(AgentKind::ClaudeAgent));
        assert_eq!(
            claude.install_command().as_deref(),
            Some("npm install -g @agentclientprotocol/claude-agent-acp@0.95.0")
        );
        assert_eq!(
            claude.website.as_deref(),
            Some("https://github.com/agentclientprotocol/claude-agent-acp")
        );
        let copilot = &agents[2];
        assert_eq!(copilot.featured(), None);
        assert_eq!(
            copilot.distribution,
            Distribution::Npm {
                package: "@github/copilot@1.0.103".into(),
                args: vec!["--acp".into()],
                env: vec![("X".into(), "1".into())],
            }
        );
        assert_eq!(agents[3].distribution, Distribution::Other);
        assert_eq!(agents[3].install_command(), None);
        // A package that isn't a plain name is never offered.
        assert_eq!(agents[0].install_command(), None);
        assert!(parse("{}").is_err());
        assert!(parse("not json").is_err());
    }

    #[test]
    fn only_plain_npm_names_are_installable() {
        for ok in [
            "opencode-ai",
            "@google/gemini-cli",
            "@google/gemini-cli@0.65.0",
            "a@1.0.0-beta.1+build",
            "x.y_z~",
        ] {
            assert!(is_npm_package(ok), "{ok}");
        }
        for bad in [
            "",
            "-g",
            "--prefix=/tmp",
            "./local",
            "../up",
            "/abs/path",
            "https://evil.example/x.tgz",
            "git+ssh://git@github.com/x/y",
            "github:x/y",
            "file:../x",
            "Upper",
            "a b",
            "@scope",
            "@/x",
            "@scope/",
            "x@",
            "x@-1",
            "x@1;rm",
            "_private",
        ] {
            assert!(!is_npm_package(bad), "{bad}");
        }
        assert_eq!(
            package_name("@google/gemini-cli@0.65.0"),
            "@google/gemini-cli"
        );
        assert_eq!(package_name("opencode-ai"), "opencode-ai");
    }

    #[test]
    fn an_installed_package_is_found_by_its_bin() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("lib").join("node_modules");
        let pkg = root.join("@github").join("copilot");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"bin": {"copilot": "index.js"}}"#,
        )
        .unwrap();
        assert_eq!(
            installed_bin("@github/copilot@1.0.103", std::slice::from_ref(&root)).as_deref(),
            Some("copilot")
        );
        assert_eq!(installed_bin("@github/other", &[root]), None);
        let one = serde_json::json!({"bin": "cli.js"});
        assert_eq!(bin_name("@scope/tool", &one).as_deref(), Some("tool"));
        let many = serde_json::json!({"bin": {"a": "a.js", "tool": "t.js"}});
        assert_eq!(bin_name("tool", &many).as_deref(), Some("tool"));
        let bad = serde_json::json!({"bin": {"../x": "a.js"}});
        assert_eq!(bin_name("tool", &bad), None);
        assert_eq!(bin_name("tool", &serde_json::json!({})), None);
    }

    #[test]
    fn a_package_that_isnt_plain_is_never_run() {
        let err = npm_install("--global-style", &[], &[]).unwrap_err();
        assert!(err.contains("isn't an npm package"), "{err}");
        let err = npm_install("ok-name", &[], &[]).unwrap_err();
        assert!(err.contains("npm isn't on this computer"), "{err}");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Whatever the registry says, the command Kanso would run installs
        /// one plain package: it never starts with `-`, never has a space,
        /// a slash outside the scope, or a URL scheme.
        #[test]
        fn prop_an_install_is_only_ever_one_plain_package(package in "[ -~]{0,40}") {
            if is_npm_package(&package) {
                prop_assert!(!package.starts_with('-'));
                prop_assert!(!package.contains(' ') && !package.contains(':'));
                prop_assert!(!package.contains('\\'));
                prop_assert!(package.matches('/').count() <= usize::from(package.starts_with('@')));
                let name = package_name(&package);
                prop_assert!(!name.is_empty());
            }
        }

        /// Reading any JSON never panics, and every agent read has an id and
        /// a name, once each.
        #[test]
        fn prop_reading_the_registry_never_panics(
            entries in prop::collection::vec(("[a-z-]{0,6}", "[ -~]{0,8}", "[ -~]{0,20}"), 0..8),
        ) {
            let agents: Vec<Value> = entries
                .iter()
                .map(|(id, name, package)| serde_json::json!({
                    "id": id, "name": name, "distribution": {"npx": {"package": package}},
                }))
                .collect();
            let json = serde_json::json!({ "agents": agents }).to_string();
            let read = parse(&json).unwrap();
            let mut ids = std::collections::HashSet::new();
            for agent in &read {
                prop_assert!(!agent.id.is_empty() && !agent.name.is_empty());
                prop_assert!(ids.insert(agent.id.clone()), "{} twice", agent.id);
                if let Some(command) = agent.install_command() {
                    prop_assert!(command.starts_with("npm install -g "));
                    prop_assert_eq!(command.split(' ').count(), 4);
                }
            }
        }
    }
}
