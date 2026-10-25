//! The iOS Keychain as a [`Secrets`](crate::Secrets) store.
//!
//! Each item is a generic password of one service, its account the item's
//! name. Items are readable after the first unlock since the phone started
//! (a background sync runs with the screen locked) and are not copied to
//! other devices or backups
//! (`kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`).
//!
//! Whether the app and its extensions share these items (a Keychain access
//! group) is decided with the App Group, not here.

use kanso_sync::{SecretError, SecretStore};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::passwords::{
    PasswordOptions, delete_generic_password_options, generic_password,
    set_generic_password_options,
};
use zeroize::Zeroizing;

/// `errSecItemNotFound`: no such item.
const ITEM_NOT_FOUND: i32 = -25300;

/// The Keychain items of one service.
#[derive(Clone, Copy, Debug)]
pub struct Keychain {
    service: &'static str,
}

impl Keychain {
    pub fn new(service: &'static str) -> Keychain {
        Keychain { service }
    }

    fn query(&self, item: &str) -> PasswordOptions {
        PasswordOptions::new_generic_password(self.service, item)
    }

    /// The options a new item is stored with.
    fn creation(&self, item: &str) -> Result<PasswordOptions, SecretError> {
        let access = SecAccessControl::create_with_protection(
            Some(ProtectionMode::AccessibleAfterFirstUnlockThisDeviceOnly),
            0,
        )
        .map_err(|err| failure("access control", &err))?;
        let mut options = self.query(item);
        options.set_access_control(access);
        Ok(options)
    }

    fn read(&self, item: &str) -> Result<Option<Zeroizing<Vec<u8>>>, SecretError> {
        match generic_password(self.query(item)) {
            Ok(bytes) => Ok(Some(Zeroizing::new(bytes))),
            Err(err) if err.code() == ITEM_NOT_FOUND => Ok(None),
            Err(err) => Err(failure("read", &err)),
        }
    }

    fn remove(&self, item: &str) -> Result<(), SecretError> {
        match delete_generic_password_options(self.query(item)) {
            Ok(()) => Ok(()),
            Err(err) if err.code() == ITEM_NOT_FOUND => Ok(()),
            Err(err) => Err(failure("delete", &err)),
        }
    }
}

fn failure(what: &str, err: &security_framework::base::Error) -> SecretError {
    SecretError(format!("the Keychain refused to {what} ({})", err.code()))
}

impl SecretStore for Keychain {
    fn get(&self, item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        self.read(item)?
            .map(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map(Zeroizing::new)
                    .map_err(|_| SecretError("a Keychain item isn't text".into()))
            })
            .transpose()
    }

    /// Replaces the item. The Keychain can't change an item's accessibility
    /// in place, so the old item goes first; if the new one can't be stored,
    /// the old one is put back.
    fn set(&self, item: &str, value: &str) -> Result<(), SecretError> {
        let before = self.read(item)?;
        self.remove(item)?;
        let stored = set_generic_password_options(value.as_bytes(), self.creation(item)?);
        if let Err(err) = stored {
            if let Some(old) = before
                && let Ok(options) = self.creation(item)
            {
                let _ = set_generic_password_options(&old, options);
            }
            return Err(failure("store", &err));
        }
        Ok(())
    }

    fn delete(&self, item: &str) -> Result<(), SecretError> {
        self.remove(item)
    }
}
