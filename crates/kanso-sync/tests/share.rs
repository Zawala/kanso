//! Publishing a note (docs/ACCOUNTS.md › Published notes) through the public
//! API against `MemoryShares`: publish, update in place, the account's other
//! Mac managing the same link, unpublish, and what the server ends up
//! holding (ciphertext only). The sealing properties are in share.rs's unit
//! tests; the Worker's own rules in `cloud/test/shares.test.ts`.

mod common;

use kanso_sync::MemoryServer;
use kanso_sync::share::{
    KansoPublish, MemoryShares, Page, ShareApi, ShareError, open_share, publish, unpublish,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const HTML: &str = "<!doctype html><title>Plan</title><p>The secret plan.</p>";

fn page<'a>(html: &'a str, body: &'a str) -> Page<'a> {
    Page {
        html: html.as_bytes(),
        title: "Plan",
        body,
        rev: "2-abc",
    }
}

#[test]
fn publish_update_and_unpublish() {
    let server = MemoryServer::new();
    let session = common::sign_up(&server, "alice", 1);
    let shares = MemoryShares::new();

    let first = publish(&shares, &session, None, &page(HTML, "v1"), None, 1_000).unwrap();
    let stored = shares.get(&first.share_id).unwrap();
    assert!(
        !common::contains(&stored.sealed, b"secret plan"),
        "only ciphertext reaches the server"
    );
    assert!(!common::contains(&stored.sealed, first.key.as_bytes()));
    let key = first.share_key().unwrap();
    assert_eq!(
        open_share(&key, &first.share_id, &stored.sealed).unwrap(),
        HTML.as_bytes()
    );
    assert_eq!(first.published_at, 1_000);
    assert_eq!(first.published_rev, "2-abc");
    assert!(!first.is_outdated("Plan", "v1"));
    assert!(first.is_outdated("Plan", "v2"));
    assert_eq!(
        first.link(&session.server),
        format!("{}/p/{}#{}", session.server, first.share_id, first.key)
    );

    // An update keeps the link and the key; the ciphertext is new.
    let html2 = "<!doctype html><p>Revised.</p>";
    let second = publish(
        &shares,
        &session,
        Some(&first),
        &page(html2, "v2"),
        Some(9_000),
        2_000,
    )
    .unwrap();
    assert_eq!(
        (&second.share_id, &second.key),
        (&first.share_id, &first.key)
    );
    let restored = shares.get(&first.share_id).unwrap();
    assert_ne!(restored.sealed[1..13], stored.sealed[1..13], "a new nonce");
    assert_eq!(restored.expires_at, Some(9_000));
    assert_eq!(
        open_share(&key, &first.share_id, &restored.sealed).unwrap(),
        html2.as_bytes()
    );
    assert!(!second.is_outdated("Plan", "v2"));

    unpublish(&shares, &session, &second).unwrap();
    assert!(shares.is_empty());
    // Already gone counts as done.
    unpublish(&shares, &session, &second).unwrap();
}

#[test]
fn another_mac_of_the_account_manages_the_same_link() {
    let server = MemoryServer::new();
    let one = common::sign_up(&server, "alice", 1);
    let two = common::sign_in(&server, "alice", 2);
    let shares = MemoryShares::new();
    let first = publish(&shares, &one, None, &page(HTML, "v1"), None, 1).unwrap();
    // The second Mac reads kansoPublish from the synced note.
    let mut extra = serde_json::Map::new();
    KansoPublish::store(Some(&first), &mut extra);
    let synced = KansoPublish::from_extra(&extra).unwrap();
    // It updates the same link in place, with the same key.
    let second = publish(&shares, &two, Some(&synced), &page(HTML, "v2"), None, 2).unwrap();
    assert_eq!(
        (&second.share_id, &second.key),
        (&first.share_id, &first.key)
    );
    assert!(second.is_outdated("Plan", "v1"));
    // And can unpublish it.
    unpublish(&shares, &two, &second).unwrap();
    assert!(shares.is_empty());
}

#[test]
fn another_account_cant_touch_a_share() {
    let server = MemoryServer::new();
    let alice = common::sign_up(&server, "alice", 1);
    let bob = common::sign_up(&server, "bob", 2);
    let shares = MemoryShares::new();
    let mine = publish(&shares, &alice, None, &page(HTML, "v1"), None, 1).unwrap();
    // Bob holding Alice's kansoPublish (he can't: it's in her sealed note)
    // still can't overwrite or delete it.
    let theirs = publish(
        &shares,
        &bob,
        Some(&mine),
        &page("<p>Bob</p>", "b"),
        None,
        2,
    )
    .unwrap();
    assert_ne!(theirs.share_id, mine.share_id);
    assert!(matches!(
        shares.delete_share(&bob, &mine.share_id),
        Err(kanso_sync::ApiError::NotFound)
    ));
    let stored = shares.get(&mine.share_id).unwrap();
    assert_eq!(
        open_share(&mine.share_key().unwrap(), &mine.share_id, &stored.sealed).unwrap(),
        HTML.as_bytes()
    );
}

#[test]
fn a_link_unpublished_elsewhere_is_published_afresh() {
    let server = MemoryServer::new();
    let session = common::sign_up(&server, "alice", 1);
    let shares = MemoryShares::new();
    let first = publish(&shares, &session, None, &page(HTML, "v1"), None, 1).unwrap();
    unpublish(&shares, &session, &first).unwrap();
    // A Mac that hadn't synced the unpublish yet updates the old link.
    let again = publish(&shares, &session, Some(&first), &page(HTML, "v1"), None, 2).unwrap();
    assert_ne!(again.share_id, first.share_id, "the old link stays dead");
    assert_ne!(again.key, first.key);
    assert!(shares.get(&first.share_id).is_none());
}

#[test]
fn failures_leave_nothing_half_done() {
    let server = MemoryServer::new();
    let session = common::sign_up(&server, "alice", 1);
    let shares = MemoryShares::new();
    shares.go_offline(1);
    assert!(matches!(
        publish(&shares, &session, None, &page(HTML, "v1"), None, 1),
        Err(ShareError::Api(_))
    ));
    assert!(shares.is_empty());
    let big = "x".repeat(10 * 1024 * 1024 + 1);
    assert!(matches!(
        publish(&shares, &session, None, &page(&big, "v1"), None, 1),
        Err(ShareError::TooLarge(_))
    ));
    let first = publish(&shares, &session, None, &page(HTML, "v1"), None, 1).unwrap();
    shares.go_offline(1);
    assert!(unpublish(&shares, &session, &first).is_err());
    assert!(shares.get(&first.share_id).is_some());
}

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config(32))]

    /// Model-based: after any run of publishes, updates and unpublishes of
    /// one note, the server holds exactly the note's current link, which
    /// opens to the last page published, and nothing when unpublished.
    #[test]
    fn prop_the_server_holds_what_the_note_says(steps in prop::collection::vec((any::<bool>(), "[a-z ]{0,30}"), 1..12)) {
        let server = MemoryServer::new();
        let session = common::sign_up(&server, "alice", 1);
        let shares = MemoryShares::new();
        let mut note: Option<KansoPublish> = None;
        let mut last_html = String::new();
        for (n, (publish_step, text)) in steps.iter().enumerate() {
            if *publish_step {
                last_html = format!("<p>{text}</p>");
                note = Some(publish(&shares, &session, note.as_ref(), &page(&last_html, text), None, n as i64).unwrap());
            } else if let Some(p) = note.take() {
                unpublish(&shares, &session, &p).unwrap();
            }
            match &note {
                Some(p) => {
                    prop_assert_eq!(shares.len(), 1);
                    let stored = shares.get(&p.share_id).unwrap();
                    let opened = open_share(&p.share_key().unwrap(), &p.share_id, &stored.sealed).unwrap();
                    prop_assert_eq!(opened, last_html.as_bytes());
                }
                None => prop_assert!(shares.is_empty()),
            }
        }
    }
}
