//! Property tests for sync. CI pins the seed with `PROPTEST_RNG_SEED`.
//!
//! - prop_stores_converge_and_no_edit_is_lost (model-based): three stores and
//!   one server under random interleavings of local writes and syncs.
//! - prop_syncing_a_synced_store_changes_nothing (idempotence).
//! - prop_memory_remote_follows_the_acceptance_model (model-based), the same
//!   property the Worker's tests check (cloud/test/properties.test.ts).
//! - prop_batches_cover_the_input_greedily_within_limits (invariant).

mod common;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use common::*;
use kanso_core::DocType;
use kanso_store::{Change, Origin, Store, StoreError, SyncDoc};
use kanso_sync::{
    ChangesPage, MemoryRemote, PushResult, Remote, RemoteError, RemoteFile, SyncReport, batches,
    sync_once,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

const WORDS: [&str; 6] = [
    "alpha",
    "Beta",
    "über",
    "x%y",
    "- [ ] task",
    "inkdrop://note/x",
];
const NAMES: [&str; 4] = ["Inbox", "Work", "Ideas", "Archive"];

/// One step of a scenario. `s` picks a store; `pick` picks one of that store's
/// live documents of the kind (modulo their count), so steps stay meaningful
/// as the scenario shrinks.
#[derive(Clone, Debug)]
enum Op {
    CreateBook {
        s: usize,
    },
    RenameBook {
        s: usize,
        pick: usize,
        name: usize,
    },
    DeleteBook {
        s: usize,
        pick: usize,
    },
    CreateTag {
        s: usize,
    },
    DeleteTag {
        s: usize,
        pick: usize,
    },
    CreateNote {
        s: usize,
        book: usize,
        tag: Option<usize>,
        word: usize,
    },
    /// `bump: false` keeps `updatedAt`, so concurrent edits tie.
    EditNote {
        s: usize,
        pick: usize,
        word: usize,
        bump: bool,
    },
    TagNote {
        s: usize,
        pick: usize,
        tag: usize,
    },
    DeleteNote {
        s: usize,
        pick: usize,
    },
    CreateFile {
        s: usize,
        bytes: Vec<u8>,
    },
    RenameFile {
        s: usize,
        pick: usize,
        name: usize,
    },
    ReplaceFile {
        s: usize,
        pick: usize,
        bytes: Vec<u8>,
    },
    DeleteFile {
        s: usize,
        pick: usize,
    },
    Sync {
        s: usize,
    },
}

fn gen_op(stores: usize) -> impl Strategy<Value = Op> {
    let s = 0..stores;
    let pick = 0..6usize;
    // A tiny alphabet so two devices sometimes produce identical bytes.
    let bytes = prop::collection::vec(0u8..3, 0..5);
    prop_oneof![
        2 => s.clone().prop_map(|s| Op::CreateBook { s }),
        1 => (s.clone(), pick.clone(), 0..NAMES.len()).prop_map(|(s, pick, name)| Op::RenameBook { s, pick, name }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteBook { s, pick }),
        1 => s.clone().prop_map(|s| Op::CreateTag { s }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteTag { s, pick }),
        4 => (s.clone(), pick.clone(), prop::option::of(pick.clone()), 0..WORDS.len())
            .prop_map(|(s, book, tag, word)| Op::CreateNote { s, book, tag, word }),
        5 => (s.clone(), pick.clone(), 0..WORDS.len(), any::<bool>())
            .prop_map(|(s, pick, word, bump)| Op::EditNote { s, pick, word, bump }),
        1 => (s.clone(), pick.clone(), pick.clone()).prop_map(|(s, pick, tag)| Op::TagNote { s, pick, tag }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteNote { s, pick }),
        2 => (s.clone(), bytes.clone()).prop_map(|(s, bytes)| Op::CreateFile { s, bytes }),
        1 => (s.clone(), pick.clone(), 0..NAMES.len()).prop_map(|(s, pick, name)| Op::RenameFile { s, pick, name }),
        1 => (s.clone(), pick.clone(), bytes).prop_map(|(s, pick, bytes)| Op::ReplaceFile { s, pick, bytes }),
        1 => (s.clone(), pick.clone()).prop_map(|(s, pick)| Op::DeleteFile { s, pick }),
        6 => s.prop_map(|s| Op::Sync { s }),
    ]
}

/// A local write a store made: which store, document, revision, tombstone.
type Version = (usize, String, String, bool);

struct World {
    _dirs: Vec<tempfile::TempDir>,
    stores: Vec<Store>,
    feeds: Vec<async_channel::Receiver<Change>>,
    remote: MemoryRemote,
    next_id: usize,
    clock: i64,
    local: Vec<Version>,
    /// Per store, every version the server handed it.
    seen: Vec<Vec<Seen>>,
}

fn live_ids(store: &Store, prefix: &str) -> Vec<String> {
    all_docs(store)
        .into_iter()
        .filter(|(id, (_, deleted))| !deleted && id.starts_with(prefix))
        .map(|(id, _)| id)
        .collect()
}

fn choose(ids: &[String], pick: usize) -> Option<String> {
    (!ids.is_empty()).then(|| ids[pick % ids.len()].clone())
}

/// A document version: id, revision, tombstone.
type Seen = (String, String, bool);

/// Passes calls through to the server, noting every version it hands out
/// (change-feed documents and the server's side of conflicts).
struct Recording<'a> {
    inner: &'a MemoryRemote,
    seen: RefCell<Vec<Seen>>,
}

