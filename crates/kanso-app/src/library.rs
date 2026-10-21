//! The app's view of the store: notebooks, tags, counts and the note list for
//! the current filter and search. Views read it; writes go to the store, and
//! the store's change feed brings the lists back up to date.

use std::sync::Arc;

use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::tags::{TagInfo, TagNode, tag_tree};
use kanso_core::{Book, Clause, Field, NoteStatus, Query, Tag};
use kanso_store::{Change, NoteSummary, SearchParams, Sort, StatusCounts, Store, StoreError};

use crate::clock::Clock;

/// How far back Recent reaches.
pub const RECENT_DAYS: i64 = 7;
const DAY_MS: i64 = 86_400_000;

/// What the note list shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Pinned,
    /// Notes changed in the last [`RECENT_DAYS`] days, newest first.
    Recent,
    Book(String),
    Tag(String),
    /// A level of the tag tree (`work` for `work/acme`): notes with a tag
    /// at or under it.
    TagPath(String),
    Status(NoteStatus),
    Trash,
    /// The tasks across notes, in the Tasks view (`crate::tasks_view`),
    /// in the note list's place.
    Tasks,
    /// Notes other accounts shared with this one (docs/ACCOUNTS.md ›
    /// Collaboration), with invitations waiting above them.
    Shared,
}

impl Filter {
    /// The query clauses this filter adds to a search.
    pub fn clauses(&self) -> Vec<Clause> {
        let clause = |field, value: &str| Clause {
            negated: false,
            field: Some(field),
            value: value.to_owned(),
        };
        match self {
            Filter::All | Filter::Pinned | Filter::Recent | Filter::Tasks | Filter::Shared => {
                Vec::new()
            }
            Filter::Book(id) => vec![clause(Field::BookId, id)],
            Filter::Tag(id) => vec![clause(Field::TagId, id)],
            Filter::TagPath(path) => vec![clause(Field::Tag, path)],
            Filter::Status(status) => vec![clause(Field::Status, status.as_str())],
            Filter::Trash => vec![clause(Field::BookId, TRASH_BOOK_ID)],
        }
    }
}

/// Builds the search for a filter and the text in the search box.
pub fn search_params(filter: &Filter, search: &str, sort: Sort, descending: bool) -> SearchParams {
    let mut query = Query::parse(search);
    query.clauses.extend(filter.clauses());
    SearchParams {
        query,
        sort,
        descending,
        limit: None,
        skip: 0,
        // The app finds notes by the words in their pictures too.
        ocr: true,
    }
}

/// Narrows a search to one status, for the note list's filter menu.
pub fn refine_status(params: &mut SearchParams, status: Option<NoteStatus>) {
    if let Some(status) = status {
        params
            .query
            .clauses
            .push(Clause::field(Field::Status, status.as_str()));
    }
}

/// Pinned notes first, keeping the sort order within each group.
pub fn pinned_first(notes: &mut [NoteSummary]) {
    notes.sort_by_key(|note| !note.pinned);
}

/// The notes changed at or after `since_ms`, newest first; notes changed at
/// the same moment keep their order.
pub fn recent_first(mut notes: Vec<NoteSummary>, since_ms: i64) -> Vec<NoteSummary> {
    notes.retain(|note| note.updated_at >= since_ms);
    notes.sort_by_key(|note| std::cmp::Reverse(note.updated_at));
    notes
}

/// The notes matching `params` changed since `since_ms`, newest first. Asks
/// the store for the newest notes a page at a time, so a large library
/// isn't read whole for a short list.
fn recent_notes(
    store: &Store,
    params: &SearchParams,
    since_ms: i64,
) -> Result<Vec<NoteSummary>, StoreError> {
    let mut limit = 64;
    loop {
        let page = SearchParams {
            query: params.query.clone(),
            sort: Sort::UpdatedAt,
            descending: true,
            limit: Some(limit),
            skip: 0,
            ocr: params.ocr,
        };
        let notes = store.search_summaries(&page)?;
        let done = notes.len() < limit || notes.last().is_some_and(|n| n.updated_at < since_ms);
        if done {
            return Ok(recent_first(notes, since_ms));
        }
        limit *= 4;
    }
}

