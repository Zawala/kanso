//! A note's link, copied to paste into another note: the wiki-link
//! `[[Title]]` that `wikilinks` resolves, or Kanso's own link to the note
//! by ID, `[Title](<ID link>)`, the form `[[` autocomplete writes, which
//! survives renames. What to copy is decided in `kanso_engine::links`.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, ClipboardItem, Context, Window};
use kanso_engine::links::{copy_message, link_for};
use kanso_store::Store;

use crate::workspace::Workspace;

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
    let message = copy_message(&note.title, instead);
    window.push_notification(Notification::info(message), cx);
}
