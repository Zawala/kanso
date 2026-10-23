//! The notes open in the main window, kept as a row of tabs above the
//! editor: which notes are open, which one shows, and each one's own
//! back-and-forward history and place in its text.
//!
//! The bar is drawn by `tab_bar`; the workspace keeps this model in step
//! with the store and the editor. Tabs live only in the main window: a note
//! in a window of its own (`note_window`) has none.

use gpui_kit::{App, KeyBinding, actions};

use crate::models::note_visit::NotePlace;
use crate::nav_history::NavHistory;
use crate::workspace::Workspace;

actions!(
    kanso,
    [
        CloseTab,
        NextTab,
        PreviousTab,
        ReopenClosedTab,
        GoToTab1,
        GoToTab2,
        GoToTab3,
        GoToTab4,
        GoToTab5,
        GoToTab6,
        GoToTab7,
        GoToTab8,
        GoToTab9,
    ]
);

/// How many closed tabs can be reopened, the newest first.
pub const MOST_CLOSED: usize = 25;

/// Everything about a tab that survives the tab being closed and the note
/// changing elsewhere: its note, the title shown, and whether its text is
/// sealed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabMeta {
    pub id: String,
    pub title: String,
    pub locked: bool,
}

impl TabMeta {
    /// The title as the bar shows it.
    pub fn label(&self) -> &str {
        if self.title.trim().is_empty() {
            "Untitled"
        } else {
            &self.title
        }
    }
}

/// One open note, with its own history and the place the reader left in it.
#[derive(Debug)]
pub struct Tab {
    pub meta: TabMeta,
    pub nav: NavHistory,
    /// Where the reader was, and the note it was in: a place kept for a
    /// note the tab no longer shows is not gone back to.
    pub place: Option<(String, NotePlace)>,
}

impl Tab {
    fn new(meta: TabMeta) -> Self {
        let mut nav = NavHistory::default();
        nav.visit(&meta.id);
        Tab {
            meta,
            nav,
            place: None,
        }
    }
}

/// A closed tab, and the place in the row it was closed from.
#[derive(Debug)]
struct Closed {
    meta: TabMeta,
    at: usize,
}

/// The open tabs, the one showing, and the ones closed that can be
/// reopened. Empty when no note is open at all; a tab is never blank.
#[derive(Debug, Default)]
pub struct Tabs {
    open: Vec<Tab>,
    active: usize,
    closed: Vec<Closed>,
}

