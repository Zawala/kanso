//! The plugin directory's index: `index.json` at the registry, signed by
//! `index.json.sig`, and what an update check makes of it. Checking and
//! verifying live in [`crate::registry`].

use serde::{Deserialize, Serialize};

use crate::model::{Manifest, Permissions};

/// The index layout this app reads.
pub const INDEX_SCHEMA: u32 = 1;

/// `index.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Index {
    pub schema: u32,
    /// Grows with every publication; an index older than the one this Mac
    /// last saw is a replay and is refused.
    pub serial: u64,
    pub plugins: Vec<Entry>,
    #[serde(default)]
    pub revoked: Vec<Revocation>,
}

/// One published version of a plugin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub publisher: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    /// The `.kansoplugin` bundle's SHA-256, lowercase hex.
    pub sha256: String,
    /// The bundle's size in bytes.
    pub size: u64,
    /// The author's key, `ed25519:` and hex, as the manifest names it.
    pub signer: String,
    /// The author's base64 signature over the bundle's SHA-256 bytes.
    pub author_signature: String,
    pub min_app_version: String,
    pub permissions: Permissions,
    /// RFC 3339.
    pub published_at: String,
    #[serde(default)]
    pub revoked: bool,
}

/// Versions of a plugin the directory has withdrawn.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Revocation {
    pub id: String,
    /// A semver requirement (`*`, `<1.4.2`, `=1.3.0`).
    pub versions: String,
    pub reason: String,
}

/// What an update offers for one installed plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub entry: Entry,
    /// Permissions the new version asks for that the installed one didn't.
    pub added: Vec<String>,
    /// Permissions the new version no longer asks for.
    pub removed: Vec<String>,
    /// The author's key differs from the one pinned at first install.
    pub key_changed: bool,
}

impl Offer {
    /// Whether installing needs the second, explicit confirmation.
    pub fn needs_confirmation(&self) -> bool {
        !self.added.is_empty() || self.key_changed
    }
}

/// A bundle, opened and checked: its manifest and its files by path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle {
    pub manifest: Manifest,
    /// Every file but `manifest.json`, sorted by path.
    pub files: Vec<(String, Vec<u8>)>,
    /// The bundle's SHA-256, lowercase hex.
    pub sha256: String,
}
