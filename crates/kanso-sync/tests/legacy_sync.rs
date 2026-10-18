//! Example tests for the legacy v1 engine (`sync_once` over the plaintext
//! `MemoryRemote`): one per conflict path, files, failures and the edge cases
//! the property tests generalize. The app runs this engine until step 3 of
//! docs/ACCOUNTS.md › Build order; `tests/sync.rs` has the v2 cases.

mod common;

use std::cell::{Cell, RefCell};

use common::*;
use kanso_core::DocType;
use kanso_store::{Store, SyncDoc};
use kanso_sync::{
    ChangesPage, MemoryRemote, PushResult, Remote, RemoteError, RemoteFile, SyncOptions,
    SyncReport, cursor, md5_hex, reset_cursor, sync_once, sync_with,
};
use serde_json::json;

/// A remote that runs a hook on the first push and can fail `changes` calls.
struct Hooked<'a> {
    inner: &'a MemoryRemote,
    before_push: RefCell<Option<Box<dyn FnOnce() + 'a>>>,
    after_push: RefCell<Option<Box<dyn FnOnce() + 'a>>>,
    changes_left: Cell<Option<usize>>,
}

impl<'a> Hooked<'a> {
    fn new(inner: &'a MemoryRemote) -> Self {
        Hooked {
            inner,
            before_push: RefCell::new(None),
            after_push: RefCell::new(None),
            changes_left: Cell::new(None),
        }
    }
}

impl Remote for Hooked<'_> {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        if let Some(hook) = self.before_push.borrow_mut().take() {
            hook();
        }
        let result = self.inner.push(docs);
        if let Some(hook) = self.after_push.borrow_mut().take() {
            hook();
        }
        result
    }
    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        if let Some(left) = self.changes_left.get() {
            if left == 0 {
                return Err(RemoteError::Transport("offline (simulated)".into()));
            }
            self.changes_left.set(Some(left - 1));
        }
        self.inner.changes(since, limit)
    }
    fn put_file(&self, id: &str, md5: &str, ct: &str, bytes: &[u8]) -> Result<(), RemoteError> {
        self.inner.put_file(id, md5, ct, bytes)
    }
    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        self.inner.get_file(id)
    }
}

/// Two stores that both have `note:n1` in `book:b1`, synced.
fn two_synced() -> (MemoryRemote, Vec<tempfile::TempDir>, Store, Store) {
    let remote = MemoryRemote::new();
    let (da, a) = temp_store();
    let (db, b) = temp_store();
    book(&a, "book:b1", "Inbox");
    note(&a, "note:n1", "book:b1", "first", 100);
    sync_once(&a, &remote).unwrap();
    sync_once(&b, &remote).unwrap();
    assert_eq!(body(&b, "note:n1").as_deref(), Some("first"));
    (remote, vec![da, db], a, b)
}

#[test]
fn a_note_created_on_one_device_reaches_the_other() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    book(&a, "book:b1", "Inbox");
    let created = note(&a, "note:n1", "book:b1", "hello", 100);

    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(report.pushed, 2);
    assert_eq!(report.pulled, 0);
    assert_eq!(pending(&a), 0);

    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.pulled, 2);
    assert_eq!(report.pushed, 0);
    assert_eq!(body(&b, "note:n1").as_deref(), Some("hello"));
    assert_eq!(b.get("note:n1").unwrap().unwrap().rev, created.rev);
    assert_eq!(all_docs(&a), all_docs(&b));
    assert_eq!(cursor(&b).unwrap(), remote.last_seq());
}

#[test]
fn syncing_an_up_to_date_store_does_nothing() {
    let (remote, _dirs, a, b) = two_synced();
    sync_once(&a, &remote).unwrap();
    let before = (all_docs(&a), remote.last_seq(), remote.stats().pushes);
    assert_eq!(sync_once(&a, &remote).unwrap(), SyncReport::default());
    assert_eq!(sync_once(&b, &remote).unwrap(), SyncReport::default());
    assert_eq!(
        (all_docs(&a), remote.last_seq(), remote.stats().pushes),
        before
    );
}

