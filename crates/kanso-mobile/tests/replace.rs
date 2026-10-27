//! Replace in all notes through the facade: the preview, applying it, undoing
//! it, and what it never does to a locked note.

mod common;

use common::*;
use kanso_mobile::{KansoError, Library, PlannedNote, ReplaceOptions};
use kanso_store::{Origin, Put, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{LockKey, LockPassword};
use proptest::prelude::*;
use serde_json::{Map, Value, json};

/// What a locked note says, which no result may carry.
const SECRET: &str = "SEALED-MARKER-7731";

fn options(find: &str) -> ReplaceOptions {
    ReplaceOptions {
        find: find.into(),
        ..ReplaceOptions::default()
    }
}

/// A note's stored fields without its edit time, which a replace and its
/// undo move.
fn stored(store: &Store, id: &str) -> Map<String, Value> {
    let mut fields = store.get(id).unwrap().unwrap().fields;
    fields.remove("updatedAt");
    fields
}

fn everything(preview: &impl std::fmt::Debug) -> String {
    format!("{preview:?}")
}

fn planned_from(library: &Library, ids: &[&str]) -> Vec<PlannedNote> {
    ids.iter()
        .map(|id| PlannedNote {
            id: (*id).into(),
            rev: library.note(id).unwrap().rev,
        })
        .collect()
}

/// Text with multibyte characters, line breaks of both kinds and the
/// letters the finds are made of.
fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            4 => "[abAB]{1,3}",
            2 => Just(" ".to_owned()),
            1 => Just("\n".to_owned()),
            1 => Just("\r\n".to_owned()),
            1 => Just("é".to_owned()),
            1 => Just("😀".to_owned()),
            1 => Just("日本".to_owned()),
            1 => Just("```\n".to_owned()),
            1 => "\\PC{0,4}",
        ],
        0..14,
    )
    .prop_map(|parts| parts.concat())
}

