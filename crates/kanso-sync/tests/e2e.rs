//! End-to-end check against a deployed Worker. Runs only when
//! `KANSO_SYNC_E2E_ENDPOINT` is set; the token comes from `KANSO_SYNC_TOKEN`
//! or the Keychain (a test binary reading the Keychain makes macOS ask once).
//! It creates documents with unique IDs and deletes them again; the server
//! keeps four tombstones per run.
//!
//! ```text
//! KANSO_SYNC_E2E_ENDPOINT=https://… cargo test -p kanso-sync --test e2e -- --nocapture
//! ```

mod common;

use common::*;
use kanso_store::Origin;
use kanso_sync::{HttpRemote, Remote, credentials, md5_hex, sync_once};

/// A 1×1 transparent PNG.
const PNG: [u8; 67] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

#[test]
fn two_devices_sync_through_the_deployed_worker() {
    let Ok(endpoint) = std::env::var("KANSO_SYNC_E2E_ENDPOINT") else {
        eprintln!("skipped: set KANSO_SYNC_E2E_ENDPOINT to run against a deployed Worker");
        return;
    };
    let token = match std::env::var("KANSO_SYNC_TOKEN") {
        Ok(token) if !token.is_empty() => token,
        _ => credentials::load_token()
            .expect("keychain")
            .expect("no token in KANSO_SYNC_TOKEN or the Keychain"),
    };
    let remote = HttpRemote::new(endpoint, token);
    remote.health().expect("health");
    assert!(matches!(
        HttpRemote::new(remote.endpoint(), "wrong").changes(0, 1),
        Err(kanso_sync::RemoteError::Unauthorized)
    ));

    let run = format!("{:x}", kanso_core::now_ms());
    let (book_id, note_id) = (format!("book:e2e{run}"), format!("note:e2e{run}"));
    let (png_id, big_id) = (format!("file:e2e{run}a"), format!("file:e2e{run}b"));
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    let now = kanso_core::now_ms();

    // 1. A note created on A reaches B.
    book(&a, &book_id, "E2E");
    let created = note(&a, &note_id, &book_id, "created on A", now);
    let report = sync_once(&a, &remote).expect("sync A");
    assert_eq!(report.pushed, 2);
    let report = sync_once(&b, &remote).expect("sync B");
    eprintln!(
        "B's first pull: {} documents (whole server history)",
        report.pulled
    );
    assert_eq!(body(&b, &note_id).as_deref(), Some("created on A"));
    assert_eq!(b.get(&note_id).unwrap().unwrap().rev, created.rev);

    // 2. Both edit; the newer edit (B's) wins on both, the other is kept.
    let from_a = edit_note(&a, &note_id, "edited on A", Some(now + 1));
    edit_note(&b, &note_id, "edited on B", Some(now + 2));
    sync_once(&a, &remote).expect("sync A");
    let report = sync_once(&b, &remote).expect("sync B");
    assert_eq!((report.conflicts_local, report.pushed), (1, 1));
    assert!(has_revision(&b, &note_id, &from_a.rev));
    let report = sync_once(&a, &remote).expect("sync A");
    assert_eq!(report.pulled, 1);
    assert_eq!(body(&a, &note_id).as_deref(), Some("edited on B"));
    assert!(has_revision(&a, &note_id, &from_a.rev));
    eprintln!("conflict resolved: B's newer edit on both devices, A's edit kept in history");

    // 3. Images round-trip with their bytes, including a 2 MB one.
    let big: Vec<u8> = (0..2 * 1024 * 1024u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    file(&a, &png_id, &PNG);
    file(&a, &big_id, &big);
    let report = sync_once(&a, &remote).expect("sync A");
    assert_eq!(report.files_up, 2);
    let report = sync_once(&b, &remote).expect("sync B");
    assert_eq!(report.files_down, 2);
    assert_eq!(b.attachment(&png_id).unwrap().unwrap().1.data, PNG);
    assert_eq!(b.attachment(&big_id).unwrap().unwrap().1.data, big);
    let stored = remote.get_file(&png_id).unwrap().unwrap();
    assert_eq!(stored.md5_hex, md5_hex(&PNG));
    assert_eq!(stored.content_type, "image/png");
    eprintln!(
        "images round-tripped: {} and {} bytes",
        PNG.len(),
        big.len()
    );

    // 4. Clean up: delete everything on A; the deletions reach B and the
    //    server drops the bytes.
    for id in [&note_id, &png_id, &big_id, &book_id] {
        a.delete(id, Origin::Api).unwrap();
    }
    let report = sync_once(&a, &remote).expect("sync A");
    assert_eq!(report.pushed, 4);
    sync_once(&b, &remote).expect("sync B");
    for id in [&note_id, &png_id, &big_id, &book_id] {
        assert!(b.get(id).unwrap().is_none(), "{id} still live on B");
    }
    assert!(remote.get_file(&png_id).unwrap().is_none());
    assert!(remote.get_file(&big_id).unwrap().is_none());
    assert_eq!(pending(&a) + pending(&b), 0);
    assert_eq!(all_docs(&a), all_docs(&b));
    eprintln!("cleaned up: the four test documents are tombstones on the server");

    // 5. With KANSO_SYNC_E2E_BULK=1: a full-size push (the server's maximum,
    //    600 D1 statements in one batch) and a paged pull, then the same as
    //    deletions. Leaves 200 more tombstones, so it is opt-in.
    if std::env::var("KANSO_SYNC_E2E_BULK").is_err() {
        return;
    }
    let bulk = kanso_sync::SyncOptions {
        batch_docs: kanso_sync::MAX_PUSH_DOCS,
        ..kanso_sync::SyncOptions::default()
    };
    let ids: Vec<String> = (0..kanso_sync::MAX_PUSH_DOCS)
        .map(|i| format!("note:e2e{run}n{i}"))
        .collect();
    for id in &ids {
        note(&a, id, "trash", "bulk", now);
    }
    let report = kanso_sync::sync_with(&a, &remote, &bulk).expect("bulk push");
    assert_eq!(report.pushed, ids.len());
    let report = sync_once(&b, &remote).expect("paged pull");
    assert_eq!(report.pulled, ids.len());
    for id in &ids {
        a.delete(id, Origin::Api).unwrap();
    }
    let report = kanso_sync::sync_with(&a, &remote, &bulk).expect("bulk delete");
    assert_eq!(report.pushed, ids.len());
    sync_once(&b, &remote).expect("sync B");
    assert_eq!(all_docs(&a), all_docs(&b));
    eprintln!(
        "bulk: {} notes in one push, pulled in pages, then deleted",
        ids.len()
    );
}
