//! A note's link, copied to paste into another note: the wiki-link
//! `[[Title]]` that `wikilinks` resolves, or Kanso's own link to the note
//! by ID, `[Title](<ID link>)`, the form `[[` autocomplete writes, which
//! survives renames.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, ClipboardItem, Context, Window};
use kanso_core::links::parse_target;
use kanso_store::Store;

use crate::workspace::Workspace;

/// The wiki-link naming a note titled `title`, or `None` when the title
/// can't be written as one (it is empty, or has a bracket, `#`, `|` or a
/// line break in it). Pure.
pub fn wiki_link(title: &str) -> Option<String> {
    let title = title.trim();
    (!title.contains(['#', '|']) && parse_target(title).is_some()).then(|| format!("[[{title}]]"))
}

/// What to copy for note `id` titled `title`: its wiki-link, unless
/// `kanso` asks for Kanso's own link, or the title can't be a wiki-link,
/// or the title names an older note first (`resolves_to` is the note the
/// title resolves to). Says whether the wiki-link was given way. Pure.
pub fn link_for(id: &str, title: &str, kanso: bool, resolves_to: Option<&str>) -> (String, bool) {
    let wiki = (!kanso)
        .then(|| wiki_link(title))
        .flatten()
        .filter(|_| resolves_to.is_none_or(|other| other == id));
    match wiki {
        Some(link) => (link, false),
        None => (crate::wikilinks::id_link(id, title), !kanso),
    }
}

impl Workspace {
    /// Copies the open note's link: its wiki-link, or with `kanso` its
    /// Kanso link.
    pub(crate) fn copy_note_link(
        &mut self,
        kanso: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.editor.read(cx).note_id().map(str::to_owned) {
            self.copy_link_of(&id, kanso, window, cx);
        }
    }

    /// Copies note `id`'s link, as [`Workspace::copy_note_link`] does.
    pub(crate) fn copy_link_of(
        &mut self,
        id: &str,
        kanso: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.flush(cx);
        copy_link(&self.store, id, kanso, window, cx);
    }
}

/// Copies note `id`'s link from `store`, and says what was copied: the
/// work behind [`Workspace::copy_link_of`], for any window.
pub(crate) fn copy_link(store: &Store, id: &str, kanso: bool, window: &mut Window, cx: &mut App) {
    let Ok(Some(note)) = store.note(id) else {
        return;
    };
    let resolves_to = store
        .resolve_title(&note.title)
        .ok()
        .flatten()
        .map(|(other, _)| other);
    let (link, instead) = link_for(id, &note.title, kanso, resolves_to.as_deref());
    cx.write_to_clipboard(ClipboardItem::new_string(link));
    let message = if instead {
        if wiki_link(&note.title).is_none() {
            "Copied a Kanso link: the title can't be a [[wiki-link]]."
        } else {
            "Copied a Kanso link: an older note has the same title."
        }
    } else {
        "Link copied."
    };
    window.push_notification(Notification::info(message), cx);
}

#[cfg(test)]
mod tests {
    use kanso_core::links::wiki_links;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    #[test]
    fn a_plain_title_is_a_wiki_link() {
        assert_eq!(wiki_link("Plans"), Some("[[Plans]]".into()));
        assert_eq!(wiki_link("  Weekly plan "), Some("[[Weekly plan]]".into()));
    }

    #[test]
    fn titles_a_wiki_link_cant_hold_have_none() {
        for title in [
            "",
            "  ",
            "C# notes",
            "a | b",
            "[draft]",
            "x]]y",
            "two\nlines",
        ] {
            assert_eq!(wiki_link(title), None, "{title:?}");
        }
    }

    #[test]
    fn the_kanso_link_names_the_note_by_id() {
        let (link, instead) = link_for("note:abc", "Plans", true, None);
        assert_eq!(link, "[Plans](inkdrop://note/abc)");
        assert!(!instead);
    }

    #[test]
    fn a_title_an_older_note_has_copies_the_kanso_link() {
        let (link, instead) = link_for("note:new", "Plans", false, Some("note:old"));
        assert_eq!(link, "[Plans](inkdrop://note/new)");
        assert!(instead);
        let (link, instead) = link_for("note:old", "Plans", false, Some("note:old"));
        assert_eq!(link, "[[Plans]]");
        assert!(!instead);
    }

    #[test]
    fn an_untitled_note_copies_the_kanso_link() {
        let (link, instead) = link_for("note:abc", "", false, None);
        assert_eq!(link, "[Untitled](inkdrop://note/abc)");
        assert!(instead);
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// A wiki-link copied for a title reads back, from a note's text,
        /// as one link to that title and nothing else.
        #[test]
        fn a_copied_wiki_link_reads_back_as_its_title(
            title in "[ a-zA-Z0-9#|\\[\\]\\-_.,'!?é]{0,24}",
            before in "[ a-z.]{0,6}",
        ) {
            if let Some(link) = wiki_link(&title) {
                let body = format!("{before}{link} after");
                let links = wiki_links(&body);
                prop_assert_eq!(links.len(), 1);
                prop_assert_eq!(&links[0].title, title.trim());
                prop_assert_eq!(links[0].heading.as_ref(), None);
                prop_assert_eq!(links[0].alias.as_ref(), None);
            } else {
                prop_assert!(
                    title.trim().is_empty() || title.contains(['#', '|', '[', ']']),
                    "a plain title gave no link: {:?}", title
                );
            }
        }

        /// The Kanso link points at the note's ID, whatever its title.
        #[test]
        fn the_kanso_link_points_at_the_id(
            id in "[a-zA-Z0-9_-]{1,16}",
            title in "\\PC{0,24}",
        ) {
            let (link, _) = link_for(&format!("note:{id}"), &title, true, None);
            let target = link.rsplit_once("](").map(|(_, rest)| rest.trim_end_matches(')'));
            prop_assert_eq!(
                target.and_then(crate::preview::linked_note),
                Some(format!("note:{id}"))
            );
        }
    }
}