#[test]
fn newer_local_edit_wins_and_the_remote_one_is_kept() {
    let (remote, _dirs, a, b) = two_synced();
    let theirs = edit_note(&a, "note:n1", "from A", Some(200));
    sync_once(&a, &remote).unwrap();
    edit_note(&b, "note:n1", "from B", Some(300));

    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.conflicts_local, 1);
    assert_eq!(report.conflicts_remote, 0);
    assert_eq!(report.pushed, 1);
    assert_eq!(report.conflicted, vec!["note:n1".to_owned()]);
    assert_eq!(body(&b, "note:n1").as_deref(), Some("from B"));
    assert!(
        has_revision(&b, "note:n1", &theirs.rev),
        "A's edit is kept on B"
    );

    sync_once(&a, &remote).unwrap();
    assert_eq!(body(&a, "note:n1").as_deref(), Some("from B"));
    assert!(
        has_revision(&a, "note:n1", &theirs.rev),
        "A's edit is kept on A"
    );
    assert_eq!(live_docs(&a), live_docs(&b));
}

#[test]
fn newer_remote_edit_wins_and_the_local_one_is_kept() {
    let (remote, _dirs, a, b) = two_synced();
    edit_note(&a, "note:n1", "from A", Some(300));
    sync_once(&a, &remote).unwrap();
    let mine = edit_note(&b, "note:n1", "from B", Some(200));

    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.conflicts_remote, 1);
    assert_eq!(report.pushed, 0);
    assert_eq!(body(&b, "note:n1").as_deref(), Some("from A"));
    assert!(
        has_revision(&b, "note:n1", &mine.rev),
        "B's edit is kept on B"
    );
    assert_eq!(pending(&b), 0);
    assert_eq!(live_docs(&a), live_docs(&b));
}

#[test]
fn a_conflict_found_by_the_push_is_resolved_in_the_same_sync() {
    let (remote, _dirs, a, b) = two_synced();
    let theirs = edit_note(&a, "note:n1", "from A", Some(200));
    edit_note(&b, "note:n1", "from B", Some(300));
    // A pushes after B pulled but before B pushes.
    let hooked = Hooked::new(&remote);
    *hooked.before_push.borrow_mut() = Some(Box::new(|| {
        sync_once(&a, &remote).unwrap();
    }));

    let report = sync_once(&b, &hooked).unwrap();
    assert_eq!(report.conflicts_local, 1);
    assert_eq!(
        report.pushed, 1,
        "the winning edit goes out in the second round"
    );
    assert_eq!(report.pending, 0);
    assert!(has_revision(&b, "note:n1", &theirs.rev));
    sync_once(&a, &remote).unwrap();
    assert_eq!(body(&a, "note:n1").as_deref(), Some("from B"));
}

#[test]
fn an_edit_beats_a_deletion() {
    let (remote, _dirs, a, b) = two_synced();
    a.delete("note:n1", kanso_store::Origin::Api).unwrap();
    sync_once(&a, &remote).unwrap();
    edit_note(&b, "note:n1", "still needed", Some(50));

    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.conflicts_local, 1);
    assert_eq!(body(&b, "note:n1").as_deref(), Some("still needed"));
    sync_once(&a, &remote).unwrap();
    assert_eq!(body(&a, "note:n1").as_deref(), Some("still needed"));
    assert_eq!(all_docs(&a), all_docs(&b));
}

#[test]
fn a_deletion_reaches_a_clean_copy() {
    let (remote, _dirs, a, b) = two_synced();
    a.delete("note:n1", kanso_store::Origin::Api).unwrap();
    sync_once(&a, &remote).unwrap();
    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.pulled, 1);
    assert_eq!(report.conflicts(), 0);
    assert!(b.get("note:n1").unwrap().is_none());
    assert_eq!(all_docs(&a), all_docs(&b));
}

