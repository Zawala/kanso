//! One change to several notes at once: the selected notes from the list's
//! menu or the palette, or the notes dropped on a notebook or tag in the
//! sidebar. Every note changes in one write, and a notice offers Undo,
//! which puts back what each note had, in one write too.

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, NoteStatus};
use kanso_store::{NoteSummary, Origin};
use serde_json::{Value, json};

use crate::library::Library;

/// What happens to every note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// Into this notebook (the trash's ID moves them to the trash).
    Move(String),
    /// Adds this tag to each note that hasn't it.
    AddTag(String),
    /// Adds this tag to every note, or takes it off every note when they
    /// all have it, as choosing a checked tag does for one note.
    ToggleTag(String),
    /// Pins every note, or unpins them when they are all pinned.
    TogglePin,
    Status(NoteStatus),
}

/// One note's write, and the write that undoes it.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub id: String,
    pub change: Value,
    pub undo: Value,
}

/// The writes `change` takes for `notes`: one for each note it changes,
/// none for a note that is already as asked.
pub fn plan(notes: &[NoteSummary], change: &Change) -> Vec<Edit> {
    let all_have = |tag: &str| notes.iter().all(|n| n.tags.iter().any(|t| t == tag));
    let all_pinned = notes.iter().all(|n| n.pinned);
    notes
        .iter()
        .filter_map(|note| {
            let (field, before, after) = match change {
                Change::Move(book) => ("bookId", json!(note.book_id), json!(book)),
                Change::AddTag(tag) | Change::ToggleTag(tag) => {
                    let has = note.tags.iter().any(|t| t == tag);
                    let off = matches!(change, Change::ToggleTag(_)) && all_have(tag);
                    let tags: Vec<&String> = if off {
                        note.tags.iter().filter(|t| *t != tag).collect()
                    } else if has {
                        note.tags.iter().collect()
                    } else {
                        note.tags.iter().chain([tag]).collect()
                    };
                    ("tags", json!(note.tags), json!(tags))
                }
                Change::TogglePin => ("pinned", json!(note.pinned), json!(!all_pinned)),
                Change::Status(status) => (
                    "status",
                    json!(note.status.as_str()),
                    json!(status.as_str()),
                ),
            };
            (before != after).then(|| Edit {
                id: note.id.clone(),
                change: json!({ "_id": note.id, field: after }),
                undo: json!({ "_id": note.id, field: before }),
            })
        })
        .collect()
}

/// What the notice says once `count` notes changed. `place` names the
/// notebook, tag or status the change is about; `on` is whether a toggle
/// turned the tag or pin on.
pub fn summary(change: &Change, count: usize, place: &str, on: bool) -> String {
    let notes = if count == 1 {
        "1 note".to_owned()
    } else {
        format!("{count} notes")
    };
    match change {
        Change::Move(book) if book == TRASH_BOOK_ID => format!("Moved {notes} to the trash."),
        Change::Move(_) => format!("Moved {notes} to {place}."),
        Change::AddTag(_) => format!("Added #{place} to {notes}."),
        Change::ToggleTag(_) if on => format!("Added #{place} to {notes}."),
        Change::ToggleTag(_) => format!("Took #{place} off {notes}."),
        Change::TogglePin if on => format!("Pinned {notes}."),
        Change::TogglePin => format!("Unpinned {notes}."),
        Change::Status(_) => format!("Set {notes} to {place}."),
    }
}

/// What of an undo still applies to a note that is `current` now (as
/// JSON): the fields still holding what the batch wrote (`change`) go
/// back to `undo`'s; a field changed again since keeps its newer value.
/// `None` when nothing is left to put back. Pure.
pub fn undo_now(change: &Value, undo: &Value, current: &Value) -> Option<Value> {
    let mut back = serde_json::Map::new();
    for (key, value) in undo.as_object()? {
        if key != "_id" && current.get(key) == change.get(key) {
            back.insert(key.clone(), value.clone());
        }
    }
    if back.is_empty() {
        return None;
    }
    back.insert("_id".into(), undo.get("_id")?.clone());
    Some(Value::Object(back))
}

