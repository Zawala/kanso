//! Revision generations only grow (M2, docs/ACCOUNTS.md › Revision
//! generations only grow), and the binding operations `reset_sync` and
//! `replace_library` (› Library binding › kanso-store APIs). CI pins the
//! seed with `PROPTEST_RNG_SEED`.

use std::collections::BTreeMap;

use kanso_core::{DocType, Rev};
use kanso_store::{ApplyOutcome, Origin, Put, Store, SyncDoc, SyncGuard};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

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

fn generation(rev: &str) -> u64 {
    rev.parse::<Rev>().unwrap().generation
}

fn book(store: &Store, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({ "name": name }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn rename(store: &Store, id: &str, name: &str, updated_at: i64) {
    let mut doc = store.get_api(id, None, false).unwrap();
    doc["name"] = name.into();
    doc["updatedAt"] = updated_at.into();
    store
        .put(DocType::Book, doc, Put::Upsert, Origin::Ui)
        .unwrap();
}

/// Every document of the store, tombstones included, by ID.
fn all_docs(store: &Store) -> BTreeMap<String, (String, bool, Map<String, Value>)> {
    store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted, d.fields)))
        .collect()
}

/// A server holding each document's current version, accepting a push iff
/// its `base_rev` is that version (the Worker's rule), and keeping every
/// version it ever accepted.
#[derive(Default)]
struct Server {
    docs: BTreeMap<String, SyncDoc>,
    history: Vec<SyncDoc>,
}

impl Server {
    /// Pull everything, then push until nothing is pending. Fails the case
    /// when an accepted revision doesn't have a higher generation than the
    /// one it replaced.
    fn sync(&mut self, mac: &Store) -> Result<(), TestCaseError> {
        for doc in self.docs.values() {
            let outcome = mac.apply_remote(doc, None, None).unwrap();
            check_rebase(&outcome, doc)?;
        }
        for _ in 0..4 {
            let pending = mac.pending_push(500).unwrap();
            if pending.is_empty() {
                return Ok(());
            }
            for doc in pending {
                let current = self.docs.get(&doc.id).cloned();
                let accepted = match (&doc.base_rev, &current) {
                    (None, None) => true,
                    (Some(base), Some(current)) => *base == current.rev,
                    _ => false,
                };
                if accepted {
                    if let Some(current) = &current {
                        prop_assert!(
                            generation(&doc.rev) > generation(&current.rev),
                            "{} replaced {} for {}",
                            doc.rev,
                            current.rev,
                            doc.id
                        );
                    }
                    let mut stored = doc.clone();
                    stored.base_rev = None;
                    self.docs.insert(doc.id.clone(), stored.clone());
                    self.history.push(stored);
                    mac.mark_synced(&doc.id, &doc.rev, None).unwrap();
                } else if let Some(current) = current {
                    let outcome = mac.apply_remote(&current, None, None).unwrap();
                    check_rebase(&outcome, &current)?;
                }
            }
        }
        prop_assert!(
            mac.pending_push(500).unwrap().is_empty(),
            "the push never settled"
        );
        Ok(())
    }
}

/// A local win or a stale replay is rebased above the remote revision.
fn check_rebase(outcome: &ApplyOutcome, remote: &SyncDoc) -> Result<(), TestCaseError> {
    if let ApplyOutcome::LocalWon { new_rev } | ApplyOutcome::Stale { new_rev, .. } = outcome {
        prop_assert!(
            generation(new_rev) > generation(&remote.rev),
            "{new_rev} isn't above {}",
            remote.rev
        );
    }
    Ok(())
}

#[derive(Clone, Debug)]
enum Op {
    Create(usize),
    Rename(usize, usize, u8),
    Delete(usize, usize),
    Sync(usize),
    Replay(usize, usize),
    /// The server claims it lost a document (`Lost`, or left out of a
    /// re-merge when the flag is set), then offers an older version of it.
    LoseThenReplay(usize, usize, bool),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        1 => (0..3usize).prop_map(Op::Create),
        3 => (0..3usize, any::<usize>(), any::<u8>()).prop_map(|(m, k, n)| Op::Rename(m, k, n)),
        1 => (0..3usize, any::<usize>()).prop_map(|(m, k)| Op::Delete(m, k)),
        3 => (0..3usize).prop_map(Op::Sync),
        2 => (0..3usize, any::<usize>()).prop_map(|(m, k)| Op::Replay(m, k)),
        2 => (0..3usize, any::<usize>(), any::<bool>()).prop_map(|(m, k, missing)| Op::LoseThenReplay(m, k, missing)),
    ]
}

