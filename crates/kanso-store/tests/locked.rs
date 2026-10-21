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
