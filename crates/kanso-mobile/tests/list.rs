//! The lists of the mobile app: places, counts, changes to several notes,
//! scoped search, notebooks in tree order, and the tags a note's text gives
//! it.

mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use common::*;
use kanso_core::now_ms;
use kanso_mobile::{DateField, ListPlace, ListQuery, NotebookRole, SortKey};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

const DAY: i64 = 86_400_000;

fn server() -> Arc<MemoryServer> {
    Arc::new(MemoryServer::new())
}

/// A note the properties seed.
#[derive(Clone, Debug)]
struct Seed {
    book: usize,
    pinned: bool,
    trashed: bool,
    /// Days since it was changed.
    age: i64,
}

fn seeds() -> impl Strategy<Value = Vec<Seed>> {
    proptest::collection::vec(
        (0usize..2, any::<bool>(), any::<bool>(), 0i64..12).prop_map(
            |(book, pinned, trashed, age)| Seed {
                book,
                pinned,
                trashed,
                age,
            },
        ),
        0..10,
    )
}

fn seeded(seeds: &[Seed]) -> (Phone, Vec<String>, Vec<String>) {
    let mut ids = Vec::new();
    let mut books = Vec::new();
    let now = now_ms();
    let phone = phone_with(&server(), |store| {
        books = vec![book(store, "One"), book(store, "Two")];
        for (i, seed) in seeds.iter().enumerate() {
            let at = now - seed.age * DAY - 60_000;
            ids.push(raw_note(
                store,
                json!({
                    "title": format!("Note {i:02}"),
                    "body": format!("body {i}"),
                    "bookId": if seed.trashed { "trash".to_owned() } else { books[seed.book].clone() },
                    "pinned": seed.pinned,
                    "createdAt": at,
                    "updatedAt": at,
                }),
            ));
        }
    });
    (phone, ids, books)
}

fn ids_of(rows: &[kanso_mobile::NoteRow]) -> Vec<String> {
    rows.iter().map(|row| row.id.clone()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The library's counts are the lengths of its places' lists, and each
    /// place holds what it names: Pinned the pinned notes outside the trash,
    /// Recent those changed in the last seven days, newest first.
    #[test]
    fn prop_counts_are_the_lengths_of_the_places(seeds in seeds()) {
        let (phone, ids, _) = seeded(&seeds);
        let library = &phone.library;
        let list = |place: ListPlace| library.list(ListQuery { place, ..ListQuery::default() }).unwrap();
        let overview = library.overview().unwrap();
        let live = |keep: &dyn Fn(&Seed) -> bool| -> BTreeSet<String> {
            seeds.iter().zip(&ids).filter(|(s, _)| !s.trashed && keep(s)).map(|(_, id)| id.clone()).collect()
        };
        let set = |rows: &[kanso_mobile::NoteRow]| ids_of(rows).into_iter().collect::<BTreeSet<_>>();

        let all = list(ListPlace::All);
        prop_assert_eq!(overview.all as usize, all.rows.len());
        prop_assert_eq!(set(&all.rows), live(&|_| true));

        let pinned = list(ListPlace::Pinned);
        prop_assert_eq!(overview.pinned as usize, pinned.rows.len());
        prop_assert_eq!(set(&pinned.rows), live(&|s| s.pinned));

        let recent = list(ListPlace::Recent);
        prop_assert_eq!(overview.recent as usize, recent.rows.len());
        prop_assert_eq!(set(&recent.rows), live(&|s| s.age < 7));
        prop_assert!(recent.rows.windows(2).all(|w| w[0].updated_ms >= w[1].updated_ms));

        let trash = list(ListPlace::Trash);
        prop_assert_eq!(overview.trash as usize, trash.rows.len());
        prop_assert_eq!(trash.rows.len(), seeds.iter().filter(|s| s.trashed).count());
    }

    /// Pinned notes come first and keep the sort within each group; narrowed
    /// to pinned notes, a list holds the pinned ones in the same order; and
    /// Recent is by edit time whatever the sort asked for.
    #[test]
    fn prop_pinned_first_and_recent_ignores_the_sort(seeds in seeds(), descending: bool, sort in 0usize..3) {
        let (phone, _, _) = seeded(&seeds);
        let library = &phone.library;
        let sort = [SortKey::Updated, SortKey::Created, SortKey::Title][sort];
        let query = ListQuery { sort, descending, ..ListQuery::default() };
        let all = library.list(query.clone()).unwrap();
        let pins: Vec<bool> = all.rows.iter().map(|r| r.pinned).collect();
        let first_unpinned = pins.iter().position(|p| !p).unwrap_or(pins.len());
        prop_assert!(pins[first_unpinned..].iter().all(|p| !p), "{:?}", pins);

        let only = library.list(ListQuery { pinned_only: true, ..query.clone() }).unwrap();
        let expected: Vec<String> = all.rows.iter().filter(|r| r.pinned).map(|r| r.id.clone()).collect();
        prop_assert_eq!(ids_of(&only.rows), expected);
        prop_assert!(!only.shape.group_pinned);
        prop_assert!(all.shape.group_pinned);

        let recent = library.list(ListQuery { place: ListPlace::Recent, ..query }).unwrap();
        let plain = library.list(ListQuery { place: ListPlace::Recent, ..ListQuery::default() }).unwrap();
        prop_assert_eq!(ids_of(&recent.rows), ids_of(&plain.rows));
        prop_assert_eq!(recent.shape.sort, SortKey::Updated);
        prop_assert_eq!(recent.shape.date, DateField::Updated);
        prop_assert!(recent.shape.by_date && !recent.shape.group_pinned);
    }
}

// no-prop-test: the order of a small fixed tree and its roles.
#[test]
fn notebooks_come_in_tree_order_with_their_roles_and_depth() {
    let mut templates = String::new();
    let phone = phone_with(&server(), |store| {
        let put = |name: &str, parent: Option<&String>| {
            store
                .put(
                    kanso_core::DocType::Book,
                    json!({"name": name, "parentBookId": parent}),
                    kanso_store::Put::Upsert,
                    kanso_store::Origin::Ui,
                )
                .unwrap()
                .id
        };
        let work = put("Work", None);
        let acme = put("Acme", Some(&work));
        put("Plans", Some(&acme));
        put("Garden", None);
        templates = store
            .role_book(kanso_core::roles::Role::Templates, kanso_store::Origin::Ui)
            .unwrap();
    });
    let rows = phone.library.notebooks().unwrap();
    let order: Vec<_> = rows.iter().map(|r| (r.name.as_str(), r.depth)).collect();
    assert_eq!(
        order,
        [
            ("Garden", 0),
            ("Templates", 0),
            ("Work", 0),
            ("Acme", 1),
            ("Plans", 2)
        ]
    );
    // A notebook with a job, however it is named.
    let row = rows.iter().find(|r| r.id == templates).unwrap();
    assert_eq!(row.role, Some(NotebookRole::Templates));
    assert!(
        rows.iter()
            .filter(|r| r.id != templates)
            .all(|r| r.role.is_none())
    );
}
