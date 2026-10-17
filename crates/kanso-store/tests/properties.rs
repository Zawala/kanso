//! Property tests for the store. CI pins the seed with `PROPTEST_RNG_SEED`.

use std::collections::HashMap;

use kanso_core::tasks::count_tasks;
use kanso_core::{Clause, DocType, Field, NoteFacts, NoteStatus, Query, Rev};
use kanso_store::{Origin, Put, SearchParams, Store, StoreError};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

const BOOKS: [(&str, &str); 3] = [
    ("book:b0", "Alpha"),
    ("book:b1", "beta Book"),
    ("book:b2", "Gamma"),
];
const TAGS: [(&str, &str); 3] = [
    ("tag:t0", "red"),
    ("tag:t1", "Green"),
    ("tag:t2", "blue tag"),
];
/// Words mixing cases, sub-trigram lengths, non-ASCII and ID-like characters.
const WORDS: [&str; 11] = [
    "apple",
    "Banana",
    "cherry",
    "date",
    "ELDER",
    "fig",
    "go",
    "inkdrop://note/x_1",
    "a-b",
    "über",
    "x%y",
];
/// Fragments of the words above, to exercise substring matching.
const FRAGMENTS: [&str; 8] = ["ban", "RRY", "app", "ld", "üb", "note/x_", "%", "a-"];

#[derive(Clone, Debug)]
struct NoteSpec {
    title: Vec<usize>,
    body: Vec<usize>,
    /// Index into BOOKS, or 3 for the trash.
    book: usize,
    tags: Vec<usize>,
    status: NoteStatus,
}

fn gen_note() -> impl Strategy<Value = NoteSpec> {
    (
        prop::collection::vec(0..WORDS.len(), 0..3),
        prop::collection::vec(0..WORDS.len(), 0..6),
        0..4usize,
        prop::sample::subsequence(vec![0usize, 1, 2], 0..=3),
        prop::sample::select(NoteStatus::ALL.to_vec()),
    )
        .prop_map(|(title, body, book, tags, status)| NoteSpec {
            title,
            body,
            book,
            tags,
            status,
        })
}

fn gen_term() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(WORDS.to_vec()).prop_map(str::to_owned),
        prop::sample::select(FRAGMENTS.to_vec()).prop_map(str::to_owned),
    ]
}

fn gen_clause() -> impl Strategy<Value = Clause> {
    let field_value = prop_oneof![
        gen_term().prop_map(|v| (None, v)),
        gen_term().prop_map(|v| (Some(Field::Title), v)),
        gen_term().prop_map(|v| (Some(Field::Body), v)),
        prop::sample::select(vec!["alpha", "BETA BOOK", "Gamma", "trash", "nope"])
            .prop_map(|v| (Some(Field::Book), v.to_owned())),
        prop::sample::select(vec!["b0", "book:b1", "b2", "trash"])
            .prop_map(|v| (Some(Field::BookId), v.to_owned())),
        prop::sample::select(vec!["RED", "green", "Blue Tag", "none"])
            .prop_map(|v| (Some(Field::Tag), v.to_owned())),
        prop::sample::select(vec!["tag:t0", "t1", "tag:t2"])
            .prop_map(|v| (Some(Field::TagId), v.to_owned())),
        prop::sample::select(vec!["active", "ONHOLD", "none", "completed", "dropped"])
            .prop_map(|v| (Some(Field::Status), v.to_owned())),
    ];
    (any::<bool>(), field_value).prop_map(|(negated, (field, value))| Clause {
        negated,
        field,
        value,
    })
}

