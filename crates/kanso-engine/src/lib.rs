//! Kanso's engine without a window: the parts of the app that decide what
//! happens, with nothing drawn (issue #48).
//!
//! The desktop app and the mobile app both call it. [`lock`] holds the keys
//! of unlocked notes and seals and opens notes with them, [`account`] runs
//! the account tasks (sign up, sign in, recover, unlock a device, restore at
//! launch) and says their results in plain words, and [`sync`] decides when
//! a sync starts, queues, backs off or stops. Nothing here opens a window,
//! reads a clock of its own or touches the system's secret store: the caller
//! passes the time, and secrets go through [`Secrets`].
//!
//! The system's Keychain is `kanso_sync::Keychain`, built by the desktop app
//! alone; tests use [`MemorySecrets`].

pub mod account;
pub mod lock;
#[cfg(any(test, feature = "test-support"))]
pub mod strategies;
pub mod sync;

/// Where secrets live: the seam the Keychain, the Credential Manager and the
/// mobile keystores stand behind.
pub use kanso_sync::SecretStore as Secrets;
pub use kanso_sync::{MemorySecrets, SecretError};
