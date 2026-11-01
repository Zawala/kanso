//! Kanso's anonymous usage count, as docs/METRICS.md defines it: a small
//! batch once a day saying the app was open and which features were used,
//! with every value from a closed list.
//!
//! The crate holds what may be sent ([`values`]), the batch's JSON
//! ([`batch`]), the day's bookkeeping the app keeps in its settings
//! ([`Ledger`]) and the one blocking request ([`send`]). It never decides on
//! its own whether to send: the app passes `enabled` (a release build of the
//! release flavour, never a test) and only calls it after the notice was
//! seen. Plain Rust, no UI: the desktop app and the phones' library both use
//! it.

pub mod batch;
pub mod buffer;
pub mod ledger;
mod ledger_model;
pub mod ranges;
pub mod send;
pub mod values;

pub use batch::{Batch, Event, EventKind};
pub use buffer::{CountBuffer, FLUSH_EVERY_MS};
pub use ledger::Ledger;
pub use send::{ENDPOINT, Outcome, send};
pub use values::{
    Active, Day, Device, Feature, Form, Id, ImportSource, NotesRange, Platform, SyncMode,
    install_id,
};
