//! Rollback (docs/ACCOUNTS.md › Rollback, fix 1 / H5): a library synced
//! under v1 rules, then v2 with edits, creations and deletions, then
//! `unbind`, then v1 rules again. Every v2-window edit and creation reaches
//! the v1 server; v2-window deletions come back, as the rollback steps say.

mod common;

use std::collections::BTreeMap;

use common::legacy::{LegacyServer, legacy_sync};
use common::*;
use kanso_store::{Origin, Store};
use kanso_sync::{
    Choice, Flow, LocalLibrary, MemorySecrets, MemoryServer, Plan, local_library, meta_keys, plan,
};

fn v1_bodies(server: &LegacyServer) -> BTreeMap<String, Option<String>> {
    server
        .docs()
        .into_iter()
        .filter(|d| d.id.starts_with("note:"))
        .map(|d| {
            let body = (!d.deleted).then(|| d.fields["body"].as_str().unwrap().to_owned());
            (d.id, body)
        })
        .collect()
}

fn bodies(store: &Store) -> BTreeMap<String, Option<String>> {
    all_docs(store)
        .into_keys()
        .filter(|id| id.starts_with("note:"))
        .map(|id| {
            let body = body(store, &id);
            (id, body)
        })
        .collect()
}

#[test]
fn v2_then_unbind_then_v1_loses_no_edit() {
    // The v1 era: two Macs synced through the plaintext server.
    let v1 = LegacyServer::new();
    let (dir, store) = temp_store();
    let (_other_dir, other) = temp_store();
    book(&store, "book:b1", "Inbox");
    for id in ["note:keep", "note:edit", "note:del", "note:untouched"] {
        note(&store, id, "book:b1", &format!("{id} in v1"), 100);
    }
    legacy_sync(&store, &v1);
    legacy_sync(&other, &v1);
    assert_eq!(bodies(&other), bodies(&store));

    // The v2 window: sign up, bind, and change things on this Mac and on a
    // second v2 Mac.
    let server = MemoryServer::new();
    let session = sign_up(&server, "alice", 1);
    let library = local_library(&store, &session.origin, &session.account_id).unwrap();
    assert!(matches!(library, LocalLibrary::Unbound { .. }));
    assert_eq!(plan(Flow::SignUp, &library), Plan::Fresh);
    kanso_sync::bind(
        &store,
        &session,
        Choice::Add,
        &dir.path().join("backups"),
        0,
    )
    .unwrap();
    let mac = Mac {
        store,
        session,
        dir,
    };
    mac.sync(&server);
    let b = Mac::new(sign_in(&server, "alice", 2));
    b.sync(&server);
    edit_note(&mac.store, "note:edit", "edited in v2", Some(500));
    note(&mac.store, "note:new", "book:b1", "created in v2", 500);
    mac.store.delete("note:del", Origin::Api).unwrap();
    mac.sync(&server);
    b.sync(&server);
    edit_note(
        &b.store,
        "note:keep",
        "edited on the other v2 Mac",
        Some(600),
    );
    b.sync(&server);
    mac.sync(&server);
    assert_eq!(pending(&mac.store), 0);
    assert_eq!(server_docs(&server, &mac.session), all_docs(&mac.store));

    // Roll back: unbind, then the old app's rules against the v1 server.
    let secrets = MemorySecrets::new();
    kanso_sync::unbind(&mac.store, &secrets).unwrap();
    assert_eq!(
        mac.store
            .meta(meta_keys::LEGACY_LAST_SEQ)
            .unwrap()
            .as_deref(),
        Some("0")
    );
    for key in [
        meta_keys::ORIGIN,
        meta_keys::ACCOUNT_ID,
        meta_keys::LAST_SEQ,
        meta_keys::EPOCH,
    ] {
        assert_eq!(mac.store.meta(key).unwrap(), None, "{key}");
    }
    legacy_sync(&mac.store, &v1);
    legacy_sync(&mac.store, &v1);
    assert_eq!(pending(&mac.store), 0);

    let on_v1 = v1_bodies(&v1);
    // Every v2-window edit and creation reached the v1 server.
    assert_eq!(on_v1["note:edit"].as_deref(), Some("edited in v2"));
    assert_eq!(on_v1["note:new"].as_deref(), Some("created in v2"));
    assert_eq!(
        on_v1["note:keep"].as_deref(),
        Some("edited on the other v2 Mac")
    );
    assert_eq!(
        on_v1["note:untouched"].as_deref(),
        Some("note:untouched in v1")
    );
    // A v2-window deletion comes back: the v1 server never saw it.
    assert_eq!(on_v1["note:del"].as_deref(), Some("note:del in v1"));
    assert_eq!(bodies(&mac.store), on_v1);

    // The other v1 Mac gets all of it.
    legacy_sync(&other, &v1);
    assert_eq!(bodies(&other), on_v1);
    assert_eq!(all_docs(&other), all_docs(&mac.store));
}
