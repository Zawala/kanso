//! The guarantee the whole facade is built to keep: while a note's lock is
//! not open, nothing the library returns holds its text.
//!
//! A library of notes, each with a marker in its text, its front matter, its
//! tasks and an inline tag, some locked. A random sequence of calls (lists, reads, search,
//! properties, query views, previews, saves, creating, trashing, locking and
//! unlocking, the auto-lock) runs against it. After every call, the marker of
//! every note that was closed when the call was made is looked for in the debug
//! text of what the call returned, and must not be found. The streams'
//! payloads, which only ever carry IDs and states, are checked at the end for
//! any marker at all. A model of which notes are closed (locked, and the
//! session not open) decides what "closed" means; it never asks the library.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::*;
use kanso_engine::lock::AUTO_LOCK;
use kanso_mobile::{
    KansoError, ListPlace, ListQuery, LockEvent, NoteChange, NoteRow, PaletteDto, Renderer,
    SearchScope, SortKey,
};
use proptest::prelude::*;

const NOTES: usize = 4;

/// The text note `i` is made of, every part carrying its marker.
fn marker(i: usize) -> String {
    format!("SECRET-MARKER-{i:04}")
}

/// The inline tag note `i` carries.
fn tag_name(i: usize) -> String {
    format!("TAGMARK{i:04}")
}

fn body_of(i: usize) -> String {
    let m = marker(i);
    format!(
        "---\nowner: {m}\nlevel: {m}\n---\nThe plan {m} is here. #{tag}\n- [ ] do {m}\n- [x] done {m}\n",
        tag = tag_name(i)
    )
}

/// Whether note `i` is locked and the session isn't open: its text must not
/// come back.
fn is_closed(locked: &[bool], open: bool, i: usize) -> bool {
    locked[i] && !open
}

/// Every row that is a locked note carries its title and its lock only.
fn check_rows(rows: &[NoteRow], ids: &[String], locked: &[bool]) -> Result<(), TestCaseError> {
    for row in rows {
        if let Some(i) = ids.iter().position(|id| *id == row.id) {
            prop_assert_eq!(row.locked, locked[i], "{:?}", row);
            if row.locked {
                prop_assert!(row.snippet.is_empty(), "{:?}", row);
                prop_assert!(row.tags.is_empty(), "{:?}", row);
                prop_assert_eq!(row.status.as_str(), "none", "{:?}", row);
                prop_assert_eq!((row.tasks_done, row.tasks_total), (0, 0), "{:?}", row);
            }
        }
    }
    Ok(())
}

fn palette() -> PaletteDto {
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{},"preview":{}}"##;
    Renderer::theme_tokens(theme.into()).unwrap().palette
}

