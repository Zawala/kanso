//! Which grammar colors a fenced code block: the info string after the
//! opening fence names a language, in whatever spelling the writer used
//! (`sh`, `zsh`, `golang`, `c++`, `yml`), and `grammar` finds the grammar the
//! highlighter knows it by. The table is the engine's, so the desktop and the
//! mobile app agree (`kanso_engine::highlight`); registering the grammars
//! with the component library is the desktop's.

pub use kanso_engine::highlight::grammar;

/// Gives the grammars gpui-kit compiles in without a highlight query
/// (Swift and C#) the query their own crates ship, so their code is colored
/// like every other language's. Does nothing the second time.
// no-prop-test: hands fixed queries to the library; the tests below check
// that they color code.
pub fn ensure_registered() {
    use gpui_kit::component::highlighter::LanguageRegistry;
    static DONE: std::sync::Once = std::sync::Once::new();
    DONE.call_once(|| {
        let registry = LanguageRegistry::singleton();
        for (name, query) in [
            ("swift", tree_sitter_swift::HIGHLIGHTS_QUERY),
            ("csharp", tree_sitter_c_sharp::HIGHLIGHTS_QUERY),
        ] {
            if let Some(mut config) = registry.language(name)
                && config.highlights.is_empty()
            {
                config.highlights = query.into();
                registry.register(name, &config);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use kanso_engine::highlight::GRAMMARS;

    /// Swift and C# code is colored once their queries are registered:
    /// keywords, strings and type names get styles of their own.
    #[test]
    fn swift_and_csharp_are_highlighted() {
        use gpui_kit::base::input::Rope;
        use gpui_kit::component::highlighter::{HighlightTheme, SyntaxHighlighter};
        super::ensure_registered();
        let theme = HighlightTheme::default_dark();
        for (lang, code) in [
            ("swift", "struct Note { let id = \"x\" }\n"),
            ("csharp", "public class Note { string Id = \"x\"; }\n"),
        ] {
            let mut highlighter = SyntaxHighlighter::new(lang);
            highlighter.update(None, &Rope::from_str(code), None);
            let styled = highlighter
                .styles(&(0..code.len()), theme.as_ref())
                .into_iter()
                .filter(|(_, style)| style.color.is_some())
                .count();
            assert!(styled >= 2, "{lang}: {styled} styled ranges");
        }
    }

    /// Every grammar named here is compiled in: the table and the
    /// workspace's `gpui-kit` features agree.
    #[test]
    fn every_grammar_is_registered() {
        use gpui_kit::component::highlighter::LanguageRegistry;
        let registry = LanguageRegistry::singleton();
        for names in GRAMMARS {
            assert!(
                registry.language(names[0]).is_some(),
                "{} is not compiled in",
                names[0]
            );
        }
    }
}
