//! markdown-rs, guarded against its panics.
//!
//! markdown 1.0.0 (the latest release) panics on some inputs a person can
//! type, such as `"Para\n---\n---\na\n==="` ("Cannot push to non-parent"
//! in `to_mdast`). Every parse in Kanso goes through here: a panic becomes
//! an error, and the caller shows the text as plain paragraphs instead
//! (see [`plain_paragraphs`]), so a note never takes the preview, an export
//! or the app down with it.

use std::panic::{AssertUnwindSafe, catch_unwind};

use markdown::mdast::Node;
use markdown::{Options, ParseOptions};

/// Why a parse gave nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// markdown-rs reported an error (only MDX, which Kanso doesn't use).
    Error(String),
    /// markdown-rs panicked on the text.
    Panicked,
}

/// [`markdown::to_mdast`], with a panic turned into [`Failure::Panicked`].
pub fn to_mdast(text: &str, options: &ParseOptions) -> Result<Node, Failure> {
    match catch_unwind(AssertUnwindSafe(|| markdown::to_mdast(text, options))) {
        Ok(Ok(root)) => Ok(root),
        Ok(Err(message)) => Err(Failure::Error(message.to_string())),
        Err(_) => Err(Failure::Panicked),
    }
}

/// [`markdown::to_html_with_options`], with a panic turned into
/// [`Failure::Panicked`].
pub fn to_html_with_options(text: &str, options: &Options) -> Result<String, Failure> {
    match catch_unwind(AssertUnwindSafe(|| {
        markdown::to_html_with_options(text, options)
    })) {
        Ok(Ok(html)) => Ok(html),
        Ok(Err(message)) => Err(Failure::Error(message.to_string())),
        Err(_) => Err(Failure::Panicked),
    }
}

/// Whether `text` parses with `options` without markdown-rs panicking.
pub fn parses(text: &str, options: &ParseOptions) -> bool {
    !matches!(to_mdast(text, options), Err(Failure::Panicked))
}

/// Markdown that shows `text` as it is, as plain paragraphs: every ASCII
/// punctuation character escaped, lines kept apart with hard breaks and
/// their indentation dropped, so nothing in it is Markdown syntax. What a
/// view shows in place of text markdown-rs can't parse.
pub fn plain_paragraphs(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    let mut lines = text.lines().map(str::trim).peekable();
    while let Some(line) = lines.next() {
        for c in line.chars() {
            if c.is_ascii_punctuation() {
                out.push('\\');
            }
            out.push(c);
        }
        if let Some(next) = lines.peek() {
            // A hard break within a paragraph; a blank line ends one.
            if !line.is_empty() && !next.is_empty() {
                out.push('\\');
            }
            out.push('\n');
        }
    }
    out
}

/// `text` if it parses with `options`, and otherwise [`plain_paragraphs`]
/// of it.
pub fn showable<'a>(text: &'a str, options: &ParseOptions) -> std::borrow::Cow<'a, str> {
    if parses(text, options) {
        text.into()
    } else {
        plain_paragraphs(text).into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Failure, parses, plain_paragraphs, showable, to_html_with_options, to_mdast};
    use markdown::mdast::Node;
    use markdown::{Options, ParseOptions};
    use proptest::prelude::*;

    /// The parse options the preview uses: GFM and math.
    fn options() -> ParseOptions {
        let mut options = ParseOptions::gfm();
        options.constructs.frontmatter = false;
        options.constructs.math_text = true;
        options.constructs.math_flow = true;
        options
    }

    std::thread_local! {
        static QUIET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    /// Runs `f` without printing the panics markdown-rs is expected to
    /// raise on this thread; other threads' panics print as usual.
    fn quietly<R>(f: impl FnOnce() -> R) -> R {
        static INSTALL: std::sync::Once = std::sync::Once::new();
        INSTALL.call_once(|| {
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                if !QUIET.with(std::cell::Cell::get) {
                    previous(info);
                }
            }));
        });
        QUIET.with(|q| q.set(true));
        let out = f();
        QUIET.with(|q| q.set(false));
        out
    }

    /// The input markdown 1.0.0 panics on; kept so a newer release that
    /// fixes it, or a regression here, shows.
    const PANICS: &str = "Para\n---\n---\na\n===";

    #[test]
    fn a_panicking_parse_is_an_error_and_the_text_still_shows() {
        let (parsed, gfm, html) = quietly(|| {
            (
                to_mdast(PANICS, &options()),
                to_mdast(PANICS, &ParseOptions::gfm()),
                to_html_with_options(
                    PANICS,
                    &Options {
                        parse: options(),
                        ..Options::default()
                    },
                ),
            )
        });
        assert_eq!(parsed, Err(Failure::Panicked));
        assert_eq!(gfm, Err(Failure::Panicked));
        assert!(matches!(html, Ok(_) | Err(Failure::Panicked)));
        let shown = quietly(|| showable(PANICS, &options()));
        assert_eq!(shown, "Para\\\n\\-\\-\\-\\\n\\-\\-\\-\\\na\\\n\\=\\=\\=");
        let Ok(Node::Root(root)) = to_mdast(&shown, &options()) else {
            panic!("the fallback parses");
        };
        assert_eq!(root.children.len(), 1, "one paragraph");
        assert!(matches!(root.children[0], Node::Paragraph(_)));
    }

    #[test]
    fn plain_paragraphs_keep_paragraphs_and_lines() {
        assert_eq!(plain_paragraphs("a\nb\n\nc"), "a\\\nb\n\nc");
        assert_eq!(plain_paragraphs("    # x"), "\\# x");
        assert_eq!(plain_paragraphs(""), "");
    }

    /// Markdown-ish text: the characters that make block structure, on a
    /// few short lines.
    fn markdownish() -> impl Strategy<Value = String> {
        let line = prop_oneof![
            Just("---".to_owned()),
            Just("===".to_owned()),
            Just("***".to_owned()),
            Just("$$".to_owned()),
            Just("```".to_owned()),
            Just("".to_owned()),
            "[-=*#>|$`~ a1.)\\[\\]:_+]{0,8}",
            "[a-z ]{1,10}",
        ];
        prop::collection::vec(line, 0..10).prop_map(|lines| lines.join("\n"))
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        /// Safety: parsing never panics out of here, and whatever it can't
        /// parse shows as plain paragraphs that do parse.
        #[test]
        fn prop_parsing_never_panics(text in markdownish()) {
            let (parsed, shown, again) = quietly(|| {
                let shown = showable(&text, &options()).into_owned();
                let again = parses(&shown, &options());
                (to_mdast(&text, &options()), shown, again)
            });
            prop_assert!(matches!(parsed, Ok(_) | Err(Failure::Panicked)));
            prop_assert!(again, "{:?} shows as {:?}, which doesn't parse", text, shown);
        }

        /// Oracle: plain paragraphs read back as the text's words, with no
        /// Markdown in them.
        #[test]
        fn prop_plain_paragraphs_are_plain(text in markdownish()) {
            let plain = plain_paragraphs(&text);
            let Ok(root) = to_mdast(&plain, &options()) else {
                return Err(TestCaseError::fail("plain paragraphs parse"));
            };
            for block in root.children().into_iter().flatten() {
                prop_assert!(matches!(block, Node::Paragraph(_)), "{:?}", block);
            }
        }
    }
}