/// One call of the sequence.
#[derive(Clone, Debug)]
enum Op {
    ListAll,
    ListTrash,
    ReadNote(usize),
    Search(usize),
    SearchWord,
    Properties(usize),
    QueryView,
    Preview(usize),
    Save(usize),
    Create,
    Trash(usize),
    LockNote(usize),
    LockSession,
    UnlockRight,
    UnlockWrong,
    IsNoteUnlocked(usize),
    Idle,
    /// A place and narrowing of the list: which place, status narrowing,
    /// pinned only, which sort.
    List(usize, bool, bool, usize),
    Change(usize, usize),
    SearchIn(usize, usize),
    CreateNotebook,
    SaveWithTag(usize),
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    let index = 0..NOTES;
    proptest::collection::vec(
        prop_oneof![
            2 => Just(Op::ListAll),
            1 => Just(Op::ListTrash),
            3 => index.clone().prop_map(Op::ReadNote),
            2 => index.clone().prop_map(Op::Search),
            1 => Just(Op::SearchWord),
            2 => index.clone().prop_map(Op::Properties),
            1 => Just(Op::QueryView),
            2 => index.clone().prop_map(Op::Preview),
            2 => index.clone().prop_map(Op::Save),
            1 => Just(Op::Create),
            1 => index.clone().prop_map(Op::Trash),
            2 => index.clone().prop_map(Op::LockNote),
            2 => Just(Op::LockSession),
            2 => Just(Op::UnlockRight),
            1 => Just(Op::UnlockWrong),
            1 => index.clone().prop_map(Op::IsNoteUnlocked),
            1 => Just(Op::Idle),
            4 => (0..6usize, any::<bool>(), any::<bool>(), 0..3usize)
                .prop_map(|(p, s, o, k)| Op::List(p, s, o, k)),
            3 => (index.clone(), 0..3usize).prop_map(|(i, k)| Op::Change(i, k)),
            3 => (0..4usize, 0..5usize).prop_map(|(s, w)| Op::SearchIn(s, w)),
            1 => Just(Op::CreateNotebook),
            1 => index.clone().prop_map(Op::SaveWithTag),
        ],
        1..24,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn prop_a_closed_note_never_comes_back(
        initially_locked in proptest::collection::vec(any::<bool>(), NOTES),
        ops in ops(),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let locks = library.locks();
        let ids: Vec<String> = (0..NOTES)
            .map(|i| note(library, &format!("Note {i}"), &body_of(i)))
            .collect();

        // The streams, collected as debug text.
        let heard = Arc::new(Mutex::new(Vec::<String>::new()));
        let log = heard.clone();
        library.changes(move |ids| { log.lock().unwrap().push(format!("{ids:?}")); true });
        let log = heard.clone();
        locks.lock_events(move |event: LockEvent| { log.lock().unwrap().push(format!("{event:?}")); true });
        let log = heard.clone();
        library.sync().status_stream(move |status| { log.lock().unwrap().push(format!("{status:?}")); true });

        // The model: which notes are locked, and whether the session is open.
        let mut locked = [false; NOTES];
        let mut open = false;
        let mut any_lock = false;
        for (i, lock) in initially_locked.iter().enumerate() {
            if *lock {
                locks.lock_note(&ids[i], LOCK_PASSWORD.into()).unwrap();
                locked[i] = true;
                any_lock = true;
                open = true;
            }
        }
        let mut trashed = [false; NOTES];

        for op in ops {
            // What is closed as this call starts, and what it returns, as text.
            let closed_now: Vec<usize> =
                (0..NOTES).filter(|i| is_closed(&locked, open, *i)).collect();
            let mut returned: Vec<String> = Vec::new();
            match &op {
                Op::ListAll | Op::ListTrash => {
                    let trash = matches!(op, Op::ListTrash);
                    let place = if trash { ListPlace::Trash } else { ListPlace::All };
                    let rows = library.list(ListQuery { place, ..ListQuery::default() }).unwrap().rows;
                    check_rows(&rows, &ids, &locked)?;
                    returned.push(format!("{rows:?}"));
                }
                Op::ReadNote(i) => {
                    let view = library.note(&ids[*i]).unwrap();
                    if is_closed(&locked, open, *i) {
                        prop_assert_eq!(&view.body, &None);
                        prop_assert!(view.lock.is_closed());
                        prop_assert!(view.tags.is_empty());
                        prop_assert_eq!(view.status.as_str(), "none");
                    } else {
                        // The other side: what may be read is read.
                        prop_assert!(view.body.as_deref().is_some_and(|b| b.contains(&marker(*i))));
                    }
                    returned.push(format!("{view:?}"));
                }
                Op::Search(i) => {
                    let hits = library.search(&marker(*i), SearchScope::All, 0).unwrap();
                    if !locked[*i] && !trashed[*i] {
                        prop_assert!(hits.iter().any(|h| h.row.id == ids[*i]), "a plain note is found");
                    }
                    returned.push(format!("{hits:?}"));
                }
                Op::SearchWord => {
                    returned.push(format!("{:?}", library.search("SECRET MARKER plan", SearchScope::All, 0).unwrap()));
                }
                Op::Properties(i) => {
                    let props = library.properties(&ids[*i]).unwrap();
                    prop_assert_eq!(props.is_empty(), is_closed(&locked, open, *i));
                    returned.push(format!("{props:?}"));
                }
                Op::QueryView => {
                    let table = library
                        .query_view("from: SECRET\ncolumns: owner, level", None)
                        .unwrap();
                    // Locked notes are never listed, open or not.
                    for row in &table.rows {
                        let at = ids.iter().position(|id| *id == row.id).unwrap();
                        prop_assert!(!locked[at]);
                    }
                    returned.push(format!("{table:?}"));
                }
                Op::Preview(i) => {
                    match library.renderer().preview_html(&ids[*i], palette()) {
                        Ok(page) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            prop_assert!(page.contains(&marker(*i)));
                            returned.push(page);
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::Save(i) => {
                    let rev = library.note(&ids[*i]).unwrap().rev;
                    let edited = format!("{}\nedited {}", body_of(*i), marker(*i));
                    match library.save(&ids[*i], &rev, edited) {
                        Ok(saved) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            if locked[*i] { open = true; }
                            returned.push(format!("{saved:?}"));
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::List(place, status, pinned_only, sort) => {
                    let notebooks = library.notebooks().unwrap();
                    let tags = library.tags().unwrap();
                    let places: Vec<ListPlace> = match place {
                        0 => vec![ListPlace::All],
                        1 => vec![ListPlace::Pinned],
                        2 => vec![ListPlace::Recent],
                        3 => notebooks.iter().map(|b| ListPlace::Notebook { id: b.id.clone() }).collect(),
                        4 => tags.iter().map(|t| ListPlace::Tag { id: t.id.clone() }).collect(),
                        _ => vec![ListPlace::Trash],
                    };
                    for place in places {
                        let by_tag = matches!(place, ListPlace::Tag { .. });
                        let list = library
                            .list(ListQuery {
                                place,
                                status_only: status.then(|| "none".to_owned()),
                                pinned_only: *pinned_only,
                                sort: [SortKey::Updated, SortKey::Created, SortKey::Title][*sort],
                                descending: true,
                            })
                            .unwrap();
                        check_rows(&list.rows, &ids, &locked)?;
                        if by_tag || *status {
                            // Made of what a note holds: a locked note is
                            // not in it.
                            for row in &list.rows {
                                if let Some(i) = ids.iter().position(|id| *id == row.id) {
                                    prop_assert!(!locked[i], "a locked note in a tag or status list: {:?}", row);
                                }
                            }
                        }
                        returned.push(format!("{list:?}"));
                    }
                    returned.push(format!("{:?}", library.overview().unwrap()));
                }
                Op::Change(i, kind) => {
                    let notebooks = library.notebooks().unwrap();
                    let change = match kind {
                        0 => NoteChange::TogglePin,
                        1 => NoteChange::Trash,
                        _ => NoteChange::Move { notebook_id: notebooks.last().unwrap().id.clone() },
                    };
                    // Pinning, trashing and moving between unlocked
                    // notebooks are allowed on a locked note, open or not.
                    let written = library.change(vec![ids[*i].clone()], change).unwrap().count;
                    if *kind == 1 {
                        trashed[*i] = true;
                    }
                    returned.push(format!("{written}"));
                }
                Op::SearchIn(scope, word) => {
                    let notebooks = library.notebooks().unwrap();
                    let scope = match scope {
                        0 => SearchScope::All,
                        1 => SearchScope::Titles,
                        2 => SearchScope::Tags,
                        _ => SearchScope::Notebook { id: notebooks.first().unwrap().id.clone() },
                    };
                    let word = ["SECRET", "TAGMARK0000", "Note", "status:none", "tag:TAGMARK0001"][*word];
                    let by_content = matches!(scope, SearchScope::Tags)
                        || word.starts_with("status:")
                        || word.starts_with("tag:");
                    let hits = library.search(word, scope.clone(), 0).unwrap();
                    let rows: Vec<NoteRow> = hits.iter().map(|h| h.row.clone()).collect();
                    check_rows(&rows, &ids, &locked)?;
                    if by_content {
                        for row in &rows {
                            if let Some(i) = ids.iter().position(|id| *id == row.id) {
                                prop_assert!(!locked[i], "a locked note found by its tags or status: {:?}", row);
                            }
                        }
                    }
                    if scope == SearchScope::Tags && word == "TAGMARK0000" {
                        // The other side: a plain note is found by its tag.
                        if !locked[0] && !trashed[0] {
                            prop_assert!(rows.iter().any(|r| r.id == ids[0]), "not found by its tag");
                        }
                    }
                    returned.push(format!("{hits:?}"));
                }
                Op::CreateNotebook => {
                    let made = library.create_notebook("Garden".into(), None).unwrap();
                    returned.push(format!("{made:?}"));
                }
                Op::SaveWithTag(i) => {
                    let rev = library.note(&ids[*i]).unwrap().rev;
                    let tag = format!("extra{i}");
                    let edited = format!("{} #{tag}", body_of(*i));
                    match library.save(&ids[*i], &rev, edited) {
                        Ok(saved) => {
                            prop_assert!(!is_closed(&locked, open, *i));
                            if locked[*i] {
                                open = true;
                            } else {
                                // A plain note takes the tag its text names.
                                let names: Vec<String> =
                                    library.tags().unwrap().into_iter().map(|t| t.name).collect();
                                prop_assert!(names.iter().any(|n| n.eq_ignore_ascii_case(&tag)), "{names:?}");
                                let view = library.note(&ids[*i]).unwrap();
                                prop_assert!(!view.tags.is_empty());
                            }
                            returned.push(format!("{saved:?}"));
                        }
                        Err(err) => {
                            prop_assert_eq!(err, KansoError::Locked);
                            prop_assert!(is_closed(&locked, open, *i));
                        }
                    }
                }
                Op::Create => {
                    let saved = library
                        .create(kanso_mobile::NewNote {
                            notebook_id: None,
                            title: "Fresh".into(),
                            body: "fresh note".into(),
                        })
                        .unwrap();
                    returned.push(format!("{saved:?}"));
                }
                Op::Trash(i) => {
                    library.trash(&ids[*i]).unwrap();
                    trashed[*i] = true;
                }
                Op::LockNote(i) => {
                    match locks.lock_note(&ids[*i], LOCK_PASSWORD.into()) {
                        Ok(saved) => {
                            prop_assert!(!locked[*i]);
                            locked[*i] = true;
                            any_lock = true;
                            open = true;
                            returned.push(format!("{saved:?}"));
                        }
                        // Already locked.
                        Err(KansoError::Invalid { .. }) => prop_assert!(locked[*i]),
                        Err(other) => prop_assert!(false, "unexpected {other:?}"),
                    }
                }
                Op::LockSession => {
                    locks.lock().unwrap();
                    open = false;
                }
                Op::UnlockRight => match locks.unlock(LOCK_PASSWORD.into()) {
                    Ok(true) => { prop_assert!(any_lock); open = true; }
                    Ok(false) => prop_assert!(false, "the right password was refused"),
                    Err(_) => prop_assert!(!any_lock),
                },
                Op::UnlockWrong => {
                    let answer = locks.unlock("not the right one".into());
                    prop_assert!(matches!(answer, Ok(false) | Err(_)), "{answer:?}");
                }
                Op::IsNoteUnlocked(i) => {
                    prop_assert_eq!(locks.is_note_unlocked(&ids[*i]).unwrap(), !is_closed(&locked, open, *i));
                }
                Op::Idle => {
                    let later = Instant::now() + AUTO_LOCK + Duration::from_secs(1);
                    prop_assert_eq!(locks.lock_if_idle(later), open);
                    open = false;
                }
            }
            prop_assert_eq!(locks.is_unlocked().unwrap(), open, "the session after {:?}", op);

            // The guarantee: nothing this call returned holds the marker of a
            // note that was closed when it was asked.
            for i in &closed_now {
                let m = marker(*i);
                for text in &returned {
                    prop_assert!(!text.contains(&m), "{m} leaked from {op:?}: {text}");
                }
            }
        }

        // Streams only ever carry IDs and states: no marker at all.
        // (Give the change thread a moment to deliver what it was sent.)
        std::thread::sleep(Duration::from_millis(30));
        for text in heard.lock().unwrap().iter() {
            prop_assert!(!text.contains("SECRET-MARKER"), "a stream carried text: {text}");
        }
    }
}
