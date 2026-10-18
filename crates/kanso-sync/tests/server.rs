//! `MemoryServer`'s sync routes against a model: accounts are isolated
//! (their own sequence, revisions and files), the accept rule is the
//! Worker's, and the usage counters always equal the recomputed sums
//! (docs/ACCOUNTS.md › POST /v2/push, › Quotas and capacity). The shared
//! fixtures pin the exact answers (tests/conformance.rs). CI pins the seed
//! with `PROPTEST_RNG_SEED`.

use std::collections::BTreeMap;

use kanso_sync::wire::{WireChange, WireDoc, WirePushDoc, WirePushResult, WireRemote};
use kanso_sync::{Backend, DeviceIdentity, MemoryServer, Password, Session};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const SERVER: &str = "http://127.0.0.1:8787";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn account(server: &MemoryServer, n: u8) -> Session {
    let username = format!("user{n}");
    let device = DeviceIdentity::from_parts(&format!("l_{:032x}", n), "Mac", [n; 32]);
    let password = Password::new("correct horse battery", &username).unwrap();
    kanso_sync::sign_up(
        server,
        &kanso_sync::MemorySecrets::new(),
        SERVER,
        &username,
        &password,
        &device,
        &|_| {},
    )
    .unwrap()
    .session
}

const IDS: [&str; 4] = [
    "note:AAAAAAAAAAAAAAAAAAAAAA",
    "note:BBBBBBBBBBBBBBBBBBBBBB",
    "book:CCCCCCCCCCCCCCCCCCCCCC",
    "file:DDDDDDDDDDDDDDDDDDDDDD",
];

#[derive(Clone, Debug)]
enum Base {
    Current,
    Null,
    Stale,
}

#[derive(Clone, Debug)]
enum Op {
    Push(usize, Vec<(usize, Base, bool, String)>),
    PutFile(usize, usize),
}

fn op() -> impl Strategy<Value = Op> {
    let base = prop_oneof![3 => Just(Base::Current), 1 => Just(Base::Null), 1 => Just(Base::Stale)];
    let doc = (0..IDS.len(), base, any::<bool>(), "[A-Za-z0-9_-]{55,120}");
    prop_oneof![
        3 => (0..3usize, prop::collection::vec(doc, 1..5)).prop_map(|(a, docs)| Op::Push(a, docs)),
        1 => (0..3usize, 41..300usize).prop_map(|(a, size)| Op::PutFile(a, size)),
    ]
}

#[derive(Default, Clone)]
struct Model {
    docs: BTreeMap<String, WireChange>,
    files: BTreeMap<String, Vec<u8>>,
    last_seq: i64,
}

proptest! {
    #![proptest_config(config(64))]

    /// Model-based: three accounts push and upload in random interleavings.
    /// Each push result follows the accept rule against that account's own
    /// documents; each feed holds exactly that account's documents in `seq`
    /// order; file bytes are only readable by their account and an accepted
    /// file tombstone drops them; usage equals the recomputed sums.
    #[test]
    fn prop_accounts_are_isolated(ops in prop::collection::vec(op(), 1..30)) {
        let server = MemoryServer::new();
        let sessions: Vec<Session> = (1..=3).map(|n| account(&server, n)).collect();
        let remotes: Vec<Box<dyn WireRemote + '_>> = sessions.iter().map(|s| server.remote(s)).collect();
        let mut models = vec![Model::default(); 3];
        let mut counter = 0u64;
        for op in ops {
            match op {
                Op::Push(a, docs) => {
                    let model = &mut models[a];
                    let mut expected = Vec::new();
                    let mut request = Vec::new();
                    let mut next = model.clone();
                    for (i, base, deleted, sealed) in docs {
                        counter += 1;
                        let id = IDS[i].to_owned();
                        let current = next.docs.get(&id).cloned();
                        let base_rev = match base {
                            Base::Current => current.as_ref().map(|c| c.rev.clone()),
                            Base::Null => None,
                            Base::Stale => Some(format!("{counter:032x}")),
                        };
                        let rev = format!("{:032x}", counter + (1 << 40));
                        let accepted = match (&base_rev, &current) {
                            (None, None) => true,
                            (Some(b), Some(c)) => *b == c.rev,
                            _ => false,
                        };
                        expected.push(if accepted {
                            next.last_seq += 1;
                            next.docs.insert(id.clone(), WireChange { id: id.clone(), rev: rev.clone(), deleted, sealed: sealed.clone(), seq: next.last_seq });
                            WirePushResult::Ok { id: id.clone(), seq: next.last_seq }
                        } else {
                            match current {
                                Some(c) => WirePushResult::Conflict {
                                    id: id.clone(),
                                    current: WireDoc { id: c.id, rev: c.rev, deleted: c.deleted, sealed: c.sealed },
                                },
                                None => WirePushResult::Lost { id: id.clone() },
                            }
                        });
                        request.push(WirePushDoc { id, rev, deleted, sealed, base_rev });
                    }
                    // A file tombstone accepted last in the push drops the bytes.
                    let file = IDS[3];
                    if let Some(last) = request.iter().zip(&expected).rev().find(|(d, r)| d.id == file && matches!(r, WirePushResult::Ok { .. })) && last.0.deleted {
                        next.files.remove(file);
                    }
                    let results = remotes[a].push(&request).unwrap();
                    prop_assert_eq!(results, expected);
                    *model = next;
                }
                Op::PutFile(a, size) => {
                    let bytes: Vec<u8> = (0..size).map(|i| (i * 7 + a) as u8).collect();
                    remotes[a].put_file(IDS[3], &bytes).unwrap();
                    models[a].files.insert(IDS[3].to_owned(), bytes);
                }
            }
            for (a, model) in models.iter().enumerate() {
                let page = remotes[a].changes(0, 500).unwrap();
                let mut want: Vec<WireChange> = model.docs.values().cloned().collect();
                want.sort_by_key(|d| d.seq);
                prop_assert_eq!(&page.docs, &want);
                prop_assert_eq!(page.head_seq, model.last_seq);
                prop_assert_eq!(remotes[a].get_file(IDS[3]).unwrap(), model.files.get(IDS[3]).cloned());
                let usage = kanso_sync::usage(&server, &sessions[a]).unwrap();
                prop_assert_eq!(usage.docs, model.docs.values().filter(|d| !d.deleted).count() as u64);
                prop_assert_eq!(usage.doc_bytes, model.docs.values().map(|d| d.sealed.len() as u64 + 512).sum::<u64>());
                prop_assert_eq!(usage.files, model.files.len() as u64);
                prop_assert_eq!(usage.file_bytes, model.files.values().map(|f| f.len() as u64 + 1024).sum::<u64>());
            }
        }
    }
}
