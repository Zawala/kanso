//! A command's changes to notes, staged while it runs and applied after it
//! returns: at once when they touch only the open note and the person
//! doesn't ask to review, otherwise after the review sheet. Either way they
//! go to the store as one batch ([`kanso_store::Store::apply_plugin_batch`])
//! that one Undo takes back.

use kanso_plugin_host::protocol::{CODE_CONFLICT, CODE_LIMIT, Failure};
use kanso_store::{PluginCreate, PluginEdit, Store, plugin_data};
use serde_json::Value;

/// The most notes one command may change or make.
pub const MAX_WRITES_PER_COMMAND: usize = 500;
/// The most text a plugin may write in a day.
pub const MAX_WRITTEN_PER_DAY: usize = 20 * 1024 * 1024;
/// The largest data a plugin may keep on one note.
pub const MAX_PLUGIN_DATA: usize = 32 * 1024;
/// The most changed lines the review sheet shows for one note.
pub const MAX_REVIEW_LINES: usize = 200;

/// One command's staged changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    /// `plugin:` and a random ID; the store keeps the batch's history under
    /// it.
    pub id: String,
    pub edits: Vec<PluginEdit>,
    pub creates: Vec<PluginCreate>,
    /// Bytes of text staged.
    pub bytes: usize,
}

impl Default for Batch {
    fn default() -> Self {
        Batch::new()
    }
}

/// A fresh batch ID. Random, so two batches never share history.
fn batch_id() -> String {
    let mut bytes = [0u8; 12];
    getrandom::fill(&mut bytes).expect("the system's random source");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("plugin:{hex}")
}

fn text_bytes(edit: &PluginEdit) -> usize {
    edit.title.as_ref().map_or(0, String::len)
        + edit.body.as_ref().map_or(0, String::len)
        + edit
            .data
            .as_ref()
            .and_then(Option::as_ref)
            .map_or(0, |v| v.to_string().len())
}

impl Batch {
    pub fn new() -> Batch {
        Batch {
            id: batch_id(),
            edits: Vec::new(),
            creates: Vec::new(),
            bytes: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty() && self.creates.is_empty()
    }

    /// How many notes it changes or makes.
    pub fn writes(&self) -> usize {
        self.edits.len() + self.creates.len()
    }

    /// Stages `edit`, merged into any change already staged for that note.
    /// A second change must be at the same revision as the first; a batch
    /// over [`MAX_WRITES_PER_COMMAND`] notes is refused.
    pub fn stage_edit(&mut self, edit: PluginEdit) -> Result<(), Failure> {
        let bytes = text_bytes(&edit);
        match self.edits.iter_mut().find(|e| e.id == edit.id) {
            Some(staged) => {
                if staged.base_rev != edit.base_rev {
                    return Err(Failure::new(
                        CODE_CONFLICT,
                        "the note was read at another revision",
                    ));
                }
                let PluginEdit {
                    title,
                    body,
                    tags,
                    status,
                    data,
                    ..
                } = edit;
                staged.title = title.or(staged.title.take());
                staged.body = body.or(staged.body.take());
                staged.tags = tags.or(staged.tags.take());
                staged.status = status.or(staged.status.take());
                staged.data = data.or(staged.data.take());
            }
            None => {
                if self.writes() >= MAX_WRITES_PER_COMMAND {
                    return Err(Failure::new(
                        CODE_LIMIT,
                        format!("a command may change at most {MAX_WRITES_PER_COMMAND} notes"),
                    ));
                }
                self.edits.push(edit);
            }
        }
        self.bytes += bytes;
        Ok(())
    }

    /// Stages a new note.
    pub fn stage_create(&mut self, create: PluginCreate) -> Result<(), Failure> {
        if self.writes() >= MAX_WRITES_PER_COMMAND {
            return Err(Failure::new(
                CODE_LIMIT,
                format!("a command may change at most {MAX_WRITES_PER_COMMAND} notes"),
            ));
        }
        self.bytes += create.title.len() + create.body.len();
        self.creates.push(create);
        Ok(())
    }

    /// Whether the person reviews the batch before it is applied: when they
    /// asked to, or when it makes notes, changes more than one note, or
    /// changes a note that isn't the open one. Pure.
    pub fn needs_review(&self, open_note: Option<&str>, ask_before_changing: bool) -> bool {
        !self.is_empty()
            && (ask_before_changing
                || !self.creates.is_empty()
                || self.edits.len() > 1
                || self.edits.iter().any(|e| Some(e.id.as_str()) != open_note))
    }
}

/// How a line of a reviewed note changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Added,
    Removed,
}