fn seed(store: &Store, notes: &[NoteSpec]) -> Vec<String> {
    for (i, (id, name)) in BOOKS.iter().enumerate() {
        let parent = if i == 2 {
            json!("book:b0")
        } else {
            json!(null)
        };
        store
            .put(
                DocType::Book,
                json!({"_id": id, "name": name, "parentBookId": parent}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap();
    }
    for (id, name) in TAGS {
        store
            .put(
                DocType::Tag,
                json!({"_id": id, "name": name}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap();
    }
    notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let words = |ix: &[usize]| ix.iter().map(|&w| WORDS[w]).collect::<Vec<_>>().join(" ");
            let book = if n.book == 3 {
                "trash"
            } else {
                BOOKS[n.book].0
            };
            let tags: Vec<&str> = n.tags.iter().map(|&t| TAGS[t].0).collect();
            store
                .put(
                    DocType::Note,
                    json!({"title": words(&n.title), "body": words(&n.body), "bookId": book,
                           "tags": tags, "status": n.status.as_str(), "updatedAt": i}),
                    Put::Upsert,
                    Origin::Api,
                )
                .unwrap()
                .id
        })
        .collect()
}

/// The notes `query` should find, decided by `Query::matches`.
fn oracle(store: &Store, ids: &[String], query: &Query) -> Vec<String> {
    let book_names: HashMap<&str, &str> = BOOKS.into_iter().collect();
    let tag_names: HashMap<&str, &str> = TAGS.into_iter().collect();
    let mut expected: Vec<String> = ids
        .iter()
        .filter(|id| {
            let note = store.note(id).unwrap().unwrap();
            let names: Vec<String> = note
                .tags
                .iter()
                .filter_map(|t| tag_names.get(t.as_str()).map(|n| n.to_string()))
                .collect();
            let facts = NoteFacts {
                title: &note.title,
                body: &note.body,
                book_id: &note.book_id,
                book_name: book_names.get(note.book_id.as_str()).copied(),
                status: note.status.as_str(),
                tag_ids: &note.tags,
                tag_names: &names,
            };
            (query.includes_trash() || !note.is_trashed()) && query.matches(&facts)
        })
        .cloned()
        .collect();
    expected.sort();
    expected
}

#[derive(Clone, Debug)]
enum RevChoice {
    Current,
    Stale,
    Missing,
}

#[derive(Clone, Debug)]
enum Op {
    Put { body: u8, rev: RevChoice },
    Delete,
}

fn gen_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0u8..3, prop_oneof![Just(RevChoice::Current), Just(RevChoice::Stale), Just(RevChoice::Missing)])
            .prop_map(|(body, rev)| Op::Put { body, rev }),
        1 => Just(Op::Delete),
    ]
}