impl Tabs {
    pub fn len(&self) -> usize {
        self.open.len()
    }

    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }

    /// The index of the tab showing, whenever one is open.
    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active_id(&self) -> Option<&str> {
        self.open.get(self.active).map(|tab| tab.meta.id.as_str())
    }

    pub fn get(&self, index: usize) -> Option<&Tab> {
        self.open.get(index)
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.open.iter().position(|tab| tab.meta.id == id)
    }

    pub fn ids(&self) -> Vec<String> {
        self.open.iter().map(|tab| tab.meta.id.clone()).collect()
    }

    pub fn metas(&self) -> Vec<TabMeta> {
        self.open.iter().map(|tab| tab.meta.clone()).collect()
    }

    /// The tabs as they were kept in the settings: the first of any repeated
    /// note only, with `active` clamped to one that exists.
    pub fn restore(metas: impl IntoIterator<Item = (usize, TabMeta)>, active: usize) -> Tabs {
        let mut tabs = Tabs::default();
        for (_, meta) in metas {
            if tabs.index_of(&meta.id).is_some() {
                continue;
            }
            tabs.open.push(Tab::new(meta));
        }
        if !tabs.open.is_empty() {
            tabs.active = active.min(tabs.open.len() - 1);
        }
        tabs
    }

    /// Opens `meta` in the tab showing, or switches to it when it is open
    /// already.
    pub fn open(&mut self, meta: TabMeta) {
        if let Some(index) = self.index_of(&meta.id) {
            self.active = index;
            return;
        }
        if self.open.is_empty() {
            self.open.push(Tab::new(meta));
            self.active = 0;
        } else {
            // The tab keeps its back-and-forward trail and the new note is
            // a visit on it, as a browser keeps a tab's history.
            let tab = &mut self.open[self.active];
            tab.meta = meta;
            tab.nav.visit(&tab.meta.id);
            tab.place = None;
        }
    }

    /// Opens `meta` in a new tab after the one showing, or switches to it
    /// when it is open already.
    pub fn open_in_new(&mut self, meta: TabMeta) {
        if let Some(index) = self.index_of(&meta.id) {
            self.active = index;
            return;
        }
        let at = if self.open.is_empty() {
            0
        } else {
            self.active + 1
        };
        self.open.insert(at, Tab::new(meta));
        self.active = at;
    }

    /// Shows the tab at `index`, if there is one.
    pub fn activate(&mut self, index: usize) -> bool {
        if index >= self.open.len() {
            return false;
        }
        self.active = index;
        true
    }

    /// The note's title or lock changed elsewhere.
    pub fn update_note(&mut self, id: &str, title: String, locked: bool) {
        if let Some(tab) = self.open.iter_mut().find(|tab| tab.meta.id == id) {
            tab.meta.title = title;
            tab.meta.locked = locked;
        }
    }

    /// The note was trashed or deleted elsewhere: its tab goes, and it is
    /// not kept to reopen.
    pub fn remove_note(&mut self, id: &str) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        self.open.remove(index);
        if self.open.is_empty() {
            self.active = 0;
        } else if index < self.active {
            self.active -= 1;
        } else if index == self.active {
            self.active = self.active.min(self.open.len() - 1);
        }
        true
    }

    /// Closes the tab at `index`, unless it is the only one: someone who
    /// wants no notes open closes them by trashing, and there is no empty
    /// tab.
    pub fn close(&mut self, index: usize) -> bool {
        if index >= self.open.len() || self.open.len() < 2 {
            return false;
        }
        let tab = self.open.remove(index);
        self.remember(tab.meta, index);
        if index < self.active {
            self.active -= 1;
        } else if index == self.active {
            self.active = self.active.min(self.open.len() - 1);
        }
        true
    }

    /// Closes every tab but the one at `index`.
    pub fn close_others(&mut self, index: usize) -> bool {
        if index >= self.open.len() || self.open.len() < 2 {
            return false;
        }
        let mut kept = None;
        let mut closed = Vec::new();
        for (at, tab) in self.open.drain(..).enumerate() {
            if at == index {
                kept = Some(tab);
            } else {
                closed.push((tab.meta, at));
            }
        }
        for (meta, at) in closed {
            self.remember(meta, at);
        }
        self.open.extend(kept);
        self.active = 0;
        true
    }

    /// Closes every tab to the right of the one at `index`.
    pub fn close_right(&mut self, index: usize) -> bool {
        if index >= self.open.len() || index + 1 >= self.open.len() {
            return false;
        }
        let closed: Vec<(TabMeta, usize)> = self
            .open
            .drain(index + 1..)
            .enumerate()
            .map(|(offset, tab)| (tab.meta, index + 1 + offset))
            .collect();
        for (meta, at) in closed {
            self.remember(meta, at);
        }
        if self.active > index {
            self.active = index;
        }
        true
    }

    /// Moves the tab at `from` to `to`, both indices in the row; the tab
    /// showing stays showing.
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.open.len() || to >= self.open.len() || from == to {
            return;
        }
        let showing = self.active_id().map(str::to_owned);
        let tab = self.open.remove(from);
        self.open.insert(to, tab);
        self.active = showing
            .and_then(|id| self.index_of(&id))
            .unwrap_or(self.active.min(self.open.len() - 1));
    }

    /// Brings back the newest closed tab whose note `live` says still
    /// exists, at the place it was closed from, and shows it. Closed tabs
    /// for notes gone since are dropped.
    pub fn reopen_closed(&mut self, live: impl Fn(&str) -> bool) -> Option<&TabMeta> {
        while let Some(closed) = self.closed.pop() {
            if !live(&closed.meta.id) || self.index_of(&closed.meta.id).is_some() {
                continue;
            }
            let at = closed.at.min(self.open.len());
            self.open.insert(at, Tab::new(closed.meta));
            self.active = at;
            return self.open.get(at).map(|tab| &tab.meta);
        }
        None
    }

    /// The next tab, wrapping past the last to the first.
    pub fn next(&mut self) -> Option<usize> {
        if self.open.len() < 2 {
            return None;
        }
        self.active = (self.active + 1) % self.open.len();
        Some(self.active)
    }

    /// The tab before, wrapping from the first to the last.
    pub fn previous(&mut self) -> Option<usize> {
        if self.open.len() < 2 {
            return None;
        }
        self.active = (self.active + self.open.len() - 1) % self.open.len();
        Some(self.active)
    }

    /// Shows tab `number` counted from 1; 9 is the last, as in a browser.
    pub fn go_to(&mut self, number: usize) -> Option<usize> {
        if self.open.is_empty() || number == 0 {
            return None;
        }
        let index = if number >= 9 {
            self.open.len() - 1
        } else {
            number - 1
        };
        if index >= self.open.len() {
            return None;
        }
        self.active = index;
        Some(index)
    }

    /// The showing tab's history.
    pub fn nav(&self) -> Option<&NavHistory> {
        self.open.get(self.active).map(|tab| &tab.nav)
    }

    /// The showing tab's history, to add a visit to.
    pub fn nav_mut(&mut self) -> Option<&mut NavHistory> {
        self.open.get_mut(self.active).map(|tab| &mut tab.nav)
    }

    /// Keeps where the reader was in note `id` if that note's tab shows.
    pub fn leave(&mut self, id: &str, place: NotePlace) {
        if let Some(tab) = self.open.get_mut(self.active)
            && tab.meta.id == id
        {
            tab.nav.leave(id, place);
            tab.place = Some((id.to_owned(), place));
        }
    }

    /// Takes the place kept for the tab at `index`, to go back to, when it
    /// was kept in the note the tab shows now.
    pub fn take_place(&mut self, index: usize) -> Option<NotePlace> {
        let tab = self.open.get_mut(index)?;
        match tab.place.take() {
            Some((id, place)) if id == tab.meta.id => Some(place),
            _ => None,
        }
    }

    fn remember(&mut self, meta: TabMeta, at: usize) {
        self.closed.push(Closed { meta, at });
        if self.closed.len() > MOST_CLOSED {
            self.closed.remove(0);
        }
    }
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // With one note open this closes the window, but only after its
        // typing is saved (see `close_current_tab`); a note's own window
        // keeps its ⌘W (close the window).
        KeyBinding::new("secondary-w", CloseTab, None),
        KeyBinding::new("ctrl-tab", NextTab, None),
        KeyBinding::new("ctrl-shift-tab", PreviousTab, None),
        KeyBinding::new("secondary-shift-t", ReopenClosedTab, None),
        // ⌘1 to ⌘3 show the editor, split and preview, so the tabs take the
        // nearest free row of digits.
        KeyBinding::new("secondary-shift-1", GoToTab1, None),
        KeyBinding::new("secondary-shift-2", GoToTab2, None),
        KeyBinding::new("secondary-shift-3", GoToTab3, None),
        KeyBinding::new("secondary-shift-4", GoToTab4, None),
        KeyBinding::new("secondary-shift-5", GoToTab5, None),
        KeyBinding::new("secondary-shift-6", GoToTab6, None),
        KeyBinding::new("secondary-shift-7", GoToTab7, None),
        KeyBinding::new("secondary-shift-8", GoToTab8, None),
        KeyBinding::new("secondary-shift-9", GoToTab9, None),
    ]);
}

