//! The plaintext canary (docs/ACCOUNTS.md › Tests): every byte the client
//! sends (auth and account requests, pushes, file uploads, device names) is
//! captured, and no marker string, local ID, local revision, file MD5,
//! password, recovery key, data key or device name appears in it: raw, as
//! hex, or as base64 (standard and URL-safe, every alignment). CI pins the
//! seed with `PROPTEST_RNG_SEED`.

mod common;

use std::sync::Mutex;

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_sync::api::{
    AccountResponse, DeleteRequest, DeleteResponse, DeviceNameRequest, DevicesResponse, Health,
    LoginRequest, LoginResponse, PasswordRequest, PasswordResponse, PreloginRequest,
    PreloginResponse, RecoverFinishRequest, RecoverStartRequest, RecoverStartResponse,
    RecoveryKeyRequest, RecoveryKeyResponse, SessionResponse, SignupRequest,
};
use kanso_sync::wire::{PushRequest, PushResponse, PutFileResponse, WirePage};
use kanso_sync::{
    Api, ApiError, Backend, DeviceIdentity, KdfParams, MemorySecrets, MemoryServer, Password,
    SecretStore, SessionToken, keychain,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde::Serialize;
use serde_json::json;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A backend that records every request body (and file upload) on its way
/// to the server.
struct Capture<'a> {
    inner: &'a MemoryServer,
    sent: Mutex<Vec<u8>>,
}

impl Capture<'_> {
    fn note(&self, bytes: &[u8]) {
        let mut sent = self.sent.lock().unwrap();
        sent.extend_from_slice(bytes);
        sent.push(b'\n');
    }

    fn json(&self, body: &impl Serialize) {
        self.note(&serde_json::to_vec(body).unwrap());
    }
}

struct CaptureApi<'c> {
    capture: &'c Capture<'c>,
    inner: Box<dyn Api + 'c>,
}

impl Api for CaptureApi<'_> {
    fn health(&self) -> Result<Health, ApiError> {
        self.inner.health()
    }
    fn prelogin(&self, r: &PreloginRequest) -> Result<PreloginResponse, ApiError> {
        self.capture.json(r);
        self.inner.prelogin(r)
    }
    fn signup(&self, r: &SignupRequest) -> Result<SessionResponse, ApiError> {
        self.capture.json(r);
        self.inner.signup(r)
    }
    fn login(&self, r: &LoginRequest) -> Result<LoginResponse, ApiError> {
        self.capture.json(r);
        self.inner.login(r)
    }
    fn recover_start(&self, r: &RecoverStartRequest) -> Result<RecoverStartResponse, ApiError> {
        self.capture.json(r);
        self.inner.recover_start(r)
    }
    fn recover_finish(&self, r: &RecoverFinishRequest) -> Result<SessionResponse, ApiError> {
        self.capture.json(r);
        self.inner.recover_finish(r)
    }
    fn logout(&self, t: &SessionToken) -> Result<(), ApiError> {
        self.capture.note(t.expose().as_bytes());
        self.inner.logout(t)
    }
    fn account(&self, t: &SessionToken) -> Result<AccountResponse, ApiError> {
        self.inner.account(t)
    }
    fn change_password(
        &self,
        t: &SessionToken,
        r: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
        self.capture.json(r);
        self.inner.change_password(t, r)
    }
    fn replace_recovery_key(
        &self,
        t: &SessionToken,
        r: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError> {
        self.capture.json(r);
        self.inner.replace_recovery_key(t, r)
    }
    fn delete_account(
        &self,
        t: &SessionToken,
        r: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError> {
        self.capture.json(r);
        self.inner.delete_account(t, r)
    }
    fn set_device_name(&self, t: &SessionToken, r: &DeviceNameRequest) -> Result<(), ApiError> {
        self.capture.json(r);
        self.inner.set_device_name(t, r)
    }
    fn devices(&self, t: &SessionToken) -> Result<DevicesResponse, ApiError> {
        self.inner.devices(t)
    }
    fn revoke_device(&self, t: &SessionToken, device_id: &str) -> Result<(), ApiError> {
        self.capture.note(device_id.as_bytes());
        self.inner.revoke_device(t, device_id)
    }
    fn push(&self, t: &SessionToken, r: &PushRequest) -> Result<PushResponse, ApiError> {
        self.capture.json(r);
        self.inner.push(t, r)
    }
    fn changes(
        &self,
        t: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<WirePage, ApiError> {
        self.capture
            .note(format!("since={since:?}&limit={limit:?}").as_bytes());
        self.inner.changes(t, since, limit)
    }
    fn put_file(
        &self,
        t: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
    ) -> Result<PutFileResponse, ApiError> {
        self.capture.note(wire_id.as_bytes());
        self.capture.note(sealed);
        self.inner.put_file(t, wire_id, sealed)
    }
    fn get_file(&self, t: &SessionToken, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.capture.note(wire_id.as_bytes());
        self.inner.get_file(t, wire_id)
    }
}

impl Backend for Capture<'_> {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        Box::new(CaptureApi {
            capture: self,
            inner: self.inner.api(server),
        })
    }
    fn kdf_floor(&self) -> KdfParams {
        self.inner.kdf_floor()
    }
    fn kdf_default(&self) -> KdfParams {
        self.inner.kdf_default()
    }
}

