//! Secrets in the macOS Keychain (docs/ACCOUNTS.md › Keychain items).
//! The old app's `sync-token` item is never read, written or deleted here;
//! the owner removes it after the legacy purge.
//!
//! The service is `Kanso`; item names carry the library ID, so data
//! directories never share secrets. [`SecretStore`] is the seam: [`Keychain`]
//! in the app, [`MemorySecrets`] in tests and snapshots.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::account::Session;
use crate::crypto::{DataKey, RecoveryKey, b64, unb64};
use crate::wire::{SessionToken, is_account_id, is_device_id, is_token};

/// The Keychain service of every Kanso item.
pub const SERVICE: &str = "Kanso";

/// `sync-account:<library_id>`: `{"v":1,"server","origin","account_id",
/// "username","device_id","token","data_key"}`, from sign-up, sign-in or
/// recovery until sign-out, deletion, a 401 or unbind.
pub fn account_item(library_id: &str) -> String {
    format!("sync-account:{library_id}")
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
        assert_eq!(recovery_item("l_00ff"), "sync-recovery-key:l_00ff");
        assert_eq!(logout_item("l_00ff"), "sync-logout:l_00ff");
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
