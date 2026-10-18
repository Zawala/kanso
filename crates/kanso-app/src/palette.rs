//! The command palette (⌘K) and Open Note (⌘P): a search field over the
//! commands, the places in the library and the notes, drawn over the window
//! with the selected entry explained beside the list.
//!
//! It is part of the main window's view rather than a dialog, and a command
//! is sent from the workspace itself, so it reaches the window's own
//! handlers whatever had the keyboard. Closing it, by Escape, a click
//! outside or a command, gives the keyboard back to whatever had it before,
//! and a command that moves the keyboard on purpose (into the search box, or
//! the title of a new note) does so afterwards. When something else takes
//! the keyboard while it is open, such as a shortcut for a new note, it
//! closes.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{ActiveTheme as _, Icon, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{Clause, DocType, Field, Note, NoteStatus};
use kanso_store::{NoteSummary, Origin, Put, SearchParams, Sort, Store};
use serde_json::json;

use crate::commands::{self, Command, Group, Needs, Picker, Run};
use crate::config::ViewMode;
use crate::editor::{NoteEditor, rgb_color};
use crate::fuzzy;
use crate::library::{Filter, Library, RECENT_DAYS, search_params};
use crate::theme::{Colors, muted_tag_color};

actions!(
    palette,
    [SelectNext, SelectPrevious, Confirm, Dismiss, Back]
);

/// Keys in the palette's search field. They are bound after the field's
/// own keys and at the same depth, so they win: the arrows move through the
/// list rather than the text, Enter chooses, Escape closes, and Backspace in
/// an empty field leaves a second list.
pub fn bind_keys(cx: &mut App) {
    let context = Some("Palette > Input");
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Dismiss, context),
        KeyBinding::new("backspace", Back, context),
    ]);
}

/// What the palette lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// ⌘K: commands, places and, once something is typed, notes.
    Commands,
    /// ⌘P: notes, the most recently edited first.
    Notes,
    /// A list a command opened: notebooks, tags or statuses.
    Pick(Picker),
}

pub(crate) const PANEL_WIDTH: f32 = 680.;
const LIST_WIDTH: f32 = 380.;
const SEARCH_HEIGHT: f32 = 60.;
const BODY_HEIGHT: f32 = 380.;
const FOOTER_HEIGHT: f32 = 36.;
/// The panel's height: the parts above and their two dividers.
pub(crate) const PANEL_HEIGHT: f32 = SEARCH_HEIGHT + BODY_HEIGHT + FOOTER_HEIGHT + 2.;
const ROW_HEIGHT: f32 = 30.;
/// How many notes a search lists.
const NOTE_LIMIT: usize = 20;
/// How long typing pauses before notes are searched.
const SEARCH_DELAY: Duration = Duration::from_millis(80);
/// A match in a keyword counts a little less than the same match in the label.
const KEYWORD_PENALTY: i64 = 4;

/// What choosing an entry does.
#[derive(Clone)]
enum Target {
    Command(&'static Command),
    /// Show these notes in the note list.
    Show(Filter),
    Note(NoteSummary),
    /// Move the open note into this notebook.
    Book(String),
    /// Add this tag to the open note, or take it off.
    Tag(String),
    /// Create a tag with this name and add it.
    NewTag(String),
    Status(NoteStatus),
}

#[derive(Clone)]
struct Item {
    target: Target,
    label: SharedString,
    icon: IconName,
    /// Indent, for notebooks inside other notebooks.
    depth: usize,
    /// The current choice: the open note's notebook, tags and status, or the
    /// place the note list shows.
    checked: bool,
    /// Muted text at the end of the row, such as a count.
    hint: Option<SharedString>,
}

impl Item {
    fn new(target: Target, label: impl Into<SharedString>, icon: IconName) -> Self {
        Item {
            target,
            label: label.into(),
            icon,
            depth: 0,
            checked: false,
            hint: None,
        }
    }

    fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Identifies the entry across rebuilds, to keep it selected.
    fn key(&self) -> String {
        match &self.target {
            Target::Command(command) => format!("command:{}", command.id),
            Target::Show(filter) => format!("show:{filter:?}"),
            Target::Note(note) => format!("note:{}", note.id),
            Target::Book(id) => format!("book:{id}"),
            Target::Tag(id) => format!("tag:{id}"),
            Target::NewTag(name) => format!("new-tag:{name}"),
            Target::Status(status) => format!("status:{}", status.as_str()),
        }
    }
}

/// A heading and its entries, each with how well it fits the query.
struct Section {
    heading: SharedString,
    entries: Vec<(Item, Option<i64>)>,
    /// Keep entries that don't fit the query, after the ones that do: notes
    /// the store found by their text rather than their title.
    keep_unmatched: bool,
}

enum Row {
    Heading(SharedString),
    /// An index into `Palette::items`.
    Item(usize),
    Separator,
}

pub struct Palette {
    store: Arc<Store>,
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    /// The workspace's own focus, always drawn: commands are sent from
    /// here, and the keyboard goes here when nothing else should have it.
    home: FocusHandle,
    /// The note list's focus, for opening a note while the editor shows
    /// only its preview.
    list_focus: FocusHandle,
    input: Entity<InputState>,
    /// `None` while closed.
    mode: Option<Mode>,
    /// Whether the second list was opened from the commands, so Escape goes
    /// back to them. The editor's header opens one straight away; Escape
    /// closes that.
    from_commands: bool,
    /// What had the keyboard before the palette opened.
    restore_focus: Option<FocusHandle>,
    /// The note that was open when the palette opened.
    note: Option<Note>,
    items: Vec<Item>,
    rows: Vec<Row>,
    selected: Option<usize>,
    /// The selected entry, kept while results change under it once the
    /// keyboard or pointer chose it.
    chosen_key: Option<String>,
    /// Notes found for `notes_for`.
    notes: Vec<NoteSummary>,
    notes_for: Option<String>,
    search_task: Option<Task<()>>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl Palette {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        home: FocusHandle,
        list_focus: FocusHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
                InputEvent::Change => this.query_changed(cx),
                // Something else took the keyboard, a shortcut for a new
                // note or the search box say: the palette gives way rather
                // than cover what is now being typed into. Switching apps
                // keeps it.
                InputEvent::Blur if this.mode.is_some() && window.is_window_active() => {
                    this.close(window, cx);
                }
                _ => {}
            }),
            // Notebooks and tags can change while the palette is open, by
            // sync or the API.
            cx.observe(&library, |this, _, cx| {
                if this.mode.is_some() {
                    this.note = this.open_note(cx);
                    this.rebuild(cx);
                }
            }),
        ];
        Palette {
            store,
            library,
            editor,
            home,
            list_focus,
            input,
            mode: None,
            from_commands: false,
            restore_focus: None,
            note: None,
            items: Vec::new(),
            rows: Vec::new(),
            selected: None,
            chosen_key: None,
            notes: Vec::new(),
            notes_for: None,
            search_task: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    #[cfg(test)]
    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }

    #[cfg(test)]
    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// Opens the palette in `mode`, or closes it when it already shows that,
    /// so the shortcut toggles it.
    pub fn open(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == Some(mode) {
            return self.close(window, cx);
        }
        if self.mode.is_none() {
            self.restore_focus = window.focused(cx);
            self.note = self.open_note(cx);
        }
        self.from_commands = false;
        self.show(mode, window, cx);
    }

    /// Whether Escape, or Backspace in the empty field, goes back to the
    /// commands rather than closing.
    fn leads_back(&self) -> bool {
        matches!(self.mode, Some(Mode::Pick(_))) && self.from_commands
    }

    /// Switches to `mode` with an empty field.
    fn show(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = Some(mode);
        self.chosen_key = None;
        self.input.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.set_placeholder(placeholder(mode), window, cx);
            input.focus(window, cx);
        });
        self.search_notes(cx);
        self.rebuild(cx);
        cx.notify();
    }

    /// Closes the palette. The keyboard goes back to what had it before,
    /// unless something else took it meanwhile.
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode.take().is_none() {
            return;
        }
        self.search_task = None;
        self.notes.clear();
        self.notes_for = None;
        self.items.clear();
        self.rows.clear();
        self.selected = None;
        let restore = self.restore_focus.take();
        if self.input.read(cx).focus_handle(cx).is_focused(window) {
            // A handle that is no longer drawn gives way to the workspace
            // once the next frame shows it missing.
            window.focus(restore.as_ref().unwrap_or(&self.home), cx);
        }
        cx.notify();
    }

    /// Selects the first note in the list, for pictures of the detail pane.
    #[cfg(test)]
    pub fn select_first_note(&mut self, cx: &mut Context<Self>) {
        if let Some(ix) = self
            .items
            .iter()
            .position(|item| matches!(item.target, Target::Note(_)))
        {
            self.select(ix, true, cx);
        }
    }

    /// Types `query` into the field, as the user would.
    #[cfg(test)]
    pub fn set_query(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |input, cx| input.replace_all(query, window, cx));
    }

    fn query(&self, cx: &App) -> String {
        self.input.read(cx).value().trim().to_owned()
    }

    fn query_changed(&mut self, cx: &mut Context<Self>) {
        if self.mode.is_none() {
            return;
        }
        self.chosen_key = None;
        self.search_notes(cx);
        self.rebuild(cx);
        cx.notify();
    }

    fn open_note(&self, cx: &App) -> Option<Note> {
        let id = self.editor.read(cx).note_id()?;
        self.store.note(id).ok().flatten()
    }

    /// Searches the store for notes when the mode lists them, in the
    /// background: notes whose titles have the words first, then notes that
    /// have them anywhere. Results for another query never replace the
    /// current one's.
    fn search_notes(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx);
        let wanted = match self.mode {
            Some(Mode::Notes) => true,
            Some(Mode::Commands) => !query.is_empty(),
            _ => false,
        };
        if !wanted {
            self.search_task = None;
            self.notes.clear();
            self.notes_for = None;
            return;
        }
        if self.notes_for.as_deref() == Some(query.as_str()) {
            // Back to the query already shown: a search for another one
            // still on its way would replace it.
            self.search_task = None;
            return;
        }
        let store = self.store.clone();
        self.search_task = Some(cx.spawn(async move |this, cx| {
            if !query.is_empty() {
                cx.background_executor().timer(SEARCH_DELAY).await;
            }
            let mut params = search_params(&Filter::All, &query, Sort::UpdatedAt, true);
            params.limit = Some(NOTE_LIMIT);
            let by_title = title_search(&params);
            let found = cx
                .background_executor()
                .spawn(async move {
                    let titled = match by_title {
                        Some(params) => store.search_summaries(&params)?,
                        None => Vec::new(),
                    };
                    let anywhere = store.search_summaries(&params)?;
                    Ok::<_, kanso_store::StoreError>(merge_by_id(titled, anywhere, NOTE_LIMIT))
                })
                .await;
            this.update(cx, |this, cx| {
                if this.mode.is_none() || this.query(cx) != query {
                    return;
                }
                // A query the store can't run finds nothing.
                this.notes = found.unwrap_or_default();
                this.notes_for = Some(query);
                this.rebuild(cx);
                cx.notify();
            })
            .ok();
        }));
    }

    /// Whether notes for the current query are still on their way.
    fn searching(&self, cx: &App) -> bool {
        self.search_task.is_some() && self.notes_for.as_deref() != Some(self.query(cx).as_str())
    }

    // MARK: Entries

    /// The entries for the current mode and query, before ranking.
    fn sections(&self, query: &str, cx: &App) -> Vec<Section> {
        let section = |heading: &str, entries, keep_unmatched| Section {
            heading: SharedString::from(heading.to_owned()),
            entries,
            keep_unmatched,
        };
        let fit = |item: Item, keywords: &[&str]| {
            let score = item_score(query, &item.label, keywords);
            (item, score)
        };
        match self.mode {
            None => Vec::new(),
            Some(Mode::Commands) => {
                let mut sections: Vec<Section> = [Group::Note, Group::View]
                    .into_iter()
                    .map(|group| section(group.label(), self.commands(group, query), false))
                    .collect();
                let mut places = self.places(query, cx);
                places.extend(self.commands(Group::GoTo, query));
                sections.push(section(Group::GoTo.label(), places, false));
                sections.push(section(
                    Group::App.label(),
                    self.commands(Group::App, query),
                    false,
                ));
                if !query.is_empty() {
                    sections.push(section("Notes", self.note_entries(query, cx), true));
                }
                sections
            }
            Some(Mode::Notes) => {
                let heading = if query.is_empty() {
                    "Recently edited"
                } else {
                    "Notes"
                };
                vec![section(heading, self.note_entries(query, cx), true)]
            }
            Some(Mode::Pick(Picker::Notebook)) => {
                let library = self.library.read(cx);
                let current = self.note.as_ref().map(|n| n.book_id.as_str());
                let entries = library
                    .book_tree()
                    .into_iter()
                    .map(|(depth, book)| {
                        let item = Item::new(
                            Target::Book(book.id.clone()),
                            book.name.clone(),
                            IconName::Notebook,
                        )
                        // Typing flattens the tree into a ranked list.
                        .depth(if query.is_empty() { depth } else { 0 })
                        .checked(current == Some(book.id.as_str()))
                        .hint(book.count.to_string());
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Move to notebook", entries, false)]
            }
            Some(Mode::Pick(Picker::GoToNotebook)) => {
                let library = self.library.read(cx);
                let showing = &library.filter;
                let entries = library
                    .book_tree()
                    .into_iter()
                    .map(|(depth, book)| {
                        let filter = Filter::Book(book.id.clone());
                        let item = Item::new(
                            Target::Show(filter.clone()),
                            book.name.clone(),
                            IconName::Notebook,
                        )
                        .depth(if query.is_empty() { depth } else { 0 })
                        .checked(*showing == filter)
                        .hint(book.count.to_string());
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Notebooks", entries, false)]
            }
            Some(Mode::Pick(Picker::Tag)) => {
                let library = self.library.read(cx);
                let on_note = self.note.as_ref().map_or(&[][..], |n| &n.tags[..]);
                let mut tags: Vec<_> = library.tags.iter().collect();
                tags.sort_by_key(|t| t.name.to_lowercase());
                // Tags show as "#mcp", so "#mcp" typed finds "mcp".
                let name = tag_name(query);
                let mut entries: Vec<_> = tags
                    .iter()
                    .map(|tag| {
                        let item = Item::new(
                            Target::Tag(tag.id.clone()),
                            tag.name.clone(),
                            IconName::Hash,
                        )
                        .checked(on_note.contains(&tag.id))
                        .hint(tag.count.to_string());
                        let score = item_score(name, &item.label, &[]);
                        (item, score)
                    })
                    .collect();
                let exists = tags.iter().any(|t| t.name.eq_ignore_ascii_case(name));
                if !name.is_empty() && !exists {
                    let item = Item::new(
                        Target::NewTag(name.to_owned()),
                        format!("Create tag “{name}”"),
                        IconName::Plus,
                    );
                    // Last, but always there.
                    entries.push((item, Some(i64::MIN / 8)));
                }
                vec![section("Tags", entries, false)]
            }
            Some(Mode::Pick(Picker::Status)) => {
                let current = self.note.as_ref().map(|n| n.status);
                let entries = NoteStatus::ALL
                    .into_iter()
                    .map(|status| {
                        let item = Item::new(
                            Target::Status(status),
                            status_label(status),
                            status_icon(status),
                        )
                        .checked(current == Some(status));
                        fit(item, &[])
                    })
                    .collect();
                vec![section("Status", entries, false)]
            }
        }
    }

    /// The registry's commands in `group` that apply now.
    fn commands(&self, group: Group, query: &str) -> Vec<(Item, Option<i64>)> {
        let note = self.note.as_ref();
        let in_trash = note.is_some_and(|n| n.book_id == TRASH_BOOK_ID);
        commands::all()
            .iter()
            .filter(|command| command.group == group)
            .filter(|command| match command.needs {
                Needs::Nothing => true,
                Needs::Note => note.is_some(),
                Needs::LiveNote => note.is_some() && !in_trash,
            })
            .map(|command| {
                let (label, icon) = match command.label_when_pinned {
                    Some(set) if note.is_some_and(|n| n.pinned) => set,
                    _ => (command.label, command.icon),
                };
                let score = item_score(query, label, command.keywords);
                (Item::new(Target::Command(command), label, icon), score)
            })
            .collect()
    }

    /// The places the note list can show: everything, pinned and recent
    /// notes, each notebook and status, and the trash.
    fn places(&self, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        let library = self.library.read(cx);
        let counts = &library.counts;
        let showing = &library.filter;
        let place = |filter: Filter, label: String, icon, count: u32, keywords: &[&str]| {
            let score = item_score(query, &label, keywords);
            let item = Item::new(Target::Show(filter.clone()), label, icon)
                .checked(*showing == filter)
                .hint(count.to_string());
            (item, score)
        };
        let mut places = vec![
            place(
                Filter::All,
                "All Notes".into(),
                IconName::Inbox,
                counts.all,
                &["everything", "library"],
            ),
            place(
                Filter::Pinned,
                "Pinned".into(),
                IconName::Pin,
                counts.pinned,
                &["starred"],
            ),
            place(
                Filter::Recent,
                "Recent".into(),
                IconName::Clock,
                library.recent,
                &["latest", "edited", "week"],
            ),
        ];
        for (depth, book) in library.book_tree() {
            let (item, score) = place(
                Filter::Book(book.id.clone()),
                book.name.clone(),
                IconName::Notebook,
                book.count,
                &[],
            );
            places.push((item.depth(if query.is_empty() { depth } else { 0 }), score));
        }
        for status in NoteStatus::ALL
            .into_iter()
            .filter(|s| *s != NoteStatus::None)
        {
            let count = counts.by_status.get(&status).copied().unwrap_or(0);
            places.push(place(
                Filter::Status(status),
                status.label().into(),
                status_icon(status),
                count,
                &["status"],
            ));
        }
        places.push(place(
            Filter::Trash,
            "Trash".into(),
            IconName::Trash,
            counts.trash,
            &["deleted"],
        ));
        places
    }

    /// Notes from the store. Until the notes for the current query arrive,
    /// the previous ones stay if their title still fits, so the list doesn't
    /// flicker while typing.
    fn note_entries(&self, query: &str, cx: &App) -> Vec<(Item, Option<i64>)> {
        let current = self.notes_for.as_deref() == Some(query);
        let open = self.note.as_ref().map(|n| n.id.as_str());
        let library = self.library.read(cx);
        self.notes
            .iter()
            .map(|note| {
                let title = note_title(&note.title);
                let score = fuzzy::score(query, &title);
                let mut item = Item::new(Target::Note(note.clone()), title, IconName::FileText)
                    .checked(open == Some(note.id.as_str()));
                if let Some(book) = library.book(&note.book_id) {
                    item = item.hint(book.name.clone());
                }
                (item, score)
            })
            .filter(|(_, score)| current || score.is_some())
            .collect()
    }

    /// Ranks the entries for the query and lays out the rows: with a query,
    /// sections are ordered by their best entry, so the best fit comes first.
    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx);
        let mut sections: Vec<(SharedString, Vec<Item>, Option<i64>)> = Vec::new();
        for section in self.sections(&query, cx) {
            let scores: Vec<Option<i64>> = section.entries.iter().map(|(_, s)| *s).collect();
            let mut order = if query.is_empty() {
                (0..scores.len()).collect()
            } else {
                fuzzy::rank_scores(&scores)
            };
            if section.keep_unmatched {
                order.extend((0..scores.len()).filter(|&i| scores[i].is_none()));
            }
            let best = order.first().and_then(|&i| scores[i]);
            let mut entries: Vec<Option<Item>> = section
                .entries
                .into_iter()
                .map(|(item, _)| Some(item))
                .collect();
            let items: Vec<Item> = order
                .into_iter()
                .filter_map(|i| entries[i].take())
                .collect();
            if !items.is_empty() {
                sections.push((section.heading, items, best));
            }
        }
        if !query.is_empty() {
            // Stable, so equally good sections keep their order.
            sections.sort_by_key(|(_, _, best)| std::cmp::Reverse(*best));
        }

        self.items.clear();
        self.rows.clear();
        for (heading, items, _) in sections {
            if !self.rows.is_empty() {
                self.rows.push(Row::Separator);
            }
            self.rows.push(Row::Heading(heading));
            for item in items {
                self.rows.push(Row::Item(self.items.len()));
                self.items.push(item);
            }
        }
        let kept = self
            .chosen_key
            .as_ref()
            .and_then(|key| self.items.iter().position(|item| &item.key() == key));
        // Listing recent notes, the open one comes first; start on the one
        // before it, so ⌘P and Enter go back to the previous note.
        let open_first = self.mode == Some(Mode::Notes)
            && query.is_empty()
            && self.items.len() > 1
            && matches!(
                (&self.items[0].target, &self.note),
                (Target::Note(first), Some(open)) if first.id == open.id
            );
        let first = if open_first { 1 } else { 0 };
        self.selected = kept.or((!self.items.is_empty()).then_some(first));
        if kept.is_none() {
            self.scroll.scroll_to_item(0);
        }
    }

    // MARK: Actions

    fn select(&mut self, ix: usize, scroll: bool, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(ix) else {
            return;
        };
        if self.selected == Some(ix) {
            return;
        }
        self.selected = Some(ix);
        self.chosen_key = Some(item.key());
        if scroll
            && let Some(row) = self
                .rows
                .iter()
                .position(|row| matches!(row, Row::Item(i) if *i == ix))
        {
            // Bring the heading into view with the first entry.
            self.scroll.scroll_to_item(if ix == 0 { 0 } else { row });
        }
        cx.notify();
    }

    fn select_by(&mut self, step: isize, cx: &mut Context<Self>) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let next = match self.selected {
            Some(ix) => (ix as isize + step).rem_euclid(len as isize) as usize,
            None => 0,
        };
        self.select(next, true, cx);
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.select_by(1, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.select_by(-1, cx);
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self.selected {
            self.choose(ix, window, cx);
        }
    }

    /// Escape leaves a second list for the commands it came from, and
    /// otherwise closes.
    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.leads_back() {
            self.show(Mode::Commands, window, cx);
        } else {
            self.close(window, cx);
        }
    }

    /// Backspace in an empty field leaves a second list for the commands it
    /// came from; otherwise it deletes as usual.
    fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
        if self.leads_back() && self.input.read(cx).value().is_empty() {
            self.show(Mode::Commands, window, cx);
        } else {
            cx.propagate();
        }
    }

    /// Does what entry `ix` stands for.
    fn choose(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(ix).cloned() else {
            return;
        };
        match item.target {
            Target::Command(command) => match command.run {
                Run::Pick(picker) => {
                    self.from_commands = true;
                    self.show(Mode::Pick(picker), window, cx);
                }
                Run::Action(action) => {
                    self.close(window, cx);
                    // From the workspace, which is always drawn, the action
                    // finds the window's handlers whatever has the keyboard;
                    // after this one, so a handler can open the palette.
                    let home = self.home.clone();
                    let action = action();
                    window.defer(cx, move |window, cx| {
                        home.dispatch_action(action.as_ref(), window, cx);
                    });
                }
            },
            Target::Show(filter) => {
                self.close(window, cx);
                self.library
                    .update(cx, |library, cx| library.set_filter(filter, cx));
            }
            Target::Note(note) => {
                self.close(window, cx);
                self.library
                    .update(cx, |library, cx| library.select(Some(note.id), cx));
                // The preview alone has no text to type into; the list keeps
                // the keyboard then.
                if self.editor.read(cx).view_mode == ViewMode::Preview {
                    window.focus(&self.list_focus, cx);
                } else {
                    self.editor
                        .update(cx, |editor, cx| editor.focus_body(window, cx));
                }
            }
            Target::Book(id) => {
                self.close(window, cx);
                self.editor.update(cx, |editor, cx| editor.set_book(id, cx));
            }
            Target::Tag(id) => {
                self.close(window, cx);
                self.editor
                    .update(cx, |editor, cx| editor.toggle_tag(id, cx));
            }
            Target::NewTag(name) => {
                self.close(window, cx);
                match self
                    .store
                    .put(DocType::Tag, json!({"name": name}), Put::Upsert, Origin::Ui)
                {
                    Ok(created) => self
                        .editor
                        .update(cx, |editor, cx| editor.toggle_tag(created.id, cx)),
                    Err(err) => window.push_notification(
                        Notification::error(format!("Couldn't create the tag: {err}")),
                        cx,
                    ),
                }
            }
            Target::Status(status) => {
                self.close(window, cx);
                self.editor
                    .update(cx, |editor, cx| editor.set_status(status, cx));
            }
        }
    }

    // MARK: Rendering

    fn render_search(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .h(px(SEARCH_HEIGHT))
            .flex_none()
            .px_3()
            .flex()
            .items_center()
            .child(
                h_flex()
                    .w_full()
                    .h(px(40.))
                    .px_3()
                    .gap_2()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(Hsla::from(colors.accent).opacity(0.55))
                    .bg(colors.editor)
                    .child(
                        Icon::new(IconName::Search)
                            .size_4()
                            .text_color(colors.text_muted),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(15.))
                            .child(Input::new(&self.input).appearance(false).cleanable(true)),
                    ),
            )
    }

    fn render_row(
        &self,
        ix: usize,
        item: &Item,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = Colors::get(cx);
        let selected = self.selected == Some(ix);
        let keys = match &item.target {
            Target::Command(command) => command
                .action()
                .and_then(|action| shortcut(action.as_ref(), window)),
            _ => None,
        };
        h_flex()
            .id(("palette-item", ix))
            .flex_none()
            .h(px(ROW_HEIGHT))
            .mx(px(6.))
            .pl(px(8. + item.depth as f32 * 14.))
            .pr_2()
            .gap(px(10.))
            .rounded(px(6.))
            .cursor_default()
            .when(selected, |el| el.bg(selected_fill(cx)))
            .child(
                Icon::new(item.icon)
                    .size_4()
                    .flex_none()
                    .text_color(if selected {
                        colors.text_secondary
                    } else {
                        colors.text_muted
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .child(item.label.clone()),
            )
            .when_some(item.hint.clone(), |el, hint| {
                el.child(
                    div()
                        .flex_none()
                        .max_w(px(120.))
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(hint),
                )
            })
            .when(item.checked, |el| {
                el.child(
                    Icon::new(IconName::Check)
                        .size(px(14.))
                        .flex_none()
                        .text_color(colors.accent),
                )
            })
            .when_some(keys, |el, keys| el.child(render_keys(&keys, false, cx)))
            .on_mouse_move(cx.listener(move |this, _, _, cx| this.select(ix, false, cx)))
            .on_click(cx.listener(move |this, _, window, cx| this.choose(ix, window, cx)))
    }

    fn render_list(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mut list = v_flex()
            .id("palette-list")
            .size_full()
            .py_1p5()
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        for row in &self.rows {
            list = match row {
                Row::Heading(heading) => list.child(
                    div()
                        .flex_none()
                        .px(px(14.))
                        .pt_2()
                        .pb_1()
                        .text_size(px(12.))
                        .font_medium()
                        .text_color(colors.text_muted)
                        .child(heading.clone()),
                ),
                Row::Separator => list.child(
                    div()
                        .flex_none()
                        .py(px(6.))
                        .px(px(10.))
                        .child(div().h(px(1.)).bg(colors.divider)),
                ),
                Row::Item(ix) => list.child(self.render_row(*ix, &self.items[*ix], window, cx)),
            };
        }
        if self.items.is_empty() {
            let message = if self.searching(cx) {
                "Searching notes…"
            } else {
                match self.mode {
                    Some(Mode::Notes) => "No matching notes",
                    Some(Mode::Pick(Picker::Notebook | Picker::GoToNotebook)) => {
                        "No matching notebooks"
                    }
                    // Only with nothing typed: a name typed offers to make it.
                    Some(Mode::Pick(Picker::Tag)) => "No tags yet. Type a name to create one.",
                    Some(Mode::Pick(Picker::Status)) => "No matching status",
                    _ => "No matching commands or notes",
                }
            };
            list = list.child(
                div()
                    .py_6()
                    .w_full()
                    .text_center()
                    .text_size(px(13.))
                    .text_color(colors.text_muted)
                    .child(message),
            );
        }
        list
    }

    fn render_detail(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let Some(item) = self.selected.and_then(|ix| self.items.get(ix)) else {
            return div().into_any_element();
        };
        let library = self.library.read(cx);
        let mut detail = Detail {
            icon: item.icon,
            title: item.label.clone(),
            meta: Vec::new(),
            description: None,
            description_lines: 6,
            sections: Vec::new(),
        };
        let note_title = self.note.as_ref().map(|n| note_title(&n.title));
        match &item.target {
            Target::Command(command) => {
                detail.description = Some(command.description.into());
                if let Some(keys) = command
                    .action()
                    .and_then(|action| shortcut(action.as_ref(), window))
                {
                    detail
                        .sections
                        .push(("Shortcut", render_keys(&keys, true, cx).into_any_element()));
                }
                if command.needs != Needs::Nothing
                    && let Some(title) = note_title.clone()
                {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Show(filter) => {
                detail.description = Some(
                    match filter {
                        Filter::All => "Shows every note outside the trash.".to_owned(),
                        Filter::Pinned => "Shows the pinned notes.".to_owned(),
                        Filter::Recent => format!(
                            "Shows the notes edited in the last {RECENT_DAYS} days, newest first."
                        ),
                        Filter::Trash => {
                            "Shows the notes in the trash. They can be restored from there."
                                .to_owned()
                        }
                        Filter::Status(status) => {
                            format!("Shows the notes marked {}.", status.label())
                        }
                        Filter::Book(_) | Filter::Tag(_) => {
                            format!("Shows the notes in {}.", item.label)
                        }
                    }
                    .into(),
                );
                if let Some(count) = &item.hint {
                    detail.meta.push((IconName::FileText, notes_count(count)));
                }
                if let Filter::Book(id) = filter
                    && let Some(parent) = library
                        .book(id)
                        .and_then(|b| b.parent_book_id.as_deref())
                        .and_then(|p| library.book(p))
                {
                    detail
                        .meta
                        .push((IconName::Notebook, format!("In {}", parent.name).into()));
                }
            }
            Target::Note(note) => {
                let book = if note.book_id == TRASH_BOOK_ID {
                    "Trash".to_owned()
                } else {
                    library
                        .book(&note.book_id)
                        .map_or_else(|| "No notebook".to_owned(), |b| b.name.clone())
                };
                detail.meta.push((IconName::Notebook, book.into()));
                // The day as the note list names it.
                let edited = crate::clock::Clock::edited_label(cx, note.updated_at);
                detail
                    .meta
                    .push((IconName::Clock, format!("Edited {edited}").into()));
                if note.status != NoteStatus::None {
                    detail
                        .meta
                        .push((status_icon(note.status), note.status.label().into()));
                }
                let dark = cx.theme().is_dark();
                let tags: Vec<AnyElement> = note
                    .tags
                    .iter()
                    .filter_map(|id| library.tag(id))
                    .map(|tag| {
                        tag_chip(&tag.name, muted_tag_color(tag.color.rgb(), dark), cx)
                            .into_any_element()
                    })
                    .collect();
                if !tags.is_empty() {
                    detail.sections.push((
                        "Tags",
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .children(tags)
                            .into_any_element(),
                    ));
                }
                if !note.snippet.is_empty() {
                    detail.description = Some(note.snippet.clone().into());
                    // Room for the metadata and tags above and below.
                    detail.description_lines = 4;
                }
            }
            Target::Book(id) => {
                let here = self.note.as_ref().is_some_and(|n| &n.book_id == id);
                detail.description = Some(
                    if here {
                        "The note is in this notebook now.".to_owned()
                    } else {
                        format!("Moves the note into {}.", item.label)
                    }
                    .into(),
                );
                if let Some(count) = &item.hint {
                    detail.meta.push((IconName::FileText, notes_count(count)));
                }
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Tag(_) => {
                detail.title = format!("#{}", item.label).into();
                detail.description = Some(
                    if item.checked {
                        format!(
                            "The note has #{}; choosing it takes the tag off.",
                            item.label
                        )
                    } else {
                        format!("Adds #{} to the note.", item.label)
                    }
                    .into(),
                );
                if let Some(count) = &item.hint {
                    detail.meta.push((IconName::FileText, notes_count(count)));
                }
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::NewTag(name) => {
                detail.title = format!("#{name}").into();
                detail.description =
                    Some(format!("Creates the tag #{name} and adds it to the note.").into());
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
            Target::Status(status) => {
                detail.description = Some(status_description(*status).into());
                if let Some(title) = note_title.clone() {
                    detail
                        .sections
                        .push(("Note", chip(title, cx).into_any_element()));
                }
            }
        }

        v_flex()
            .size_full()
            .overflow_hidden()
            .p(px(22.))
            .gap_2()
            .child(
                Icon::new(detail.icon)
                    .size(px(28.))
                    .text_color(colors.text_secondary),
            )
            .child(
                div()
                    .mt_2()
                    .text_size(px(18.))
                    .font_semibold()
                    .line_clamp(2)
                    .text_ellipsis()
                    .child(detail.title),
            )
            .when(!detail.meta.is_empty(), |el| {
                el.child(
                    v_flex()
                        .gap_1()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .children(detail.meta.into_iter().map(|(icon, text)| {
                            h_flex()
                                .gap_1p5()
                                .child(Icon::new(icon).size(px(13.)).flex_none())
                                .child(div().min_w_0().truncate().child(text))
                        })),
                )
            })
            .when_some(detail.description, |el, description| {
                el.child(
                    div()
                        .mt_1()
                        .flex_none()
                        .text_size(px(13.5))
                        .line_height(relative(1.5))
                        .text_color(colors.text_secondary)
                        .line_clamp(detail.description_lines)
                        .text_ellipsis()
                        .child(description),
                )
            })
            .children(detail.sections.into_iter().map(|(label, content)| {
                v_flex()
                    .mt_3()
                    .gap_1p5()
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(label),
                    )
                    .child(content)
            }))
            .into_any_element()
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let leave = if self.leads_back() { "Back" } else { "Close" };
        let cell = |glyph: AnyElement, label: &'static str| {
            h_flex()
                .flex_1()
                .h_full()
                .justify_center()
                .gap_1p5()
                .child(glyph)
                .child(label)
        };
        let icon = |name: IconName| Icon::new(name).size(px(14.)).into_any_element();
        let divider = || div().w(px(1.)).h_full().bg(colors.divider);
        h_flex()
            .h(px(FOOTER_HEIGHT))
            .flex_none()
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .child(
                h_flex()
                    .w(px(LIST_WIDTH))
                    .h_full()
                    .flex_none()
                    .child(cell(icon(IconName::CornerDownLeft), "Select"))
                    .child(divider())
                    .child(cell(icon(IconName::ArrowUpDown), "Navigate")),
            )
            .child(divider())
            .child(cell(keycap("esc", cx).into_any_element(), leave))
    }
}

/// What the detail pane shows for an entry.
struct Detail {
    icon: IconName,
    title: SharedString,
    /// Small lines under the title: where a note lives, when it changed.
    meta: Vec<(IconName, SharedString)>,
    description: Option<SharedString>,
    /// At most this many lines of the description show.
    description_lines: usize,
    /// Labelled parts at the bottom: shortcut, the note a command acts on.
    sections: Vec<(&'static str, AnyElement)>,
}

impl Render for Palette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.mode.is_none() {
            return div().into_any_element();
        }
        let colors = Colors::get(cx);
        let divider = || div().h(px(1.)).w_full().flex_none().bg(colors.divider);
        div()
            .id("palette-overlay")
            .absolute()
            .inset_0()
            .occlude()
            .bg(cx.theme().overlay)
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .id("palette")
                    .key_context("Palette")
                    .on_action(cx.listener(Self::select_next))
                    .on_action(cx.listener(Self::select_previous))
                    .on_action(cx.listener(Self::confirm))
                    .on_action(cx.listener(Self::dismiss))
                    .on_action(cx.listener(Self::back))
                    .on_mouse_down_out(cx.listener(|this, _, window, cx| this.close(window, cx)))
                    .w(px(PANEL_WIDTH))
                    .h(px(PANEL_HEIGHT))
                    .overflow_hidden()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.elevated)
                    .text_color(colors.text)
                    .child(self.render_search(cx))
                    .child(divider())
                    .child(
                        h_flex()
                            .h(px(BODY_HEIGHT))
                            .flex_none()
                            .items_start()
                            .child(
                                div()
                                    .relative()
                                    .w(px(LIST_WIDTH))
                                    .h_full()
                                    .flex_none()
                                    .child(self.render_list(window, cx))
                                    .child(Scrollbar::vertical(&self.scroll)),
                            )
                            .child(div().w(px(1.)).h_full().bg(colors.divider))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .child(self.render_detail(window, cx)),
                            ),
                    )
                    .child(divider())
                    .child(self.render_footer(cx)),
            )
            .into_any_element()
    }
}

