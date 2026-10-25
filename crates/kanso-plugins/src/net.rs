//! The network proxy behind `kanso.fetch`.
//!
//! A plugin's host process has no sockets. Every request it makes comes here
//! as a host call, and Kanso makes it. That is what lets the grant mean
//! something: a request goes out only to a host the person allowed, over
//! HTTPS, and never to this Mac or the network around it.
//!
//! Each request is checked in layers, and each layer fails closed:
//!
//! 1. [`check_url`] accepts only `https://` to a *name* on the plugin's
//!    allowlist: no address literals (however spelled), no user name or
//!    password, no port but 443.
//! 2. The [`Fetcher`] resolves the name itself and refuses if any address it
//!    gets back is not public ([`is_public`]).
//! 3. It connects to an address it just checked, not to the name again, so a
//!    DNS answer that changes between the check and the connection can't
//!    point it somewhere else. The certificate is still checked for the
//!    name.
//! 4. Redirects are followed here, never by the HTTP client, and every hop
//!    goes back through 1 to 3.
//! 5. Caps on what is sent and received, and a [`RateLimit`] per plugin.
//!
//! Every attempt, refused or not, lands in an [`EgressLog`] that the app
//! shows in Settings. The log keeps the path but never the query, since a
//! query may carry a secret.
//!
//! The resolver and the transport are traits so the whole policy is tested
//! without a network.

use std::collections::{BTreeSet, VecDeque};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::mpsc;
use std::time::Duration;

use kanso_plugin_host::protocol::{
    CODE_DENIED, CODE_LIMIT, CODE_TIMEOUT, CODE_UNAVAILABLE, Failure,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ureq::unversioned::resolver::{ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};

/// The most a response may carry. More than this is refused, not cut short.
pub const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;
/// The most a request body may carry.
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
/// The most headers a request may carry.
pub const MAX_HEADERS: usize = 32;
/// The longest a header name or value may be.
pub const MAX_HEADER_BYTES: usize = 8 * 1024;
/// How many redirects a request follows.
pub const MAX_REDIRECTS: usize = 5;
/// How long one hop may take, name lookup included.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Requests a plugin may make in a rolling minute.
pub const MAX_REQUESTS_PER_MINUTE: usize = 30;
/// Bytes a plugin may receive in a rolling day.
pub const MAX_BYTES_PER_DAY: u64 = 20 * 1024 * 1024;
/// How many entries the egress log keeps.
pub const EGRESS_LOG_ENTRIES: usize = 100;

const MINUTE_MS: i64 = 60_000;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Why a request was refused or failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum NetError {
    /// The URL isn't `https://`.
    #[error("only https addresses are allowed")]
    NotHttps,
    /// The URL doesn't parse, or carries a user name, password or port.
    #[error("that isn't an address this plugin may use")]
    BadUrl,
    /// The host isn't on the plugin's allowlist.
    #[error("{0} isn't a host this plugin was allowed")]
    NotAllowed(String),
    /// The host is a numeric address, not a name.
    #[error("numeric addresses aren't allowed")]
    LiteralAddress,
    /// The host's name resolves to an address inside this Mac or its network.
    #[error("{0} points at a private address")]
    PrivateAddress(String),
    /// The host's name didn't resolve.
    #[error("couldn't find {0}")]
    Unresolved(String),
    /// More redirects than are followed.
    #[error("too many redirects")]
    TooManyRedirects,
    /// The method isn't one of GET, POST, PUT, PATCH, DELETE, HEAD.
    #[error("that method isn't allowed")]
    BadMethod,
    /// A header that is malformed, too big, or one the plugin may not set.
    #[error("the header {0} isn't allowed")]
    BadHeader(String),
    /// A request or a response over its cap.
    #[error("that's more than a request or response may carry")]
    TooLarge,
    /// The plugin made too many requests, or received too much.
    #[error("this plugin has made too many requests")]
    RateLimited,
    /// The request ran out of time.
    #[error("the request ran out of time")]
    Timeout,
    /// The request is malformed in a way the other kinds don't say.
    #[error("the request is wrong: {0}")]
    BadRequest(String),
    /// The connection or the server failed.
    #[error("the request failed: {0}")]
    Failed(String),
}

impl NetError {
    /// The failure a plugin sees. Policy refusals are `denied`, caps and
    /// rates are `limit`, a slow request is `timeout`, a malformed one is
    /// `invalid`, and a broken connection is `unavailable`. Pure.
    pub fn failure(&self) -> Failure {
        let code = match self {
            NetError::NotHttps
            | NetError::BadUrl
            | NetError::NotAllowed(_)
            | NetError::LiteralAddress
            | NetError::PrivateAddress(_)
            | NetError::BadHeader(_) => CODE_DENIED,
            NetError::BadMethod | NetError::BadRequest(_) => {
                kanso_plugin_host::protocol::CODE_INVALID
            }
            NetError::TooManyRedirects | NetError::TooLarge | NetError::RateLimited => CODE_LIMIT,
            NetError::Timeout => CODE_TIMEOUT,
            NetError::Unresolved(_) | NetError::Failed(_) => CODE_UNAVAILABLE,
        };
        Failure::new(code, self.to_string())
    }
}

/// The hosts a plugin was granted: exact lowercase names, no wildcards.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetPolicy {
    /// The allowed hostnames.
    pub hosts: BTreeSet<String>,
}

/// Checks a URL against the allowlist and returns it cleaned: no fragment,
/// no trailing dot on the host. Only `https`, only a name on the list, no
/// user name or password, no port but 443. Any numeric address is refused,
/// including the spellings the URL parser turns into one (`127.1`,
/// `0x7f.1`, `[::1]`). Pure.
pub fn check_url(policy: &NetPolicy, url: &str) -> Result<url::Url, NetError> {
    let mut parsed = url::Url::parse(url).map_err(|_| NetError::BadUrl)?;
    if parsed.scheme() != "https" {
        return Err(NetError::NotHttps);
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(NetError::BadUrl);
    }
    let host = match parsed.host() {
        Some(url::Host::Domain(d)) => d.to_ascii_lowercase(),
        Some(_) => return Err(NetError::LiteralAddress),
        None => return Err(NetError::BadUrl),
    };
    let host = host.trim_end_matches('.').to_owned();
    if host.is_empty() {
        return Err(NetError::BadUrl);
    }
    if !policy.hosts.contains(&host) {
        return Err(NetError::NotAllowed(host));
    }
    if parsed.port().is_some_and(|p| p != 443) {
        return Err(NetError::BadUrl);
    }
    parsed.set_host(Some(&host)).map_err(|_| NetError::BadUrl)?;
    let _ = parsed.set_port(None);
    parsed.set_fragment(None);
    Ok(parsed)
}

fn v4_is_public(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224)
}

/// Whether an address is one a plugin's request may reach: not loopback,
/// private, link-local, shared, multicast, reserved, documentation or
/// benchmarking. IPv6 forms that carry an IPv4 address (mapped, compatible,
/// NAT64, 6to4) are public only if the IPv4 address inside is; Teredo is
/// refused whole. Of the rest of IPv6 only global unicast (`2000::/3`)
/// passes. Pure.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4_is_public(v4),
        IpAddr::V6(v6) => v6_is_public(v6),
    }
}

