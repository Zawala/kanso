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
//!
//! The library's queries also leave some markers uncaptured, so they took
//! the text color while every other marker receded: table pipes and
//! delimiter rows, task boxes, and the brackets around links and images.
//! Link reference definitions used capture names no theme styles. Kanso adds
//! the missing captures.
//!
//! Themes can color quotes, rules, task boxes, front matter and HTML blocks
//! apart from other markers, so each gets a capture of its own. Highlight
//! names are a fixed list, so they borrow names no bundled code grammar uses
//! (`theme_file::CAPTURES` maps them to the theme's keys).

use std::sync::Once;

use gpui_kit::component::highlighter::LanguageRegistry;

/// Block markers the library's Markdown query leaves in the text color.
/// Node names are tree-sitter-md 0.5.3's; a misspelled one fails the whole
/// query, which the tests below would catch as missing colors.
const BLOCK_MARKERS: &str = r#"
(pipe_table_header "|" @punctuation.delimiter)
(pipe_table_row "|" @punctuation.delimiter)
(pipe_table_delimiter_row "|" @punctuation.delimiter)
(pipe_table_delimiter_cell) @punctuation.delimiter
[(task_list_marker_checked) (task_list_marker_unchecked)] @enum
(link_reference_definition ":" @punctuation.delimiter)
(link_label ["[" "]"] @punctuation.delimiter)
(thematic_break) @variant
(block_quote) @primary
(minus_metadata) @preproc
(html_block) @tag.doctype
"#;

/// The brackets around links and images, which the inline query leaves in
/// the text color between the link text and its muted destination.
const INLINE_MARKERS: &str = r#"
(inline_link ["[" "]" "(" ")"] @punctuation.delimiter)
(image ["!" "[" "]" "(" ")"] @punctuation.delimiter)
(full_reference_link ["[" "]"] @punctuation.delimiter)
(collapsed_reference_link ["[" "]"] @punctuation.delimiter)
(shortcut_link ["[" "]"] @punctuation.delimiter)
"#;

/// Makes the editor's Markdown parse each paragraph's inline text on its own
/// and mute every marker. Only editors created afterwards pick it up; it does
/// nothing the second time.
pub fn ensure_registered() {
    static DONE: Once = Once::new();
    DONE.call_once(|| {
        let registry = LanguageRegistry::singleton();
        // no-prop-test: fixed edits to the library's queries; the tests below
        // check what they highlight.
        if let Some(mut inline) = registry.language("markdown_inline") {
            inline.highlights = format!("{}\n{INLINE_MARKERS}", inline.highlights).into();
            registry.register("markdown_inline", &inline);
        }
        let Some(mut config) = registry.language("markdown") else {
            return;
        };
        config.injections = config
            .injections
            .replace("(#set! injection.combined)", "")
            .into();
        // Reference definitions' labels and destinations, named the way the
        // inline query names a link's.
        // Rules leave the list markers' group for a capture of their own.
        let highlights = config
            .highlights
            .replace("@text.uri", "@link_uri")
            .replace("@text.reference", "@link_text")
            .replace("  (thematic_break)\n", "");
        config.highlights = format!("{highlights}\n{BLOCK_MARKERS}").into();
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

    /// Table pipes and delimiter rows, task boxes and the brackets around
    /// links, images and reference definitions recede like every other
    /// marker, while link text keeps the link color, in both themes.
    #[test]
    fn table_task_and_link_markers_recede() {
        ensure_registered();
        let source = "| a |\n|:--|\n| b |\n\n- [x] done\n- [ ] todo\n\n\
                      [t](u) ![i](p) [f][r] [s]\n\n[r]: https://e.x\n";
        for (dark, c) in [(true, &DARK), (false, &LIGHT)] {
            let colors = colors(source, dark);
            let muted = Some(Hsla::from(c.text_muted));
            let link = Some(Hsla::from(c.link));
            // `m` marks a muted byte, `l` a link-colored one, and `.` one in
            // the text color.
            let expect = |needle: &str, pattern: &str| {
                let expected: Vec<_> = pattern
                    .bytes()
                    .map(|b| match b {
                        b'm' => muted,
                        b'l' => link,
                        _ => None,
                    })
                    .collect();
                assert_eq!(
                    colors_of(source, &colors, needle),
                    expected,
                    "{dark}: {needle}"
                );
            };
            expect("| a |", "m...m");
            expect("|:--|", "mmmmm");
            expect("| b |", "m...m");
            expect("[x]", "mmm");
            expect("[ ]", "mmm");
            expect("[t](u)", "mlmmmm");
            expect("![i](p)", "mmlmmmm");
            expect("[f][r]", "mlmlll");
            expect("[s]", "mlm");
            expect("[r]: https://e.x", "mlmm.mmmmmmmmmmm");
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
