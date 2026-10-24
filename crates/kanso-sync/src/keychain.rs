//! Secrets in the macOS Keychain (docs/ACCOUNTS.md › Keychain items).
//! The old app's `sync-token` item is never read, written or deleted here;
//! the owner removes it after the legacy purge. A `sync-pending` item holds
//! only the server session metadata for a session whose data key is not yet
//! available; it never contains a data key.
//!
//! The service is `Kanso`; item names carry the library ID, so data
//! directories never share secrets. [`SecretStore`] is the seam: [`Keychain`]
//! in the app, [`MemorySecrets`] in tests and snapshots.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::account::Session;
use crate::api::{LoginDevice, WireDevice};
use crate::approvals::{ApprovalResume, LockedSession};
use crate::crypto::{DataKey, RecoveryKey, b64, unb64};
use crate::wire::{
    SessionToken, is_account_id, is_approval_id, is_base64url, is_device_id, is_hex16,
    is_sealed_name, is_token,
};

/// The Keychain service of every Kanso item.
pub const SERVICE: &str = "Kanso";

/// `sync-account:<library_id>`: `{"v":1,"server","origin","account_id",
/// "username","device_id","token","data_key"}`, from sign-up, sign-in or
/// recovery until sign-out, deletion, a 401 or unbind.
pub fn account_item(library_id: &str) -> String {
    format!("sync-account:{library_id}")
}

/// `sync-pending:<library_id>`: a versioned server session awaiting
/// local-key, password or device-approval unlock. It never contains a data key.
pub fn pending_item(library_id: &str) -> String {
    format!("sync-pending:{library_id}")
}

/// `sync-recovery-key:<library_id>`: `{"v":1,"origin","account_id","username",
/// "recovery_key"}`, from the moment sign-up, recovery or "New recovery key"
/// succeeds until "I've saved it", or sign-out.
pub fn recovery_item(library_id: &str) -> String {
    format!("sync-recovery-key:{library_id}")
}

/// `sync-logout:<library_id>`: `{"v":1,"pending":[{"server","token"}]}`,
/// until each logout succeeds.
pub fn logout_item(library_id: &str) -> String {
    format!("sync-logout:{library_id}")
}

/// `lock-keys:<library_id>`: `{"v":1,"keys":[…]}`, the lock keys (not the
/// password) for unlocking locked notes with Touch ID on this device, from
/// turning Touch ID on until turning it off or changing the lock password.
pub fn lock_keys_item(library_id: &str) -> String {
    format!("lock-keys:{library_id}")
}

/// A Keychain failure.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("keychain: {0}")]
pub struct SecretError(pub String);

/// Where secrets live. `Send + Sync`: the app shares it as `Arc<dyn SecretStore>`.
pub trait SecretStore: Send + Sync {
    /// The item's value, or `None` when there is no such item.
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError>;
    /// Creates or replaces the item.
    fn set(&self, item: &str, value: &str) -> Result<(), SecretError>;
    /// Removes the item; succeeds when there is none.
    fn delete(&self, item: &str) -> Result<(), SecretError>;
}

/// The macOS Keychain, service [`SERVICE`] (or another, for Kanso Dev):
/// generic passwords whose account is the item name. Tests never construct
/// it; they use [`MemorySecrets`].
#[derive(Clone, Copy, Debug)]
pub struct Keychain {
    service: &'static str,
}

impl Default for Keychain {
    fn default() -> Self {
        Keychain::with_service(SERVICE)
    }
}

// no-prop-test: a thin wrapper over the system Keychain, which tests must
// never touch; the item formats above it are tested through `MemorySecrets`.
impl Keychain {
    /// The Keychain under `service`: a separate build (Kanso Dev) keeps
    /// its secrets apart from the release's.
    pub const fn with_service(service: &'static str) -> Keychain {
        Keychain { service }
    }

    pub fn service(&self) -> &'static str {
        self.service
    }

    fn entry(&self, item: &str) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(self.service, item).map_err(keychain_error)
    }
}

fn keychain_error(err: keyring::Error) -> SecretError {
    SecretError(err.to_string())
}

impl SecretStore for Keychain {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        match self.entry(item)?.get_password() {
            Ok(value) => Ok(Some(Zeroizing::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(keychain_error(err)),
        }
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        self.entry(item)?
            .set_password(value)
            .map_err(keychain_error)
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        match self.entry(item)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(keychain_error(err)),
        }
    }
}

/// Secrets held in memory, for tests and snapshots. Writes can be made to
/// fail, as a locked Keychain or a denied access prompt makes them.
#[derive(Default)]
pub struct MemorySecrets {
    items: Mutex<BTreeMap<String, Zeroizing<String>>>,
    faults: Mutex<Faults>,
}

/// Which writes of a [`MemorySecrets`] fail.
#[derive(Default)]
struct Faults {
    /// Writes (sets and deletes) tried so far.
    writes: usize,
    /// The numbers of the writes that fail, counted from the first.
    fail_at: std::collections::BTreeSet<usize>,
    /// Every write to an item whose name starts with this fails.
    prefix: Option<String>,
    /// Every read fails.
    reads: bool,
}

impl MemorySecrets {
    pub fn new() -> MemorySecrets {
        MemorySecrets::default()
    }