fn v6_is_public(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    let b = ip.octets();
    let embedded = |a: u8, b_: u8, c: u8, d: u8| v4_is_public(Ipv4Addr::new(a, b_, c, d));
    // ::ffff:0:0/96 (mapped) and ::/96 (compatible, which also holds ::).
    if s[..5] == [0, 0, 0, 0, 0] && (s[5] == 0xffff || s[5] == 0) {
        return embedded(b[12], b[13], b[14], b[15]);
    }
    // 64:ff9b::/96, NAT64.
    if s[0] == 0x64 && s[1] == 0xff9b && s[2..6] == [0, 0, 0, 0] {
        return embedded(b[12], b[13], b[14], b[15]);
    }
    // 64:ff9b:1::/48, local-use NAT64: the IPv4 address sits in bytes 6-7
    // and 9-10 (RFC 6052 for a /48 prefix).
    if s[0] == 0x64 && s[1] == 0xff9b && s[2] == 1 {
        return embedded(b[6], b[7], b[9], b[10]);
    }
    // 2002::/16, 6to4.
    if s[0] == 0x2002 {
        return embedded(b[2], b[3], b[4], b[5]);
    }
    // 2001::/32, Teredo.
    if s[0] == 0x2001 && s[1] == 0 {
        return false;
    }
    // 2001:db8::/32, documentation. 2001:2::/48, benchmarking.
    if s[0] == 0x2001 && (s[1] == 0xdb8 || (s[1] == 2 && s[2] == 0)) {
        return false;
    }
    // Everything but global unicast: this also drops loopback, unique-local,
    // link-local, site-local, multicast and the discard prefix.
    (s[0] & 0xe000) == 0x2000
}

/// Looks a name up. A trait so tests can answer without DNS.
pub trait Resolve {
    /// Every address for `host`.
    fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>>;
}

/// Resolves with the system's resolver, giving up after the request timeout.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemResolver;

impl Resolve for SystemResolver {
    fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>> {
        let (tx, rx) = mpsc::sync_channel(1);
        let host = host.to_owned();
        std::thread::spawn(move || {
            let found = (host.as_str(), port)
                .to_socket_addrs()
                .map(|addrs| addrs.collect::<Vec<_>>());
            let _ = tx.send(found);
        });
        match rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(found) => found,
            Err(_) => Err(std::io::ErrorKind::TimedOut.into()),
        }
    }
}

/// One request on its way out, already checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outgoing {
    /// The checked URL.
    pub url: url::Url,
    /// Uppercase method.
    pub method: String,
    /// Headers to send.
    pub headers: Vec<(String, String)>,
    /// The body, empty for none.
    pub body: Vec<u8>,
}

/// What came back from one hop, redirects not followed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Incoming {
    /// The status code.
    pub status: u16,
    /// Response headers, names as sent.
    pub headers: Vec<(String, String)>,
    /// The body, at most [`MAX_RESPONSE_BYTES`].
    pub body: Vec<u8>,
}

/// Sends one request to one address. A trait so tests need no network.
pub trait Transport {
    /// Connects to `addr` (not to the URL's name), verifies the certificate
    /// for the URL's host, and returns the response without following a
    /// redirect. A body over [`MAX_RESPONSE_BYTES`] is [`NetError::TooLarge`].
    fn send(&self, addr: SocketAddr, request: &Outgoing) -> Result<Incoming, NetError>;
}

/// Answers every lookup with the one address that was already checked, so
/// the client connects there and nowhere else.
#[derive(Debug)]
struct Pinned(SocketAddr);

impl Resolver for Pinned {
    fn resolve(
        &self,
        _uri: &ureq::http::Uri,
        _config: &ureq::config::Config,
        _timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let mut found = self.empty();
        found.push(self.0);
        Ok(found)
    }
}

/// The real transport: ureq over TLS, pinned to the checked address.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpsTransport;

fn map_ureq(error: ureq::Error) -> NetError {
    match error {
        ureq::Error::Timeout(_) => NetError::Timeout,
        ureq::Error::Io(e) if e.kind() == std::io::ErrorKind::TimedOut => NetError::Timeout,
        other => NetError::Failed(other.to_string().chars().take(200).collect()),
    }
}

impl Transport for HttpsTransport {
    fn send(&self, addr: SocketAddr, request: &Outgoing) -> Result<Incoming, NetError> {
        // No proxy from the environment: it would take the connection
        // somewhere the checks never saw. No redirects: the fetcher does
        // those. No error for a 4xx or 5xx: the plugin reads the status.
        let config = ureq::config::Config::builder()
            .https_only(true)
            .proxy(None)
            .max_redirects(0)
            .max_redirects_will_error(false)
            .http_status_as_error(false)
            .timeout_global(Some(REQUEST_TIMEOUT))
            .user_agent("Kanso-plugin")
            .build();
        let agent = ureq::Agent::with_parts(config, DefaultConnector::new(), Pinned(addr));
        let mut builder = ureq::http::Request::builder()
            .method(request.method.as_str())
            .uri(request.url.as_str());
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        let built = builder
            .body(request.body.clone())
            .map_err(|e| NetError::Failed(e.to_string()))?;
        let response = agent.run(built).map_err(map_ureq)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(n, v)| {
                (
                    n.as_str().to_owned(),
                    String::from_utf8_lossy(v.as_bytes()).into_owned(),
                )
            })
            .collect();
        let mut body = Vec::new();
        response
            .into_body()
            .into_reader()
            .take(MAX_RESPONSE_BYTES as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    NetError::Timeout
                } else {
                    NetError::Failed(e.to_string().chars().take(200).collect())
                }
            })?;
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(NetError::TooLarge);
        }
        Ok(Incoming {
            status,
            headers,
            body,
        })
    }
}

/// What a plugin asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchRequest {
    /// The address, as the plugin wrote it.
    pub url: String,
    /// Uppercase method.
    pub method: String,
    /// Headers the plugin set.
    pub headers: Vec<(String, String)>,
    /// A text body.
    pub body: Option<String>,
}

/// What a plugin gets back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchResponse {
    /// The status code.
    pub status: u16,
    /// Response headers, cookies left out.
    pub headers: Vec<(String, String)>,
    /// The body as text; bytes that aren't UTF-8 become U+FFFD.
    pub body: String,
}

impl FetchResponse {
    /// The JSON a plugin receives: `{status, headers, body}`, with headers
    /// as an object (a repeated name keeps its last value). Pure.
    pub fn to_json(&self) -> Value {
        let mut headers = Map::new();
        for (name, value) in &self.headers {
            headers.insert(name.to_ascii_lowercase(), Value::String(value.clone()));
        }
        serde_json::json!({
            "status": self.status,
            "headers": headers,
            "body": self.body,
        })
    }
}

const METHODS: [&str; 6] = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

/// Reads `{url, method?, headers?, body?}` from a plugin's call. The method
/// is uppercased; headers must be an object of strings. Pure.
pub fn request_from_args(args: &Value) -> Result<FetchRequest, NetError> {
    let object = args
        .as_object()
        .ok_or_else(|| NetError::BadRequest("the arguments must be an object".into()))?;
    let url = object
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| NetError::BadRequest("url must be a string".into()))?
        .to_owned();
    let method = match object.get("method") {
        None | Some(Value::Null) => "GET".to_owned(),
        Some(Value::String(m)) => m.to_ascii_uppercase(),
        Some(_) => return Err(NetError::BadMethod),
    };
    let mut headers = Vec::new();
    match object.get("headers") {
        None | Some(Value::Null) => {}
        Some(Value::Object(map)) => {
            for (name, value) in map {
                let Some(value) = value.as_str() else {
                    return Err(NetError::BadHeader(name.clone()));
                };
                headers.push((name.clone(), value.to_owned()));
            }
        }
        Some(_) => return Err(NetError::BadRequest("headers must be an object".into())),
    }
    let body = match object.get("body") {
        None | Some(Value::Null) => None,
        Some(Value::String(b)) => Some(b.clone()),
        Some(_) => return Err(NetError::BadRequest("body must be a string".into())),
    };
    Ok(FetchRequest {
        url,
        method,
        headers,
        body,
    })
}

