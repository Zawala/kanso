//! Data integrity of the sync bookkeeping (docs/ACCOUNTS.md › Sync with
//! encryption › What a Mac remembers per document): `synced_rev` means a
//! version this account's server had, and the replay floor `synced_gen`
//! survives losses and re-merges. Each property is paired with the example
//! that reproduced the review's confirmed sequence (review-sync P1–P5,
//! security review M2 bypass). CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::collections::BTreeSet;

use common::*;
use kanso_core::DocType;
use kanso_store::{Origin, Store, StoreError, SyncDoc};
use kanso_sync::wire::{WireChange, WireDoc, WirePushResult};
use kanso_sync::{Choice, MemoryServer, SyncOptions, SyncReport};
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

#[allow(dead_code)]
fn generation(rev: &str) -> u64 {
    rev.parse::<kanso_core::Rev>().unwrap().generation
}

fn hash(rev: &str) -> String {
    rev.parse::<kanso_core::Rev>().unwrap().hash
}

fn floor(store: &Store, id: &str) -> u64 {
    store
        .sync_states()
        .unwrap()
        .into_iter()
        .find(|s| s.id == id)
        .map_or(0, |s| s.synced_gen)
}

/// An unbound library with the same history as `from` (as after the old
/// shared-token sync): every document and its bytes, applied as synced.
fn copy_of(from: &Store) -> (tempfile::TempDir, Store) {
    let (dir, store) = temp_store();
    for doc in from.sync_docs().unwrap() {
        let blob = from.attachment(&doc.id).unwrap().map(|(_, blob)| blob);
        store
            .apply_remote(&SyncDoc { base_rev: None, ..doc }, blob, None)
            .unwrap();
    }
    (dir, store)
}

/// Binds `store` to `username`'s account from Mac `n`, choosing Add.
fn add(server: &MemoryServer, store: &Store, dir: &tempfile::TempDir, username: &str, n: u8) -> kanso_sync::Session {
    let session = sign_in(server, username, n);
    kanso_sync::bind(store, &session, Choice::Add, &dir.path().join("backups"), 0).unwrap();
    session
}

fn sync_store(store: &Store, server: &MemoryServer, session: &kanso_sync::Session) -> SyncReport {
    kanso_sync::sync(store, server, session, &SyncOptions::default()).unwrap()
}

fn rename_file(store: &Store, id: &str, name: &str) {
    merge(store, DocType::File, json!({"_id": id, "name": name})).unwrap();
}

// ---------------------------------------------------------------------------
// a. Add never pushes a local tombstone over the account's live copy
// ---------------------------------------------------------------------------

/// Review-sync P1: Mac B deleted a note while unbound; Mac A edited it
/// (any number of times, so its generation is below, at or above B's
/// tombstone). B adds its library to the account: the edit beats the
/// deletion, nothing is stale, and B ends with the account's version.
#[test]
fn add_never_pushes_a_tombstone_over_the_accounts_live_copy() {
    for b_edits in 0..=2 {
        for a_edits in 0..=3 {
            let server = MemoryServer::new();
            let a = Mac::new(sign_up(&server, "alice", 1));
            note(&a.store, "note:x", "trash", "v1", 100);
            a.sync(&server);
            let (bdir, b) = copy_of(&a.store);
            for i in 0..b_edits {
                edit_note(&b, "note:x", &format!("B {i}"), Some(200 + i));
            }
            b.delete("note:x", Origin::Ui).unwrap();
            let tombstone = all_docs(&b)["note:x"].0.clone();
            for i in 0..a_edits {
                edit_note(&a.store, "note:x", &format!("A {i}"), Some(500 + i));
            }
            a.sync(&server);
            let a_body = body(&a.store, "note:x");

            let session = add(&server, &b, &bdir, "alice", 2);
            let report = sync_store(&b, &server, &session);
            let case = format!("B edits {b_edits} (tombstone {tombstone}), A edits {a_edits}");
            assert!(report.stale.is_empty(), "{case}: {report:?}");
            assert!(
                !server_docs(&server, &a.session)["note:x"].1,
                "{case}: B's tombstone was pushed over the account's live copy"
            );
            a.sync(&server);
            assert_eq!(body(&a.store, "note:x"), a_body, "{case}: A lost its version");
            assert_eq!(body(&b, "note:x"), a_body, "{case}: B didn't take the account's version");
            assert_eq!(pending(&b), 0, "{case}");
        }
    }
}

