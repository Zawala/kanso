//! Data integrity of the sync bookkeeping (docs/ACCOUNTS.md › Sync with
//! encryption › What a Mac remembers per document): `synced_rev` means a
//! version this account's server had, and the replay floor `synced_gen`
//! survives losses and re-merges. Each property is paired with the example
//! that reproduced the review's confirmed sequence (review-sync P1–P5,
//! security review M2 bypass). CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::*;
use kanso_core::DocType;
use kanso_store::{Origin, Store, StoreError, SyncDoc};
use kanso_sync::wire::{WireChange, WireDoc, WirePushResult};
use kanso_sync::{Choice, MemoryServer, QuotaOverride, Route, SyncOptions, SyncReport};
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
            .apply_remote(
                &SyncDoc {
                    base_rev: None,
                    ..doc
                },
                blob,
                None,
            )
            .unwrap();
    }
    (dir, store)
}

/// Binds `store` to `username`'s account from Mac `n`, choosing Add.
fn add(
    server: &MemoryServer,
    store: &Store,
    dir: &tempfile::TempDir,
    username: &str,
    n: u8,
) -> kanso_sync::Session {
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
            assert_eq!(
                body(&a.store, "note:x"),
                a_body,
                "{case}: A lost its version"
            );
            assert_eq!(
                body(&b, "note:x"),
                a_body,
                "{case}: B didn't take the account's version"
            );
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
            a.store
                .attachment("file:f1")
                .unwrap()
                .map(|(_, blob)| blob.data),
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
    assert!(
        a.store.get("note:x").unwrap().is_none(),
        "the deleted note came back"
    );
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
    assert!(
        a.store.get("note:x").unwrap().is_none(),
        "the deleted note came back"
    );
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
    merge(
        &a.store,
        DocType::Note,
        json!({"_id": "note:x", "body": "v2"}),
    )
    .unwrap();
    a.sync(&server);
    merge(
        &a.store,
        DocType::Note,
        json!({"_id": "note:x", "body": "v3"}),
    )
    .unwrap();
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
    assert_eq!(server_live(&server, &a.session)["note:x"].1["body"], "v3");
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

const SHARED: [&str; 6] = [
    "note:s0", "note:s1", "note:s2", "note:s3", "file:f0", "file:f1",
];

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

// ---------------------------------------------------------------------------
// b. No live edit is ever silently lost (restores and rewinds)
// ---------------------------------------------------------------------------

/// Whether `id`'s content `rev` (by content hash) is current or in the
/// history of `store`.
fn holds(store: &Store, id: &str, rev: &str) -> bool {
    store
        .get(id)
        .unwrap()
        .is_some_and(|d| hash(&d.rev) == hash(rev))
        || store
            .revisions(id)
            .unwrap()
            .iter()
            .any(|r| hash(&r.rev) == hash(rev))
}

/// Review-sync P2: after a restore, Mac A's offline edit lands below Mac
/// B's floor (B edited twice), or above it (A edited three times). Either
/// way both versions are kept and the document is flagged; nothing is
/// silently dropped.
#[test]
fn a_restore_never_silently_drops_an_offline_edit() {
    for a_offline_edits in [1, 3] {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        let b = Mac::new(sign_in(&server, "alice", 2));
        note(&a.store, "note:x", "trash", "v1", 100);
        a.sync(&server);
        b.sync(&server);
        let snapshot = server.snapshot("alice");
        edit_note(&b.store, "note:x", "B one", Some(200));
        b.sync(&server);
        let b_two = edit_note(&b.store, "note:x", "B two", Some(300));
        b.sync(&server);
        let mut a_last = None;
        for i in 0..a_offline_edits {
            a_last = Some(edit_note(
                &a.store,
                "note:x",
                &format!("A offline {i}"),
                Some(900 + i),
            ));
        }
        let a_last = a_last.unwrap();
        server.restore("alice", &snapshot);

        let ra = a.sync(&server);
        let rb = b.sync(&server);
        let ra2 = a.sync(&server);
        let case = format!("{a_offline_edits} offline edits on A");
        assert!(ra.remerged && rb.remerged, "{case}");
        let flagged = [&ra, &rb, &ra2]
            .iter()
            .any(|r| r.conflicted.contains(&"note:x".to_owned()));
        assert!(flagged, "{case}: nothing flagged ({ra:?} {rb:?} {ra2:?})");
        assert_eq!(
            body(&a.store, "note:x"),
            body(&b.store, "note:x"),
            "{case}: the Macs differ"
        );
        // Both versions survive somewhere, whichever won.
        for rev in [&a_last.rev, &b_two.rev] {
            assert!(
                holds(&a.store, "note:x", rev) || holds(&b.store, "note:x", rev),
                "{case}: {rev}"
            );
        }
        assert_eq!((pending(&a.store), pending(&b.store)), (0, 0), "{case}");
    }
}

/// Review-verify 2: after a restore, Mac A's deletion and Mac B's offline edit
/// land at the same generation. Whichever Mac syncs first, the edit beats
/// the deletion: the note (or file, with B's bytes) is live on every Mac
/// and on the account, and a Mac that flags it can open it.
#[test]
fn after_a_restore_an_edit_this_mac_never_held_beats_its_deletion() {
    for as_file in [false, true] {
        for b_first in [true, false] {
            let case = format!("file {as_file}, B first {b_first}");
            let server = MemoryServer::new();
            let a = Mac::new(sign_up(&server, "alice", 1));
            let b = Mac::new(sign_in(&server, "alice", 2));
            let id = if as_file { "file:x" } else { "note:x" };
            let write = |store: &Store, text: &str, at: i64| {
                if as_file {
                    if all_docs(store).contains_key(id) {
                        write_file(store, id, text.as_bytes());
                    } else {
                        file(store, id, text.as_bytes());
                    }
                } else if all_docs(store).contains_key(id) {
                    edit_note(store, id, text, Some(at));
                } else {
                    note(store, id, "trash", text, at);
                }
            };
            write(&a.store, "v1", 100);
            a.sync(&server);
            write(&a.store, "v2", 200);
            a.sync(&server);
            b.sync(&server);
            let snapshot = server.snapshot("alice");
            a.store.delete(id, Origin::Ui).unwrap();
            a.sync(&server);
            server.restore("alice", &snapshot);
            write(&b.store, "B's edit", 300);
            let b_edit = all_docs(&b.store)[id].0.clone();
            assert_eq!(
                generation(&b_edit),
                generation(&all_docs(&a.store)[id].0),
                "{case}: the edit and the deletion share a generation"
            );

            let mut reports = Vec::new();
            for _ in 0..2 {
                if b_first {
                    reports.push(("B", b.sync(&server)));
                    reports.push(("A", a.sync(&server)));
                } else {
                    reports.push(("A", a.sync(&server)));
                    reports.push(("B", b.sync(&server)));
                }
            }
            for (mac, store) in [("A", &a.store), ("B", &b.store)] {
                let (rev, deleted) = all_docs(store)[id].clone();
                assert!(!deleted, "{case}: deleted on {mac}");
                assert_eq!(hash(&rev), hash(&b_edit), "{case}: {mac} lacks B's edit");
                assert_eq!(pending(store), 0, "{case}: {mac}");
            }
            assert_eq!(
                server_docs(&server, &a.session).get(id).map(|(_, d)| *d),
                Some(false),
                "{case}: the account's copy"
            );
            if as_file {
                let c = Mac::new(sign_in(&server, "alice", 3));
                c.sync(&server);
                for (mac, store) in [("A", &a.store), ("B", &b.store), ("C", &c.store)] {
                    assert_eq!(
                        bytes(store, id).as_deref(),
                        Some(&b"B's edit"[..]),
                        "{case}: {mac}'s bytes"
                    );
                }
            }
            // Every flag names a document the flagging Mac can open.
            for (mac, report) in &reports {
                for flagged in &report.conflicted {
                    let store = if *mac == "A" { &a.store } else { &b.store };
                    assert!(
                        store.get(flagged).unwrap().is_some(),
                        "{case}: {mac} flagged {flagged}, which is gone"
                    );
                }
            }
        }
    }
}

/// Review-sync P5, with the epoch the admin's `restore` rotates: a rewound
/// account re-merges on every Mac, so no Mac skips another's documents and
/// every lost document goes back, even after others pushed past the cursor.
#[test]
fn a_rewind_with_a_new_epoch_loses_nothing_once_others_push_past_it() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    note(&a.store, "note:base", "trash", "base", 100);
    a.sync(&server);
    b.sync(&server);
    a.sync(&server);
    b.sync(&server);
    let c = a.cursor().max(b.cursor());
    note(
        &a.store,
        "note:a-late",
        "trash",
        "A's, lost by the rewind",
        200,
    );
    a.sync(&server);
    a.sync(&server);
    server.rewind("alice", c);
    server.set_epoch("alice");
    for i in 0..3 {
        note(&b.store, &format!("note:b{i}"), "trash", "B's", 300);
    }
    assert!(b.sync(&server).remerged);
    assert!(a.sync(&server).remerged);
    b.sync(&server);
    for i in 0..3 {
        assert!(
            body(&a.store, &format!("note:b{i}")).is_some(),
            "A skipped note:b{i}"
        );
    }
    assert!(server_docs(&server, &a.session).contains_key("note:a-late"));
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
    assert_eq!(all_docs(&a.store), server_docs(&server, &a.session));
}