/// The search for notes whose titles have the query's words, or `None` when
/// it has none: its free text, asked of the title instead.
fn title_search(params: &SearchParams) -> Option<SearchParams> {
    let mut titled = params.clone();
    let mut any = false;
    for clause in &mut titled.query.clauses {
        if clause.field.is_none() && !clause.negated {
            *clause = Clause::field(Field::Title, std::mem::take(&mut clause.value));
            any = true;
        }
    }
    any.then_some(titled)
}

/// `first`, then the notes of `then` not already in it, at most `limit` in
/// all.
fn merge_by_id(first: Vec<NoteSummary>, then: Vec<NoteSummary>, limit: usize) -> Vec<NoteSummary> {
    let mut merged: Vec<NoteSummary> = Vec::with_capacity(limit);
    for note in first.into_iter().chain(then) {
        if merged.len() == limit {
            break;
        }
        if !merged.iter().any(|m| m.id == note.id) {
            merged.push(note);
        }
    }
    merged
}

/// The tag name in what was typed in the tag list: without the "#" tags are
/// shown with, and the spaces around it.
fn tag_name(query: &str) -> &str {
    query
        .trim_start_matches(|c: char| c == '#' || c.is_whitespace())
        .trim_end()
}

/// How well `query` fits an entry: its label, or a little less, one of its
/// keywords.
fn item_score(query: &str, label: &str, keywords: &[&str]) -> Option<i64> {
    let in_keywords = keywords
        .iter()
        .filter_map(|keyword| fuzzy::score(query, keyword))
        .max()
        .map(|score| score - KEYWORD_PENALTY);
    fuzzy::score(query, label).max(in_keywords)
}