#[test]
fn files_travel_with_their_bytes_and_deletions_drop_them() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    let png = b"\x89PNG\r\n\x1a\n not really".to_vec();
    file(&a, "file:img1", &png);

    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(report.files_up, 1);
    assert_eq!(remote.file("file:img1").unwrap().md5_hex, md5_hex(&png));

    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.files_down, 1);
    let (content_type, blob) = b.attachment("file:img1").unwrap().unwrap();
    assert_eq!(content_type, "image/png");
    assert_eq!(blob.data, png);

    // A rename re-sends the bytes; B already has them and does not download.
    merge(
        &a,
        DocType::File,
        json!({"_id": "file:img1", "name": "renamed.png"}),
    )
    .unwrap();
    sync_once(&a, &remote).unwrap();
    let report = sync_once(&b, &remote).unwrap();
    assert_eq!((report.pulled, report.files_down), (1, 0));
    assert_eq!(
        b.file_meta("file:img1").unwrap().unwrap().name,
        "renamed.png"
    );

    a.delete("file:img1", kanso_store::Origin::Api).unwrap();
    sync_once(&a, &remote).unwrap();
    assert!(remote.file("file:img1").is_none());
    sync_once(&b, &remote).unwrap();
    assert!(b.attachment("file:img1").unwrap().is_none());
}

#[test]
fn replaced_file_bytes_reach_the_other_device() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    file(&a, "file:img1", b"one");
    sync_once(&a, &remote).unwrap();
    sync_once(&b, &remote).unwrap();
    let mut doc = a.get_api("file:img1", None, false).unwrap();
    doc["_attachments"] = file_body("file:img1", "image.png", b"two")["_attachments"].clone();
    put(&a, DocType::File, doc).unwrap();
    sync_once(&a, &remote).unwrap();
    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.files_down, 1);
    assert_eq!(b.attachment("file:img1").unwrap().unwrap().1.data, b"two");
}

#[test]
fn a_lost_push_response_is_recovered_without_a_conflict() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    book(&a, "book:b1", "Inbox");
    note(&a, "note:n1", "book:b1", "hello", 100);
    remote.drop_next_push_responses(1);
    assert!(matches!(
        sync_once(&a, &remote),
        Err(kanso_sync::SyncError::Remote(RemoteError::Transport(_)))
    ));
    assert_eq!(pending(&a), 2, "nothing is marked synced without an answer");

    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(report.conflicts(), 0);
    assert_eq!(
        report.pushed, 0,
        "the pull showed the server already has them"
    );
    assert_eq!(pending(&a), 0);
    assert!(a.revisions("note:n1").unwrap().is_empty());
}

#[test]
fn an_edit_made_while_pushing_goes_out_next_time() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    book(&a, "book:b1", "Inbox");
    note(&a, "note:n1", "book:b1", "v1", 100);
    let hooked = Hooked::new(&remote);
    // The user types while the push is in flight; the merge keeps updatedAt.
    *hooked.after_push.borrow_mut() = Some(Box::new(|| {
        merge(&a, DocType::Note, json!({"_id": "note:n1", "body": "v2"})).unwrap();
    }));
    let report = sync_once(&a, &hooked).unwrap();
    assert_eq!(report.pending, 1);
    assert_eq!(report.conflicts(), 0);

    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(
        report.conflicts(),
        0,
        "our own earlier push is not a conflict"
    );
    assert_eq!(report.pushed, 1);
    assert_eq!(body(&a, "note:n1").as_deref(), Some("v2"));
    sync_once(&b, &remote).unwrap();
    assert_eq!(body(&b, "note:n1").as_deref(), Some("v2"));
}

#[test]
fn our_own_push_coming_back_does_not_revert_a_later_merge() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    book(&a, "book:b1", "Inbox");
    note(&a, "note:n1", "book:b1", "v1", 100);
    sync_once(&a, &remote).unwrap();
    merge(
        &a,
        DocType::Note,
        json!({"_id": "note:n1", "status": "active"}),
    )
    .unwrap();

    // The pull sees our own v1 (same updatedAt as the merge) before the push.
    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(report.conflicts(), 0);
    assert_eq!(report.pushed, 1);
    assert_eq!(
        a.get_api("note:n1", None, false).unwrap()["status"],
        "active"
    );
}

