//! Third-party sign-in and the account's ways in (docs/ACCOUNTS.md › Sign-in
//! methods › Sign in with Apple, Google and GitHub, › Client: modules, ›
//! Public API the app calls).
//!
//! Pure: the wire shapes of a provider flow, [`Provider`]'s open name,
//! Google's loopback rule, Apple's callback URL and the parse of a started
//! flow or a completed one. The network calls are in [`crate::account`]; the
//! app opens the browser. No ID token, access token or client secret ever
//! reaches this crate: the Worker exchanges and verifies them and answers a
//! session or a new-account ticket.

use std::fmt;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::AccountError;
use crate::api::{LoginDevice, WireDevice};
use crate::wire::SessionToken;

/// The custom scheme Apple's callback page sends the person back to
/// (`kanso://auth/apple?flow=<flow id>`).
pub const APPLE_CALLBACK_SCHEME: &str = "kanso";
pub const APPLE_CALLBACK_HOST: &str = "auth";
pub const APPLE_CALLBACK_PATH: &str = "/apple";

/// A provider's lowercase name (1–32 characters, `[a-z][a-z0-9_]*`). An open
/// name, not a closed enum: a server may advertise one this app doesn't show
/// yet, and an app that doesn't know one simply doesn't offer its button.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Provider {
    Apple,
    Google,
    Github,
    Other(String),
}

impl Provider {
    pub const APPLE: Provider = Provider::Apple;
    pub const GOOGLE: Provider = Provider::Google;
    pub const GITHUB: Provider = Provider::Github;

    /// The provider with this name, when the name is well formed.
    pub fn parse(name: &str) -> Option<Provider> {
        if !valid_name(name) {
            return None;
        }
        Some(match name {
            "apple" => Provider::Apple,
            "google" => Provider::Google,
            "github" => Provider::Github,
            _ => Provider::Other(name.to_owned()),
        })
    }

    pub fn as_str(&self) -> &str {
        match self {
            Provider::Apple => "apple",
            Provider::Google => "google",
            Provider::Github => "github",
            Provider::Other(name) => name,
        }
    }

    /// Whether the flow runs in the system browser and comes back through
    /// the `kanso` scheme (Apple) rather than a loopback listener (Google) or
    /// a device code (GitHub).
    pub fn callback_scheme(&self) -> bool {
        matches!(self, Provider::Apple)
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Provider {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Provider::parse(&name).ok_or_else(|| serde::de::Error::custom("not a provider name"))
    }
}

/// `^[a-z][a-z0-9_]{1,31}$`.
fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    (2..=32).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
}

// ------------------------------------------------------------------ the start

/// `POST /v2/auth/apple/start` 200.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleStart {
    pub flow_id: String,
    pub auth_url: String,
    pub expires_at: i64,
}

/// `POST /v2/auth/google/start` 200. The app keeps the verifier whose
/// challenge it sent; `state` and `nonce` are the Worker's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleStart {
    pub flow_id: String,
    pub auth_url: String,
    pub nonce: String,
    pub state: String,
    pub expires_at: i64,
}

/// `POST /v2/auth/github/start` 200: the device-code screen's contents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubStart {
    pub flow_id: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u64,
    pub expires_in: u64,
}

/// A started flow. The app opens what `auth_url` names, or shows the device
/// code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderStart {
    Apple(AppleStart),
    Google(GoogleStart),
    Github(GithubStart),
}

