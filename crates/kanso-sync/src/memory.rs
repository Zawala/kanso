//! In-memory sync servers for tests: [`MemoryRemote`], today's v1 server
//! (step 2b moves it to `tests/common/legacy.rs`), and [`MemoryServer`], the
//! v2 server with accounts, frozen by step 1.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::time::Duration;

use kanso_store::SyncDoc;
use serde_json::{Map, Value};

use crate::account::Session;
use crate::api::{Api, Backend, Route};
use crate::crypto::KdfParams;
use crate::remote::{
    ChangesPage, MAX_CHANGES_LIMIT, MAX_FILE_BYTES, MAX_PUSH_BYTES, PushResult, Remote, RemoteDoc,
    RemoteError, RemoteFile, is_file_id, is_md5_hex, md5_hex, validate_push,
};
use crate::sealed::PlainView;
use crate::wire::SignOutReason;

#[derive(Clone, Debug)]
struct Stored {
    rev: String,
    seq: i64,
    deleted: bool,
    fields: Map<String, Value>,
}

#[derive(Default)]
struct State {
    docs: BTreeMap<String, Stored>,
    /// The change-sequence counter: the seq of the last accepted write.
    counter: i64,
    files: HashMap<String, RemoteFile>,
    drop_push_responses: usize,
    stats: MemoryStats,
}

/// Request counts, for tests that check what a sync sent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryStats {
    pub pushes: usize,
    pub pushed_docs: usize,
    pub changes: usize,
    pub file_puts: usize,
    pub file_gets: usize,
}

/// A sync server held in memory. It follows the Worker's rules exactly: a
/// pushed document is accepted iff its `base_rev` is the server's current
/// revision (or the server has none and `base_rev` is null); every accepted
/// write takes the next value of one global sequence; documents in a push are
/// applied in order; accepting a file tombstone drops the file's bytes.
#[derive(Default)]
pub struct MemoryRemote {
    state: Mutex<State>,
}

impl MemoryRemote {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("memory remote poisoned")
    }

    /// Every document the server has, including tombstones, in `seq` order.
    pub fn docs(&self) -> Vec<RemoteDoc> {
        let state = self.state();
        let mut docs: Vec<RemoteDoc> = state.docs.iter().map(|(id, d)| remote_doc(id, d)).collect();
        docs.sort_by_key(|d| d.seq);
        docs
    }

    /// The highest change sequence handed out so far.
    pub fn last_seq(&self) -> i64 {
        self.state().counter
    }

    /// The stored bytes of attachment `id`.
    pub fn file(&self, id: &str) -> Option<RemoteFile> {
        self.state().files.get(id).cloned()
    }

    pub fn stats(&self) -> MemoryStats {
        self.state().stats
    }

    /// The next `n` pushes are applied but answer with a transport error, as
    /// when the connection drops after the server committed.
    pub fn drop_next_push_responses(&self, n: usize) {
        self.state().drop_push_responses = n;
    }
}

fn remote_doc(id: &str, d: &Stored) -> RemoteDoc {
    RemoteDoc {
        id: id.to_owned(),
        rev: d.rev.clone(),
        deleted: d.deleted,
        fields: d.fields.clone(),
        seq: d.seq,
    }
}

fn status(status: u16, message: impl Into<String>) -> RemoteError {
    RemoteError::Status {
        status,
        message: message.into(),
    }
}

impl Remote for MemoryRemote {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let body = serde_json::to_vec(&serde_json::json!({ "docs": docs }))
            .map_err(|e| RemoteError::Protocol(e.to_string()))?;
        if body.len() > MAX_PUSH_BYTES {
            return Err(status(413, "The push is too large."));
        }
        validate_push(docs).map_err(|(code, message)| status(code, message))?;

