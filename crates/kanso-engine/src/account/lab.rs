//! A device for the tests of the account tasks: its library, its secret
//! store and its flow, over a shared in-memory server.

use std::sync::Arc;
use std::time::Duration;

use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::api::LoginDevice;
use kanso_sync::keychain;
use kanso_sync::{
    Api, Backend, DeviceIdentity, KdfParams, LockedSession, Machine, MemorySecrets, MemoryServer,
    Password, Session, SessionToken, sign_in,
};
use serde_json::json;

use super::Flow;

pub(crate) const SERVER: &str = kanso_sync::DEFAULT_SERVER;
pub(crate) const PASSWORD: &str = "correct horse battery staple";

/// The shared server, resending a request whose answer was lost at once.
pub(crate) struct Quick(pub Arc<MemoryServer>);

impl Backend for Quick {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        self.0.api(server)
    }
    fn kdf_floor(&self) -> KdfParams {
        self.0.kdf_floor()
    }
    fn kdf_default(&self) -> KdfParams {
        self.0.kdf_default()
    }
    fn retry_pause(&self, _: usize) -> Duration {
        Duration::ZERO
    }
}

/// One device: its folder, library, secret store and flow.
pub(crate) struct Lab {
    pub dir: tempfile::TempDir,
    pub store: Arc<Store>,
    pub secrets: Arc<MemorySecrets>,
    pub flow: Flow,
}

impl Lab {
    /// A device called `name` with `notes` notes, signed in nowhere.
    pub fn new(server: &Arc<MemoryServer>, name: &str, notes: usize) -> Lab {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        if notes > 0 {
            let book = store
                .put(
                    DocType::Book,
                    json!({"name": "Work"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            for n in 0..notes {
                store
                    .put(
                        DocType::Note,
                        json!({"title": format!("{name} note {n}"), "body": "text", "bookId": book}),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap();
            }
        }
        let secrets = Arc::new(MemorySecrets::new());
        let flow = Flow {
            backend: Arc::new(Quick(server.clone())),
            secrets: secrets.clone(),
            store: store.clone(),
            machine: Some(Machine {
                hardware_uuid: Some(format!("UUID-{name}")),
                computer_name: name.to_owned(),
            }),
            backups: dir.path().join("backups"),
            now_ms: 1_000,
        };
        Lab {
            dir,
            store,
            secrets,
            flow,
        }
    }
}

pub(crate) fn password(username: &str) -> Password {
    Password::new(PASSWORD, username).unwrap()
}

/// The new device of `alice`, locked and waiting to be approved, and
/// the unlocked first device that can approve it.
pub(crate) struct Paired {
    pub server: Arc<MemoryServer>,
    pub alice: Session,
    pub second: Lab,
    pub locked: LockedSession,
    pub device: DeviceIdentity,
    /// The new device's own session, from the password sign-in that
    /// stands in for a provider's.
    pub signed_in: Session,
}

pub(crate) fn paired() -> Paired {
    let server = Arc::new(MemoryServer::new());
    let first = Lab::new(&server, "Studio", 0);
    let (signed, _) = first
        .flow
        .create(SERVER, "alice", &password("alice"), &|_| {})
        .unwrap();
    let second = Lab::new(&server, "Laptop", 0);
    let device = second.flow.device().unwrap();
    let signed_in = sign_in(
        server.as_ref(),
        SERVER,
        "alice",
        &password("alice"),
        &device,
        &|_| {},
    )
    .unwrap()
    .session;
    let token = SessionToken::new(
        signed_in
            .authorization()
            .trim_start_matches("Bearer ")
            .to_owned(),
    );
    let mut locked = LockedSession::new(
        signed_in.server.clone(),
        signed_in.origin.clone(),
        signed_in.account_id.clone(),
        signed_in.username.clone(),
        signed_in.device_id.clone(),
        token,
        signed_in.key_id(),
        "0000000000000000".to_owned(),
        None,
        None,
    )
    .with_login_device(LoginDevice::for_identity(
        &device,
        &signed_in.origin,
        &signed_in.account_id,
    ));
    locked.set_library_id(&device.library_id);
    keychain::save_locked_pending(second.secrets.as_ref(), &device.library_id, &locked).unwrap();
    Paired {
        server,
        alice: signed.session,
        second,
        locked,
        device,
        signed_in,
    }
}