impl ProviderStart {
    pub fn flow_id(&self) -> &str {
        match self {
            ProviderStart::Apple(start) => &start.flow_id,
            ProviderStart::Google(start) => &start.flow_id,
            ProviderStart::Github(start) => &start.flow_id,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StartReply {
    Google(GoogleStart),
    Apple(AppleStart),
    Github(GithubStart),
}

impl ProviderStart {
    /// Reads a start answer; Google's shape is tried first because Apple's
    /// would otherwise accept it without its `nonce` and `state`.
    pub fn from_json(value: &[u8]) -> Result<ProviderStart, String> {
        let reply: StartReply =
            serde_json::from_slice(value).map_err(|e| format!("a provider start: {e}"))?;
        Ok(match reply {
            StartReply::Google(start) => ProviderStart::Google(start),
            StartReply::Apple(start) => ProviderStart::Apple(start),
            StartReply::Github(start) => ProviderStart::Github(start),
        })
    }
}

/// Apple's authorization page (docs/ACCOUNTS.md › Sign in with Apple).
pub const APPLE_AUTHORIZE: (&str, &str) = ("appleid.apple.com", "/auth/authorize");
/// Google's authorization pages live under this path.
pub const GOOGLE_AUTHORIZE: (&str, &str) = ("accounts.google.com", "/o/oauth2/");
/// GitHub's device page, exactly.
pub const GITHUB_DEVICE_PAGE: &str = "https://github.com/login/device";

/// Checks the page a provider start asks the app to open, before anything
/// opens it: the start is `provider`'s, and its URL is an https URL with no
/// user info, fragment, whitespace or control characters, on the
/// provider's own host (Apple's `appleid.apple.com/auth/authorize`, Google's
/// `accounts.google.com/o/oauth2/…`, exactly `https://github.com/login/device`)
/// or on the server's own origin (`http` only when that origin is this
/// device). Anything else (a file, a UNC path, `search-ms:`, an argument
/// that starts with `-`, another host) would have the system opener launch
/// whatever the server chose. Pure.
pub fn check_provider_start(
    provider: &Provider,
    start: &ProviderStart,
    server: &str,
) -> Result<(), AccountError> {
    let fits = match (provider, start) {
        (Provider::Apple, ProviderStart::Apple(start)) => {
            provider_page(&start.auth_url, server, |url| {
                url.host_str() == Some(APPLE_AUTHORIZE.0) && url.path() == APPLE_AUTHORIZE.1
            })
        }
        (Provider::Google, ProviderStart::Google(start)) => {
            provider_page(&start.auth_url, server, |url| {
                url.host_str() == Some(GOOGLE_AUTHORIZE.0)
                    && url.path().starts_with(GOOGLE_AUTHORIZE.1)
            })
        }
        (Provider::Github, ProviderStart::Github(start)) => {
            provider_page(&start.verification_uri, server, |url| {
                url.as_str() == GITHUB_DEVICE_PAGE
            })
        }
        _ => false,
    };
    if fits {
        Ok(())
    } else {
        Err(AccountError::Protocol(
            "the provider start names a page Kanso won't open".into(),
        ))
    }
}

/// Whether `text` is an https page `provider_host` accepts, or a page on
/// the server's own origin.
fn provider_page(text: &str, server: &str, provider_host: impl Fn(&Url) -> bool) -> bool {
    if text.is_empty() || text.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    let Ok(url) = Url::parse(text) else {
        return false;
    };
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return false;
    }
    let same_origin = Url::parse(server)
        .is_ok_and(|server| server.origin() == url.origin() && server.origin().is_tuple());
    match url.scheme() {
        "https" => (url.port().is_none() && provider_host(&url)) || same_origin,
        "http" => {
            same_origin
                && match url.host() {
                    Some(url::Host::Domain(host)) => host == "localhost",
                    Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
                    Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
                    None => false,
                }
        }
        _ => false,
    }
}

/// A device for a provider flow. A first sign-in has no account ID yet, so
/// it uses an ephemeral one for the flow; a link or a sign-in whose account
/// this library knows uses its own device.
impl LoginDevice {
    /// A device ID and proof that live for one flow.
    pub fn ephemeral() -> LoginDevice {
        LoginDevice {
            id: format!("d_{}", crate::crypto::hex(&crate::crypto::random::<16>())),
            proof: crate::crypto::b64(&crate::crypto::random::<32>()),
        }
    }

    /// This Mac's device for an account it knows: the same ID and proof every
    /// other route uses.
    pub fn for_identity(
        device: &crate::binding::DeviceIdentity,
        origin: &str,
        account_id: &str,
    ) -> LoginDevice {
        LoginDevice {
            id: device.device_id(account_id),
            proof: device.device_proof(origin, account_id).to_base64url(),
        }
    }
}

// ------------------------------------------------------------ the requests

/// `POST /v2/auth/apple/start` and `/v2/auth/github/start`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderStartRequest {
    pub device: LoginDevice,
}

/// `POST /v2/auth/google/start`: the app's loopback and its PKCE challenge
/// (S256). The verifier stays in the app until the finish.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleStartRequest {
    pub device: LoginDevice,
    pub redirect_uri: String,
    pub code_challenge: String,
}

/// `POST /v2/auth/apple/poll` and `/v2/auth/github/poll`. Apple's poll also
/// carries `s`, the secret from the callback URL that only the app process
/// which opened the sign-in sheet receives; GitHub's has none.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPollRequest {
    pub flow_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s: Option<String>,
    pub device: LoginDevice,
}