/// The keys that run `action`, as the keymap has them now, split for
/// showing apart: "⇧", "⌘", "F".
pub(crate) fn shortcut(action: &dyn Action, window: &Window) -> Option<Vec<String>> {
    let binding = window.highest_precedence_binding_for_action(action)?;
    let keystroke = binding.keystrokes().first()?.as_keystroke().clone();
    Some(shortcut_parts(&Kbd::format(&keystroke)))
}

const MODIFIER_SYMBOLS: [char; 4] = ['⌃', '⌥', '⇧', '⌘'];

/// Splits a shortcut as written ("⇧⌘F") into its modifier symbols and the
/// key. The key is what's left after the leading modifiers, so a key that is
/// itself a modifier (a bare "⌘") stays.
pub(crate) fn shortcut_parts(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if !MODIFIER_SYMBOLS.contains(&c) || rest.len() == c.len_utf8() {
            break;
        }
        parts.push(c.to_string());
        rest = &rest[c.len_utf8()..];
    }
    if !rest.is_empty() {
        parts.push(rest.to_owned());
    }
    parts
}

// no-prop-test: drawing; the parts come from `shortcut_parts`.
fn render_keys(keys: &[String], caps: bool, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    h_flex()
        .flex_none()
        .gap(px(if caps { 4. } else { 3. }))
        .text_size(px(12.5))
        .text_color(colors.text_muted)
        .children(keys.iter().map(|key| {
            if caps {
                keycap(key.clone(), cx).into_any_element()
            } else {
                div()
                    .min_w(px(9.))
                    .text_center()
                    .child(key.clone())
                    .into_any_element()
            }
        }))
}

