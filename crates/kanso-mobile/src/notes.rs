//! The note calls of [`Library`]: lists, one note, saving, creating,
//! trashing, notebooks, tags, search, properties and query views.
//!
//! A locked note shows its title and nothing else while it is closed (and
//! its snippet, tasks and properties never, open or not): its text is only
//! ever returned by [`Library::note`] and read for [`Library::properties`]
//! and the preview when a key this session holds opens it.

use std::time::Instant;

use kanso_core::model::{BOOK_COLOR_FIELD, BOOK_ICON_FIELD, TRASH_BOOK_ID};
use kanso_core::query::{Clause, Field, Query};
use kanso_core::query_view::ViewNote;
use kanso_core::{DocType, Note, NoteStatus, new_id, now_ms};
use kanso_engine::lock::LockView;
use kanso_store::{Origin, Put, SearchParams, Sort};
use serde_json::{Value, json};

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::matching::ranges;
use crate::model::{
    Hit, LockState, NewNote, NoteQuery, NoteRow, NoteView, NotebookRow, Property, PropertyKind,
    Saved, SortKey, Table, TableRow, TagRow,
};

impl Library {
    /// The notes `query` lists, as list rows. Locked notes are listed with
    /// their titles; their snippets are empty and they count no tasks.
    pub fn notes(&self, query: NoteQuery) -> Result<Vec<NoteRow>, KansoError> {
        guard(|| {
            let params = SearchParams {
                query: list_query(&query)?,
                sort: match query.sort {
                    SortKey::Updated => Sort::UpdatedAt,
                    SortKey::Created => Sort::CreatedAt,
                    SortKey::Title => Sort::Title,
                },
                descending: query.descending,
                limit: (query.limit > 0).then_some(query.limit as usize),
                skip: query.skip as usize,
                ocr: false,
            };
            Ok(self
                .inner
                .store
                .search_summaries(&params)?
                .into_iter()
                .map(row)
                .collect())
        })
    }