impl fmt::Debug for ProviderPollRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderPollRequest")
            .field("flow_id", &self.flow_id)
            .field("s", &self.s.as_ref().map(|_| "…"))
            .field("device", &self.device)
            .finish()
    }
}

/// `POST /v2/auth/google/finish`: the loopback's code and the verifier whose
/// challenge the start carried.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleFinishRequest {
    pub flow_id: String,
    pub code: String,
    pub code_verifier: String,
    pub device: LoginDevice,
}

/// `POST /v2/auth/social/signup`: the account a provider ticket names.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialSignupRequest {
    pub ticket: String,
    pub username: String,
    pub account_id: String,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
    pub key_id: String,
    pub device: crate::api::NewDevice,
}

/// A fresh proof for a sensitive change, as the wire carries it
/// (docs/ACCOUNTS.md › Changing a way in needs a fresh proof):
///
/// ```json
/// {"kind": "password", "auth_key"}
/// | {"kind": "passkey", "flow", "assertion": {"credential_id", "authenticator_data", "client_data_json", "signature"}}
/// | {"kind": "provider", "flow"}
/// ```
///
/// A provider proof is a completed proof flow; it can only unlink that same
/// provider, and setting a password, replacing the recovery key or deleting
/// with it also needs the recovery key. [`crate::Proof`] builds these.
///
/// [`ProofWire::Unknown`] is what a server reads for a `kind` it doesn't know,
/// or for a required proof that is missing or `null` ([`required_proof`]); it
/// answers 401 `reauth_required`. The client never sends it.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProofWire {
    Password {
        auth_key: String,
    },
    Passkey {
        flow: String,
        assertion: crate::api::PasskeyProofAssertion,
    },
    Provider {
        flow: String,
    },
    /// No proof, or a kind this protocol doesn't have.
    #[default]
    #[serde(other)]
    Unknown,
}

/// Reads a required `proof` that may be missing or `null` as
/// [`ProofWire::Unknown`], so the server answers `reauth_required` rather
/// than a malformed body. Used with `#[serde(default, deserialize_with)]`.
pub fn required_proof<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ProofWire, D::Error> {
    Ok(Option::<ProofWire>::deserialize(d)?.unwrap_or_default())
}

impl ProofWire {
    pub fn password(auth_key: String) -> ProofWire {
        ProofWire::Password { auth_key }
    }

    pub fn passkey(flow: String, assertion: crate::api::PasskeyProofAssertion) -> ProofWire {
        ProofWire::Passkey { flow, assertion }
    }

    pub fn provider(flow: String) -> ProofWire {
        ProofWire::Provider { flow }
    }

    /// The wire's `kind`.
    pub fn kind(&self) -> &'static str {
        match self {
            ProofWire::Password { .. } => "password",
            ProofWire::Passkey { .. } => "passkey",
            ProofWire::Provider { .. } => "provider",
            ProofWire::Unknown => "unknown",
        }
    }
}

impl fmt::Debug for ProofWire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofWire::Password { .. } => f
                .debug_struct("Password")
                .field("auth_key", &"<redacted>")
                .finish(),
            ProofWire::Passkey { flow, assertion } => f
                .debug_struct("Passkey")
                .field("flow", flow)
                .field("assertion", assertion)
                .finish(),
            ProofWire::Provider { flow } => f.debug_struct("Provider").field("flow", flow).finish(),
            ProofWire::Unknown => f.write_str("Unknown"),
        }
    }
}

impl Drop for ProofWire {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        if let ProofWire::Password { auth_key } = self {
            auth_key.zeroize();
        }
    }
}

/// `POST /v2/account/sign-in/identities/<provider>/start`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityStartRequest {
    pub intent: IdentityIntent,
    /// A password or passkey proof; required for `link`, absent for `proof`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<ProofWire>,
    /// Google's loopback, and the S256 challenge the app made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_challenge: Option<String>,
    pub device: LoginDevice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityIntent {
    Link,
    Proof,
}

/// `POST /v2/account/sign-in/identities/<provider>/unlink`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlinkRequest {
    #[serde(default, deserialize_with = "required_proof")]
    pub proof: ProofWire,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_auth_key: Option<String>,
}

impl fmt::Debug for GoogleFinishRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GoogleFinishRequest")
            .field("flow_id", &self.flow_id)
            .field("code", &"<redacted>")
            .field("code_verifier", &"<redacted>")
            .field("device", &self.device)
            .finish()
    }
}

impl Drop for GoogleFinishRequest {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        self.code.zeroize();
        self.code_verifier.zeroize();
    }
}