impl Remote for Recording<'_> {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let results = self.inner.push(docs)?;
        for result in &results {
            if let PushResult::Conflict {
                current: Some(c), ..
            } = result
            {
                self.seen
                    .borrow_mut()
                    .push((c.id.clone(), c.rev.clone(), c.deleted));
            }
        }
        Ok(results)
    }
    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        let page = self.inner.changes(since, limit)?;
        for d in &page.docs {
            self.seen
                .borrow_mut()
                .push((d.id.clone(), d.rev.clone(), d.deleted));
        }
        Ok(page)
    }
    fn put_file(&self, id: &str, md5: &str, ct: &str, bytes: &[u8]) -> Result<(), RemoteError> {
        self.inner.put_file(id, md5, ct, bytes)
    }
    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        self.inner.get_file(id)
    }
}

/// Syncs a store, adding what the server handed it to `seen`. A sync must
/// never fail or leave anything it could not handle.
fn sync_cleanly(store: &Store, remote: &MemoryRemote, seen: &mut Vec<Seen>) -> SyncReport {
    let recording = Recording {
        inner: remote,
        seen: RefCell::default(),
    };
    let report = sync_once(store, &recording).expect("sync");
    seen.extend(recording.seen.into_inner());
    assert!(report.invalid.is_empty(), "invalid: {:?}", report.invalid);
    assert!(report.too_large.is_empty());
    assert_eq!(report.files_missing, 0, "a live file without bytes");
    report
}

impl World {
    fn new(stores: usize) -> World {
        let mut dirs = Vec::new();
        let mut opened = Vec::new();
        let mut feeds = Vec::new();
        for _ in 0..stores {
            let (dir, store) = temp_store();
            feeds.push(store.subscribe());
            dirs.push(dir);
            opened.push(store);
        }
        World {
            _dirs: dirs,
            stores: opened,
            feeds,
            remote: MemoryRemote::new(),
            next_id: 0,
            clock: 0,
            local: Vec::new(),
            seen: vec![Vec::new(); stores],
        }
    }