        let mut state = self.state();
        state.stats.pushes += 1;
        state.stats.pushed_docs += docs.len();
        let mut results = Vec::with_capacity(docs.len());
        for doc in docs {
            let current = state.docs.get(&doc.id);
            let accepted = match current {
                None => doc.base_rev.is_none(),
                Some(current) => doc.base_rev.as_deref() == Some(current.rev.as_str()),
            };
            if !accepted {
                results.push(PushResult::Conflict {
                    id: doc.id.clone(),
                    current: current.map(|c| SyncDoc {
                        id: doc.id.clone(),
                        rev: c.rev.clone(),
                        deleted: c.deleted,
                        fields: c.fields.clone(),
                        base_rev: None,
                    }),
                });
                continue;
            }
            state.counter += 1;
            let seq = state.counter;
            state.docs.insert(
                doc.id.clone(),
                Stored {
                    rev: doc.rev.clone(),
                    seq,
                    deleted: doc.deleted,
                    fields: doc.fields.clone(),
                },
            );
            if doc.deleted && is_file_id(&doc.id) {
                state.files.remove(&doc.id);
            }
            results.push(PushResult::Ok {
                id: doc.id.clone(),
                seq,
            });
        }
        if state.drop_push_responses > 0 {
            state.drop_push_responses -= 1;
            return Err(RemoteError::Transport(
                "connection reset (simulated)".into(),
            ));
        }
        Ok(results)
    }

    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        if since < 0 {
            return Err(status(400, "since must be a non-negative integer."));
        }
        if !(1..=MAX_CHANGES_LIMIT).contains(&limit) {
            return Err(status(
                400,
                format!("limit must be between 1 and {MAX_CHANGES_LIMIT}."),
            ));
        }
        let mut state = self.state();
        state.stats.changes += 1;
        let mut docs: Vec<RemoteDoc> = state
            .docs
            .iter()
            .filter(|(_, d)| d.seq > since)
            .map(|(id, d)| remote_doc(id, d))
            .collect();
        docs.sort_by_key(|d| d.seq);
        docs.truncate(limit);
        let last_seq = docs.last().map_or(since, |d| d.seq);
        Ok(ChangesPage { docs, last_seq })
    }

    fn put_file(
        &self,
        id: &str,
        claimed_md5: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError> {
        if !is_file_id(id) {
            return Err(status(400, "Not a file ID."));
        }
        if !is_md5_hex(claimed_md5) {
            return Err(status(400, "x-kanso-md5 must be 32 hex digits."));
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(status(413, "The file is too large."));
        }
        let actual = md5_hex(bytes);
        if actual != claimed_md5.to_ascii_lowercase() {
            return Err(status(400, "The bytes do not match x-kanso-md5."));
        }
        let mut state = self.state();
        state.stats.file_puts += 1;
        state.files.insert(
            id.to_owned(),
            RemoteFile {
                md5_hex: actual,
                content_type: if content_type.is_empty() {
                    "application/octet-stream".into()
                } else {
                    content_type.to_owned()
                },
                bytes: bytes.to_vec(),
            },
        );
        Ok(())
    }

    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        if !is_file_id(id) {
            return Err(status(400, "Not a file ID."));
        }
        let mut state = self.state();
        state.stats.file_gets += 1;
        Ok(state.files.get(id).cloned())
    }
}

// ---------------------------------------------------------------------------
// MemoryServer: the v2 test server (docs/ACCOUNTS.md › Modules, › Build order)
// ---------------------------------------------------------------------------

/// The `settings` rows `cloud/migrations/0002_accounts.sql` seeds, which
/// [`MemoryServer::new`] starts from, except that its sign-ups are open.
pub const DEFAULT_SETTINGS: [(&str, &str); 12] = [
    ("signup", "closed"),
    ("max_accounts", "50"),
    ("signups_per_day", "5"),
    ("quota_docs", "50000"),
    ("quota_doc_bytes", "104857600"),
    ("quota_files", "10000"),
    ("quota_file_bytes", "524288000"),
    ("writes_per_day", "100000"),
    ("global_doc_bytes", "6000000000"),
    ("global_file_bytes", "50000000000"),
    ("global_writes_per_day", "300000"),
    ("global_upload_bytes_per_day", "5000000000"),
];

/// Whether D1's `CHECK` on `settings` accepts this row.
pub fn is_valid_setting(name: &str, value: &str) -> bool {
    if name == "signup" {
        matches!(value, "open" | "closed")
            || (value.starts_with("only:") && (8..=37).contains(&value.len()))
    } else {
        (1..=15).contains(&value.len()) && value.bytes().all(|b| b.is_ascii_digit())
    }
}

/// The sign-up switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signup {
    Open,
    Closed,
    /// Only this (normalized) username may sign up.
    Only(String),
}

/// Default per-account quotas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
}

impl Limits {
    /// The migration's defaults.
    pub const DEFAULT: Limits = Limits {
        docs: 50_000,
        doc_bytes: 104_857_600,
        files: 10_000,
        file_bytes: 524_288_000,
    };
}

