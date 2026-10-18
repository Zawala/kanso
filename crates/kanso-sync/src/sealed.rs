//! Between the engine's plaintext documents and the wire: `Sealed<W:
//! WireRemote>` (step 2b) seals outgoing documents, opens incoming ones, maps
//! local IDs and revisions to wire ones, and keeps repair bases.
//!
//! Step 1 freezes only [`PlainView`], which the test suites use instead of
//! reading server state directly (docs/ACCOUNTS.md › Tests). `Sealed` itself
//! is internal to the Rust builder.

use kanso_store::SyncDoc;

use crate::remote::{PushResult, RemoteDoc, RemoteError};

/// The decrypted view of one account on a [`crate::MemoryServer`], under a
/// test session: what the server holds, opened, and a way to push plaintext
/// as another client of the same account. Tests only.
pub struct PlainView {
    _step_2b: (),
}

impl PlainView {
    /// Every document of the account, opened, tombstones included, in `seq`
    /// order, with local IDs and revisions.
    pub fn docs(&self) -> Vec<RemoteDoc> {
        crate::stub!("2b", self)
    }

    /// The opened bytes of the file document `local_id`, if the server has them.
    pub fn file(&self, local_id: &str) -> Option<Vec<u8>> {
        crate::stub!("2b", self, local_id)
    }

    /// Seals and pushes plaintext documents as another device of the account.
    pub fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        crate::stub!("2b", self, docs)
    }
}
