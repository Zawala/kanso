//! End to end against a local Worker (docs/ACCOUNTS.md › Tests › End to end).
//! `cloud/scripts/e2e-local.sh` starts `wrangler dev --local` with sign-ups
//! open and runs:
//!
//! ```text
//! KANSO_SYNC_E2E_ENDPOINT=http://127.0.0.1:8787 KANSO_SYNC_E2E_CANARIES=$list \
//!     cargo test -p kanso-sync --test e2e -- --nocapture
//! # then, once wrangler dev has exited:
//! KANSO_SYNC_E2E_STATE=$dir KANSO_SYNC_E2E_CANARIES=$list \
//!     cargo test -p kanso-sync --test e2e -- --ignored the_stopped_state_holds_no_canary
//! ```
//!
//! Beside the sync run, `a_new_mac_is_approved_live_and_the_dropped_socket_reconnects`
//! approves a new Mac over the live account event socket and checks the
//! socket's reconnect after a drop.
//!
//! It runs at production KDF parameters (`HttpBackend`) and passes without
//! doing anything when `KANSO_SYNC_E2E_ENDPOINT` is unset. Sessions stay in
//! memory; nothing touches the Keychain. The canary scan reads the state
//! only after the Worker stops: while it runs, D1's recent writes aren't in
//! its files yet (the scan's positive control caught that).

mod common;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::*;
use kanso_store::Origin;
use kanso_sync::api::{LoginDevice, LoginRequest};
use kanso_sync::wire::{
    MAX_CHANGES_LIMIT, MAX_PUSH_BODY, MAX_PUSH_DOCS, WirePage, WirePushDoc, WirePushResult,
    WireRemote,
};
use kanso_sync::{
    AccountError, Api, ApiError, Backend, DeviceIdentity, HttpBackend, KdfParams, MemorySecrets,
    Password, Session, SignOutReason, SyncError, SyncOptions,
};

/// `HttpBackend`, keeping a copy of every sealed file it uploads so the
/// state scan can check R2 let go of them.
struct Recording {
    uploads: Mutex<Vec<Vec<u8>>>,
}

struct RecordingRemote<'a> {
    inner: Box<dyn WireRemote + 'a>,
    uploads: &'a Mutex<Vec<Vec<u8>>>,
}

impl WireRemote for RecordingRemote<'_> {
    fn push(&self, docs: &[WirePushDoc]) -> Result<Vec<WirePushResult>, ApiError> {
        self.inner.push(docs)
    }
    fn changes(&self, since: i64, limit: usize) -> Result<WirePage, ApiError> {
        self.inner.changes(since, limit)
    }
    fn put_file(&self, wire_id: &str, sealed: &[u8]) -> Result<(), ApiError> {
        self.uploads.lock().unwrap().push(sealed.to_vec());
        self.inner.put_file(wire_id, sealed)
    }
    fn get_file(&self, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.inner.get_file(wire_id)
    }
}

impl Backend for Recording {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        HttpBackend.api(server)
    }
    fn remote(&self, session: &Session) -> Box<dyn WireRemote + '_> {
        Box::new(RecordingRemote {
            inner: HttpBackend.remote(session),
            uploads: &self.uploads,
        })
    }
    fn kdf_floor(&self) -> KdfParams {
        HttpBackend.kdf_floor()
    }
    fn kdf_default(&self) -> KdfParams {
        HttpBackend.kdf_default()
    }
}

/// The auth front filter's window (`AUTH_LIMITER`: 20 requests a minute per
/// IP), which is also its `retry_after`.
const BINDING_SECS: u64 = 60;

/// Runs an account call, waiting out the auth front filter when it answers
/// 429. A longer `retry_after` is a D1 rule's, and comes back as it is.
fn patiently<T>(call: impl Fn() -> Result<T, AccountError>) -> Result<T, AccountError> {
    for _ in 0..3 {
        match call() {
            Err(AccountError::RateLimited { retry_after }) if retry_after <= BINDING_SECS => {
                eprintln!("rate limited by the binding; waiting {retry_after} s");
                std::thread::sleep(Duration::from_secs(retry_after + 1));
            }
            other => return other,
        }
    }
    call()
}