#[derive(Clone, Debug)]
enum RestoreOp {
    /// Mac `m` edits (or re-creates) note `pick`; `bump` moves `updatedAt`.
    Write {
        m: usize,
        pick: usize,
        bump: bool,
    },
    /// Mac `m` deletes note `pick`, from the UI (whose snapshots are
    /// throttled) or the API.
    Delete {
        m: usize,
        pick: usize,
        ui: bool,
    },
    Sync {
        m: usize,
    },
    /// The owner takes a Time Travel bookmark.
    Snapshot,
    /// The owner restores the last bookmark (which rotates the epoch).
    Restore,
}

fn gen_restore_op() -> impl Strategy<Value = RestoreOp> {
    prop_oneof![
        5 => (0..3usize, 0..3usize, any::<bool>()).prop_map(|(m, pick, bump)| RestoreOp::Write { m, pick, bump }),
        1 => (0..3usize, 0..3usize, any::<bool>()).prop_map(|(m, pick, ui)| RestoreOp::Delete { m, pick, ui }),
        5 => (0..3usize).prop_map(|m| RestoreOp::Sync { m }),
        1 => Just(RestoreOp::Snapshot),
        1 => Just(RestoreOp::Restore),
    ]
}

/// The node every tombstone shares in the ancestry graph.
const TOMBSTONE: &str = "tombstone";

