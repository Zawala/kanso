//! The lock session: the lock password's keys stay in Rust.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, LockEvent, LockSession, Saved};

#[frb(opaque)]
pub struct KansoLocks(LockSession);

impl KansoLocks {
    pub(super) fn new(session: LockSession) -> KansoLocks {
        KansoLocks(session)
    }

    pub fn unlock(&self, password: String) -> Result<bool, KansoError> {
        self.0.unlock(password)
    }

    pub fn lock(&self) -> Result<(), KansoError> {
        self.0.lock()
    }

    pub fn is_unlocked(&self) -> Result<bool, KansoError> {
        self.0.is_unlocked()
    }

    pub fn is_note_unlocked(&self, id: String) -> Result<bool, KansoError> {
        self.0.is_note_unlocked(&id)
    }

    pub fn lock_note(&self, id: String, password: String) -> Result<Saved, KansoError> {
        self.0.lock_note(&id, password)
    }

    /// Each time the notes open or close, until Dart stops listening.
    pub fn lock_events(&self, sink: StreamSink<LockEvent>) {
        self.0.lock_events(move |event| sink.add(event).is_ok());
    }
}