/// One note on the review sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewNote {
    /// `None` for a note the batch makes.
    pub id: Option<String>,
    pub title: String,
    /// What changes, in words: "Title", "Text", "Tags", "Status", "Its data".
    pub changes: Vec<&'static str>,
    pub added: usize,
    pub removed: usize,
    /// The changed lines, at most [`MAX_REVIEW_LINES`].
    pub lines: Vec<(LineKind, String)>,
}

/// The changed lines between two texts, and how many were added and
/// removed. Pure.
pub fn changed_lines(before: &str, after: &str) -> (Vec<(LineKind, String)>, usize, usize) {
    let diff = similar::TextDiff::from_lines(before, after);
    let (mut lines, mut added, mut removed) = (Vec::new(), 0, 0);
    for change in diff.iter_all_changes() {
        let kind = match change.tag() {
            similar::ChangeTag::Insert => {
                added += 1;
                LineKind::Added
            }
            similar::ChangeTag::Delete => {
                removed += 1;
                LineKind::Removed
            }
            similar::ChangeTag::Equal => continue,
        };
        if lines.len() < MAX_REVIEW_LINES {
            lines.push((kind, change.value().trim_end_matches('\n').to_owned()));
        }
    }
    (lines, added, removed)
}

/// The review sheet for `batch`, from the notes as they are now.
pub fn review(store: &Store, plugin_id: &str, batch: &Batch) -> Vec<ReviewNote> {
    let mut notes = Vec::new();
    for edit in &batch.edits {
        let Ok(Some(doc)) = store.get(&edit.id) else {
            continue;
        };
        let text = |key: &str| {
            doc.fields
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        let (title, body) = (text("title"), text("body"));
        let mut changes = Vec::new();
        if edit.title.as_ref().is_some_and(|t| *t != title) {
            changes.push("Title");
        }
        if edit.body.as_ref().is_some_and(|b| *b != body) {
            changes.push("Text");
        }
        if edit
            .tags
            .as_ref()
            .is_some_and(|t| Some(&Value::from(t.clone())) != doc.fields.get("tags"))
        {
            changes.push("Tags");
        }
        if edit.status.as_ref().is_some_and(|s| *s != text("status")) {
            changes.push("Status");
        }
        if edit
            .data
            .as_ref()
            .is_some_and(|d| d.as_ref() != plugin_data(&doc.fields, plugin_id))
        {
            changes.push("Its data");
        }
        let (lines, added, removed) = match &edit.body {
            Some(after) => changed_lines(&body, after),
            None => (Vec::new(), 0, 0),
        };
        notes.push(ReviewNote {
            id: Some(edit.id.clone()),
            title: edit.title.clone().unwrap_or(title),
            changes,
            added,
            removed,
            lines,
        });
    }
    for create in &batch.creates {
        let (lines, added, removed) = changed_lines("", &create.body);
        notes.push(ReviewNote {
            id: None,
            title: create.title.clone(),
            changes: vec!["New note"],
            added,
            removed,
            lines,
        });
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn edit(id: &str, body: &str) -> PluginEdit {
        PluginEdit {
            id: id.into(),
            base_rev: "1-a".into(),
            body: Some(body.into()),
            ..PluginEdit::default()
        }
    }

    #[test]
    fn review_is_needed_beyond_the_open_note() {
        let mut batch = Batch::new();
        assert!(!batch.needs_review(Some("note:a"), true));
        batch.stage_edit(edit("note:a", "x")).unwrap();
        assert!(!batch.needs_review(Some("note:a"), false));
        assert!(batch.needs_review(Some("note:a"), true));
        assert!(batch.needs_review(Some("note:b"), false));
        assert!(batch.needs_review(None, false));
        batch.stage_edit(edit("note:b", "y")).unwrap();
        assert!(batch.needs_review(Some("note:a"), false));
    }

    #[test]
    fn a_second_change_to_a_note_merges() {
        let mut batch = Batch::new();
        batch.stage_edit(edit("note:a", "x")).unwrap();
        batch
            .stage_edit(PluginEdit {
                id: "note:a".into(),
                base_rev: "1-a".into(),
                title: Some("T".into()),
                ..PluginEdit::default()
            })
            .unwrap();
        assert_eq!(batch.edits.len(), 1);
        assert_eq!(batch.edits[0].body.as_deref(), Some("x"));
        assert_eq!(batch.edits[0].title.as_deref(), Some("T"));
        let stale = PluginEdit {
            id: "note:a".into(),
            base_rev: "2-b".into(),
            ..PluginEdit::default()
        };
        assert_eq!(batch.stage_edit(stale).unwrap_err().code, CODE_CONFLICT);
    }

    #[test]
    fn batch_ids_are_distinct() {
        assert_ne!(Batch::new().id, Batch::new().id);
        assert!(Batch::new().id.starts_with("plugin:"));
    }

    proptest! {
        /// Invariant: a batch never holds more than the per-command cap,
        /// however many notes a command tries to change.
        #[test]
        fn prop_the_write_cap_holds(n in 0usize..700, creates in 0usize..50) {
            let mut batch = Batch::new();
            for i in 0..n {
                let _ = batch.stage_edit(edit(&format!("note:{i}"), "x"));
            }
            for _ in 0..creates {
                let _ = batch.stage_create(PluginCreate { book_id: "book:a".into(), title: "t".into(), body: String::new() });
            }
            prop_assert!(batch.writes() <= MAX_WRITES_PER_COMMAND);
            prop_assert_eq!(batch.writes(), (n + creates).min(MAX_WRITES_PER_COMMAND));
        }

        /// Invariant: a batch that makes notes or touches a note other than
        /// the open one is always reviewed.
        #[test]
        fn prop_beyond_the_open_note_is_reviewed(ids in prop::collection::vec("note:[a-c]", 1..4), open in prop::option::of("note:[a-c]"), ask in any::<bool>()) {
            let mut batch = Batch::new();
            for id in &ids {
                batch.stage_edit(edit(id, "x")).unwrap();
            }
            let beyond = batch.edits.iter().any(|e| Some(&e.id) != open.as_ref()) || batch.edits.len() > 1;
            if beyond || ask {
                prop_assert!(batch.needs_review(open.as_deref(), ask));
            }
        }

        /// Invariant: the changed lines count every added and removed line,
        /// and an unchanged text has none.
        #[test]
        fn prop_changed_lines_count(before in "([a-c]{0,3}\n){0,6}", after in "([a-c]{0,3}\n){0,6}") {
            let (lines, added, removed) = changed_lines(&before, &after);
            prop_assert_eq!(lines.iter().filter(|(k, _)| *k == LineKind::Added).count(), added.min(MAX_REVIEW_LINES));
            prop_assert!(lines.len() <= MAX_REVIEW_LINES);
            let (same, a, r) = changed_lines(&before, &before);
            prop_assert!(same.is_empty() && a == 0 && r == 0);
            prop_assert!(added + removed > 0 || before == after);
        }
    }
}