/// A document's current content hash on `store`, or [`TOMBSTONE`].
fn content(store: &Store, id: &str) -> String {
    match all_docs(store).get(id) {
        Some((rev, false)) => hash(rev),
        _ => TOMBSTONE.to_owned(),
    }
}

/// Every document `report` names both stale and conflicted is live on
/// `store`: a stale conflict keeps the local content, which is never a
/// tombstone (review-verify 2).
fn stale_conflicts_are_live(store: &Store, report: &SyncReport) -> Result<(), TestCaseError> {
    for id in report
        .conflicted
        .iter()
        .filter(|id| report.stale.contains(id))
    {
        prop_assert!(
            store.get(id).unwrap().is_some(),
            "{} was flagged stale over a deletion: {:?}",
            id,
            report
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(config(48))]

    /// Invariant (b), over three Macs with random edits, deletions (from
    /// the UI or the API), syncs, bookmarks and restores: every live version
    /// a Mac wrote is either an ancestor of the document's final version
    /// (something was built on it), or a sync after it was written flagged
    /// the document in `conflicted` and the version is still current or in
    /// history on some Mac. A document a sync reports both stale and
    /// conflicted is live on that Mac afterwards: below the floor, a local
    /// deletion never beats a version this Mac never held (review-verify 2), so
    /// the flag leads to a note the user can open. The Macs converge, and
    /// nothing is stale without a restore.
    #[test]
    fn prop_no_live_edit_is_lost_silently(ops in prop::collection::vec(gen_restore_op(), 1..36)) {
        let server = MemoryServer::new();
        let macs = [
            Mac::new(sign_up(&server, "alice", 1)),
            Mac::new(sign_in(&server, "alice", 2)),
            Mac::new(sign_in(&server, "alice", 3)),
        ];
        let ids = ["note:p0", "note:p1", "note:p2"];
        for id in ids {
            note(&macs[0].store, id, "trash", "seed", 1);
        }
        for mac in &macs {
            mac.sync(&server);
        }
        // (doc, content hash) -> the content hashes it was built on.
        let mut parents: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
        // Live versions written: (step, doc, rev).
        let mut written: Vec<(usize, String, String)> = Vec::new();
        // doc -> steps of the syncs that flagged it.
        let mut flagged: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut snapshot = None;
        let mut restored = false;
        let mut clock = 10_000;
        for (step, op) in ops.iter().enumerate() {
            clock += 1;
            match op {
                RestoreOp::Write { m, pick, bump } => {
                    let store = &macs[*m].store;
                    let id = ids[*pick];
                    let before = content(store, id);
                    let text = format!("{m} at {clock}");
                    let rev = match store.get_api(id, None, false) {
                        Ok(mut doc) => {
                            doc["body"] = text.into();
                            if *bump {
                                doc["updatedAt"] = clock.into();
                            }
                            put(store, DocType::Note, doc).unwrap().rev
                        }
                        Err(_) => put(
                            store,
                            DocType::Note,
                            json!({"_id": id, "bookId": "trash", "body": text, "createdAt": 1, "updatedAt": clock}),
                        )
                        .unwrap()
                        .rev,
                    };
                    parents.entry((id.to_owned(), hash(&rev))).or_default().insert(before);
                    written.push((step, id.to_owned(), rev));
                }
                RestoreOp::Delete { m, pick, ui } => {
                    let store = &macs[*m].store;
                    let id = ids[*pick];
                    let before = content(store, id);
                    let origin = if *ui { Origin::Ui } else { Origin::Api };
                    if store.delete(id, origin).is_ok() {
                        parents.entry((id.to_owned(), TOMBSTONE.to_owned())).or_default().insert(before);
                    }
                }
                RestoreOp::Sync { m } => {
                    let report = macs[*m].sync(&server);
                    prop_assert!(report.invalid.is_empty() && report.skipped.is_empty());
                    prop_assert!(restored || report.stale.is_empty(), "stale without a restore: {:?}", report);
                    stale_conflicts_are_live(&macs[*m].store, &report)?;
                    for id in &report.conflicted {
                        flagged.entry(id.clone()).or_default().push(step);
                    }
                }
                RestoreOp::Snapshot => snapshot = Some(server.snapshot("alice")),
                RestoreOp::Restore => {
                    if let Some(snapshot) = &snapshot {
                        server.restore("alice", snapshot);
                        restored = true;
                    }
                }
            }
        }
        let last = ops.len();
        for round in 0..3 {
            for mac in &macs {
                let report = mac.sync(&server);
                stale_conflicts_are_live(&mac.store, &report)?;
                for id in &report.conflicted {
                    flagged.entry(id.clone()).or_default().push(last + round);
                }
            }
        }
        let on_server = server_docs(&server, &macs[0].session);
        for mac in &macs {
            prop_assert_eq!(pending(&mac.store), 0);
            prop_assert_eq!(&all_docs(&mac.store), &on_server);
        }
        for (step, id, rev) in &written {
            let last_version = content(&macs[0].store, id);
            let mut ancestors = BTreeSet::from([last_version.clone()]);
            let mut todo = vec![last_version];
            while let Some(node) = todo.pop() {
                for parent in parents.get(&(id.clone(), node)).into_iter().flatten() {
                    if ancestors.insert(parent.clone()) {
                        todo.push(parent.clone());
                    }
                }
            }
            if ancestors.contains(&hash(rev)) {
                continue;
            }
            let reported = flagged.get(id).is_some_and(|steps| steps.iter().any(|s| s > step));
            prop_assert!(reported, "{} {} (step {}) was dropped and never flagged", id, rev, step);
            prop_assert!(
                macs.iter().any(|mac| holds(&mac.store, id, rev)),
                "{} {} (step {}) was dropped and kept nowhere", id, rev, step
            );
        }
    }
}

// ---------------------------------------------------------------------------
// d. Our own echo after a lost push answer
// ---------------------------------------------------------------------------

/// Review-sync P3: the push of an API merge (which keeps `updatedAt`) is
/// stored but its answer is lost; another merge follows. The echo is
/// recognised as ours: the later merge isn't reverted and nothing is
/// flagged. Also with editor-style edits that move `updatedAt`.
#[test]
fn a_lost_push_answer_then_another_edit_keeps_the_edit() {
    for editor in [false, true] {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        let b = Mac::new(sign_in(&server, "alice", 2));
        note(&a.store, "note:x", "trash", "v1", 100);
        a.sync(&server);
        let write = |text: &str, at: i64| {
            if editor {
                edit_note(&a.store, "note:x", text, Some(at));
            } else {
                merge(
                    &a.store,
                    DocType::Note,
                    json!({"_id": "note:x", "body": text}),
                )
                .unwrap();
            }
        };
        write("v2", 200);
        server.drop_next_responses(kanso_sync::Route::Push, 1);
        assert!(a.try_sync(&server).is_err());
        write("v3", 300);
        let report = a.sync(&server);
        let case = if editor { "editor" } else { "API merge" };
        assert_eq!(
            body(&a.store, "note:x").as_deref(),
            Some("v3"),
            "{case}: the later edit was reverted"
        );
        assert!(
            report.conflicted.is_empty(),
            "{case}: a false conflict: {report:?}"
        );
        assert_eq!(report.conflicts(), 0, "{case}");
        assert_eq!(pending(&a.store), 0, "{case}");
        b.sync(&server);
        assert_eq!(body(&b.store, "note:x").as_deref(), Some("v3"), "{case}");
        assert!(
            a.store
                .revisions("note:x")
                .unwrap()
                .iter()
                .all(|r| r.origin != "sync"),
            "{case}"
        );
    }
}

#[derive(Clone, Debug)]
enum EchoOp {
    /// Edit note `pick` on A; `bump` moves `updatedAt` (editor), else an
    /// API merge keeps it.
    Edit {
        pick: usize,
        bump: bool,
    },
    Delete {
        pick: usize,
    },
    Sync,
    /// A syncs, and the server applies its push but the answer is lost.
    SyncLosingTheAnswer,
    /// B (the account's other Mac) syncs: it only reads.
    SyncB,
}

fn gen_echo_op() -> impl Strategy<Value = EchoOp> {
    prop_oneof![
        5 => (0..3usize, any::<bool>()).prop_map(|(pick, bump)| EchoOp::Edit { pick, bump }),
        1 => (0..3usize).prop_map(|pick| EchoOp::Delete { pick }),
        2 => Just(EchoOp::Sync),
        3 => Just(EchoOp::SyncLosingTheAnswer),
        1 => Just(EchoOp::SyncB),
    ]
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant (d): with one writer whose push answers get lost at random,
    /// no sync ever flags a conflict (there is none), and after honest syncs
    /// every note holds the writer's last version, on both Macs and the
    /// server. Our own echo never reverts a newer edit.
    #[test]
    fn prop_our_own_echo_never_reverts_an_edit(ops in prop::collection::vec(gen_echo_op(), 1..30)) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        let b = Mac::new(sign_in(&server, "alice", 2));
        let ids = ["note:e0", "note:e1", "note:e2"];
        for id in ids {
            note(&a.store, id, "trash", "seed", 1);
        }
        a.sync(&server);
        let mut clock = 1_000;
        for op in &ops {
            clock += 1;
            match op {
                EchoOp::Edit { pick, bump } => {
                    let id = ids[*pick];
                    if a.store.get(id).unwrap().is_some() {
                        if *bump {
                            edit_note(&a.store, id, &format!("at {clock}"), Some(clock));
                        } else {
                            merge(&a.store, DocType::Note, json!({"_id": id, "body": format!("at {clock}")})).unwrap();
                        }
                    }
                }
                EchoOp::Delete { pick } => {
                    let _ = a.store.delete(ids[*pick], Origin::Api);
                }
                EchoOp::Sync => {
                    let report = a.sync(&server);
                    prop_assert!(report.conflicted.is_empty(), "{:?}", report);
                }
                EchoOp::SyncLosingTheAnswer => {
                    server.drop_next_responses(kanso_sync::Route::Push, 1);
                    if let Ok(report) = a.try_sync(&server) {
                        prop_assert!(report.conflicted.is_empty(), "{:?}", report);
                    }
                    server.drop_next_responses(kanso_sync::Route::Push, 0);
                }
                EchoOp::SyncB => {
                    let report = b.sync(&server);
                    prop_assert!(report.conflicted.is_empty(), "{:?}", report);
                }
            }
        }
        let last = live_docs(&a.store);
        for _ in 0..2 {
            let report = a.sync(&server);
            prop_assert!(report.conflicted.is_empty(), "{:?}", report);
        }
        b.sync(&server);
        prop_assert_eq!(&live_docs(&a.store), &last, "a sync changed A's own notes");
        prop_assert_eq!(pending(&a.store), 0);
        prop_assert_eq!(&live_docs(&b.store), &last);
        prop_assert_eq!(server_live(&server, &a.session), last);
    }
}

