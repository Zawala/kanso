//! Vim keys through the real text editor: keys, the writing aids in Insert
//! mode, undo, the clipboard and the footer.

use gpui_kit::{ClipboardItem, TestAppContext};

use super::writing_tests::Open;
use crate::config::EditorSettings;
use crate::vim::Mode;

fn vim_on(cx: &mut TestAppContext, body: &str) -> Open {
    let open = Open::new(cx, body);
    set_vim(&open, cx, true);
    open.select(cx, 0..0);
    open
}

fn set_vim(open: &Open, cx: &mut TestAppContext, on: bool) {
    open.with(cx, |editor, _, cx| {
        editor.set_settings(
            EditorSettings {
                vim: on,
                ..EditorSettings::default()
            },
            cx,
        )
    });
}

fn mode(open: &Open, cx: &mut TestAppContext) -> Option<Mode> {
    open.with(cx, |editor, _, _| editor.vim_mode())
}

/// What the footer shows: the mode's name and the command being typed.
fn footer(open: &Open, cx: &mut TestAppContext) -> Option<(String, String)> {
    open.with(cx, |editor, _, cx| {
        editor
            .footer
            .read(cx)
            .vim_shown()
            .map(|status| (status.mode.label().to_owned(), status.pending.clone()))
    })
}

/// Normal mode moves and changes the text with its keys and types
/// nothing; `i` types, Escape steps back; the footer follows.
// no-prop-test: drives the editor with keys; `vim` has the properties.
#[gpui_kit::test]
fn normal_and_insert_modes(cx: &mut TestAppContext) {
    let open = vim_on(cx, "foo bar\nbaz");
    assert_eq!(mode(&open, cx), Some(Mode::Normal));
    assert_eq!(footer(&open, cx), Some(("NORMAL".into(), String::new())));
    open.keys(cx, "w");
    assert_eq!(open.text(cx), ("foo bar\nbaz".into(), 4..4));
    open.keys(cx, "d");
    assert_eq!(footer(&open, cx), Some(("NORMAL".into(), "d".into())));
    open.keys(cx, "w");
    assert_eq!(open.text(cx), ("foo \nbaz".into(), 3..3));
    open.keys(cx, "j $");
    assert_eq!(open.text(cx), ("foo \nbaz".into(), 7..7));
    // Letters Vim has no use for type nothing.
    open.keys(cx, "q z shift-q");
    assert_eq!(open.text(cx).0, "foo \nbaz");

    open.keys(cx, "i");
    assert_eq!(footer(&open, cx), Some(("INSERT".into(), String::new())));
    open.input(cx, "X");
    assert_eq!(open.text(cx), ("foo \nbaXz".into(), 8..8));
    open.keys(cx, "escape");
    assert_eq!(mode(&open, cx), Some(Mode::Normal));
    assert_eq!(open.text(cx), ("foo \nbaXz".into(), 7..7));

    // Turned off: keys type again, and the footer shows no mode.
    set_vim(&open, cx, false);
    open.input(cx, "j");
    assert_eq!(open.text(cx).0, "foo \nbajXz");
    assert_eq!(footer(&open, cx), None);
}

/// A change with what was typed for it, Enters and smart edits included,
/// is one undo step for `u` and ⌘Z alike, and Ctrl-R brings it back.
// no-prop-test: drives the editor's undo with keys.
#[gpui_kit::test]
fn one_change_is_one_undo_step(cx: &mut TestAppContext) {
    let open = vim_on(cx, "one two\nthree");
    open.keys(cx, "c w");
    assert_eq!(open.text(cx), (" two\nthree".into(), 0..0));
    open.input(cx, "uno ()");
    open.keys(cx, "enter");
    open.input(cx, "dos");
    open.keys(cx, "backspace");
    open.input(cx, "s");
    open.keys(cx, "escape");
    let changed = "uno ()\ndos two\nthree";
    assert_eq!(open.text(cx).0, changed);
    open.keys(cx, "u");
    assert_eq!(open.text(cx), ("one two\nthree".into(), 0..0));
    open.keys(cx, "ctrl-r");
    assert_eq!(open.text(cx).0, changed);
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, "one two\nthree");
    open.keys(cx, "cmd-shift-z");
    assert_eq!(open.text(cx).0, changed);

    // `.` makes the same change again, one more step.
    open.keys(cx, "g g j w .");
    assert_eq!(open.text(cx).0, "uno ()\ndos uno ()\ndos\nthree");
    open.keys(cx, "u");
    assert_eq!(open.text(cx).0, changed);

    // Normal mode changes are a step each.
    open.keys(cx, "g g x x u");
    assert_eq!(open.text(cx).0, "no ()\ndos two\nthree");
    open.keys(cx, "u");
    assert_eq!(open.text(cx).0, changed);

    // A count, and typing that comes back to where it began.
    open.keys(cx, "shift-g shift-a");
    open.input(cx, "!");
    open.keys(cx, "backspace");
    open.input(cx, "?");
    open.keys(cx, "escape");
    assert_eq!(open.text(cx).0, "uno ()\ndos two\nthree?");
    open.keys(cx, "u");
    assert_eq!(open.text(cx).0, changed);
    open.keys(cx, "3 shift-a");
    open.input(cx, "ab");
    open.keys(cx, "escape");
    assert_eq!(open.text(cx).0, "uno ()\ndos two\nthreeababab");
    open.keys(cx, "u");
    assert_eq!(open.text(cx).0, changed);
}

