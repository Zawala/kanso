//! `kanso-sync verify` and `estimate` (docs/ACCOUNTS.md › Command line): the
//! whole feed opened and compared with the library, the repair of
//! undecryptable server copies, the legacy manifest check (H4), and the
//! charged bytes a library needs. CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_sync::{
    LegacyManifest, LegacyRow, MemoryServer, SyncError, estimate, manifest_sha256, meta_keys,
    verify,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::URL_SAFE_NO_PAD;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A Mac of `alice` with a notebook, notes, a tombstone and a file, synced
/// and caught up with its own pushes; and a second Mac of the account.
fn library() -> (MemoryServer, Mac, Mac) {
    let server = MemoryServer::new();
    let a = Mac::new(sign_up(&server, "alice", 1));
    let b = Mac::new(sign_in(&server, "alice", 2));
    book(&a.store, "book:b1", "Inbox");
    note(&a.store, "note:n1", "book:b1", "one", 100);
    note(&a.store, "note:n2", "book:b1", "two", 100);
    note(&a.store, "note:gone", "book:b1", "gone", 100);
    a.store
        .delete("note:gone", kanso_store::Origin::Api)
        .unwrap();
    file(&a.store, "file:img", b"image bytes");
    a.sync(&server);
    a.sync(&server);
    b.sync(&server);
    (server, a, b)
}

#[test]
fn a_synced_library_verifies() {
    let (server, a, _b) = library();
    let report = verify(&a.store, &server, &a.session, None, false).unwrap();
    assert_eq!(report.matched, 5);
    assert_eq!(
        (
            report.missing,
            report.extra,
            report.different,
            report.undecryptable,
            report.pending
        ),
        (0, 0, 0, 0, 0)
    );
    assert!(report.ok());
    assert_eq!(report.legacy, None);
}

#[test]
fn pending_extra_different_and_missing_documents_are_counted() {
    let (server, a, b) = library();
    // Pending here.
    edit_note(&a.store, "note:n1", "not synced yet", Some(200));
    // On the server but not here yet: an edit and a new note from B.
    edit_note(&b.store, "note:n2", "edited on B", Some(300));
    note(&b.store, "note:extra", "book:b1", "from B", 300);
    b.sync(&server);
    let report = verify(&a.store, &server, &a.session, None, false).unwrap();
    assert_eq!(report.pending, 1);
    assert_eq!(report.extra, 1);
    assert_eq!(report.different, 1);
    assert_eq!(report.matched, 3);
    assert!(!report.ok());

    // The server loses everything after the notebook.
    a.sync(&server);
    a.sync(&server);
    server.rewind("alice", 1);
    let report = verify(&a.store, &server, &a.session, None, false).unwrap();
    assert_eq!(report.matched, 1);
    assert_eq!(report.missing, 5);
    assert!(!report.ok());
}

#[test]
fn file_bytes_must_open_to_the_local_ones() {
    let (server, a, _b) = library();
    let view = server.plain_view(&a.session);
    view.put_file("file:img", b"other bytes").unwrap();
    let report = verify(&a.store, &server, &a.session, None, false).unwrap();
    assert_eq!((report.different, report.matched), (1, 4));
}

