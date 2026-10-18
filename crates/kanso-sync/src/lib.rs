//! Kanso's sync client.
//!
//! Today ([`sync_once`]) it syncs one plaintext library behind one shared
//! token: the Worker (`cloud/`) holds every document's current revision in
//! D1 and attachment bytes in R2; a sync pulls the changes after the stored
//! cursor, then pushes local ones; the store's bookkeeping (`synced_rev`,
//! last writer by `updatedAt` wins, losers kept in revisions) decides the
//! rest. [`HttpRemote`] talks to that Worker; [`MemoryRemote`] is an
//! in-memory server with the same rules.
//!
//! Accounts with end-to-end encryption replace it (docs/ACCOUNTS.md). Step 1
//! of that spec's build order froze their contract here: [`account`]'s
//! functions, [`Session`], [`Backend`] ([`HttpBackend`], [`MemoryServer`]),
//! [`wire`] and [`api`] shapes, [`binding`], [`keychain`], [`crypto`] and
//! [`username`] types, [`SyncReport`], [`SyncProblem`] and [`verify()`]. Bodies
//! marked with `stub!("2a" | "2b")` are the Rust builder's steps 2a and 2b.

/// Marks a body that a later step of docs/ACCOUNTS.md › Build order writes.
/// The remaining arguments are the parameters that body will use.
macro_rules! stub {
    ($step:literal $(, $arg:expr)* $(,)?) => {{
        $( let _ = &$arg; )*
        todo!(concat!("docs/ACCOUNTS.md › Build order, step ", $step))
    }};
}
pub(crate) use stub;

pub mod account;
pub mod api;
pub mod binding;
pub mod credentials;
pub mod crypto;
mod engine;
mod http;
pub mod keychain;
mod memory;
mod remote;
mod sealed;
pub mod username;
mod verify;
pub mod wire;

pub use account::{
    AccountError, DEFAULT_SERVER, DeviceInfo, Quota, Recovered, Session, SignedIn, SignedUp,
    SignupState, Step, Usage, change_password, delete_account, devices, flush_pending_logouts,
    new_recovery_key, recover, revoke_device, server_info, sign_in, sign_out, sign_up, usage,
};
pub use api::{Api, Backend, Route};
pub use binding::{
    BindError, Choice, DeviceIdentity, Flow, LocalLibrary, Machine, Plan, bind, device_identity,
    device_name, local_library, meta_keys, plan, unbind,
};
pub use crypto::{KdfParams, Password, PasswordError, RecoveryKey, RecoveryKeyError};
pub use engine::{
    CURSOR_KEY, Schedule, SyncError, SyncOptions, SyncProblem, SyncReport, batches, cursor,
    problem, report_problem, reset_cursor, sync, sync_once, sync_with,
};
pub use http::{HttpBackend, HttpRemote};
pub use kanso_store::SyncDoc;
pub use keychain::{Keychain, MemorySecrets, SecretError, SecretStore};
pub use memory::{
    DEFAULT_SETTINGS, Limits, MemoryRemote, MemoryServer, MemoryStats, QuotaOverride, ServerDevice,
    ServerStats, Signup, is_valid_setting,
};
pub use remote::{
    ChangesPage, DEFAULT_CHANGES_LIMIT, MAX_CHANGES_LIMIT, MAX_FIELDS_BYTES, MAX_FILE_BYTES,
    MAX_PUSH_BYTES, MAX_PUSH_DOCS, PushResult, Remote, RemoteDoc, RemoteError, RemoteFile,
    is_doc_id, is_file_id, is_md5_hex, is_rev, md5_hex, validate_push,
};
pub use sealed::PlainView;
pub use username::{UsernameError, check_new_username, is_reserved_username, normalize_username};
pub use verify::{
    Estimate, LegacyManifest, LegacyReport, LegacyRow, VerifyReport, estimate, manifest_sha256,
    verify,
};
pub use wire::{ApiError, QuotaKind, SessionToken, SignOutReason, SignupClosedReason};