/// Every file under `dir`, recursively.
fn files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn identity(n: u8, name: &str) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{n:032x}"), name, [n; 32])
}

fn publishes_without_a_short_link(endpoint: &str, session: &Session) {
    use kanso_sync::share::{HttpShares, Page, open_share, publish, unpublish};
    let page = Page {
        html: b"<!doctype html><title>E2E</title><p>Published.</p>",
        title: "E2E plan \u{2615}",
        body: "Published.",
        rev: "1-a",
    };
    let published = publish(
        &HttpShares,
        session,
        None,
        &page,
        None,
        kanso_core::now_ms(),
    )
    .expect("publish");
    assert_eq!(published.short_url, None, "no Rerout behind wrangler dev");
    assert_eq!(published.link(endpoint), published.long_link(endpoint));
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into();
    let get = |path: &str| {
        let mut response = agent.get(format!("{endpoint}{path}")).call().expect(path);
        let status = response.status().as_u16();
        let kind = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        (status, kind, response.body_mut().read_to_vec().unwrap())
    };
    let (status, _, sealed) = get(&format!("/v2/public/{}", published.share_id));
    assert_eq!(status, 200);
    assert_eq!(
        open_share(
            &published.share_key().unwrap(),
            &published.share_id,
            &sealed
        )
        .unwrap(),
        page.html
    );
    let (status, _, viewer) = get(&format!("/p/{}", published.share_id));
    let viewer = String::from_utf8(viewer).unwrap();
    assert_eq!(status, 200);
    // wrangler dev hands the Worker the route's host, so only the path is checked.
    let image_tag = viewer
        .lines()
        .find(|l| l.starts_with("<meta property=\"og:image\""))
        .expect("a preview image");
    assert!(image_tag.ends_with("/og/kanso.png\">"), "{image_tag}");
    assert!(viewer.contains("noindex"));
    let (status, kind, image) = get("/og/kanso.png");
    assert_eq!((status, kind.as_str()), (200, "image/png"));
    assert!(image.starts_with(b"\x89PNG\r\n\x1a\n"));
    unpublish(&HttpShares, session, &published).expect("unpublish");
    assert_eq!(get(&format!("/v2/public/{}", published.share_id)).0, 404);
}