    /// The names of the stored items, sorted.
    pub fn items(&self) -> Vec<String> {
        self.items
            .lock()
            .expect("secrets poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Makes the write `n` writes from now fail once (0: the next one).
    pub fn fail_write(&self, n: usize) {
        let mut faults = self.faults.lock().expect("secrets poisoned");
        let at = faults.writes + n;
        faults.fail_at.insert(at);
    }

    /// Makes every write (set or delete) to an item whose name starts with
    /// `prefix` fail, until called again with `None`.
    pub fn fail_writes_to(&self, prefix: Option<&str>) {
        self.faults.lock().expect("secrets poisoned").prefix = prefix.map(str::to_owned);
    }

    /// Makes every read fail (or work again).
    pub fn fail_reads(&self, fail: bool) {
        self.faults.lock().expect("secrets poisoned").reads = fail;
    }

    /// Every write and read works again.
    pub fn heal(&self) {
        let mut faults = self.faults.lock().expect("secrets poisoned");
        faults.fail_at.clear();
        faults.prefix = None;
        faults.reads = false;
    }

    /// How many writes were tried so far, failed ones included.
    pub fn writes(&self) -> usize {
        self.faults.lock().expect("secrets poisoned").writes
    }

    /// Counts a write to `item`, and says whether it fails.
    fn write_fails(&self, item: &str) -> bool {
        let mut faults = self.faults.lock().expect("secrets poisoned");
        let at = faults.writes;
        faults.writes += 1;
        let by_number = faults.fail_at.remove(&at);
        by_number
            || faults
                .prefix
                .as_deref()
                .is_some_and(|p| item.starts_with(p))
    }
}

/// The error a failing [`MemorySecrets`] gives.
fn refused() -> SecretError {
    SecretError("the Keychain refused (simulated)".into())
}

impl SecretStore for MemorySecrets {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        if self.faults.lock().expect("secrets poisoned").reads {
            return Err(refused());
        }
        Ok(self
            .items
            .lock()
            .expect("secrets poisoned")
            .get(item)
            .cloned())
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        if self.write_fails(item) {
            return Err(refused());
        }
        self.items
            .lock()
            .expect("secrets poisoned")
            .insert(item.to_owned(), Zeroizing::new(value.to_owned()));
        Ok(())
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        if self.write_fails(item) {
            return Err(refused());
        }
        self.items.lock().expect("secrets poisoned").remove(item);
        Ok(())
    }
}

/// The `sync-account` item's JSON.
#[derive(Serialize, Deserialize)]
struct AccountItem {
    v: u32,
    server: String,
    origin: String,
    account_id: String,
    username: String,
    device_id: String,
    token: SessionToken,
    /// 43 base64url characters.
    #[serde(with = "zeroizing_string")]
    data_key: Zeroizing<String>,
}

/// The v2 `sync-pending` item. It contains the original device proof and raw
/// sealed eviction/approval data, but never a data key or derived approval key.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingSessionItem {
    v: u32,
    server: String,
    origin: String,
    account_id: String,
    username: String,
    device_id: String,
    #[serde(with = "zeroizing_string")]
    device_proof: Zeroizing<String>,
    token: SessionToken,
    key_id: String,
    epoch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evicted_device: Option<PendingEvictedDevice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approval: Option<PendingApprovalResume>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingEvictedDevice {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sealed_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingApprovalResume {
    request_id: String,
    #[serde(with = "zeroizing_string")]
    requester_secret: Zeroizing<String>,
    expires_at: i64,
    other_devices: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approver_epk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approver_device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sealed_name: Option<String>,
}

impl PendingSessionItem {
    fn from_locked(locked: &LockedSession) -> Result<PendingSessionItem, SecretError> {
        let Some(login_device) = locked.login_device() else {
            return Err(invalid_pending());
        };
        if login_device.id != locked.device_id {
            return Err(invalid_pending());
        }
        let evicted = locked
            .evicted_wire_device()
            .map(|device| PendingEvictedDevice {
                id: device.id.clone(),
                sealed_name: device.sealed_name.clone(),
            })
            .or_else(|| {
                locked.evicted.as_ref().map(|device| PendingEvictedDevice {
                    id: device.id.clone(),
                    sealed_name: locked.evicted_sealed_name().map(str::to_owned),
                })
            });
        let approval = locked
            .approval_resume()
            .map(|resume| PendingApprovalResume {
                request_id: resume.request_id.clone(),
                requester_secret: Zeroizing::new(b64(resume.requester_secret())),
                expires_at: resume.expires_at,
                other_devices: resume.other_devices,
                approver_epk: resume.approver_epk.map(|epk| b64(&epk)),
                approver_device_id: resume.approver_device_id.clone(),
                sealed_name: resume.sealed_name.clone(),
            });
        Ok(PendingSessionItem {
            v: 2,
            server: locked.server.clone(),
            origin: locked.origin.clone(),
            account_id: locked.account_id.clone(),
            username: locked.username.clone(),
            device_id: locked.device_id.clone(),
            device_proof: Zeroizing::new(login_device.proof.clone()),
            token: locked.token().clone(),
            key_id: locked.key_id.clone(),
            epoch: locked.epoch.clone(),
            evicted_device: evicted,
            approval,
        })
    }

    fn into_locked(self, library_id: &str) -> Option<LockedSession> {
        if !valid_pending(&self, library_id) {
            return None;
        }
        let login_device = LoginDevice {
            id: self.device_id.clone(),
            proof: self.device_proof.to_string(),
        };
        let raw_evicted = self.evicted_device.map(|device| WireDevice {
            id: device.id,
            sealed_name: device.sealed_name,
        });
        let approval = self.approval.and_then(|resume| {
            let requester_secret = decode_secret32(&resume.requester_secret)?;
            Some(ApprovalResume::from_parts(
                resume.request_id,
                requester_secret,
                resume.expires_at,
                resume.other_devices,
                resume
                    .approver_epk
                    .as_deref()
                    .and_then(crate::approval::parse_epk),
                resume.approver_device_id,
                resume.sealed_name,
            ))
        });
        let mut locked = LockedSession::new(
            self.server,
            self.origin,
            self.account_id,
            self.username,
            self.device_id,
            self.token,
            self.key_id,
            self.epoch,
            None,
            None,
        )
        .with_login_device(login_device)
        .with_evicted_wire_device(raw_evicted);
        if let Some(approval) = approval {
            locked = locked.with_approval_resume(approval);
        }
        locked.set_library_id(library_id);
        Some(locked)
    }
}

fn invalid_pending() -> SecretError {
    SecretError("the locked-session item is invalid".into())
}

fn decode_secret32(value: &str) -> Option<[u8; 32]> {
    if !is_base64url(value) {
        return None;
    }
    let mut decoded = unb64(value)?;
    let canonical = b64(&decoded) == value;
    let secret = canonical
        .then(|| <[u8; 32]>::try_from(decoded.as_slice()).ok())
        .flatten();
    decoded.zeroize();
    secret
}

fn is_secret32(value: &str) -> bool {
    decode_secret32(value).is_some()
}

fn is_canonical_base64url(value: &str) -> bool {
    is_base64url(value) && unb64(value).is_some_and(|decoded| b64(&decoded) == value)
}

fn is_canonical_approval_id(value: &str) -> bool {
    is_approval_id(value) && is_secret32(value)
}

fn is_canonical_epk(value: &str) -> bool {
    crate::approval::parse_epk(value).is_some_and(|epk| b64(&epk) == value)
}

fn is_canonical_sealed_name(value: &str) -> bool {
    is_sealed_name(value) && is_canonical_base64url(value)
}

fn valid_pending(item: &PendingSessionItem, library_id: &str) -> bool {
    let valid_library_id = library_id.strip_prefix("l_").is_some_and(|hex| {
        hex.len() == 32
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    });
    let device_proof = item.device_proof.as_str();
    let base = item.v == 2
        && valid_library_id
        && crate::check_endpoint(&item.server).is_ok()
        && crate::origin(&item.server).as_deref() == Ok(item.origin.as_str())
        && crate::origin(&item.origin).as_deref() == Ok(item.origin.as_str())
        && is_account_id(&item.account_id)
        && is_device_id(&item.device_id)
        && is_token(item.token.expose())
        && item
            .token
            .expose()
            .strip_prefix("kanso_")
            .is_some_and(is_secret32)
        && crate::normalize_username(&item.username).is_ok_and(|name| name == item.username)
        && is_secret32(device_proof)
        && is_hex16(&item.key_id)
        && is_hex16(&item.epoch);
    if !base {
        return false;
    }
    if let Some(device) = &item.evicted_device
        && (!is_device_id(&device.id)
            || device
                .sealed_name
                .as_deref()
                .is_some_and(|sealed| !is_canonical_sealed_name(sealed)))
    {
        return false;
    }
    if let Some(approval) = &item.approval
        && (!is_canonical_approval_id(&approval.request_id)
            || approval.expires_at < 0
            || approval.other_devices > crate::wire::MAX_ACTIVE_DEVICES
            || !is_secret32(&approval.requester_secret)
            || approval
                .approver_epk
                .as_deref()
                .is_some_and(|epk| !is_canonical_epk(epk))
            || approval
                .approver_device_id
                .as_deref()
                .is_some_and(|id| !is_device_id(id))
            || approval
                .sealed_name
                .as_deref()
                .is_some_and(|sealed| !is_canonical_sealed_name(sealed))
            || approval.approver_epk.is_some() != approval.approver_device_id.is_some()
            || (approval.sealed_name.is_some() && approval.approver_epk.is_none()))
    {
        return false;
    }
    true
}

/// Saves a locked session as `sync-pending:<library_id>`, without a data key.
pub fn save_locked_pending(
    secrets: &dyn SecretStore,
    library_id: &str,
    locked: &LockedSession,
) -> Result<(), SecretError> {
    if locked.library_id().is_some_and(|saved| saved != library_id) {
        return Err(invalid_pending());
    }
    let item = PendingSessionItem::from_locked(locked)?;
    if !valid_pending(&item, library_id) {
        return Err(invalid_pending());
    }
    let json = Zeroizing::new(serde_json::to_string(&item).map_err(|_| invalid_pending())?);
    secrets.set(&pending_item(library_id), &json)
}

/// Updates an existing pending record for the same locked identity.
///
/// The token may change when password unlock or recovery issues a fresh
/// token; server, origin, account, username, device, key ID, epoch, and login
/// device remain fixed so this operation cannot replace the pending identity.
pub fn update_locked_pending(
    secrets: &dyn SecretStore,
    library_id: &str,
    locked: &LockedSession,
) -> Result<(), SecretError> {
    let Some(current) = load_locked_pending(secrets, library_id)? else {
        return Err(invalid_pending());
    };
    if current.library_id() != locked.library_id()
        || locked.library_id() != Some(library_id)
        || current.server != locked.server
        || current.origin != locked.origin
        || current.account_id != locked.account_id
        || current.username != locked.username
        || current.device_id != locked.device_id
        || current.key_id != locked.key_id
        || current.epoch != locked.epoch
        || current.login_device() != locked.login_device()
    {
        return Err(invalid_pending());
    }
    save_locked_pending(secrets, library_id, locked)
}

/// Loads and self-validates `sync-pending:<library_id>` without relying on
/// the library's config binding, which may not exist before unlock.
pub fn load_locked_pending(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<Option<LockedSession>, SecretError> {
    let Some(json) = secrets.get(&pending_item(library_id))? else {
        return Ok(None);
    };
    let json = Zeroizing::new(json);
    let Ok(item) = serde_json::from_str::<PendingSessionItem>(&json) else {
        return Ok(None);
    };
    Ok(item.into_locked(library_id))
}

/// The library's locked session, if its item is valid and names this
/// `(origin, account_id)`. Malformed or foreign items count as none.
pub fn load_pending(
    secrets: &dyn SecretStore,
    library_id: &str,
    origin: &str,
    account_id: &str,
) -> Result<Option<LockedSession>, SecretError> {
    Ok(load_locked_pending(secrets, library_id)?
        .filter(|locked| locked.origin == origin && locked.account_id == account_id))
}

/// Clears only the approval resume object, retaining the locked token/session.
pub fn clear_pending_approval(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<(), SecretError> {
    let Some(json) = secrets.get(&pending_item(library_id))? else {
        return Ok(());
    };
    let json = Zeroizing::new(json);
    let Ok(mut item) = serde_json::from_str::<PendingSessionItem>(&json) else {
        return Ok(());
    };
    if !valid_pending(&item, library_id) || item.approval.is_none() {
        return Ok(());
    }
    item.approval = None;
    let json = Zeroizing::new(serde_json::to_string(&item).map_err(|_| invalid_pending())?);
    secrets.set(&pending_item(library_id), &json)
}

/// Removes `sync-pending:<library_id>`.
pub fn delete_locked_pending(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<(), SecretError> {
    secrets.delete(&pending_item(library_id))
}

/// Compatibility names used by the in-progress app integration.
pub use delete_locked_pending as delete_pending;
pub use delete_locked_pending as delete_pending_session;
pub use load_locked_pending as load_pending_record;
pub use save_locked_pending as save_pending;

pub use load_pending as load_pending_session;
pub use save_locked_pending as save_pending_session;

/// Serde for a `Zeroizing<String>` (zeroize's `serde` feature is off).
mod zeroizing_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use zeroize::Zeroizing;

    pub fn serialize<S: Serializer>(
        value: &Zeroizing<String>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Zeroizing<String>, D::Error> {
        String::deserialize(deserializer).map(Zeroizing::new)
    }
}

/// Saves the session as `sync-account:<library_id>`.
pub fn save_session(
    secrets: &dyn SecretStore,
    library_id: &str,
    session: &Session,
) -> Result<(), SecretError> {
    let item = AccountItem {
        v: 1,
        server: session.server.clone(),
        origin: session.origin.clone(),
        account_id: session.account_id.clone(),
        username: session.username.clone(),
        device_id: session.device_id.clone(),
        token: session.token().clone(),
        data_key: Zeroizing::new(b64(session.data_key().bytes())),
    };
    let json = Zeroizing::new(serde_json::to_string(&item).expect("the item serializes"));
    secrets.set(&account_item(library_id), &json)
}

/// The library's session, if its item exists and names this `(origin,
/// account_id)`: both must equal the library's binding and the config's
/// account (security re-check 3). Any other item is ignored, and the Mac
/// counts as signed out.
pub fn load_session(
    secrets: &dyn SecretStore,
    library_id: &str,
    origin: &str,
    account_id: &str,
) -> Result<Option<Session>, SecretError> {
    let Some(json) = secrets.get(&account_item(library_id))? else {
        return Ok(None);
    };
    let Ok(item) = serde_json::from_str::<AccountItem>(&json) else {
        return Ok(None);
    };
    let data_key = unb64(&item.data_key)
        .map(Zeroizing::new)
        .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok());
    let valid = item.v == 1
        && item.origin == origin
        && item.account_id == account_id
        && is_account_id(&item.account_id)
        && is_device_id(&item.device_id)
        && is_token(item.token.expose())
        && crate::origin(&item.server).as_deref() == Ok(origin);
    match data_key {
        Some(data_key) if valid => {
            let mut session = Session::new(
                item.server,
                item.origin,
                item.account_id,
                item.username,
                item.device_id,
                item.token,
                DataKey::from_bytes(data_key),
            );
            session.set_library_id(library_id);
            Ok(Some(session))
        }
        _ => Ok(None),
    }
}

/// Removes `sync-account:<library_id>`.
pub fn delete_session(secrets: &dyn SecretStore, library_id: &str) -> Result<(), SecretError> {
    secrets.delete(&account_item(library_id))
}

/// Removes `sync-account:<library_id>` after the server ended `ended`,
/// unless the item holds a newer session of the same account by now: one
/// a recovery on this Mac saved meanwhile (review-verify 3).
pub fn delete_ended_session(
    secrets: &dyn SecretStore,
    library_id: &str,
    ended: &Session,
) -> Result<(), SecretError> {
    if let Some(saved) = load_session(secrets, library_id, &ended.origin, &ended.account_id)?
        && saved.token() != ended.token()
    {
        return Ok(());
    }
    delete_session(secrets, library_id)
}

/// A recovery key the user hasn't confirmed saving yet.
#[derive(Clone, Debug)]
pub struct PendingRecoveryKey {
    pub origin: String,
    pub account_id: String,
    pub username: String,
    /// The server's URL, for the key screen and the key's file (items saved
    /// before it was kept carry the origin here).
    pub server: String,
    pub recovery_key: RecoveryKey,
    /// The request that sets this key was sent, and its answer never came:
    /// the server may or may not hold it. It is saved this way before the
    /// request leaves (sec-client 1), and turns false once the server says
    /// yes.
    pub unconfirmed: bool,
}

/// The `sync-recovery-key` item's JSON; the key as it is shown.
#[derive(Serialize, Deserialize)]
struct RecoveryItem {
    v: u32,
    origin: String,
    account_id: String,
    username: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    server: String,
    #[serde(with = "zeroizing_string")]
    recovery_key: Zeroizing<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    unconfirmed: bool,
}

/// Saves `sync-recovery-key:<library_id>`: marked unconfirmed before the
/// request that sets the key is sent, then confirmed once the server says
/// yes.
pub fn save_recovery_key(
    secrets: &dyn SecretStore,
    library_id: &str,
    key: &PendingRecoveryKey,
) -> Result<(), SecretError> {
    let item = RecoveryItem {
        v: 1,
        origin: key.origin.clone(),
        account_id: key.account_id.clone(),
        username: key.username.clone(),
        server: key.server.clone(),
        recovery_key: Zeroizing::new(key.recovery_key.display()),
        unconfirmed: key.unconfirmed,
    };
    let json = Zeroizing::new(serde_json::to_string(&item).expect("the item serializes"));
    secrets.set(&recovery_item(library_id), &json)
}

/// The library's unsaved recovery key, whichever account it is for, if
/// any. A damaged item counts as none.
pub fn load_any_recovery_key(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<Option<PendingRecoveryKey>, SecretError> {
    let Some(json) = secrets.get(&recovery_item(library_id))? else {
        return Ok(None);
    };
    let Ok(item) = serde_json::from_str::<RecoveryItem>(&json) else {
        return Ok(None);
    };
    if item.v != 1 {
        return Ok(None);
    }
    Ok(RecoveryKey::parse(&item.recovery_key)
        .ok()
        .map(|recovery_key| PendingRecoveryKey {
            server: if item.server.is_empty() {
                item.origin.clone()
            } else {
                item.server
            },
            origin: item.origin,
            account_id: item.account_id,
            username: item.username,
            recovery_key,
            unconfirmed: item.unconfirmed,
        }))
}

/// The unconfirmed recovery key for this `(origin, account_id)`, if any:
/// while it exists the Account page shows the key screen again. An item for
/// another account is ignored.
pub fn load_recovery_key(
    secrets: &dyn SecretStore,
    library_id: &str,
    origin: &str,
    account_id: &str,
) -> Result<Option<PendingRecoveryKey>, SecretError> {
    Ok(load_any_recovery_key(secrets, library_id)?
        .filter(|key| key.origin == origin && key.account_id == account_id))
}

/// Removes `sync-recovery-key:<library_id>` ("I've saved it", or sign-out).
pub fn delete_recovery_key(secrets: &dyn SecretStore, library_id: &str) -> Result<(), SecretError> {
    secrets.delete(&recovery_item(library_id))
}

/// A logout the server hasn't accepted yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLogout {
    pub server: String,
    pub token: SessionToken,
}

/// The `sync-logout` item's JSON.
#[derive(Serialize, Deserialize)]
struct LogoutItem {
    v: u32,
    pending: Vec<PendingLogout>,
}

/// The library's pending logouts. An unreadable item counts as none.
pub fn pending_logouts(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<Vec<PendingLogout>, SecretError> {
    let Some(json) = secrets.get(&logout_item(library_id))? else {
        return Ok(Vec::new());
    };
    Ok(serde_json::from_str::<LogoutItem>(&json)
        .ok()
        .filter(|item| item.v == 1)
        .map(|item| item.pending)
        .unwrap_or_default())
}

/// Replaces the library's pending logouts (an empty list deletes the item).
pub fn set_pending_logouts(
    secrets: &dyn SecretStore,
    library_id: &str,
    pending: &[PendingLogout],
) -> Result<(), SecretError> {
    if pending.is_empty() {
        return secrets.delete(&logout_item(library_id));
    }
    let item = LogoutItem {
        v: 1,
        pending: pending.to_vec(),
    };
    let json = Zeroizing::new(serde_json::to_string(&item).expect("the item serializes"));
    secrets.set(&logout_item(library_id), &json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn item_names() {
        assert_eq!(account_item("l_00ff"), "sync-account:l_00ff");
        assert_eq!(pending_item("l_00ff"), "sync-pending:l_00ff");
        assert_eq!(recovery_item("l_00ff"), "sync-recovery-key:l_00ff");
        assert_eq!(logout_item("l_00ff"), "sync-logout:l_00ff");
    }

    fn locked() -> LockedSession {
        LockedSession::new(
            "https://a.example".into(),
            "https://a.example".into(),
            "a_0123456789abcdef0123456789abcdef".into(),
            "alice".into(),
            "d_0123456789abcdef0123456789abcdef".into(),
            SessionToken::new(format!("kanso_{}", b64(&[6; 32]))),
            "0123456789abcdef".into(),
            "fedcba9876543210".into(),
            None,
            None,
        )
        .with_has_password(true)
        .with_login_device(LoginDevice {
            id: "d_0123456789abcdef0123456789abcdef".into(),
            proof: b64(&[7; 32]),
        })
    }

    #[test]
    fn locked_pending_v2_round_trips_raw_eviction_and_approval_without_unwrapping_secrets() {
        let secrets = MemorySecrets::new();
        let library_id = "l_00000000000000000000000000000001";
        let requester_keys = crate::approval::ApprovalKeys::from_secret([8; 32]);
        let request_id = b64(&[9; 32]);
        let requester_secret = b64(&[8; 32]);
        let sealed_name = "A".repeat(55);
        let mut resume = ApprovalResume::new(request_id, &requester_keys, 123_456, 2);
        resume.approver_epk = Some([10; 32]);
        resume.approver_device_id = Some("d_11111111111111111111111111111111".into());
        resume.sealed_name = Some(sealed_name.clone());
        let locked = locked()
            .with_evicted_wire_device(Some(WireDevice {
                id: "d_22222222222222222222222222222222".into(),
                sealed_name: Some(sealed_name.clone()),
            }))
            .with_approval_resume(resume);
        let mut locked = locked;
        locked.set_library_id(library_id);
        save_locked_pending(&secrets, library_id, &locked).unwrap();

        let raw = secrets.get(&pending_item(library_id)).unwrap().unwrap();
        assert!(raw.contains("\"v\":2"));
        assert!(raw.contains("\"device_proof\""));
        assert!(raw.contains("\"evicted_device\""));
        assert!(raw.contains("\"approval\""));
        assert!(!raw.contains("data_key"));
        assert!(!raw.contains("wrapped_key"));
        assert!(!raw.contains("has_password"));
        let loaded = load_locked_pending(&secrets, library_id).unwrap().unwrap();
        assert_eq!(loaded.server, locked.server);
        assert_eq!(loaded.origin, locked.origin);
        assert_eq!(loaded.account_id, locked.account_id);
        assert_eq!(loaded.username, locked.username);
        assert_eq!(loaded.device_id, locked.device_id);
        assert_eq!(loaded.key_id, locked.key_id);
        assert_eq!(loaded.epoch, locked.epoch);
        assert!(
            !loaded.has_password,
            "password availability is refreshed from the server"
        );
        assert_eq!(loaded.library_id(), Some(library_id));
        assert_eq!(
            loaded.login_device().map(|device| device.proof.as_str()),
            Some(b64(&[7; 32]).as_str())
        );
        assert_eq!(
            loaded
                .evicted_wire_device()
                .and_then(|device| device.sealed_name.as_deref()),
            Some(sealed_name.as_str())
        );
        let loaded_resume = loaded.approval_resume().unwrap();
        assert_eq!(loaded_resume.requester_epk(), requester_keys.epk());
        assert_eq!(loaded_resume.expires_at(), 123_456);
        assert_eq!(loaded_resume.other_devices(), 2);
        assert_eq!(loaded_resume.sealed_name(), Some(sealed_name.as_str()));

        let debug = format!("{loaded:?} {:?}", loaded.login_device().unwrap());
        assert!(!debug.contains(&b64(&[7; 32])));
        assert!(!debug.contains(&requester_secret));
        assert!(!debug.contains(&sealed_name));
        assert!(!debug.contains("kanso_aaa"));
    }

    #[test]
    fn malformed_v2_and_legacy_v1_pending_records_are_ignored() {
        let secrets = MemorySecrets::new();
        let library_id = "l_00000000000000000000000000000001";
        let keys = crate::approval::ApprovalKeys::from_secret([13; 32]);
        let valid_locked = locked()
            .with_evicted_wire_device(Some(WireDevice {
                id: "d_33333333333333333333333333333333".into(),
                sealed_name: Some("A".repeat(55)),
            }))
            .with_approval_resume(ApprovalResume::new(b64(&[14; 32]), &keys, 123, 1));
        let mut valid_locked = valid_locked;
        valid_locked.set_library_id(library_id);
        save_locked_pending(&secrets, library_id, &valid_locked).unwrap();
        let valid: serde_json::Value =
            serde_json::from_str(&secrets.get(&pending_item(library_id)).unwrap().unwrap())
                .unwrap();
        let legacy = serde_json::json!({
            "v": 1,
            "server": "https://a.example",
            "origin": "https://a.example",
            "account_id": "a_0123456789abcdef0123456789abcdef",
            "username": "alice",
            "device_id": "d_0123456789abcdef0123456789abcdef",
            "token": format!("kanso_{}", "a".repeat(43)),
            "key_id": "0123456789abcdef",
            "epoch": "fedcba9876543210",
            "has_password": true
        });
        secrets
            .set(&pending_item(library_id), &legacy.to_string())
            .unwrap();
        assert!(
            load_locked_pending(&secrets, library_id).unwrap().is_none(),
            "v1 has no provider proof and cannot be upgraded with invented state"
        );
        save_locked_pending(&secrets, library_id, &valid_locked).unwrap();
        assert!(
            load_pending(
                &secrets,
                library_id,
                "https://a.example",
                "a_ffffffffffffffffffffffffffffffff",
            )
            .unwrap()
            .is_none()
        );
        assert!(
            load_pending(
                &secrets,
                library_id,
                "https://foreign.example",
                "a_0123456789abcdef0123456789abcdef",
            )
            .unwrap()
            .is_none()
        );

        for (field, value) in [
            ("v", serde_json::json!(1)),
            ("server", serde_json::json!("https://foreign.example")),
            ("account_id", serde_json::json!("bad")),
            ("device_id", serde_json::json!("bad")),
            ("device_proof", serde_json::json!("bad")),
            ("token", serde_json::json!("bad")),
            ("key_id", serde_json::json!("0123456789ABCDEF")),
            ("epoch", serde_json::json!("short")),
            ("data_key", serde_json::json!("must-never-be-present")),
        ] {
            let mut malformed = valid.clone();
            malformed[field] = value;
            secrets
                .set(&pending_item(library_id), &malformed.to_string())
                .unwrap();
            assert!(
                load_locked_pending(&secrets, library_id).unwrap().is_none(),
                "accepted malformed {field}"
            );
        }

        for (path, value) in [
            (
                "evicted_device",
                serde_json::json!({ "id": "bad", "sealed_name": "A".repeat(55) }),
            ),
            ("approval", serde_json::json!({ "request_id": "bad" })),
        ] {
            let mut malformed = valid.clone();
            malformed[path] = value;
            secrets
                .set(&pending_item(library_id), &malformed.to_string())
                .unwrap();
            assert!(
                load_locked_pending(&secrets, library_id).unwrap().is_none(),
                "accepted malformed {path}"
            );
        }

        let mut malformed = valid.clone();
        malformed["evicted_device"]["sealed_name"] = serde_json::json!("short");
        secrets
            .set(&pending_item(library_id), &malformed.to_string())
            .unwrap();
        assert!(load_locked_pending(&secrets, library_id).unwrap().is_none());

        let mut malformed = valid.clone();
        malformed["approval"]["requester_secret"] = serde_json::json!("bad");
        secrets
            .set(&pending_item(library_id), &malformed.to_string())
            .unwrap();
        assert!(load_locked_pending(&secrets, library_id).unwrap().is_none());
    }

    #[test]
    fn clearing_approval_resume_keeps_the_locked_session() {
        let secrets = MemorySecrets::new();
        let library_id = "l_00000000000000000000000000000001";
        let keys = crate::approval::ApprovalKeys::from_secret([11; 32]);
        let resume = ApprovalResume::new(b64(&[12; 32]), &keys, 123, 1);
        let mut locked = locked().with_approval_resume(resume);
        locked.set_library_id(library_id);
        save_locked_pending(&secrets, library_id, &locked).unwrap();

        clear_pending_approval(&secrets, library_id).unwrap();

        let loaded = load_locked_pending(&secrets, library_id).unwrap().unwrap();
        assert_eq!(loaded.token(), locked.token());
        assert_eq!(loaded.account_id, locked.account_id);
        assert!(loaded.approval_resume().is_none());
    }

    #[test]
    fn updating_locked_pending_allows_token_refresh_but_rejects_identity_change() {
        let secrets = MemorySecrets::new();
        let library_id = "l_00000000000000000000000000000001";
        let original = locked();
        let refreshed = LockedSession::new(
            original.server.clone(),
            original.origin.clone(),
            original.account_id.clone(),
            original.username.clone(),
            original.device_id.clone(),
            SessionToken::new(format!("kanso_{}", b64(&[15; 32]))),
            original.key_id.clone(),
            original.epoch.clone(),
            None,
            None,
        )
        .with_login_device(original.login_device().unwrap().clone());
        let mut original = original;
        original.set_library_id(library_id);
        let mut refreshed = refreshed;
        refreshed.set_library_id(library_id);
        save_locked_pending(&secrets, library_id, &original).unwrap();

        update_locked_pending(&secrets, library_id, &refreshed).unwrap();
        let loaded = load_locked_pending(&secrets, library_id).unwrap().unwrap();
        assert_eq!(loaded.token(), refreshed.token());
        assert_ne!(loaded.token(), original.token());

        let mut changed_account = LockedSession::new(
            refreshed.server.clone(),
            refreshed.origin.clone(),
            "a_ffffffffffffffffffffffffffffffff".into(),
            refreshed.username.clone(),
            refreshed.device_id.clone(),
            refreshed.token().clone(),
            refreshed.key_id.clone(),
            refreshed.epoch.clone(),
            None,
            None,
        )
        .with_login_device(refreshed.login_device().unwrap().clone());
        changed_account.set_library_id(library_id);
        assert!(update_locked_pending(&secrets, library_id, &changed_account).is_err());
        assert_eq!(
            load_locked_pending(&secrets, library_id)
                .unwrap()
                .unwrap()
                .token(),
            refreshed.token()
        );
    }

    /// An item saved before the key's server and state were kept still
    /// loads: confirmed, with its origin as the server.
    #[test]
    fn older_recovery_items_still_load() {
        let secrets = MemorySecrets::new();
        let key = RecoveryKey::from_bytes([3; 16]);
        let old = serde_json::json!({
            "v": 1, "origin": "https://a.example", "account_id": "a_1",
            "username": "alice", "recovery_key": key.display(),
        });
        secrets
            .set(&recovery_item("l_1"), &old.to_string())
            .unwrap();
        let loaded = load_any_recovery_key(&secrets, "l_1").unwrap().unwrap();
        assert_eq!(loaded.recovery_key, key);
        assert_eq!(loaded.server, "https://a.example");
        assert!(!loaded.unconfirmed);
        assert!(
            load_recovery_key(&secrets, "l_1", "https://a.example", "a_2")
                .unwrap()
                .is_none()
        );
        let pending = PendingRecoveryKey {
            unconfirmed: true,
            server: "https://a.example/".into(),
            ..loaded
        };
        save_recovery_key(&secrets, "l_1", &pending).unwrap();
        let back = load_any_recovery_key(&secrets, "l_1").unwrap().unwrap();
        assert!(back.unconfirmed);
        assert_eq!(back.server, "https://a.example/");
    }

    #[derive(Clone, Debug)]
    enum Op {
        Set(u8, String),
        Delete(u8),
        /// The write this many writes from now fails.
        Fail(u8),
    }

    fn gen_op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0..4u8, "[a-z]{0,6}").prop_map(|(k, v)| Op::Set(k, v)),
            (0..4u8).prop_map(Op::Delete),
            (0..3u8).prop_map(Op::Fail),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Model-based: `MemorySecrets` behaves like a map, deleting a
        /// missing item succeeds, and a write made to fail fails and changes
        /// nothing.
        #[test]
        fn prop_memory_secrets_follow_a_map(ops in prop::collection::vec(gen_op(), 0..24)) {
            let secrets = MemorySecrets::new();
            let mut model = BTreeMap::new();
            let mut writes = 0usize;
            let mut failing = std::collections::BTreeSet::new();
            for op in ops {
                match op {
                    Op::Set(k, v) => {
                        let fails = failing.remove(&writes);
                        writes += 1;
                        prop_assert_eq!(secrets.set(&format!("item{k}"), &v).is_err(), fails);
                        if !fails {
                            model.insert(format!("item{k}"), v);
                        }
                    }
                    Op::Delete(k) => {
                        let fails = failing.remove(&writes);
                        writes += 1;
                        prop_assert_eq!(secrets.delete(&format!("item{k}")).is_err(), fails);
                        if !fails {
                            model.remove(&format!("item{k}"));
                        }
                    }
                    Op::Fail(n) => {
                        secrets.fail_write(usize::from(n));
                        failing.insert(writes + usize::from(n));
                    }
                }
                prop_assert_eq!(secrets.writes(), writes);
                for k in 0..4u8 {
                    let name = format!("item{k}");
                    let got = secrets.get(&name).unwrap().map(|v| v.to_string());
                    prop_assert_eq!(got, model.get(&name).cloned());
                }
                prop_assert_eq!(secrets.items(), model.keys().cloned().collect::<Vec<_>>());
            }
        }
    }
}

/// The first line of a `lock-keys` item; each line after it is one key.
const LOCK_KEYS_V1: &str = "kanso-lock-keys/1";

/// Keeps `keys` for Touch ID, replacing any kept before.
pub fn save_lock_keys(
    secrets: &dyn SecretStore,
    library_id: &str,
    keys: &[&crate::lock::LockKey],
) -> Result<(), SecretError> {
    let mut value = Zeroizing::new(String::from(LOCK_KEYS_V1));
    for key in keys {
        value.push('\n');
        value.push_str(&key.to_secret());
    }
    secrets.set(&lock_keys_item(library_id), &value)
}

/// The lock keys kept for Touch ID, or `None` when there are none. An item
/// this version can't read counts as none.
pub fn load_lock_keys(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<Option<Vec<crate::lock::LockKey>>, SecretError> {
    let Some(value) = secrets.get(&lock_keys_item(library_id))? else {
        return Ok(None);
    };
    let mut lines = value.lines();
    if lines.next() != Some(LOCK_KEYS_V1) {
        return Ok(None);
    }
    let keys: Vec<_> = lines
        .filter_map(|line| crate::lock::LockKey::from_secret(line).ok())
        .collect();
    Ok((!keys.is_empty()).then_some(keys))
}

/// Forgets the lock keys kept for Touch ID.
pub fn delete_lock_keys(secrets: &dyn SecretStore, library_id: &str) -> Result<(), SecretError> {
    secrets.delete(&lock_keys_item(library_id))
}

#[cfg(test)]
mod lock_key_tests {
    use super::*;
    use crate::crypto::KdfParams;
    use crate::lock::{LockKey, LockPassword};

    #[test]
    fn lock_keys_are_kept_per_library_and_forgotten() {
        let secrets = MemorySecrets::new();
        let password = LockPassword::new("correct horse").unwrap();
        let first = LockKey::create(&password, KdfParams::TEST).unwrap();
        let second = LockKey::create(&password, KdfParams::TEST).unwrap();
        assert!(load_lock_keys(&secrets, "l_a").unwrap().is_none());
        save_lock_keys(&secrets, "l_a", &[&first, &second]).unwrap();
        let kept = load_lock_keys(&secrets, "l_a").unwrap().unwrap();
        assert_eq!(kept.len(), 2);
        assert!(kept[0].same_kdf(&first) && kept[1].same_kdf(&second));
        assert!(load_lock_keys(&secrets, "l_b").unwrap().is_none());
        // The password itself is never kept.
        let raw = secrets.get(&lock_keys_item("l_a")).unwrap().unwrap();
        assert!(!raw.contains("correct horse"));
        delete_lock_keys(&secrets, "l_a").unwrap();
        assert!(load_lock_keys(&secrets, "l_a").unwrap().is_none());
    }
}