// The app shares these across threads (docs/ACCOUNTS.md › Services).
const _: fn() = || {
    fn send_sync<T: Send + Sync + ?Sized>() {}
    send_sync::<Session>();
    send_sync::<dyn Backend>();
    send_sync::<HttpBackend>();
    send_sync::<MemoryServer>();
    send_sync::<dyn SecretStore>();
    send_sync::<Keychain>();
    send_sync::<MemorySecrets>();
    send_sync::<SyncOptions>();
    send_sync::<DeviceIdentity>();
};

/// Why a Worker URL was refused.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EndpointError {
    #[error("that isn't a URL")]
    Invalid,
    #[error("use an https:// URL (http:// only reaches this Mac: 127.0.0.1, localhost or [::1])")]
    Insecure,
    #[error("the URL can't contain a user name, password, query or fragment")]
    Extra,
}

/// The origin of a server URL, as the key derivations bind it:
/// `url::Url::parse(server)?.origin().ascii_serialization()`: lowercase,
/// punycode, default port omitted, path dropped. The server never supplies
/// it. Pure.
pub fn origin(server: &str) -> Result<String, EndpointError> {
    let url = url::Url::parse(server.trim()).map_err(|_| EndpointError::Invalid)?;
    match url.origin() {
        url::Origin::Tuple(..) => Ok(url.origin().ascii_serialization()),
        url::Origin::Opaque(_) => Err(EndpointError::Invalid),
    }
}

/// Checks a Worker URL and returns it normalized, without a trailing slash.
/// Only https is accepted, apart from plain http to this Mac (for `wrangler
/// dev`), so the access token never crosses the network unencrypted.
pub fn check_endpoint(input: &str) -> Result<String, EndpointError> {
    let url = url::Url::parse(input.trim()).map_err(|_| EndpointError::Invalid)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(EndpointError::Extra);
    }
    let loopback = match url.host() {
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        None => return Err(EndpointError::Invalid),
    };
    match url.scheme() {
        "https" => {}
        "http" if loopback => {}
        _ => return Err(EndpointError::Insecure),
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

/// Where the app syncs to. The token is kept separately, in the Keychain.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SyncConfig {
    /// The Worker's base URL, e.g. `https://kanso-sync.example.workers.dev`.
    pub endpoint: String,
}