// ---------------------------------------------------------------------------
// e. Missing attachment bytes come back
// ---------------------------------------------------------------------------

/// What `store`'s blob for `id` holds, if anything.
fn bytes(store: &Store, id: &str) -> Option<Vec<u8>> {
    store.attachment(id).unwrap().map(|(_, blob)| blob.data)
}

/// Writes file `id` with `data`, replacing whatever version `store` has.
fn write_file(store: &Store, id: &str, data: &[u8]) {
    let mut body = file_body(id, "image.png", data);
    if let Some((rev, _)) = all_docs(store).remove(id) {
        body["_rev"] = json!(rev);
    }
    put(store, DocType::File, body).unwrap();
}

/// A backend that loses every file's bytes.
fn losing_bytes(
    server: &MemoryServer,
) -> Tap<'_, impl FnMut(Event) -> Result<(), kanso_sync::ApiError> + Send> {
    Tap::new(server, |event| {
        if let Event::File(bytes) = event {
            *bytes = None;
        }
        Ok(())
    })
}

/// Review-sync P4: a file whose bytes didn't come with the first pull is
/// fetched by the next ordinary sync, with nothing new in the feed, and not
/// asked for again once it is there.
#[test]
fn missing_bytes_are_fetched_by_the_next_sync() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    file(&a.store, "file:f1", b"\x89PNG bytes");
    a.sync(&server);
    assert_eq!(b.sync(&losing_bytes(&server)).files_missing, 1);
    assert_eq!(bytes(&b.store, "file:f1"), None);
    let report = b.sync(&server);
    assert_eq!((report.files_down, report.files_missing), (1, 0));
    assert_eq!(
        bytes(&b.store, "file:f1").as_deref(),
        Some(&b"\x89PNG bytes"[..])
    );
    let gets = server.stats().count(Route::GetFile);
    assert_eq!(b.sync(&server).files_down, 0);
    assert_eq!(server.stats().count(Route::GetFile), gets);
}

