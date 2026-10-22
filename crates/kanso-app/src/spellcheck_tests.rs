//! Spelling through the real editor, with a word list standing in for the
//! system's dictionary: the underlines, the context menu's items and what
//! each of them does, and the setting.

use std::sync::Arc;

use gpui_kit::{Action, AppContext as _, TestAppContext};

use super::writing_tests::Open;
use crate::config::EditorSettings;
use crate::spellcheck::{self, CHECK_DELAY, WordList};

const WORDS: &[&str] = &["the", "cat", "car", "cot", "sat", "on", "mat", "code"];

/// A note open with `body`, checked with [`WORDS`].
fn open(cx: &mut TestAppContext, body: &str) -> Open {
    cx.update(|cx| spellcheck::set_dictionary(Arc::new(WordList::new(WORDS)), cx));
    let open = Open::new(cx, body);
    cx.run_until_parked();
    open
}

fn marks(open: &Open, cx: &mut TestAppContext) -> usize {
    open.with(cx, |editor, _, _| editor.spelling_marks_for_test())
}

/// The spelling items of the context menu with the caret at `at`.
fn menu_at(open: &Open, cx: &mut TestAppContext, at: usize) -> Vec<(String, Box<dyn Action>)> {
    open.select(cx, at..at);
    open.with(cx, |editor, _, cx| editor.spelling_menu_for_test(cx))
}

fn labels(items: &[(String, Box<dyn Action>)]) -> Vec<&str> {
    items.iter().map(|(label, _)| label.as_str()).collect()
}

/// Picks the item labeled `label`, as a click in the menu would.
fn pick(open: &Open, cx: &mut TestAppContext, items: Vec<(String, Box<dyn Action>)>, label: &str) {
    let (_, action) = items.into_iter().find(|(l, _)| l == label).unwrap();
    cx.update_window(open.window, |_, window, cx| {
        window.dispatch_action(action, cx)
    })
    .unwrap();
    cx.run_until_parked();
}

/// Misspelled prose is underlined and code isn't; the menu on one offers
/// its guesses, Learn Spelling and Ignore Spelling, and a correct word
/// offers none. A guess replaces the word, Ignore and Learn take the
/// underline off, and new typing is checked once it pauses.
// no-prop-test: drives the editor; `spellcheck` has the properties.
#[gpui_kit::test]
fn context_menu_corrects_learns_and_ignores(cx: &mut TestAppContext) {
    let open = open(cx, "The cqt sat on teh mat\n\n`zzzq` code\n");
    assert_eq!(marks(&open, cx), 2);

    assert!(menu_at(&open, cx, 1).is_empty());
    let items = menu_at(&open, cx, 5);
    assert_eq!(
        labels(&items),
        ["cat", "car", "cot", "Learn Spelling", "Ignore Spelling"]
    );
    pick(&open, cx, items, "cot");
    assert_eq!(open.text(cx).0, "The cot sat on teh mat\n\n`zzzq` code\n");
    assert_eq!(marks(&open, cx), 1);

    // Ignored: every "teh" passes, also ones typed later.
    let items = menu_at(&open, cx, 16);
    assert_eq!(labels(&items), ["the", "Learn Spelling", "Ignore Spelling"]);
    pick(&open, cx, items, "Ignore Spelling");
    assert_eq!(marks(&open, cx), 0);
    open.select(cx, 22..22);
    open.input(cx, " teh blorp");
    cx.executor().advance_clock(CHECK_DELAY);
    cx.run_until_parked();
    assert_eq!(marks(&open, cx), 1);

    // Learned: the dictionary has it now.
    let items = menu_at(&open, cx, 29);
    pick(&open, cx, items, "Learn Spelling");
    assert_eq!(marks(&open, cx), 0);
    assert!(cx.update(|cx| spellcheck::speller(cx).unwrap().knows("blorp")));
    open.select(cx, 0..0);
    open.input(cx, "blorp ");
    cx.executor().advance_clock(CHECK_DELAY);
    cx.run_until_parked();
    assert_eq!(marks(&open, cx), 0);
}

/// Turned off in Settings the underlines go, and back on they return.
// no-prop-test: drives the editor; `spellcheck` has the properties.
#[gpui_kit::test]
fn the_setting_turns_it_off_and_on(cx: &mut TestAppContext) {
    let open = open(cx, "teh cat\n");
    assert_eq!(marks(&open, cx), 1);
    let set = |cx: &mut TestAppContext, on: bool| {
        open.with(cx, |editor, _, cx| {
            editor.set_settings(
                EditorSettings {
                    spellcheck: on,
                    ..EditorSettings::default()
                },
                cx,
            )
        });
        cx.run_until_parked();
    };
    set(cx, false);
    assert_eq!(marks(&open, cx), 0);
    assert!(menu_at(&open, cx, 1).is_empty());
    open.select(cx, 7..7);
    open.input(cx, " zzq");
    cx.executor().advance_clock(CHECK_DELAY);
    cx.run_until_parked();
    assert_eq!(marks(&open, cx), 0);
    set(cx, true);
    assert_eq!(marks(&open, cx), 2);
}