proptest! {
    #![proptest_config(config(128))]

    /// Model-based (oracle): SQL search returns exactly the notes
    /// `Query::matches` accepts, with trash excluded unless asked for.
    #[test]
    fn prop_search_matches_the_query_oracle(
        notes in prop::collection::vec(gen_note(), 0..16),
        clauses in prop::collection::vec(gen_clause(), 0..4),
    ) {
        let (_dir, store) = temp_store();
        let ids = seed(&store, &notes);
        let query = Query { clauses };
        let mut found: Vec<String> = store
            .search(&SearchParams { query: query.clone(), ..SearchParams::default() })
            .unwrap()
            .into_iter()
            .map(|d| d.id)
            .collect();
        found.sort();
        prop_assert_eq!(found, oracle(&store, &ids, &query), "query: {}", query);
    }

    /// Model-based (state machine): replacing a note succeeds only with its
    /// current revision, deleting needs a live note, and every effective write
    /// bumps the revision generation by exactly one.
    #[test]
    fn prop_rev_rules_follow_the_model(ops in prop::collection::vec(gen_op(), 1..20)) {
        let (_dir, store) = temp_store();
        store.put(DocType::Book, json!({"_id": "book:b", "name": "B"}), Put::Upsert, Origin::Api).unwrap();
        let id = "note:sm1";
        // Model: generation of the latest revision, whether the note is live, and its body.
        let mut generation = 0u64;
        let mut live: Option<u8> = None;
        let mut current_rev: Option<String> = None;
        for op in ops {
            match op {
                Op::Put { body, rev } => {
                    let mut doc = json!({"_id": id, "bookId": "book:b", "body": format!("b{body}"), "updatedAt": 1, "createdAt": 1});
                    match (&rev, &current_rev) {
                        (RevChoice::Current, Some(r)) => doc["_rev"] = r.clone().into(),
                        (RevChoice::Stale, _) => doc["_rev"] = format!("{}-{}", generation.max(1), "f".repeat(32)).into(),
                        _ => {}
                    }
                    let result = store.put(DocType::Note, doc, Put::Upsert, Origin::Api);
                    let accepted = live.is_none() || (matches!(rev, RevChoice::Current) && current_rev.is_some());
                    if accepted {
                        let written = result.unwrap();
                        if live != Some(body) {
                            generation += 1;
                        }
                        live = Some(body);
                        prop_assert_eq!(written.rev.parse::<Rev>().unwrap().generation, generation);
                        current_rev = Some(written.rev);
                    } else {
                        prop_assert!(matches!(result, Err(StoreError::Conflict)), "{:?}", result);
                    }
                }
                Op::Delete => {
                    let result = store.delete(id, Origin::Api);
                    if live.is_some() {
                        generation += 1;
                        prop_assert_eq!(result.unwrap().rev.parse::<Rev>().unwrap().generation, generation);
                        live = None;
                        current_rev = None;
                    } else {
                        prop_assert!(matches!(result, Err(StoreError::NotFound)));
                    }
                }
            }
            prop_assert_eq!(store.get(id).unwrap().is_some(), live.is_some());
        }
    }

    /// Round-trip: a note reads back with the fields it was written with, and
    /// its task counts are derived from the body.
    #[test]
    fn prop_note_roundtrip(spec in gen_note(), pinned in any::<bool>(), extra in "[a-z]{1,8}") {
        let (_dir, store) = temp_store();
        seed(&store, &[]);
        let body = spec.body.iter().map(|&w| WORDS[w]).collect::<Vec<_>>().join("\n- [ ] ");
        let tags: Vec<String> = spec.tags.iter().map(|&t| TAGS[t].0.to_owned()).collect();
        let book = if spec.book == 3 { "trash" } else { BOOKS[spec.book].0 };
        let id = store
            .put(
                DocType::Note,
                json!({"title": "t", "body": body, "bookId": book, "tags": tags,
                       "status": spec.status.as_str(), "pinned": pinned, "sourceTemplateId": extra}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
            .id;
        let note = store.note(&id).unwrap().unwrap();
        prop_assert_eq!(&note.body, &body);
        prop_assert_eq!(note.tags, tags);
        prop_assert_eq!(note.status, spec.status);
        prop_assert_eq!(note.pinned, pinned);
        prop_assert_eq!(note.book_id, book);
        prop_assert_eq!(&note.extra["sourceTemplateId"], &json!(extra));
        let tasks = count_tasks(&body);
        prop_assert_eq!((note.num_of_tasks, note.num_of_checked_tasks), (tasks.total, tasks.checked));
    }

    /// Invariant: deleting a notebook never loses notes; every note that was in
    /// it or its sub-notebooks ends up in the trash.
    #[test]
    fn prop_deleting_a_book_conserves_notes(notes in prop::collection::vec(gen_note(), 0..12), victim in 0..3usize) {
        let (_dir, store) = temp_store();
        let ids = seed(&store, &notes);
        store.delete(BOOKS[victim].0, Origin::Api).unwrap();
        prop_assert_eq!(store.counts().unwrap().get(&DocType::Note).copied().unwrap_or(0), ids.len());
        let deleted: Vec<&str> = if victim == 0 { vec!["book:b0", "book:b2"] } else { vec![BOOKS[victim].0] };
        for (id, spec) in ids.iter().zip(&notes) {
            let note = store.note(id).unwrap().unwrap();
            let was_in_victim = spec.book < 3 && deleted.contains(&BOOKS[spec.book].0);
            prop_assert_eq!(note.is_trashed(), was_in_victim || spec.book == 3);
        }
    }
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant: recorded revisions read back newest first, and a document
    /// never keeps more than 50 of them.
    #[test]
    fn prop_recorded_revisions_are_bounded_and_newest_first(bodies in prop::collection::vec("\\PC{0,40}", 1..60)) {
        let (_dir, store) = temp_store();
        for (n, body) in bodies.iter().enumerate() {
            let fields = json!({"title": format!("v{n}"), "body": body}).as_object().unwrap().clone();
            store.record_revision("note:r1", &format!("{}-x", n + 1), &fields, Origin::Ui).unwrap();
        }
        let revisions = store.revisions("note:r1").unwrap();
        prop_assert_eq!(revisions.len(), bodies.len().min(50));
        let last = bodies.len() - 1;
        prop_assert_eq!(&revisions[0].fields["body"], &json!(bodies[last]));
        prop_assert_eq!(&revisions[0].rev, &format!("{}-x", last + 1));
    }
}