/// The notes a list of `filter` shows for `params`: Recent's newest first,
/// the rest pinned first, and only the pinned ones when `pinned_only`.
/// The note list and the board read their notes through this.
pub fn list_notes(
    store: &Store,
    filter: &Filter,
    params: &SearchParams,
    pinned_only: bool,
    recent_since: i64,
) -> Result<Vec<NoteSummary>, StoreError> {
    let mut notes = match filter {
        Filter::Recent => recent_notes(store, params, recent_since)?,
        Filter::Shared => {
            let shared: std::collections::HashSet<String> =
                store.shared_with_me()?.into_iter().collect();
            let mut notes = store.search_summaries(params)?;
            notes.retain(|note| shared.contains(&note.id));
            pinned_first(&mut notes);
            notes
        }
        _ => {
            let mut notes = store.search_summaries(params)?;
            pinned_first(&mut notes);
            notes
        }
    };
    if pinned_only || *filter == Filter::Pinned {
        notes.retain(|note| note.pinned);
    }
    Ok(notes)
}

/// The place a board lays out for the list's `filter`: the same notebook
/// or tag, but every status, since the board's columns are the statuses.
/// Tasks and the trash have no board of their own; theirs is all notes.
pub fn board_scope(filter: &Filter) -> Filter {
    match filter {
        Filter::Status(_) | Filter::Tasks | Filter::Trash => Filter::All,
        other => other.clone(),
    }
}

pub enum LibraryEvent {
    /// The selected note changed: `asked` when someone picked it, rather
    /// than the app on its own (the first note of a new list, the last note
    /// at launch).
    Selected { id: Option<String>, asked: bool },
    /// A place was asked for (a notebook, tag, status or list), even the
    /// one already showing: the notes should be in view.
    Shown,
    /// A document changed in the store.
    Changed(Change),
}

pub struct Library {
    pub store: Arc<Store>,
    pub books: Vec<Book>,
    pub tags: Vec<Tag>,
    /// The tags nested by `/` (`kanso_core::tags::tag_tree`).
    pub tag_tree: Vec<TagNode>,
    pub counts: StatusCounts,
    /// How many notes Recent lists, without the search.
    pub recent: u32,
    /// How many open tasks are overdue, for the sidebar's Tasks badge.
    pub overdue: u32,
    /// How many notes others shared with this account.
    pub shared: u32,
    pub filter: Filter,
    pub search: String,
    pub sort: Sort,
    pub descending: bool,
    pub notes: Vec<NoteSummary>,
    /// Changes whenever `notes` is replaced, so views can tell a new list
    /// from a repaint.
    pub notes_version: u64,
    /// The note list's own narrowing on top of the filter: one status, or
    /// pinned notes only. A new filter clears both.
    pub status_only: Option<NoteStatus>,
    pub pinned_only: bool,
    pub selected: Option<String>,
    pub load_error: Option<String>,
    generation: u64,
    /// After a filter or search change, select the first note when the
    /// selection is not in the new list.
    reselect: bool,
    reload_task: Option<Task<()>>,
    _watch: Task<()>,
}

impl EventEmitter<LibraryEvent> for Library {}

struct Loaded {
    books: Vec<Book>,
    tags: Vec<Tag>,
    tag_tree: Vec<TagNode>,
    counts: StatusCounts,
    recent: u32,
    overdue: u32,
    shared: u32,
    /// `None` while the Tasks view shows: the list keeps its notes.
    notes: Option<Vec<NoteSummary>>,
}