fn keycap(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    div()
        .flex_none()
        .min_w(px(20.))
        .h(px(20.))
        .px(px(5.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .border_1()
        .border_color(colors.divider)
        .bg(colors.editor)
        .text_size(px(11.5))
        .text_color(colors.text_secondary)
        .child(text.into())
}

/// A bordered chip, like the scope chips in the design, as wide as its
/// text up to the pane's width.
fn chip(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    h_flex().child(
        div()
            .min_w_0()
            .px_2p5()
            .py_1()
            .rounded(px(6.))
            .border_1()
            .border_color(colors.divider)
            .text_size(px(12.5))
            .text_color(colors.text)
            .truncate()
            .child(text.into()),
    )
}

/// A tag as a chip with its color as a dot. Bordered rather than filled:
/// the dark theme's fill is the panel's own color.
fn tag_chip(name: &str, rgb: u32, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    h_flex()
        .gap_1p5()
        .px_2()
        .py_0p5()
        .rounded(px(6.))
        .border_1()
        .border_color(colors.divider)
        .text_size(px(12.))
        .text_color(colors.text_secondary)
        .child(div().size_2().rounded_full().bg(rgb_color(rgb)))
        .child(name.to_owned())
}

/// The selected row: a quiet tint of the accent, which shows on the
/// elevated panel in both themes.
fn selected_fill(cx: &App) -> Hsla {
    let alpha = if cx.theme().is_dark() { 0.26 } else { 0.18 };
    Hsla::from(Colors::get(cx).accent).opacity(alpha)
}

fn placeholder(mode: Mode) -> &'static str {
    match mode {
        Mode::Commands => "Search commands and notes…",
        Mode::Notes => "Open a note by title…",
        Mode::Pick(Picker::Notebook) => "Move to notebook…",
        Mode::Pick(Picker::Tag) => "Add or create a tag…",
        Mode::Pick(Picker::Status) => "Set status…",
        Mode::Pick(Picker::GoToNotebook) => "Go to notebook…",
    }
}

fn note_title(title: &str) -> String {
    if title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        title.to_owned()
    }
}

fn notes_count(count: &str) -> SharedString {
    if count == "1" {
        "1 note".into()
    } else {
        format!("{count} notes").into()
    }
}

fn status_label(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "No status",
        other => other.label(),
    }
}

fn status_icon(status: NoteStatus) -> IconName {
    match status {
        NoteStatus::None => IconName::Minus,
        NoteStatus::Active => IconName::CircleDot,
        NoteStatus::OnHold => IconName::CirclePause,
        NoteStatus::Completed => IconName::CircleCheck,
        NoteStatus::Dropped => IconName::CircleX,
    }
}