impl fmt::Debug for SocialSignupRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocialSignupRequest")
            .field("ticket", &self.ticket)
            .field("username", &self.username)
            .field("account_id", &self.account_id)
            .field("recovery_auth_key", &"<redacted>")
            .field("recovery_wrapped_key", &self.recovery_wrapped_key)
            .field("key_id", &self.key_id)
            .field("device", &self.device)
            .finish()
    }
}

impl Drop for SocialSignupRequest {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        self.recovery_auth_key.zeroize();
    }
}

impl fmt::Debug for UnlinkRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnlinkRequest")
            .field("proof", &self.proof)
            .field(
                "recovery_auth_key",
                &self.recovery_auth_key.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl Drop for UnlinkRequest {
    fn drop(&mut self) {
        use zeroize::Zeroize as _;
        if let Some(key) = &mut self.recovery_auth_key {
            key.zeroize();
        }
    }
}

// -------------------------------------------------------------- the outcome

/// A session from a provider flow. It never carries a password wrap or, for
/// a provider, a passkey wrap: the app holds the data key for this account
/// already, or the device is locked and needs a password, approval or the
/// recovery key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSession {
    pub account_id: String,
    pub username: String,
    pub token: SessionToken,
    pub key_id: String,
    pub epoch: String,
    pub has_password: bool,
    pub passkey_wrapped_key: Option<String>,
    pub evicted_device: Option<WireDevice>,
}

/// An identity that isn't linked: the app offers to create an account with
/// this ticket and suggested username.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAccount {
    pub ticket: String,
    pub suggested_username: String,
}

/// A finish or poll that has an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderSignIn {
    Session(ProviderSession),
    NewAccount(NewAccount),
}

/// One Apple or GitHub poll, or one Google finish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderStep {
    Session(ProviderSession),
    NewAccount(NewAccount),
    Pending { slow_down: bool },
}

#[derive(Deserialize)]
struct OutcomeReply {
    kind: Option<String>,
    status: Option<String>,
    slow_down: Option<bool>,
    account_id: Option<String>,
    username: Option<String>,
    token: Option<SessionToken>,
    key_id: Option<String>,
    epoch: Option<String>,
    has_password: Option<bool>,
    passkey_wrapped_key: Option<String>,
    evicted_device: Option<WireDevice>,
    ticket: Option<String>,
    suggested_username: Option<String>,
}

impl ProviderStep {
    /// Reads a finish or poll answer.
    pub fn from_json(value: &[u8]) -> Result<ProviderStep, String> {
        let reply: OutcomeReply =
            serde_json::from_slice(value).map_err(|e| format!("a provider answer: {e}"))?;
        if reply.kind.as_deref() == Some("session") {
            let session = ProviderSession {
                account_id: reply
                    .account_id
                    .ok_or_else(|| "a session without an account ID".to_owned())?,
                username: reply
                    .username
                    .ok_or_else(|| "a session without a username".to_owned())?,
                token: reply
                    .token
                    .ok_or_else(|| "a session without a token".to_owned())?,
                key_id: reply
                    .key_id
                    .ok_or_else(|| "a session without a key ID".to_owned())?,
                epoch: reply
                    .epoch
                    .ok_or_else(|| "a session without an epoch".to_owned())?,
                has_password: reply.has_password.unwrap_or(false),
                passkey_wrapped_key: reply.passkey_wrapped_key,
                evicted_device: reply.evicted_device,
            };
            return Ok(ProviderStep::Session(session));
        }
        if reply.kind.as_deref() == Some("new_account") {
            return Ok(ProviderStep::NewAccount(NewAccount {
                ticket: reply
                    .ticket
                    .ok_or_else(|| "a ticket-less new account".to_owned())?,
                suggested_username: reply.suggested_username.unwrap_or_default(),
            }));
        }
        if reply.status.as_deref() == Some("pending") {
            return Ok(ProviderStep::Pending {
                slow_down: reply.slow_down.unwrap_or(false),
            });
        }
        Err("an answer that is neither a session, a new account nor pending".to_owned())
    }

    /// The answer, when it has one.
    pub fn sign_in(self) -> Option<ProviderSignIn> {
        match self {
            ProviderStep::Session(session) => Some(ProviderSignIn::Session(session)),
            ProviderStep::NewAccount(account) => Some(ProviderSignIn::NewAccount(account)),
            ProviderStep::Pending { .. } => None,
        }
    }
}

// ------------------------------------------------------- linking and proofs