fn find() -> impl Strategy<Value = String> {
    prop_oneof![
        "[ab]{1,2}",
        Just("é".to_owned()),
        Just("😀".to_owned()),
        Just("\n".to_owned())
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Replacing in every note the preview found, then undoing the batch,
    /// gives each note back byte for byte (title, text and everything else
    /// stored but its edit time); a note with no match is never written.
    #[test]
    fn prop_replace_then_undo_restores_every_note(
        notes in prop::collection::vec((text(), any::<bool>()), 1..6),
        find in find(),
        with in "[xy$é\n]{0,4}",
        (match_case, whole_word, regex, skip_code, include_trash) in
            (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let ids: Vec<String> = notes
            .iter()
            .enumerate()
            .map(|(i, (body, trashed))| {
                let id = note(library, &format!("Note {i} ☃"), body);
                if *trashed {
                    library.trash(&id).unwrap();
                }
                id
            })
            .collect();
        let store = Store::open(phone.path()).unwrap();
        let before: Vec<_> = ids.iter().map(|id| stored(&store, id)).collect();
        let revs: Vec<_> = ids.iter().map(|id| store.get(id).unwrap().unwrap().rev).collect();

        let options = ReplaceOptions { find, match_case, whole_word, regex, skip_code, include_trash };
        let preview = library.replace_preview(options.clone()).unwrap();
        prop_assert_eq!(preview.error, None);
        let planned: Vec<PlannedNote> = preview
            .notes
            .iter()
            .map(|n| PlannedNote { id: n.id.clone(), rev: n.rev.clone() })
            .collect();
        let outcome = library.replace_apply(options.clone(), with, planned).unwrap();
        prop_assert_eq!(outcome.skipped, 0);
        prop_assert_eq!(outcome.notes as usize, preview.notes.len());
        prop_assert_eq!(outcome.replaced, preview.total_matches);

        for (i, id) in ids.iter().enumerate() {
            if !preview.notes.iter().any(|n| &n.id == id) {
                prop_assert_eq!(store.get(id).unwrap().unwrap().rev, revs[i].clone(), "an unmatched note was written");
            }
        }

        library.replace_undo(&outcome.batch).unwrap();
        for (i, id) in ids.iter().enumerate() {
            prop_assert_eq!(&stored(&store, id), &before[i]);
        }
        // Undone once: the second undo puts nothing back.
        prop_assert_eq!(library.replace_undo(&outcome.batch).unwrap(), "Put back 0 notes.");
        for (i, id) in ids.iter().enumerate() {
            prop_assert_eq!(&stored(&store, id), &before[i]);
        }
    }

    /// The preview's lines and ranges are in UTF-16 units: the range cuts
    /// out of the line exactly what the find matched.
    #[test]
    fn prop_preview_ranges_are_utf16_units_of_their_line(body in text(), find in "[ab]{1,2}") {
        let phone = lone_phone();
        note(&phone.library, "T", &body);
        let preview = phone.library.replace_preview(ReplaceOptions {
            find: find.clone(),
            match_case: true,
            ..ReplaceOptions::default()
        }).unwrap();
        for found in &preview.notes {
            prop_assert_eq!(found.count as usize, body.matches(&find).count());
            for m in &found.matches {
                let units: Vec<u16> = m.line.encode_utf16().collect();
                let cut = String::from_utf16(&units[m.start_utf16 as usize..m.end_utf16 as usize]).unwrap();
                prop_assert_eq!(&cut, &find);
                prop_assert_eq!(
                    body.split('\n').nth(m.line_number as usize - 1).map(|l| l.strip_suffix('\r').unwrap_or(l)),
                    Some(m.line.as_str()).map(|l| l.strip_suffix('\r').unwrap_or(l))
                );
            }
        }
    }
}

// no-prop-test: bad and empty finds, one of each.
#[test]
fn a_bad_pattern_says_why_and_an_empty_find_previews_nothing() {
    let phone = lone_phone();
    note(&phone.library, "T", "(x");
    let bad = phone
        .library
        .replace_preview(ReplaceOptions {
            find: "(".into(),
            regex: true,
            ..ReplaceOptions::default()
        })
        .unwrap();
    assert!(bad.error.is_some() && bad.notes.is_empty() && bad.total_matches == 0);
    let empty = phone.library.replace_preview(options("")).unwrap();
    assert_eq!((empty.error, empty.notes.len()), (None, 0));
    assert!(matches!(
        phone.library.replace_apply(
            ReplaceOptions {
                find: "(".into(),
                regex: true,
                ..ReplaceOptions::default()
            },
            "x".into(),
            vec![]
        ),
        Err(KansoError::Invalid { .. })
    ));
}

// no-prop-test: the words of the button.
#[test]
fn the_button_and_the_outcome_say_what_happens() {
    assert_eq!(kanso_mobile::replace_apply_label(0, 0), None);
    assert_eq!(
        kanso_mobile::replace_apply_label(7, 3).as_deref(),
        Some("Replace 7 in 3 notes")
    );
    let phone = lone_phone();
    let id = note(&phone.library, "Pets", "cat and cat");
    let preview = phone.library.replace_preview(options("cat")).unwrap();
    let outcome = phone
        .library
        .replace_apply(
            options("cat"),
            "dog".into(),
            planned_from(&phone.library, &[&id]),
        )
        .unwrap();
    assert_eq!(preview.total_matches, 2);
    assert_eq!(outcome.message, "Replaced 2 matches in 1 note.");
    assert_eq!(
        phone.library.note(&id).unwrap().body.as_deref(),
        Some("dog and dog")
    );
    // A plan at an old revision is left alone and said so.
    let stale = vec![PlannedNote {
        id: id.clone(),
        rev: preview.notes[0].rev.clone(),
    }];
    let again = phone
        .library
        .replace_apply(options("dog"), "x".into(), stale)
        .unwrap();
    assert_eq!((again.notes, again.skipped), (0, 1));
    assert!(again.message.contains("changed elsewhere"));
    assert_eq!(
        phone.library.replace_undo(&outcome.batch).unwrap(),
        "Put back 1 note."
    );
    assert_eq!(
        phone.library.note(&id).unwrap().body.as_deref(),
        Some("cat and cat")
    );
}

/// A library with a plain note, a sealed note and a locked notebook with two
/// notes in it, each locked one holding [`SECRET`] and "cat".
struct Vaults {
    phone: Phone,
    plain: String,
    sealed: String,
    vaulted: [String; 2],
    vault: String,
}

fn vaults() -> Vaults {
    let mut vaulted = [String::new(), String::new()];
    let mut vault = String::new();
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            vault = book(store, "Vault");
            for (i, id) in vaulted.iter_mut().enumerate() {
                *id = raw_note(
                    store,
                    json!({"title": format!("Vaulted {i}"), "body": format!("cat {SECRET} {i}"), "bookId": vault}),
                );
            }
            let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST)
                .unwrap();
            assert_eq!(
                kanso_engine::lock::seal_notebook(store, &vault, &key),
                Ok(2)
            );
        },
    );
    let plain = note(&phone.library, "Plain", "a cat sat");
    let sealed = note(&phone.library, "Sealed", &format!("a cat with {SECRET}"));
    phone
        .library
        .locks()
        .lock_note(&sealed, LOCK_PASSWORD.into())
        .unwrap();
    Vaults {
        phone,
        plain,
        sealed,
        vaulted,
        vault,
    }
}

