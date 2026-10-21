//! What an import did, and a human summary of it.

use std::fmt;

/// Live documents of each kind found in the PouchDB library.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceCounts {
    pub notes: usize,
    pub trashed_notes: usize,
    pub books: usize,
    pub tags: usize,
    pub files: usize,
    pub activities: usize,
}

/// The outcome of [`crate::import`].
///
/// `notes`, `books`, `tags`, `files` and `archived` count documents written by
/// this run; documents the store already had at the same revision count as
/// `unchanged`. The link counts describe every live note in the source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub notes: usize,
    /// Written notes whose `bookId` is the trash.
    pub trashed_notes: usize,
    pub books: usize,
    pub tags: usize,
    pub files: usize,
    /// `activity:` documents written to the archive table.
    pub archived: usize,
    pub skipped_design_and_config: usize,
    /// Documents whose winning revision is a deletion.
    pub skipped_deleted: usize,
    /// Documents of a kind Kanso doesn't know (none in real data so far).
    pub skipped_unknown: usize,
    /// Documents already in the store at the same revision.
    pub unchanged: usize,
    /// Written documents that replaced a different revision already in the store.
    pub replaced: usize,
    /// `"<file id>: <reason>"` for every file whose bytes don't match its digest.
    /// Those files are not imported.
    pub attachment_mismatches: Vec<String>,
    /// `"<id>: <reason>"` for documents that couldn't be read or that the store refused.
    pub rejected: Vec<String>,
    /// IDs that are not in the store at their source revision after the import.
    pub unverified: Vec<String>,
    /// Notes whose body links another note with a note link
    /// ([`NOTE_LINK_PREFIX`](kanso_core::links::NOTE_LINK_PREFIX)`<id>`).
    pub notes_linking_notes: usize,
    /// Notes whose body embeds a file with an attachment link
    /// ([`FILE_LINK_PREFIX`](kanso_core::links::FILE_LINK_PREFIX)`<id>`).
    pub notes_embedding_files: usize,
    /// Distinct linked note IDs that don't exist after the import.
    pub dangling_note_links: usize,
    /// Distinct embedded file IDs that don't exist after the import.
    pub dangling_file_links: usize,
    pub source: SourceCounts,
}

impl ImportReport {
    /// Documents written by this run.
    pub fn written(&self) -> usize {
        self.notes + self.books + self.tags + self.files + self.archived
    }

    /// No attachment mismatches, rejected or unverified documents.
    pub fn is_clean(&self) -> bool {
        self.attachment_mismatches.is_empty()
            && self.rejected.is_empty()
            && self.unverified.is_empty()
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

impl fmt::Display for ImportReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &self.source;
        writeln!(
            f,
            "PouchDB library: {} ({} in trash), {}, {}, {}, {}.",
            plural(s.notes, "note", "notes"),
            s.trashed_notes,
            plural(s.books, "notebook", "notebooks"),
            plural(s.tags, "tag", "tags"),
            plural(s.files, "file", "files"),
            plural(s.activities, "activity doc", "activity docs"),
        )?;
        writeln!(
            f,
            "Imported: {} ({} in trash), {}, {}, {}; archived {}.",
            plural(self.notes, "note", "notes"),
            self.trashed_notes,
            plural(self.books, "notebook", "notebooks"),
            plural(self.tags, "tag", "tags"),
            plural(self.files, "file", "files"),
            plural(self.archived, "activity doc", "activity docs"),
        )?;
        writeln!(
            f,
            "Unchanged: {}. Replaced an older copy: {}.",
            self.unchanged, self.replaced
        )?;
        writeln!(
            f,
            "Skipped: {} design/config, {} deleted, {} unknown.",
            self.skipped_design_and_config, self.skipped_deleted, self.skipped_unknown
        )?;
        writeln!(
            f,
            "Links: {} link other notes ({} dangling targets); {} embed files ({} dangling targets).",
            plural(self.notes_linking_notes, "note", "notes"),
            self.dangling_note_links,
            self.notes_embedding_files,
            self.dangling_file_links,
        )?;
        write!(
            f,
            "Attachment mismatches: {}.",
            self.attachment_mismatches.len()
        )?;
        for line in &self.attachment_mismatches {
            write!(f, "\n  {line}")?;
        }
        if !self.rejected.is_empty() {
            write!(f, "\nRejected: {}.", self.rejected.len())?;
            for line in &self.rejected {
                write!(f, "\n  {line}")?;
            }
        }
        if !self.unverified.is_empty() {
            write!(
                f,
                "\nNot in the store at their source revision: {}.",
                self.unverified.len()
            )?;
            for id in &self.unverified {
                write!(f, "\n  {id}")?;
            }
        }
        Ok(())
    }
}
