//! The process a Kanso plugin runs in.
//!
//! The app starts one `kanso-plugin-host` per plugin in use, with an empty
//! environment, the plugin's own folder as its working directory and a pipe
//! as its only input. It runs one QuickJS runtime and context holding only
//! what [`runtime`] puts on the global object: the `kanso` API, whose every
//! call is a message to the app ([`protocol`]). There is no filesystem,
//! network, timer, module loader or console in it, and nothing runs between
//! the app's requests.
//!
//! This crate links no `kanso-store`, `kanso-sync` or Keychain code, so an
//! engine bug here stays away from the notes, the keys and the database.

pub mod protocol;
pub mod runtime;