/// Bytes fetched again for one revision never land on a newer one: the file
/// is replaced on this Mac while its old bytes are on the way. The guarded
/// write sees the new revision and keeps the new bytes, which then reach
/// the account.
#[test]
fn refetched_bytes_never_replace_a_newer_versions_bytes() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    file(&a.store, "file:f1", b"old bytes");
    a.sync(&server);
    assert_eq!(b.sync(&losing_bytes(&server)).files_missing, 1);
    let race = Tap::new(&server, |event| {
        if let Event::File(_) = event {
            write_file(&b.store, "file:f1", b"new bytes");
        }
        Ok(())
    });
    let report = b.sync(&race);
    assert_eq!(report.files_down, 0, "{report:?}");
    assert_eq!(
        bytes(&b.store, "file:f1").as_deref(),
        Some(&b"new bytes"[..])
    );
    assert_eq!(pending(&b.store), 0);
    a.sync(&server);
    assert_eq!(
        bytes(&a.store, "file:f1").as_deref(),
        Some(&b"new bytes"[..])
    );
}

/// Too many missing files: each sync asks for at most
/// `SyncOptions::refetch_files`, going round from where the last stopped.
#[test]
fn refetching_is_bounded_and_goes_round() {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    for i in 0..5 {
        file(
            &a.store,
            &format!("file:f{i}"),
            format!("bytes {i}").as_bytes(),
        );
    }
    a.sync(&server);
    let options = SyncOptions {
        refetch_files: 2,
        ..SyncOptions::default()
    };
    let lose = losing_bytes(&server);
    assert_eq!(b.sync_with(&lose, &options).unwrap().files_missing, 5);
    let gets = server.stats().count(Route::GetFile);
    assert_eq!(b.sync_with(&lose, &options).unwrap().files_missing, 2);
    assert_eq!(server.stats().count(Route::GetFile), gets + 2);
    // f0 and f1 were asked for last: f2 to f4 come first, then round again.
    for (want, got, not_yet) in [
        (2, ["file:f2", "file:f3"], "file:f4"),
        (2, ["file:f4", "file:f0"], "file:f1"),
    ] {
        let report = b.sync_with(&server, &options).unwrap();
        assert_eq!(report.files_down, want, "{report:?}");
        for id in got {
            assert_eq!(bytes(&b.store, id), bytes(&a.store, id), "{id}");
        }
        assert_eq!(bytes(&b.store, not_yet), None, "{not_yet}");
    }
    assert_eq!(b.sync_with(&server, &options).unwrap().files_down, 1);
    for i in 0..5 {
        let id = format!("file:f{i}");
        assert_eq!(bytes(&b.store, &id), bytes(&a.store, &id), "{id}");
    }
}

