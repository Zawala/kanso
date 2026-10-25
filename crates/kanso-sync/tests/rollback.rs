//! Rollback (docs/ACCOUNTS.md › Rollback, fix 1 / H5): a library synced
//! under v1 rules, then v2 with edits, creations and deletions, then
//! `unbind`, then v1 rules again. Every v2-window edit and creation reaches
//! the v1 server; v2-window deletions come back, as the rollback steps say.
//! The old app and the old Worker are stand-ins (tests/common/legacy.rs),
//! and the old Worker's stand-in is checked against its own shared cases.
// no-prop-test for the v1 cases: fixed cases shared with the v1 Worker.

mod common;

use std::collections::BTreeMap;

use base64::Engine as _;
use common::legacy::{LegacyPage, LegacyServer, legacy_sync};
use common::*;
use kanso_store::{Origin, Store, SyncDoc};
use kanso_sync::{
    Choice, Flow, LocalLibrary, MemorySecrets, MemoryServer, Plan, PushResult, RemoteError,
    local_library, meta_keys, plan,
};
use serde_json::Value;

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

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn v1_fixture() -> Value {
    let path = manifest_dir().join("../../cloud/test/fixtures/conformance-v1.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("read the fixture"))
        .expect("parse the fixture")
}

fn expected_status(step: &Value) -> u16 {
    step.get("status").and_then(Value::as_u64).unwrap_or(200) as u16
}

fn status_of<T>(result: &Result<T, RemoteError>) -> u16 {
    match result {
        Ok(_) => 200,
        Err(err) => err
            .status()
            .unwrap_or_else(|| panic!("expected an HTTP status, got {err}")),
    }
}

fn run_step(remote: &LegacyServer, case: &str, index: usize, step: &Value) {
    let at = format!("{case} / step {index}");
    if let Some(docs) = step.get("push") {
        let docs: Vec<SyncDoc> = match serde_json::from_value(docs.clone()) {
            Ok(docs) => docs,
            Err(_) => {
                // A shape the typed client cannot even express.
                assert_eq!(expected_status(step), 400, "{at}: unparseable push");
                return;
            }
        };
        let result = remote.push(&docs);
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
        if let Ok(results) = result {
            let expected: Vec<PushResult> =
                serde_json::from_value(step["results"].clone()).expect("results");
            assert_eq!(results, expected, "{at}: results");
        }
    } else if let Some(query) = step.get("changes") {
        let since = query["since"].as_i64().unwrap();
        let limit = query["limit"].as_u64().unwrap() as usize;
        let result = remote.changes(since, limit);
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
        if let Ok(page) = result {
            let expected: LegacyPage = serde_json::from_value(step["page"].clone()).expect("page");
            assert_eq!(page, expected, "{at}: page");
        }
    } else if let Some(file) = step.get("put_file") {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(file["base64"].as_str().unwrap())
            .unwrap();
        let result = remote.put_file(
            file["id"].as_str().unwrap(),
            file["md5"].as_str().unwrap(),
            "application/octet-stream",
            &bytes,
        );
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
    } else if let Some(id) = step.get("get_file") {
        let result = remote.get_file(id.as_str().unwrap()).map(|f| f.ok_or(()));
        let file = match result {
            Ok(Ok(file)) => file,
            Ok(Err(())) => {
                assert_eq!(expected_status(step), 404, "{at}: missing file");
                return;
            }
            Err(err) => panic!("{at}: {err}"),
        };
        assert_eq!(expected_status(step), 200, "{at}: status");
        assert_eq!(file.md5_hex, step["md5"].as_str().unwrap(), "{at}: md5");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(step["base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(file.bytes, bytes, "{at}: bytes");
    } else if step.get("push_body").is_some() || step.get("changes_query").is_some() {
        // Raw HTTP input; only the Worker's tests send it.
    } else {
        panic!("{at}: unknown step {step}");
    }
}

/// The stand-in for the old Worker follows the v1 Worker's own cases, so the
/// rollback test replays what the old Worker really did.
#[test]
fn the_legacy_server_follows_the_v1_worker_cases() {
    let fixture = v1_fixture();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 10);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let remote = LegacyServer::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            run_step(&remote, name, index, step);
        }
    }
}