/// One account's quota overrides; `None` uses the default (D1's NULL).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuotaOverride {
    pub docs: Option<u64>,
    pub doc_bytes: Option<u64>,
    pub files: Option<u64>,
    pub file_bytes: Option<u64>,
}

/// A session row as the server holds it (names stay sealed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerDevice {
    pub id: String,
    pub sealed_name: Option<String>,
    pub created_at: i64,
    pub last_seen_at: i64,
    /// `None` while active.
    pub revoked: Option<SignOutReason>,
}

/// Requests answered, per route, and documents pushed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerStats {
    pub requests: BTreeMap<Route, usize>,
    pub pushed_docs: usize,
}

impl ServerStats {
    pub fn count(&self, route: Route) -> usize {
        self.requests.get(&route).copied().unwrap_or(0)
    }
}

#[derive(Default)]
#[allow(dead_code, reason = "read by the step 2a bodies")]
struct ServerState {
    settings: BTreeMap<String, String>,
    /// `None`: the wall clock.
    now: Option<i64>,
    drops: BTreeMap<Route, usize>,
    stats: ServerStats,
}

/// The test server: implements [`Backend`] with the Worker's sync, auth,
/// session, quota and limit rules (the migration's triggers included), in
/// memory, and runs the shared fixtures. Its auth hashes are SHA-256, as in
/// the Worker; its KDF floor and default are [`KdfParams::TEST`]. Tests and
/// snapshots only; `main.rs` never constructs it.
pub struct MemoryServer {
    state: Mutex<ServerState>,
}

impl Default for MemoryServer {
    fn default() -> Self {
        MemoryServer::new()
    }
}

impl MemoryServer {
    /// An empty server with the migration's settings, except that sign-ups
    /// are open, on the wall clock.
    pub fn new() -> MemoryServer {
        let mut settings: BTreeMap<String, String> = DEFAULT_SETTINGS
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        settings.insert("signup".into(), "open".into());
        MemoryServer {
            state: Mutex::new(ServerState {
                settings,
                ..ServerState::default()
            }),
        }
    }

    fn server(&self) -> std::sync::MutexGuard<'_, ServerState> {
        self.state.lock().expect("memory server poisoned")
    }

    /// Pins the server's clock (ms). Fixture steps set it to `start + t`.
    pub fn set_now(&self, now_ms: i64) {
        self.server().now = Some(now_ms);
    }

    /// Writes one `settings` row, as `UPDATE settings SET value = ? WHERE
    /// name = ?` would. Panics when D1's `CHECK` would refuse it.
    pub fn set_setting(&self, name: &str, value: &str) {
        assert!(
            is_valid_setting(name, value),
            "D1 refuses settings row {name} = {value}"
        );
        self.server()
            .settings
            .insert(name.to_owned(), value.to_owned());
    }

    pub fn set_signup(&self, signup: Signup) {
        let value = match signup {
            Signup::Open => "open".to_owned(),
            Signup::Closed => "closed".to_owned(),
            Signup::Only(name) => format!("only:{name}"),
        };
        self.set_setting("signup", &value);
    }

    pub fn set_max_accounts(&self, n: u64) {
        self.set_setting("max_accounts", &n.to_string());
    }

    pub fn set_default_quota(&self, limits: Limits) {
        self.set_setting("quota_docs", &limits.docs.to_string());
        self.set_setting("quota_doc_bytes", &limits.doc_bytes.to_string());
        self.set_setting("quota_files", &limits.files.to_string());
        self.set_setting("quota_file_bytes", &limits.file_bytes.to_string());
    }

    pub fn set_writes_per_day(&self, n: u64) {
        self.set_setting("writes_per_day", &n.to_string());
    }

    /// Sets an account's quota overrides (all four columns).
    pub fn set_quota(&self, username: &str, quota: QuotaOverride) {
        crate::stub!("2a", username, quota)
    }

    /// Sets `disabled_at`, like `admin disable`.
    pub fn disable(&self, username: &str) {
        crate::stub!("2a", username)
    }

    /// Clears `disabled_at`.
    pub fn enable(&self, username: &str) {
        crate::stub!("2a", username)
    }

    /// Every session row of the account, active and revoked.
    pub fn devices(&self, username: &str) -> Vec<ServerDevice> {
        crate::stub!("2a", username)
    }

    /// Revokes every active session of the account with `reason`.
    pub fn revoke_all(&self, username: &str, reason: SignOutReason) {
        crate::stub!("2a", username, reason)
    }

    /// The cron's expiry: revokes (`expired`) active sessions unseen for at
    /// least `unseen_for`.
    pub fn expire_sessions(&self, unseen_for: Duration) {
        crate::stub!("2a", unseen_for)
    }

    /// A new random epoch for the account, like `admin restored` (M1).
    pub fn set_epoch(&self, username: &str) {
        crate::stub!("2a", username)
    }

    /// Rewinds the account as a restore would: writes after `to_seq` are
    /// gone (documents last written after it disappear) and `last_seq`
    /// becomes `to_seq`; the epoch stays, so clients see `head_seq` drop.
    pub fn rewind(&self, username: &str, to_seq: i64) {
        crate::stub!("2a", username, to_seq)
    }

    /// The next `n` requests to `route` are applied but answer with a
    /// transport error, as when the connection drops after the server
    /// committed.
    pub fn drop_next_responses(&self, route: Route, n: usize) {
        self.server().drops.insert(route, n);
    }

    pub fn stats(&self) -> ServerStats {
        self.server().stats.clone()
    }

    /// The account of `session`, decrypted with its keys.
    pub fn plain_view(&self, session: &Session) -> PlainView {
        crate::stub!("2b", session)
    }
}