    /// A world whose stores already share a few documents, so random edits
    /// on different stores often hit the same document.
    fn seeded(stores: usize) -> World {
        let mut world = World::new(stores);
        let first = &world.stores[0];
        put(
            first,
            DocType::Book,
            json!({"_id": "book:base1", "name": "Inbox", "createdAt": 1, "updatedAt": 1}),
        )
        .unwrap();
        put(first, DocType::Book, json!({"_id": "book:base2", "name": "Work", "parentBookId": "book:base1", "createdAt": 1, "updatedAt": 1})).unwrap();
        put(
            first,
            DocType::Tag,
            json!({"_id": "tag:base1", "name": "red", "createdAt": 1, "updatedAt": 1}),
        )
        .unwrap();
        for (i, book) in ["book:base1", "book:base2", "book:base1"]
            .iter()
            .enumerate()
        {
            put(
                first,
                DocType::Note,
                json!({"_id": format!("note:base{i}"), "bookId": book, "tags": ["tag:base1"], "title": "base",
                       "body": "- [x] seeded", "createdAt": 1, "updatedAt": 1}),
            )
            .unwrap();
        }
        put(
            first,
            DocType::File,
            file_body("file:base1", "base.png", &[1, 2, 3]),
        )
        .unwrap();
        world.run(&Op::Sync { s: 0 });
        for s in 1..stores {
            world.run(&Op::Sync { s });
        }
        world
    }

    fn fresh(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }

    fn run(&mut self, op: &Op) {
        self.clock += 1;
        let now = 1_000_000 + self.clock;
        let result = self.write(op, now);
        match result {
            Ok(()) | Err(StoreError::Invalid(_) | StoreError::Conflict | StoreError::NotFound) => {}
            Err(err) => panic!("{op:?}: {err}"),
        }
        for (s, feed) in self.feeds.iter().enumerate() {
            while let Ok(change) = feed.try_recv() {
                if change.origin != Origin::Sync {
                    self.local.push((s, change.id, change.rev, change.deleted));
                }
            }
        }
    }

