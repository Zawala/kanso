//! Role notebooks, daily notes, templates and nested tags in the store:
//! examples, a model-based test of the daily note's get-or-create, and two
//! devices making the same day's note apart. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

use std::collections::BTreeMap;

use kanso_core::dates::{Date, DateTime};
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::roles::{Role, daily_id};
use kanso_core::{DocType, Query};
use kanso_store::{ApplyOutcome, NewNote, Origin, Put, SearchParams, Store, SyncDoc};
use proptest::prelude::*;
use serde_json::{Value, json};

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn day(d: u32) -> Date {
    Date::new(2026, 10, d).unwrap()
}

fn new(body: &str) -> NewNote {
    NewNote {
        title: String::new(),
        body: body.into(),
        tags: Vec::new(),
    }
}

fn field(store: &Store, id: &str, key: &str) -> Value {
    store
        .get(id)
        .unwrap()
        .map(|doc| doc.fields.get(key).cloned().unwrap_or(Value::Null))
        .unwrap_or(Value::Null)
}

#[test]
fn role_books_are_made_once_and_keep_their_role_when_renamed() {
    let (_dir, store) = temp_store();
    assert_eq!(store.find_role_book(Role::Inbox).unwrap(), None);
    let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
    assert_eq!(inbox, "book:inbox");
    assert_eq!(field(&store, &inbox, "name"), "Inbox");
    assert_eq!(field(&store, &inbox, "kansoRole"), "inbox");
    let rev = store.get(&inbox).unwrap().unwrap().rev;
    // Asking again makes nothing new.
    assert_eq!(store.role_book(Role::Inbox, Origin::Ui).unwrap(), inbox);
    assert_eq!(store.get(&inbox).unwrap().unwrap().rev, rev);
    // Renamed, it is still the Inbox.
    store
        .put(
            DocType::Book,
            json!({"_id": inbox, "name": "Capture"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(store.role_book(Role::Inbox, Origin::Ui).unwrap(), inbox);
    assert_eq!(field(&store, &inbox, "kansoRole"), "inbox");
    assert_eq!(field(&store, &inbox, "name"), "Capture");
    // Deleted, it comes back on next use under the same ID.
    store.delete(&inbox, Origin::Ui).unwrap();
    assert_eq!(store.find_role_book(Role::Inbox).unwrap(), None);
    assert_eq!(store.role_book(Role::Inbox, Origin::Ui).unwrap(), inbox);
    assert_eq!(field(&store, &inbox, "name"), "Inbox");
}

#[test]
fn a_notebook_that_says_it_has_the_role_is_used() {
    let (_dir, store) = temp_store();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:mine", "name": "Daybook", "kansoRole": "journal"}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    assert_eq!(
        store.role_book(Role::Journal, Origin::Ui).unwrap(),
        "book:mine"
    );
    let daily = store.daily_note(day(20), &new(""), Origin::Ui).unwrap();
    assert_eq!(field(&store, &daily.id, "bookId"), "book:mine");
}

#[test]
fn daily_notes_examples() {
    let (_dir, store) = temp_store();
    let made = store
        .daily_note(day(20), &new("## Plan\n"), Origin::Ui)
        .unwrap();
    assert_eq!(made.id, "note:daily-2026-10-20");
    assert!(made.created);
    let note = store.note(&made.id).unwrap().unwrap();
    assert_eq!(
        (
            note.title.as_str(),
            note.body.as_str(),
            note.book_id.as_str()
        ),
        ("2026-10-20", "## Plan\n", "book:journal")
    );
    // Found again, unchanged, whatever the template says now.
    let again = store
        .daily_note(day(20), &new("other"), Origin::Ui)
        .unwrap();
    assert!(!again.created);
    assert_eq!(store.note(&made.id).unwrap().unwrap().body, "## Plan\n");
    // In the trash, it comes back to the Journal.
    store
        .put(
            DocType::Note,
            json!({"_id": made.id, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(store.daily_dates().unwrap().is_empty());
    store.daily_note(day(20), &new(""), Origin::Ui).unwrap();
    assert_eq!(field(&store, &made.id, "bookId"), "book:journal");
    store.daily_note(day(3), &new(""), Origin::Ui).unwrap();
    assert_eq!(store.daily_dates().unwrap(), [day(3), day(20)]);
}

#[test]
fn notes_from_templates() {
    let (_dir, store) = temp_store();
    let templates = store.role_book(Role::Templates, Origin::Ui).unwrap();
    let tag = store.tag_named("meeting", Origin::Ui).unwrap();
    let tpl = store
        .put(
            DocType::Note,
            json!({"title": "Meeting {{date}}", "body": "# {{title}}\n\n- {{cursor}}\n\n{{who}} at {{time:%H}}h",
                   "bookId": templates, "tags": [tag]}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(
        store.templates().unwrap(),
        [(tpl.id.clone(), "Meeting {{date}}".to_owned())]
    );
    let now = DateTime {
        date: day(20),
        hour: 9,
        minute: 41,
        second: 0,
    };
    let made = store
        .create_from_template(&tpl.id, None, None, now, Origin::Api)
        .unwrap();
    let note = store.note(&made.id).unwrap().unwrap();
    assert_eq!(note.title, "Meeting 2026-10-20");
    assert_eq!(note.body, "# Meeting 2026-10-20\n\n- \n\n{{who}} at 09h");
    assert_eq!(made.cursor, Some("# Meeting 2026-10-20\n\n- ".len()));
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.tags, [tag]);
    assert_eq!(note.extra["sourceTemplateId"], tpl.id);
    let titled = store
        .create_from_template(&tpl.id, Some("Standup"), Some(&templates), now, Origin::Api)
        .unwrap();
    let note = store.note(&titled.id).unwrap().unwrap();
    assert_eq!(
        (note.title.as_str(), note.book_id.as_str()),
        ("Standup", templates.as_str())
    );
    assert!(note.body.starts_with("# Standup\n"));
    assert!(
        store
            .create_from_template("note:missing", None, None, now, Origin::Api)
            .is_err()
    );
}

#[test]
fn tag_search_finds_nested_tags_and_tag_named_reuses_tags() {
    let (_dir, store) = temp_store();
    let book = store.role_book(Role::Inbox, Origin::Ui).unwrap();
    let work = store.tag_named("work", Origin::Ui).unwrap();
    let acme = store.tag_named("work/clients/acme", Origin::Ui).unwrap();
    let workshop = store.tag_named("workshop", Origin::Ui).unwrap();
    assert_eq!(store.tag_named("WORK", Origin::Ui).unwrap(), work);
    let note = |tags: Vec<&str>| {
        store
            .put(
                DocType::Note,
                json!({"title": "n", "bookId": book, "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let (a, b, _c) = (note(vec![&work]), note(vec![&acme]), note(vec![&workshop]));
    let found = |q: &str| {
        let mut ids: Vec<String> = store
            .search(&SearchParams {
                query: Query::parse(q),
                ..SearchParams::default()
            })
            .unwrap()
            .into_iter()
            .map(|d| d.id)
            .collect();
        ids.sort();
        ids
    };
    let mut both = vec![a.clone(), b.clone()];
    both.sort();
    assert_eq!(found("tag:work"), both);
    assert_eq!(found("tag:Work/Clients"), std::slice::from_ref(&b));
    assert_eq!(found("tag:work/clients/acme"), [b]);
    assert_eq!(store.tag_links().unwrap().len(), 3);
}

/// Two devices make the same day's note apart; the second to hear of the
/// other's merges both rather than losing either.
#[test]
fn the_same_daily_note_made_on_two_devices_is_merged() {
    let (_a_dir, a) = temp_store();
    let (_b_dir, b) = temp_store();
    let template = "## Plan\n\n## Log\n";
    let id = a
        .daily_note(day(20), &new(template), Origin::Ui)
        .unwrap()
        .id;
    b.daily_note(day(20), &new(template), Origin::Ui).unwrap();
    a.put(
        DocType::Note,
        json!({"_id": id, "body": "## Plan\n- from A\n\n## Log\n"}),
        Put::Merge,
        Origin::Ui,
    )
    .unwrap();
    b.put(
        DocType::Note,
        json!({"_id": id, "body": "## Plan\n- from B\n\n## Log\n"}),
        Put::Merge,
        Origin::Ui,
    )
    .unwrap();
    // A's note (and its Journal) reach B as a server's feed would bring them.
    for doc in a.pending_push(usize::MAX).unwrap() {
        let pushed = SyncDoc {
            base_rev: None,
            ..doc
        };
        let outcome = b.apply_remote(&pushed, None, None).unwrap();
        if pushed.id == id {
            assert!(
                matches!(outcome, ApplyOutcome::LocalWon { .. }),
                "{outcome:?}"
            );
        }
    }
    let body = b.note(&id).unwrap().unwrap().body;
    assert!(body.contains("- from A"), "{body}");
    assert!(body.contains("- from B"), "{body}");
    assert!(body.starts_with("## Plan\n"), "{body}");
    // B's merge is pending, based on A's version, and A's is in history.
    let pending = b
        .pending_push(usize::MAX)
        .unwrap()
        .into_iter()
        .find(|d| d.id == id)
        .unwrap();
    let a_rev = a.get(&id).unwrap().unwrap().rev;
    assert_eq!(pending.base_rev.as_deref(), Some(a_rev.as_str()));
    assert!(b.revisions(&id).unwrap().iter().any(|r| r.rev == a_rev));
    // A, which pushed first, takes the merge as it is.
    a.mark_synced(&id, &a_rev, None).unwrap();
    let outcome = a
        .apply_remote(
            &SyncDoc {
                base_rev: Some(a_rev),
                ..pending
            },
            None,
            None,
        )
        .unwrap();
    assert_eq!(outcome, ApplyOutcome::Applied);
    assert_eq!(a.note(&id).unwrap().unwrap().body, body);
}

/// A note that isn't a daily note keeps the usual rule: the newer one wins.
#[test]
fn other_notes_made_with_the_same_id_still_take_the_newer() {
    let (_a_dir, a) = temp_store();
    let (_b_dir, b) = temp_store();
    for (store, body, at) in [(&a, "A", 100), (&b, "B", 200)] {
        store
            .put(
                DocType::Note,
                json!({"_id": "note:same", "bookId": TRASH_BOOK_ID, "body": body, "updatedAt": at}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap();
    }
    let doc = a
        .pending_push(usize::MAX)
        .unwrap()
        .into_iter()
        .find(|d| d.id == "note:same")
        .unwrap();
    let outcome = b.apply_remote(&doc, None, None).unwrap();
    assert!(matches!(outcome, ApplyOutcome::LocalWon { .. }));
    assert_eq!(b.note("note:same").unwrap().unwrap().body, "B");
}

// MARK: Model-based: the daily note's get-or-create

#[derive(Clone, Debug)]
enum Op {
    /// Today, or another day's note.
    Open(u32),
    Edit(u32, String),
    Trash(u32),
    Delete(u32),
    DeleteJournal,
    RenameJournal(String),
}

fn gen_op() -> impl Strategy<Value = Op> {
    let d = 1u32..5;
    prop_oneof![
        3 => d.clone().prop_map(Op::Open),
        2 => (d.clone(), "[a-z ]{1,8}").prop_map(|(d, s)| Op::Edit(d, s)),
        1 => d.clone().prop_map(Op::Trash),
        1 => d.prop_map(Op::Delete),
        1 => Just(Op::DeleteJournal),
        1 => "[A-Z][a-z]{1,6}".prop_map(Op::RenameJournal),
    ]
}

/// What the model knows of a day's note.
#[derive(Clone, Debug, PartialEq)]
struct Model {
    body: String,
    trashed: bool,
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// Model-based: whatever happened before, opening a day gives its one
    /// note, live and outside the trash, with the text it had (or the
    /// template's, when it is new), in the Journal unless it was there
    /// already; and the Journal is one notebook with the journal role.
    #[test]
    fn prop_daily_get_or_create_follows_the_model(ops in prop::collection::vec(gen_op(), 1..24)) {
        let (_dir, store) = temp_store();
        let mut model: BTreeMap<u32, Model> = BTreeMap::new();
        let template = "## Today\n";
        for op in ops {
            match op {
                Op::Open(d) => {
                    let made = store.daily_note(day(d), &new(template), Origin::Ui).unwrap();
                    prop_assert_eq!(&made.id, &daily_id(day(d)));
                    prop_assert_eq!(made.created, !model.contains_key(&d));
                    let entry = model.entry(d).or_insert(Model { body: template.into(), trashed: false });
                    entry.trashed = false;
                    let note = store.note(&made.id).unwrap().unwrap();
                    prop_assert_eq!(&note.body, &entry.body);
                    prop_assert_ne!(note.book_id.as_str(), TRASH_BOOK_ID);
                    prop_assert_eq!(note.title, day(d).iso());
                    let journal = store.find_role_book(Role::Journal).unwrap();
                    prop_assert!(journal.is_some());
                    if made.created {
                        prop_assert_eq!(Some(note.book_id), journal);
                    }
                }
                Op::Edit(d, text) => {
                    if model.get(&d).is_some_and(|m| !m.trashed) {
                        store.put(DocType::Note, json!({"_id": daily_id(day(d)), "body": text}), Put::Merge, Origin::Ui).unwrap();
                        model.get_mut(&d).unwrap().body = text;
                    }
                }
                Op::Trash(d) => {
                    if let Some(m) = model.get_mut(&d) {
                        store.put(DocType::Note, json!({"_id": daily_id(day(d)), "bookId": TRASH_BOOK_ID}), Put::Merge, Origin::Ui).unwrap();
                        m.trashed = true;
                    }
                }
                Op::Delete(d) => {
                    if model.remove(&d).is_some() {
                        store.delete(&daily_id(day(d)), Origin::Ui).unwrap();
                    }
                }
                Op::DeleteJournal => {
                    if let Some(journal) = store.find_role_book(Role::Journal).unwrap() {
                        store.delete(&journal, Origin::Ui).unwrap();
                        // Its notes go to the trash.
                        for m in model.values_mut() {
                            m.trashed = true;
                        }
                    }
                }
                Op::RenameJournal(name) => {
                    if let Some(journal) = store.find_role_book(Role::Journal).unwrap() {
                        store.put(DocType::Book, json!({"_id": journal, "name": name}), Put::Merge, Origin::Ui).unwrap();
                    }
                }
            }
            // The days with a live note outside the trash are the model's.
            let live: Vec<Date> = model.iter().filter(|(_, m)| !m.trashed).map(|(d, _)| day(*d)).collect();
            prop_assert_eq!(store.daily_dates().unwrap(), live);
            // At most one notebook has the journal role.
            let journals = store.books().unwrap().into_iter()
                .filter(|b| b.extra.get("kansoRole").and_then(Value::as_str) == Some("journal"))
                .count();
            prop_assert!(journals <= 1);
        }
    }
}

#[test]
fn daily_notes_are_made_from_the_journals_template() {
    let (_dir, store) = temp_store();
    assert_eq!(store.daily_template().unwrap(), None);
    let templates = store.role_book(Role::Templates, Origin::Ui).unwrap();
    let tpl = store
        .put(
            DocType::Note,
            json!({"title": "Daily", "body": "# {{date:%A, %e %B}}\n\n{{cursor}}\n\nMade at {{time}} for {{title}}",
                   "bookId": templates}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store.set_daily_template(Some(&tpl.id), Origin::Ui).unwrap();
    assert_eq!(store.daily_template().unwrap(), Some(tpl.id.clone()));
    assert_eq!(field(&store, "book:journal", "kansoDailyTemplate"), tpl.id);
    let now = DateTime {
        date: day(20),
        hour: 7,
        minute: 5,
        second: 0,
    };
    let daily = store.open_daily(now, Origin::Ui).unwrap();
    let note = store.note(&daily.id).unwrap().unwrap();
    assert_eq!(
        note.body,
        "# Tuesday, 20 October\n\n\n\nMade at 07:05 for 2026-10-20"
    );
    assert_eq!(daily.cursor, Some("# Tuesday, 20 October\n\n".len()));
    // Found again: no cursor, nothing rewritten.
    let again = store.open_daily(now, Origin::Ui).unwrap();
    assert_eq!((again.created, again.cursor), (false, None));
    // A template in the trash is no template.
    store
        .put(
            DocType::Note,
            json!({"_id": tpl.id, "bookId": TRASH_BOOK_ID}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(store.daily_template().unwrap(), None);
    let next = store
        .open_daily(
            DateTime {
                date: day(21),
                ..now
            },
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(store.note(&next.id).unwrap().unwrap().body, "");
    store.set_daily_template(None, Origin::Ui).unwrap();
    assert_eq!(
        field(&store, "book:journal", "kansoDailyTemplate"),
        Value::Null
    );
}