/// The answer to a link or proof flow's completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked {
        provider: Provider,
        created_at: i64,
    },
    Proof {
        provider: Provider,
    },
    /// Apple's or GitHub's flow isn't finished yet: poll again.
    Pending {
        slow_down: bool,
    },
}

#[derive(Deserialize)]
struct LinkReply {
    kind: Option<String>,
    status: Option<String>,
    slow_down: Option<bool>,
    provider: Option<Provider>,
    created_at: Option<i64>,
}

impl LinkOutcome {
    pub fn from_json(value: &[u8]) -> Result<LinkOutcome, String> {
        let reply: LinkReply =
            serde_json::from_slice(value).map_err(|e| format!("a link answer: {e}"))?;
        if reply.kind.is_none() && reply.status.as_deref() == Some("pending") {
            return Ok(LinkOutcome::Pending {
                slow_down: reply.slow_down.unwrap_or(false),
            });
        }
        let provider = reply
            .provider
            .ok_or_else(|| "a link answer without a provider".to_owned())?;
        match reply.kind.as_deref() {
            Some("linked") => Ok(LinkOutcome::Linked {
                provider,
                created_at: reply
                    .created_at
                    .ok_or_else(|| "a link answer without a time".to_owned())?,
            }),
            Some("proof") => Ok(LinkOutcome::Proof { provider }),
            _ => Err("an answer that is neither a link nor a proof".to_owned()),
        }
    }
}

// -------------------------------------------------------- the account's ways

/// A provider the account has linked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedProvider {
    pub provider: Provider,
    pub created_at: i64,
}

/// A passkey of the account, as the method list gives it. The credential ID
/// is bytes; the subject and public key are never returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasskeyInfo {
    pub credential_id: Vec<u8>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub prf: bool,
    pub transports: Vec<String>,
}

/// Whether the server runs passkey ceremonies and accepts security keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasskeyMethods {
    pub rp_id: String,
    pub security_key: bool,
}

/// What a server advertises (docs/ACCOUNTS.md › GET /v2/health). An old
/// server advertises nothing: username and password only, no live approval
/// channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerAuth {
    pub password: bool,
    pub passkeys: Option<PasskeyMethods>,
    pub providers: Vec<Provider>,
    pub account_events: Option<String>,
}

impl Default for ServerAuth {
    fn default() -> Self {
        ServerAuth {
            // The pre-advertisement protocol supported username/password.
            password: true,
            passkeys: None,
            providers: Vec::new(),
            account_events: None,
        }
    }
}

/// `GET /v2/account/sign-in`: the account's ways in. `available` is what the
/// server advertises; `linked` and `passkeys` are this account's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountMethods {
    pub has_password: bool,
    pub rp_id: Option<String>,
    pub available: Vec<Provider>,
    pub linked: Vec<LinkedProvider>,
    pub passkeys: Vec<PasskeyInfo>,
}

impl AccountMethods {
    /// The advertised providers this account hasn't linked, in advertisement
    /// order; the app offers a Link button for each.
    pub fn unlinked(&self) -> Vec<Provider> {
        self.available
            .iter()
            .filter(|provider| !self.linked.iter().any(|l| &l.provider == *provider))
            .cloned()
            .collect()
    }

    /// Whether removing `provider` would leave no password, no other
    /// identity and no passkey: the call needs the recovery key.
    pub fn last_method(&self, provider: &Provider) -> bool {
        !self.has_password
            && self.passkeys.is_empty()
            && self.linked.len() == 1
            && &self.linked[0].provider == provider
    }
}

/// The wire answer `GET /v2/account/sign-in` (the worker's exact shape).
#[derive(Deserialize)]
pub struct MethodsReply {
    pub username: String,
    pub has_password: bool,
    pub identities: Vec<IdentityRow>,
    pub passkeys: Vec<PasskeyRow>,
    pub recovery: bool,
}

#[derive(Deserialize)]
pub struct IdentityRow {
    pub provider: Provider,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct PasskeyRow {
    pub credential_id: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub prf: bool,
    pub transports: Vec<String>,
}

impl MethodsReply {
    pub fn methods(&self, available: Vec<Provider>, rp_id: Option<String>) -> AccountMethods {
        AccountMethods {
            has_password: self.has_password,
            rp_id,
            available,
            linked: self
                .identities
                .iter()
                .map(|row| LinkedProvider {
                    provider: row.provider.clone(),
                    created_at: row.created_at,
                })
                .collect(),
            passkeys: self
                .passkeys
                .iter()
                .filter_map(|row| {
                    crate::crypto::unb64(&row.credential_id).map(|credential_id| PasskeyInfo {
                        credential_id,
                        created_at: row.created_at,
                        last_used_at: row.last_used_at,
                        prf: row.prf,
                        transports: row.transports.clone(),
                    })
                })
                .collect(),
        }
    }
}

// ------------------------------------------------------------- the loopback

/// A Google loopback redirect: exactly `http://127.0.0.1:<port>`, with no
/// path, query or fragment, and a port in 1024–65535.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loopback {
    pub port: u16,
}