#[derive(Clone, Debug)]
struct BytesRound {
    /// A replaces file `.0`'s bytes (version `.1`) and syncs first.
    edit: Option<(usize, u8)>,
    /// What happens to each `GET /v2/files` of B's sync, in order: 0
    /// delivers, 1 loses the bytes, 2 answers garbage. Past the end: deliver.
    faults: Vec<u8>,
}

fn gen_bytes_rounds() -> impl Strategy<Value = (usize, Vec<BytesRound>)> {
    (1usize..=5).prop_flat_map(|files| {
        let round = (
            proptest::option::of((0..files, any::<u8>())),
            prop::collection::vec(0u8..3, 0..8),
        )
            .prop_map(|(edit, faults)| BytesRound { edit, faults });
        (Just(files), prop::collection::vec(round, 1..6))
    })
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant (e): whatever bytes the server loses or garbles, B only
    /// ever holds bytes A wrote for that file and never asks for more than
    /// every file plus `refetch_files` in one sync. Once the server answers
    /// honestly B catches up within a bounded number of ordinary syncs, and
    /// then stops asking.
    #[test]
    fn prop_missing_bytes_come_back((files, rounds) in gen_bytes_rounds()) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        let b = Mac::new(sign_in(&server, "alice", 2));
        let ids: Vec<String> = (0..files).map(|i| format!("file:f{i}")).collect();
        let mut written: BTreeMap<String, BTreeSet<Vec<u8>>> = BTreeMap::new();
        let write = |written: &mut BTreeMap<String, BTreeSet<Vec<u8>>>, i: usize, v: u16| {
            let data = format!("bytes of {i}, version {v}").into_bytes();
            write_file(&a.store, &ids[i], &data);
            written.entry(ids[i].clone()).or_default().insert(data);
        };
        for i in 0..files {
            write(&mut written, i, 0);
        }
        a.sync(&server);
        let options = SyncOptions { refetch_files: 2, ..SyncOptions::default() };
        for round in &rounds {
            if let Some((i, v)) = round.edit {
                write(&mut written, i, u16::from(v) + 1);
                a.sync(&server);
            }
            let gets = std::sync::atomic::AtomicUsize::new(0);
            let mut faults = round.faults.clone().into_iter();
            let tap = Tap::new(&server, |event| {
                if let Event::File(bytes) = event {
                    gets.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    match faults.next() {
                        Some(1) => *bytes = None,
                        Some(2) => *bytes = Some(vec![7; 64]),
                        _ => {}
                    }
                }
                Ok(())
            });
            b.sync_with(&tap, &options).unwrap();
            drop(tap);
            prop_assert!(gets.into_inner() <= files + 2);
            for id in &ids {
                if let Some(held) = bytes(&b.store, id) {
                    prop_assert!(written[id].contains(&held), "{} holds bytes A never wrote", id);
                }
            }
        }
        for _ in 0..files / 2 + 2 {
            b.sync_with(&server, &options).unwrap();
        }
        for id in &ids {
            prop_assert_eq!(bytes(&b.store, id), bytes(&a.store, id), "{}", id);
        }
        let gets = server.stats().count(Route::GetFile);
        b.sync_with(&server, &options).unwrap();
        prop_assert_eq!(server.stats().count(Route::GetFile), gets, "asked again after catching up");
    }
}

