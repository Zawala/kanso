//! Secrets in the macOS Keychain (docs/ACCOUNTS.md › Keychain items).
//! Replaces `credentials.rs` (deleted in step 2a).
//!
//! The service is `Kanso`; item names carry the library ID, so data
//! directories never share secrets. [`SecretStore`] is the seam: [`Keychain`]
//! in the app, [`MemorySecrets`] in tests and snapshots.

use std::collections::BTreeMap;
use std::sync::Mutex;

use zeroize::Zeroizing;

use crate::account::Session;
use crate::crypto::RecoveryKey;
use crate::wire::SessionToken;

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

/// The legacy shared token. Never read or deleted by the new app.
pub const LEGACY_TOKEN_ITEM: &str = "sync-token";

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

/// The macOS Keychain, service [`SERVICE`] (step 2a).
#[derive(Clone, Copy, Debug, Default)]
pub struct Keychain;

impl SecretStore for Keychain {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        crate::stub!("2a", item)
    }

    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        crate::stub!("2a", item, value)
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        crate::stub!("2a", item)
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

/// Saves the session as `sync-account:<library_id>`.
pub fn save_session(
    secrets: &dyn SecretStore,
    library_id: &str,
    session: &Session,
) -> Result<(), SecretError> {
    crate::stub!("2a", secrets, library_id, session)
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
    crate::stub!("2a", secrets, library_id, origin, account_id)
}

/// Removes `sync-account:<library_id>`.
pub fn delete_session(secrets: &dyn SecretStore, library_id: &str) -> Result<(), SecretError> {
    crate::stub!("2a", secrets, library_id)
}

/// A recovery key the user hasn't confirmed saving yet.
#[derive(Debug)]
pub struct PendingRecoveryKey {
    pub origin: String,
    pub account_id: String,
    pub username: String,
    pub recovery_key: RecoveryKey,
}

/// Saves `sync-recovery-key:<library_id>` (right after sign-up, recovery or
/// "New recovery key" succeeds, before binding).
pub fn save_recovery_key(
    secrets: &dyn SecretStore,
    library_id: &str,
    key: &PendingRecoveryKey,
) -> Result<(), SecretError> {
    crate::stub!("2a", secrets, library_id, key)
}

/// The unconfirmed recovery key for this `(origin, account_id)`, if any:
/// while it exists the Account page shows the key screen again.
pub fn load_recovery_key(
    secrets: &dyn SecretStore,
    library_id: &str,
    origin: &str,
    account_id: &str,
) -> Result<Option<PendingRecoveryKey>, SecretError> {
    crate::stub!("2a", secrets, library_id, origin, account_id)
}

/// Removes `sync-recovery-key:<library_id>` ("I've saved it", or sign-out).
pub fn delete_recovery_key(secrets: &dyn SecretStore, library_id: &str) -> Result<(), SecretError> {
    crate::stub!("2a", secrets, library_id)
}

/// A logout the server hasn't accepted yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingLogout {
    pub server: String,
    pub token: SessionToken,
}

/// The library's pending logouts.
pub fn pending_logouts(
    secrets: &dyn SecretStore,
    library_id: &str,
) -> Result<Vec<PendingLogout>, SecretError> {
    crate::stub!("2a", secrets, library_id)
}

/// Replaces the library's pending logouts (an empty list deletes the item).
pub fn set_pending_logouts(
    secrets: &dyn SecretStore,
    library_id: &str,
    pending: &[PendingLogout],
) -> Result<(), SecretError> {
    crate::stub!("2a", secrets, library_id, pending)
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
