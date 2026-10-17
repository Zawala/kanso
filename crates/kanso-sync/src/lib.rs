//! Kanso's Cloudflare sync client.
//!
//! The server is a Cloudflare Worker (`cloud/`) over D1, which holds every
//! document's current revision under a global change sequence, and R2, which
//! holds attachment bytes. A sync ([`sync_once`]) pulls the server's changes
//! after the stored cursor and then pushes local changes; the store's
//! bookkeeping (`synced_rev`, last writer by `updatedAt` wins, losers kept in
//! revisions) decides the rest.
//!
//! [`HttpRemote`] talks to the Worker; [`MemoryRemote`] is an in-memory server
//! with the same rules, for tests. The token lives in the Keychain
//! ([`credentials`]); the app keeps the endpoint in a [`SyncConfig`].

pub mod credentials;
mod engine;
mod http;
mod memory;
mod remote;

pub use engine::{
    CURSOR_KEY, SyncError, SyncOptions, SyncReport, batches, cursor, reset_cursor, sync_once,
    sync_with,
};
pub use http::HttpRemote;
pub use kanso_store::SyncDoc;
pub use memory::{MemoryRemote, MemoryStats};
pub use remote::{
    ChangesPage, DEFAULT_CHANGES_LIMIT, MAX_CHANGES_LIMIT, MAX_FIELDS_BYTES, MAX_FILE_BYTES,
    MAX_PUSH_BYTES, MAX_PUSH_DOCS, PushResult, Remote, RemoteDoc, RemoteError, RemoteFile,
    is_doc_id, is_file_id, is_md5_hex, is_rev, md5_hex, validate_push,
};

/// Where the app syncs to. The token is kept separately, in the Keychain.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SyncConfig {
    /// The Worker's base URL, e.g. `https://kanso-sync.example.workers.dev`.
    pub endpoint: String,
}

impl SyncConfig {
    pub fn new(endpoint: impl Into<String>) -> Self {
        SyncConfig {
            endpoint: endpoint.into().trim().trim_end_matches('/').to_owned(),
        }
    }

    /// A client for this endpoint using `token`.
    pub fn remote(&self, token: impl Into<String>) -> HttpRemote {
        HttpRemote::new(self.endpoint.clone(), token)
    }

    /// A client for this endpoint using the token in the Keychain, if any.
    pub fn remote_from_keychain(&self) -> Result<Option<HttpRemote>, credentials::CredentialError> {
        Ok(credentials::load_token()?.map(|token| self.remote(token)))
    }
}
