//! The formatting shortcuts through the real text editor: the keys reach a
//! focused source and nothing else, each is one undo step, Vim's Normal mode
//! is left alone, and every shortcut is in the menu and the keymap once.

use gpui_kit::{Focusable as _, TestAppContext};

use super::format_keys::{FORMAT_CONTEXT, bind_format_keys, format_action};
use super::writing_tests::Open;
use crate::config::EditorSettings;
use crate::markdown_edit::Format;
use crate::vim::Mode;

fn open_with_keys(cx: &mut TestAppContext, body: &str) -> Open {
    let open = Open::new(cx, body);
    cx.update(bind_format_keys);
    open
}

/// ⌘B wraps "hello" and keeps it selected, ⌘B again unwraps it, and ⌘Z
/// takes each back in one step.
// no-prop-test: drives the editor with keys; `kanso_engine::edit` has the
// properties.
#[gpui_kit::test]
fn bold_wraps_the_selection_and_toggles(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "hello");
    open.select(cx, 0..5);
    open.keys(cx, "secondary-b");
    assert_eq!(open.text(cx), ("**hello**".into(), 2..7));
    open.keys(cx, "secondary-b");
    assert_eq!(open.text(cx), ("hello".into(), 0..5));
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, "**hello**");
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, "hello");
}

/// With nothing selected, ⌘B puts the markers in and the caret between them.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn bold_without_a_selection_puts_the_caret_between(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "");
    open.keys(cx, "secondary-b");
    assert_eq!(open.text(cx), ("****".into(), 2..2));
    open.input(cx, "x");
    assert_eq!(open.text(cx).0, "**x**");
}

/// ⇧⌘L's replacement, ⌥⌘9, makes both selected lines tasks, and again
/// turns them back.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn task_on_two_lines(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "one\ntwo");
    open.select(cx, 0..7);
    open.keys(cx, "secondary-alt-9");
    assert_eq!(open.text(cx).0, "- [ ] one\n- [ ] two");
    open.keys(cx, "secondary-alt-9");
    assert_eq!(open.text(cx).0, "one\ntwo");
}

/// ⇧⌘K wraps the selection as a link with the address selected, and with
/// nothing selected puts an empty link in with the caret in its text.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn link_selects_the_address(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "see docs");
    open.select(cx, 4..8);
    open.keys(cx, "secondary-shift-k");
    assert_eq!(open.text(cx), ("see [docs](https://)".into(), 11..19));
    // ⌘K stays the command palette's: ⇧⌘K is the link's alone.
    let open = open_with_keys(cx, "");
    open.keys(cx, "secondary-shift-k");
    assert_eq!(open.text(cx), ("[](https://)".into(), 1..1));
}

/// Each shortcut does what its toolbar button does, on "word".
// no-prop-test: drives the editor with keys; `kanso_engine::edit` has the
// properties.
#[gpui_kit::test]
fn every_shortcut_formats(cx: &mut TestAppContext) {
    let cases = [
        ("secondary-b", "**word**"),
        ("secondary-i", "_word_"),
        ("secondary-shift-x", "~~word~~"),
        ("secondary-e", "`word`"),
        ("secondary-shift-k", "[word](https://)"),
        ("secondary-alt-9", "- [ ] word"),
        ("secondary-alt-8", "- word"),
        ("secondary-alt-7", "1. word"),
        ("secondary-shift-.", "> word"),
        ("secondary-alt-1", "# word"),
        ("secondary-alt-2", "## word"),
        ("secondary-alt-3", "### word"),
        ("secondary-alt-4", "#### word"),
        ("secondary-alt-5", "##### word"),
        ("secondary-alt-6", "###### word"),
        ("secondary-alt-c", "```\nword\n```"),
    ];
    for (keys, expected) in cases {
        let open = open_with_keys(cx, "word");
        open.select(cx, 0..4);
        open.keys(cx, keys);
        assert_eq!(open.text(cx).0, expected, "{keys}");
        // One undo step takes it back.
        open.keys(cx, "cmd-z");
        assert_eq!(open.text(cx).0, "word", "{keys} undone");
    }
}

/// A heading shortcut on the cursor's line replaces another level's and
/// toggles off at its own.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn headings_switch_and_toggle(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "## word");
    open.select(cx, 5..5);
    open.keys(cx, "secondary-alt-3");
    assert_eq!(open.text(cx).0, "### word");
    open.keys(cx, "secondary-alt-3");
    assert_eq!(open.text(cx).0, "word");
}

/// The keys are the source's alone: in the title, ⌘B changes neither the
/// title nor the body.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn the_title_does_not_take_them(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "hello");
    open.select(cx, 0..5);
    open.with(cx, |editor, window, cx| editor.focus_title(window, cx));
    open.keys(cx, "secondary-b secondary-i secondary-alt-1");
    assert_eq!(open.text(cx).0, "hello");
    let title = open.with(cx, |editor, _, cx| {
        editor.title.read(cx).value().to_string()
    });
    assert_eq!(title, "t");
}

