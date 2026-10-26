//! Where the phone keeps the library's secrets: the session token and data
//! key of the signed-in account, a recovery key not yet saved, a pending
//! logout.
//!
//! Rust reaches the platform's store itself, never through Dart: the
//! background sync has no Dart at all, and the threat model keeps these
//! secrets out of the shell. Behind [`Secrets`], the seam the engine and the
//! sync client share:
//!
//! - iOS: the Keychain (`Keychain`), generic passwords of the service
//!   [`KEYCHAIN_SERVICE`], readable after the first unlock and never leaving
//!   the device.
//! - Android: the Keystore, through a Kotlin class the app ships
//!   (`KeystoreSecrets`; [`android`] says exactly which).
//! - Anywhere else (the tests, `flutter test` on a computer): memory only.
//!   It never touches the developer's own Keychain.

use std::sync::Arc;

use crate::Secrets;

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "android")]
pub use android::KeystoreSecrets;
#[cfg(target_os = "ios")]
pub use ios::Keychain;

/// The Keychain service of the app's items on iOS.
pub const KEYCHAIN_SERVICE: &str = "app.kanso.Kanso";

/// The secret store of this platform.
pub fn platform() -> Arc<dyn Secrets> {
    #[cfg(target_os = "ios")]
    {
        Arc::new(Keychain::new(KEYCHAIN_SERVICE))
    }
    #[cfg(target_os = "android")]
    {
        Arc::new(KeystoreSecrets)
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        Arc::new(kanso_sync::MemorySecrets::new())
    }
}