impl Loopback {
    pub fn new(port: u16) -> Option<Loopback> {
        (1024..=65535).contains(&port).then_some(Loopback { port })
    }

    /// Reads the exact shape. Anything else, including a trailing slash, a
    /// path or a leading zero in the port, is refused.
    pub fn parse(text: &str) -> Option<Loopback> {
        let url = Url::parse(text).ok()?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return None;
        }
        let port = url.port()?;
        let loopback = Loopback::new(port)?;
        (loopback.uri() == text).then_some(loopback)
    }

    pub fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

// -------------------------------------------------------- Apple's callback

/// What Apple's callback URL hands the app: the flow, and the secret the
/// Worker made at the callback for the app's poll.
#[derive(Clone, PartialEq, Eq)]
pub struct AppleCallback {
    pub flow: String,
    pub secret: String,
}

impl fmt::Debug for AppleCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppleCallback")
            .field("flow", &self.flow)
            .finish_non_exhaustive()
    }
}

fn is_b64_32(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// The flow ID and secret in `kanso://auth/apple?flow=<flow id>&s=<secret>`,
/// when the URL is exactly the callback the Worker's page makes.
pub fn apple_callback_flow(url: &str) -> Option<AppleCallback> {
    let url = Url::parse(url).ok()?;
    if url.scheme() != APPLE_CALLBACK_SCHEME
        || url.host_str() != Some(APPLE_CALLBACK_HOST)
        || url.path() != APPLE_CALLBACK_PATH
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let mut pairs = url.query_pairs();
    let (flow_key, flow) = pairs.next()?;
    let (secret_key, secret) = pairs.next()?;
    if flow_key != "flow" || secret_key != "s" || pairs.next().is_some() {
        return None;
    }
    let callback = AppleCallback {
        flow: flow.into_owned(),
        secret: secret.into_owned(),
    };
    (is_b64_32(&callback.flow) && is_b64_32(&callback.secret)).then_some(callback)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn a_link_poll_can_be_pending_linked_or_a_proof() {
        assert_eq!(
            LinkOutcome::from_json(br#"{"status":"pending"}"#),
            Ok(LinkOutcome::Pending { slow_down: false })
        );
        assert_eq!(
            LinkOutcome::from_json(br#"{"status":"pending","slow_down":true}"#),
            Ok(LinkOutcome::Pending { slow_down: true })
        );
        assert_eq!(
            LinkOutcome::from_json(br#"{"kind":"linked","provider":"github","created_at":5}"#),
            Ok(LinkOutcome::Linked {
                provider: Provider::Github,
                created_at: 5
            })
        );
        assert_eq!(
            LinkOutcome::from_json(br#"{"kind":"proof","provider":"apple"}"#),
            Ok(LinkOutcome::Proof {
                provider: Provider::Apple
            })
        );
        assert!(LinkOutcome::from_json(br#"{"status":"done"}"#).is_err());
    }

    #[test]
    fn provider_names_are_open_but_well_formed() {
        assert_eq!(Provider::parse("apple"), Some(Provider::Apple));
        assert_eq!(Provider::parse("github"), Some(Provider::Github));
        assert_eq!(
            Provider::parse("microsoft"),
            Some(Provider::Other("microsoft".into()))
        );
        let too_long = "x".repeat(33);
        for bad in [
            "",
            "a",
            "Apple",
            "1password",
            "two words",
            "a-b",
            too_long.as_str(),
        ] {
            assert_eq!(Provider::parse(bad), None, "{bad}");
        }
    }

    proptest! {
        #[test]
        fn prop_provider_round_trips_through_its_name(name in "[a-z][a-z0-9_]{1,31}") {
            let provider = Provider::parse(&name).unwrap();
            prop_assert_eq!(provider.as_str(), name.as_str());
            let json = serde_json::to_string(&provider).unwrap();
            let back: Provider = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back, provider);
        }

        #[test]
        fn prop_loopback_matches_its_rule(port in 0u16..=65535u16, tail in "(/[^?]*|\\?[^#]*)?") {
            let text = format!("http://127.0.0.1:{port}{tail}");
            let valid = (1024..=65535).contains(&port) && tail.is_empty();
            prop_assert_eq!(Loopback::parse(&text).is_some(), valid, "{}", text);
            if let Some(loopback) = Loopback::new(port).filter(|_| valid) {
                prop_assert_eq!(loopback.uri(), text);
            }
        }
    }

    #[test]
    fn only_the_exact_callback_names_a_flow() {
        let flow = "A".repeat(43);
        let secret = "s".repeat(43);
        assert_eq!(
            apple_callback_flow(&format!("kanso://auth/apple?flow={flow}&s={secret}")),
            Some(AppleCallback {
                flow: flow.clone(),
                secret: secret.clone()
            })
        );
        for bad in [
            format!("kanso://auth/apple?flow={flow}"),
            format!("kanso://auth/apple?flow={flow}&s={secret}&x=1"),
            format!("kanso://auth/apple?s={secret}&flow={flow}"),
            format!("kanso://auth/apple?flow={flow}&flow={flow}"),
            format!("kanso://auth/apple?flow={flow}&s=short"),
            format!("kanso://auth/apple?flow={}&s={secret}", "short".repeat(8)),
            format!("https://auth/apple?flow={flow}&s={secret}"),
            format!("kanso://auth/apple/extra?flow={flow}&s={secret}"),
            format!("kanso://auth/apple?flow={flow}&s={secret}#x"),
            "kanso://auth/apple".to_owned(),
        ] {
            assert_eq!(apple_callback_flow(&bad), None, "{bad}");
        }
    }

    proptest! {
        /// Any well-formed flow and secret round-trip through the callback URL.
        #[test]
        fn prop_apple_callback_round_trips(
            flow in "[A-Za-z0-9_-]{43}",
            secret in "[A-Za-z0-9_-]{43}",
        ) {
            let parsed = apple_callback_flow(&format!("kanso://auth/apple?flow={flow}&s={secret}"));
            prop_assert_eq!(parsed, Some(AppleCallback { flow, secret }));
        }
    }

    fn apple(url: &str) -> ProviderStart {
        ProviderStart::Apple(AppleStart {
            flow_id: "f".into(),
            auth_url: url.into(),
            expires_at: 1,
        })
    }

    fn google(url: &str) -> ProviderStart {
        ProviderStart::Google(GoogleStart {
            flow_id: "f".into(),
            auth_url: url.into(),
            nonce: "n".into(),
            state: "s".into(),
            expires_at: 1,
        })
    }

    fn github(url: &str) -> ProviderStart {
        ProviderStart::Github(GithubStart {
            flow_id: "f".into(),
            user_code: "ABCD-EFGH".into(),
            verification_uri: url.into(),
            interval: 5,
            expires_in: 600,
        })
    }

    #[test]
    fn provider_pages_are_the_providers_own_or_the_servers() {
        let server = "https://sync.example.com";
        let ok = |provider: &Provider, start: &ProviderStart| {
            check_provider_start(provider, start, server).is_ok()
        };
        assert!(ok(
            &Provider::Apple,
            &apple("https://appleid.apple.com/auth/authorize?client_id=a&state=s")
        ));
        assert!(ok(
            &Provider::Google,
            &google("https://accounts.google.com/o/oauth2/v2/auth?client_id=g")
        ));
        assert!(ok(
            &Provider::Github,
            &github("https://github.com/login/device")
        ));
        // The server's own origin, as a development server's page.
        assert!(ok(
            &Provider::Apple,
            &apple("https://sync.example.com/apple")
        ));
        assert!(
            check_provider_start(
                &Provider::Github,
                &github("http://127.0.0.1:8787/device"),
                "http://127.0.0.1:8787"
            )
            .is_ok()
        );

        for bad in [
            "file:///C:/Windows/System32/calc.exe",
            "\\\\evil\\share\\x.exe",
            "search-ms:query=x",
            "ms-settings:",
            "-aflag",
            " https://appleid.apple.com/auth/authorize",
            "https://appleid.apple.com/auth/authorize\n",
            "https://user@appleid.apple.com/auth/authorize",
            "https://appleid.apple.com:8443/auth/authorize",
            "https://appleid.apple.com/auth/authorize#x",
            "https://appleid.apple.com.evil.example/auth/authorize",
            "https://appleid.apple.com/other",
            "http://appleid.apple.com/auth/authorize",
            "https://evil.example/auth/authorize",
            "http://sync.example.com/apple",
            "",
        ] {
            assert!(!ok(&Provider::Apple, &apple(bad)), "{bad:?}");
        }
        assert!(!ok(
            &Provider::Google,
            &google("https://accounts.google.com/logout")
        ));
        assert!(!ok(
            &Provider::Github,
            &github("https://github.com/login/device/evil")
        ));
        assert!(!ok(
            &Provider::Github,
            &github("https://github.com/login/device?x=1")
        ));
        // A start for another provider than the one asked for.
        assert!(!ok(
            &Provider::Google,
            &apple("https://appleid.apple.com/auth/authorize")
        ));
        // Plain http to the server only when the server is this device.
        assert!(
            check_provider_start(
                &Provider::Github,
                &github("http://sync.example.com/device"),
                "http://sync.example.com"
            )
            .is_err()
        );
    }

    #[test]
    fn start_answers_are_told_apart_by_their_fields() {
        let apple = ProviderStart::from_json(
            br#"{"flow_id":"f","auth_url":"https://apple","expires_at":1}"#,
        )
        .unwrap();
        assert!(matches!(apple, ProviderStart::Apple(_)));
        let google = ProviderStart::from_json(
            br#"{"flow_id":"f","auth_url":"https://google","nonce":"n","state":"s","expires_at":1}"#,
        )
        .unwrap();
        assert!(matches!(google, ProviderStart::Google(_)));
        let github = ProviderStart::from_json(
            br#"{"flow_id":"f","user_code":"ABCD-EFGH","verification_uri":"https://github.com/login/device","interval":5,"expires_in":600}"#,
        )
        .unwrap();
        assert!(matches!(github, ProviderStart::Github(_)));
    }

    #[test]
    fn outcomes_parse_their_three_shapes() {
        let session = ProviderStep::from_json(
            br#"{"kind":"session","account_id":"a_x","username":"alice","token":"t","key_id":"k","epoch":"e","has_password":false,"passkey_wrapped_key":null,"evicted_device":null}"#,
        )
        .unwrap();
        assert!(matches!(session, ProviderStep::Session(_)));
        let account = ProviderStep::from_json(
            br#"{"kind":"new_account","ticket":"t","suggested_username":"quiet-otter-4821"}"#,
        )
        .unwrap();
        assert_eq!(
            account.sign_in(),
            Some(ProviderSignIn::NewAccount(NewAccount {
                ticket: "t".into(),
                suggested_username: "quiet-otter-4821".into()
            }))
        );
        let pending = ProviderStep::from_json(br#"{"status":"pending","slow_down":true}"#).unwrap();
        assert_eq!(pending, ProviderStep::Pending { slow_down: true });
    }

    #[test]
    fn a_device_is_ephemeral_or_this_macs() {
        let a = LoginDevice::ephemeral();
        let b = LoginDevice::ephemeral();
        assert_ne!(a.id, b.id);
        assert!(a.id.starts_with("d_") && a.id.len() == 34);
        assert_eq!(a.proof.len(), 43);

        let identity = crate::binding::DeviceIdentity::from_parts(
            "l_0123456789abcdef0123456789abcdef",
            "Mac",
            [9; 32],
        );
        let device = LoginDevice::for_identity(&identity, "https://sync.kanso.test", "a_x");
        assert_eq!(device.id, identity.device_id("a_x"));
        assert_eq!(
            device.proof,
            identity
                .device_proof("https://sync.kanso.test", "a_x")
                .to_base64url()
        );
    }

    #[test]
    fn the_last_method_rule_counts_every_way_in() {
        let provider = Provider::Github;
        let only = AccountMethods {
            has_password: false,
            rp_id: None,
            available: vec![provider.clone()],
            linked: vec![LinkedProvider {
                provider: provider.clone(),
                created_at: 1,
            }],
            passkeys: vec![],
        };
        assert!(only.last_method(&provider));
        assert!(
            !AccountMethods {
                has_password: true,
                ..only.clone()
            }
            .last_method(&provider)
        );
        assert!(
            !AccountMethods {
                passkeys: vec![PasskeyInfo {
                    credential_id: vec![1],
                    created_at: 1,
                    last_used_at: None,
                    prf: false,
                    transports: vec![]
                }],
                ..only.clone()
            }
            .last_method(&provider)
        );
        assert!(
            !AccountMethods {
                linked: vec![],
                ..only
            }
            .last_method(&provider)
        );
    }
}