// MARK: In the window

impl Workspace {
    /// Closes the showing tab. With one note open it saves that note and
    /// closes the window: `remove_window` does not run the window's close
    /// hook, so the save has to happen here or the last typing is lost.
    pub(crate) fn close_current_tab(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::Context<Self>,
    ) {
        let index = self.tabs.active_index();
        if self.tabs.len() < 2 {
            self.editor.update(cx, |editor, cx| editor.flush(cx));
            window.defer(cx, |window, _| window.remove_window());
            return;
        }
        self.close_tab(index, window, cx);
    }

    /// Shows the next tab, wrapping to the first.
    pub(crate) fn next_tab(&mut self, cx: &mut gpui_kit::Context<Self>) {
        if let Some(index) = self.tabs.next() {
            self.show_tab(index, cx);
        }
    }

    /// Shows the tab before, wrapping to the last.
    pub(crate) fn previous_tab(&mut self, cx: &mut gpui_kit::Context<Self>) {
        if let Some(index) = self.tabs.previous() {
            self.show_tab(index, cx);
        }
    }

    /// Shows tab `number` counted from 1; 9 is the last.
    pub(crate) fn go_to_tab(&mut self, number: usize, cx: &mut gpui_kit::Context<Self>) {
        if let Some(index) = self.tabs.go_to(number) {
            self.show_tab(index, cx);
        }
    }