    /// One note. Its text is `None` while it is locked and closed; reading
    /// a note in a locked notebook seals it first when a key is held.
    pub fn note(&self, id: &str) -> Result<NoteView, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            let state = self.reveal(&mut note);
            Ok(view(note, state))
        })
    }

    /// Replaces the note's text with `body`. `base_rev` is the revision the
    /// caller read: if the note changed since, nothing is written and the
    /// error says its revision now. A locked note takes it sealed, and stays
    /// open for another auto-lock time; a closed one refuses with
    /// [`KansoError::Locked`].
    pub fn save(&self, id: &str, base_rev: &str, body: String) -> Result<Saved, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            // Reading a note in a locked notebook seals it, which moves its
            // revision.
            let mut shown = note.clone();
            let state = self.reveal(&mut shown);
            if shown.rev != note.rev {
                note = self.stored(id)?;
            }
            if note.rev.as_deref() != Some(base_rev) {
                return Err(KansoError::Conflict {
                    current_rev: note.rev.unwrap_or_default(),
                });
            }
            if state.is_closed() {
                return Err(KansoError::Locked);
            }
            note.body = body;
            note.updated_at = now_ms();
            if state == LockView::Open {
                held(&self.inner.keys)
                    .seal(&mut note, Instant::now())
                    .map_err(|_| KansoError::Locked)?;
            }
            let value =
                serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
            match self
                .inner
                .store
                .put(DocType::Note, value, Put::Upsert, Origin::Ui)
            {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: self.stored(id).ok().and_then(|n| n.rev).unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    /// Makes a note. In a locked notebook it is sealed, which needs the lock
    /// open ([`KansoError::Locked`] otherwise).
    pub fn create(&self, new: NewNote) -> Result<Saved, KansoError> {
        guard(|| {
            let book_id = self.notebook_for(new.notebook_id.as_deref())?;
            let now = now_ms();
            let mut note: Note = serde_json::from_value(json!({
                "_id": new_id(DocType::Note),
                "title": new.title,
                "body": new.body,
                "bookId": book_id,
                "createdAt": now,
                "updatedAt": now,
            }))
            .map_err(|err| KansoError::invalid(err.to_string()))?;
            let in_locked_notebook = self.inner.store.locked_books()?.contains(&book_id);
            let stored = if in_locked_notebook {
                held(&self.inner.keys)
                    .seal_for_notebook(&mut note, &self.inner.store)
                    .map_err(|_| KansoError::Locked)?
            } else {
                note
            };
            let value = serde_json::to_value(&stored)
                .map_err(|err| KansoError::storage(err.to_string()))?;
            let written = self
                .inner
                .store
                .put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
            Ok(Saved {
                id: written.id,
                rev: written.rev,
            })
        })
    }

    /// Moves the note to the trash. A locked note stays locked there.
    pub fn trash(&self, id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.stored(id)?;
            self.inner.store.put(
                DocType::Note,
                json!({"_id": id, "bookId": TRASH_BOOK_ID}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(())
        })
    }

    /// The notebooks, by name.
    pub fn notebooks(&self) -> Result<Vec<NotebookRow>, KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .books()?
                .into_iter()
                .map(|book| NotebookRow {
                    locked: book.is_locked(),
                    icon: book
                        .extra
                        .get(BOOK_ICON_FIELD)
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    color: book
                        .extra
                        .get(BOOK_COLOR_FIELD)
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    id: book.id,
                    name: book.name,
                    parent_id: book.parent_book_id,
                    count: book.count,
                })
                .collect())
        })
    }

    /// The tags, by name.
    pub fn tags(&self) -> Result<Vec<TagRow>, KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .tags()?
                .into_iter()
                .map(|tag| TagRow {
                    color: tag.color.rgb(),
                    id: tag.id,
                    name: tag.name,
                    count: tag.count,
                })
                .collect())
        })
    }

    /// Searches titles and texts (and the words in images) for `text`, which
    /// may use the search language (`tag:work "two words" -status:dropped`).
    /// The hits come with where the words matched. A locked note is found by
    /// its title; its text isn't searched, and its snippet stays empty.
    /// `limit` 0 is no limit.
    pub fn search(&self, text: &str, limit: u32) -> Result<Vec<Hit>, KansoError> {
        guard(|| {
            let query = Query::parse(text);
            if query.is_empty() {
                return Ok(Vec::new());
            }
            let params = SearchParams {
                query: query.clone(),
                limit: (limit > 0).then_some(limit as usize),
                ocr: true,
                ..SearchParams::default()
            };
            let title_terms: Vec<&str> = query.terms_for(Field::Title).collect();
            let body_terms: Vec<&str> = query.terms_for(Field::Body).collect();
            Ok(self
                .inner
                .store
                .search_summaries(&params)?
                .into_iter()
                .map(|summary| {
                    let row = row(summary);
                    Hit {
                        title_ranges: ranges(&row.title, &title_terms),
                        snippet_ranges: ranges(&row.snippet, &body_terms),
                        row,
                    }
                })
                .collect())
        })
    }

    /// The properties in a note's front matter. Empty while the note is
    /// locked and closed.
    pub fn properties(&self, id: &str) -> Result<Vec<Property>, KansoError> {
        guard(|| {
            let mut note = self.stored(id)?;
            if self.reveal(&mut note).is_closed() {
                return Ok(Vec::new());
            }
            Ok(kanso_core::properties::parse(&note.body)
                .into_iter()
                .map(|(key, value)| property(key, &value))
                .collect())
        })
    }

    /// Runs a query view (the body of a `query` fence) and returns its
    /// table. Locked notes and the trash are never listed; `self_id` leaves
    /// out the note the query is in.
    pub fn query_view(&self, source: &str, self_id: Option<String>) -> Result<Table, KansoError> {
        guard(|| {
            let spec = kanso_core::query_view::parse(source)
                .map_err(|err| KansoError::invalid(err.message))?;
            let notes: Vec<ViewNote> = self
                .inner
                .store
                .query_view_notes()?
                .into_iter()
                .map(ViewNote::from)
                .collect();
            let rows = kanso_core::query_view::evaluate(&spec, &notes, self_id.as_deref());
            Ok(Table {
                columns: spec.labels(),
                rows: rows
                    .into_iter()
                    .map(|row| TableRow {
                        id: row.id,
                        title: row.title,
                        cells: row
                            .cells
                            .iter()
                            .map(|cell| cell.as_ref().map(kanso_core::properties::Value::text))
                            .collect(),
                    })
                    .collect(),
            })
        })
    }

    /// The note as the store has it (sealed text empty), or not found.
    pub(crate) fn stored(&self, id: &str) -> Result<Note, KansoError> {
        if DocType::of_id(id) != Some(DocType::Note) {
            return Err(KansoError::NotFound);
        }
        self.inner.store.note(id)?.ok_or(KansoError::NotFound)
    }

    /// Opens `note`'s text when this session's keys do, and says what its
    /// lock allows. A closed note's text is emptied.
    pub(crate) fn reveal(&self, note: &mut Note) -> LockView {
        held(&self.inner.keys).reveal(note, Some(&self.inner.store))
    }

    /// The text of note `id` if it can be read now.
    pub(crate) fn readable_text(&self, id: &str) -> Result<Option<(Note, String)>, KansoError> {
        let mut note = self.stored(id)?;
        if self.reveal(&mut note).is_closed() {
            return Ok(None);
        }
        let text = note.body.clone();
        Ok(Some((note, text)))
    }

    /// The notebook a new note goes in: `wanted`, which must exist, or the
    /// first one, made if there is none.
    fn notebook_for(&self, wanted: Option<&str>) -> Result<String, KansoError> {
        let books = self.inner.store.books()?;
        match wanted {
            Some(id) => books
                .iter()
                .find(|book| book.id == id)
                .map(|book| book.id.clone())
                .ok_or(KansoError::NotFound),
            None => match books.first() {
                Some(book) => Ok(book.id.clone()),
                None => Ok(self
                    .inner
                    .store
                    .put(
                        DocType::Book,
                        json!({"name": "Notes"}),
                        Put::Upsert,
                        Origin::Ui,
                    )?
                    .id),
            },
        }
    }
}