    fn write(&mut self, op: &Op, now: i64) -> Result<(), StoreError> {
        let id = |world: &World, s: usize, prefix: &str, pick: usize| {
            choose(&live_ids(&world.stores[s], prefix), pick)
        };
        match op.clone() {
            Op::CreateBook { s } => {
                let id = self.fresh("book:b");
                let name = NAMES[self.next_id % NAMES.len()];
                put(
                    &self.stores[s],
                    DocType::Book,
                    json!({"_id": id, "name": name, "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::RenameBook { s, pick, name } => {
                if let Some(id) = id(self, s, "book:", pick) {
                    merge(
                        &self.stores[s],
                        DocType::Book,
                        json!({"_id": id, "name": NAMES[name], "updatedAt": now}),
                    )?;
                }
            }
            Op::DeleteBook { s, pick }
            | Op::DeleteTag { s, pick }
            | Op::DeleteNote { s, pick }
            | Op::DeleteFile { s, pick } => {
                let prefix = match op {
                    Op::DeleteBook { .. } => "book:",
                    Op::DeleteTag { .. } => "tag:",
                    Op::DeleteNote { .. } => "note:",
                    _ => "file:",
                };
                if let Some(id) = id(self, s, prefix, pick) {
                    self.stores[s].delete(&id, Origin::Api)?;
                }
            }
            Op::CreateTag { s } => {
                let id = self.fresh("tag:t");
                let name = format!("tag{}", self.next_id);
                put(
                    &self.stores[s],
                    DocType::Tag,
                    json!({"_id": id, "name": name, "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::CreateNote { s, book, tag, word } => {
                let id = self.fresh("note:n");
                let book = id_or_trash(choose(&live_ids(&self.stores[s], "book:"), book));
                let tags: Vec<String> = tag
                    .and_then(|t| choose(&live_ids(&self.stores[s], "tag:"), t))
                    .into_iter()
                    .collect();
                put(
                    &self.stores[s],
                    DocType::Note,
                    json!({"_id": id, "bookId": book, "tags": tags, "title": WORDS[word],
                           "body": format!("{} {now}", WORDS[word]), "createdAt": now, "updatedAt": now}),
                )?;
            }
            Op::EditNote {
                s,
                pick,
                word,
                bump,
            } => {
                if let Some(id) = id(self, s, "note:", pick) {
                    let mut doc = self.stores[s].get_api(&id, None, false)?;
                    doc["body"] = format!("{} {now}", WORDS[word]).into();
                    if bump {
                        doc["updatedAt"] = now.into();
                    }
                    put(&self.stores[s], DocType::Note, doc)?;
                }
            }
            Op::TagNote { s, pick, tag } => {
                let note = id(self, s, "note:", pick);
                let tag = id(self, s, "tag:", tag);
                if let (Some(note), Some(tag)) = (note, tag) {
                    merge(
                        &self.stores[s],
                        DocType::Note,
                        json!({"_id": note, "tags": [tag]}),
                    )?;
                }
            }
            Op::CreateFile { s, bytes } => {
                let id = self.fresh("file:f");
                put(
                    &self.stores[s],
                    DocType::File,
                    file_body(&id, "f.png", &bytes),
                )?;
            }
            Op::RenameFile { s, pick, name } => {
                if let Some(id) = id(self, s, "file:", pick) {
                    merge(
                        &self.stores[s],
                        DocType::File,
                        json!({"_id": id, "name": format!("{}.png", NAMES[name])}),
                    )?;
                }
            }
            Op::ReplaceFile { s, pick, bytes } => {
                if let Some(id) = id(self, s, "file:", pick) {
                    let mut doc = self.stores[s].get_api(&id, None, false)?;
                    doc["_attachments"] = file_body(&id, "f.png", &bytes)["_attachments"].clone();
                    put(&self.stores[s], DocType::File, doc)?;
                }
            }
            Op::Sync { s } => {
                sync_cleanly(&self.stores[s], &self.remote, &mut self.seen[s]);
            }
        }
        Ok(())
    }
}

fn id_or_trash(id: Option<String>) -> String {
    id.unwrap_or_else(|| "trash".to_owned())
}

fn server_docs(remote: &MemoryRemote) -> BTreeMap<String, (String, bool)> {
    remote
        .docs()
        .into_iter()
        .map(|d| (d.id, (d.rev, d.deleted)))
        .collect()
}

proptest! {
    #![proptest_config(config(128))]

    #[test]
    fn prop_stores_converge_and_no_edit_is_lost(ops in prop::collection::vec(gen_op(3), 0..40)) {
        let mut world = World::seeded(3);
        for op in &ops {
            world.run(op);
        }
        // Final round: every store syncs twice.
        for _ in 0..2 {
            for s in 0..world.stores.len() {
                world.run(&Op::Sync { s });
            }
        }
        let stores = &world.stores;
        let server = server_docs(&world.remote);

        for (s, store) in stores.iter().enumerate() {
            prop_assert_eq!(pending(store), 0, "store {} has pending pushes", s);
            // Same documents, revisions and tombstones as the server.
            prop_assert_eq!(&all_docs(store), &server, "store {} differs from the server", s);
        }
        // Same fields everywhere, equal to what the server holds.
        let server_fields: BTreeMap<String, _> = world
            .remote
            .docs()
            .into_iter()
            .filter(|d| !d.deleted)
            .map(|d| (d.id, (d.rev, d.fields)))
            .collect();
        for store in stores {
            prop_assert_eq!(&live_docs(store), &server_fields);
        }
        // Same attachment bytes everywhere, and on the server.
        for id in server_fields.keys().filter(|id| id.starts_with("file:")) {
            let first = stores[0].attachment(id).unwrap();
            prop_assert!(first.is_some(), "{} has no bytes", id);
            let (content_type, blob) = first.unwrap();
            for store in &stores[1..] {
                let other = store.attachment(id).unwrap().unwrap();
                prop_assert_eq!(&other.0, &content_type);
                prop_assert_eq!(&other.1.data, &blob.data);
            }
            prop_assert_eq!(world.remote.file(id).map(|f| f.md5_hex), Some(blob.md5_hex()));
        }
        for (id, (_, deleted)) in &server {
            if *deleted && id.starts_with("file:") {
                prop_assert!(world.remote.file(id).is_none(), "{} kept its bytes", id);
            }
        }
        // No edit is lost: every version a store wrote or was handed by the
        // server (including the losing side of each conflict it resolved) is
        // still its current version or in its revision history.
        for (s, store) in stores.iter().enumerate() {
            let wrote = world
                .local
                .iter()
                .filter(|v| v.0 == s)
                .map(|(_, id, rev, deleted)| (id, rev, *deleted));
            let received = world.seen[s].iter().map(|(id, rev, deleted)| (id, rev, *deleted));
            for (id, rev, deleted) in wrote.chain(received) {
                if deleted {
                    continue;
                }
                let current = store.get(id).unwrap().map(|d| d.rev);
                prop_assert!(
                    current.as_deref() == Some(rev.as_str()) || has_revision(store, id, rev),
                    "store {} lost {} {}", s, id, rev
                );
            }
        }
        // Sync never invents revisions: each final one was written by a store.
        let written: BTreeSet<(&str, &str)> =
            world.local.iter().map(|(_, id, rev, _)| (id.as_str(), rev.as_str())).collect();
        for (id, (rev, _)) in &server {
            prop_assert!(written.contains(&(id.as_str(), rev.as_str())), "{} {} came from nowhere", id, rev);
        }
        // A document only one store ever wrote ends at that store's last
        // version: with no concurrent edit there is nothing to lose to.
        let mut writers: BTreeMap<&str, (BTreeSet<usize>, &str)> = BTreeMap::new();
        for (s, id, rev, _) in &world.local {
            let entry = writers.entry(id.as_str()).or_insert_with(|| (BTreeSet::new(), rev.as_str()));
            entry.0.insert(*s);
            entry.1 = rev.as_str();
        }
        for (id, (stores_that_wrote, last)) in &writers {
            if stores_that_wrote.len() == 1 {
                prop_assert_eq!(server[*id].0.as_str(), *last, "single-writer {} was reverted", id);
            }
        }
        // And one more sync on a converged store does nothing.
        let (seq, pushes) = (world.remote.last_seq(), world.remote.stats().pushes);
        for store in stores {
            prop_assert_eq!(sync_once(store, &world.remote).unwrap(), SyncReport::default());
        }
        prop_assert_eq!((world.remote.last_seq(), world.remote.stats().pushes), (seq, pushes));
    }

    #[test]
    fn prop_syncing_a_synced_store_changes_nothing(ops in prop::collection::vec(gen_op(1), 0..20)) {
        let mut world = World::new(1);
        for op in &ops {
            world.run(op);
        }
        let store = &world.stores[0];
        sync_cleanly(store, &world.remote, &mut Vec::new());
        prop_assert_eq!(pending(store), 0);
        let before = (all_docs(store), live_docs(store), server_docs(&world.remote), world.remote.stats());
        prop_assert_eq!(sync_once(store, &world.remote).unwrap(), SyncReport::default());
        let after = (all_docs(store), live_docs(store), server_docs(&world.remote), world.remote.stats());
        prop_assert_eq!(&after.0, &before.0);
        prop_assert_eq!(&after.1, &before.1);
        prop_assert_eq!(&after.2, &before.2);
        prop_assert_eq!(after.3.pushes, before.3.pushes);
        prop_assert_eq!(after.3.file_puts, before.3.file_puts);
        prop_assert_eq!(after.3.file_gets, before.3.file_gets);
    }
}

/// How a generated push picks its base revision.
#[derive(Clone, Copy, Debug)]
enum Base {
    Current,
    Null,
    Stale,
}

proptest! {
    #![proptest_config(config(256))]

    /// Model: a map of current revisions and one counter. A document is
    /// accepted iff its base is the current revision (or both are absent);
    /// each accepted write takes the next seq; the feed lists every document
    /// once, at its latest seq.
    #[test]
    fn prop_memory_remote_follows_the_acceptance_model(
        pushes in prop::collection::vec(
            prop::collection::vec(
                (0..3usize, prop_oneof![2 => Just(Base::Current), 1 => Just(Base::Null), 1 => Just(Base::Stale)], any::<bool>()),
                0..4,
            ),
            1..8,
        )
    ) {
        const IDS: [&str; 3] = ["note:a", "note:b", "file:c"];
        let remote = MemoryRemote::new();
        let mut model: BTreeMap<String, (String, bool, i64)> = BTreeMap::new();
        let mut counter = 0;
        let mut revs = 0;
        for batch in &pushes {
            let sent: Vec<SyncDoc> = batch
                .iter()
                .map(|(i, base, deleted)| {
                    revs += 1;
                    let known = model.get(IDS[*i]).map(|m| m.0.clone());
                    SyncDoc {
                        id: IDS[*i].into(),
                        rev: format!("{revs}-{revs:x}"),
                        deleted: *deleted,
                        fields: Default::default(),
                        base_rev: match base {
                            Base::Current => known,
                            Base::Null => None,
                            Base::Stale => Some("1-0".into()),
                        },
                    }
                })
                .collect();
            let results = remote.push(&sent).unwrap();
            for (doc, result) in sent.iter().zip(results) {
                let current = model.get(&doc.id).cloned();
                let accepted = match &current {
                    None => doc.base_rev.is_none(),
                    Some((rev, ..)) => doc.base_rev.as_deref() == Some(rev.as_str()),
                };
                if accepted {
                    counter += 1;
                    model.insert(doc.id.clone(), (doc.rev.clone(), doc.deleted, counter));
                    prop_assert_eq!(result, PushResult::Ok { id: doc.id.clone(), seq: counter });
                } else {
                    let PushResult::Conflict { current: got, .. } = result else {
                        return Err(TestCaseError::fail(format!("{} should conflict", doc.id)));
                    };
                    prop_assert_eq!(got.map(|c| (c.rev, c.deleted)), current.map(|c| (c.0, c.1)));
                }
            }
        }
        let feed = remote.changes(0, 500).unwrap();
        let mut expected: Vec<(String, String, bool, i64)> =
            model.into_iter().map(|(id, (rev, deleted, seq))| (id, rev, deleted, seq)).collect();
        expected.sort_by_key(|e| e.3);
        let got: Vec<(String, String, bool, i64)> =
            feed.docs.into_iter().map(|d| (d.id, d.rev, d.deleted, d.seq)).collect();
        prop_assert_eq!(feed.last_seq, expected.last().map_or(0, |e| e.3));
        prop_assert_eq!(got, expected);
    }

    #[test]
    fn prop_batches_cover_the_input_greedily_within_limits(
        sizes in prop::collection::vec(0usize..5_000, 0..60),
        max_docs in 1usize..10,
        max_bytes in 1usize..12_000,
    ) {
        let ranges = batches(&sizes, max_docs, max_bytes);
        let mut next = 0;
        for (i, range) in ranges.iter().enumerate() {
            prop_assert_eq!(range.start, next, "batches are consecutive");
            prop_assert!(!range.is_empty());
            prop_assert!(range.len() <= max_docs);
            let bytes: usize = sizes[range.clone()].iter().sum();
            prop_assert!(bytes <= max_bytes || range.len() == 1, "over the byte budget");
            if i + 1 < ranges.len() {
                // Greedy: the next item would not have fitted.
                prop_assert!(range.len() == max_docs || bytes + sizes[range.end] > max_bytes);
            }
            next = range.end;
        }
        prop_assert_eq!(next, sizes.len(), "every item is in a batch");
    }
}

#[test]
fn values_round_trip_through_the_wire_shape() {
    let result = PushResult::Conflict {
        id: "note:a".into(),
        current: Some(SyncDoc {
            id: "note:a".into(),
            rev: "2-ab".into(),
            deleted: false,
            fields: json!({"body": "ü", "updatedAt": 1_700_000_000_000_i64})
                .as_object()
                .unwrap()
                .clone(),
            base_rev: None,
        }),
    };
    let text = serde_json::to_string(&result).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["status"], "conflict");
    assert_eq!(serde_json::from_str::<PushResult>(&text).unwrap(), result);
}