/// The writing aids work in Insert mode: pairing, Tab in a list, and
/// paste.
// no-prop-test: drives the editor with keys; the aids have their own
// properties.
#[gpui_kit::test]
fn writing_aids_in_insert_mode(cx: &mut TestAppContext) {
    let open = vim_on(cx, "- a\n- b");
    open.keys(cx, "j shift-a");
    open.keys(cx, "tab");
    assert_eq!(open.text(cx).0, "- a\n  - b");
    open.input(cx, " (");
    assert_eq!(open.text(cx).0, "- a\n  - b ()");
    open.keys(cx, "escape");
    assert_eq!(mode(&open, cx), Some(Mode::Normal));
    // One step takes the indent and the pair back.
    open.keys(cx, "u");
    assert_eq!(open.text(cx).0, "- a\n- b");
}

/// `y` puts the text on the clipboard; `p` puts what was yanked, or what
/// was copied elsewhere since; deletes leave the clipboard alone.
// no-prop-test: drives the editor and the test clipboard.
#[gpui_kit::test]
fn the_register_and_the_clipboard(cx: &mut TestAppContext) {
    let open = vim_on(cx, "alpha\nbeta");
    open.keys(cx, "y y");
    let copied = cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some("alpha\n"));
    open.keys(cx, "p");
    assert_eq!(open.text(cx).0, "alpha\nalpha\nbeta");
    open.keys(cx, "shift-g d d");
    let copied = cx.update(|cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some("alpha\n"), "a delete keeps it");
    open.keys(cx, "shift-p");
    assert_eq!(open.text(cx).0, "alpha\nbeta\nalpha");
    cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string("gamma".into())));
    open.keys(cx, "g g p");
    assert_eq!(open.text(cx).0, "agammalpha\nbeta\nalpha");
}

/// Visual mode selects as the editor selects; `d` takes it. Visual Line
/// takes whole lines.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn visual_modes_select_in_the_editor(cx: &mut TestAppContext) {
    let open = vim_on(cx, "one two\nthree");
    open.keys(cx, "v e");
    assert_eq!(open.text(cx).1, 0..3);
    assert_eq!(footer(&open, cx), Some(("VISUAL".into(), String::new())));
    open.keys(cx, "d");
    assert_eq!(open.text(cx), (" two\nthree".into(), 0..0));
    open.keys(cx, "shift-v");
    assert_eq!(open.text(cx).1, 0..4);
    assert_eq!(footer(&open, cx), Some(("V-LINE".into(), String::new())));
    open.keys(cx, "j d");
    assert_eq!(open.text(cx).0, "");
    assert_eq!(mode(&open, cx), Some(Mode::Normal));

    // A click elsewhere ends Visual mode: the next key starts from it.
    open.with(cx, |editor, window, cx| {
        editor.edit_for_test("abc def", window, cx)
    });
    open.keys(cx, "v l");
    open.select(cx, 5..5);
    open.keys(cx, "x");
    assert_eq!(open.text(cx).0, "abc df");
}

/// ⌘ shortcuts reach the app and the editor in Normal mode; `:w` saves.
// no-prop-test: drives the editor with keys.
#[gpui_kit::test]
fn shortcuts_and_ex_commands(cx: &mut TestAppContext) {
    let open = vim_on(cx, "abc");
    open.keys(cx, "x");
    open.keys(cx, "cmd-z");
    assert_eq!(open.text(cx).0, "abc");
    open.keys(cx, "x : w enter");
    let saved = open.store.note(&open.id).unwrap().unwrap().body;
    assert_eq!(saved, "bc");
    assert_eq!(mode(&open, cx), Some(Mode::Normal));
    // Ex commands Kanso has no use for do nothing.
    open.keys(cx, ": s e t space n u enter");
    assert_eq!(open.text(cx).0, "bc");
    assert_eq!(footer(&open, cx), Some(("NORMAL".into(), String::new())));
}
