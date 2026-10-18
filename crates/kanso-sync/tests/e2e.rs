//! End to end against a local Worker (docs/ACCOUNTS.md › Tests › End to end).
//! `cloud/scripts/e2e-local.sh` starts `wrangler dev --local` with sign-ups
//! open and runs:
//!
//! ```text
//! KANSO_SYNC_E2E_ENDPOINT=http://127.0.0.1:8787 KANSO_SYNC_E2E_STATE=$dir \
//!     cargo test -p kanso-sync --test e2e -- --nocapture
//! ```
//!
//! It runs at production KDF parameters (`HttpBackend`) and passes without
//! doing anything when `KANSO_SYNC_E2E_ENDPOINT` is unset. Sessions stay in
//! memory; nothing touches the Keychain.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::*;
use kanso_store::Origin;
use kanso_sync::wire::{
    MAX_CHANGES_LIMIT, MAX_PUSH_BODY, MAX_PUSH_DOCS, WirePage, WirePushDoc, WirePushResult,
    WireRemote,
};
use kanso_sync::{
    AccountError, Api, ApiError, Backend, DeviceIdentity, HttpBackend, KdfParams, Password,
    Session, SignOutReason, SyncError, SyncOptions,
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

/// Runs an account call, waiting out the auth front filter (20 requests a
/// minute per IP) when it answers 429.
fn patiently<T>(call: impl Fn() -> Result<T, AccountError>) -> Result<T, AccountError> {
    for _ in 0..3 {
        match call() {
            Err(AccountError::RateLimited { retry_after }) => {
                eprintln!("rate limited; waiting {retry_after} s");
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

#[test]
fn two_macs_sync_through_a_local_worker() {
    let Ok(endpoint) = std::env::var("KANSO_SYNC_E2E_ENDPOINT") else {
        eprintln!("skipped: set KANSO_SYNC_E2E_ENDPOINT (see cloud/scripts/e2e-local.sh)");
        return;
    };
    let backend = Recording {
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
            &endpoint,
            &username,
            &password,
            &device_a,
            &|_| {},
        )
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

    // 2. A password change gives the other device 401 password_changed.
    let second = Password::new(&format!("{PASSWORD} two {marker}"), &username).unwrap();
    patiently(|| kanso_sync::change_password(&backend, &a.session, &password, &second, &|_| {}))
        .expect("change password");
    match b.try_sync(&backend) {
        Err(SyncError::Api(ApiError::Unauthorized { reason })) => {
            assert_eq!(reason, SignOutReason::PasswordChanged)
        }
        other => panic!("expected 401 password_changed, got {other:?}"),
    }

    // 3. Recovery rotates the key: the used one gets 401, and so does
    //    every session from before it.
    let third = Password::new(&format!("{PASSWORD} three {marker}"), &username).unwrap();
    let recover = || {
        patiently(|| {
            kanso_sync::recover(
                &backend,
                &endpoint,
                &username,
                &recovery_key,
                &third,
                &device_b,
                &|_| {},
            )
        })
    };
    let recovered = recover().expect("recover");
    assert_ne!(recovered.recovery_key, recovery_key);
    let again = recover();
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

    // 4. Wrong passwords end in 429 (login-user: 10 in 15 minutes).
    let stranger = identity(9, "Stranger");
    let wrong = Password::new("definitely not the password", &username).unwrap();
    let mut limited = false;
    for _ in 0..15 {
        match kanso_sync::sign_in(&backend, &endpoint, &username, &wrong, &stranger, &|_| {}) {
            Err(AccountError::InvalidCredentials) => {}
            Err(AccountError::RateLimited { .. }) => {
                limited = true;
                break;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    assert!(limited, "wrong passwords never hit the limit");

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

    // 7. Every file under the state directory (D1's SQLite and R2's blobs)
    //    is scanned for the canary markers.
    let Ok(state) = std::env::var("KANSO_SYNC_E2E_STATE") else {
        eprintln!("KANSO_SYNC_E2E_STATE unset: no on-disk canary scan");
        return;
    };
    let mut secrets: Vec<(String, Vec<u8>)> = vec![
        ("the marker".into(), marker.clone().into_bytes()),
        ("the password".into(), password_text.into_bytes()),
        (
            "the recovery key".into(),
            recovery_key.display().into_bytes(),
        ),
    ];
    for doc in a.store.sync_docs().unwrap() {
        let hash = doc.rev.split_once('-').unwrap().1.to_owned();
        secrets.push((format!("the revision {}", doc.rev), hash.into_bytes()));
    }
    let uploads = backend.uploads.lock().unwrap();
    let mut files = Vec::new();
    files_under(Path::new(&state), &mut files);
    assert!(!files.is_empty(), "{state} is empty");
    for path in files {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        for (what, secret) in &secrets {
            for form in encodings(secret) {
                assert!(!contains(&bytes, &form), "{what} is in {}", path.display());
            }
        }
        for sealed in uploads.iter().filter(|s| s.len() >= 64) {
            let middle = &sealed[sealed.len() / 2..sealed.len() / 2 + 32];
            assert!(
                !contains(&bytes, middle),
                "a deleted account's file is still in {}",
                path.display()
            );
        }
    }
    eprintln!("no canary on disk");
}