proptest! {
    #![proptest_config(config(48))]

    /// Invariants over random three-Mac histories against a server that
    /// accepts a push iff its base is current: every accepted revision has a
    /// higher generation than the one it replaced; a local win or a stale
    /// replay is rebased above the remote revision; a replayed older version
    /// comes back `Stale` without changing the Mac's content; and the Macs
    /// converge. (A Mac's own generation may drop when a remote version
    /// wins: that is the remote's revision, already accepted.)
    #[test]
    fn prop_accepted_generations_only_grow(ops in prop::collection::vec(op(), 1..40)) {
        let macs: Vec<(tempfile::TempDir, Store)> = (0..3).map(|_| temp_store()).collect();
        let mut server = Server::default();
        let mut clock = 1_000_i64;
        for op in ops {
            clock += 1;
            match op {
                Op::Create(m) => {
                    book(&macs[m].1, &format!("b{clock}"));
                }
                Op::Rename(m, k, n) => {
                    let live: Vec<String> = macs[m].1.books().unwrap().into_iter().map(|b| b.id).collect();
                    if !live.is_empty() {
                        rename(&macs[m].1, &live[k % live.len()], &format!("n{n}"), clock);
                    }
                }
                Op::Delete(m, k) => {
                    let live: Vec<String> = macs[m].1.books().unwrap().into_iter().map(|b| b.id).collect();
                    if !live.is_empty() {
                        macs[m].1.delete(&live[k % live.len()], Origin::Ui).unwrap();
                    }
                }
                Op::Sync(m) => server.sync(&macs[m].1)?,
                Op::Replay(m, k) => {
                    let mac = &macs[m].1;
                    server.sync(mac)?;
                    // Older versions of documents this Mac holds, synced.
                    let held = all_docs(mac);
                    let old: Vec<&SyncDoc> = server
                        .history
                        .iter()
                        .filter(|d| {
                            held.contains_key(&d.id)
                                && generation(&d.rev) < generation(&server.docs[&d.id].rev)
                        })
                        .collect();
                    if old.is_empty() {
                        continue;
                    }
                    let replay = old[k % old.len()].clone();
                    let (_, deleted, fields) = held[&replay.id].clone();
                    let outcome = mac.apply_remote(&replay, None, None).unwrap();
                    prop_assert!(matches!(outcome, ApplyOutcome::Stale { .. }), "{outcome:?} for a replay");
                    let (_, deleted_after, fields_after) = all_docs(mac)[&replay.id].clone();
                    prop_assert_eq!(deleted_after, deleted);
                    prop_assert_eq!(fields_after, fields);
                }
                Op::LoseThenReplay(m, k, missing) => {
                    let mac = &macs[m].1;
                    server.sync(mac)?;
                    let held = all_docs(mac);
                    let old: Vec<&SyncDoc> = server
                        .history
                        .iter()
                        .filter(|d| {
                            held.contains_key(&d.id)
                                && generation(&d.rev) < generation(&server.docs[&d.id].rev)
                        })
                        .collect();
                    if old.is_empty() {
                        continue;
                    }
                    let replay = old[k % old.len()].clone();
                    let floor = |mac: &Store| {
                        mac.sync_states().unwrap().into_iter().find(|s| s.id == replay.id).unwrap().synced_gen
                    };
                    let before = floor(mac);
                    if missing {
                        mac.mark_missing_unsynced(&Default::default(), None).unwrap();
                    } else {
                        mac.mark_unsynced(&replay.id, None).unwrap();
                    }
                    prop_assert_eq!(floor(mac), before, "a reported loss lowered the floor");
                    let (_, deleted, fields) = held[&replay.id].clone();
                    let outcome = mac.apply_remote(&replay, None, None).unwrap();
                    prop_assert!(matches!(outcome, ApplyOutcome::Stale { .. }), "{outcome:?} for a replay after a loss");
                    let (_, deleted_after, fields_after) = all_docs(mac)[&replay.id].clone();
                    prop_assert_eq!(deleted_after, deleted, "a loss let a replay change a deletion");
                    prop_assert_eq!(fields_after, fields);
                }
            }
        }
        // Everyone converges after a final round of syncs.
        for _ in 0..2 {
            for (_, mac) in &macs {
                server.sync(mac)?;
            }
        }
        let names = |store: &Store| -> BTreeMap<String, String> {
            store.books().unwrap().into_iter().map(|b| (b.id, b.name)).collect()
        };
        prop_assert_eq!(names(&macs[0].1), names(&macs[1].1));
        prop_assert_eq!(names(&macs[1].1), names(&macs[2].1));
    }
}

