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
//! - [`background`] is the sync that runs with no app: the code behind the
//!   C entry `kanso_background_sync` that `mobile/rust` exports.
//! - A note's past and the library's safety nets: [`Library::history`] lists
//!   the saved versions, [`Library::history_diff`] says what changed since
//!   one and [`Library::restore_version`] brings one back (a locked note's
//!   sealed versions are opened with the session's keys, never returned
//!   sealed, and not at all while it is closed). [`Library::replace_preview`],
//!   [`Library::replace_apply`] and [`Library::replace_undo`] are Replace in
//!   all notes, which never touches a locked note. [`Library::backup_if_due`],
//!   [`Library::back_up_now`], [`Library::backups`] and
//!   [`Library::delete_backup`] keep a daily copy of the database inside the
//!   library's folder (so out of the device's backups) and one on request.
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

pub mod background;
mod backup;
mod backups;
pub mod boundary;
mod builtin_themes;
pub mod diagnostics;
mod errors;
mod history;
mod library;
mod locks;
mod matching;
mod model;
mod model_history;
mod notes;
mod renderer;
mod replace;
pub mod secrets;
mod stream;
mod syncing;
mod theme_tokens;

pub use backups::BACKUPS_SUMMARY;
pub use errors::KansoError;
pub use library::Library;
pub use locks::LockSession;
pub use model::*;
pub use model_history::*;
pub use renderer::Renderer;
pub use replace::replace_apply_label;
pub use stream::Sink;
pub use syncing::SyncHandle;

/// Where secrets live: the seam the Keychain, the Keystore and the tests'
/// memory store stand behind.
pub use kanso_engine::{MemorySecrets, Secrets};
/// The server a library syncs with (the sync client's `Backend`), so a test
/// or the background entry can pass its own.
pub use kanso_sync::Backend;
