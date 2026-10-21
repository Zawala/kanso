//! Locked notes: the store keeps only the sealed text (`kansoLock`), and
//! locking a note forgets what it said before, on disk too.

use kanso_core::{DocType, Query};
use kanso_store::{Origin, Put, SearchParams, Store, StoreError};
use serde_json::{Value, json};

const CANARY: &str = "zebracanary4471";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

/// A note with the canary in its text, edited once so it has history.
fn note_with_history(store: &Store) -> Value {
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let body = format!("# Plans\n\n{CANARY} - [ ] call [[Sam]] @2026-10-25");
    let first = store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": "Plans", "body": body}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let second = store
        .put(
            DocType::Note,
            json!({"_id": first.id, "_rev": first.rev, "bookId": book, "title": "Plans",
                   "body": format!("{body} and more")}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    assert!(!store.revisions(&first.id).unwrap().is_empty());
    let mut note = store.get_api(&first.id, None, false).unwrap();
    note["_rev"] = second.rev.into();
    note
}

fn locked(mut note: Value) -> Value {
    note["body"] = "".into();
    note["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    note
}

fn search(store: &Store, text: &str) -> usize {
    let params = SearchParams {
        query: Query::parse(text),
        ..SearchParams::default()
    };
    store.search(&params).unwrap().len()
}

fn files_contain(dir: &std::path::Path, needle: &str) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).ok()?;
            bytes
                .windows(needle.len())
                .any(|w| w == needle.as_bytes())
                .then(|| path.display().to_string())
        })
        .collect()
}

#[test]
fn locking_forgets_the_text_in_search_links_tasks_history_and_on_disk() {
    let (dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    assert_eq!(search(&store, CANARY), 1);
    assert!(!files_contain(dir.path(), CANARY).is_empty());

    store.lock_note(locked(note)).unwrap();

    assert_eq!(search(&store, CANARY), 0);
    // The title stays: it shows in the list and wiki-links reach it.
    assert_eq!(search(&store, "title:Plans"), 1);
    assert!(store.revisions(&id).unwrap().is_empty());
    assert_eq!(store.derived_rows(&id).unwrap(), (0, 0, 0));
    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    assert!(summaries[0].locked);
    assert_eq!(summaries[0].snippet, "");
    // Nothing of the old text is left in the database or its logs.
    assert_eq!(files_contain(dir.path(), CANARY), Vec::<String>::new());
}

#[test]
fn only_a_locked_note_can_be_written_by_lock_note() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    assert!(matches!(store.lock_note(note), Err(StoreError::Invalid(_))));
}

#[test]
fn local_writes_cant_put_text_beside_the_lock_but_sync_can() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = store.lock_note(locked(note)).unwrap().rev;

    for origin in [Origin::Ui, Origin::Api, Origin::Import] {
        let result = store.put(
            DocType::Note,
            json!({"_id": id, "_rev": rev, "body": "plain text"}),
            Put::Merge,
            origin,
        );
        assert!(matches!(result, Err(StoreError::Invalid(_))), "{origin:?}");
    }
    // Moving it, retitling it or tagging it is fine.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": "Later"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    // A plain-text version can't be slipped into its history either.
    let fields = json!({"title": "Later", "body": "plain text"});
    assert!(matches!(
        store.record_revision(&id, &rev, fields.as_object().unwrap(), Origin::Ui),
        Err(StoreError::Invalid(_))
    ));
    // Sync takes what another device wrote, whatever it is.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "from an older Kanso"}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

/// The note as another device sealed it: `salt` names the key, `data` the
/// sealed text.
fn sealed_by(note: &Value, rev: &str, salt: &str, data: &str) -> Value {
    let mut note = note.clone();
    note["_rev"] = rev.into();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": salt, "m": 64, "t": 1, "p": 1, "data": data});
    note
}

#[test]
fn a_lock_arriving_by_sync_forgets_the_plain_text_here_too() {
    let (dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = note["_rev"].as_str().unwrap().to_owned();
    assert!(!store.revisions(&id).unwrap().is_empty());

    store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "c2FsdA", "sealed"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();

    // Locked on another device: no plain version stays in this one's history,
    // search or files.
    assert!(store.revisions(&id).unwrap().is_empty());
    assert_eq!(search(&store, CANARY), 0);
    assert_eq!(files_contain(dir.path(), CANARY), Vec::<String>::new());
}

#[test]
fn a_new_lock_password_forgets_versions_sealed_under_the_old_one() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = store.lock_note(locked(note.clone())).unwrap().rev;
    // Edits under the same key keep their sealed versions.
    let rev = store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "c2FsdA", "edited"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap()
        .rev;
    assert_eq!(store.revisions(&id).unwrap().len(), 1);
    // The password changed on another device: the old key's versions go.
    store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "bmV3c2FsdA", "rekeyed"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    assert!(store.revisions(&id).unwrap().is_empty());
}

#[test]
fn text_beside_a_lock_is_never_searched_linked_or_served() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    store.lock_note(locked(note)).unwrap();
    // An older Kanso typed into the note it showed as empty.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": format!("{CANARY} - [ ] call [[Sam]]")}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();

    assert_eq!(search(&store, CANARY), 0);
    assert_eq!(store.derived_rows(&id).unwrap(), (0, 0, 0));
    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    assert_eq!(summaries[0].snippet, "");
    let served = store.get_api(&id, None, false).unwrap();
    assert_eq!(served["body"], "");
    // What it typed is kept as it came, for the app to deal with.
    assert!(store.note(&id).unwrap().unwrap().body.contains(CANARY));
}

#[test]
fn the_api_never_serves_the_sealed_text() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    store.lock_note(locked(note)).unwrap();
    let served = store.get_api(&id, None, false).unwrap();
    assert!(served.get("kansoLock").is_none());
    assert_eq!(served["body"], "");
    assert_eq!(served["title"], "Plans");
    for listed in store.list_api(DocType::Note, None, 0).unwrap() {
        assert!(listed.get("kansoLock").is_none());
    }
}