    /// Brings back the last closed tab, if its note still exists.
    pub(crate) fn reopen_closed_tab(&mut self, cx: &mut gpui_kit::Context<Self>) {
        let store = self.store.clone();
        let reopened = self
            .tabs
            .reopen_closed(|id| matches!(store.note(id), Ok(Some(note)) if !note.is_trashed()))
            .map(|meta| meta.id.clone());
        let Some(id) = reopened else {
            return;
        };
        if let Ok(Some(note)) = self.store.note(&id) {
            let locked = note.is_locked();
            self.tabs.update_note(&id, note.title, locked);
        }
        self.show_tab(self.tabs.active_index(), cx);
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    fn meta(id: &str) -> TabMeta {
        TabMeta {
            id: id.to_owned(),
            title: format!("Note {id}"),
            locked: false,
        }
    }

    fn ids(tabs: &Tabs) -> Vec<&str> {
        tabs.open.iter().map(|tab| tab.meta.id.as_str()).collect()
    }

    #[test]
    fn opening_a_note_in_a_new_tab_puts_it_after_the_showing_one() {
        let mut tabs = Tabs::default();
        tabs.open(meta("a"));
        tabs.open_in_new(meta("b"));
        tabs.open_in_new(meta("c"));
        assert_eq!(ids(&tabs), ["a", "b", "c"]);
        assert_eq!(tabs.active_id(), Some("c"));
        tabs.activate(0);
        tabs.open_in_new(meta("d"));
        assert_eq!(ids(&tabs), ["a", "d", "b", "c"]);
        assert_eq!(tabs.active_id(), Some("d"));
    }

    #[test]
    fn opening_a_note_already_open_switches_to_it() {
        let mut tabs = Tabs::default();
        tabs.open(meta("a"));
        tabs.open_in_new(meta("b"));
        tabs.open(meta("a"));
        assert_eq!(ids(&tabs), ["a", "b"], "no note in two tabs");
        assert_eq!(tabs.active_id(), Some("a"));
        tabs.open(meta("c"));
        assert_eq!(ids(&tabs), ["c", "b"], "the showing tab takes the note");
        assert_eq!(tabs.active_id(), Some("c"));
    }

    #[test]
    fn closing_the_only_tab_is_not_allowed() {
        let mut tabs = Tabs::default();
        tabs.open(meta("a"));
        assert!(!tabs.close(0));
        assert_eq!(ids(&tabs), ["a"]);
    }

    #[test]
    fn closing_shows_the_neighbour() {
        let mut tabs = Tabs::default();
        for id in ["a", "b", "c"] {
            tabs.open_in_new(meta(id));
        }
        assert_eq!(tabs.active_id(), Some("c"));
        tabs.close(2);
        assert_eq!(ids(&tabs), ["a", "b"]);
        assert_eq!(tabs.active_id(), Some("b"));
        tabs.close(0);
        assert_eq!(ids(&tabs), ["b"]);
        assert_eq!(tabs.active_id(), Some("b"));
    }

    #[test]
    fn closing_right_and_others_keep_their_tab_showing() {
        let mut tabs = Tabs::default();
        for id in ["a", "b", "c", "d"] {
            tabs.open_in_new(meta(id));
        }
        tabs.activate(1);
        assert!(tabs.close_right(1));
        assert_eq!(ids(&tabs), ["a", "b"]);
        assert_eq!(tabs.active_id(), Some("b"));
        assert!(tabs.close_others(1));
        assert_eq!(ids(&tabs), ["b"]);
        assert_eq!(tabs.active_id(), Some("b"));
        assert!(!tabs.close_others(0), "nothing left to close");
    }

    #[test]
    fn a_closed_tab_reopens_where_it_was() {
        let mut tabs = Tabs::default();
        for id in ["a", "b", "c"] {
            tabs.open_in_new(meta(id));
        }
        tabs.activate(1);
        tabs.close(1);
        assert_eq!(ids(&tabs), ["a", "c"]);
        assert_eq!(tabs.active_id(), Some("c"));
        let reopened = tabs.reopen_closed(|_| true).unwrap();
        assert_eq!(reopened.id, "b");
        assert_eq!(ids(&tabs), ["a", "b", "c"]);
        assert_eq!(tabs.active_id(), Some("b"));
    }

    #[test]
    fn a_closed_tab_of_a_gone_note_does_not_reopen() {
        let mut tabs = Tabs::default();
        tabs.open(meta("a"));
        tabs.open_in_new(meta("b"));
        tabs.close(1);
        assert!(tabs.reopen_closed(|id| id == "a").is_none());
        assert_eq!(ids(&tabs), ["a"]);
    }

    #[test]
    fn next_and_previous_wrap_and_nine_is_the_last() {
        let mut tabs = Tabs::default();
        for id in ["a", "b", "c"] {
            tabs.open_in_new(meta(id));
        }
        assert_eq!(tabs.next(), Some(0));
        assert_eq!(tabs.previous(), Some(2));
        assert_eq!(tabs.go_to(2), Some(1));
        assert_eq!(tabs.go_to(9), Some(2));
        assert_eq!(tabs.go_to(4), None);
        assert_eq!(tabs.go_to(0), None);
    }

    #[test]
    fn a_note_removed_from_the_store_closes_its_tab() {
        let mut tabs = Tabs::default();
        for id in ["a", "b", "c"] {
            tabs.open_in_new(meta(id));
        }
        tabs.activate(1);
        assert!(tabs.remove_note("b"));
        assert_eq!(ids(&tabs), ["a", "c"]);
        assert_eq!(tabs.active_id(), Some("c"));
        assert!(tabs.remove_note("a"));
        assert!(tabs.remove_note("c"));
        assert!(tabs.is_empty(), "no blank tab is left behind");
        assert!(!tabs.remove_note("a"));
    }

    #[test]
    fn a_rename_updates_the_title() {
        let mut tabs = Tabs::default();
        tabs.open(meta("a"));
        tabs.update_note("a", "New title".into(), false);
        assert_eq!(tabs.get(0).unwrap().meta.title, "New title");
        tabs.open_in_new(meta("b"));
        tabs.update_note("a", "Elsewhere".into(), false);
        assert_eq!(
            tabs.get(0).unwrap().meta.title,
            "Elsewhere",
            "renaming a tab that isn't showing still lands"
        );
    }

    #[test]
    fn an_empty_title_shows_as_untitled() {
        let tab = TabMeta {
            id: "note:a".into(),
            title: "  ".into(),
            locked: false,
        };
        assert_eq!(tab.label(), "Untitled");
    }

    // MARK: Against a model

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Op {
        Open(u8),
        New(u8),
        Close(usize),
        CloseOthers(usize),
        CloseRight(usize),
        Remove(u8),
        Reorder(usize, usize),
        Next,
        Previous,
        GoTo(usize),
        Reopen,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            4 => (0..8u8).prop_map(Op::Open),
            4 => (0..8u8).prop_map(Op::New),
            3 => (0..8usize).prop_map(Op::Close),
            1 => (0..8usize).prop_map(Op::CloseOthers),
            1 => (0..8usize).prop_map(Op::CloseRight),
            1 => (0..8u8).prop_map(Op::Remove),
            2 => (0..8usize, 0..8usize).prop_map(|(from, to)| Op::Reorder(from, to)),
            1 => Just(Op::Next),
            1 => Just(Op::Previous),
            1 => (0..10usize).prop_map(Op::GoTo),
            2 => Just(Op::Reopen),
        ]
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

        /// Any run of opens, closes, reorders and switches keeps the
        /// showing tab valid and never opens a note twice.
        #[test]
        fn the_tabs_stay_consistent(ops in prop::collection::vec(op(), 0..120)) {
            let mut tabs = Tabs::default();
            let gone: std::collections::BTreeSet<String> = Default::default();
            for op in ops {
                let live = |id: &str| !gone.contains(id);
                match op {
                    Op::Open(n) => {
                        let id = n.to_string();
                        if live(&id) {
                            tabs.open(meta(&id));
                        }
                    }
                    Op::New(n) => {
                        let id = n.to_string();
                        if live(&id) {
                            tabs.open_in_new(meta(&id));
                        }
                    }
                    Op::Close(at) => {
                        tabs.close(at);
                    }
                    Op::CloseOthers(at) => {
                        tabs.close_others(at);
                    }
                    Op::CloseRight(at) => {
                        tabs.close_right(at);
                    }
                    Op::Remove(n) => {
                        tabs.remove_note(&n.to_string());
                    }
                    Op::Reorder(from, to) => tabs.reorder(from, to),
                    Op::Next => {
                        tabs.next();
                    }
                    Op::Previous => {
                        tabs.previous();
                    }
                    Op::GoTo(n) => {
                        tabs.go_to(n);
                    }
                    Op::Reopen => {
                        tabs.reopen_closed(live);
                    }
                }
                if !tabs.is_empty() {
                    prop_assert!(tabs.active_index() < tabs.len());
                    prop_assert!(tabs.active_id().is_some());
                } else {
                    prop_assert_eq!(tabs.active_index(), 0);
                }
                let mut seen = std::collections::BTreeSet::new();
                for tab in &tabs.open {
                    prop_assert!(seen.insert(tab.meta.id.clone()), "a note is open twice");
                }
            }
        }

        /// Closing a tab and bringing it back leaves the row as it was.
        #[test]
        fn close_and_reopen_restore_the_row(
            count in 2..8usize,
            at in 0..8usize,
            active in 0..8usize,
        ) {
            let mut tabs = Tabs::default();
            for n in 0..count {
                tabs.open_in_new(meta(&n.to_string()));
            }
            let active = active % count;
            tabs.activate(active);
            let at = at % count;
            let before = ids(&tabs).into_iter().map(str::to_owned).collect::<Vec<_>>();
            prop_assume!(tabs.close(at));
            let reopened = tabs.reopen_closed(|_| true);
            prop_assert!(reopened.is_some());
            let after = ids(&tabs).into_iter().map(str::to_owned).collect::<Vec<_>>();
            prop_assert_eq!(before, after);
            prop_assert_eq!(tabs.active_index(), at);
        }

        /// The note showing before a drag is the note showing after it.
        #[test]
        fn reordering_keeps_the_showing_note(
            count in 2..8usize,
            from in 0..8usize,
            to in 0..8usize,
            active in 0..8usize,
        ) {
            let mut tabs = Tabs::default();
            for n in 0..count {
                tabs.open_in_new(meta(&n.to_string()));
            }
            tabs.activate(active % count);
            let showing = tabs.active_id().unwrap().to_owned();
            let mut expected = ids(&tabs).into_iter().map(str::to_owned).collect::<Vec<_>>();
            let from = from % count;
            let to = to % count;
            let moved = expected.remove(from);
            expected.insert(to, moved);
            tabs.reorder(from, to);
            let after = ids(&tabs).into_iter().map(str::to_owned).collect::<Vec<_>>();
            prop_assert_eq!(after, expected);
            prop_assert_eq!(tabs.active_id(), Some(showing.as_str()));
        }

        /// A new tab goes right after the showing one, and a note already
        /// open just shows.
        #[test]
        fn a_new_tab_goes_after_the_showing_one(
            before in prop::collection::vec(0..8u8, 0..6),
            active in 0..6usize,
            next in prop::option::of(0..8u8),
        ) {
            let mut tabs = Tabs::default();
            for n in before {
                tabs.open_in_new(meta(&n.to_string()));
            }
            if tabs.is_empty() {
                tabs.open(meta("0"));
            }
            let active = active % tabs.len();
            tabs.activate(active);
            let at = tabs.active_index();
            if let Some(next) = next.filter(|n| tabs.index_of(&n.to_string()).is_none()) {
                let id = next.to_string();
                tabs.open_in_new(meta(&id));
                prop_assert_eq!(tabs.active_index(), at + 1);
                prop_assert_eq!(tabs.active_id(), Some(id.as_str()));
            }
        }
    }
}
