//! The app's view of the store: notebooks, tags, counts and the note list for
//! the current filter and search. Views read it; writes go to the store, and
//! the store's change feed brings the lists back up to date.

use std::sync::Arc;

use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{Book, Clause, Field, NoteStatus, Query, Tag};
use kanso_store::{Change, NoteSummary, SearchParams, Sort, StatusCounts, Store};

/// What the note list shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Pinned,
    Book(String),
    Tag(String),
    Status(NoteStatus),
    Trash,
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
            Filter::All | Filter::Pinned => Vec::new(),
            Filter::Book(id) => vec![clause(Field::BookId, id)],
            Filter::Tag(id) => vec![clause(Field::TagId, id)],
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
    }
}

/// Pinned notes first, keeping the sort order within each group.
pub fn pinned_first(notes: &mut [NoteSummary]) {
    notes.sort_by_key(|note| !note.pinned);
}

pub enum LibraryEvent {
    /// The selected note changed.
    Selected(Option<String>),
    /// A document changed in the store.
    Changed(Change),
}

pub struct Library {
    pub store: Arc<Store>,
    pub books: Vec<Book>,
    pub tags: Vec<Tag>,
    pub counts: StatusCounts,
    pub filter: Filter,
    pub search: String,
    pub sort: Sort,
    pub descending: bool,
    pub notes: Vec<NoteSummary>,
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
    counts: StatusCounts,
    notes: Vec<NoteSummary>,
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
            counts: StatusCounts::default(),
            filter: Filter::All,
            search: String::new(),
            sort: Sort::UpdatedAt,
            descending: true,
            notes: Vec::new(),
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

    /// Reloads everything on a background thread; a newer reload wins.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let store = self.store.clone();
        let params = search_params(&self.filter, &self.search, self.sort, self.descending);
        let pinned_only = self.filter == Filter::Pinned;
        let task = cx.background_executor().spawn(async move {
            let mut notes = store.search_summaries(&params)?;
            if pinned_only {
                notes.retain(|note| note.pinned);
            }
            pinned_first(&mut notes);
            Ok::<_, kanso_store::StoreError>(Loaded {
                books: store.books()?,
                tags: store.tags()?,
                counts: store.status_counts()?,
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
                        this.counts = loaded.counts;
                        this.notes = loaded.notes;
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
                        this.select(first, cx);
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        if self.filter != filter {
            self.filter = filter;
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

    pub fn select(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if self.selected != id {
            self.selected = id.clone();
            cx.emit(LibraryEvent::Selected(id));
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

    pub fn filter_title(&self) -> String {
        match &self.filter {
            Filter::All => "All Notes".into(),
            Filter::Pinned => "Pinned".into(),
            Filter::Book(id) => self.book(id).map_or("Notebook".into(), |b| b.name.clone()),
            Filter::Tag(id) => self
                .tag(id)
                .map_or("Tag".into(), |t| format!("#{}", t.name)),
            Filter::Status(status) => status.label().into(),
            Filter::Trash => "Trash".into(),
        }
    }
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
    use super::{Filter, book_tree, pinned_first, search_params};
    use kanso_core::{Book, NoteStatus, Query};
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
        fn prop_filter_appends_its_clauses(search in "[a-z :\"-]{0,20}", which in 0usize..6) {
            let filter = [
                Filter::All,
                Filter::Pinned,
                Filter::Book("book:x".into()),
                Filter::Tag("tag:y".into()),
                Filter::Status(NoteStatus::Active),
                Filter::Trash,
            ][which].clone();
            let typed = Query::parse(&search);
            let params = search_params(&filter, &search, Sort::Title, false);
            prop_assert_eq!(&params.query.clauses[..typed.clauses.len()], &typed.clauses[..]);
            prop_assert_eq!(&params.query.clauses[typed.clauses.len()..], &filter.clauses()[..]);
            prop_assert_eq!(params.query.includes_trash(), filter == Filter::Trash || typed.includes_trash());
        }

        /// Invariant: pinned notes come first and each group keeps its order.
        #[test]
        fn prop_pinned_first_is_a_stable_partition(pins in prop::collection::vec(any::<bool>(), 0..30)) {
            let mut notes: Vec<NoteSummary> = pins.iter().enumerate().map(|(ix, &pinned)| NoteSummary {
                id: format!("note:{ix}"), rev: String::new(), title: String::new(), snippet: String::new(),
                book_id: String::new(), status: NoteStatus::None, tags: Vec::new(), pinned,
                updated_at: 0, created_at: 0,
            }).collect();
            let expected: Vec<String> = notes.iter().filter(|n| n.pinned)
                .chain(notes.iter().filter(|n| !n.pinned)).map(|n| n.id.clone()).collect();
            pinned_first(&mut notes);
            prop_assert_eq!(notes.into_iter().map(|n| n.id).collect::<Vec<_>>(), expected);
        }
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

    #[test]
    fn trash_filter_searches_the_trash() {
        let params = search_params(&Filter::Trash, "hello", Sort::UpdatedAt, true);
        assert!(params.query.includes_trash());
        assert_eq!(params.query.to_string(), "hello bookId:trash");
    }
}
