//! The sync token in the macOS Keychain (service "Kanso", account "sync-token").
//!
//! Legacy (v1, one shared token): kept while the app still uses it; step 3
//! of docs/ACCOUNTS.md › Build order replaces it with [`crate::keychain`].

// no-prop-test: thin wrapper over the system keychain with no observable
// oracle beyond the keychain itself; exercised by the deploy and e2e steps.

use keyring::Entry;

pub const SERVICE: &str = "Kanso";
pub const ACCOUNT: &str = "sync-token";

#[derive(Debug, thiserror::Error)]
#[error("keychain: {0}")]
pub struct CredentialError(#[from] keyring::Error);

fn entry() -> Result<Entry, CredentialError> {
    Ok(Entry::new(SERVICE, ACCOUNT)?)
}

/// Stores (or replaces) the sync token.
pub fn save_token(token: &str) -> Result<(), CredentialError> {
    Ok(entry()?.set_password(token.trim())?)
}

/// The stored sync token, if any.
pub fn load_token() -> Result<Option<String>, CredentialError> {
    match entry()?.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Removes the sync token; succeeds when there is none.
pub fn delete_token() -> Result<(), CredentialError> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