#[test]
fn repair_replaces_undecryptable_server_copies_but_never_a_newer_format() {
    let (server, a, _b) = library();
    let view = server.plain_view(&a.session);
    let (broken, newer) = (view.wire_id("note:n1"), view.wire_id("note:n2"));
    let before = a.store.get("note:n1").unwrap().unwrap().rev;
    let damage = || {
        let (broken, newer) = (broken.clone(), newer.clone());
        Tap::new(&server, move |event| {
            if let Event::Page(page) = event {
                for doc in &mut page.docs {
                    let mut raw = B64.decode(&doc.sealed).unwrap();
                    if doc.id == broken {
                        raw[50] ^= 4;
                    } else if doc.id == newer {
                        raw[0] = 7;
                    } else {
                        continue;
                    }
                    doc.sealed = B64.encode(raw);
                }
            }
            Ok(())
        })
    };
    let report = verify(&a.store, &damage(), &a.session, None, false).unwrap();
    assert_eq!(
        (report.undecryptable, report.repaired, report.matched),
        (2, 0, 3)
    );
    assert!(!report.ok());

    let report = verify(&a.store, &damage(), &a.session, None, true).unwrap();
    assert_eq!(
        (report.undecryptable, report.repaired, report.matched),
        (1, 1, 4)
    );
    let after = a.store.get("note:n1").unwrap().unwrap().rev;
    assert_eq!(
        after.split_once('-').unwrap().1,
        before.split_once('-').unwrap().1
    );
    assert!(
        after.parse::<kanso_core::Rev>().unwrap().generation
            > before.parse::<kanso_core::Rev>().unwrap().generation
    );
    assert_eq!(server_docs(&server, &a.session)["note:n1"].0, after);
    assert_eq!(pending(&a.store), 0);
    assert_eq!(body(&a.store, "note:n2").as_deref(), Some("two"));
}

#[test]
fn a_different_key_id_is_refused() {
    let (server, a, _b) = library();
    let tap = Tap::new(&server, |event| {
        if let Event::Page(page) = event {
            page.key_id = "00000000000000aa".into();
        }
        Ok(())
    });
    assert!(matches!(
        verify(&a.store, &tap, &a.session, None, false),
        Err(SyncError::WrongKey)
    ));
}

#[test]
fn a_library_bound_elsewhere_is_refused() {
    let (server, a, _b) = library();
    let bob = sign_up(&server, "bob", 3);
    assert!(matches!(
        verify(&a.store, &server, &bob, None, false),
        Err(SyncError::Cancelled)
    ));
}

/// Files imported from Inkdrop can have no `md5digest` in the manifest;
/// they pass when the library holds bytes that hash to what it recorded
/// (the owner's migration, 2026-10-19: 9 such files failed before).
#[test]
fn a_legacy_file_without_an_md5_passes_on_its_bytes() {
    let (server, a, _b) = library();
    let docs = a.store.sync_docs().unwrap();
    let rev = docs
        .iter()
        .find(|d| d.id == "file:img")
        .unwrap()
        .rev
        .clone();
    let rows = vec![LegacyRow {
        id: "file:img".into(),
        rev,
        deleted: false,
        md5digest: None,
    }];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest.json");
    let manifest = LegacyManifest {
        exported_at: 5,
        rows: rows.clone(),
    };
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let report = verify(&a.store, &server, &a.session, Some(&path), false).unwrap();
    let legacy = report.legacy.unwrap();
    assert!(legacy.ok, "{legacy:?}");

    // Without bytes it still fails.
    let missing = vec![LegacyRow {
        id: "file:none".into(),
        rev: "1-aa".into(),
        deleted: false,
        md5digest: None,
    }];
    let manifest = LegacyManifest {
        exported_at: 5,
        rows: missing,
    };
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let report = verify(&a.store, &server, &a.session, Some(&path), false).unwrap();
    assert_eq!(report.legacy.unwrap().failed, vec!["file:none".to_owned()]);
}

