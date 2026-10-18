//! Device identity and library binding (docs/ACCOUNTS.md › Device identity,
//! › Library binding): stable, unlinkable device IDs; a copied library gets a
//! new identity and keeps its binding; a model of the binding plan and its
//! choices; and re-binding during a sync (fix 2). Tests never touch the real
//! Keychain. CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::binding::meta_keys;
use kanso_sync::{
    Choice, DeviceIdentity, Flow, LocalLibrary, Machine, MemorySecrets, MemoryServer, Password,
    Plan, SecretStore, Session, bind, device_identity, keychain, local_library, plan, unbind,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

const SERVER: &str = "http://127.0.0.1:8787";

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

fn machine(uuid: Option<&str>) -> Machine {
    Machine {
        hardware_uuid: uuid.map(str::to_owned),
        computer_name: "Studio\n".into(),
    }
}

/// Signs `username` up on `server` from `device`.
fn account(server: &MemoryServer, username: &str, device: &DeviceIdentity) -> Session {
    let password = Password::new("correct horse battery", username).unwrap();
    kanso_sync::sign_up(
        server,
        &kanso_sync::MemorySecrets::new(),
        SERVER,
        username,
        &password,
        device,
        &|_| {},
    )
    .unwrap()
    .session
}

fn binding(store: &Store) -> (Option<String>, Option<String>) {
    (
        store.meta(meta_keys::ORIGIN).unwrap(),
        store.meta(meta_keys::ACCOUNT_ID).unwrap(),
    )
}

#[test]
fn device_ids_are_stable_per_account_and_unlinkable_across_accounts() {
    let (_dir, store) = temp_store();
    let here = machine(Some("UUID-1"));
    let identity = device_identity(&store, &here).unwrap();
    assert!(identity.library_id.starts_with("l_"));
    assert_eq!(identity.name, "Studio");
    let again = device_identity(&store, &here).unwrap();
    assert_eq!(again.library_id, identity.library_id);
    let (a, b) = (
        "a_0123456789abcdef0123456789abcdef",
        "a_fedcba9876543210fedcba9876543210",
    );
    assert_eq!(identity.device_id(a), again.device_id(a));
    assert_ne!(identity.device_id(a), identity.device_id(b));
    assert!(identity.device_id(a).starts_with("d_"));
    assert_eq!(identity.device_id(a).len(), 34);
    assert_ne!(
        identity.device_proof(SERVER, a).to_base64url(),
        identity
            .device_proof("https://kanso.example", a)
            .to_base64url()
    );
    assert_eq!(
        store.meta(meta_keys::HARDWARE).unwrap(),
        Some(here.hardware_hash())
    );
    // No UUID: the check is skipped and the identity stays.
    assert_eq!(
        device_identity(&store, &machine(None)).unwrap().library_id,
        identity.library_id
    );
}

#[test]
fn a_copied_library_gets_a_new_identity_and_keeps_its_binding() {
    let (_dir, store) = temp_store();
    let identity = device_identity(&store, &machine(Some("UUID-1"))).unwrap();
    store.set_meta(meta_keys::ORIGIN, SERVER).unwrap();
    store
        .set_meta(meta_keys::ACCOUNT_ID, "a_0123456789abcdef0123456789abcdef")
        .unwrap();
    store.set_meta(meta_keys::LAST_SEQ, "42").unwrap();
    let copy = device_identity(&store, &machine(Some("UUID-2"))).unwrap();
    assert_ne!(copy.library_id, identity.library_id);
    let account = "a_0123456789abcdef0123456789abcdef";
    assert_ne!(copy.device_id(account), identity.device_id(account));
    assert_eq!(
        store.meta(meta_keys::LAST_SEQ).unwrap().as_deref(),
        Some("42")
    );
    assert_eq!(binding(&store).1.as_deref(), Some(account));
    // And it is stable on the new Mac.
    assert_eq!(
        device_identity(&store, &machine(Some("UUID-2")))
            .unwrap()
            .library_id,
        copy.library_id
    );
    // A library made before the hardware was known adopts this Mac's.
    let (_dir2, older) = temp_store();
    let first = device_identity(&older, &machine(None)).unwrap();
    assert_eq!(
        older.meta(meta_keys::HARDWARE).unwrap().as_deref(),
        Some("")
    );
    let later = device_identity(&older, &machine(Some("UUID-3"))).unwrap();
    assert_eq!(later.library_id, first.library_id);
    assert_eq!(
        older.meta(meta_keys::HARDWARE).unwrap(),
        Some(machine(Some("UUID-3")).hardware_hash())
    );
}

/// Replace backs up first, then swaps the library in one transaction: at
/// any point the library is either the old one, still bound as before, or
/// empty and bound to the new account.
#[test]
fn replace_leaves_the_old_library_or_an_empty_one_bound_fresh() {
    let server = MemoryServer::new();
    let (dir, store) = temp_store();
    let identity = device_identity(&store, &machine(Some("UUID-1"))).unwrap();
    let alice = account(&server, "alice", &identity);
    let bob = account(&server, "bob", &identity);
    bind(&store, &alice, Choice::Add, &dir.path().join("backups"), 0).unwrap();
    for n in 0..3 {
        note(&store, n);
    }
    let before = docs(&store);
    // Step 1 alone (a crash right after it): the old library, still alice's.
    let backup = store.backup_now(&dir.path().join("backups"), 1).unwrap();
    assert_eq!(docs(&store), before);
    assert_eq!(binding(&store).1, Some(alice.account_id.clone()));
    // Both steps: empty and bound to bob, with the cursor at zero.
    let path = bind(
        &store,
        &bob,
        Choice::Replace,
        &dir.path().join("backups"),
        2,
    )
    .unwrap()
    .unwrap();
    assert!(docs(&store).is_empty());
    assert_eq!(binding(&store).1, Some(bob.account_id.clone()));
    assert_eq!(
        store.meta(meta_keys::LAST_SEQ).unwrap().as_deref(),
        Some("0")
    );
    for copy in [backup, path] {
        assert_eq!(docs(&Store::open(copy).unwrap()), before);
    }
}

fn note(store: &Store, n: usize) -> String {
    store
        .put(
            DocType::Note,
            json!({ "bookId": "trash", "title": format!("n{n}"), "body": "x" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Every document, tombstones included: ID → (rev, deleted).
fn docs(store: &Store) -> BTreeMap<String, (String, bool)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted)))
        .collect()
}

fn sync_meta(store: &Store) -> Vec<Option<String>> {
    [
        meta_keys::ORIGIN,
        meta_keys::ACCOUNT_ID,
        meta_keys::USERNAME,
        meta_keys::SERVER,
        meta_keys::LAST_SEQ,
        meta_keys::EPOCH,
        meta_keys::LIBRARY_ID,
        meta_keys::DEVICE_SECRET,
        meta_keys::HARDWARE,
    ]
    .iter()
    .map(|key| store.meta(key).unwrap())
    .collect()
}

#[derive(Clone, Debug)]
enum Op {
    Write,
    Edit(usize),
    Delete(usize),
    /// Everything pending reached the server.
    Synced,
    /// A flow into account 0 or 1, answering a question with this choice.
    Enter(Flow, usize, Choice),
    SignOut(usize),
    Unbind,
}

fn op() -> impl Strategy<Value = Op> {
    let flow = prop_oneof![Just(Flow::SignUp), Just(Flow::SignIn), Just(Flow::Recover)];
    let choice = prop_oneof![
        Just(Choice::Add),
        Just(Choice::Replace),
        Just(Choice::Cancel)
    ];
    prop_oneof![
        3 => Just(Op::Write),
        2 => any::<usize>().prop_map(Op::Edit),
        2 => any::<usize>().prop_map(Op::Delete),
        2 => Just(Op::Synced),
        4 => (flow, 0..2usize, choice).prop_map(|(f, a, c)| Op::Enter(f, a, c)),
        1 => (0..2usize).prop_map(Op::SignOut),
        1 => Just(Op::Unbind),
    ]
}

/// What the test expects of the library.
#[derive(Default)]
struct Model {
    bound: Option<usize>,
    live: BTreeSet<String>,
    pending: BTreeSet<String>,
}

proptest! {
    #![proptest_config(config(64))]

    /// Model-based: the plan follows the table; notes reach another account
    /// only through an explicit add (merging makes exactly the live notes
    /// pending, with no tombstones); Replace always leaves a backup with
    /// every document and an empty library bound fresh; Cancel and sign-out
    /// never change the library; unbind keeps the identity.
    #[test]
    fn prop_binding_plan(ops in prop::collection::vec(op(), 1..30)) {
        let server = MemoryServer::new();
        let (dir, store) = temp_store();
        let backups = dir.path().join("backups");
        let identity = device_identity(&store, &machine(Some("UUID-1"))).unwrap();
        let sessions = [account(&server, "alice", &identity), account(&server, "bob", &identity)];
        let secrets = MemorySecrets::new();
        let mut model = Model::default();
        let mut now = 0;
        for op in ops {
            now += 1;
            match op {
                Op::Write => {
                    let id = note(&store, now as usize);
                    model.live.insert(id.clone());
                    model.pending.insert(id);
                }
                Op::Edit(k) | Op::Delete(k) if model.live.is_empty() => { let _ = k; }
                Op::Edit(k) => {
                    let id = model.live.iter().nth(k % model.live.len()).unwrap().clone();
                    let mut doc = store.get_api(&id, None, false).unwrap();
                    doc["body"] = format!("edit {now}").into();
                    store.put(DocType::Note, doc, Put::Upsert, Origin::Ui).unwrap();
                    model.pending.insert(id);
                }
                Op::Delete(k) => {
                    let id = model.live.iter().nth(k % model.live.len()).unwrap().clone();
                    store.delete(&id, Origin::Ui).unwrap();
                    model.live.remove(&id);
                    model.pending.insert(id);
                }
                Op::Synced => {
                    for doc in store.pending_push(1000).unwrap() {
                        store.mark_synced(&doc.id, &doc.rev, None).unwrap();
                    }
                    model.pending.clear();
                }
                Op::Enter(flow, a, choice) => {
                    let session = &sessions[a];
                    let library = local_library(&store, &session.origin, &session.account_id).unwrap();
                    let expected_library = match model.bound {
                        Some(b) if b == a => LocalLibrary::Same,
                        _ if model.live.is_empty() => LocalLibrary::Empty,
                        Some(b) => LocalLibrary::Other {
                            username: sessions[b].username.clone(),
                            notes: model.live.len(),
                            unsynced: model.pending.len(),
                        },
                        None => LocalLibrary::Unbound { notes: model.live.len(), unsynced: model.pending.len() },
                    };
                    prop_assert_eq!(&library, &expected_library);
                    let planned = plan(flow, &library);
                    let answer = match &planned {
                        Plan::Fresh | Plan::Resume => Choice::Add,
                        Plan::AskBeforeSignUp { from, notes } => {
                            prop_assert_eq!(flow, Flow::SignUp);
                            prop_assert_eq!(Some(from), model.bound.map(|b| &sessions[b].username));
                            prop_assert_eq!(*notes, model.live.len());
                            if choice == Choice::Replace { Choice::Add } else { choice }
                        }
                        Plan::Ask { from, notes, unsynced } => {
                            prop_assert_ne!(flow, Flow::SignUp);
                            prop_assert_eq!(from.as_ref(), model.bound.map(|b| &sessions[b].username));
                            prop_assert_eq!((*notes, *unsynced), (model.live.len(), model.pending.len()));
                            choice
                        }
                    };
                    if matches!(library, LocalLibrary::Same) {
                        prop_assert_eq!(&planned, &Plan::Resume);
                    }
                    let before = docs(&store);
                    let backup = bind(&store, session, answer, &backups, now).unwrap();
                    match answer {
                        Choice::Cancel => {
                            prop_assert!(backup.is_none());
                            prop_assert_eq!(docs(&store), before);
                        }
                        Choice::Add => {
                            prop_assert!(backup.is_none());
                            if model.bound != Some(a) {
                                // Bound fresh: exactly the live documents are pending, no tombstones.
                                prop_assert!(store.pending_push(1000).unwrap().iter().all(|d| !d.deleted));
                                model.pending = model.live.clone();
                                model.bound = Some(a);
                            }
                        }
                        Choice::Replace => {
                            let path = backup.expect("Replace always backs up");
                            prop_assert_eq!(docs(&Store::open(path).unwrap()), before);
                            prop_assert!(docs(&store).is_empty());
                            model.live.clear();
                            model.pending.clear();
                            model.bound = Some(a);
                        }
                    }
                    if model.bound == Some(a) {
                        prop_assert_eq!(store.meta(meta_keys::USERNAME).unwrap(), Some(session.username.clone()));
                        prop_assert_eq!(store.meta(meta_keys::SERVER).unwrap(), Some(session.server.clone()));
                    }
                }
                Op::SignOut(a) => {
                    let library = identity.library_id.clone();
                    keychain::save_session(&secrets, &library, &sessions[a]).unwrap();
                    let (before, meta) = (docs(&store), sync_meta(&store));
                    kanso_sync::sign_out(&server, &secrets, &sessions[a]).unwrap();
                    prop_assert_eq!(docs(&store), before);
                    prop_assert_eq!(sync_meta(&store), meta);
                    prop_assert!(secrets.get(&keychain::account_item(&library)).unwrap().is_none());
                }
                Op::Unbind => {
                    let identity_meta = sync_meta(&store)[6..].to_vec();
                    unbind(&store, &secrets).unwrap();
                    prop_assert_eq!(binding(&store), (None, None));
                    prop_assert_eq!(store.meta(meta_keys::LEGACY_LAST_SEQ).unwrap(), Some("0".to_owned()));
                    prop_assert_eq!(sync_meta(&store)[6..].to_vec(), identity_meta);
                    model.pending = model.live.clone();
                    model.bound = None;
                }
            }
            // The library always matches the model.
            let live: BTreeSet<String> = docs(&store).into_iter().filter(|(_, (_, d))| !d).map(|(id, _)| id).collect();
            prop_assert_eq!(&live, &model.live);
            let pending: BTreeSet<String> = store.pending_push(1000).unwrap().into_iter().map(|d| d.id).collect();
            prop_assert_eq!(&pending, &model.pending);
            prop_assert_eq!(store.pending_count().unwrap(), model.pending.len());
            let expected = model.bound.map(|b| (sessions[b].origin.clone(), sessions[b].account_id.clone()));
            let (origin, account_id) = binding(&store);
            prop_assert_eq!(origin.zip(account_id), expected);
        }
    }
}

proptest! {
    #![proptest_config(config(64))]

    /// Fix 2 / M6: a library is re-bound to another account (Add or Replace)
    /// between any two of a running sync's calls to the server (one-document
    /// pages and batches, so there are many). No write of the first
    /// account's sync lands after the new binding (the library is exactly as
    /// the bind left it), nothing more is sent to the first account apart
    /// from the call already on its way, the new account's cursor is never
    /// the first one's, and the new account's next sync works on its own
    /// feed.
    #[test]
    fn prop_rebinding_during_a_sync_never_mixes_accounts(
        at in 0..16usize,
        replace in any::<bool>(),
        pending_notes in 0..3usize,
        remote_notes in 0..3usize,
        with_file in any::<bool>(),
    ) {
        let server = MemoryServer::new();
        let a = common::Mac::new(common::sign_up(&server, "alice", 1));
        let other = common::Mac::new(common::sign_in(&server, "alice", 2));
        let bob = common::sign_up(&server, "bob", 3);
        let bobs = common::Mac::new(common::sign_in(&server, "bob", 4));
        // Something for each account's feed, and something pending here.
        common::note(&bobs.store, "note:bob", "trash", "bob's", 1);
        bobs.sync(&server);
        common::note(&a.store, "note:synced", "trash", "synced", 1);
        a.sync(&server);
        for i in 0..remote_notes {
            common::note(&other.store, &format!("note:r{i}"), "trash", "from the other Mac", 2);
        }
        other.sync(&server);
        for i in 0..pending_notes {
            common::note(&a.store, &format!("note:p{i}"), "trash", "pending", 3);
        }
        if with_file {
            common::file(&a.store, "file:f", b"bytes");
        }

        let choice = if replace { Choice::Replace } else { Choice::Add };
        let mut calls = 0;
        let mut sent_after_bind = Vec::new();
        let after_bind = std::sync::Mutex::new(None);
        let tap = common::Tap::new(&server, |event| {
            if calls > at {
                match event {
                    common::Event::Push(docs) => sent_after_bind.push(format!("a push of {}", docs.len())),
                    common::Event::PutFile { .. } => sent_after_bind.push("an upload".to_owned()),
                    _ => {}
                }
            }
            if calls == at {
                bind(&a.store, &bob, choice, &a.dir.path().join("backups"), 1).unwrap();
                *after_bind.lock().unwrap() = Some(common::snapshot(&a.store));
            }
            calls += 1;
            Ok(())
        });
        let small = kanso_sync::SyncOptions { page_size: 1, batch_docs: 1, ..kanso_sync::SyncOptions::default() };
        let result = a.sync_with(&tap, &small);
        drop(tap);
        let Some(bound) = after_bind.into_inner().unwrap() else {
            // The sync made fewer calls than `at`: nothing to race.
            prop_assert!(result.is_ok());
            return Ok(());
        };
        prop_assert!(
            matches!(result, Ok(_) | Err(kanso_sync::SyncError::Cancelled)),
            "{:?}", result
        );
        prop_assert_eq!(&common::snapshot(&a.store), &bound, "a write of alice's landed after bob's binding");
        prop_assert!(sent_after_bind.is_empty(), "sent to alice after bob's binding: {:?}", sent_after_bind);
        prop_assert_eq!(a.store.meta(meta_keys::ACCOUNT_ID).unwrap(), Some(bob.account_id.clone()));
        prop_assert_eq!(a.store.meta(meta_keys::LAST_SEQ).unwrap(), Some("0".to_owned()));
        prop_assert_eq!(a.store.meta(meta_keys::EPOCH).unwrap(), None);

        // Bob's sync runs on bob's feed: his note arrives, his cursor is his.
        let report = kanso_sync::sync(&a.store, &server, &bob, &kanso_sync::SyncOptions::default()).unwrap();
        prop_assert!(!report.remerged);
        prop_assert_eq!(common::body(&a.store, "note:bob"), Some("bob's".to_owned()));
        let bob_view = server.plain_view(&bob);
        let head = bob_view.docs().iter().map(|d| d.seq).max().unwrap_or(0);
        let cursor: i64 = a.store.meta(meta_keys::LAST_SEQ).unwrap().unwrap().parse().unwrap();
        prop_assert!(cursor <= head);
        // Alice's notes reach bob only through Add.
        let reached = bob_view.docs().iter().any(|d| d.id == "note:synced");
        prop_assert_eq!(reached, !replace);
        prop_assert_eq!(server.plain_view(&a.session).unreadable(), 0);
    }
}