/// Review-sync P1b: the same with a file. Its bytes stay on the server and
/// on A, and come back to B.
#[test]
fn add_never_deletes_the_accounts_file_or_its_bytes() {
    for a_renames in 0..=3 {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        file(&a.store, "file:f1", b"\x89PNG precious bytes");
        a.sync(&server);
        let (bdir, b) = copy_of(&a.store);
        b.delete("file:f1", Origin::Ui).unwrap();
        for i in 0..a_renames {
            rename_file(&a.store, "file:f1", &format!("renamed {i}.png"));
        }
        a.sync(&server);

        let session = add(&server, &b, &bdir, "alice", 2);
        let report = sync_store(&b, &server, &session);
        assert!(report.stale.is_empty(), "{a_renames}: {report:?}");
        a.sync(&server);
        let precious = b"\x89PNG precious bytes".to_vec();
        assert_eq!(
            a.store.attachment("file:f1").unwrap().map(|(_, blob)| blob.data),
            Some(precious.clone()),
            "{a_renames}: A's bytes are gone"
        );
        assert_eq!(
            server.plain_view(&a.session).file("file:f1"),
            Some(precious.clone()),
            "{a_renames}: the server's bytes are gone"
        );
        assert_eq!(
            b.attachment("file:f1").unwrap().map(|(_, blob)| blob.data),
            Some(precious),
            "{a_renames}: B didn't get the file back"
        );
    }
}

/// A note deleted in the account but live on the Mac that adds its library
/// goes back to the account (the other half of the Add promise).
#[test]
fn add_brings_a_note_deleted_in_the_account_back() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    note(&a.store, "note:x", "trash", "keep me", 100);
    a.sync(&server);
    let (bdir, b) = copy_of(&a.store);
    a.store.delete("note:x", Origin::Ui).unwrap();
    a.sync(&server);
    let session = add(&server, &b, &bdir, "alice", 2);
    let report = sync_store(&b, &server, &session);
    assert_eq!(report.conflicted, vec!["note:x".to_owned()]);
    a.sync(&server);
    assert_eq!(body(&a.store, "note:x").as_deref(), Some("keep me"));
}

/// Tombstones Add leaves are never pushed, even by a later re-merge, and
/// `verify` neither misses them nor counts them pending.
#[test]
fn tombstones_from_before_the_binding_never_reach_the_account() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let (bdir, b) = temp_store();
    note(&b, "note:old", "trash", "deleted before signing in", 100);
    b.delete("note:old", Origin::Ui).unwrap();
    note(&b, "note:kept", "trash", "kept", 100);
    let session = add(&server, &b, &bdir, "alice", 2);
    sync_store(&b, &server, &session);
    assert!(!server_docs(&server, &a.session).contains_key("note:old"));
    server.set_epoch("alice");
    let report = sync_store(&b, &server, &session);
    assert!(report.remerged);
    assert!(!server_docs(&server, &a.session).contains_key("note:old"));
    let verified = kanso_sync::verify(&b, &server, &session, None, false).unwrap();
    assert!(verified.ok(), "{verified:?}");
}

// ---------------------------------------------------------------------------
// c. The replay floor survives a reported loss and an epoch change
// ---------------------------------------------------------------------------

