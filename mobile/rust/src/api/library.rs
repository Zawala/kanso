//! A library of notes: lists and counts, reading, writing, changing several
//! notes, notebooks, search, properties and query views, and the stream of
//! changes.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    Hit, ImportEvent, KansoError, Library, LibraryOverview, ListQuery, NewNote, NoteChange,
    NoteList, NoteView, NotebookRow, OpenedDocuments, OpenedFile, Property, Saved, SearchScope,
    Table, TagRow,
};

use super::locks::KansoLocks;
use super::renderer::KansoRenderer;
use super::sync::KansoSync;

/// An open library. Cheap to clone on the Rust side; Dart holds one.
#[frb(opaque)]
pub struct KansoLibrary(Library);

impl KansoLibrary {
    /// Opens (creating if needed) the library whose database file is `path`.
    pub fn open(path: String) -> Result<KansoLibrary, KansoError> {
        Library::open(std::path::Path::new(&path)).map(KansoLibrary)
    }

    pub fn list(&self, query: ListQuery) -> Result<NoteList, KansoError> {
        self.0.list(query)
    }

    pub fn overview(&self) -> Result<LibraryOverview, KansoError> {
        self.0.overview()
    }

    pub fn change(&self, ids: Vec<String>, change: NoteChange) -> Result<u32, KansoError> {
        self.0.change(ids, change)
    }

    pub fn note(&self, id: String) -> Result<NoteView, KansoError> {
        self.0.note(&id)
    }

    pub fn save(&self, id: String, base_rev: String, body: String) -> Result<Saved, KansoError> {
        self.0.save(&id, &base_rev, body)
    }

    pub fn create(&self, note: NewNote) -> Result<Saved, KansoError> {
        self.0.create(note)
    }

    pub fn trash(&self, id: String) -> Result<(), KansoError> {
        self.0.trash(&id)
    }

    pub fn notebooks(&self) -> Result<Vec<NotebookRow>, KansoError> {
        self.0.notebooks()
    }

    pub fn create_notebook(
        &self,
        name: String,
        parent_id: Option<String>,
    ) -> Result<NotebookRow, KansoError> {
        self.0.create_notebook(name, parent_id)
    }

    pub fn tags(&self) -> Result<Vec<TagRow>, KansoError> {
        self.0.tags()
    }

    pub fn search(
        &self,
        text: String,
        scope: SearchScope,
        limit: u32,
    ) -> Result<Vec<Hit>, KansoError> {
        self.0.search(&text, scope, limit)
    }

    pub fn properties(&self, id: String) -> Result<Vec<Property>, KansoError> {
        self.0.properties(&id)
    }

    pub fn query_view(&self, source: String, self_id: Option<String>) -> Result<Table, KansoError> {
        self.0.query_view(&source, self_id)
    }

    /// The IDs of what changed, in batches, until Dart stops listening.
    pub fn changes(&self, sink: StreamSink<Vec<String>>) {
        self.0.changes(move |ids| sink.add(ids).is_ok());
    }

    /// What importing `path` as a `kind` would do (one of `import_sources`):
    /// progress, then one summary or one failure. Writes nothing. Returns at
    /// once; the work runs on a thread of its own.
    pub fn import_plan(&self, kind: String, path: String, sink: StreamSink<ImportEvent>) {
        self.0
            .import_plan(&kind, &path, move |event| sink.add(event).is_ok());
    }

    /// Imports `path` as a `kind`: progress, then one summary or one
    /// failure. Returns at once; the notes written reach `changes`.
    pub fn import(&self, kind: String, path: String, sink: StreamSink<ImportEvent>) {
        self.0
            .import(&kind, &path, move |event| sink.add(event).is_ok());
    }

    /// Opens the Markdown or text files the system handed over, each into
    /// the Inbox; the last note opened and what went wrong with the others.
    /// Copies of a document the platform names (`identity`) are staged under
    /// `staging` so opening it again updates the same note.
    pub fn open_documents(
        &self,
        files: Vec<OpenedFile>,
        staging: String,
    ) -> Result<OpenedDocuments, KansoError> {
        self.0.open_documents(files, &staging)
    }

    #[frb(sync)]
    pub fn locks(&self) -> KansoLocks {
        KansoLocks::new(self.0.locks())
    }

    #[frb(sync)]
    pub fn syncing(&self) -> KansoSync {
        KansoSync::new(self.0.sync())
    }

    #[frb(sync)]
    pub fn renderer(&self) -> KansoRenderer {
        KansoRenderer::new(self.0.renderer())
    }
}