proptest! {
    #![proptest_config(config(24))]

    #[test]
    fn prop_nothing_plain_leaves_the_mac(
        marker in "[a-z]{6}",
        words in prop::collection::vec("[a-z]{3,8}", 1..4),
        bytes in prop::collection::vec(any::<u8>(), 8..64),
        notes in 1..4usize,
        secret in any::<[u8; 32]>(),
    ) {
        let marker = format!("zq{marker}canary");
        let server = MemoryServer::new();
        let capture = Capture { inner: &server, sent: Mutex::new(Vec::new()) };
        let username = "canary";
        let password_text = format!("{PASSWORD} {marker}");
        let password = Password::new(&password_text, username).unwrap();
        let device_name = format!("Studio of {marker}");
        let device = DeviceIdentity::from_parts(&format!("l_{:032x}", 7), &device_name, secret);
        let other = DeviceIdentity::from_parts(&format!("l_{:032x}", 8), &format!("Laptop {marker}"), [9; 32]);

        let signed_up = kanso_sync::sign_up(&capture, SERVER, username, &password, &device, &|_| {}).unwrap();
        let recovery = signed_up.recovery_key;
        let a = Mac::new(signed_up.session);
        let b = Mac::new(kanso_sync::sign_in(&capture, SERVER, username, &password, &other, &|_| {}).unwrap().session);

        // Content full of markers: notebook, tag, notes and a file.
        let book_id = format!("book:{marker}b");
        let tag_id = format!("tag:{marker}t");
        let file_id = format!("file:{marker}f");
        put(&a.store, DocType::Book, json!({"_id": book_id, "name": format!("{marker} book"), "createdAt": 1, "updatedAt": 1})).unwrap();
        put(&a.store, DocType::Tag, json!({"_id": tag_id, "name": format!("{marker} tag"), "createdAt": 1, "updatedAt": 1})).unwrap();
        let mut note_ids = Vec::new();
        for i in 0..notes {
            let id = format!("note:{marker}n{i}");
            put(&a.store, DocType::Note, json!({"_id": id, "bookId": book_id, "tags": [tag_id], "title": format!("{} {marker}", words.join(" ")),
                "body": format!("# {marker}\n{}", words.join("\n")), "createdAt": 1, "updatedAt": 1})).unwrap();
            note_ids.push(id);
        }
        let mut file_bytes = marker.as_bytes().to_vec();
        file_bytes.extend_from_slice(&bytes);
        put(&a.store, DocType::File, file_body(&file_id, &format!("{marker}.png"), &file_bytes)).unwrap();
        a.store.delete(&note_ids[0], kanso_store::Origin::Api).unwrap();
        a.sync(&capture);
        b.sync(&capture);
        if notes > 1 {
            edit_note(&b.store, note_ids.last().unwrap(), &format!("edited {marker}"), Some(5));
        } else {
            merge(&b.store, DocType::Book, json!({"_id": book_id, "name": format!("renamed {marker}")})).unwrap();
        }
        b.sync(&capture);
        a.sync(&capture);
        // Account requests carry wrapped keys and proofs.
        let new_password = Password::new(&format!("{PASSWORD} new {marker}"), username).unwrap();
        kanso_sync::change_password(&capture, &a.session, &password, &new_password, &|_| {}).unwrap();
        let new_recovery = kanso_sync::new_recovery_key(&capture, &a.session, &new_password, &|_| {}).unwrap();

        let secrets = MemorySecrets::new();
        keychain::save_session(&secrets, "l_canary", &a.session).unwrap();
        let item: serde_json::Value = serde_json::from_str(&secrets.get(&keychain::account_item("l_canary")).unwrap().unwrap()).unwrap();
        let data_key = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(item["data_key"].as_str().unwrap()).unwrap();
        prop_assert_eq!(data_key.len(), 32);

        // What must never leave the Mac.
        let mut secrets_list: Vec<(String, Vec<u8>)> = vec![
            ("marker".into(), marker.clone().into_bytes()),
            ("password".into(), password_text.clone().into_bytes()),
            ("new password".into(), format!("{PASSWORD} new {marker}").into_bytes()),
            ("device name".into(), device_name.clone().into_bytes()),
            ("recovery key".into(), recovery.display().into_bytes()),
            ("new recovery key".into(), new_recovery.display().into_bytes()),
            ("data key".into(), data_key),
            ("device secret".into(), secret.to_vec()),
        ];
        for doc in a.store.sync_docs().unwrap().into_iter().chain(b.store.sync_docs().unwrap()) {
            let random = doc.id.split_once(':').unwrap().1.to_owned();
            secrets_list.push((format!("id {}", doc.id), random.into_bytes()));
            secrets_list.push((format!("rev {}", doc.rev), doc.rev.clone().into_bytes()));
            secrets_list.push((format!("rev hash {}", doc.rev), doc.rev.split_once('-').unwrap().1.as_bytes().to_vec()));
            for revision in a.store.revisions(&doc.id).unwrap() {
                secrets_list.push((format!("old rev {}", revision.rev), revision.rev.split_once('-').unwrap().1.as_bytes().to_vec()));
            }
        }
        let (_, blob) = a.store.attachment(&file_id).unwrap().unwrap();
        secrets_list.push(("file md5".into(), blob.md5_hex().into_bytes()));
        secrets_list.push(("file md5 bytes".into(), blob.md5.to_vec()));
        secrets_list.push(("file bytes".into(), file_bytes.clone()));

        let sent = capture.sent.lock().unwrap();
        prop_assert!(sent.len() > 1000, "the capture saw the traffic");
        for (what, secret) in &secrets_list {
            for form in encodings(secret) {
                prop_assert!(!contains(&sent, &form), "{} left the Mac as {:?}", what, String::from_utf8_lossy(&form));
            }
        }
    }
}

#[test]
fn the_encodings_find_a_secret_wherever_it_sits() {
    let secret = b"a secret value";
    for prefix in 0..4usize {
        let mut data = vec![b'x'; prefix];
        data.extend_from_slice(secret);
        data.extend_from_slice(b"tail");
        for engine in [
            base64::engine::general_purpose::STANDARD,
            base64::engine::general_purpose::URL_SAFE_NO_PAD,
        ] {
            let encoded = engine.encode(&data);
            assert!(
                encodings(secret)
                    .iter()
                    .any(|f| contains(encoded.as_bytes(), f)),
                "missed at offset {prefix}"
            );
        }
        let hex: String = data.iter().map(|b| format!("{b:02x}")).collect();
        assert!(
            encodings(secret)
                .iter()
                .any(|f| contains(hex.as_bytes(), f))
        );
    }
}