impl Library {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        let changes = store.subscribe();
        let watch = cx.spawn(async move |this, cx| {
            while let Ok(first) = changes.recv().await {
                let mut batch = vec![first];
                while let Ok(change) = changes.try_recv() {
                    batch.push(change);
                }
                let alive = this.update(cx, |this, cx| {
                    this.reload(cx);
                    for change in batch {
                        cx.emit(LibraryEvent::Changed(change));
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let mut library = Library {
            store,
            books: Vec::new(),
            tags: Vec::new(),
            tag_tree: Vec::new(),
            counts: StatusCounts::default(),
            recent: 0,
            overdue: 0,
            shared: 0,
            filter: Filter::All,
            search: String::new(),
            sort: Sort::UpdatedAt,
            descending: true,
            notes: Vec::new(),
            notes_version: 0,
            status_only: None,
            pinned_only: false,
            selected: None,
            load_error: None,
            generation: 0,
            reselect: true,
            reload_task: None,
            _watch: watch,
        };
        library.reload(cx);
        library
    }

    /// Reads the overdue count again: tasks fall overdue as time passes.
    pub fn refresh_overdue(&mut self, cx: &mut Context<Self>) {
        let now =
            kanso_core::dates::DateTime::from_local_ms(Clock::local_ms(cx, Clock::now_ms(cx)));
        if let Ok(overdue) = self.store.overdue_count(now)
            && overdue != self.overdue
        {
            self.overdue = overdue;
            cx.notify();
        }
    }

    /// Reloads everything on a background thread; a newer reload wins.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let store = self.store.clone();
        let mut params = search_params(&self.filter, &self.search, self.sort, self.descending);
        refine_status(&mut params, self.status_only);
        let filter = self.filter.clone();
        let pinned_only = filter == Filter::Pinned || self.pinned_only;
        // Recent as listed, unnarrowed, is the count itself.
        let recent_is_listed =
            filter == Filter::Recent && self.search.trim().is_empty() && !self.is_refined();
        let recent_since = Clock::now_ms(cx) - RECENT_DAYS * DAY_MS;
        let now =
            kanso_core::dates::DateTime::from_local_ms(Clock::local_ms(cx, Clock::now_ms(cx)));
        let task = cx.background_executor().spawn(async move {
            let notes = match filter {
                Filter::Tasks => None,
                _ => Some(list_notes(
                    &store,
                    &filter,
                    &params,
                    pinned_only,
                    recent_since,
                )?),
            };
            let recent = match &notes {
                Some(notes) if recent_is_listed => u32::try_from(notes.len()).unwrap_or(u32::MAX),
                _ => store.count_recent(recent_since)?,
            };
            let tags = store.tags()?;
            let infos: Vec<TagInfo> = tags
                .iter()
                .map(|tag| TagInfo {
                    id: tag.id.clone(),
                    name: tag.name.clone(),
                })
                .collect();
            Ok::<_, StoreError>(Loaded {
                books: store.books()?,
                tag_tree: tag_tree(&infos, &store.tag_links()?),
                tags,
                counts: store.status_counts()?,
                recent,
                overdue: store.overdue_count(now)?,
                shared: u32::try_from(store.shared_with_me()?.len()).unwrap_or(u32::MAX),
                notes,
            })
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let loaded = task.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match loaded {
                    Ok(loaded) => {
                        this.books = loaded.books;
                        this.tags = loaded.tags;
                        this.tag_tree = loaded.tag_tree;
                        this.counts = loaded.counts;
                        this.recent = loaded.recent;
                        this.overdue = loaded.overdue;
                        this.shared = loaded.shared;
                        if let Some(notes) = loaded.notes {
                            this.notes = notes;
                            this.notes_version += 1;
                        }
                        this.load_error = None;
                    }
                    Err(err) => this.load_error = Some(err.to_string()),
                }
                if std::mem::take(&mut this.reselect) {
                    let listed = this
                        .selected
                        .as_ref()
                        .is_some_and(|id| this.notes.iter().any(|n| &n.id == id));
                    if !listed {
                        let first = this.notes.first().map(|n| n.id.clone());
                        this.select_quietly(first, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        cx.emit(LibraryEvent::Shown);
        if self.filter != filter {
            // The Tasks view leaves the open note as it is.
            self.reselect = filter != Filter::Tasks;
            self.filter = filter;
            self.status_only = None;
            self.pinned_only = false;
            self.reload(cx);
        }
    }

    pub fn set_status_only(&mut self, status: Option<NoteStatus>, cx: &mut Context<Self>) {
        if self.status_only != status {
            self.status_only = status;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_pinned_only(&mut self, pinned_only: bool, cx: &mut Context<Self>) {
        if self.pinned_only != pinned_only {
            self.pinned_only = pinned_only;
            self.reselect = true;
            self.reload(cx);
        }
    }

    /// Whether the note list narrows the filter further.
    pub fn is_refined(&self) -> bool {
        self.status_only.is_some() || self.pinned_only
    }

    pub fn clear_refinements(&mut self, cx: &mut Context<Self>) {
        if self.is_refined() {
            self.status_only = None;
            self.pinned_only = false;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_search(&mut self, search: String, cx: &mut Context<Self>) {
        if self.search != search {
            self.search = search;
            self.reselect = true;
            self.reload(cx);
        }
    }

    pub fn set_sort(&mut self, sort: Sort, descending: bool, cx: &mut Context<Self>) {
        self.sort = sort;
        self.descending = descending;
        self.reload(cx);
    }

    /// Selects the note someone picked.
    pub fn select(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.select_as(id, true, cx);
    }

    /// Selects a note the app picked on its own, which doesn't take the
    /// window away from what it shows (Settings, say).
    pub fn select_quietly(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.select_as(id, false, cx);
    }

    /// A note picked again that is already selected still says so, so
    /// whatever hides it (Settings) gives way.
    fn select_as(&mut self, id: Option<String>, asked: bool, cx: &mut Context<Self>) {
        let changed = self.selected != id;
        if changed || asked {
            self.selected = id.clone();
            cx.emit(LibraryEvent::Selected { id, asked });
        }
        if changed {
            cx.notify();
        }
    }

    /// Selects the note `offset` rows away from the current one.
    pub fn select_relative(&mut self, offset: isize, cx: &mut Context<Self>) {
        if self.notes.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|id| self.notes.iter().position(|n| &n.id == id));
        let next = match current {
            Some(ix) => ix.saturating_add_signed(offset).min(self.notes.len() - 1),
            None => 0,
        };
        let id = self.notes[next].id.clone();
        self.select(Some(id), cx);
    }

    pub fn book(&self, id: &str) -> Option<&Book> {
        self.books.iter().find(|b| b.id == id)
    }

    pub fn tag(&self, id: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == id)
    }

    /// The notebook a new note goes into: the selected one, else the first.
    pub fn default_book(&self) -> Option<String> {
        match &self.filter {
            Filter::Book(id) => Some(id.clone()),
            _ => self.books.first().map(|b| b.id.clone()),
        }
    }

    /// Notebooks in tree order with their depth.
    pub fn book_tree(&self) -> Vec<(usize, &Book)> {
        book_tree(&self.books)
    }

    /// The notebook `id` and the ones it is inside, outermost first.
    pub fn book_path(&self, id: &str) -> Vec<&Book> {
        book_path(id, &self.books)
    }

    /// Whether notebook `id` is locked: by its own lock, or by one of the
    /// notebooks it is inside.
    pub fn is_book_locked(&self, id: &str) -> bool {
        is_book_locked(id, &self.books)
    }

    /// Whether the notebooks inside `id` hold notes, though it may have
    /// none of its own.
    pub fn has_notes_inside(&self, id: &str) -> bool {
        has_notes_inside(id, &self.books)
    }

    /// The title for an empty place, "No notes in Research" or "No notes
    /// yet", when the list is empty only because all notes or a notebook
    /// has none: loaded, no search, no narrowing. The editor then offers
    /// the first note. A notebook whose notes are all in the notebooks
    /// inside it isn't empty.
    // no-prop-test: a few plain conditions; the note list's
    // `empty_place_cases` covers each.
    pub fn empty_place(&self) -> Option<String> {
        let empty = self.notes_version > 0
            && self.notes.is_empty()
            && self.load_error.is_none()
            && self.search.trim().is_empty()
            && !self.is_refined();
        if !empty {
            return None;
        }
        match &self.filter {
            Filter::All => Some("No notes yet".into()),
            Filter::Book(id) if self.has_notes_inside(id) => None,
            Filter::Book(id) => Some(format!(
                "No notes in {}",
                self.book(id).map_or("this notebook", |b| b.name.as_str())
            )),
            _ => None,
        }
    }

    /// Whether the library has no notes at all, not even in the trash, as
    /// on first run: the editor then offers the first note or an import.
    // no-prop-test: a conjunction of loaded counts; `first_run_cases` in
    // the note list tests covers it.
    pub fn is_empty_library(&self) -> bool {
        self.notes_version > 0
            && self.load_error.is_none()
            && self.counts.all == 0
            && self.counts.trash == 0
    }

    pub fn filter_title(&self) -> String {
        self.title_of(&self.filter)
    }

    /// A place's name, as the list's title says it.
    pub fn title_of(&self, filter: &Filter) -> String {
        match filter {
            Filter::All => "All Notes".into(),
            Filter::Pinned => "Pinned".into(),
            Filter::Recent => "Recent".into(),
            Filter::Book(id) => self.book(id).map_or("Notebook".into(), |b| b.name.clone()),
            Filter::Tag(id) => self
                .tag(id)
                .map_or("Tag".into(), |t| format!("#{}", t.name)),
            Filter::TagPath(path) => format!("#{path}"),
            Filter::Status(status) => status.label().into(),
            Filter::Trash => "Trash".into(),
            Filter::Tasks => "Tasks".into(),
            Filter::Shared => "Shared with me".into(),
        }
    }
}

/// Notebook `id` and its parents, outermost first: the path a breadcrumb
/// shows. A parent that is missing ends it, and so does a parent already on
/// it, should the notebooks form a cycle.
pub fn book_path<'a>(id: &str, books: &'a [Book]) -> Vec<&'a Book> {
    let mut path: Vec<&Book> = Vec::new();
    let mut next = Some(id);
    while let Some(id) = next {
        let Some(book) = books.iter().find(|b| b.id == id) else {
            break;
        };
        if path.iter().any(|b| b.id == book.id) {
            break;
        }
        path.push(book);
        next = book.parent_book_id.as_deref();
    }
    path.reverse();
    path
}

/// Whether notebook `id` is locked: by its own lock, or by one of the
/// notebooks it is inside (`kanso_store`'s `locked_books`, from the books
/// the app has).
pub fn is_book_locked(id: &str, books: &[Book]) -> bool {
    book_path(id, books).iter().any(|book| book.is_locked())
}

/// Notebook `id` and every notebook inside it, at any depth: what locking
/// it seals. Should the notebooks form a cycle, each is taken once.
pub fn book_subtree(id: &str, books: &[Book]) -> Vec<String> {
    let mut found = vec![id.to_owned()];
    let mut i = 0;
    while i < found.len() {
        for book in books {
            if book.parent_book_id.as_deref() == Some(found[i].as_str())
                && !found.contains(&book.id)
            {
                found.push(book.id.clone());
            }
        }
        i += 1;
    }
    found
}

/// Whether a notebook inside `id`, at any depth, has notes of its own. A
/// notebook can hold only notebooks, its notes all a level or more down;
/// it isn't empty then. Should the notebooks form a cycle, each is looked
/// at once.
pub fn has_notes_inside(id: &str, books: &[Book]) -> bool {
    let mut seen = vec![id];
    let mut parents = vec![id];
    while let Some(parent) = parents.pop() {
        for book in books
            .iter()
            .filter(|b| b.parent_book_id.as_deref() == Some(parent))
        {
            if seen.contains(&book.id.as_str()) {
                continue;
            }
            if book.count > 0 {
                return true;
            }
            seen.push(&book.id);
            parents.push(&book.id);
        }
    }
    false
}

/// Orders notebooks depth-first by name, children under their parent. A book
/// whose parent is missing is shown at the top level.
pub fn book_tree(books: &[Book]) -> Vec<(usize, &Book)> {
    fn visit<'a>(
        parent: Option<&str>,
        depth: usize,
        books: &'a [Book],
        out: &mut Vec<(usize, &'a Book)>,
    ) {
        let mut children: Vec<&Book> = books
            .iter()
            .filter(|b| match (parent, b.parent_book_id.as_deref()) {
                (None, None) => true,
                (None, Some(p)) => !books.iter().any(|other| other.id == p),
                (Some(parent), Some(p)) => p == parent,
                (Some(_), None) => false,
            })
            .collect();
        children.sort_by(|a, b| {
            (a.order, a.name.to_lowercase(), &a.id).cmp(&(b.order, b.name.to_lowercase(), &b.id))
        });
        for book in children {
            if out.iter().any(|(_, b)| b.id == book.id) {
                continue;
            }
            out.push((depth, book));
            visit(Some(&book.id), depth + 1, books, out);
        }
    }
    let mut out = Vec::with_capacity(books.len());
    visit(None, 0, books, &mut out);
    // Books in a parent cycle are unreachable from the top; still show them.
    for book in books {
        if !out.iter().any(|(_, b)| b.id == book.id) {
            out.push((0, book));
            visit(Some(&book.id), 1, books, &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        Filter, book_path, book_subtree, book_tree, has_notes_inside, is_book_locked, list_notes,
        pinned_first, recent_first, recent_notes, refine_status, search_params,
    };
    use kanso_core::{Book, NoteFacts, NoteStatus, Query};
    use kanso_store::{NoteSummary, Sort};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::Map;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn book(id: usize, parent: Option<usize>, name: &str) -> Book {
        Book {
            id: format!("book:{id}"),
            rev: None,
            name: name.into(),
            parent_book_id: parent.map(|p| format!("book:{p}")),
            order: None,
            count: 0,
            created_at: 0,
            updated_at: 0,
            extra: Map::new(),
        }
    }

    /// Forests where each book's parent has a smaller index (no cycles), and
    /// some parents point at books that don't exist.
    fn gen_books() -> impl Strategy<Value = Vec<Book>> {
        prop::collection::vec((prop::option::of(0usize..40), "[a-c]{1,3}"), 0..20).prop_map(
            |specs| {
                specs
                    .into_iter()
                    .enumerate()
                    .map(|(ix, (parent, name))| {
                        book(ix, parent.filter(|p| *p < ix || *p >= 20), &name)
                    })
                    .collect()
            },
        )
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariant: the tree lists every notebook exactly once, and each
        /// child comes after its parent, one level deeper.
        #[test]
        fn prop_book_tree_lists_each_book_once_under_its_parent(books in gen_books()) {
            let tree = book_tree(&books);
            prop_assert_eq!(tree.len(), books.len());
            for (ix, (depth, book)) in tree.iter().enumerate() {
                prop_assert_eq!(tree.iter().filter(|(_, b)| b.id == book.id).count(), 1);
                match book.parent_book_id.as_deref().and_then(|p| tree.iter().position(|(_, b)| b.id == p)) {
                    Some(parent_ix) => {
                        prop_assert!(parent_ix < ix);
                        prop_assert_eq!(*depth, tree[parent_ix].0 + 1);
                    }
                    None => prop_assert_eq!(*depth, 0),
                }
            }
        }

        /// Metamorphic: a filter only narrows a search, and its clauses come
        /// after the ones typed in the search box.
        #[test]
        fn prop_filter_appends_its_clauses(search in "[a-z :\"-]{0,20}", which in 0usize..7) {
            let filter = [
                Filter::All,
                Filter::Pinned,
                Filter::Book("book:x".into()),
                Filter::Tag("tag:y".into()),
                Filter::Status(NoteStatus::Active),
                Filter::Trash,
                Filter::Shared,
            ][which].clone();
            let typed = Query::parse(&search);
            let params = search_params(&filter, &search, Sort::Title, false);
            prop_assert_eq!(&params.query.clauses[..typed.clauses.len()], &typed.clauses[..]);
            prop_assert_eq!(&params.query.clauses[typed.clauses.len()..], &filter.clauses()[..]);
            prop_assert_eq!(params.query.includes_trash(), filter == Filter::Trash || typed.includes_trash());
        }

        /// Invariants: the path ends at the notebook, each step is the
        /// previous one's child, nothing repeats, and it is never longer than
        /// the list of notebooks, even when parents form cycles.
        #[test]
        fn prop_book_path_walks_up_to_the_top(
            parents in prop::collection::vec(prop::option::of(0usize..12), 0..10),
            target in 0usize..12,
        ) {
            // Any parent at all, so cycles and missing parents both occur.
            let books: Vec<Book> = parents.iter().enumerate().map(|(ix, p)| book(ix, *p, "b")).collect();
            let id = format!("book:{target}");
            let path = book_path(&id, &books);
            prop_assert!(path.len() <= books.len());
            prop_assert_eq!(path.is_empty(), target >= books.len());
            if let Some(last) = path.last() {
                prop_assert_eq!(&last.id, &id);
            }
            for pair in path.windows(2) {
                prop_assert_eq!(pair[1].parent_book_id.as_deref(), Some(pair[0].id.as_str()));
            }
            for (ix, b) in path.iter().enumerate() {
                prop_assert!(path[ix + 1..].iter().all(|other| other.id != b.id));
            }
            // The walk stops only at the top, a missing parent, or a cycle.
            if let Some(first) = path.first() {
                match first.parent_book_id.as_deref() {
                    None => {}
                    Some(parent) => prop_assert!(
                        !books.iter().any(|b| b.id == parent) || path.iter().any(|b| b.id == parent)
                    ),
                }
            }
        }

        /// Model: a notebook is inside a notebook's subtree exactly when the
        /// second is on the first one's path up to the top, cycles and
        /// missing parents included; and a notebook is locked exactly when
        /// a notebook on its path has a lock, which is when it is in the
        /// subtree of some notebook with a lock: locking that one seals it.
        #[test]
        fn prop_a_locked_notebook_reaches_every_notebook_inside_it(
            specs in prop::collection::vec((prop::option::of(0usize..12), any::<bool>()), 1..10),
            target in any::<prop::sample::Index>(),
        ) {
            let books: Vec<Book> = specs
                .iter()
                .enumerate()
                .map(|(ix, (parent, locked))| {
                    let mut b = book(ix, *parent, "b");
                    if *locked {
                        b.extra.insert(kanso_core::model::LOCK_FIELD.into(), serde_json::json!({}));
                    }
                    b
                })
                .collect();
            let root = &books[target.index(books.len())].id;
            let inside = book_subtree(root, &books);
            prop_assert_eq!(inside.first(), Some(root));
            for b in &books {
                let below = book_path(&b.id, &books).iter().any(|on| &on.id == root);
                prop_assert_eq!(inside.contains(&b.id), below, "{}", b.id);
                let by_subtree = books
                    .iter()
                    .filter(|lock| lock.is_locked())
                    .any(|lock| book_subtree(&lock.id, &books).contains(&b.id));
                prop_assert_eq!(is_book_locked(&b.id, &books), by_subtree, "{}", b.id);
            }
            prop_assert_eq!(
                inside.len(),
                inside.iter().collect::<std::collections::HashSet<_>>().len()
            );
        }

        /// Model: a notebook holds notes inside it exactly when some other
        /// notebook with notes has it on its path up to the top. The path
        /// walks parents bottom-up; the function walks children top-down.
        #[test]
        fn prop_notes_inside_are_below_on_some_path(
            specs in prop::collection::vec((prop::option::of(0usize..12), 0u32..3), 1..10),
            target in any::<prop::sample::Index>(),
        ) {
            // Any parent at all, so cycles and missing parents both occur.
            let books: Vec<Book> = specs
                .iter()
                .enumerate()
                .map(|(ix, (parent, count))| Book { count: *count, ..book(ix, *parent, "b") })
                .collect();
            let id = books[target.index(books.len())].id.clone();
            let expected = books.iter().any(|b| {
                b.id != id && b.count > 0 && book_path(&b.id, &books).iter().any(|p| p.id == id)
            });
            prop_assert_eq!(has_notes_inside(&id, &books), expected);
        }

        /// Model: Recent is exactly the notes changed at or after the cut,
        /// newest first, with ties in their original order.
        #[test]
        fn prop_recent_first_keeps_new_notes_newest_first(
            times in prop::collection::vec(0i64..20, 0..30),
            since in 0i64..22,
        ) {
            let notes: Vec<NoteSummary> = times.iter().enumerate().map(|(ix, &t)| summary(ix, t)).collect();
            let mut expected: Vec<(i64, usize)> = times.iter().enumerate()
                .filter(|(_, t)| **t >= since).map(|(ix, t)| (-t, ix)).collect();
            expected.sort();
            let got: Vec<String> = recent_first(notes, since).into_iter().map(|n| n.id).collect();
            let want: Vec<String> = expected.into_iter().map(|(_, ix)| format!("note:{ix}")).collect();
            prop_assert_eq!(got, want);
        }

        /// Metamorphic: narrowing to a status keeps exactly the notes the
        /// search found that have that status; no status changes nothing.
        #[test]
        fn prop_refine_status_narrows_to_that_status(
            search in "[a-z :-]{0,12}",
            want in prop::option::of(0usize..5),
            notes in prop::collection::vec((0usize..5, "[a-z ]{0,8}"), 0..12),
        ) {
            let want = want.map(|ix| NoteStatus::ALL[ix]);
            let plain = search_params(&Filter::All, &search, Sort::UpdatedAt, true);
            let mut refined = plain.clone();
            refine_status(&mut refined, want);
            if want.is_none() {
                prop_assert_eq!(&refined.query, &plain.query);
            }
            for (status, title) in &notes {
                let status = NoteStatus::ALL[*status];
                let facts = NoteFacts {
                    title, body: "", book_id: "book:x", book_name: None,
                    status: status.as_str(), tag_ids: &[], tag_names: &[],
                };
                let expected = plain.query.matches(&facts) && want.is_none_or(|w| w == status);
                prop_assert_eq!(refined.query.matches(&facts), expected);
            }
        }

        /// Invariant: pinned notes come first and each group keeps its order.
        #[test]
        fn prop_pinned_first_is_a_stable_partition(pins in prop::collection::vec(any::<bool>(), 0..30)) {
            let mut notes: Vec<NoteSummary> = pins.iter().enumerate()
                .map(|(ix, &pinned)| NoteSummary { pinned, ..summary(ix, 0) }).collect();
            let expected: Vec<String> = notes.iter().filter(|n| n.pinned)
                .chain(notes.iter().filter(|n| !n.pinned)).map(|n| n.id.clone()).collect();
            pinned_first(&mut notes);
            prop_assert_eq!(notes.into_iter().map(|n| n.id).collect::<Vec<_>>(), expected);
        }
    }

    fn summary(ix: usize, updated_at: i64) -> NoteSummary {
        NoteSummary {
            id: format!("note:{ix}"),
            rev: String::new(),
            title: String::new(),
            snippet: String::new(),
            book_id: String::new(),
            status: NoteStatus::None,
            tags: Vec::new(),
            pinned: false,
            updated_at,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    #[test]
    fn book_path_examples() {
        let books = vec![
            book(1, None, "Work"),
            book(2, Some(1), "Projects"),
            book(3, Some(2), "Kanso"),
            book(4, Some(5), "Loop"),
            book(5, Some(4), "Back"),
        ];
        let names = |id: &str| -> Vec<String> {
            book_path(id, &books)
                .into_iter()
                .map(|b| b.name.clone())
                .collect()
        };
        assert_eq!(names("book:3"), ["Work", "Projects", "Kanso"]);
        assert_eq!(names("book:1"), ["Work"]);
        assert_eq!(names("book:4"), ["Back", "Loop"]);
        assert!(names("book:9").is_empty());
    }

    #[test]
    fn notes_inside_examples() {
        let counted = |id, parent, count| Book {
            count,
            ..book(id, parent, "b")
        };
        let books = vec![
            // Clients holds its notes in a sub-notebook.
            counted(1, None, 0),
            counted(2, Some(1), 17),
            // Projects holds one empty notebook.
            counted(3, None, 0),
            counted(4, Some(3), 0),
            // Archive's notes are two levels down.
            counted(5, None, 0),
            counted(6, Some(5), 0),
            counted(7, Some(6), 2),
            // A loop with notes only at the start.
            counted(8, Some(9), 4),
            counted(9, Some(8), 0),
        ];
        let inside = |id: &str| has_notes_inside(id, &books);
        assert!(inside("book:1"));
        assert!(!inside("book:2"), "notes of its own only");
        assert!(!inside("book:3"));
        assert!(inside("book:5"));
        assert!(inside("book:6"));
        assert!(inside("book:9"));
        assert!(!inside("book:8"), "its own notes come back round the loop");
        assert!(!inside("book:99"));
    }

    /// Recent reads the store a page at a time and still finds every recent
    /// note past the first page, and none of the old or trashed ones.
    #[test]
    fn recent_notes_page_through_the_store() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, SearchParams, Store};
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        for ix in 0..150i64 {
            let book_id = if ix == 100 {
                "trash".to_owned()
            } else {
                book.clone()
            };
            store
                .put(
                    DocType::Note,
                    serde_json::json!({"title": format!("n{ix}"), "bookId": book_id, "updatedAt": 1_000 + ix}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        let recent = recent_notes(&store, &SearchParams::default(), 1_010).unwrap();
        assert_eq!(recent.len(), 139);
        assert_eq!(recent.first().unwrap().updated_at, 1_149);
        assert!(recent.windows(2).all(|w| w[0].updated_at > w[1].updated_at));
        let typed = search_params(&Filter::Recent, "title:n14", Sort::Title, false);
        let found: Vec<String> = recent_notes(&store, &typed, 1_010)
            .unwrap()
            .into_iter()
            .map(|n| n.title)
            .collect();
        assert_eq!(
            found,
            [
                "n149", "n148", "n147", "n146", "n145", "n144", "n143", "n142", "n141", "n140",
                "n14"
            ]
        );
    }

    #[test]
    fn nested_books_follow_their_parent() {
        let books = vec![
            book(1, None, "Work"),
            book(2, Some(1), "Projects"),
            book(3, None, "Archive"),
        ];
        let tree: Vec<(usize, &str)> = book_tree(&books)
            .into_iter()
            .map(|(d, b)| (d, b.name.as_str()))
            .collect();
        assert_eq!(tree, vec![(0, "Archive"), (0, "Work"), (1, "Projects")]);
    }

    /// Shared with me lists the notes others shared with this account:
    /// not the ones it shares itself, not private ones, and not trashed
    /// ones; the search box still narrows it.
    #[test]
    fn shared_with_me_lists_only_others_notes() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, SharedNote, SharedStatus, Store};
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = |title: &str, book: &str| {
            store
                .put(
                    DocType::Note,
                    serde_json::json!({"title": title, "bookId": book}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id
        };
        let theirs = note("Theirs", &book);
        let mine = note("Mine", &book);
        let trashed = note("Trashed", "trash");
        note("Private", &book);
        for (n, (id, role)) in [(&theirs, "edit"), (&mine, "owner"), (&trashed, "view")]
            .into_iter()
            .enumerate()
        {
            store
                .put_shared_note(&SharedNote {
                    share_id: format!("s{n}"),
                    note_id: id.clone(),
                    owner_id: "a_owner".into(),
                    owner_name: "user2".into(),
                    role: role.into(),
                    status: SharedStatus::Active,
                    epoch: 1,
                    list: String::new(),
                    list_sig: String::new(),
                    state: Vec::new(),
                    applied_seq: 0,
                    since_snapshot: 0,
                    bytes_since_snapshot: 0,
                    counter: 0,
                    needs_snapshot: false,
                    updated_at: 0,
                })
                .unwrap();
        }
        let listed = |search: &str| -> Vec<String> {
            let params = search_params(&Filter::Shared, search, Sort::Title, false);
            list_notes(&store, &Filter::Shared, &params, false, 0)
                .unwrap()
                .into_iter()
                .map(|n| n.title)
                .collect()
        };
        assert_eq!(listed(""), ["Theirs"]);
        assert!(listed("nothing-like-it").is_empty());
    }

    #[test]
    fn trash_filter_searches_the_trash() {
        let params = search_params(&Filter::Trash, "hello", Sort::UpdatedAt, true);
        assert!(params.query.includes_trash());
        assert_eq!(params.query.to_string(), "hello bookId:trash");
    }
}