fn is_token(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}

fn header_forbidden(lower: &str) -> bool {
    matches!(
        lower,
        "host"
            | "content-length"
            | "connection"
            | "transfer-encoding"
            | "upgrade"
            | "te"
            | "trailer"
            | "keep-alive"
            | "cookie"
            | "set-cookie"
    ) || lower.starts_with("proxy-")
}

/// Checks a request's method, headers and body against the caps and the
/// forbidden list, and returns the headers lowercased. Pure.
fn check_request(request: &FetchRequest) -> Result<Vec<(String, String)>, NetError> {
    if !METHODS.contains(&request.method.as_str()) {
        return Err(NetError::BadMethod);
    }
    if request.headers.len() > MAX_HEADERS {
        return Err(NetError::BadHeader("(too many headers)".into()));
    }
    let mut headers = Vec::with_capacity(request.headers.len());
    for (name, value) in &request.headers {
        let lower = name.to_ascii_lowercase();
        let shown: String = lower.chars().take(64).collect();
        if !is_token(name)
            || name.len() > MAX_HEADER_BYTES
            || value.len() > MAX_HEADER_BYTES
            || value.bytes().any(|b| b == b'\r' || b == b'\n' || b == 0)
            || header_forbidden(&lower)
        {
            return Err(NetError::BadHeader(shown));
        }
        headers.push((lower, value.clone()));
    }
    if let Some(body) = &request.body {
        if body.len() > MAX_REQUEST_BYTES {
            return Err(NetError::TooLarge);
        }
        if matches!(request.method.as_str(), "GET" | "HEAD") {
            return Err(NetError::BadRequest(
                "GET and HEAD can't carry a body".into(),
            ));
        }
    }
    Ok(headers)
}

/// One line of the egress log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Egress {
    /// When it was tried, ms since the epoch.
    pub at_ms: i64,
    /// The method.
    pub method: String,
    /// The host asked for, as far as the address could be read.
    pub host: String,
    /// The path, never the query: a query may carry a secret.
    pub url_path: String,
    /// The status, if a response came.
    pub status: Option<u16>,
    /// Body bytes sent.
    pub sent: u64,
    /// Body bytes received.
    pub received: u64,
    /// Why the attempt was refused or failed, if it was.
    pub refused: Option<String>,
}

/// The last [`EGRESS_LOG_ENTRIES`] attempts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressLog {
    entries: VecDeque<Egress>,
}

impl EgressLog {
    /// An empty log.
    pub fn new() -> EgressLog {
        EgressLog::default()
    }

    /// Adds an entry, dropping the oldest past the limit.
    pub fn push(&mut self, entry: Egress) {
        self.entries.push_back(entry);
        while self.entries.len() > EGRESS_LOG_ENTRIES {
            self.entries.pop_front();
        }
    }

    /// The entries, newest first.
    pub fn entries(&self) -> Vec<Egress> {
        self.entries.iter().rev().cloned().collect()
    }

    /// How many entries it holds.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Requests per minute and bytes per day for one plugin, over time the
/// caller supplies. Pure.
#[derive(Clone, Debug, Default)]
pub struct RateLimit {
    requests: VecDeque<i64>,
    received: VecDeque<(i64, u64)>,
}

impl RateLimit {
    /// A limit with nothing used.
    pub fn new() -> RateLimit {
        RateLimit::default()
    }

    fn prune(&mut self, now_ms: i64) {
        while self
            .requests
            .front()
            .is_some_and(|t| now_ms - *t >= MINUTE_MS)
        {
            self.requests.pop_front();
        }
        while self
            .received
            .front()
            .is_some_and(|(t, _)| now_ms - *t >= DAY_MS)
        {
            self.received.pop_front();
        }
    }

    /// Counts a request if it fits: fewer than 30 in the last minute and
    /// under 20 MiB received in the last day. Returns whether it fit.
    pub fn admit(&mut self, now_ms: i64) -> bool {
        self.prune(now_ms);
        let day: u64 = self.received.iter().map(|(_, n)| n).sum();
        if self.requests.len() >= MAX_REQUESTS_PER_MINUTE || day >= MAX_BYTES_PER_DAY {
            return false;
        }
        self.requests.push_back(now_ms);
        true
    }

    /// Counts bytes received.
    pub fn record_bytes(&mut self, now_ms: i64, n: u64) {
        self.prune(now_ms);
        self.received.push_back((now_ms, n));
    }
}

/// Makes a plugin's requests: checks, resolves, pins, follows redirects and
/// logs.
pub struct Fetcher<R: Resolve, T: Transport> {
    resolver: R,
    transport: T,
}

impl Fetcher<SystemResolver, HttpsTransport> {
    /// A fetcher on the system resolver and real HTTPS.
    pub fn system() -> Self {
        Fetcher::new(SystemResolver, HttpsTransport)
    }
}

fn log_entry(now_ms: i64, method: &str, url: Option<&url::Url>, raw: &str) -> Egress {
    let (host, url_path) = match url {
        Some(u) => (u.host_str().unwrap_or("").to_owned(), u.path().to_owned()),
        None => (
            url::Url::parse(raw)
                .ok()
                .and_then(|u| u.host_str().map(str::to_owned))
                .unwrap_or_default(),
            url::Url::parse(raw)
                .map(|u| u.path().to_owned())
                .unwrap_or_default(),
        ),
    };
    Egress {
        at_ms: now_ms,
        method: method.to_owned(),
        host,
        url_path,
        status: None,
        sent: 0,
        received: 0,
        refused: None,
    }
}

impl<R: Resolve, T: Transport> Fetcher<R, T> {
    /// A fetcher over the given resolver and transport.
    pub fn new(resolver: R, transport: T) -> Self {
        Fetcher {
            resolver,
            transport,
        }
    }

