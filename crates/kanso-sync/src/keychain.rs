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

/// The macOS Keychain, service [`SERVICE`]: generic passwords whose account
/// is the item name. Tests never construct it; they use [`MemorySecrets`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Keychain;

// no-prop-test: a thin wrapper over the system Keychain, which tests must
// never touch; the item formats above it are tested through `MemorySecrets`.
impl Keychain {
    fn entry(item: &str) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(SERVICE, item).map_err(keychain_error)
    }
}

fn keychain_error(err: keyring::Error) -> SecretError {
    SecretError(err.to_string())
}

impl SecretStore for Keychain {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        match Keychain::entry(item)?.get_password() {
            Ok(value) => Ok(Some(Zeroizing::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(keychain_error(err)),
        }
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        Keychain::entry(item)?
            .set_password(value)
            .map_err(keychain_error)
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        match Keychain::entry(item)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(keychain_error(err)),
        }
    }
}

/// Secrets held in memory, for tests and snapshots.
#[derive(Default)]
pub struct MemorySecrets {
    items: Mutex<BTreeMap<String, Zeroizing<String>>>,
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
}

impl SecretStore for MemorySecrets {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        Ok(self
            .items
            .lock()
            .expect("secrets poisoned")
            .get(item)
            .cloned())
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        self.items
            .lock()
            .expect("secrets poisoned")
            .insert(item.to_owned(), Zeroizing::new(value.to_owned()));
        Ok(())
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
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

/// A recovery key the user hasn't confirmed saving yet.
#[derive(Debug)]
pub struct PendingRecoveryKey {
    pub origin: String,
    pub account_id: String,
    pub username: String,
    pub recovery_key: RecoveryKey,
}

/// The `sync-recovery-key` item's JSON; the key as it is shown.
#[derive(Serialize, Deserialize)]
struct RecoveryItem {
    v: u32,
    origin: String,
    account_id: String,
    username: String,
    #[serde(with = "zeroizing_string")]
    recovery_key: Zeroizing<String>,
}

/// Saves `sync-recovery-key:<library_id>` (right after sign-up, recovery or
/// "New recovery key" succeeds, before binding).
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
        recovery_key: Zeroizing::new(key.recovery_key.display()),
    };
    let json = Zeroizing::new(serde_json::to_string(&item).expect("the item serializes"));
    secrets.set(&recovery_item(library_id), &json)
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
    let Some(json) = secrets.get(&recovery_item(library_id))? else {
        return Ok(None);
    };
    let Ok(item) = serde_json::from_str::<RecoveryItem>(&json) else {
        return Ok(None);
    };
    if item.v != 1 || item.origin != origin || item.account_id != account_id {
        return Ok(None);
    }
    Ok(RecoveryKey::parse(&item.recovery_key)
        .ok()
        .map(|recovery_key| PendingRecoveryKey {
            origin: item.origin,
            account_id: item.account_id,
            username: item.username,
            recovery_key,
        }))
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

    #[derive(Clone, Debug)]
    enum Op {
        Set(u8, String),
        Delete(u8),
    }

    fn gen_op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0..4u8, "[a-z]{0,6}").prop_map(|(k, v)| Op::Set(k, v)),
            (0..4u8).prop_map(Op::Delete),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Model-based: `MemorySecrets` behaves like a map, and deleting a
        /// missing item succeeds.
        #[test]
        fn prop_memory_secrets_follow_a_map(ops in prop::collection::vec(gen_op(), 0..24)) {
            let secrets = MemorySecrets::new();
            let mut model = BTreeMap::new();
            for op in ops {
                match op {
                    Op::Set(k, v) => {
                        secrets.set(&format!("item{k}"), &v).unwrap();
                        model.insert(format!("item{k}"), v);
                    }
                    Op::Delete(k) => {
                        secrets.delete(&format!("item{k}")).unwrap();
                        model.remove(&format!("item{k}"));
                    }
                }
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
