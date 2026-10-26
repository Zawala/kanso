//! Formatting shortcuts in the note's source: ⌘B for bold and the rest of the
//! table below, each the toolbar's [`Format`] on the text editor's selection.
//!
//! The keys are bound in the `NoteBody` context, which only the source pane
//! sets, so they never fire in the title, the search box or any other input.
//! The pane handles each action itself, so a menu item for one is greyed out
//! (and does nothing) wherever the note's text doesn't have the keyboard.
//! `secondary` is ⌘ on macOS and Ctrl elsewhere.
//!
//! Taken elsewhere, so moved: ⇧⌘L (back to the note list), ⇧⌘7 and ⇧⌘8 (go to
//! tab 7 and 8) give way to ⌥⌘9 (task), ⌥⌘7 (numbered) and ⌥⌘8 (bulleted).

use super::*;

/// The key context of the note's source pane.
pub(crate) const FORMAT_CONTEXT: &str = "NoteBody";

/// Declares one action per shortcut, binds its keys, and ties it to the
/// [`Format`] it applies.
macro_rules! format_shortcuts {
    ($($action:ident, $keys:literal, $format:expr;)*) => {
        actions!(kanso, [$($action),*]);

        /// Binds the formatting shortcuts to the note's source pane.
        pub fn bind_format_keys(cx: &mut App) {
            cx.bind_keys([$(KeyBinding::new($keys, $action, Some(FORMAT_CONTEXT))),*]);
        }

        /// The action that applies `format` from the keyboard, when it has one.
        pub(crate) fn format_action(format: Format) -> Option<Box<dyn Action>> {
            $(if format == $format {
                return Some(Box::new($action));
            })*
            None
        }

        /// `pane` with a handler for every formatting action.
        fn format_listeners(pane: Div, cx: &mut Context<NoteEditor>) -> Div {
            pane$(.on_action(cx.listener(|this, _: &$action, window, cx| {
                this.format_key($format, window, cx)
            })))*
        }
    };
}

format_shortcuts! {
    FormatBold, "secondary-b", Format::Bold;
    FormatItalic, "secondary-i", Format::Italic;
    FormatStrikethrough, "secondary-shift-x", Format::Strikethrough;
    FormatCode, "secondary-e", Format::Code;
    FormatLink, "secondary-shift-k", Format::Link;
    FormatTask, "secondary-alt-9", Format::Task;
    FormatBulleted, "secondary-alt-8", Format::Bulleted;
    FormatNumbered, "secondary-alt-7", Format::Numbered;
    FormatQuote, "secondary-shift-.", Format::Quote;
    FormatHeading1, "secondary-alt-1", Format::Heading(1);
    FormatHeading2, "secondary-alt-2", Format::Heading(2);
    FormatHeading3, "secondary-alt-3", Format::Heading(3);
    FormatHeading4, "secondary-alt-4", Format::Heading(4);
    FormatHeading5, "secondary-alt-5", Format::Heading(5);
    FormatHeading6, "secondary-alt-6", Format::Heading(6);
    FormatCodeBlock, "secondary-alt-c", Format::CodeBlock;
}

impl NoteEditor {
    /// A formatting shortcut: the format on the source's selection, as one
    /// undo step. Vim's Normal and Visual modes keep their keys to
    /// themselves, so only Insert mode (or no Vim) takes these.
    // no-prop-test: key glue over `markdown_edit::format`, which has the
    // properties; `formatting_shortcuts_*` press the keys.
    fn format_key(&mut self, format: Format, window: &mut Window, cx: &mut Context<Self>) {
        if !self.body_takes_edits(window, cx) || self.vim_holds_keys() {
            return;
        }
        self.apply_format(format, window, cx);
    }

    /// The source pane's key context and formatting actions on `pane`.
    pub(super) fn with_format_shortcuts(pane: Div, cx: &mut Context<Self>) -> Div {
        format_listeners(pane.key_context(FORMAT_CONTEXT), cx)
    }
}