impl Backend for MemoryServer {
    fn api(&self, server: &str) -> Box<dyn Api + '_> {
        crate::stub!("2a", server)
    }

    fn kdf_floor(&self) -> KdfParams {
        KdfParams::TEST
    }

    fn kdf_default(&self) -> KdfParams {
        KdfParams::TEST
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;
    use proptest::prelude::*;

    /// `DEFAULT_SETTINGS` is exactly what the migration seeds.
    #[test]
    fn default_settings_mirror_the_migration() {
        let sql = include_str!("../../../cloud/migrations/0002_accounts.sql");
        let start = sql.find("INSERT INTO settings").unwrap();
        let end = start + sql[start..].find(';').unwrap();
        let seeded: Vec<(String, String)> = sql[start..end]
            .lines()
            .filter_map(|line| {
                let line = line.trim().trim_end_matches(',');
                let inner = line.strip_prefix("('")?.strip_suffix("')")?;
                let (name, value) = inner.split_once("', '")?;
                Some((name.to_owned(), value.to_owned()))
            })
            .collect();
        let expected: Vec<(String, String)> = DEFAULT_SETTINGS
            .iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect();
        assert_eq!(seeded, expected);
        let limits = Limits::DEFAULT;
        for (name, value) in [
            ("quota_docs", limits.docs),
            ("quota_doc_bytes", limits.doc_bytes),
            ("quota_files", limits.files),
            ("quota_file_bytes", limits.file_bytes),
        ] {
            assert!(
                DEFAULT_SETTINGS.contains(&(name, value.to_string().as_str())),
                "{name}"
            );
        }
    }

    #[test]
    fn a_new_server_takes_sign_ups() {
        let server = MemoryServer::new();
        assert_eq!(server.server().settings["signup"], "open");
        server.set_signup(Signup::Only("alice".into()));
        assert_eq!(server.server().settings["signup"], "only:alice");
        assert_eq!(server.kdf_floor(), KdfParams::TEST);
        assert_eq!(server.stats(), ServerStats::default());
    }

    #[test]
    #[should_panic(expected = "D1 refuses")]
    fn settings_d1_refuses_are_refused() {
        MemoryServer::new().set_setting("max_accounts", "12a");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: the settings check agrees with D1's CHECK, spelled out
        /// with explicit cases for each branch.
        #[test]
        fn prop_settings_check_matches_d1(value in "(only:)?[a-z0-9]{0,40}") {
            let signup_ok = value == "open" || value == "closed"
                || (value.starts_with("only:") && value.len() >= 8 && value.len() <= 37);
            prop_assert_eq!(is_valid_setting("signup", &value), signup_ok);
            let numeric_ok = !value.is_empty() && value.len() <= 15 && value.chars().all(|c| c.is_ascii_digit());
            prop_assert_eq!(is_valid_setting("max_accounts", &value), numeric_ok);
        }
    }
}