/// Vim's Normal mode keeps its keys; Insert mode takes the shortcuts.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn vim_normal_mode_is_not_affected(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "hello");
    open.with(cx, |editor, _, cx| {
        editor.set_settings(
            EditorSettings {
                vim: true,
                ..EditorSettings::default()
            },
            cx,
        )
    });
    open.select(cx, 0..0);
    assert_eq!(open.with(cx, |e, _, _| e.vim_mode()), Some(Mode::Normal));
    open.keys(cx, "secondary-b secondary-alt-9 secondary-e");
    assert_eq!(open.text(cx).0, "hello");
    assert_eq!(open.with(cx, |e, _, _| e.vim_mode()), Some(Mode::Normal));

    open.keys(cx, "i");
    assert_eq!(open.with(cx, |e, _, _| e.vim_mode()), Some(Mode::Insert));
    open.select(cx, 0..5);
    open.keys(cx, "secondary-b");
    assert_eq!(open.text(cx).0, "**hello**");
}

/// A note shared to read can't be formatted.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn a_read_only_note_is_left_alone(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "hello");
    open.with(cx, |editor, _, _| editor.text_read_only = true);
    open.select(cx, 0..5);
    open.keys(cx, "secondary-b");
    assert_eq!(open.text(cx).0, "hello");
}

/// Every format with a shortcut shows it where the text has the keyboard,
/// which is where the toolbar's tooltips and menu look it up.
// no-prop-test: reads the keymap.
#[gpui_kit::test]
fn the_toolbar_finds_each_shortcut(cx: &mut TestAppContext) {
    let open = open_with_keys(cx, "hello");
    open.keys(cx, "secondary-b");
    let formats = [
        Format::Bold,
        Format::Italic,
        Format::Strikethrough,
        Format::Code,
        Format::Link,
        Format::Task,
        Format::Bulleted,
        Format::Numbered,
        Format::Quote,
        Format::Heading(1),
        Format::Heading(2),
        Format::Heading(3),
        Format::Heading(4),
        Format::Heading(5),
        Format::Heading(6),
        Format::CodeBlock,
    ];
    open.with(cx, |editor, window, cx| {
        let focus = editor.body.read(cx).focus_handle(cx);
        for format in formats {
            let action = format_action(format).expect("an action");
            assert!(
                window
                    .highest_precedence_binding_for_action_in(action.as_ref(), &focus)
                    .is_some(),
                "{format:?}"
            );
        }
    });
    // Image, table and rule are buttons without a key.
    assert!(format_action(Format::Image).is_none());
    assert!(format_action(Format::Table).is_none());
    assert!(format_action(Format::Rule).is_none());
}

/// No other binding anywhere in the window uses a formatting shortcut's
/// keys, and each Note › Format menu item has one.
// no-prop-test: reads the keymap.
#[gpui_kit::test]
fn the_keys_are_free_and_in_the_menu(cx: &mut TestAppContext) {
    cx.update(|cx| {
        crate::workspace::bind_keys(cx);
        crate::note_list::bind_keys(cx);
        crate::note_window::bind_keys(cx);
        crate::capture::bind_keys(cx);
        crate::graph_view::bind_keys(cx);
        crate::board_view::bind_keys(cx);
        let keymap = cx.key_bindings();
        let keymap = keymap.borrow();
        let mine: Vec<_> = keymap
            .bindings()
            .filter(|binding| {
                binding
                    .predicate()
                    .is_some_and(|p| p.to_string() == FORMAT_CONTEXT)
            })
            .collect();
        assert_eq!(mine.len(), 16);
        let keys = |binding: &gpui_kit::KeyBinding| {
            binding
                .keystrokes()
                .iter()
                .map(|k| k.inner().unparse())
                .collect::<Vec<_>>()
        };
        for binding in &mine {
            let others: Vec<_> = keymap
                .bindings()
                .filter(|other| {
                    !other.action().partial_eq(binding.action())
                        && keys(other) == keys(binding)
                        // Bindings for one view don't meet the text's.
                        && other.predicate().is_none_or(|p| p.to_string() == FORMAT_CONTEXT)
                })
                .map(|other| other.action().name())
                .collect();
            assert!(
                others.is_empty(),
                "{:?} also runs {others:?}",
                keys(binding)
            );
        }
        for item in crate::workspace::format_menu() {
            if let gpui_kit::MenuItem::Action { action, name, .. } = item {
                assert!(
                    mine.iter().any(|b| b.action().partial_eq(action.as_ref())),
                    "{name} has no shortcut"
                );
            }
        }
    });
}
