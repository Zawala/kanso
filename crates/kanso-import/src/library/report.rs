//! What an import of a library found and did, and a summary of it.

use std::fmt;

use crate::library::{Outcome, SourceKind};

/// What a source holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub notes: usize,
    pub notebooks: usize,
    /// Distinct tag names the notes use.
    pub tags: usize,
    /// Attachments notes link to.
    pub attachments: usize,
}

/// What happened to the documents of one kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcomes {
    pub created: usize,
    pub updated: usize,
    /// The same as last time.
    pub unchanged: usize,
    /// Changed in Kanso since the last import, so left as they are.
    pub kept_edited: usize,
    /// Deleted in Kanso, so not brought back.
    pub kept_deleted: usize,
}

impl Outcomes {
    pub(crate) fn add(&mut self, outcome: Outcome, id: &str, rejected: &mut Vec<String>) {
        match outcome {
            Outcome::Created => self.created += 1,
            Outcome::Updated => self.updated += 1,
            Outcome::Unchanged => self.unchanged += 1,
            Outcome::KeptEdited => self.kept_edited += 1,
            Outcome::KeptDeleted => self.kept_deleted += 1,
            Outcome::Rejected(reason) => rejected.push(format!("{id}: {reason}")),
        }
    }

    /// Documents written.
    pub fn written(&self) -> usize {
        self.created + self.updated
    }

    pub fn kept(&self) -> usize {
        self.kept_edited + self.kept_deleted
    }
}

/// Something left out of an import, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    /// The file (or folder, ending in `/`) in the source, or what else it was.
    pub path: String,
    pub reason: String,
}

impl Skipped {
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Skipped {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

/// The outcome of [`crate::library::run`]: with `dry_run`, what would happen.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LibraryReport {
    pub kind: Option<SourceKind>,
    /// The source's name: the vault's, the folder's or the export's.
    pub source: String,
    pub dry_run: bool,
    pub found: Counts,
    pub notes: Outcomes,
    /// Notes written to the trash, as they were in the source.
    pub trashed: usize,
    pub notebooks: Outcomes,
    pub attachments: Outcomes,
    /// Tags made because no tag had their name.
    pub tags_created: usize,
    /// `![[Note]]` embeds of notes, which became links.
    pub embeds_as_links: usize,
    /// Links to notes or files that aren't in the source. Wiki-links among
    /// them still work in Kanso: they make the note when followed.
    pub unresolved_links: usize,
    pub skipped: Vec<Skipped>,
    /// `"<id or path>: <reason>"` for documents the store refused.
    pub rejected: Vec<String>,
    /// The first note imported, for opening it.
    pub first_note: Option<String>,
}

impl LibraryReport {
    pub fn is_clean(&self) -> bool {
        self.rejected.is_empty()
    }

    /// Skipped files grouped by reason: `(reason, count, first few paths)`.
    pub fn skipped_by_reason(&self) -> Vec<(String, usize, Vec<String>)> {
        let mut groups: Vec<(String, usize, Vec<String>)> = Vec::new();
        for skipped in &self.skipped {
            match groups
                .iter_mut()
                .find(|(reason, _, _)| *reason == skipped.reason)
            {
                Some((_, count, paths)) => {
                    *count += 1;
                    if paths.len() < 5 {
                        paths.push(skipped.path.clone());
                    }
                }
                None => groups.push((skipped.reason.clone(), 1, vec![skipped.path.clone()])),
            }
        }
        groups
    }
}

pub(crate) fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

fn outcome_line(f: &mut fmt::Formatter<'_>, what: &str, o: &Outcomes, dry: bool) -> fmt::Result {
    let (create, update) = if dry {
        ("new", "to update")
    } else {
        ("new", "updated")
    };
    write!(
        f,
        "\n  {what}: {} {create}, {} {update}, {} unchanged",
        o.created, o.updated, o.unchanged
    )?;
    if o.kept_edited > 0 {
        write!(f, ", {} kept as edited in Kanso", o.kept_edited)?;
    }
    if o.kept_deleted > 0 {
        write!(f, ", {} left deleted", o.kept_deleted)?;
    }
    Ok(())
}

impl fmt::Display for LibraryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind.map_or("Source", SourceKind::label);
        write!(
            f,
            "{kind} “{}”: {}, {}, {}, {}.",
            self.source,
            plural(self.found.notes, "note", "notes"),
            plural(self.found.notebooks, "notebook", "notebooks"),
            plural(self.found.tags, "tag", "tags"),
            plural(self.found.attachments, "attachment", "attachments"),
        )?;
        write!(
            f,
            "\n{}",
            if self.dry_run {
                "An import would make:"
            } else {
                "Imported:"
            }
        )?;
        outcome_line(f, "Notes", &self.notes, self.dry_run)?;
        if self.trashed > 0 {
            write!(f, " ({} in the trash)", self.trashed)?;
        }
        outcome_line(f, "Notebooks", &self.notebooks, self.dry_run)?;
        outcome_line(f, "Attachments", &self.attachments, self.dry_run)?;
        write!(f, "\n  Tags: {} new", self.tags_created)?;
        if self.embeds_as_links > 0 {
            write!(
                f,
                "\n{}.",
                plural(
                    self.embeds_as_links,
                    "embedded note became a link",
                    "embedded notes became links"
                )
            )?;
        }
        if self.unresolved_links > 0 {
            write!(
                f,
                "\n{} at nothing in the source.",
                plural(self.unresolved_links, "link points", "links point")
            )?;
        }
        if !self.skipped.is_empty() {
            write!(f, "\nSkipped {}:", self.skipped.len())?;
            for (reason, count, paths) in self.skipped_by_reason() {
                write!(f, "\n  {count} — {reason}: {}", paths.join(", "))?;
                if count > paths.len() {
                    write!(f, ", …")?;
                }
            }
        }
        if !self.rejected.is_empty() {
            write!(f, "\nRejected {}:", self.rejected.len())?;
            for line in &self.rejected {
                write!(f, "\n  {line}")?;
            }
        }
        Ok(())
    }
}
