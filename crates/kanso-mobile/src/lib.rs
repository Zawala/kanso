//! Kanso's library for the phones' app (#48).
//!
//! The Flutter shell calls this crate, through the bridge in `mobile/rust`,
//! and nothing else of Kanso's. It is the facade over the shared core: the
//! store, the engine (lock sessions, accounts, sync) and the renderer. It
//! opens no window, reads no setting of the shell's and depends on no bridge
//! generator, so everything here is plain Rust, tested on a computer and on
//! the simulators.
//!
//! # What is here
//!
//! - [`Library`] opens a library (one database file) and answers for its
//!   notes: lists by place with the desktop's sort, narrowing and grouping,
//!   the counts of the places, one note, saving, creating, changing several
//!   notes at once, notebooks in tree order, tags, search by scope with the
//!   words matched, properties and query views, and the
//!   [`Library::changes`] stream.
//! - [`LockSession`] (`library.locks()`) holds the lock password's keys. A
//!   locked note's text, snippet, properties, tasks and preview never come
//!   back while it is closed; only its title and its lock show in a list.
//!   Keys never cross this edge.
//! - [`SyncHandle`] (`library.sync()`) signs in to a server, syncs and
//!   reports its status.
//! - [`Renderer`] (`library.renderer()`) draws a note's preview, highlights
//!   source, fits a theme (the built-in ones are in, as the desktop has
//!   them, and source can be colored in any of them) and draws math and
//!   diagrams.
//! - [`AccountHandle`] (`library.account()`) signs in and up with every
//!   way the server offers (password, passkey, Apple, Google, GitHub),
//!   opens a locked account with the password, another device or the
//!   recovery key, and asks the binding question.
//!   [`AccountSettings`] (`library.account_settings()`) is the account's
//!   page, and [`Approvals`] (`library.approvals()`) the requests of other
//!   devices this phone may approve.
//! - [`SharedNotes`] (`library.shared()`) shares notes with other accounts
//!   and keeps an open shared note live.
//! - [`Gallery`] (`library.gallery()`) lists and installs the gallery's
//!   themes, checked as the desktop checks them: the signed index, then
//!   each file's pinned hash.
//! - [`background`] is the sync that runs with no app: the code behind the
//!   C entry `kanso_background_sync` that `mobile/rust` exports.
//! - [`diagnostics`] has the version, the TLS check and the panic probe.
//!
//! # The edge
//!
//! Every function that can fail returns [`KansoError`], and runs inside
//! [`boundary::guard`]: a panic is caught and returned as
//! [`KansoError::Internal`], and the next call works. Streams are sinks,
//! closures `FnMut(T) -> bool + Send + 'static` that return `false` once
//! nobody listens; the bridge wraps them in its stream type.
//!
//! Secrets (the session, the data key) are kept by Rust in the platform's
//! store ([`secrets`]), never handed to the shell.

mod account;
mod account_methods;
mod account_model;
mod account_provider;
mod account_runtime;
mod account_settings;
mod approvals;
mod approver_model;
pub mod background;
mod backup;
pub mod boundary;
mod builtin_themes;
mod comments_model;
pub mod diagnostics;
mod errors;
mod gallery;
mod gallery_model;
mod gallery_net;
mod gallery_store;
mod library;
mod locks;
mod matching;
mod model;
mod notes;
pub mod passkey_bridge;
mod passkey_flow;
mod provider_loopback;
mod renderer;
mod schedule;
pub mod secrets;
mod sharing;
mod sharing_model;
mod stream;
mod sync_model;
mod sync_words;
mod syncing;
mod theme_tokens;

pub use account::AccountHandle;
pub use account_model::*;
pub use account_settings::AccountSettings;
pub use approvals::Approvals;
pub use comments_model::*;
pub use errors::KansoError;
pub use gallery::Gallery;
pub use gallery_model::*;
pub use library::Library;
pub use locks::LockSession;
pub use model::*;
pub use renderer::Renderer;
pub use sharing::SharedNotes;
pub use sharing_model::*;
pub use stream::Sink;
pub use sync_model::*;
pub use syncing::SyncHandle;

/// Where secrets live: the seam the Keychain, the Keystore and the tests'
/// memory store stand behind.
pub use kanso_engine::{MemorySecrets, Secrets};
/// The server a library syncs with (the sync client's `Backend`), so a test
/// or the background entry can pass its own.
pub use kanso_sync::Backend;
