//! Daily notes and role notebooks over a `MemoryServer` account: two Macs
//! that make the same day's note, or the same Journal, while offline end
//! up with one note holding both their words, sealed end to end.

mod common;

use common::*;
use kanso_core::dates::Date;
use kanso_core::roles::Role;
use kanso_store::{NewNote, Origin};
use kanso_sync::MemoryServer;
use serde_json::json;

fn today() -> Date {
    Date::new(2026, 10, 20).unwrap()
}

fn template() -> NewNote {
    NewNote {
        title: String::new(),
        body: "## Plan\n\n## Log\n".into(),
        tags: Vec::new(),
    }
}

#[test]
fn the_same_days_note_made_on_two_macs_offline_keeps_both() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    // Both make today's note before either syncs.
    let id = a
        .store
        .daily_note(today(), &template(), Origin::Ui)
        .unwrap()
        .id;
    b.store
        .daily_note(today(), &template(), Origin::Ui)
        .unwrap();
    edit_note(&a.store, &id, "## Plan\n- ship the beta\n\n## Log\n", None);
    edit_note(&b.store, &id, "## Plan\n- call the bank\n\n## Log\n", None);

    a.sync(&server);
    let report = b.sync(&server);
    assert_eq!(report.conflicted, vec![id.clone()], "{report:?}");
    a.sync(&server);

    let merged = "## Plan\n- ship the beta\n- call the bank\n\n## Log\n";
    for mac in [&a, &b] {
        let note = mac.store.note(&id).unwrap().unwrap();
        // Either order of the two lines is fine; both are there once.
        assert!(note.body.contains("- ship the beta\n"), "{}", note.body);
        assert!(note.body.contains("- call the bank\n"), "{}", note.body);
        assert_eq!(note.body.len(), merged.len(), "{}", note.body);
        assert_eq!(note.book_id, "book:journal");
    }
    assert_eq!(live_docs(&a.store), live_docs(&b.store));
    assert_eq!(server_docs(&server, &a.session), all_docs(&a.store));
    assert_eq!(pending(&a.store), 0);
    assert_eq!(pending(&b.store), 0);
    // One Journal on both, still with its role.
    for mac in [&a, &b] {
        assert_eq!(
            mac.store.find_role_book(Role::Journal).unwrap().as_deref(),
            Some("book:journal")
        );
    }
}

#[test]
fn a_journal_renamed_on_one_mac_keeps_its_role_on_the_other() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    a.store
        .daily_note(today(), &template(), Origin::Ui)
        .unwrap();
    a.sync(&server);
    b.sync(&server);
    merge(
        &b.store,
        kanso_core::DocType::Book,
        json!({"_id": "book:journal", "name": "Daybook"}),
    )
    .unwrap();
    b.sync(&server);
    a.sync(&server);
    let tomorrow = a
        .store
        .daily_note(today().add_days(1), &template(), Origin::Ui)
        .unwrap();
    let note = a.store.note(&tomorrow.id).unwrap().unwrap();
    assert_eq!(note.book_id, "book:journal");
    let book = a.store.get("book:journal").unwrap().unwrap();
    assert_eq!(book.fields["name"], "Daybook");
    assert_eq!(book.fields["kansoRole"], "journal");
}
