//! The plaintext v1 server and the old app's sync, for the rollback test
//! (docs/ACCOUNTS.md › Rollback, fix 1 / H5).
//!
//! The server is today's `MemoryRemote`, conformance-tested against the v1
//! Worker (`cloud/test/fixtures/conformance-v1.json`); the old app's sync is
//! `sync_once`, which keeps its cursor under the legacy `sync.last_seq` and
//! applies the v1 store rules. Both still live in kanso-sync until step 3 of
//! docs/ACCOUNTS.md › Build order; when that step removes them, move them
//! here (they only use public kanso-store APIs), so the rollback test never
//! needs a live old Worker.

use kanso_store::Store;
use kanso_sync::SyncReport;

pub use kanso_sync::MemoryRemote as LegacyServer;

/// One sync of the old app against the v1 server.
pub fn legacy_sync(store: &Store, server: &LegacyServer) -> SyncReport {
    kanso_sync::sync_once(store, server).expect("v1 sync")
}