/// The search a list asks for.
fn list_query(query: &NoteQuery) -> Result<Query, KansoError> {
    let mut clauses = Vec::new();
    if query.trash {
        clauses.push(Clause::field(Field::BookId, TRASH_BOOK_ID));
    } else if let Some(book) = &query.notebook_id {
        clauses.push(Clause::field(Field::BookId, book.as_str()));
    }
    if let Some(tag) = &query.tag_id {
        clauses.push(Clause::field(Field::TagId, tag.as_str()));
    }
    if let Some(status) = &query.status {
        let status = NoteStatus::parse(status)
            .ok_or_else(|| KansoError::invalid(format!("“{status}” isn't a status.")))?;
        clauses.push(Clause::field(Field::Status, status.as_str()));
    }
    Ok(Query { clauses })
}

fn row(summary: kanso_store::NoteSummary) -> NoteRow {
    NoteRow {
        id: summary.id,
        rev: summary.rev,
        title: summary.title,
        // A locked note's snippet is empty in the store; keep it that way
        // even if the store changes its mind.
        snippet: if summary.locked {
            String::new()
        } else {
            summary.snippet
        },
        notebook_id: summary.book_id,
        status: summary.status.as_str().to_owned(),
        tags: summary.tags,
        pinned: summary.pinned,
        created_ms: summary.created_at,
        updated_ms: summary.updated_at,
        tasks_done: if summary.locked {
            0
        } else {
            summary.tasks_done
        },
        tasks_total: if summary.locked {
            0
        } else {
            summary.tasks_total
        },
        locked: summary.locked,
    }
}

fn view(note: Note, state: LockView) -> NoteView {
    let lock = match state {
        LockView::Plain => LockState::Plain,
        LockView::Open => LockState::Open,
        LockView::Closed(reason) => LockState::Closed { reason },
    };
    NoteView {
        id: note.id,
        rev: note.rev.unwrap_or_default(),
        title: note.title,
        body: (!lock.is_closed()).then_some(note.body),
        lock,
        notebook_id: note.book_id,
        tags: note.tags,
        status: note.status.as_str().to_owned(),
        pinned: note.pinned,
        created_ms: note.created_at,
        updated_ms: note.updated_at,
    }
}

fn property(key: String, value: &kanso_core::properties::Value) -> Property {
    use kanso_core::properties::Value as V;
    let (kind, items) = match value {
        V::Text(_) => (PropertyKind::Text, Vec::new()),
        V::Number(_) => (PropertyKind::Number, Vec::new()),
        V::Bool(_) => (PropertyKind::Bool, Vec::new()),
        V::Date(_) => (PropertyKind::Date, Vec::new()),
        V::List(items) => (PropertyKind::List, items.iter().map(V::text).collect()),
        V::Raw(_) => (PropertyKind::Raw, Vec::new()),
    };
    Property {
        key,
        kind,
        text: value.text(),
        items,
    }
}
