//! Which grammar colors a fenced code block: the info string after the
//! opening fence names a language, in whatever spelling the writer used
//! (`sh`, `zsh`, `golang`, `c++`, `yml`), and this finds the grammar the
//! highlighter knows it by.

/// Each grammar Kanso highlights, with the names a fence may give it. The
/// first name is the grammar's own.
pub const GRAMMARS: &[&[&str]] = &[
    &[
        "bash",
        "sh",
        "shell",
        "zsh",
        "console",
        "shellscript",
        "shell-script",
    ],
    &["c", "h"],
    &["cpp", "c++", "cc", "cxx", "hpp", "hxx", "hh"],
    &["csharp", "cs", "c#"],
    &["css", "scss"],
    &["diff", "patch"],
    &["go", "golang"],
    &["html", "htm", "xhtml", "xml", "svg"],
    &["java"],
    &["javascript", "js", "jsx", "mjs", "cjs", "node"],
    &["json", "jsonc", "json5"],
    &["kotlin", "kt", "kts"],
    &["lua"],
    &["markdown", "md", "mdx"],
    &["php"],
    &["python", "py", "python3", "py3"],
    &["ruby", "rb"],
    &["rust", "rs"],
    &[
        "sql",
        "postgres",
        "postgresql",
        "psql",
        "pgsql",
        "mysql",
        "sqlite",
        "plsql",
    ],
    &["swift"],
    &["toml"],
    &["tsx"],
    &["typescript", "ts", "mts", "cts"],
    &["yaml", "yml"],
];

/// The grammar for a fence's info string, or `None` when it names none
/// Kanso has. Only the first word counts (`rust,ignore`, `js {1,3}` and
/// `{.python}` work), and case doesn't matter.
pub fn grammar(info: &str) -> Option<&'static str> {
    let name = info
        .trim_start_matches(|c: char| c.is_whitespace() || c == '{' || c == '.')
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '{' | '}' | ':'))
        .next()?
        .to_lowercase();
    if name.is_empty() {
        return None;
    }
    GRAMMARS
        .iter()
        .find(|names| names.contains(&name.as_str()))
        .map(|names| names[0])
}

#[cfg(test)]
mod tests {
    use super::{GRAMMARS, grammar};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// Any name a fence may use, or something else.
    fn info() -> impl Strategy<Value = String> {
        let names: Vec<&'static str> = GRAMMARS.iter().flat_map(|n| n.iter().copied()).collect();
        prop_oneof![
            proptest::sample::select(names).prop_map(str::to_owned),
            "[a-zA-Z0-9][a-zA-Z0-9#+._-]{0,9}",
        ]
    }

    proptest! {
        #![proptest_config(config())]

        /// Closure and idempotence: a grammar is always one of the
        /// grammars, and it names itself.
        #[test]
        fn prop_grammars_name_themselves(info in info()) {
            if let Some(g) = grammar(&info) {
                prop_assert!(GRAMMARS.iter().any(|names| names[0] == g));
                prop_assert_eq!(grammar(g), Some(g));
            }
        }

        /// Invariant: case doesn't matter.
        #[test]
        fn prop_case_insensitive(info in info()) {
            prop_assert_eq!(grammar(&info.to_uppercase()), grammar(&info));
            prop_assert_eq!(grammar(&info.to_lowercase()), grammar(&info));
        }

        /// Metamorphic: what follows the first word, after a space, a
        /// comma or a brace, changes nothing.
        #[test]
        fn prop_only_the_first_word_counts(
            info in info(),
            sep in prop::sample::select(vec![" ", ",", " {", "{", ":"]),
            rest in "[ -~]{0,12}",
        ) {
            prop_assume!(!info.is_empty());
            prop_assert_eq!(grammar(&format!("{info}{sep}{rest}")), grammar(&info));
        }

        /// Every listed spelling finds its grammar, padded or not.
        #[test]
        fn prop_every_alias_resolves(ix in 0..GRAMMARS.len(), pick in any::<prop::sample::Index>()) {
            let names = GRAMMARS[ix];
            let alias = names[pick.index(names.len())];
            prop_assert_eq!(grammar(alias), Some(names[0]));
            prop_assert_eq!(grammar(&format!("  {alias}  ")), Some(names[0]));
        }
    }

    #[test]
    fn examples() {
        assert_eq!(grammar("rust"), Some("rust"));
        assert_eq!(grammar("rust,ignore"), Some("rust"));
        assert_eq!(grammar("Shell"), Some("bash"));
        assert_eq!(grammar("zsh"), Some("bash"));
        assert_eq!(grammar("golang"), Some("go"));
        assert_eq!(grammar("c++"), Some("cpp"));
        assert_eq!(grammar("C#"), Some("csharp"));
        assert_eq!(grammar("jsx"), Some("javascript"));
        assert_eq!(grammar("yml"), Some("yaml"));
        assert_eq!(grammar("js {1,3}"), Some("javascript"));
        assert_eq!(grammar("{.python}"), Some("python"));
        assert_eq!(grammar("dart"), None);
        assert_eq!(grammar("text"), None);
        assert_eq!(grammar(""), None);
        assert_eq!(grammar("   "), None);
    }

    /// Each spelling belongs to one grammar only.
    #[test]
    fn aliases_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for name in GRAMMARS.iter().flat_map(|n| n.iter()) {
            assert!(seen.insert(*name), "{name} listed twice");
        }
    }
}