fn status_description(status: NoteStatus) -> &'static str {
    match status {
        NoteStatus::None => "Clears the note's status.",
        NoteStatus::Active => "Marks the note Active: being worked on.",
        NoteStatus::OnHold => "Marks the note On hold: paused for now.",
        NoteStatus::Completed => "Marks the note Completed: done.",
        NoteStatus::Dropped => "Marks the note Dropped: it won't be done.",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
    use kanso_core::{DocType, NoteStatus};
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    use super::{
        MODIFIER_SYMBOLS, Mode, Palette, merge_by_id, shortcut, shortcut_parts, tag_name,
        title_search,
    };
    use crate::config::AppConfig;
    use crate::library::Filter;
    use crate::services::Services;
    use crate::workspace::{AppState, FocusSearch, NewNote, SearchAllNotes, Workspace};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariant: the parts put back together are the shortcut; every
        /// part but the last is one modifier symbol, and the last is never
        /// a modifier followed by more (the split takes them all).
        #[test]
        fn prop_shortcut_parts_split_off_every_leading_modifier(text in "[⌃⌥⇧⌘]{0,4}[A-Z1↓\\\\,]{0,2}") {
            let parts = shortcut_parts(&text);
            prop_assert_eq!(parts.concat(), text.clone());
            prop_assert!(parts.iter().all(|p| !p.is_empty()));
            if let Some((key, modifiers)) = parts.split_last() {
                for part in modifiers {
                    let mut chars = part.chars();
                    prop_assert!(chars.next().is_some_and(|c| MODIFIER_SYMBOLS.contains(&c)));
                    prop_assert!(chars.next().is_none());
                }
                let mut chars = key.chars();
                let first = chars.next().unwrap();
                prop_assert!(!MODIFIER_SYMBOLS.contains(&first) || chars.next().is_none(), "{:?}", parts);
            }
        }

        /// Invariants: the name is part of what was typed, with no "#" or
        /// space in front and no space after, and a second pass keeps it.
        #[test]
        fn prop_tag_name_drops_the_hash_and_spaces(query in "[# a-z\t]{0,12}") {
            let name = tag_name(&query);
            prop_assert!(query.contains(name));
            prop_assert!(!name.starts_with(|c: char| c == '#' || c.is_whitespace()));
            prop_assert!(!name.ends_with(char::is_whitespace));
            prop_assert_eq!(tag_name(name), name);
        }

        /// Metamorphic: any run of "#" and spaces in front changes nothing,
        /// and a plain name comes back as it is.
        #[test]
        fn prop_tag_name_ignores_what_tags_are_shown_with(
            prefix in "[# ]{0,4}",
            name in "[a-z][a-z#-]{0,8}",
        ) {
            prop_assert_eq!(tag_name(&name), name.as_str());
            let typed = format!("{prefix}{name}");
            prop_assert_eq!(tag_name(&typed), name.as_str());
        }
    }

    fn summary(id: usize) -> kanso_store::NoteSummary {
        kanso_store::NoteSummary {
            id: format!("note:{id}"),
            rev: String::new(),
            title: String::new(),
            snippet: String::new(),
            book_id: String::new(),
            status: NoteStatus::None,
            tags: Vec::new(),
            pinned: false,
            updated_at: 0,
            created_at: 0,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: the merge lists each note once, the first list's
        /// notes first and in their order, then the second's in theirs, as
        /// many as the limit allows of the notes in either.
        #[test]
        fn prop_merge_puts_the_first_list_first(
            first in prop::collection::vec(0usize..30, 0..25),
            then in prop::collection::vec(0usize..30, 0..25),
            limit in 0usize..30,
        ) {
            let dedup = |ids: &[usize]| {
                let mut seen = Vec::new();
                for id in ids {
                    if !seen.contains(id) {
                        seen.push(*id);
                    }
                }
                seen
            };
            let (first, then) = (dedup(&first), dedup(&then));
            let merged: Vec<String> = merge_by_id(
                first.iter().map(|&i| summary(i)).collect(),
                then.iter().map(|&i| summary(i)).collect(),
                limit,
            )
            .into_iter()
            .map(|n| n.id)
            .collect();
            let union = dedup(&[first.clone(), then.clone()].concat());
            prop_assert_eq!(merged.len(), union.len().min(limit));
            for (ix, id) in merged.iter().enumerate() {
                prop_assert!(!merged[..ix].contains(id));
            }
            let lead: Vec<String> = first.iter().take(limit).map(|&i| format!("note:{i}")).collect();
            prop_assert_eq!(&merged[..lead.len()], &lead[..]);
            let rest: Vec<String> = then.iter().map(|&i| format!("note:{i}")).filter(|id| !lead.contains(id)).collect();
            prop_assert_eq!(&merged[lead.len()..], &rest[..merged.len() - lead.len()]);
        }
    }

    #[test]
    fn title_searches_ask_the_title() {
        let params = crate::library::search_params(
            &Filter::All,
            "sync -draft tag:x",
            Default::default(),
            true,
        );
        let titled = title_search(&params).unwrap();
        assert_eq!(titled.query.to_string(), "title:sync -draft tag:x");
        let none = crate::library::search_params(&Filter::All, "tag:x", Default::default(), true);
        assert!(title_search(&none).is_none());
        let merged: Vec<String> = merge_by_id(
            vec![summary(1), summary(2)],
            vec![summary(2), summary(3)],
            3,
        )
        .into_iter()
        .map(|n| n.id)
        .collect();
        assert_eq!(merged, ["note:1", "note:2", "note:3"]);
    }

    #[test]
    fn tag_name_examples() {
        assert_eq!(tag_name("#mcp"), "mcp");
        assert_eq!(tag_name("  ##mcp "), "mcp");
        assert_eq!(tag_name("c#"), "c#");
        assert_eq!(tag_name("to do"), "to do");
        assert_eq!(tag_name("#"), "");
        assert_eq!(tag_name(""), "");
    }

    #[test]
    fn shortcut_examples() {
        assert_eq!(shortcut_parts("⇧⌘F"), ["⇧", "⌘", "F"]);
        assert_eq!(shortcut_parts("⌘\\"), ["⌘", "\\"]);
        assert_eq!(shortcut_parts("⌥⌘↓"), ["⌥", "⌘", "↓"]);
        assert_eq!(shortcut_parts("⌘"), ["⌘"]);
        assert_eq!(shortcut_parts("Space"), ["Space"]);
        assert!(shortcut_parts("").is_empty());
    }

    struct Fixture {
        _dir: tempfile::TempDir,
        store: Arc<Store>,
        window: AnyWindowHandle,
        workspace: Entity<Workspace>,
        /// The open note, in notebook "Work".
        note: String,
        other_book: String,
        /// What has the keyboard while writing the note.
        body: gpui_kit::FocusHandle,
    }

    fn setup(cx: &mut TestAppContext) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let book = |name: &str| {
            store
                .put(
                    DocType::Book,
                    json!({"name": name}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let work = book("Work");
        let other_book = book("Garden");
        let note = |title: &str, body: &str| {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": body, "bookId": work}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        note("Quarterly planning", "goals");
        let open = note("Weekly sync notes", "agenda");
        let app_store = store.clone();
        let data_dir = dir.path().to_path_buf();
        let (window, workspace) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(AppState {
                store: app_store,
                data_dir,
                config: AppConfig::default(),
            });
            cx.set_global(Services::default());
            crate::workspace::bind_keys(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| Workspace::new(window, cx))
            })
            .unwrap()
        });
        // Active, as the window in front is: focus changes report blurs.
        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        let library = cx.read(|cx| workspace.read(cx).library.clone());
        let id = open.clone();
        library.update(cx, |library, cx| library.select(Some(id), cx));
        cx.run_until_parked();
        // The keyboard in the note's text, as when writing.
        let body = window
            .update(cx, |_, window, cx| {
                let editor = workspace.read(cx).editor.clone();
                editor.update(cx, |editor, cx| editor.focus_body(window, cx));
                window.focused(cx).expect("the body takes the keyboard")
            })
            .unwrap();
        cx.run_until_parked();
        Fixture {
            _dir: dir,
            store,
            window,
            workspace,
            note: open,
            other_book,
            body,
        }
    }

    impl Fixture {
        fn update<R>(
            &self,
            cx: &mut TestAppContext,
            f: impl FnOnce(
                &mut Workspace,
                &mut gpui_kit::Window,
                &mut gpui_kit::Context<Workspace>,
            ) -> R,
        ) -> R {
            let workspace = self.workspace.clone();
            let result = self
                .window
                .update(cx, |_, window, cx| {
                    workspace.update(cx, |workspace, cx| f(workspace, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
            result
        }

        fn palette(&self, cx: &mut TestAppContext) -> Entity<Palette> {
            cx.read(|cx| self.workspace.read(cx).palette.clone())
        }

        fn mode(&self, cx: &mut TestAppContext) -> Option<Mode> {
            let palette = self.palette(cx);
            cx.read(|cx| palette.read(cx).mode())
        }

        fn keys(&self, cx: &mut TestAppContext, keys: &str) {
            cx.simulate_keystrokes(self.window, keys);
            // Notes are searched after a pause in typing.
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            cx.run_until_parked();
        }

        fn type_query(&self, cx: &mut TestAppContext, query: &str) {
            let palette = self.palette(cx);
            self.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| palette.set_query(query, window, cx))
                })
                .unwrap();
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(200));
            cx.run_until_parked();
        }

        fn selected_label(&self, cx: &mut TestAppContext) -> Option<String> {
            let palette = self.palette(cx);
            cx.read(|cx| {
                let palette = palette.read(cx);
                palette
                    .selected
                    .map(|ix| palette.items[ix].label.to_string())
            })
        }

        fn labels(&self, cx: &mut TestAppContext) -> Vec<String> {
            let palette = self.palette(cx);
            cx.read(|cx| {
                palette
                    .read(cx)
                    .items
                    .iter()
                    .map(|i| i.label.to_string())
                    .collect()
            })
        }

        fn body_focused(&self, cx: &mut TestAppContext) -> bool {
            let body = self.body.clone();
            self.update(cx, |_, window, _| body.is_focused(window))
        }

        /// Draws, so focus changes reach their listeners and actions find
        /// what is on screen.
        fn draw(&self, cx: &mut TestAppContext) {
            self.window
                .update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .unwrap();
            cx.run_until_parked();
        }

        fn open(&self, cx: &mut TestAppContext, mode: Mode) {
            self.update(cx, |workspace, window, cx| {
                workspace
                    .palette
                    .update(cx, |palette, cx| palette.open(mode, window, cx));
            });
        }

        /// Clicks the middle of what is drawn with `selector`.
        fn click(&self, cx: &mut TestAppContext, selector: &'static str) {
            self.draw(cx);
            let mut vcx = gpui_kit::VisualTestContext::from_window(self.window, cx);
            let bounds = vcx
                .debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is drawn"));
            vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
            cx.run_until_parked();
        }
    }

    #[gpui_kit::test]
    fn cmd_k_opens_and_escape_gives_the_keyboard_back(cx: &mut TestAppContext) {
        let f = setup(cx);
        assert!(f.body_focused(cx));
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        assert!(!f.body_focused(cx), "the field has the keyboard");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New note"));
        f.keys(cx, "down down");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Search all notes"));
        f.keys(cx, "up");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New notebook…"));
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx), "back where it was");
        // The shortcut again closes it too.
        f.keys(cx, "cmd-k");
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx));
    }

    #[gpui_kit::test]
    fn rows_show_the_real_shortcuts(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        let (new_note, search_all, search_list) = f.update(cx, |_, window, _| {
            (
                shortcut(&NewNote, window),
                shortcut(&SearchAllNotes, window),
                shortcut(&FocusSearch, window),
            )
        });
        assert_eq!(new_note, Some(vec!["⌘".to_owned(), "N".to_owned()]));
        assert_eq!(
            search_all,
            Some(vec!["⇧".to_owned(), "⌘".to_owned(), "F".to_owned()])
        );
        assert_eq!(search_list, Some(vec!["⌘".to_owned(), "L".to_owned()]));
    }

    /// Global search leaves the notebook that was showing: what is typed
    /// finds notes in every notebook. Searching this list keeps to it.
    #[gpui_kit::test]
    fn search_all_notes_searches_every_notebook(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.store
            .put(
                DocType::Note,
                json!({"title": "Tomato seedlings", "body": "", "bookId": f.other_book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let work = cx.read(|cx| {
            let books = &library.read(cx).books;
            books.iter().find(|b| b.name == "Work").unwrap().id.clone()
        });
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(work.clone()), cx);
            library.set_pinned_only(true, cx);
        });
        cx.run_until_parked();
        let search = cx.read(|cx| f.workspace.read(cx).note_list.read(cx).search.clone());
        let titles = |cx: &mut TestAppContext| -> Vec<String> {
            cx.read(|cx| {
                library
                    .read(cx)
                    .notes
                    .iter()
                    .map(|n| n.title.clone())
                    .collect()
            })
        };
        let search_focused = |cx: &mut TestAppContext| {
            let search = search.clone();
            f.update(cx, |_, window, cx| {
                gpui_kit::Focusable::focus_handle(search.read(cx), cx).is_focused(window)
            })
        };

        // This list: still Work, and the tomato isn't in it.
        f.keys(cx, "cmd-k");
        f.type_query(cx, "search this list");
        f.keys(cx, "enter");
        assert!(search_focused(cx));
        assert_eq!(
            cx.read(|cx| library.read(cx).filter.clone()),
            Filter::Book(work)
        );
        f.keys(cx, "t o m a t o");
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert!(titles(cx).is_empty());

        // All notes: the same words find it in Garden.
        f.keys(cx, "cmd-k");
        f.type_query(cx, "search all notes");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Search all notes"));
        f.keys(cx, "enter");
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert!(search_focused(cx));
        let library_state = cx.read(|cx| {
            let library = library.read(cx);
            (
                library.filter.clone(),
                library.is_refined(),
                library.search.clone(),
            )
        });
        assert_eq!(library_state, (Filter::All, false, "tomato".to_owned()));
        assert_eq!(titles(cx), ["Tomato seedlings"]);
    }

    #[gpui_kit::test]
    fn typing_ranks_commands_and_finds_notes(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "sync");
        let labels = f.labels(cx);
        assert!(labels.contains(&"Sync now".to_owned()), "{labels:?}");
        assert!(
            labels.contains(&"Weekly sync notes".to_owned()),
            "{labels:?}"
        );
        assert!(!labels.contains(&"New note".to_owned()), "{labels:?}");
        f.type_query(cx, "trash");
        // The place called Trash fits better than the command that moves
        // notes there.
        assert_eq!(f.selected_label(cx).as_deref(), Some("Trash"));
    }

    #[gpui_kit::test]
    fn a_command_runs_through_the_window(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "pin note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Pin note"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert!(f.store.note(&f.note).unwrap().unwrap().pinned);
        assert!(f.body_focused(cx));
        f.keys(cx, "cmd-k");
        f.type_query(cx, "pin note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Unpin note"));
    }

    #[gpui_kit::test]
    fn moving_to_a_notebook_uses_a_second_list(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "move to notebook");
        f.keys(cx, "enter");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Notebook))
        );
        assert_eq!(f.labels(cx), ["Garden", "Work"]);
        // Escape goes back to the commands rather than closing.
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.type_query(cx, "move to notebook");
        f.keys(cx, "enter");
        f.type_query(cx, "gard");
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().book_id,
            f.other_book
        );
    }

    #[gpui_kit::test]
    fn backspace_in_an_empty_field_leaves_the_second_list(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "set status");
        f.keys(cx, "enter");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Status))
        );
        f.keys(cx, "a backspace");
        assert_eq!(
            f.mode(cx),
            Some(Mode::Pick(crate::commands::Picker::Status))
        );
        f.keys(cx, "backspace");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
    }

    #[gpui_kit::test]
    fn setting_a_status_and_creating_a_tag(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "set status");
        f.keys(cx, "enter");
        f.type_query(cx, "hold");
        f.keys(cx, "enter");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().status,
            NoteStatus::OnHold
        );

        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, "errands");
        assert_eq!(f.labels(cx), ["Create tag “errands”"]);
        f.keys(cx, "enter");
        let tags = f.store.tags().unwrap();
        let tag = tags.iter().find(|t| t.name == "errands").expect("created");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().tags,
            vec![tag.id.clone()]
        );
    }

    /// Tags are shown as "#errands"; typed that way, the tag list finds the
    /// tag rather than offering to make "#errands" as well.
    #[gpui_kit::test]
    fn a_tag_typed_with_its_hash_is_found(cx: &mut TestAppContext) {
        let f = setup(cx);
        let errands = f
            .store
            .put(
                DocType::Tag,
                json!({"name": "errands"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        cx.run_until_parked();
        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, "#");
        assert_eq!(f.labels(cx), ["errands"], "every tag, nothing to create");
        // The tag first and chosen; making "err" stays possible, last.
        f.type_query(cx, "#err");
        assert_eq!(f.labels(cx), ["errands", "Create tag “err”"]);
        assert_eq!(f.selected_label(cx).as_deref(), Some("errands"));
        f.keys(cx, "enter");
        assert_eq!(f.store.note(&f.note).unwrap().unwrap().tags, [errands]);
        let names: Vec<String> = f
            .store
            .tags()
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(names, ["errands"]);
        // A new one is made under its name, without the "#".
        f.keys(cx, "cmd-k");
        f.type_query(cx, "add tag");
        f.keys(cx, "enter");
        f.type_query(cx, " #garden ");
        assert_eq!(f.labels(cx), ["Create tag “garden”"]);
        f.keys(cx, "enter");
        let tags = f.store.tags().unwrap();
        assert!(tags.iter().any(|t| t.name == "garden"), "{tags:?}");
        assert!(!tags.iter().any(|t| t.name.contains('#')), "{tags:?}");
    }

    /// The note's notebook path and "Add tag" open the palette's lists
    /// straight away, which filter, scroll and make new tags. Escape closes
    /// them rather than going back to commands that were never shown, and a
    /// library with no tags yet can make its first one from there.
    #[gpui_kit::test]
    fn the_header_opens_the_notebook_and_tag_lists(cx: &mut TestAppContext) {
        use crate::commands::Picker;

        let f = setup(cx);
        f.click(cx, "add-tag");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Tag)));
        assert!(f.labels(cx).is_empty(), "no tags yet");
        // Backspace in the empty field has no list to go back to.
        f.keys(cx, "backspace");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Tag)));
        f.keys(cx, "escape");
        assert_eq!(f.mode(cx), None);
        assert!(f.body_focused(cx), "back to the note");

        f.click(cx, "add-tag");
        f.type_query(cx, "errands");
        assert_eq!(f.labels(cx), ["Create tag “errands”"]);
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let tags = f.store.tags().unwrap();
        let errands = tags.iter().find(|t| t.name == "errands").expect("created");
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().tags,
            std::slice::from_ref(&errands.id)
        );
        assert!(f.body_focused(cx));

        f.click(cx, "breadcrumb");
        assert_eq!(f.mode(cx), Some(Mode::Pick(Picker::Notebook)));
        assert_eq!(f.labels(cx), ["Garden", "Work"]);
        f.type_query(cx, "gard");
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        assert_eq!(
            f.store.note(&f.note).unwrap().unwrap().book_id,
            f.other_book
        );
    }

    #[gpui_kit::test]
    fn cmd_p_opens_a_note_by_title(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // The most recently edited first, with nothing typed.
        assert_eq!(f.labels(cx), ["Weekly sync notes", "Quarterly planning"]);
        // The open note is first; Enter goes to the one before it.
        assert_eq!(f.selected_label(cx).as_deref(), Some("Quarterly planning"));
        f.type_query(cx, "qplan");
        assert_eq!(
            f.labels(cx),
            Vec::<String>::new(),
            "the store searches text"
        );
        f.type_query(cx, "plan");
        assert_eq!(f.labels(cx), ["Quarterly planning"]);
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let selected = cx.read(|cx| library.read(cx).selected.clone()).unwrap();
        assert_eq!(
            f.store.note(&selected).unwrap().unwrap().title,
            "Quarterly planning"
        );
        assert!(f.body_focused(cx));
    }

    #[gpui_kit::test]
    fn opening_a_note_from_the_command_list_switches_lists(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "open note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Open note…"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // Closing returns the keyboard to the note, not to the first list.
        f.keys(cx, "escape");
        assert!(f.body_focused(cx));
    }

    /// Every command the palette runs as an action finds a handler from
    /// where the palette sends it (the note being written, inside the
    /// workspace), whatever the layout. Quit and Settings are the app's own:
    /// main.rs registers them with `cx.on_action`.
    #[gpui_kit::test]
    fn every_command_reaches_a_handler(cx: &mut TestAppContext) {
        let f = setup(cx);
        let app_wide = ["kanso::Quit", "kanso::OpenSettings"];
        let missing: Vec<&str> = f.update(cx, |_, window, cx| {
            crate::commands::all()
                .iter()
                .filter(|command| {
                    command.action().is_some_and(|action| {
                        !window.is_action_available(action.as_ref(), cx)
                            && !app_wide.contains(&action.name())
                    })
                })
                .map(|command| command.id)
                .collect()
        });
        assert!(missing.is_empty(), "no handler for {missing:?}");
        // The exceptions are real commands, so the list can't go stale.
        for name in app_wide {
            assert!(
                crate::commands::all()
                    .iter()
                    .any(|c| c.action().is_some_and(|a| a.name() == name)),
                "{name}"
            );
        }
    }

    #[gpui_kit::test]
    fn go_to_shows_recent(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        f.type_query(cx, "recent");
        assert_eq!(f.selected_label(cx).as_deref(), Some("Recent"));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        assert_eq!(
            cx.read(|cx| library.read(cx).filter.clone()),
            Filter::Recent
        );
    }

    #[gpui_kit::test]
    fn note_actions_need_an_open_note(cx: &mut TestAppContext) {
        let f = setup(cx);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        library.update(cx, |library, cx| library.select(None, cx));
        cx.run_until_parked();
        f.open(cx, Mode::Commands);
        let labels = f.labels(cx);
        assert!(labels.contains(&"New note".to_owned()));
        for hidden in [
            "Pin note",
            "Move to notebook…",
            "Move to trash",
            "Show history…",
        ] {
            assert!(
                !labels.contains(&hidden.to_owned()),
                "{hidden} in {labels:?}"
            );
        }
    }

    /// Notes with the words in their titles come first, even when many newer
    /// notes only mention them.
    #[gpui_kit::test]
    fn titled_notes_beat_newer_mentions(cx: &mut TestAppContext) {
        let f = setup(cx);
        let book = f.store.books().unwrap()[0].id.clone();
        f.store
            .put(
                DocType::Note,
                json!({"title": "Sync design", "body": "D1 and R2", "bookId": book,
                    "createdAt": 1_000, "updatedAt": 1_000}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        for n in 0..25 {
            f.store
                .put(
                    DocType::Note,
                    json!({"title": format!("Task {n}"), "body": "rechecked after the sync", "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        f.keys(cx, "cmd-p");
        f.type_query(cx, "sync");
        let labels = f.labels(cx);
        let at = labels.iter().position(|l| l == "Sync design");
        assert!(at.is_some_and(|at| at < 2), "{labels:?}");
    }

    /// Going back to the query already shown drops the search for the one
    /// in between, so its results never replace the right ones.
    #[gpui_kit::test]
    fn an_abandoned_search_never_lands(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-p");
        f.type_query(cx, "sync");
        let shown = f.labels(cx);
        assert_eq!(shown, ["Weekly sync notes"]);
        let palette = f.palette(cx);
        let set = |cx: &mut TestAppContext, query: &str| {
            f.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| palette.set_query(query, window, cx))
                })
                .unwrap();
            cx.run_until_parked();
        };
        set(cx, "syncx");
        set(cx, "sync");
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(200));
        cx.run_until_parked();
        assert_eq!(f.labels(cx), shown);
        assert!(!cx.read(|cx| palette.read(cx).searching(cx)));
    }

    /// A shortcut that moves the keyboard elsewhere closes the palette
    /// rather than leaving it over what is being typed into.
    #[gpui_kit::test]
    fn a_shortcut_that_takes_the_keyboard_closes_the_palette(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.keys(cx, "cmd-n");
        f.draw(cx);
        assert_eq!(f.mode(cx), None);
        let editor = cx.read(|cx| f.workspace.read(cx).editor.clone());
        assert!(f.update(cx, |_, window, cx| {
            editor.read(cx).title_focused(window, cx)
        }));
        // Typing goes into the new note's title.
        f.keys(cx, "h i");
        let id = cx
            .read(|cx| editor.read(cx).note_id().map(str::to_owned))
            .unwrap();
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(1));
        cx.run_until_parked();
        assert_eq!(f.store.note(&id).unwrap().unwrap().title, "hi");
    }

    /// The empty notebook's palette row, with the keyboard last in a note's
    /// text that is no longer drawn: New note still makes a note there.
    #[gpui_kit::test]
    fn commands_run_whatever_had_the_keyboard(cx: &mut TestAppContext) {
        let f = setup(cx);
        let library = cx.read(|cx| f.workspace.read(cx).library.clone());
        let research = f
            .store
            .put(
                DocType::Book,
                json!({"name": "Research"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        cx.run_until_parked();
        library.update(cx, |library, cx| {
            library.set_filter(Filter::Book(research.clone()), cx)
        });
        cx.run_until_parked();
        assert!(f.update(cx, |workspace, _, cx| {
            workspace.editor.read(cx).note_id().is_none()
        }));
        let empty = cx.read(|cx| f.workspace.read(cx).editor.read(cx).empty_focus.clone());
        f.draw(cx);
        f.window
            .update(cx, |_, window, cx| {
                empty.dispatch_action(&crate::workspace::OpenCommandPalette, window, cx)
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(f.mode(cx), Some(Mode::Commands));
        f.type_query(cx, "new note");
        assert_eq!(f.selected_label(cx).as_deref(), Some("New note"));
        f.keys(cx, "enter");
        let in_research = f
            .store
            .search_summaries(&Default::default())
            .unwrap()
            .into_iter()
            .filter(|n| n.book_id == research)
            .count();
        assert_eq!(in_research, 1);
    }

    /// Opening a note from ⌘P while only the preview shows leaves the
    /// keyboard somewhere drawn, so the shortcuts keep working.
    #[gpui_kit::test]
    fn shortcuts_work_after_opening_a_note_in_preview(cx: &mut TestAppContext) {
        let f = setup(cx);
        f.update(cx, |workspace, _, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.set_view_mode(crate::config::ViewMode::Preview, cx)
            });
        });
        f.draw(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        f.keys(cx, "enter");
        assert_eq!(f.mode(cx), None);
        f.draw(cx);
        f.keys(cx, "cmd-p");
        assert_eq!(f.mode(cx), Some(Mode::Notes));
        // And choosing Preview from the palette, which hides the text that
        // gets the keyboard back.
        f.keys(cx, "escape");
        f.update(cx, |workspace, _, cx| {
            workspace.editor.update(cx, |editor, cx| {
                editor.set_view_mode(crate::config::ViewMode::Edit, cx)
            });
        });
        f.update(cx, |workspace, window, cx| {
            workspace
                .editor
                .update(cx, |editor, cx| editor.focus_body(window, cx));
        });
        f.keys(cx, "cmd-k");
        f.type_query(cx, "preview");
        f.keys(cx, "enter");
        f.draw(cx);
        f.keys(cx, "cmd-k");
        assert_eq!(f.mode(cx), Some(Mode::Commands));
    }
}