// ---------------------------------------------------------------------------
// f. Over quota, writes only grow by what is sent
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum QuotaOp {
    /// Creates `note:q<i>` if it isn't there.
    New(usize),
    /// Edits `note:q<i>`: a long body when `grow`, else a short one.
    Edit(usize, bool),
    Delete(usize),
    /// Writes `file:f<i>` with `40 × len + 1` bytes.
    File(usize, u8),
    /// Sets one quota (0 docs, 1 doc_bytes, 2 files, 3 file_bytes) to the
    /// account's usage plus `delta` (bytes quotas: `delta × 300`).
    Squeeze(usize, i8),
    /// Back to the default quotas.
    Relax,
    Sync,
}

fn gen_quota_op() -> impl Strategy<Value = QuotaOp> {
    prop_oneof![
        3 => (0..6usize).prop_map(QuotaOp::New),
        2 => (0..6usize, any::<bool>()).prop_map(|(i, grow)| QuotaOp::Edit(i, grow)),
        2 => (0..6usize).prop_map(QuotaOp::Delete),
        2 => (0..3usize, 0..20u8).prop_map(|(i, len)| QuotaOp::File(i, len)),
        2 => (0..4usize, -2..4i8).prop_map(|(k, d)| QuotaOp::Squeeze(k, d)),
        1 => Just(QuotaOp::Relax),
        5 => Just(QuotaOp::Sync),
    ]
}