#[test]
fn a_missing_attachment_is_reported_and_fetched_by_a_full_resync() {
    let remote = MemoryRemote::new();
    let (_db, b) = temp_store();
    let bytes = b"late bytes".to_vec();
    let md5 = md5_hex(&bytes);
    let doc = SyncDoc {
        id: "file:late".into(),
        rev: "1-ab".into(),
        deleted: false,
        fields: json!({"name": "late.png", "contentType": "image/png", "md5digest": md5, "createdAt": 1})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    // A device pushed the document but its upload never arrived.
    remote.push(&[doc]).unwrap();
    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.files_missing, 1);
    assert!(b.get("file:late").unwrap().is_some());
    assert!(b.attachment("file:late").unwrap().is_none());

    remote
        .put_file("file:late", &md5, "image/png", &bytes)
        .unwrap();
    assert_eq!(
        sync_once(&b, &remote).unwrap().files_down,
        0,
        "nothing new in the feed"
    );
    reset_cursor(&b).unwrap();
    let report = sync_once(&b, &remote).unwrap();
    assert_eq!(report.files_down, 1);
    assert_eq!(b.attachment("file:late").unwrap().unwrap().1.data, bytes);
}

#[test]
fn a_document_too_large_for_the_server_stays_pending_and_is_reported() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    book(&a, "book:b1", "Inbox");
    // 600k four-byte characters: within the store's limit, over D1's row size.
    note(&a, "note:big", "book:b1", &"😀".repeat(600_000), 100);
    note(&a, "note:small", "book:b1", "fine", 100);
    let report = sync_once(&a, &remote).unwrap();
    assert_eq!(report.too_large, vec!["note:big".to_owned()]);
    assert_eq!(report.pushed, 2);
    assert_eq!(report.pending, 1);
}

#[test]
fn a_server_without_our_base_does_not_take_the_document() {
    let (_remote, _dirs, a, _b) = two_synced();
    edit_note(&a, "note:n1", "edited", Some(500));
    let fresh = MemoryRemote::new();
    reset_cursor(&a).unwrap();
    let report = sync_once(&a, &fresh).unwrap();
    assert_eq!(report.pushed, 0);
    assert_eq!(report.pending, 1);
}

#[test]
fn the_cursor_is_saved_after_each_page() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    let (_db, b) = temp_store();
    book(&a, "book:b1", "Inbox");
    for i in 0..4 {
        note(&a, &format!("note:n{i}"), "book:b1", "x", 100);
    }
    sync_once(&a, &remote).unwrap();
    let options = SyncOptions {
        page_size: 2,
        ..SyncOptions::default()
    };
    let hooked = Hooked::new(&remote);
    hooked.changes_left.set(Some(2));
    assert!(sync_with(&b, &hooked, &options).is_err());
    assert_eq!(cursor(&b).unwrap(), 4, "two full pages were applied");

    let report = sync_with(&b, &remote, &options).unwrap();
    assert_eq!(report.pulled, 1);
    assert_eq!(all_docs(&a), all_docs(&b));
}

#[test]
fn pushes_are_split_into_batches() {
    let remote = MemoryRemote::new();
    let (_da, a) = temp_store();
    book(&a, "book:b1", "Inbox");
    for i in 0..9 {
        note(&a, &format!("note:n{i}"), "book:b1", "x", 100);
    }
    let options = SyncOptions {
        batch_docs: 4,
        ..SyncOptions::default()
    };
    let report = sync_with(&a, &remote, &options).unwrap();
    assert_eq!(report.pushed, 10);
    assert_eq!(remote.stats().pushes, 3);
}

#[test]
fn the_http_remote_never_shows_its_token() {
    let remote = kanso_sync::HttpRemote::new("https://example.test/", "s3cret");
    assert_eq!(remote.endpoint(), "https://example.test");
    assert!(!format!("{remote:?}").contains("s3cret"));
}
