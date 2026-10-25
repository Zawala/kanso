//! The environment an agent starts with: a short allowlist of what any
//! program needs, plus the vendor keys and proxy settings the person already
//! set, and never anything of Kanso's.

/// Variables passed through by exact name (compared without case on
/// Windows, where names are case-insensitive).
const PASSED: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "TMPDIR",
    "TEMP",
    "TMP",
    "LANG",
    "LANGUAGE",
    "TZ",
    // Where agents keep their own sign-in and settings.
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
    "XDG_STATE_HOME",
    "CLAUDE_CONFIG_DIR",
    "CODEX_HOME",
    "CODEX_PATH",
    "GEMINI_CLI_HOME",
    "OPENCODE_CONFIG",
    // Vendor keys the person set themselves.
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "CODEX_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GOOGLE_AI_API_KEY",
    "GOOGLE_CLOUD_PROJECT",
    "GOOGLE_APPLICATION_CREDENTIALS",
    // Proxies and certificates, so an agent works on the same network.
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
    "NODE_EXTRA_CA_CERTS",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    // Windows: what a process needs to start and find its folders.
    "SYSTEMROOT",
    "SYSTEMDRIVE",
    "WINDIR",
    "COMSPEC",
    "PATHEXT",
    "USERPROFILE",
    "USERNAME",
    "APPDATA",
    "LOCALAPPDATA",
    "PROGRAMDATA",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "PROGRAMW6432",
    "COMMONPROGRAMFILES",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
    "OS",
];

/// Whether a variable named `name` may reach an agent. Nothing starting
/// with `KANSO_` ever does, whatever else is true.
pub fn passes(name: &str) -> bool {
    if name.to_ascii_uppercase().starts_with("KANSO_") {
        return false;
    }
    if name.starts_with("LC_") {
        return true;
    }
    if cfg!(windows) {
        PASSED
            .iter()
            .any(|passed| passed.eq_ignore_ascii_case(name))
    } else {
        PASSED.contains(&name)
    }
}

/// The agent's environment from `inherited`: the allowlist, `PATH` replaced
/// by `path` when one is given, then `extra` (the person's own variables for
/// a custom agent, and Kanso's documented mode switches). `extra` cannot add
/// a `KANSO_` variable either.
pub fn agent_env(
    inherited: impl IntoIterator<Item = (String, String)>,
    path: Option<String>,
    extra: &[(String, String)],
) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = inherited
        .into_iter()
        .filter(|(name, _)| passes(name))
        .filter(|(name, _)| path.is_none() || !name.eq_ignore_ascii_case("PATH"))
        .collect();
    if let Some(path) = path {
        env.push(("PATH".to_owned(), path));
    }
    for (name, value) in extra {
        let name = name.trim();
        if name.is_empty() || name.contains('=') || name.to_ascii_uppercase().starts_with("KANSO_")
        {
            continue;
        }
        env.retain(|(have, _)| !have.eq_ignore_ascii_case(name));
        env.push((name.to_owned(), value.clone()));
    }
    env
}

#[cfg(test)]
mod tests {
    use super::{agent_env, passes};
    use proptest::prelude::*;

    #[test]
    fn env_allowlist_has_no_kanso_secrets() {
        let inherited = [
            ("PATH", "/usr/bin"),
            ("HOME", "/Users/me"),
            ("KANSO_DATA_DIR", "/secret"),
            ("KANSO_API_PASSWORD", "pw"),
            ("kanso_sync_token", "t"),
            ("AWS_SECRET_ACCESS_KEY", "aws"),
            ("GITHUB_TOKEN", "gh"),
            ("ANTHROPIC_API_KEY", "mine"),
            ("LC_ALL", "en_US.UTF-8"),
        ]
        .map(|(name, value)| (name.to_owned(), value.to_owned()));
        let env = agent_env(
            inherited,
            Some("/opt/homebrew/bin:/usr/bin".to_owned()),
            &[
                ("KANSO_MCP_TOKEN".to_owned(), "x".to_owned()),
                ("INITIAL_AGENT_MODE".to_owned(), "read-only".to_owned()),
            ],
        );
        let names: Vec<&str> = env.iter().map(|(name, _)| name.as_str()).collect();
        assert!(
            names
                .iter()
                .all(|name| !name.to_ascii_uppercase().starts_with("KANSO_"))
        );
        assert!(!names.contains(&"AWS_SECRET_ACCESS_KEY"));
        assert!(!names.contains(&"GITHUB_TOKEN"));
        assert!(names.contains(&"ANTHROPIC_API_KEY"));
        assert!(names.contains(&"LC_ALL"));
        assert!(names.contains(&"INITIAL_AGENT_MODE"));
        let path: Vec<&str> = env
            .iter()
            .filter(|(name, _)| name == "PATH")
            .map(|(_, value)| value.as_str())
            .collect();
        assert_eq!(path, vec!["/opt/homebrew/bin:/usr/bin"]);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Whatever is inherited or added, no `KANSO_` variable reaches the
        /// agent, every variable passed is on the allowlist or was added on
        /// purpose, and no name appears twice.
        #[test]
        fn prop_agent_env_never_carries_kanso_variables(
            inherited in prop::collection::vec(("[A-Za-z_]{1,12}", "[ -~]{0,8}"), 0..20),
            kanso in prop::collection::vec("[A-Z_]{1,8}", 0..4),
            extra in prop::collection::vec(("[A-Za-z_]{1,10}", "[ -~]{0,8}"), 0..5),
        ) {
            let mut all = inherited.clone();
            all.extend(kanso.iter().map(|name| (format!("KANSO_{name}"), "secret".to_owned())));
            let mut added = extra.clone();
            added.extend(kanso.iter().map(|name| (format!("kanso_{name}"), "secret".to_owned())));
            let env = agent_env(all, None, &added);
            let mut seen = std::collections::HashSet::new();
            for (name, _) in &env {
                prop_assert!(!name.to_ascii_uppercase().starts_with("KANSO_"));
                prop_assert!(passes(name) || added.iter().any(|(extra, _)| extra.trim() == name));
                if extra.iter().any(|(extra, _)| extra.trim().eq_ignore_ascii_case(name)) {
                    prop_assert!(seen.insert(name.to_ascii_uppercase()));
                }
            }
        }
    }
}