/// Note `x` live at generation 1 (its sealed row captured), edited to 3 and
/// synced, then deleted on A.
fn deleted_after_edits() -> (MemoryServer, Mac, WireChange) {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    note(&a.store, "note:x", "trash", "months old", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let wire = view.wire_id("note:x");
    let old = view.wire_docs().into_iter().find(|d| d.id == wire).unwrap();
    edit_note(&a.store, "note:x", "newer", Some(200));
    a.sync(&server);
    edit_note(&a.store, "note:x", "newest", Some(300));
    a.sync(&server);
    a.store.delete("note:x", Origin::Ui).unwrap();
    (server, a, old)
}

fn conflict_with(old: &WireChange) -> WirePushResult {
    WirePushResult::Conflict {
        id: old.id.clone(),
        current: WireDoc {
            id: old.id.clone(),
            rev: old.rev.clone(),
            deleted: old.deleted,
            sealed: old.sealed.clone(),
        },
    }
}

/// Security review, M2 bypass, within one sync: the server answers the
/// tombstone's push with `Lost`, then answers the retry (no base) with a
/// genuine old version as the conflict's `current`. The note stays deleted.
#[test]
fn a_reported_loss_then_an_old_version_never_brings_a_deleted_note_back() {
    let (server, a, old) = deleted_after_edits();
    let mut pushes = 0;
    let tap = Tap::new(&server, |event| {
        if let Event::Pushed(results) = event {
            pushes += 1;
            for result in results.iter_mut().filter(|r| r.id() == old.id) {
                match pushes {
                    1 => *result = WirePushResult::Lost { id: old.id.clone() },
                    2 => *result = conflict_with(&old),
                    _ => {}
                }
            }
        }
        Ok(())
    });
    let report = a.sync(&tap);
    drop(tap);
    assert!(report.stale.contains(&"note:x".to_owned()), "{report:?}");
    assert!(a.store.get("note:x").unwrap().is_none(), "the deleted note came back");
    assert!(floor(&a.store, "note:x") >= 4);
    a.sync(&server);
    assert!(server_docs(&server, &a.session)["note:x"].1);
    assert_eq!(pending(&a.store), 0);
}

/// The same across an epoch change: the server rotates its epoch and leaves
/// the note out of the re-merge's feed, then answers the push with the old
/// version. The note stays deleted.
#[test]
fn an_epoch_change_then_an_old_version_never_brings_a_deleted_note_back() {
    let (server, a, old) = deleted_after_edits();
    a.sync(&server);
    assert!(server_docs(&server, &a.session)["note:x"].1);
    let fake_epoch = "00000000000000aa".to_owned();
    let mut since = 0;
    let tap = Tap::new(&server, |event| {
        match event {
            Event::Changes { since: asked } => since = asked,
            Event::Page(page) => {
                page.epoch = fake_epoch.clone();
                page.docs.retain(|d| d.id != old.id);
                page.last_seq = page.docs.last().map_or(since, |d| d.seq);
                page.more = page.more && !page.docs.is_empty();
            }
            Event::Pushed(results) => {
                for result in results.iter_mut().filter(|r| r.id() == old.id) {
                    *result = conflict_with(&old);
                }
            }
            _ => {}
        }
        Ok(())
    });
    let report = a.sync(&tap);
    drop(tap);
    assert!(report.remerged);
    assert!(report.stale.contains(&"note:x".to_owned()), "{report:?}");
    assert!(a.store.get("note:x").unwrap().is_none(), "the deleted note came back");
    a.sync(&server);
    assert!(server_docs(&server, &a.session)["note:x"].1);
}

/// An edit can't be rolled back that way either, even when its `updatedAt`
/// didn't move (an API merge).
#[test]
fn a_reported_loss_then_an_old_version_never_rolls_back_an_edit() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    note(&a.store, "note:x", "trash", "v1", 100);
    a.sync(&server);
    let view = server.plain_view(&a.session);
    let wire = view.wire_id("note:x");
    let old = view.wire_docs().into_iter().find(|d| d.id == wire).unwrap();
    merge(&a.store, DocType::Note, json!({"_id": "note:x", "body": "v2"})).unwrap();
    a.sync(&server);
    merge(&a.store, DocType::Note, json!({"_id": "note:x", "body": "v3"})).unwrap();
    let mut pushes = 0;
    let tap = Tap::new(&server, |event| {
        if let Event::Pushed(results) = event {
            pushes += 1;
            for result in results.iter_mut().filter(|r| r.id() == old.id) {
                *result = match pushes {
                    1 => WirePushResult::Lost { id: old.id.clone() },
                    _ => conflict_with(&old),
                };
            }
        }
        Ok(())
    });
    let _ = a.sync(&tap);
    drop(tap);
    assert_eq!(body(&a.store, "note:x").as_deref(), Some("v3"));
    a.sync(&server);
    assert_eq!(body(&a.store, "note:x").as_deref(), Some("v3"));
    assert_eq!(
        server_live(&server, &a.session)["note:x"].1["body"],
        "v3"
    );
}

// ---------------------------------------------------------------------------
// Property: Add over random diverged histories
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum AddOp {
    /// Mac `m` (0 = A, bound; 1 = B, unbound) edits note `pick`.
    Edit { m: usize, pick: usize },
    /// Mac `m` renames file `pick`.
    Rename { m: usize, pick: usize },
    /// Mac `m` deletes document `pick` (notes first, then files).
    Delete { m: usize, pick: usize },
    /// A syncs.
    SyncA,
}