#[test]
fn the_legacy_manifest_is_checked_row_by_row() {
    let (server, a, _b) = library();
    let docs = a.store.sync_docs().unwrap();
    let rev = |id: &str| docs.iter().find(|d| d.id == id).unwrap().rev.clone();
    let md5 = kanso_sync::md5_hex(b"image bytes");
    let mut rows = vec![
        LegacyRow {
            id: "book:b1".into(),
            rev: rev("book:b1"),
            deleted: false,
            md5digest: None,
        },
        LegacyRow {
            id: "note:n1".into(),
            rev: rev("note:n1"),
            deleted: false,
            md5digest: None,
        },
        // An older generation of a document the library has moved past.
        LegacyRow {
            id: "note:n2".into(),
            rev: "0-00".into(),
            deleted: false,
            md5digest: None,
        },
        LegacyRow {
            id: "file:img".into(),
            rev: rev("file:img"),
            deleted: false,
            md5digest: Some(md5.clone()),
        },
        // Deleted rows need nothing.
        LegacyRow {
            id: "note:old".into(),
            rev: "3-cc".into(),
            deleted: true,
            md5digest: None,
        },
    ];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifest.json");
    let write = |rows: &[LegacyRow]| {
        let manifest = LegacyManifest {
            exported_at: 5,
            rows: rows.to_vec(),
        };
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    };
    write(&rows);
    let report = verify(&a.store, &server, &a.session, Some(&path), false).unwrap();
    let legacy = report.legacy.clone().unwrap();
    assert!(legacy.ok, "{legacy:?}");
    assert_eq!(legacy.rows, 5);
    assert_eq!(legacy.manifest_sha256, manifest_sha256(&rows));
    assert_eq!(legacy.account_id, a.session.account_id);
    assert!(report.ok());

    // A live row the library lacks, a file whose bytes differ, and a row
    // whose document has changes not on the server all fail.
    rows.push(LegacyRow {
        id: "note:lost".into(),
        rev: "1-aa".into(),
        deleted: false,
        md5digest: None,
    });
    rows[3].md5digest = Some("0".repeat(32));
    edit_note(&a.store, "note:n1", "pending", Some(500));
    write(&rows);
    let report = verify(&a.store, &server, &a.session, Some(&path), false).unwrap();
    let legacy = report.legacy.unwrap();
    assert!(!legacy.ok);
    assert_eq!(
        legacy.failed,
        vec![
            "file:img".to_owned(),
            "note:lost".to_owned(),
            "note:n1".to_owned()
        ]
    );

    // A manifest that isn't one is an error, not a pass.
    std::fs::write(&path, b"{\"rows\": 3}").unwrap();
    assert!(verify(&a.store, &server, &a.session, Some(&path), false).is_err());
}

proptest! {
    #![proptest_config(config(32))]

    /// Oracle: the estimate is exactly what the server charges once the
    /// library is synced: live documents, sealed length plus 512 bytes per
    /// row (tombstones included), live files and their sealed size plus
    /// 1,024 bytes each.
    #[test]
    fn prop_the_estimate_is_what_the_server_charges(
        notes in prop::collection::vec(("\\PC{0,200}", any::<bool>()), 0..8),
        files in prop::collection::vec((prop::collection::vec(any::<u8>(), 1..300), any::<bool>()), 0..3),
    ) {
        let server = MemoryServer::new();
        let a = Mac::new(sign_up(&server, "alice", 1));
        book(&a.store, "book:b1", "Inbox");
        for (i, (text, delete)) in notes.iter().enumerate() {
            let id = format!("note:n{i}");
            note(&a.store, &id, "book:b1", text, 100);
            if *delete {
                a.store.delete(&id, kanso_store::Origin::Api).unwrap();
            }
        }
        for (i, (bytes, delete)) in files.iter().enumerate() {
            let id = format!("file:f{i}");
            put(&a.store, DocType::File, file_body(&id, "f.png", bytes)).unwrap();
            if *delete {
                a.store.delete(&id, kanso_store::Origin::Api).unwrap();
            }
        }
        merge(&a.store, DocType::Book, json!({"_id": "book:b1", "name": "Renamed"})).unwrap();
        let estimated = estimate(&a.store).unwrap();
        a.sync(&server);
        let usage = kanso_sync::usage(&server, &a.session).unwrap();
        prop_assert_eq!(
            (estimated.docs, estimated.doc_bytes, estimated.files, estimated.file_bytes),
            (usage.docs, usage.doc_bytes, usage.files, usage.file_bytes)
        );
        prop_assert_eq!(a.store.meta(meta_keys::ACCOUNT_ID).unwrap(), Some(a.session.account_id.clone()));
    }
}
