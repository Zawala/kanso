//! How the editor parses Markdown source for highlighting.
//!
//! The component library parses the inline Markdown of a note (emphasis,
//! code spans, links) as one combined document, putting back the text
//! between the paragraphs it keeps. A fenced code block between two such
//! paragraphs then reads as an inline code span, so its opening fence is
//! drawn in the code color, and a stray backtick in one list item pairs up
//! with one in the next. Kanso parses each paragraph on its own instead, as
//! Markdown does.
//!
//! Separate parses are capped at 512 a note; past that, paragraphs keep
//! their block styles but lose inline ones. One note in the real library
//! (about 2,900 styled paragraphs) reaches it, and the combined parse cut
//! that note short too. Highlighting every note of that library took 3.9 s
//! this way against 4.9 s combined.

use std::sync::Once;

use gpui_kit::component::highlighter::LanguageRegistry;

/// Makes the editor's Markdown parse each paragraph's inline text on its own.
/// Only editors created afterwards pick it up; it does nothing the second
/// time.
pub fn ensure_registered() {
    static DONE: Once = Once::new();
    DONE.call_once(|| {
        let registry = LanguageRegistry::singleton();
        let Some(mut config) = registry.language("markdown") else {
            return;
        };
        // no-prop-test: a fixed edit to the library's query; the tests below
        // check what it highlights.
        config.injections = config
            .injections
            .replace("(#set! injection.combined)", "")
            .into();
        registry.register("markdown", &config);
    });
}

#[cfg(test)]
mod tests {
    use super::ensure_registered;
    use crate::theme::{DARK, LIGHT, Rgb, theme_set};
    use gpui_kit::Hsla;
    use gpui_kit::base::input::Rope;
    use gpui_kit::component::highlighter::{HighlightTheme, LanguageRegistry, SyntaxHighlighter};
    use gpui_kit::component::{ThemeMode, ThemeSet};

    /// The color each byte of `source` is drawn in, with Kanso's theme.
    fn colors(source: &str, dark: bool) -> Vec<Option<Hsla>> {
        let set: ThemeSet = serde_json::from_value(theme_set(None)).unwrap();
        let theme = HighlightTheme {
            name: "Kanso".into(),
            appearance: if dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            },
            style: set.themes[usize::from(dark)].highlight.clone().unwrap(),
        };
        let mut highlighter = SyntaxHighlighter::new("markdown");
        highlighter.update(None, &Rope::from_str(source), None);
        let mut colors = vec![None; source.len()];
        for (range, style) in highlighter.styles(&(0..source.len()), &theme) {
            for color in &mut colors[range] {
                *color = style.color;
            }
        }
        colors
    }

    /// The colors of every byte of `needle`'s first match.
    fn colors_of(source: &str, colors: &[Option<Hsla>], needle: &str) -> Vec<Option<Hsla>> {
        let start = source.find(needle).unwrap();
        colors[start..start + needle.len()].to_vec()
    }

    /// A fenced block between paragraphs keeps its quiet fence and label,
    /// while inline code around it is still code, in both themes.
    #[test]
    fn fences_recede_and_code_spans_stay_code() {
        ensure_registered();
        let source = "*a*\n\n```json\n{}\n```\n\nx `y`";
        for (dark, c) in [(true, &DARK), (false, &LIGHT)] {
            let colors = colors(source, dark);
            let color = |rgb: Rgb| Some(Hsla::from(rgb));
            let quiet = [color(c.text_muted), color(c.text_secondary)];
            let fence = colors_of(source, &colors, "```json");
            assert!(fence.iter().all(|c| quiet.contains(c)), "{dark}: {fence:?}");
            assert!(!fence.contains(&color(c.code_text)), "{dark}");
            let muted = color(c.text_muted);
            let closing = &colors[source.rfind("```").unwrap()..][..3];
            assert!(closing.iter().all(|c| *c == muted), "{dark}: {closing:?}");
            assert_eq!(
                colors_of(source, &colors, "`y`"),
                vec![color(c.code_text); 3]
            );
            assert_eq!(colors_of(source, &colors, "*a*")[0], muted);
        }
    }

    /// Backticks in separate list items don't pair up into one code span.
    #[test]
    fn code_spans_stay_inside_their_list_item() {
        ensure_registered();
        let source = "- `one\n- two` and\n- `three`\n";
        let colors = colors(source, true);
        let code = Some(Hsla::from(DARK.code_text));
        assert!(!colors_of(source, &colors, "one").contains(&code));
        assert!(!colors_of(source, &colors, "two").contains(&code));
        assert_eq!(colors_of(source, &colors, "`three`"), vec![code; 7]);
    }

    /// The edit takes: the library's query no longer combines inline text,
    /// which it would if the directive were spelled differently.
    #[test]
    fn inline_text_is_no_longer_combined() {
        ensure_registered();
        let config = LanguageRegistry::singleton().language("markdown").unwrap();
        assert!(config.injections.contains("markdown_inline"));
        assert!(!config.injections.contains("injection.combined"));
    }
}