fn locked_rows(v: &Vaults, store: &Store) -> Vec<(Map<String, Value>, String)> {
    [&v.sealed, &v.vaulted[0], &v.vaulted[1]]
        .iter()
        .map(|id| {
            let doc = store.get(id).unwrap().unwrap();
            (doc.fields, doc.rev)
        })
        .collect()
}

// no-prop-test: the fixed cases of the locked notes, closed and open.
#[test]
fn locked_notes_are_never_previewed_or_changed_closed_or_open() {
    let v = vaults();
    let library = &v.phone.library;
    let locks = library.locks();
    let store = Store::open(v.phone.path()).unwrap();
    for open in [false, true] {
        if open {
            assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
        } else {
            locks.lock().unwrap();
        }
        let before = locked_rows(&v, &store);
        let preview = library.replace_preview(options("cat")).unwrap();
        let ids: Vec<&str> = preview.notes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, [v.plain.as_str()], "open: {open}");
        assert!(!everything(&preview).contains(SECRET));

        // A forged plan names every locked note at its current revision.
        let rev = |id: &str| store.get(id).unwrap().unwrap().rev;
        let forged: Vec<PlannedNote> = [&v.sealed, &v.vaulted[0], &v.vaulted[1]]
            .iter()
            .map(|id| PlannedNote {
                id: (*id).clone(),
                rev: rev(id),
            })
            .collect();
        let outcome = library
            .replace_apply(options("cat"), "LEAK".into(), forged)
            .unwrap();
        assert_eq!(
            (outcome.notes, outcome.replaced, outcome.skipped),
            (0, 0, 3),
            "open: {open}"
        );
        assert!(!everything(&outcome).contains(SECRET));
        assert_eq!(locked_rows(&v, &store), before, "open: {open}");
        let message = library.replace_undo(&outcome.batch).unwrap();
        assert!(!message.contains(SECRET));
        assert_eq!(locked_rows(&v, &store), before);
    }
    // The text, once opened, is still the text.
    assert!(
        library
            .note(&v.sealed)
            .unwrap()
            .body
            .unwrap()
            .contains(SECRET)
    );
    assert!(
        library
            .note(&v.vaulted[0])
            .unwrap()
            .body
            .unwrap()
            .contains("cat")
    );
    let _ = &v.vault;
}

// no-prop-test: a note locked after it was replaced is left as it is by the undo.
#[test]
fn undo_leaves_a_note_that_was_locked_since() {
    let phone = lone_phone();
    let library = &phone.library;
    let a = note(library, "A", "cat one");
    let b = note(library, "B", "cat two");
    let store = Store::open(phone.path()).unwrap();
    let preview = library.replace_preview(options("cat")).unwrap();
    let planned = preview
        .notes
        .iter()
        .map(|n| PlannedNote {
            id: n.id.clone(),
            rev: n.rev.clone(),
        })
        .collect();
    let outcome = library
        .replace_apply(options("cat"), "dog".into(), planned)
        .unwrap();
    assert_eq!(outcome.notes, 2);

    // A is sealed on its own; B's whole notebook is locked from elsewhere.
    library.locks().lock_note(&a, LOCK_PASSWORD.into()).unwrap();
    let book_id = library.note(&b).unwrap().notebook_id;
    let mut notebook = store.get_api(&book_id, None, false).unwrap();
    notebook.as_object_mut().unwrap().remove("count");
    notebook["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store
        .put(
            kanso_core::DocType::Book,
            notebook,
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    library.locks().lock().unwrap();
    let held = [stored(&store, &a), stored(&store, &b)];

    let message = library.replace_undo(&outcome.batch).unwrap();
    assert!(message.starts_with("Put back 0 notes"), "{message}");
    assert_eq!([stored(&store, &a), stored(&store, &b)], held);
    assert!(!format!("{held:?}").contains("cat one"));
}