    /// The transport, for tests to inspect.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Resolves `host` and returns an address to connect to, refusing if any
    /// address it names is not public.
    fn pick_address(&self, host: &str) -> Result<SocketAddr, NetError> {
        let addrs = self.resolver.resolve(host, 443).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                NetError::Timeout
            } else {
                NetError::Unresolved(host.to_owned())
            }
        })?;
        if addrs.is_empty() {
            return Err(NetError::Unresolved(host.to_owned()));
        }
        if addrs.iter().any(|a| !is_public(a.ip())) {
            return Err(NetError::PrivateAddress(host.to_owned()));
        }
        // Prefer IPv4 when both are there; either was checked.
        let first = addrs.iter().find(|a| a.is_ipv4()).unwrap_or(&addrs[0]);
        Ok(SocketAddr::new(first.ip(), 443))
    }

    /// Makes the request the plugin asked for. `now_ms` is the time to log
    /// and to rate-limit by. Each hop, the first included, is checked and
    /// logged; a request that fails any check is never sent. Bytes received
    /// count against `limit`.
    pub fn fetch(
        &self,
        policy: &NetPolicy,
        request: FetchRequest,
        now_ms: i64,
        limit: &mut RateLimit,
        log: &mut EgressLog,
    ) -> Result<FetchResponse, NetError> {
        let mut method = request.method.clone();
        let refuse = |log: &mut EgressLog, raw: &str, error: NetError| {
            let mut entry = log_entry(now_ms, &request.method, None, raw);
            entry.refused = Some(error.to_string());
            log.push(entry);
            error
        };
        let mut headers = match check_request(&request) {
            Ok(h) => h,
            Err(e) => return Err(refuse(log, &request.url, e)),
        };
        if !limit.admit(now_ms) {
            return Err(refuse(log, &request.url, NetError::RateLimited));
        }
        let mut body = request.body.clone().unwrap_or_default().into_bytes();
        let mut current = request.url.clone();
        let mut hops = 0usize;
        loop {
            let mut entry = log_entry(now_ms, &method, None, &current);
            let url = match check_url(policy, &current) {
                Ok(u) => u,
                Err(e) => {
                    entry.refused = Some(e.to_string());
                    log.push(entry);
                    return Err(e);
                }
            };
            let host = url.host_str().unwrap_or("").to_owned();
            entry = log_entry(now_ms, &method, Some(&url), &current);
            let addr = match self.pick_address(&host) {
                Ok(a) => a,
                Err(e) => {
                    entry.refused = Some(e.to_string());
                    log.push(entry);
                    return Err(e);
                }
            };
            entry.sent = body.len() as u64;
            let outgoing = Outgoing {
                url: url.clone(),
                method: method.clone(),
                headers: headers.clone(),
                body: body.clone(),
            };
            let incoming = match self.transport.send(addr, &outgoing) {
                Ok(i) => i,
                Err(e) => {
                    entry.refused = Some(e.to_string());
                    log.push(entry);
                    return Err(e);
                }
            };
            entry.status = Some(incoming.status);
            entry.received = incoming.body.len() as u64;
            log.push(entry);
            limit.record_bytes(now_ms, incoming.body.len() as u64);

            let location = incoming
                .headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case("location"))
                .map(|(_, v)| v.clone());
            let redirect = matches!(incoming.status, 301 | 302 | 303 | 307 | 308);
            let Some(location) = location.filter(|_| redirect) else {
                return Ok(FetchResponse {
                    status: incoming.status,
                    headers: incoming
                        .headers
                        .into_iter()
                        .filter(|(n, _)| {
                            !n.eq_ignore_ascii_case("set-cookie")
                                && !n.eq_ignore_ascii_case("set-cookie2")
                        })
                        .collect(),
                    body: String::from_utf8_lossy(&incoming.body).into_owned(),
                });
            };
            if hops >= MAX_REDIRECTS {
                return Err(NetError::TooManyRedirects);
            }
            hops += 1;
            let next = url.join(&location).map_err(|_| NetError::BadUrl);
            let next = match next {
                Ok(n) => n,
                Err(e) => {
                    let mut entry = log_entry(now_ms, &method, None, &location);
                    entry.refused = Some(e.to_string());
                    log.push(entry);
                    return Err(e);
                }
            };
            let to_get = incoming.status == 303 && method != "HEAD"
                || matches!(incoming.status, 301 | 302) && method == "POST";
            if to_get {
                method = "GET".to_owned();
                body.clear();
                headers.retain(|(n, _)| n != "content-type" && n != "content-encoding");
            }
            if next.host_str() != url.host_str() {
                headers.retain(|(n, _)| n != "authorization");
            }
            current = next.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    const PUBLIC: &str = "93.184.216.34";

    fn policy(hosts: &[&str]) -> NetPolicy {
        NetPolicy {
            hosts: hosts.iter().map(|h| (*h).to_owned()).collect(),
        }
    }

    // --- check_url ---

    #[test]
    fn accepts_a_listed_https_host() {
        let url = check_url(
            &policy(&["api.example"]),
            "https://API.example./v1/x?q=1#frag",
        )
        .unwrap();
        assert_eq!(url.as_str(), "https://api.example/v1/x?q=1");
    }

    #[test]
    fn explicit_443_is_fine() {
        assert!(check_url(&policy(&["api.example"]), "https://api.example:443/").is_ok());
    }

    #[test]
    fn refuses_each_kind() {
        let p = policy(&["api.example"]);
        assert_eq!(
            check_url(&p, "http://api.example/"),
            Err(NetError::NotHttps)
        );
        assert_eq!(check_url(&p, "ftp://api.example/"), Err(NetError::NotHttps));
        assert_eq!(check_url(&p, "not a url"), Err(NetError::BadUrl));
        assert_eq!(
            check_url(&p, "https://u:p@api.example/"),
            Err(NetError::BadUrl)
        );
        assert_eq!(
            check_url(&p, "https://u@api.example/"),
            Err(NetError::BadUrl)
        );
        assert_eq!(
            check_url(&p, "https://api.example:8443/"),
            Err(NetError::BadUrl)
        );
        assert_eq!(
            check_url(&p, "https://other.example/"),
            Err(NetError::NotAllowed("other.example".into()))
        );
        assert_eq!(
            check_url(&p, "https://sub.api.example/"),
            Err(NetError::NotAllowed("sub.api.example".into()))
        );
        for literal in [
            "https://127.0.0.1/",
            "https://127.1/",
            "https://0x7f.1/",
            "https://2130706433/",
            "https://[::1]/",
            "https://[::ffff:127.0.0.1]/",
            "https://8.8.8.8/",
            "https://127.0.0.1./",
        ] {
            assert_eq!(
                check_url(&p, literal),
                Err(NetError::LiteralAddress),
                "{literal}"
            );
        }
    }

    proptest! {
        /// Invariant: a URL whose scheme isn't https is never accepted,
        /// whatever the host.
        #[test]
        fn never_accepts_non_https(scheme in "[a-z]{1,8}", host in "[a-z]{1,10}\\.example") {
            prop_assume!(scheme != "https");
            let p = policy(&[host.as_str()]);
            let result = check_url(&p, &format!("{scheme}://{host}/"));
            prop_assert!(result.is_err());
        }

        /// Invariant: a numeric address is never accepted as the host, in
        /// any of its spellings, even when the same text is on the list.
        #[test]
        fn never_accepts_an_address_literal(
            a in any::<u8>(), b in any::<u8>(), c in any::<u8>(), d in any::<u8>(),
            form in 0u8..5,
        ) {
            let n = u32::from_be_bytes([a, b, c, d]);
            let host = match form {
                0 => format!("{a}.{b}.{c}.{d}"),
                1 => format!("{n}"),
                2 => format!("0x{a:x}.{b}.{c}.{d}"),
                3 => format!("[::ffff:{a}.{b}.{c}.{d}]"),
                _ => format!("[2001:db8::{a:x}:{b:x}]"),
            };
            let p = policy(&[host.as_str(), "127.0.0.1", "localhost"]);
            let result = check_url(&p, &format!("https://{host}/"));
            prop_assert!(result.is_err());
        }

        /// Invariant: a user name or password never gets through.
        #[test]
        fn never_accepts_userinfo(user in "[a-z]{1,8}", pass in proptest::option::of("[a-z]{1,8}")) {
            let userinfo = match pass {
                Some(p) => format!("{user}:{p}"),
                None => user,
            };
            let result = check_url(&policy(&["api.example"]), &format!("https://{userinfo}@api.example/"));
            prop_assert!(result.is_err());
        }

        /// Invariant: no port but 443 gets through.
        #[test]
        fn never_accepts_another_port(port in 1u16..=65535) {
            prop_assume!(port != 443);
            let result = check_url(&policy(&["api.example"]), &format!("https://api.example:{port}/"));
            prop_assert!(result.is_err());
        }

        /// Invariant: a host not on the list never gets through, whatever
        /// the case or a trailing dot; and one on the list always does.
        #[test]
        fn host_must_be_listed(host in "[a-z]{2,10}\\.example", other in "[a-z]{2,10}\\.example") {
            let p = policy(&[host.as_str()]);
            let listed = check_url(&p, &format!("https://{}./x", host.to_uppercase()));
            prop_assert!(listed.is_ok());
            if other != host {
                let refused = check_url(&p, &format!("https://{other}/"));
                prop_assert!(refused.is_err());
            }
        }
    }

    // --- is_public ---

    #[test]
    fn known_public_addresses_pass() {
        for ip in [
            "8.8.8.8",
            "1.1.1.1",
            "93.184.216.34",
            "2606:4700::1111",
            "2001:4860:4860::8888",
        ] {
            assert!(is_public(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn known_private_addresses_fail() {
        for ip in [
            "0.0.0.0",
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
            "192.0.2.1",
            "198.51.100.1",
            "203.0.113.1",
            "198.18.0.1",
            "192.0.0.1",
            "::",
            "::1",
            "fc00::1",
            "fd12::1",
            "fe80::1",
            "fec0::1",
            "ff02::1",
            "2001:db8::1",
            "100::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "::127.0.0.1",
            "64:ff9b::7f00:1",
            "64:ff9b::a00:1",
            "2002:7f00:1::1",
            "2002:c0a8:101::",
            "2001:0:4136:e378:8000:63bf:3fff:fdd2",
        ] {
            assert!(!is_public(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn embedded_public_addresses_pass() {
        for ip in ["::ffff:8.8.8.8", "64:ff9b::808:808", "2002:808:808::1"] {
            assert!(is_public(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn local_use_nat64_reads_the_embedded_address() {
        // 64:ff9b:1::/48 with 127.0.0.1 at bytes 6-7 and 9-10.
        let bad = Ipv6Addr::from([
            0x00, 0x64, 0xff, 0x9b, 0x00, 0x01, 127, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        ]);
        assert!(!is_public(IpAddr::V6(bad)));
        let good = Ipv6Addr::from([
            0x00, 0x64, 0xff, 0x9b, 0x00, 0x01, 8, 8, 0, 8, 8, 0, 0, 0, 0, 0,
        ]);
        assert!(is_public(IpAddr::V6(good)));
    }

    /// An IPv4 address inside `base/prefix`, the host bits taken from `bits`.
    fn in_v4(base: [u8; 4], prefix: u32, bits: u32) -> IpAddr {
        let base = u32::from_be_bytes(base);
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        IpAddr::V4(Ipv4Addr::from((base & mask) | (bits & !mask)))
    }

    /// An IPv6 address inside `base/prefix`, the host bits taken from `bits`.
    fn in_v6(base: u128, prefix: u32, bits: u128) -> IpAddr {
        let mask = if prefix == 0 {
            0
        } else {
            u128::MAX << (128 - prefix)
        };
        IpAddr::V6(Ipv6Addr::from((base & mask) | (bits & !mask)))
    }

    const V4_REFUSED: [([u8; 4], u32); 14] = [
        ([127, 0, 0, 0], 8),
        ([0, 0, 0, 0], 8),
        ([10, 0, 0, 0], 8),
        ([172, 16, 0, 0], 12),
        ([192, 168, 0, 0], 16),
        ([169, 254, 0, 0], 16),
        ([100, 64, 0, 0], 10),
        ([224, 0, 0, 0], 4),
        ([240, 0, 0, 0], 4),
        ([192, 0, 2, 0], 24),
        ([198, 51, 100, 0], 24),
        ([203, 0, 113, 0], 24),
        ([198, 18, 0, 0], 15),
        ([192, 0, 0, 0], 24),
    ];

    proptest! {
        /// Invariant: every address inside a refused IPv4 range is not
        /// public, and the same address wrapped as IPv4-mapped,
        /// IPv4-compatible, NAT64 or 6to4 IPv6 isn't either.
        #[test]
        fn refused_v4_ranges_are_not_public(i in 0..V4_REFUSED.len(), bits in any::<u32>()) {
            let (base, prefix) = V4_REFUSED[i];
            let ip = in_v4(base, prefix, bits);
            prop_assert!(!is_public(ip));
            let IpAddr::V4(v4) = ip else { unreachable!() };
            prop_assert!(!is_public(IpAddr::V6(v4.to_ipv6_mapped())));
            let n = u32::from(v4) as u128;
            prop_assert!(!is_public(IpAddr::V6(Ipv6Addr::from(n))));
            prop_assert!(!is_public(IpAddr::V6(Ipv6Addr::from((0x64_ff9bu128 << 96) | n))));
            let six_to_four = (0x2002u128 << 112) | (n << 80) | (bits as u128 & 0xffff);
            prop_assert!(!is_public(IpAddr::V6(Ipv6Addr::from(six_to_four))));
        }

        /// Invariant: every address inside a refused IPv6 range is not
        /// public, Teredo included whatever it embeds.
        #[test]
        fn refused_v6_ranges_are_not_public(i in 0usize..9, bits in any::<u128>()) {
            let ranges: [(u128, u32); 9] = [
                (0, 128),
                (1, 128),
                (0xfc00u128 << 112, 7),
                (0xfe80u128 << 112, 10),
                (0xfec0u128 << 112, 10),
                (0xffu128 << 120, 8),
                ((0x2001u128 << 112) | (0xdb8u128 << 96), 32),
                (0x0100u128 << 112, 64),
                (0x2001u128 << 112, 32),
            ];
            let (base, prefix) = ranges[i];
            prop_assert!(!is_public(in_v6(base, prefix, bits)));
        }

        /// Invariant: a public IPv4 address stays public mapped into IPv6,
        /// and a private one never becomes public that way.
        #[test]
        fn mapping_keeps_the_verdict(a in any::<u8>(), b in any::<u8>(), c in any::<u8>(), d in any::<u8>()) {
            let v4 = Ipv4Addr::new(a, b, c, d);
            prop_assert_eq!(is_public(IpAddr::V4(v4)), is_public(IpAddr::V6(v4.to_ipv6_mapped())));
        }
    }

    // --- fakes ---

    struct FakeResolver {
        map: HashMap<String, Vec<IpAddr>>,
    }

    impl FakeResolver {
        fn public() -> FakeResolver {
            FakeResolver {
                map: HashMap::new(),
            }
        }
        fn with(mut self, host: &str, ips: &[IpAddr]) -> FakeResolver {
            self.map.insert(host.to_owned(), ips.to_vec());
            self
        }
    }

    impl Resolve for FakeResolver {
        fn resolve(&self, host: &str, port: u16) -> std::io::Result<Vec<SocketAddr>> {
            let ips = self
                .map
                .get(host)
                .cloned()
                .unwrap_or_else(|| vec![PUBLIC.parse().unwrap()]);
            Ok(ips
                .into_iter()
                .map(|ip| SocketAddr::new(ip, port))
                .collect())
        }
    }

    type Script = Box<dyn Fn(&Outgoing) -> Result<Incoming, NetError>>;

    struct FakeTransport {
        script: Script,
        sent: RefCell<Vec<(SocketAddr, Outgoing)>>,
    }

    impl FakeTransport {
        fn new(
            script: impl Fn(&Outgoing) -> Result<Incoming, NetError> + 'static,
        ) -> FakeTransport {
            FakeTransport {
                script: Box::new(script),
                sent: RefCell::new(Vec::new()),
            }
        }
    }

    impl Transport for FakeTransport {
        fn send(&self, addr: SocketAddr, request: &Outgoing) -> Result<Incoming, NetError> {
            self.sent.borrow_mut().push((addr, request.clone()));
            (self.script)(request)
        }
    }

    fn ok(body: &str) -> Incoming {
        Incoming {
            status: 200,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: body.as_bytes().to_vec(),
        }
    }

    fn redirect(status: u16, to: &str) -> Incoming {
        Incoming {
            status,
            headers: vec![("Location".into(), to.into())],
            body: Vec::new(),
        }
    }

    fn get(url: &str) -> FetchRequest {
        FetchRequest {
            url: url.into(),
            method: "GET".into(),
            headers: Vec::new(),
            body: None,
        }
    }

    fn run<R: Resolve, T: Transport>(
        fetcher: &Fetcher<R, T>,
        p: &NetPolicy,
        request: FetchRequest,
    ) -> (Result<FetchResponse, NetError>, EgressLog) {
        let mut limit = RateLimit::new();
        let mut log = EgressLog::new();
        let result = fetcher.fetch(p, request, 1_000, &mut limit, &mut log);
        (result, log)
    }

    // --- fetcher: examples ---

    #[test]
    fn a_plain_request_goes_to_the_checked_address() {
        let fetcher = Fetcher::new(FakeResolver::public(), FakeTransport::new(|_| Ok(ok("hi"))));
        let (result, log) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/v1?token=secret"),
        );
        let response = result.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "hi");
        let sent = fetcher.transport().sent.borrow();
        assert_eq!(sent[0].0, SocketAddr::new(PUBLIC.parse().unwrap(), 443));
        let entries = log.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].host, "api.example");
        assert_eq!(entries[0].url_path, "/v1");
        assert!(!serde_json::to_string(&entries).unwrap().contains("secret"));
        assert_eq!(entries[0].status, Some(200));
        assert_eq!(entries[0].received, 2);
    }

    #[test]
    fn a_private_resolution_is_refused_without_sending() {
        for bad in [
            "127.0.0.1",
            "10.0.0.5",
            "192.168.1.1",
            "169.254.169.254",
            "::1",
            "fd00::1",
        ] {
            let resolver = FakeResolver::public().with("api.example", &[bad.parse().unwrap()]);
            let fetcher = Fetcher::new(resolver, FakeTransport::new(|_| Ok(ok(""))));
            let (result, log) = run(
                &fetcher,
                &policy(&["api.example"]),
                get("https://api.example/"),
            );
            assert_eq!(
                result,
                Err(NetError::PrivateAddress("api.example".into())),
                "{bad}"
            );
            assert!(fetcher.transport().sent.borrow().is_empty());
            assert!(log.entries()[0].refused.is_some());
        }
    }

    #[test]
    fn one_private_address_among_public_ones_refuses() {
        let resolver = FakeResolver::public().with(
            "api.example",
            &[PUBLIC.parse().unwrap(), "10.0.0.1".parse().unwrap()],
        );
        let fetcher = Fetcher::new(resolver, FakeTransport::new(|_| Ok(ok(""))));
        let (result, _) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        assert_eq!(result, Err(NetError::PrivateAddress("api.example".into())));
        assert!(fetcher.transport().sent.borrow().is_empty());
    }

    #[test]
    fn no_addresses_is_unresolved() {
        let resolver = FakeResolver::public().with("api.example", &[]);
        let fetcher = Fetcher::new(resolver, FakeTransport::new(|_| Ok(ok(""))));
        let (result, _) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        assert_eq!(result, Err(NetError::Unresolved("api.example".into())));
    }

    #[test]
    fn an_unlisted_host_is_logged_and_never_sent() {
        let fetcher = Fetcher::new(FakeResolver::public(), FakeTransport::new(|_| Ok(ok(""))));
        let (result, log) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://evil.example/x"),
        );
        assert_eq!(result, Err(NetError::NotAllowed("evil.example".into())));
        assert!(fetcher.transport().sent.borrow().is_empty());
        assert_eq!(log.len(), 1);
        assert_eq!(log.entries()[0].host, "evil.example");
    }

    #[test]
    fn a_redirect_to_an_unlisted_host_is_refused() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|_| Ok(redirect(302, "https://evil.example/steal"))),
        );
        let (result, log) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        assert_eq!(result, Err(NetError::NotAllowed("evil.example".into())));
        assert_eq!(fetcher.transport().sent.borrow().len(), 1);
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn a_redirect_to_loopback_or_http_is_refused() {
        for (to, want) in [
            ("https://127.0.0.1/", NetError::LiteralAddress),
            ("https://[::1]/", NetError::LiteralAddress),
            ("http://api.example/", NetError::NotHttps),
        ] {
            let target = to.to_owned();
            let fetcher = Fetcher::new(
                FakeResolver::public(),
                FakeTransport::new(move |_| Ok(redirect(301, &target))),
            );
            let (result, _) = run(
                &fetcher,
                &policy(&["api.example"]),
                get("https://api.example/"),
            );
            assert_eq!(result, Err(want), "{to}");
            assert_eq!(fetcher.transport().sent.borrow().len(), 1);
        }
    }

    #[test]
    fn relative_redirects_resolve_against_the_current_url() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|r| {
                if r.url.path() == "/a/b" {
                    Ok(redirect(302, "../c?x=1"))
                } else {
                    Ok(ok(r.url.as_str()))
                }
            }),
        );
        let (result, _) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/a/b"),
        );
        assert_eq!(result.unwrap().body, "https://api.example/c?x=1");
    }

    #[test]
    fn too_many_redirects() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|_| Ok(redirect(302, "/again"))),
        );
        let (result, log) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        assert_eq!(result, Err(NetError::TooManyRedirects));
        assert_eq!(fetcher.transport().sent.borrow().len(), MAX_REDIRECTS + 1);
        assert_eq!(log.len(), MAX_REDIRECTS + 1);
    }

    #[test]
    fn post_redirects_become_get_and_authorization_stays_on_its_host() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|r| match r.url.host_str().unwrap() {
                "api.example" if r.url.path() == "/start" => Ok(redirect(302, "/next")),
                "api.example" => Ok(redirect(307, "https://cdn.example/file")),
                _ => Ok(ok("done")),
            }),
        );
        let request = FetchRequest {
            url: "https://api.example/start".into(),
            method: "POST".into(),
            headers: vec![
                ("Authorization".into(), "Bearer t".into()),
                ("Content-Type".into(), "text/plain".into()),
            ],
            body: Some("payload".into()),
        };
        let (result, _) = run(&fetcher, &policy(&["api.example", "cdn.example"]), request);
        assert!(result.is_ok());
        let sent = fetcher.transport().sent.borrow();
        assert_eq!(sent[0].1.method, "POST");
        assert_eq!(sent[0].1.body, b"payload");
        assert_eq!(sent[1].1.method, "GET");
        assert!(sent[1].1.body.is_empty());
        assert!(sent[1].1.headers.iter().any(|(n, _)| n == "authorization"));
        assert_eq!(sent[2].1.url.host_str(), Some("cdn.example"));
        assert!(!sent[2].1.headers.iter().any(|(n, _)| n == "authorization"));
    }

    #[test]
    fn a_307_keeps_the_method_and_body() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|r| {
                if r.url.path() == "/start" {
                    Ok(redirect(307, "/next"))
                } else {
                    Ok(ok("done"))
                }
            }),
        );
        let request = FetchRequest {
            url: "https://api.example/start".into(),
            method: "PUT".into(),
            headers: Vec::new(),
            body: Some("x".into()),
        };
        run(&fetcher, &policy(&["api.example"]), request).0.unwrap();
        let sent = fetcher.transport().sent.borrow();
        assert_eq!(sent[1].1.method, "PUT");
        assert_eq!(sent[1].1.body, b"x");
    }

    #[test]
    fn a_transport_failure_is_logged() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|_| Err(NetError::Timeout)),
        );
        let (result, log) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        assert_eq!(result, Err(NetError::Timeout));
        assert_eq!(log.len(), 1);
        assert!(log.entries()[0].refused.is_some());
        assert_eq!(log.entries()[0].status, None);
    }

    #[test]
    fn set_cookie_is_not_passed_back_and_body_is_lossy() {
        let fetcher = Fetcher::new(
            FakeResolver::public(),
            FakeTransport::new(|_| {
                Ok(Incoming {
                    status: 200,
                    headers: vec![
                        ("Set-Cookie".into(), "a=b".into()),
                        ("X-A".into(), "1".into()),
                    ],
                    body: vec![b'a', 0xff, b'b'],
                })
            }),
        );
        let (result, _) = run(
            &fetcher,
            &policy(&["api.example"]),
            get("https://api.example/"),
        );
        let response = result.unwrap();
        assert_eq!(response.headers, vec![("X-A".to_owned(), "1".to_owned())]);
        assert_eq!(response.body, "a\u{fffd}b");
        let json = response.to_json();
        assert_eq!(json["status"], 200);
        assert_eq!(json["headers"]["x-a"], "1");
    }

    #[test]
    fn request_caps_and_forbidden_headers() {
        let fetcher = Fetcher::new(FakeResolver::public(), FakeTransport::new(|_| Ok(ok(""))));
        let p = policy(&["api.example"]);
        let with = |m: &str, h: Vec<(&str, &str)>, body: Option<String>| FetchRequest {
            url: "https://api.example/".into(),
            method: m.into(),
            headers: h
                .into_iter()
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .collect(),
            body,
        };
        for name in [
            "Host",
            "content-length",
            "Connection",
            "Transfer-Encoding",
            "Upgrade",
            "TE",
            "Trailer",
            "Proxy-Authorization",
            "proxy-x",
            "Keep-Alive",
            "Cookie",
        ] {
            let (r, _) = run(&fetcher, &p, with("GET", vec![(name, "x")], None));
            assert!(matches!(r, Err(NetError::BadHeader(_))), "{name}");
        }
        let (r, _) = run(
            &fetcher,
            &p,
            with("GET", vec![("X-A", "a\r\nX-B: b")], None),
        );
        assert!(matches!(r, Err(NetError::BadHeader(_))));
        let (r, _) = run(&fetcher, &p, with("GET", vec![("bad name", "a")], None));
        assert!(matches!(r, Err(NetError::BadHeader(_))));
        let (r, _) = run(
            &fetcher,
            &p,
            with(
                "GET",
                vec![("x-a", &"a".repeat(MAX_HEADER_BYTES + 1))],
                None,
            ),
        );
        assert!(matches!(r, Err(NetError::BadHeader(_))));
        let many: Vec<(String, String)> = (0..=MAX_HEADERS)
            .map(|i| (format!("x-{i}"), "v".into()))
            .collect();
        let request = FetchRequest {
            headers: many,
            ..get("https://api.example/")
        };
        assert!(matches!(
            run(&fetcher, &p, request).0,
            Err(NetError::BadHeader(_))
        ));
        let (r, _) = run(
            &fetcher,
            &p,
            with("POST", vec![], Some("a".repeat(MAX_REQUEST_BYTES + 1))),
        );
        assert_eq!(r, Err(NetError::TooLarge));
        let (r, _) = run(&fetcher, &p, with("TRACE", vec![], None));
        assert_eq!(r, Err(NetError::BadMethod));
        let (r, _) = run(&fetcher, &p, with("get", vec![], None));
        assert_eq!(r, Err(NetError::BadMethod));
        let (r, _) = run(&fetcher, &p, with("GET", vec![], Some("x".into())));
        assert!(matches!(r, Err(NetError::BadRequest(_))));
        assert!(fetcher.transport().sent.borrow().is_empty());
    }

    #[test]
    fn rate_limit_refuses_the_thirty_first() {
        let fetcher = Fetcher::new(FakeResolver::public(), FakeTransport::new(|_| Ok(ok(""))));
        let p = policy(&["api.example"]);
        let mut limit = RateLimit::new();
        let mut log = EgressLog::new();
        for _ in 0..MAX_REQUESTS_PER_MINUTE {
            fetcher
                .fetch(&p, get("https://api.example/"), 5_000, &mut limit, &mut log)
                .unwrap();
        }
        let r = fetcher.fetch(&p, get("https://api.example/"), 5_000, &mut limit, &mut log);
        assert_eq!(r, Err(NetError::RateLimited));
        assert!(
            fetcher
                .fetch(
                    &p,
                    get("https://api.example/"),
                    65_001,
                    &mut limit,
                    &mut log
                )
                .is_ok()
        );
    }

    #[test]
    fn daily_bytes_cap() {
        let mut limit = RateLimit::new();
        assert!(limit.admit(0));
        limit.record_bytes(0, MAX_BYTES_PER_DAY);
        assert!(!limit.admit(1_000));
        assert!(limit.admit(DAY_MS + 1));
    }

    // --- request_from_args ---

    #[test]
    fn args_parse() {
        let r = request_from_args(&serde_json::json!({
            "url": "https://api.example/",
            "method": "post",
            "headers": {"X-A": "1"},
            "body": "b",
        }))
        .unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.headers, vec![("X-A".to_owned(), "1".to_owned())]);
        assert_eq!(r.body.as_deref(), Some("b"));
        let r = request_from_args(&serde_json::json!({"url": "https://api.example/"})).unwrap();
        assert_eq!(r.method, "GET");
    }

    #[test]
    fn bad_args_are_refused() {
        for args in [
            serde_json::json!("https://x"),
            serde_json::json!({}),
            serde_json::json!({"url": 3}),
            serde_json::json!({"url": "https://a/", "method": 3}),
            serde_json::json!({"url": "https://a/", "headers": {"a": 1}}),
            serde_json::json!({"url": "https://a/", "headers": ["a"]}),
            serde_json::json!({"url": "https://a/", "body": 5}),
        ] {
            assert!(request_from_args(&args).is_err(), "{args}");
        }
    }

    #[test]
    fn failures_map_to_codes() {
        use kanso_plugin_host::protocol::CODE_INVALID;
        let code = |e: NetError| e.failure().code;
        assert_eq!(code(NetError::NotHttps), CODE_DENIED);
        assert_eq!(code(NetError::NotAllowed("x".into())), CODE_DENIED);
        assert_eq!(code(NetError::LiteralAddress), CODE_DENIED);
        assert_eq!(code(NetError::PrivateAddress("x".into())), CODE_DENIED);
        assert_eq!(code(NetError::BadHeader("x".into())), CODE_DENIED);
        assert_eq!(code(NetError::TooLarge), CODE_LIMIT);
        assert_eq!(code(NetError::RateLimited), CODE_LIMIT);
        assert_eq!(code(NetError::TooManyRedirects), CODE_LIMIT);
        assert_eq!(code(NetError::Timeout), CODE_TIMEOUT);
        assert_eq!(code(NetError::Failed("x".into())), CODE_UNAVAILABLE);
        assert_eq!(code(NetError::Unresolved("x".into())), CODE_UNAVAILABLE);
        assert_eq!(code(NetError::BadMethod), CODE_INVALID);
    }

    #[test]
    fn the_log_keeps_the_last_hundred_newest_first() {
        let mut log = EgressLog::new();
        for i in 0..150 {
            log.push(Egress {
                at_ms: i,
                method: "GET".into(),
                host: "h".into(),
                url_path: "/".into(),
                status: Some(200),
                sent: 0,
                received: 0,
                refused: None,
            });
        }
        let entries = log.entries();
        assert_eq!(entries.len(), EGRESS_LOG_ENTRIES);
        assert_eq!(entries[0].at_ms, 149);
        assert_eq!(entries[99].at_ms, 50);
        let back: EgressLog = serde_json::from_str(&serde_json::to_string(&log).unwrap()).unwrap();
        assert_eq!(back, log);
    }

    // --- fetcher: properties ---

    #[derive(Clone, Copy, Debug)]
    enum Bad {
        Unlisted,
        V4Literal,
        V6Literal,
        Http,
        Private,
    }

    fn bad_url(kind: Bad) -> &'static str {
        match kind {
            Bad::Unlisted => "https://evil.example/x",
            Bad::V4Literal => "https://127.0.0.1/x",
            Bad::V6Literal => "https://[::1]/x",
            Bad::Http => "http://a.example/x",
            Bad::Private => "https://priv.example/x",
        }
    }

    fn bad_strategy() -> impl Strategy<Value = Bad> {
        prop_oneof![
            Just(Bad::Unlisted),
            Just(Bad::V4Literal),
            Just(Bad::V6Literal),
            Just(Bad::Http),
            Just(Bad::Private),
        ]
    }

    fn chain_fetcher(urls: Vec<String>) -> Fetcher<FakeResolver, FakeTransport> {
        // Each request is answered by a redirect to the next URL in the
        // chain, keyed by how many requests have been sent.
        let count = std::rc::Rc::new(RefCell::new(0usize));
        let urls = std::rc::Rc::new(urls);
        let resolver = FakeResolver::public().with("priv.example", &["10.9.8.7".parse().unwrap()]);
        let script = move |_: &Outgoing| {
            let mut n = count.borrow_mut();
            *n += 1;
            match urls.get(*n - 1) {
                Some(next) => Ok(redirect(302, next)),
                None => Ok(ok("end")),
            }
        };
        Fetcher::new(resolver, FakeTransport::new(script))
    }

    proptest! {
        /// Invariant: a resolver that returns a private address means the
        /// transport is never called, for any listed host and any private
        /// address among the answers.
        #[test]
        fn private_resolution_never_reaches_the_transport(
            idx in 0..V4_REFUSED.len(), bits in any::<u32>(), public_first in any::<bool>(),
        ) {
            let (base, prefix) = V4_REFUSED[idx];
            let private = in_v4(base, prefix, bits);
            let mut ips = vec![private];
            if public_first {
                ips.insert(0, PUBLIC.parse().unwrap());
            }
            let fetcher = Fetcher::new(
                FakeResolver::public().with("api.example", &ips),
                FakeTransport::new(|_| Ok(ok(""))),
            );
            let (result, log) = run(&fetcher, &policy(&["api.example"]), get("https://api.example/"));
            prop_assert!(matches!(result, Err(NetError::PrivateAddress(_))));
            prop_assert!(fetcher.transport().sent.borrow().is_empty());
            prop_assert_eq!(log.len(), 1);
        }

        /// Invariant: in a redirect chain where one hop is bad (unlisted,
        /// an address literal, plain http or a name that resolves to a
        /// private address), every hop before it is sent, that hop is not,
        /// and the log holds one entry per attempted hop.
        #[test]
        fn a_bad_hop_is_refused_before_it_is_sent(
            good in 0usize..=MAX_REDIRECTS, kind in bad_strategy(),
        ) {
            // Request 0 goes to a.example; `good` redirects stay on the list;
            // the next redirect is the bad one. `good` redirects mean
            // good + 1 sent requests before the bad hop; the bad one is the
            // (good + 1)th redirect, so it must fit within the limit.
            prop_assume!(good < MAX_REDIRECTS);
            let mut urls: Vec<String> = (0..good).map(|i| format!("https://b.example/{i}")).collect();
            urls.push(bad_url(kind).to_owned());
            let fetcher = chain_fetcher(urls);
            let (result, log) = run(&fetcher, &policy(&["a.example", "b.example", "priv.example"]), get("https://a.example/"));
            prop_assert!(result.is_err());
            prop_assert_eq!(fetcher.transport().sent.borrow().len(), good + 1);
            prop_assert_eq!(log.len(), good + 2);
            prop_assert!(log.entries()[0].refused.is_some());
        }

        /// Invariant: a chain of listed, public hops within the limit is
        /// followed to the end, with one log entry per hop, and a chain over
        /// the limit is refused.
        #[test]
        fn good_chains_follow_and_long_ones_stop(redirects in 0usize..=8) {
            let urls: Vec<String> = (0..redirects).map(|i| format!("https://b.example/{i}")).collect();
            let fetcher = chain_fetcher(urls);
            let (result, log) = run(&fetcher, &policy(&["a.example", "b.example"]), get("https://a.example/"));
            if redirects <= MAX_REDIRECTS {
                prop_assert_eq!(result.unwrap().body, "end");
                prop_assert_eq!(log.len(), redirects + 1);
            } else {
                prop_assert_eq!(result, Err(NetError::TooManyRedirects));
                prop_assert_eq!(log.len(), MAX_REDIRECTS + 1);
            }
        }

        /// Invariant: the connection goes to an address the resolver
        /// returned and that passed the check, never to anything else.
        #[test]
        fn connects_only_to_a_checked_address(a in any::<u8>(), b in any::<u8>(), c in any::<u8>(), d in any::<u8>()) {
            let ip = IpAddr::V4(Ipv4Addr::new(a, b, c, d));
            let fetcher = Fetcher::new(
                FakeResolver::public().with("api.example", &[ip]),
                FakeTransport::new(|_| Ok(ok(""))),
            );
            let (result, _) = run(&fetcher, &policy(&["api.example"]), get("https://api.example/"));
            let sent = fetcher.transport().sent.borrow();
            if is_public(ip) {
                prop_assert!(result.is_ok());
                prop_assert_eq!(sent[0].0, SocketAddr::new(ip, 443));
            } else {
                prop_assert!(sent.is_empty());
            }
        }

        /// Invariant: whatever the request times, no 60 s window holds more
        /// than 30 admitted requests.
        #[test]
        fn never_admits_more_than_thirty_in_a_minute(gaps in proptest::collection::vec(0i64..4_000, 1..200)) {
            let mut limit = RateLimit::new();
            let mut now = 0i64;
            let mut admitted: Vec<i64> = Vec::new();
            for gap in gaps {
                now += gap;
                if limit.admit(now) {
                    admitted.push(now);
                }
                let in_window = admitted.iter().filter(|t| now - **t < MINUTE_MS).count();
                prop_assert!(in_window <= MAX_REQUESTS_PER_MINUTE);
            }
        }

        /// Invariant: once 20 MiB has been received in a day, nothing more
        /// is admitted until the day rolls over.
        #[test]
        fn daily_bytes_are_capped(chunks in proptest::collection::vec(1u64..(8 * 1024 * 1024), 1..10)) {
            let mut limit = RateLimit::new();
            let mut total = 0u64;
            let mut now = 0i64;
            for chunk in chunks {
                now += 61_000;
                let admitted = limit.admit(now);
                prop_assert_eq!(admitted, total < MAX_BYTES_PER_DAY);
                if admitted {
                    limit.record_bytes(now, chunk);
                    total += chunk;
                }
            }
        }
    }
}
