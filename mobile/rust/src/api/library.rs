//! A library of notes: lists and counts, reading, writing, changing several
//! notes, notebooks, search, properties and query views, and the stream of
//! changes.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    Hit, KansoError, Library, LibraryOverview, ListQuery, NewNote, NoteChange, NoteList, NoteView,
    NotebookRow, Property, Saved, SearchScope, Table, TagRow,
};
// A note's history, Replace in all notes and the backups.
use kanso_mobile::{
    BackupFile, DiffHunk, NoteHistory, PlannedNote, ReplaceOptions, ReplaceOutcome, ReplacePreview,
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

    /// The note's saved versions, newest first, beside the note now.
    pub fn history(&self, id: String) -> Result<NoteHistory, KansoError> {
        self.0.history(&id)
    }

    /// What changed from version `rev` to the note now, section by section.
    pub fn history_diff(&self, id: String, rev: String) -> Result<Vec<DiffHunk>, KansoError> {
        self.0.history_diff(&id, &rev)
    }

    /// Sets the note's title and text to those of version `rev`.
    pub fn restore_version(
        &self,
        id: String,
        rev: String,
        base_rev: String,
    ) -> Result<Saved, KansoError> {
        self.0.restore_version(&id, &rev, &base_rev)
    }

    pub fn replace_preview(&self, options: ReplaceOptions) -> Result<ReplacePreview, KansoError> {
        self.0.replace_preview(options)
    }

    pub fn replace_apply(
        &self,
        options: ReplaceOptions,
        replacement: String,
        planned: Vec<PlannedNote>,
    ) -> Result<ReplaceOutcome, KansoError> {
        self.0.replace_apply(options, replacement, planned)
    }

    /// Undoes a replace by its batch; says what was put back.
    pub fn replace_undo(&self, batch: String) -> Result<String, KansoError> {
        self.0.replace_undo(&batch)
    }

    /// Makes the daily backup if one is due.
    pub fn backup_if_due(&self) -> Result<Option<BackupFile>, KansoError> {
        self.0.backup_if_due()
    }

    pub fn back_up_now(&self) -> Result<BackupFile, KansoError> {
        self.0.back_up_now()
    }

    /// The backups, newest first.
    pub fn backups(&self) -> Result<Vec<BackupFile>, KansoError> {
        self.0.backups()
    }

    /// Deletes a backup made with `back_up_now`; refuses any other.
    pub fn delete_backup(&self, path: String) -> Result<(), KansoError> {
        self.0.delete_backup(&path)
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

/// What the Replace button says, or nothing when there is nothing to do.
#[frb(sync)]
pub fn replace_apply_label(matches: u32, notes: u32) -> Option<String> {
    kanso_mobile::replace_apply_label(matches, notes)
}

/// What the backups screen says of the daily backup and Back up now.
#[frb(sync)]
pub fn backups_summary() -> String {
    kanso_mobile::BACKUPS_SUMMARY.to_owned()
}