impl SyncConfig {
    /// A config for `endpoint`, which must pass [`check_endpoint`].
    pub fn new(endpoint: &str) -> Result<Self, EndpointError> {
        Ok(SyncConfig {
            endpoint: check_endpoint(endpoint)?,
        })
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

#[cfg(test)]
mod endpoint_tests {
    use super::{EndpointError, check_endpoint, origin};
    use proptest::prelude::*;

    #[test]
    fn examples() {
        assert_eq!(
            check_endpoint(" https://kanso-sync.imngonii.workers.dev/ ").as_deref(),
            Ok("https://kanso-sync.imngonii.workers.dev")
        );
        assert_eq!(
            check_endpoint("http://127.0.0.1:8787").as_deref(),
            Ok("http://127.0.0.1:8787")
        );
        assert_eq!(
            check_endpoint("http://localhost:8787/").as_deref(),
            Ok("http://localhost:8787")
        );
        assert_eq!(
            check_endpoint("http://[::1]:8787").as_deref(),
            Ok("http://[::1]:8787")
        );
        assert_eq!(
            check_endpoint("http://127.0.0.1.evil.example"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("http://localhost.evil.example"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("http://example.com"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("ftp://example.com"),
            Err(EndpointError::Insecure)
        );
        assert_eq!(
            check_endpoint("https://user:pw@example.com"),
            Err(EndpointError::Extra)
        );
        assert_eq!(
            check_endpoint("https://example.com/?a=1"),
            Err(EndpointError::Extra)
        );
        assert_eq!(check_endpoint("https://"), Err(EndpointError::Invalid));
        assert_eq!(check_endpoint("example.com"), Err(EndpointError::Invalid));
        assert_eq!(check_endpoint("https://"), Err(EndpointError::Invalid));
    }

    #[test]
    fn origin_examples() {
        assert_eq!(
            origin("https://Kanso-Sync.imngonii.workers.dev/v2/").as_deref(),
            Ok("https://kanso-sync.imngonii.workers.dev")
        );
        assert_eq!(
            origin("https://example.com:443").as_deref(),
            Ok("https://example.com")
        );
        assert_eq!(
            origin("http://127.0.0.1:8787").as_deref(),
            Ok("http://127.0.0.1:8787")
        );
        assert_eq!(
            origin("https://bücher.example").as_deref(),
            Ok("https://xn--bcher-kva.example")
        );
        assert_eq!(origin("not a url"), Err(EndpointError::Invalid));
    }

    fn gen_url() -> impl Strategy<Value = String> {
        let scheme = prop_oneof![Just("http"), Just("https"), Just("HTTP"), Just("ftp")];
        let host = prop_oneof![
            Just("127.0.0.1".to_owned()),
            Just("localhost".to_owned()),
            Just("[::1]".to_owned()),
            "[a-z]{1,8}\\.[a-z]{2,3}",
            "(127\\.0\\.0\\.1|localhost)\\.[a-z]{1,8}\\.[a-z]{2,3}",
            "(127\\.0\\.0\\.1|localhost)@[a-z]{1,8}\\.[a-z]{2,3}",
        ];
        let port = prop::option::of(1u16..);
        let path = "(/[a-z]{0,4}){0,2}";
        (scheme, host, port, path).prop_map(|(scheme, host, port, path)| {
            let port = port.map(|p| format!(":{p}")).unwrap_or_default();
            format!("{scheme}://{host}{port}{path}")
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Refinement: an accepted URL is https, or http to a loopback host,
        /// and carries nothing that could hide where it really goes.
        #[test]
        fn prop_accepted_urls_are_https_or_this_mac(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                let parsed = url::Url::parse(&url).unwrap();
                let loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
                prop_assert!(parsed.scheme() == "https" || (parsed.scheme() == "http" && loopback), "{}", url);
                prop_assert!(parsed.username().is_empty() && parsed.password().is_none());
                prop_assert!(!url.ends_with('/'));
            }
        }

        /// Metamorphic: plain http is refused unless the host is this Mac,
        /// and switching to https makes any well-formed host acceptable.
        #[test]
        fn prop_http_only_reaches_this_mac(input in gen_url()) {
            let parsed = url::Url::parse(&input).unwrap();
            if parsed.scheme() == "http" && parsed.username().is_empty() {
                let loopback = matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
                prop_assert_eq!(check_endpoint(&input).is_ok(), loopback);
                let secure = input.replacen("http://", "https://", 1).replacen("HTTP://", "https://", 1);
                prop_assert!(check_endpoint(&secure).is_ok(), "{}", secure);
            }
        }

        /// Invariant: an accepted URL's origin is its scheme and host (and a
        /// non-default port) only, and the URL starts with it.
        #[test]
        fn prop_origins_drop_the_path(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                let o = origin(&url).unwrap();
                prop_assert!(url.starts_with(&o), "{} {}", url, o);
                prop_assert!(!o[o.find("://").unwrap() + 3..].contains('/'));
                prop_assert_eq!(origin(&o), Ok(o.clone()));
            }
        }

        /// Idempotence: checking an accepted URL again returns it unchanged.
        #[test]
        fn prop_check_endpoint_is_idempotent(input in gen_url()) {
            if let Ok(url) = check_endpoint(&input) {
                prop_assert_eq!(check_endpoint(&url), Ok(url.clone()));
            }
        }
    }
}