#[test]
fn two_macs_sync_through_a_local_worker() {
    let Ok(endpoint) = std::env::var("KANSO_SYNC_E2E_ENDPOINT") else {
        eprintln!("skipped: set KANSO_SYNC_E2E_ENDPOINT (see cloud/scripts/e2e-local.sh)");
        return;
    };
    let backend = Recording {
        uploads: Mutex::new(Vec::new()),
    };
    // A second account that stays, for the canary scan's positive control.
    let witness_backend = Recording {
        uploads: Mutex::new(Vec::new()),
    };
    let run = format!("{:x}", kanso_core::now_ms());
    let marker = format!("zqcanary{run}");
    let username = format!("e2e{run}");
    let password_text = format!("{PASSWORD} {marker}");
    let password = Password::new(&password_text, &username).unwrap();
    let device_a = identity(1, &format!("Mac A {marker}"));
    let device_b = identity(2, &format!("Mac B {marker}"));

    // 1. Two libraries sign up and sign in, then exchange notes, a
    //    conflict, a 2 MB image and deletions.
    let start = Instant::now();
    let signed_up = patiently(|| {
        kanso_sync::sign_up(
            &backend,
            &MemorySecrets::new(),
            &endpoint,
            &username,
            &password,
            &device_a,
            &|_| {},
        )
        .map_err(|err| err.error)
    })
    .expect("sign up");
    eprintln!("sign-up at production parameters: {:?}", start.elapsed());
    let recovery_key = signed_up.recovery_key;
    let a = Mac::new(signed_up.session);
    let signed_in = patiently(|| {
        kanso_sync::sign_in(
            &backend,
            &endpoint,
            &username,
            &password,
            &device_b,
            &|_| {},
        )
    })
    .expect("sign in");
    let b = Mac::new(signed_in.session);

    let book_id = format!("book:{marker}b");
    let note_id = format!("note:{marker}n");
    let img_id = format!("file:{marker}f");
    book(&a.store, &book_id, &format!("{marker} notebook"));
    note(&a.store, &note_id, &book_id, &format!("{marker} body"), 100);
    let image: Vec<u8> = (0..2 * 1024 * 1024u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    file(&a.store, &img_id, &image);
    let report = a.sync(&backend);
    assert_eq!((report.pushed, report.files_up), (3, 1), "{report:?}");
    let report = b.sync(&backend);
    assert_eq!((report.pulled, report.files_down), (3, 1), "{report:?}");
    assert_eq!(b.store.attachment(&img_id).unwrap().unwrap().1.data, image);

    let from_a = edit_note(&a.store, &note_id, &format!("{marker} from A"), Some(200));
    a.sync(&backend);
    edit_note(&b.store, &note_id, &format!("{marker} from B"), Some(300));
    let report = b.sync(&backend);
    assert_eq!(report.conflicts_local, 1, "{report:?}");
    a.sync(&backend);
    assert_eq!(body(&a.store, &note_id), Some(format!("{marker} from B")));
    assert!(has_revision(&b.store, &note_id, &from_a.rev));

    a.store.delete(&img_id, Origin::Api).unwrap();
    a.sync(&backend);
    b.sync(&backend);
    assert!(b.store.attachment(&img_id).unwrap().is_none());
    assert_eq!(all_docs(&a.store), all_docs(&b.store));
    eprintln!("notes, a conflict, a 2 MB image and a deletion went across");

    // 5. A full-size push (five ~800 KB notes in 4 MiB) and an 8 MiB page;
    //    the wall times are the CPU bench.
    let bulk = SyncOptions {
        batch_bytes: MAX_PUSH_BODY - 1024,
        batch_docs: MAX_PUSH_DOCS,
        page_size: MAX_CHANGES_LIMIT,
        ..SyncOptions::default()
    };
    for i in 0..11usize {
        let text: String = (0..600_000usize)
            .map(|j| char::from(b'a' + ((i * 7 + j) % 26) as u8))
            .collect();
        note(
            &a.store,
            &format!("note:{marker}big{i}"),
            &book_id,
            &text,
            400,
        );
    }
    let start = Instant::now();
    let report = a.sync_with(&backend, &bulk).expect("bulk push");
    eprintln!(
        "bench: pushed {} large notes in {:?}",
        report.pushed,
        start.elapsed()
    );
    assert_eq!(report.pushed, 11, "{report:?}");
    let start = Instant::now();
    let report = b.sync_with(&backend, &bulk).expect("big page");
    eprintln!(
        "bench: pulled {} large notes in {:?}",
        report.pulled,
        start.elapsed()
    );
    assert_eq!(all_docs(&a.store), all_docs(&b.store));

    // 5b. Publishing over HTTP with the title: a Worker without Rerout (wrangler
    //     dev has no key, and its origin isn't https) answers no short link,
    //     so the long link works; the viewer's preview image is a static
    //     asset; unpublishing stops the link.
    publishes_without_a_short_link(&endpoint, &a.session);
    eprintln!("published and unpublished a note on its long link");

    // 2. A password change gives the other device 401 password_changed, and
    //    replaces the recovery key (security review 1).
    let second = Password::new(&format!("{PASSWORD} two {marker}"), &username).unwrap();
    let changed = patiently(|| {
        kanso_sync::change_password(
            &backend,
            &MemorySecrets::new(),
            &a.session,
            &password,
            &second,
            &|_| {},
        )
        .map_err(|err| err.error)
    })
    .expect("change password");
    match b.try_sync(&backend) {
        Err(SyncError::Api(ApiError::Unauthorized { reason })) => {
            assert_eq!(reason, SignOutReason::PasswordChanged)
        }
        other => panic!("expected 401 password_changed, got {other:?}"),
    }

    let password_change_key = changed.recovery_key;
    assert_ne!(password_change_key, recovery_key);

    // 3. Recovery rotates the key: the used one gets 401, and so does
    //    every session from before it. The sign-up key already stopped
    //    working with the password change.
    let third = Password::new(&format!("{PASSWORD} three {marker}"), &username).unwrap();
    let recover = |key: &kanso_sync::RecoveryKey| {
        patiently(|| {
            kanso_sync::recover(
                &backend,
                &MemorySecrets::new(),
                &endpoint,
                &username,
                key,
                &third,
                &device_b,
                &|_| {},
            )
            .map_err(|err| err.error)
        })
    };
    let before_change = recover(&recovery_key);
    assert!(
        matches!(before_change, Err(AccountError::InvalidCredentials)),
        "{before_change:?}"
    );
    let recovered = recover(&password_change_key).expect("recover");
    assert_ne!(recovered.recovery_key, password_change_key);
    let again = recover(&password_change_key);
    assert!(
        matches!(again, Err(AccountError::InvalidCredentials)),
        "{again:?}"
    );
    match a.try_sync(&backend) {
        Err(SyncError::Api(ApiError::Unauthorized { reason })) => {
            assert_eq!(reason, SignOutReason::Recovered)
        }
        other => panic!("expected 401 recovered, got {other:?}"),
    }
    let b = Mac {
        session: recovered.session,
        ..b
    };
    b.sync(&backend);

    // 4. Wrong passwords end in 429 from login-user (10 in 15 minutes per
    //    account), not just from the binding, whose 429s are waited out.
    //    The raw route takes any well-formed auth key, so no KDF runs.
    //    D1's windows are fixed (they start on the quarter hour), so a run
    //    that crosses a boundary counts again from zero: 22 tries covers 10
    //    before it and 11 after, still under login-ip's 30 per window.
    let origin = kanso_sync::origin(&endpoint).unwrap();
    let account_id = b.session.account_id.clone();
    let stranger = identity(9, "Stranger");
    let wrong_login = LoginRequest {
        username: username.clone(),
        auth_key: URL_SAFE_NO_PAD.encode([7u8; 32]),
        device: LoginDevice {
            id: stranger.device_id(&account_id),
            proof: stranger.device_proof(&origin, &account_id).to_base64url(),
        },
    };
    let api = HttpBackend.api(&endpoint);
    let mut refused_by = None;
    let mut attempts = 0;
    while attempts < 22 && refused_by.is_none() {
        match api.login(&wrong_login) {
            Err(ApiError::InvalidCredentials) => attempts += 1,
            Err(ApiError::RateLimited { limit, retry_after }) if limit == "binding" => {
                eprintln!("binding 429; waiting {retry_after} s");
                std::thread::sleep(Duration::from_secs(retry_after + 1));
            }
            Err(ApiError::RateLimited { limit, retry_after }) => {
                refused_by = Some((limit, retry_after))
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    let (limit, retry_after) = refused_by.expect("wrong passwords never hit a D1 rule");
    assert_eq!(limit, "login-user", "after {attempts} wrong passwords");
    // The rest of the fixed window: anything up to 15 minutes, and under a
    // minute when the tenth wrong password lands just before the boundary.
    assert!((1..=15 * 60).contains(&retry_after), "{retry_after}");
    // A Mac that already has a session for the account is exempt (M8): its
    // device proof matches that session's row.
    let exempt = patiently(|| {
        kanso_sync::sign_in(&backend, &endpoint, &username, &third, &device_b, &|_| {})
    })
    .expect("the known Mac signs in past login-user");
    let b = Mac {
        session: exempt.session,
        ..b
    };
    b.sync(&backend);
    eprintln!(
        "login-user refused a stranger after {attempts} wrong passwords; the known Mac got in"
    );

    // The witness: an account that stays, with one note and one file.
    let witness_name = format!("w{run}");
    let witness_password = Password::new(&format!("{PASSWORD} witness"), &witness_name).unwrap();
    let witness_session = patiently(|| {
        kanso_sync::sign_up(
            &witness_backend,
            &MemorySecrets::new(),
            &endpoint,
            &witness_name,
            &witness_password,
            &identity(3, "Witness"),
            &|_| {},
        )
        .map_err(|err| err.error)
    })
    .expect("witness sign-up")
    .session;
    let w = Mac::new(witness_session);
    book(&w.store, "book:witnessb", "witness notebook");
    file(&w.store, "file:witnessf", &image[..64 * 1024]);
    let report = w.sync(&witness_backend);
    assert_eq!(report.files_up, 1, "{report:?}");

    // 6. Deleting the account gives 401; its R2 objects go with the cron.
    patiently(|| kanso_sync::delete_account(&backend, &b.session, &third, &username, &|_| {}))
        .expect("delete account");
    assert!(matches!(
        b.try_sync(&backend),
        Err(SyncError::Api(ApiError::Unauthorized { .. }))
    ));
    for _ in 0..3 {
        let scheduled = ureq::get(format!("{endpoint}/__scheduled?cron=*/15+*+*+*+*")).call();
        eprintln!("/__scheduled: {:?}", scheduled.map(|r| r.status()));
    }

    // 7. What the scan of the stopped Worker's state looks for, written for
    //    `the_stopped_state_holds_no_canary` (the script runs it once
    //    `wrangler dev` has exited, so D1 has checkpointed everything into
    //    its files). The file lives outside the state directory.
    let Ok(canaries) = std::env::var("KANSO_SYNC_E2E_CANARIES") else {
        eprintln!("KANSO_SYNC_E2E_CANARIES unset: no on-disk canary scan");
        return;
    };
    let hex = |bytes: &[u8]| -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() };
    let mut secrets: Vec<(String, String)> = vec![
        ("the marker".into(), hex(marker.as_bytes())),
        ("the password".into(), hex(password_text.as_bytes())),
        (
            "the recovery key".into(),
            hex(recovery_key.display().as_bytes()),
        ),
        (
            "the password change's recovery key".into(),
            hex(password_change_key.display().as_bytes()),
        ),
    ];
    for doc in a.store.sync_docs().unwrap() {
        let hash = doc.rev.split_once('-').unwrap().1.to_owned();
        secrets.push((format!("the revision {}", doc.rev), hex(hash.as_bytes())));
    }
    let middle = |sealed: &[u8]| hex(&sealed[sealed.len() / 2..sealed.len() / 2 + 32]);
    let deleted_files: Vec<String> = backend
        .uploads
        .lock()
        .unwrap()
        .iter()
        .filter(|sealed| sealed.len() >= 64)
        .map(|sealed| middle(sealed))
        .collect();
    let witness_file = middle(&witness_backend.uploads.lock().unwrap()[0]);
    let list = serde_json::json!({
        "secrets": secrets,
        "deleted_files": deleted_files,
        "witness_name": hex(witness_name.as_bytes()),
        "witness_file": witness_file,
    });
    std::fs::write(&canaries, list.to_string()).expect("write the canary list");
    eprintln!("canary list written for the scan of the stopped state");
}

/// Wraps the first socket so it drops right after its first event, as a
/// lost connection would: the loop must reconnect and the next event must
/// arrive on the new socket.
struct DropsAfterFirstEvent {
    inner: Box<dyn kanso_sync::events::EventSource>,
    dropped: bool,
}

impl kanso_sync::events::EventSource for DropsAfterFirstEvent {
    fn next(
        &mut self,
        wait: Duration,
    ) -> Result<Option<kanso_sync::events::Frame>, kanso_sync::events::EventError> {
        if self.dropped {
            return Err(kanso_sync::events::EventError::Dropped(
                "dropped by the test".into(),
            ));
        }
        let frame = self.inner.next(wait)?;
        if matches!(frame, Some(kanso_sync::events::Frame::Event(_))) {
            self.dropped = true;
        }
        Ok(frame)
    }

    fn ping(&mut self) -> Result<(), kanso_sync::events::EventError> {
        self.inner.ping()
    }
}

/// docs/ACCOUNTS.md › Delivery plan, step 7: a new Mac is approved by the
/// signed-in one, which learns of the request and of its name over the live
/// account event socket (no timer), and that socket, dropped after the first
/// event, reconnects and delivers the second.
#[test]
fn a_new_mac_is_approved_live_and_the_dropped_socket_reconnects() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;

    use kanso_sync::events::{self, AccountEvent, EventError, EventSource, Signal};
    use kanso_sync::{
        ApprovalKeys, LockedSession, PinnedRequest, RequestState, SessionToken,
        account_events_path, challenge_request, complete_request, finish_approval, finish_unlock,
        name_device, open_request, pending_approvals, poll_approval, request_approval,
    };

    let Ok(endpoint) = std::env::var("KANSO_SYNC_E2E_ENDPOINT") else {
        eprintln!("skipped: set KANSO_SYNC_E2E_ENDPOINT (see cloud/scripts/e2e-local.sh)");
        return;
    };
    let backend = HttpBackend;
    let run = format!("{:x}", kanso_core::now_ms());
    let username = format!("e2eapprove{run}");
    let password = Password::new(&format!("{PASSWORD} approve"), &username).unwrap();
    let mac_a = identity(11, "Approver Mac");
    let mac_b = identity(12, "New Mac");
    let a = patiently(|| {
        kanso_sync::sign_up(
            &backend,
            &MemorySecrets::new(),
            &endpoint,
            &username,
            &password,
            &mac_a,
            &|_| {},
        )
        .map_err(|err| err.error)
    })
    .expect("sign up")
    .session;

    // Mac A's event loop, on its own thread, as the app runs it.
    let path = account_events_path(&backend, &endpoint)
        .expect("health")
        .unwrap_or_else(|| events::ACCOUNT_EVENTS_PATH.to_owned());
    let stop = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel::<Signal>();
    let (session_a, device_a, path) = (&a, &mac_a, path.as_str());
    std::thread::scope(|scope| {
        let events_thread = {
            let stop = &stop;
            scope.spawn(move || {
                let mut connects = 0;
                let mut connect = || -> Result<Box<dyn EventSource>, EventError> {
                    connects += 1;
                    let inner = HttpBackend.events(session_a, device_a, path)?;
                    Ok(if connects == 1 {
                        Box::new(DropsAfterFirstEvent {
                            inner,
                            dropped: false,
                        })
                    } else {
                        inner
                    })
                };
                let mut on = |signal: Signal| {
                    tx.send(signal).ok();
                };
                let sleep = |wait: Duration| events::sleep_unless_stopped(stop, wait);
                events::run(&mut connect, stop, &mut on, &sleep);
            })
        };
        let next_signal = |what: &str| -> Signal {
            rx.recv_timeout(Duration::from_secs(20))
                .unwrap_or_else(|_| panic!("no {what} within 20 s"))
        };
        let expect_connected = |what: &str| match next_signal(what) {
            Signal::Connected => {}
            other => panic!("{what}: expected Connected, got {other:?}"),
        };
        expect_connected("first connection");

        // Mac B signs in and, holding no data key, asks to be approved.
        let b = patiently(|| {
            kanso_sync::sign_in(&backend, &endpoint, &username, &password, &mac_b, &|_| {})
        })
        .expect("sign in")
        .session;
        let token = SessionToken::new(b.authorization().trim_start_matches("Bearer ").to_owned());
        let mut locked = LockedSession::new(
            b.server.clone(),
            b.origin.clone(),
            b.account_id.clone(),
            b.username.clone(),
            b.device_id.clone(),
            token,
            b.key_id(),
            "0000000000000000".to_owned(),
            None,
            None,
        )
        .with_login_device(LoginDevice::for_identity(&mac_b, &b.origin, &b.account_id));
        locked.set_library_id(&mac_b.library_id);
        let keys_b = ApprovalKeys::generate();
        let started = Instant::now();
        let request = request_approval(&backend, &locked, &keys_b).expect("ask");

        // A hears of it live, then its socket drops and comes back.
        match next_signal("approval.created") {
            Signal::Event(AccountEvent::ApprovalCreated(id)) => assert_eq!(id, request.id),
            other => panic!("expected approval.created, got {other:?}"),
        }
        eprintln!("approval.created after {:?}", started.elapsed());
        expect_connected("the reconnect after the drop");

        // A challenges; B shows the code and names itself; A hears that on the
        // new socket.
        let pending = pending_approvals(&backend, &a).expect("list");
        let row = pending
            .iter()
            .find(|row| row.id == request.id)
            .expect("the request is listed");
        let pin = PinnedRequest::of(row);
        let keys_a = ApprovalKeys::generate();
        challenge_request(&backend, &a, row, &keys_a).expect("challenge");
        let RequestState::Challenged { approver_epk, .. } =
            poll_approval(&backend, &locked, &request).expect("poll")
        else {
            panic!("B should be challenged");
        };
        let approval_b = complete_request(&locked, &request, &keys_b, &approver_epk).expect("code");
        name_device(&backend, &locked, &request, &approval_b, &mac_b).expect("name");
        loop {
            match next_signal("approval.named") {
                Signal::Event(AccountEvent::ApprovalNamed(id)) => {
                    assert_eq!(id, request.id);
                    break;
                }
                // A challenge isn't an event, but a late duplicate is harmless.
                Signal::Event(_) | Signal::Connected => {}
                Signal::Ended(err) => panic!("the event loop ended: {err:?}"),
            }
        }

        // The codes agree; A approves and B unlocks with the same data key.
        let pending = pending_approvals(&backend, &a).expect("list");
        let row = pending.iter().find(|row| row.id == request.id).unwrap();
        let opened = open_request(&a, row, &pin, &keys_a)
            .expect("open")
            .expect("named");
        assert_eq!(opened.approval.code(), approval_b.code());
        assert_eq!(opened.name, "New Mac");
        finish_approval(&backend, &a, row, &opened.approval).expect("approve");
        let RequestState::Approved(approved) =
            poll_approval(&backend, &locked, &request).expect("poll")
        else {
            panic!("B should be approved");
        };
        let unlocked =
            finish_unlock(&backend, &locked, &approval_b, &approved, &mac_b).expect("unlock");
        assert_eq!(unlocked.session.key_id(), a.key_id());
        eprintln!("live approval end to end: {:?}", started.elapsed());

        stop.store(true, Ordering::SeqCst);
        events_thread.join().expect("the event loop stops");
    });
}

/// The state directory of the stopped Worker (D1's SQLite files and R2's
/// blobs, everything under it) holds none of the canaries
/// `two_macs_sync_through_a_local_worker` listed, and does hold the live
/// witness account's username and sealed file, so the scan is known to read
/// both. `cloud/scripts/e2e-local.sh` runs it after `wrangler dev` exits.
#[test]
#[ignore = "run by cloud/scripts/e2e-local.sh once the Worker has stopped"]
fn the_stopped_state_holds_no_canary() {
    let (Ok(state), Ok(canaries)) = (
        std::env::var("KANSO_SYNC_E2E_STATE"),
        std::env::var("KANSO_SYNC_E2E_CANARIES"),
    ) else {
        panic!(
            "set KANSO_SYNC_E2E_STATE and KANSO_SYNC_E2E_CANARIES (see cloud/scripts/e2e-local.sh)"
        );
    };
    let list: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&canaries).expect("the canary list"))
            .unwrap();
    let unhex = |text: &serde_json::Value| -> Vec<u8> {
        let text = text.as_str().unwrap();
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    };
    let secrets: Vec<(String, Vec<u8>)> = list["secrets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| (pair[0].as_str().unwrap().to_owned(), unhex(&pair[1])))
        .collect();
    let deleted_files: Vec<Vec<u8>> = list["deleted_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(unhex)
        .collect();
    let witness_name = unhex(&list["witness_name"]);
    let witness_file = unhex(&list["witness_file"]);
    assert!(!secrets.is_empty() && !deleted_files.is_empty());

    let (mut saw_witness_name, mut saw_witness_file) = (false, false);
    let mut files = Vec::new();
    files_under(Path::new(&state), &mut files);
    assert!(!files.is_empty(), "{state} is empty");
    for path in &files {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        // Positive controls: D1's database stores usernames as they are,
        // and R2's blobs hold the witness's sealed file.
        saw_witness_name |= contains(&bytes, &witness_name);
        saw_witness_file |= contains(&bytes, &witness_file);
        for (what, secret) in &secrets {
            for form in encodings(secret) {
                assert!(!contains(&bytes, &form), "{what} is in {}", path.display());
            }
        }
        for middle in &deleted_files {
            assert!(
                !contains(&bytes, middle),
                "a deleted account's file is still in {}",
                path.display()
            );
        }
    }
    assert!(saw_witness_name, "the scan never read D1's database");
    assert!(saw_witness_file, "the scan never read R2's blobs");
    eprintln!(
        "no canary in {} files; the scan found the live witness in D1 and R2",
        files.len()
    );
}