/// A local win's new revision is one generation above both sides and keeps
/// the local content hash.
#[test]
fn a_local_win_beats_the_remote_generation() {
    let (_dir, store) = temp_store();
    let id = book(&store, "mine");
    let first = store.pending_push(10).unwrap().remove(0);
    store.mark_synced(&id, &first.rev, None).unwrap();
    rename(&store, &id, "edited", 9_000_000_000_000);
    let local = store.get(&id).unwrap().unwrap().rev;
    let remote = SyncDoc {
        id: id.clone(),
        rev: format!("7-{}", "a".repeat(32)),
        deleted: false,
        fields: json!({ "name": "theirs", "updatedAt": 1 })
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    let ApplyOutcome::LocalWon { new_rev } = store.apply_remote(&remote, None, None).unwrap()
    else {
        panic!("the newer local edit should win");
    };
    assert_eq!(generation(&new_rev), 8);
    assert_eq!(
        new_rev.parse::<Rev>().unwrap().hash,
        local.parse::<Rev>().unwrap().hash
    );
    let pending = store.pending_push(10).unwrap();
    assert_eq!(pending[0].rev, new_rev);
    assert_eq!(pending[0].base_rev.as_deref(), Some(remote.rev.as_str()));
}

/// A replayed older revision or tombstone is `Stale`: the Mac keeps its
/// content, gets a higher revision and pushes again on top of the replay.
#[test]
fn replayed_older_revisions_and_tombstones_are_stale() {
    let (_dir, store) = temp_store();
    let id = book(&store, "v1");
    let v1 = store.pending_push(10).unwrap().remove(0);
    store.mark_synced(&id, &v1.rev, None).unwrap();
    rename(&store, &id, "v2", 2_000);
    let v2 = store.pending_push(10).unwrap().remove(0);
    store.mark_synced(&id, &v2.rev, None).unwrap();

    let replay = SyncDoc {
        base_rev: None,
        ..v1.clone()
    };
    let ApplyOutcome::Stale { new_rev, conflict: false } = store.apply_remote(&replay, None, None).unwrap() else {
        panic!("a replay must be stale");
    };
    assert_eq!(store.get(&id).unwrap().unwrap().fields["name"], "v2");
    assert!(generation(&new_rev) > generation(&v2.rev));
    let pending = store.pending_push(10).unwrap();
    assert_eq!(pending[0].base_rev.as_deref(), Some(v1.rev.as_str()));
    // The replayed version was already in history (this Mac wrote it), so
    // it isn't kept twice and isn't a conflict.
    let copies = |store: &Store, name: &str| {
        store
            .revisions(&id)
            .unwrap()
            .iter()
            .filter(|r| r.fields["name"] == name)
            .count()
    };
    assert_eq!(copies(&store, "v1"), 1);

    // A stale version this Mac never held is kept and flagged: after a
    // restore it can be another Mac's newer edit.
    let unseen = SyncDoc {
        id: id.clone(),
        rev: format!("2-{}", "c".repeat(32)),
        deleted: false,
        fields: json!({ "name": "edited elsewhere", "updatedAt": 9_000 })
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    assert!(matches!(
        store.apply_remote(&unseen, None, None).unwrap(),
        ApplyOutcome::Stale { conflict: true, .. }
    ));
    assert_eq!(store.get(&id).unwrap().unwrap().fields["name"], "v2");
    assert_eq!(copies(&store, "edited elsewhere"), 1);

    let (_dir2, other) = temp_store();
    let id2 = book(&other, "alive");
    let first = other.pending_push(10).unwrap().remove(0);
    other.mark_synced(&id2, &first.rev, None).unwrap();
    rename(&other, &id2, "still alive", 3_000);
    let second = other.pending_push(10).unwrap().remove(0);
    other.mark_synced(&id2, &second.rev, None).unwrap();
    let tombstone = SyncDoc {
        id: id2.clone(),
        rev: format!("2-{}", "b".repeat(32)),
        deleted: true,
        fields: Map::new(),
        base_rev: None,
    };
    assert!(matches!(
        other.apply_remote(&tombstone, None, None).unwrap(),
        ApplyOutcome::Stale { .. }
    ));
    assert_eq!(
        other.get(&id2).unwrap().unwrap().fields["name"],
        "still alive"
    );
}

/// `reset_sync` makes live documents pending with no base and tombstones
/// synced, and writes the meta entries; `replace_library` empties the
/// library in one go, keeps change sequences growing and broadcasts each
/// removal.
#[test]
fn reset_and_replace() {
    let (_dir, store) = temp_store();
    let live = book(&store, "keep");
    let gone = book(&store, "gone");
    for doc in store.pending_push(10).unwrap() {
        store.mark_synced(&doc.id, &doc.rev, None).unwrap();
    }
    store.delete(&gone, Origin::Ui).unwrap();
    store.set_meta("sync.v2.account_id", "a_old").unwrap();
    store
        .reset_sync(&[
            ("sync.v2.account_id", Some("a_new")),
            ("sync.v2.last_seq", None),
        ])
        .unwrap();
    let pending = store.pending_push(10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, live);
    assert_eq!(pending[0].base_rev, None);
    assert_eq!(
        store.meta("sync.v2.account_id").unwrap().as_deref(),
        Some("a_new")
    );

    let guard_pairs = [("sync.v2.account_id", "a_new")];
    let guard = SyncGuard {
        expect: &guard_pairs,
    };
    assert!(store.bump_revision(&live, Some(&guard)).unwrap().is_some());
    let before = store.last_seq().unwrap();
    let changes = store.subscribe();
    let removed = store
        .replace_library(&[
            ("sync.v2.account_id", Some("a_other")),
            ("sync.v2.epoch", None),
        ])
        .unwrap();
    assert_eq!(removed, 2);
    assert!(all_docs(&store).is_empty());
    assert!(store.revisions(&live).unwrap().is_empty());
    assert_eq!(
        store.meta("sync.v2.account_id").unwrap().as_deref(),
        Some("a_other")
    );
    let mut deleted = Vec::new();
    while let Ok(change) = changes.try_recv() {
        assert!(change.deleted);
        assert_eq!(change.origin, Origin::Sync);
        deleted.push(change.id);
    }
    deleted.sort();
    let mut expected = vec![live.clone(), gone.clone()];
    expected.sort();
    assert_eq!(deleted, expected);
    // Sequences keep growing across the replacement.
    assert_eq!(store.last_seq().unwrap(), before);
    book(&store, "after");
    assert!(store.last_seq().unwrap() > before);
    // The guard refuses once the binding changed.
    assert!(store.bump_revision("book:none", Some(&guard)).is_err());
}

proptest! {
    #![proptest_config(config(32))]

    /// Invariant: after `reset_sync`, exactly the live documents are pending,
    /// all with no base revision, whatever was synced before.
    #[test]
    fn prop_reset_sync_pends_exactly_the_live_documents(
        names in prop::collection::vec("[a-z]{1,8}", 1..8),
        synced in prop::collection::vec(any::<bool>(), 8),
        deleted in prop::collection::vec(any::<bool>(), 8),
    ) {
        let (_dir, store) = temp_store();
        let ids: Vec<String> = names.iter().map(|n| book(&store, n)).collect();
        for doc in store.pending_push(100).unwrap() {
            let i = ids.iter().position(|id| *id == doc.id).unwrap();
            if synced[i] {
                store.mark_synced(&doc.id, &doc.rev, None).unwrap();
            }
        }
        for (i, id) in ids.iter().enumerate() {
            if deleted[i] {
                store.delete(id, Origin::Ui).unwrap();
            }
        }
        store.reset_sync(&[]).unwrap();
        let mut pending: Vec<String> = store.pending_push(100).unwrap().into_iter().map(|d| {
            assert!(d.base_rev.is_none());
            d.id
        }).collect();
        pending.sort();
        let mut live: Vec<String> = ids.iter().enumerate().filter(|(i, _)| !deleted[*i]).map(|(_, id)| id.clone()).collect();
        live.sort();
        prop_assert_eq!(pending, live);
    }
}