/// Whether the notes must go one at a time: unlocked notes into a locked
/// notebook are sealed as they go, which only the editor does.
pub fn needs_one_by_one(notes: &[NoteSummary], change: &Change, library: &Library) -> bool {
    matches!(change, Change::Move(book) if library.is_book_locked(book))
        && notes.iter().any(|n| !n.locked)
}

/// Makes `change` to the listed notes among `ids`, in one write, and says
/// so with Undo. Typing waiting in any editor is saved first, so an open
/// note takes the change like any other.
pub fn apply(
    ids: &[String],
    change: Change,
    library: &Entity<Library>,
    window: &mut Window,
    cx: &mut App,
) {
    let (notes, place) = {
        let library = library.read(cx);
        let notes: Vec<NoteSummary> = library
            .notes
            .iter()
            .filter(|n| ids.contains(&n.id))
            .cloned()
            .collect();
        if needs_one_by_one(&notes, &change, library) {
            window.push_notification(
                Notification::info(
                    "Notes go into a locked notebook one at a time: open the note and choose Move to notebook….",
                ),
                cx,
            );
            return;
        }
        let place = match &change {
            Change::Move(book) => library.book(book).map(|b| b.name.clone()),
            // A tag just made isn't in the library until it reloads.
            Change::AddTag(tag) | Change::ToggleTag(tag) => {
                library.tag(tag).map(|t| t.name.clone()).or_else(|| {
                    let tags = library.store.tags().ok()?;
                    tags.into_iter().find(|t| &t.id == tag).map(|t| t.name)
                })
            }
            Change::Status(status) => Some(status.label().to_owned()),
            Change::TogglePin => None,
        };
        (notes, place.unwrap_or_default())
    };
    let edits = plan(&notes, &change);
    if edits.is_empty() {
        return;
    }
    let on = match &change {
        Change::TogglePin => edits[0].change["pinned"] == json!(true),
        Change::ToggleTag(tag) => edits[0].change["tags"]
            .as_array()
            .is_some_and(|tags| tags.iter().any(|t| t == tag)),
        _ => true,
    };
    crate::editor::peers::flush_everywhere(cx);
    let store = library.read(cx).store.clone();
    let changes: Vec<Value> = edits.iter().map(|edit| edit.change.clone()).collect();
    let count = changes.len();
    if let Err(err) = store.merge_many(DocType::Note, changes, Origin::Ui) {
        window.push_notification(
            Notification::error(format!("Couldn't change the notes: {err}")),
            cx,
        );
        return;
    }
    let notification =
        // Each batch's notice takes the place of the one before, so its
        // Undo is always for the latest change.
        Notification::info(summary(&change, count, &place, on))
            .id::<Edit>()
            .action(move |_, _, _| {
            let (store, edits) = (store.clone(), edits.clone());
            Button::new("undo-notes")
                .label("Undo")
                .debug_selector(|| "undo-notes".into())
                .on_click(move |_, window, cx| {
                    crate::editor::peers::flush_everywhere(cx);
                    // A note deleted since is skipped, and so is what was
                    // changed again after the batch.
                    let undo: Vec<Value> = edits
                        .iter()
                        .filter_map(|edit| {
                            let note = store.note(&edit.id).ok().flatten()?;
                            let current = serde_json::to_value(note).ok()?;
                            undo_now(&edit.change, &edit.undo, &current)
                        })
                        .collect();
                    if undo.is_empty() {
                        return;
                    }
                    if let Err(err) = store.merge_many(DocType::Note, undo, Origin::Ui) {
                        window.push_notification(
                            Notification::error(format!("Couldn't undo: {err}")),
                            cx,
                        );
                    }
                })
        });
    window.push_notification(notification, cx);
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use kanso_core::NoteStatus;
    use kanso_core::model::TRASH_BOOK_ID;
    use kanso_store::NoteSummary;
    use serde_json::{Value, json};

    use super::{Change, plan, summary, undo_now};

    fn note(id: usize, book: &str, tags: &[&str], pinned: bool, status: NoteStatus) -> NoteSummary {
        NoteSummary {
            id: format!("note:{id}"),
            rev: "1-a".into(),
            title: format!("n{id}"),
            snippet: String::new(),
            book_id: book.into(),
            status,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            pinned,
            updated_at: 0,
            created_at: 0,
            tasks_done: 0,
            tasks_total: 0,
            locked: false,
        }
    }

    /// Writes `edit`'s fields into the matching note, as a merge would.
    fn merge(notes: &mut [NoteSummary], fields: &Value) {
        let note = notes
            .iter_mut()
            .find(|n| fields["_id"] == json!(n.id))
            .expect("a write for a listed note");
        for (key, value) in fields.as_object().unwrap() {
            match key.as_str() {
                "_id" => {}
                "bookId" => note.book_id = value.as_str().unwrap().into(),
                "tags" => note.tags = serde_json::from_value(value.clone()).unwrap(),
                "pinned" => note.pinned = value.as_bool().unwrap(),
                "status" => note.status = NoteStatus::parse(value.as_str().unwrap()).unwrap(),
                other => panic!("unexpected field {other}"),
            }
        }
    }

    fn notes() -> impl Strategy<Value = Vec<NoteSummary>> {
        let one = (
            prop::sample::select(vec!["book:a", "book:b"]),
            prop::collection::btree_set(prop::sample::select(vec!["t1", "t2", "t3"]), 0..3),
            any::<bool>(),
            prop::sample::select(NoteStatus::ALL.to_vec()),
        );
        prop::collection::vec(one, 0..8).prop_map(|notes| {
            notes
                .into_iter()
                .enumerate()
                .map(|(ix, (book, tags, pinned, status))| {
                    let tags: Vec<&str> = tags.into_iter().collect();
                    note(ix, book, &tags, pinned, status)
                })
                .collect()
        })
    }

    fn change() -> impl Strategy<Value = Change> {
        prop_oneof![
            prop::sample::select(vec!["book:a", "book:b", TRASH_BOOK_ID])
                .prop_map(|b| Change::Move(b.into())),
            prop::sample::select(vec!["t1", "t2"]).prop_map(|t| Change::AddTag(t.into())),
            prop::sample::select(vec!["t1", "t2"]).prop_map(|t| Change::ToggleTag(t.into())),
            Just(Change::TogglePin),
            prop::sample::select(NoteStatus::ALL.to_vec()).prop_map(Change::Status),
        ]
    }

    /// Whether `note` is as `change` asks, given whether a toggle turned on.
    fn is_as_asked(note: &NoteSummary, change: &Change, on: bool) -> bool {
        match change {
            Change::Move(book) => &note.book_id == book,
            Change::AddTag(tag) => note.tags.contains(tag),
            Change::ToggleTag(tag) => note.tags.contains(tag) == on,
            Change::TogglePin => note.pinned == on,
            Change::Status(status) => note.status == *status,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: afterwards every note is as asked, a tag never shows
        /// twice on a note, and only notes that weren't already are
        /// written, once each. Round trip: Undo brings back every note
        /// exactly as it was. Idempotence: asking again for a move, tag or
        /// status writes nothing; a toggle asked again flips back.
        #[test]
        fn prop_a_batch_changes_all_and_undoes_exactly(before in notes(), change in change()) {
            check_batch(&before, &change)?;
        }

        /// Invariant: a late Undo puts back exactly the fields that still
        /// hold the batch's value, and never touches one changed since.
        /// With nothing changed since, it is the whole undo.
        #[test]
        fn prop_a_late_undo_keeps_later_changes(
            book_kept in any::<bool>(),
            pinned_kept in any::<bool>(),
        ) {
            let change = json!({"_id": "note:1", "bookId": "b2", "pinned": true});
            let undo = json!({"_id": "note:1", "bookId": "b1", "pinned": false});
            let current = json!({
                "_id": "note:1",
                "bookId": if book_kept { "b2" } else { "b3" },
                "pinned": pinned_kept,
            });
            let back = undo_now(&change, &undo, &current);
            match back {
                None => prop_assert!(!book_kept && !pinned_kept),
                Some(back) => {
                    prop_assert_eq!(&back["_id"], "note:1");
                    prop_assert_eq!(back.get("bookId").is_some(), book_kept);
                    prop_assert_eq!(back.get("pinned").is_some(), pinned_kept);
                    if book_kept && pinned_kept {
                        prop_assert_eq!(&back, &undo);
                    }
                }
            }
        }
    }

    /// The batch property's body, out of the macro, which can't expand
    /// this much.
    fn check_batch(before: &[NoteSummary], change: &Change) -> Result<(), TestCaseError> {
        let edits = plan(before, change);
        let mut after = before.to_vec();
        for edit in &edits {
            merge(&mut after, &edit.change);
        }
        let on = match change {
            Change::ToggleTag(tag) => !before.iter().all(|n| n.tags.contains(tag)),
            Change::TogglePin => !before.iter().all(|n| n.pinned),
            _ => true,
        };
        for (old, new) in before.iter().zip(&after) {
            prop_assert!(is_as_asked(new, change, on), "{new:?} after {change:?}");
            let unique: std::collections::BTreeSet<_> = new.tags.iter().collect();
            prop_assert_eq!(unique.len(), new.tags.len());
            let written = edits.iter().filter(|e| e.id == old.id).count();
            prop_assert_eq!(written, usize::from(old != new));
        }

        let mut undone = after.clone();
        for edit in &edits {
            merge(&mut undone, &edit.undo);
        }
        prop_assert_eq!(&undone[..], before);

        let again = plan(&after, change);
        match change {
            Change::ToggleTag(_) | Change::TogglePin if !after.is_empty() => {
                let mut back = after.clone();
                for edit in &again {
                    merge(&mut back, &edit.change);
                }
                // Back to all off when it turned on; when it turned off,
                // every note gets it.
                for note in &back {
                    prop_assert!(is_as_asked(note, change, !on));
                }
            }
            _ => prop_assert!(again.is_empty()),
        }
        Ok(())
    }

    #[test]
    fn examples() {
        let notes = vec![
            note(1, "book:a", &["t1"], true, NoteStatus::None),
            note(2, "book:b", &[], false, NoteStatus::Active),
        ];
        let moved = plan(&notes, &Change::Move("book:b".into()));
        assert_eq!(moved.len(), 1, "the note already there isn't written");
        assert_eq!(
            moved[0].change,
            json!({"_id": "note:1", "bookId": "book:b"})
        );
        assert_eq!(moved[0].undo, json!({"_id": "note:1", "bookId": "book:a"}));

        let pinned = plan(&notes, &Change::TogglePin);
        assert_eq!(pinned.len(), 1, "one unpinned: all pin");
        assert_eq!(pinned[0].change["pinned"], json!(true));

        let tagged = plan(&notes, &Change::ToggleTag("t1".into()));
        assert_eq!(tagged[0].change["tags"], json!(["t1"]));

        assert_eq!(
            summary(&Change::Move(TRASH_BOOK_ID.into()), 3, "", true),
            "Moved 3 notes to the trash."
        );
        assert_eq!(
            summary(&Change::Move("book:a".into()), 1, "Work", true),
            "Moved 1 note to Work."
        );
        assert_eq!(
            summary(&Change::ToggleTag("t".into()), 2, "ideas", false),
            "Took #ideas off 2 notes."
        );
        assert_eq!(
            summary(&Change::TogglePin, 2, "", false),
            "Unpinned 2 notes."
        );
    }

    #[test]
    fn undo_after_a_tag_was_typed_leaves_the_tags() {
        let change = json!({"_id": "note:1", "tags": ["a", "b"]});
        let undo = json!({"_id": "note:1", "tags": ["a"]});
        let current = json!({"_id": "note:1", "tags": ["a", "b", "c"]});
        assert_eq!(undo_now(&change, &undo, &current), None);
        let current = json!({"_id": "note:1", "tags": ["a", "b"]});
        assert_eq!(undo_now(&change, &undo, &current), Some(undo));
    }
}