fn gen_add_op() -> impl Strategy<Value = AddOp> {
    prop_oneof![
        4 => (0..2usize, 0..4usize).prop_map(|(m, pick)| AddOp::Edit { m, pick }),
        2 => (0..2usize, 0..2usize).prop_map(|(m, pick)| AddOp::Rename { m, pick }),
        3 => (0..2usize, 0..6usize).prop_map(|(m, pick)| AddOp::Delete { m, pick }),
        3 => Just(AddOp::SyncA),
    ]
}

const SHARED: [&str; 6] = ["note:s0", "note:s1", "note:s2", "note:s3", "file:f0", "file:f1"];

proptest! {
    #![proptest_config(config(48))]

    /// Invariants, for any history in which an unbound copy of the library
    /// (B) and the account's Mac (A) edit, rename and delete the same
    /// documents any number of times: when B adds its library, no document
    /// live on the account becomes a tombstone, no file loses its bytes on
    /// the server or on A, nothing is stale, B's live edits are kept
    /// (current or in history), and everyone converges.
    #[test]
    fn prop_add_never_pushes_a_tombstone_over_a_live_copy(ops in prop::collection::vec(gen_add_op(), 0..24)) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        for id in &SHARED[..4] {
            note(&a.store, id, "trash", "shared", 100);
        }
        file(&a.store, "file:f0", b"bytes zero");
        file(&a.store, "file:f1", b"bytes one");
        a.sync(&server);
        let (bdir, b) = copy_of(&a.store);
        let b_feed = b.subscribe();
        let mut clock = 1_000;
        for op in &ops {
            clock += 1;
            let store = |m: usize| if m == 0 { &a.store } else { &b };
            let result = match op {
                AddOp::Edit { m, pick } => {
                    let id = SHARED[*pick];
                    match store(*m).get_api(id, None, false) {
                        Ok(mut doc) => {
                            doc["body"] = format!("{m} at {clock}").into();
                            doc["updatedAt"] = clock.into();
                            put(store(*m), DocType::Note, doc).map(|_| ())
                        }
                        Err(err) => Err(err),
                    }
                }
                AddOp::Rename { m, pick } => merge(
                    store(*m),
                    DocType::File,
                    json!({"_id": SHARED[4 + pick], "name": format!("{m}-{clock}.png")}),
                )
                .map(|_| ()),
                AddOp::Delete { m, pick } => store(*m).delete(SHARED[*pick], Origin::Api).map(|_| ()),
                AddOp::SyncA => {
                    a.sync(&server);
                    Ok(())
                }
            };
            match result {
                Ok(()) | Err(StoreError::NotFound | StoreError::Conflict | StoreError::Invalid(_)) => {}
                Err(err) => panic!("{op:?}: {err}"),
            }
        }
        a.sync(&server);
        let live_before: BTreeSet<String> = server_live(&server, &a.session).into_keys().collect();
        let mut b_wrote: Vec<(String, String)> = Vec::new();
        while let Ok(change) = b_feed.try_recv() {
            if change.origin != Origin::Sync && !change.deleted {
                b_wrote.push((change.id, change.rev));
            }
        }

        let session = add(&server, &b, &bdir, "alice", 2);
        let report = sync_store(&b, &server, &session);
        prop_assert!(report.stale.is_empty(), "{:?}", report);
        let after = server_docs(&server, &a.session);
        for id in &live_before {
            prop_assert!(!after[id].1, "B's tombstone replaced the account's live {}", id);
        }
        a.sync(&server);
        sync_store(&b, &server, &session);
        let view = server.plain_view(&a.session);
        for id in live_before.iter().filter(|id| id.starts_with("file:")) {
            let (_, blob) = a.store.attachment(id).unwrap().expect("A kept the file");
            prop_assert_eq!(view.file(id), Some(blob.data.clone()), "the server lost {}'s bytes", id);
            prop_assert_eq!(b.attachment(id).unwrap().map(|(_, b)| b.data), Some(blob.data));
        }
        let on_server = server_docs(&server, &a.session);
        prop_assert_eq!(&all_docs(&a.store), &on_server);
        prop_assert_eq!(&all_docs(&b), &on_server);
        prop_assert_eq!(pending(&b), 0);
        for (id, rev) in &b_wrote {
            let current = b.get(id).unwrap().map(|d| hash(&d.rev));
            let kept = b.revisions(id).unwrap().iter().any(|r| hash(&r.rev) == hash(rev));
            prop_assert!(current.as_deref() == Some(hash(rev).as_str()) || kept, "B lost {} {}", id, rev);
        }
    }
}