/// The quota entries a sync left, as written.
fn quota_entries(store: &Store) -> [Option<String>; 2] {
    [
        store.meta(kanso_sync::meta_keys::QUOTA_DOCS).unwrap(),
        store.meta(kanso_sync::meta_keys::QUOTA_FILES).unwrap(),
    ]
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant (f) (review-sync 6): with quotas squeezed and relaxed at
    /// random under one writer,
    /// - a sync right after another, with nothing changed in between, gets
    ///   no 413: what was refused isn't sent again until usage drops, and
    ///   then only what fits;
    /// - one sync gets at most four refused pushes and three refused
    ///   uploads, so a refusal costs a bounded number of writes;
    /// - a deletion of a document the server has always goes, unless even
    ///   a batch of deletions was refused (level `all`);
    /// - `report.quota` is set exactly while an entry is in place;
    /// - once quotas are relaxed, two syncs send everything and clear the
    ///   entries.
    #[test]
    fn prop_over_quota_writes_only_grow_by_what_is_sent(
        ops in prop::collection::vec(gen_quota_op(), 1..40),
    ) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        book(&a.store, "book:b1", "Inbox");
        for i in 0..3 {
            note(&a.store, &format!("note:q{i}"), "book:b1", "seed", 1);
        }
        a.sync(&server);
        let mut quota = QuotaOverride::default();
        let mut clock = 1_000;
        let mut idle = false;
        for op in &ops {
            clock += 1;
            match op {
                QuotaOp::New(i) => {
                    let id = format!("note:q{i}");
                    if a.store.get(&id).unwrap().is_none() {
                        note(&a.store, &id, "book:b1", "new", clock);
                    }
                }
                QuotaOp::Edit(i, grow) => {
                    let id = format!("note:q{i}");
                    if a.store.get(&id).unwrap().is_some() {
                        let text = if *grow { "grown ".repeat(80) } else { format!("short {clock}") };
                        edit_note(&a.store, &id, &text, Some(clock));
                    }
                }
                QuotaOp::Delete(i) => {
                    let _ = a.store.delete(&format!("note:q{i}"), Origin::Api);
                }
                QuotaOp::File(i, len) => {
                    let data = vec![b'x' + (clock % 3) as u8; 40 * usize::from(*len) + 1];
                    write_file(&a.store, &format!("file:f{i}"), &data);
                }
                QuotaOp::Squeeze(kind, delta) => {
                    let usage = kanso_sync::usage(&server, &a.session).unwrap();
                    let set = |used: u64, step: i64| Some(used.saturating_add_signed(i64::from(*delta) * step).max(1));
                    match kind {
                        0 => quota.docs = set(usage.docs, 1),
                        1 => quota.doc_bytes = set(usage.doc_bytes, 300),
                        2 => quota.files = set(usage.files, 1),
                        _ => quota.file_bytes = set(usage.file_bytes, 300),
                    }
                    server.set_quota("alice", quota);
                }
                QuotaOp::Relax => {
                    quota = QuotaOverride::default();
                    server.set_quota("alice", quota);
                }
                QuotaOp::Sync => {
                    let pushes = std::sync::atomic::AtomicUsize::new(0);
                    let answers = std::sync::atomic::AtomicUsize::new(0);
                    let uploads = std::sync::atomic::AtomicUsize::new(0);
                    let tap = Tap::new(&server, |event| {
                        let counter = match event {
                            Event::Push(_) => &pushes,
                            Event::Pushed(_) => &answers,
                            Event::PutFile { .. } => &uploads,
                            _ => return Ok(()),
                        };
                        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok(())
                    });
                    let report = a.sync(&tap);
                    drop(tap);
                    let refused_pushes = pushes.into_inner() - answers.into_inner();
                    let refused_uploads = uploads.into_inner() - report.files_up;
                    if idle {
                        prop_assert_eq!(
                            (refused_pushes, refused_uploads), (0, 0),
                            "a sync with nothing new was refused again: {:?}", report
                        );
                    }
                    prop_assert!(refused_pushes <= 4, "{} refused pushes", refused_pushes);
                    prop_assert!(refused_uploads <= 3, "{} refused uploads", refused_uploads);
                    let entries = quota_entries(&a.store);
                    prop_assert_eq!(
                        report.quota.is_some(),
                        entries.iter().any(Option::is_some),
                        "{:?} {:?}", report, entries
                    );
                    let level_all = entries[0].as_deref().is_some_and(|e| e.ends_with(" all"));
                    let on_server = server_docs(&server, &a.session);
                    for (id, (_, deleted)) in all_docs(&a.store) {
                        if deleted && on_server.get(&id).is_some_and(|(_, gone)| !gone) {
                            prop_assert!(level_all, "the deletion of {} was held: {:?}", id, entries);
                        }
                    }
                }
            }
            idle = matches!(op, QuotaOp::Sync);
        }
        server.set_quota("alice", QuotaOverride::default());
        a.sync(&server);
        let report = a.sync(&server);
        prop_assert_eq!((report.pending, report.quota), (0, None));
        prop_assert_eq!(quota_entries(&a.store), [None, None]);
        prop_assert_eq!(server_live(&server, &a.session), live_docs(&a.store));
    }
}
