//! `kanso://` callbacks on Windows and Linux (docs/ACCOUNTS.md › Sign in with
//! Apple, Google and GitHub).
//!
//! On macOS, `ASWebAuthenticationSession` hands Apple's callback straight to
//! the process that started it. Elsewhere the system browser opens
//! `kanso://auth/apple?flow=…&s=…`, and the operating system starts a new
//! `kanso` with that URL as its argument (the `x-scheme-handler/kanso` desktop
//! entry on Linux, `HKCU\Software\Classes\kanso` from the Windows installer).
//! That second process can't open the data folder the running app holds, so
//! it forwards the URL here over a loopback connection and exits.
//!
//! Another app could register the same scheme; the owner accepts that. What
//! limits it:
//!
//! - [`Router`] accepts a callback only for a flow this process started and
//!   is still waiting on, once, and only until the flow's deadline.
//! - The URL's `s` is never logged, printed or shown: the failures here say
//!   what happened without the URL.
//! - The endpoint file is readable only by its owner, and each side proves
//!   it read it: the app greets with one token before the forwarder sends
//!   anything, so a port another process took over after Kanso quit never
//!   receives a secret; the forwarder answers with the other.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::provider_browser::BrowserError;

/// The scheme the browser returns to.
pub const SCHEME: &str = "kanso";

/// The longest a callback wait lasts, whatever the server says.
pub const MAX_WAIT: Duration = Duration::from_secs(10 * 60);

/// How long either side of a forward waits for the other.
const IO_TIMEOUT: Duration = Duration::from_secs(3);

/// The longest message a forwarder may send: two lines, a token and a URL.
const MAX_MESSAGE: u64 = 4096;

/// The endpoint file in the data folder.
const ENDPOINT_FILE: &str = "sign-in-callback";

/// Whether a command-line argument is a `kanso://` URL the OS passed in,
/// rather than a command.
pub fn is_callback_argument(arg: &str) -> bool {
    arg.len() > SCHEME.len() + 3
        && arg[..SCHEME.len()].eq_ignore_ascii_case(SCHEME)
        && arg[SCHEME.len()..].starts_with("://")
}

/// What became of a callback URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// The flow that was waiting for it has it now.
    Delivered,
    /// No flow of this process is waiting for it (never started, already
    /// answered, cancelled or expired).
    NotWaiting,
    /// It isn't Apple's callback URL.
    NotACallback,
}

struct Waiter {
    flow: String,
    deadline: Instant,
    reply: async_channel::Sender<Result<String, BrowserError>>,
}

/// The flows waiting for their callback.
pub struct Router {
    waiters: Mutex<Vec<Waiter>>,
}

/// This process's router. macOS gets Apple's callback from its own sheet.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub static ROUTER: Router = Router::new();

impl Router {
    pub const fn new() -> Router {
        Router {
            waiters: Mutex::new(Vec::new()),
        }
    }

    /// Waits for `flow`'s callback until `deadline` (at most [`MAX_WAIT`]
    /// from now). The receiver gets the URL once, or closes when the wait
    /// is dropped. A second wait for the same flow replaces the first.
    pub fn wait_for(
        &self,
        flow: &str,
        deadline: Instant,
    ) -> async_channel::Receiver<Result<String, BrowserError>> {
        let (reply, receiver) = async_channel::bounded(1);
        let deadline = deadline.min(Instant::now() + MAX_WAIT);
        let mut waiters = self.lock();
        waiters.retain(|waiter| waiter.flow != flow);
        waiters.push(Waiter {
            flow: flow.to_owned(),
            deadline,
            reply,
        });
        receiver
    }

    /// Hands `url` to the flow waiting for it, if there is one.
    pub fn deliver(&self, url: &str) -> Delivery {
        self.deliver_at(url, Instant::now())
    }

    fn deliver_at(&self, url: &str, now: Instant) -> Delivery {
        let Some(callback) = crate::provider_browser::apple_flow(url) else {
            return Delivery::NotACallback;
        };
        let mut waiters = self.lock();
        // Expired waits and those whose flow stopped listening go first.
        waiters.retain(|waiter| waiter.deadline > now && !waiter.reply.is_closed());
        let Some(index) = waiters
            .iter()
            .position(|waiter| waiter.flow == callback.flow)
        else {
            return Delivery::NotWaiting;
        };
        let waiter = waiters.swap_remove(index);
        match waiter.reply.try_send(Ok(url.to_owned())) {
            Ok(()) => Delivery::Delivered,
            Err(_) => Delivery::NotWaiting,
        }
    }

    /// How many flows are waiting (for tests).
    #[cfg(test)]
    fn waiting(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Waiter>> {
        self.waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// ---------------------------------------------------------------- the link

/// Where the running app listens, and the two tokens.
#[derive(Clone, PartialEq, Eq)]
struct Endpoint {
    port: u16,
    /// What the app greets a forwarder with.
    server_token: String,
    /// What a forwarder answers with.
    client_token: String,
}

impl Endpoint {
    fn new(port: u16) -> Endpoint {
        Endpoint {
            port,
            server_token: token(),
            client_token: token(),
        }
    }

    fn to_text(&self) -> String {
        format!(
            "{}\n{}\n{}\n",
            self.port, self.server_token, self.client_token
        )
    }

    fn parse(text: &str) -> Option<Endpoint> {
        let mut lines = text.lines();
        let port = lines.next()?.parse().ok()?;
        let server_token = lines.next()?.to_owned();
        let client_token = lines.next()?.to_owned();
        (is_token(&server_token) && is_token(&client_token)).then_some(Endpoint {
            port,
            server_token,
            client_token,
        })
    }
}

fn token() -> String {
    (0..32)
        .map(|_| format!("{:02x}", fastrand::u8(..)))
        .collect()
}

fn is_token(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Compares two tokens in time that doesn't depend on where they differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |diff, (x, y)| diff | (x ^ y))
            == 0
}

fn endpoint_path(data_dir: &Path) -> PathBuf {
    data_dir.join(ENDPOINT_FILE)
}

/// Writes the endpoint file readable only by this user.
fn write_endpoint(path: &Path, endpoint: &Endpoint) -> std::io::Result<()> {
    let temporary = path.with_extension("tmp");
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(endpoint.to_text().as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, path)
}

/// Starts listening for forwarded callbacks for `router`, and writes the
/// endpoint into `data_dir`. Runs for the rest of the process.
pub fn listen(data_dir: &Path, router: &'static Router) -> std::io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let endpoint = Endpoint::new(listener.local_addr()?.port());
    write_endpoint(&endpoint_path(data_dir), &endpoint)?;
    std::thread::Builder::new()
        .name("kanso-callbacks".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                // One forward at a time: each is a few bytes and short-lived.
                if let Err(error) = serve(stream, &endpoint, router) {
                    // The error never carries the URL.
                    eprintln!("kanso: a sign-in callback couldn't be read: {error}");
                }
            }
        })?;
    Ok(())
}

fn serve(stream: TcpStream, endpoint: &Endpoint, router: &Router) -> std::io::Result<()> {
    if !stream.peer_addr()?.ip().is_loopback() {
        return Ok(());
    }
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    let mut writer = stream.try_clone()?;
    writer.write_all(format!("{}\n", endpoint.server_token).as_bytes())?;
    let mut reader = BufReader::new(stream.take(MAX_MESSAGE));
    let mut client_token = String::new();
    reader.read_line(&mut client_token)?;
    if !same(client_token.trim_end(), &endpoint.client_token) {
        return Ok(());
    }
    let mut url = String::new();
    reader.read_line(&mut url)?;
    let delivery = router.deliver(url.trim_end());
    let answer = if delivery == Delivery::Delivered {
        "ok\n"
    } else {
        "no\n"
    };
    writer.write_all(answer.as_bytes())
}

/// Why a URL couldn't be forwarded to the running app.
#[derive(Debug, PartialEq, Eq)]
pub enum ForwardError {
    /// No running Kanso has written an endpoint, or it isn't listening.
    NotRunning,
    /// Something answered on the port that isn't the Kanso that wrote the
    /// file; nothing was sent to it.
    WrongPeer,
    /// The running Kanso has no flow waiting for this callback.
    NotWaiting,
    Io(String),
}

impl std::fmt::Display for ForwardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForwardError::NotRunning => f.write_str("Kanso isn't running"),
            ForwardError::WrongPeer => f.write_str("another program answered instead of Kanso"),
            ForwardError::NotWaiting => f.write_str("Kanso isn't waiting for this sign-in"),
            ForwardError::Io(error) => f.write_str(error),
        }
    }
}

/// Sends `url` to the Kanso running with `data_dir`.
pub fn forward(data_dir: &Path, url: &str) -> Result<(), ForwardError> {
    if url.contains(['\n', '\r']) {
        return Err(ForwardError::NotWaiting);
    }
    let text =
        std::fs::read_to_string(endpoint_path(data_dir)).map_err(|_| ForwardError::NotRunning)?;
    let endpoint = Endpoint::parse(&text).ok_or(ForwardError::NotRunning)?;
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)
        .map_err(|_| ForwardError::NotRunning)?;
    let io = |error: std::io::Error| ForwardError::Io(error.to_string());
    stream.set_read_timeout(Some(IO_TIMEOUT)).map_err(io)?;
    stream.set_write_timeout(Some(IO_TIMEOUT)).map_err(io)?;
    let mut writer = stream.try_clone().map_err(io)?;
    let mut reader = BufReader::new(stream.take(MAX_MESSAGE));
    let mut greeting = String::new();
    reader
        .read_line(&mut greeting)
        .map_err(|_| ForwardError::WrongPeer)?;
    if !same(greeting.trim_end(), &endpoint.server_token) {
        return Err(ForwardError::WrongPeer);
    }
    writer
        .write_all(format!("{}\n{url}\n", endpoint.client_token).as_bytes())
        .map_err(io)?;
    let mut answer = String::new();
    reader.read_line(&mut answer).map_err(io)?;
    match answer.trim_end() {
        "ok" => Ok(()),
        _ => Err(ForwardError::NotWaiting),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn b64_32(byte: u8) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([byte; 32])
    }

    fn callback(flow: &str) -> String {
        format!("kanso://auth/apple?flow={flow}&s={}", b64_32(9))
    }

    fn leaked() -> &'static Router {
        Box::leak(Box::new(Router::new()))
    }

    #[test]
    fn callback_arguments_are_told_from_commands() {
        assert!(is_callback_argument(&callback(&b64_32(1))));
        assert!(is_callback_argument("KANSO://auth/apple"));
        for not in ["serve", "kanso", "kanso:", "kanso://", "https://kanso", "-psn_0_1"] {
            assert!(!is_callback_argument(not), "{not}");
        }
    }

    /// The OS starts `kanso <url>`, or `kanso --data-dir <dir> <url>`: the
    /// URL arrives as the command, which `main` forwards instead of running.
    #[test]
    fn the_launch_arguments_carry_the_url_as_the_command() {
        let url = callback(&b64_32(1));
        let parsed = crate::parse_args_from([url.clone()]).unwrap();
        assert!(parsed.command.as_deref().is_some_and(is_callback_argument));
        let parsed =
            crate::parse_args_from(["--data-dir".into(), "/tmp/k".into(), url.clone()]).unwrap();
        assert_eq!(parsed.command.as_deref(), Some(url.as_str()));
        assert_eq!(parsed.data_dir, PathBuf::from("/tmp/k"));
    }

    /// The desktop entry and the installer register the scheme the router
    /// expects, passing the URL as one argument.
    #[test]
    fn the_scheme_is_registered_where_the_os_looks() {
        let desktop = include_str!("../../../assets/linux/kanso.desktop");
        assert!(desktop.contains("MimeType=x-scheme-handler/kanso;"));
        assert!(desktop.lines().any(|line| line == "Exec=kanso %u"));
        let installer = include_str!("../../../packaging/windows/kanso.iss");
        assert!(installer.contains(r#"Subkey: "Software\Classes\kanso""#));
        assert!(installer.contains(r#"ValueName: "URL Protocol""#));
        assert!(installer.contains(r#"""{app}\kanso.exe"" ""%1"""#));
    }

    #[test]
    fn a_callback_reaches_only_the_flow_waiting_for_it_once() {
        let router = Router::new();
        let flow = b64_32(1);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        // Another flow's callback, and something that isn't a callback.
        assert_eq!(router.deliver(&callback(&b64_32(2))), Delivery::NotWaiting);
        assert_eq!(
            router.deliver("kanso://auth/apple?flow=short&s=x"),
            Delivery::NotACallback
        );
        assert_eq!(
            router.deliver(&format!("https://evil.example/auth/apple?flow={flow}&s={}", b64_32(9))),
            Delivery::NotACallback
        );
        assert_eq!(router.deliver(&callback(&flow)), Delivery::Delivered);
        assert_eq!(waiting.try_recv().unwrap().unwrap(), callback(&flow));
        // Answered: a replay finds nobody waiting.
        assert_eq!(router.deliver(&callback(&flow)), Delivery::NotWaiting);
        assert_eq!(router.waiting(), 0);
    }

    #[test]
    fn an_expired_or_abandoned_wait_takes_nothing() {
        let router = Router::new();
        let flow = b64_32(3);
        let start = Instant::now();
        let _waiting = router.wait_for(&flow, start + Duration::from_secs(5));
        assert_eq!(
            router.deliver_at(&callback(&flow), start + Duration::from_secs(6)),
            Delivery::NotWaiting
        );
        assert_eq!(router.waiting(), 0);

        let dropped = router.wait_for(&flow, start + Duration::from_secs(60));
        drop(dropped);
        assert_eq!(router.deliver(&callback(&flow)), Delivery::NotWaiting);
    }

    #[test]
    fn no_wait_outlasts_the_cap() {
        let router = Router::new();
        let flow = b64_32(4);
        let _waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(86_400));
        assert_eq!(
            router.deliver_at(&callback(&flow), Instant::now() + MAX_WAIT + Duration::from_secs(1)),
            Delivery::NotWaiting
        );
    }

    #[test]
    fn a_forwarded_callback_reaches_the_running_app() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(5);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        listen(dir.path(), router).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(endpoint_path(dir.path()))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(6))),
            Err(ForwardError::NotWaiting)
        );
        assert_eq!(forward(dir.path(), &callback(&flow)), Ok(()));
        assert_eq!(waiting.try_recv().unwrap().unwrap(), callback(&flow));
    }

    #[test]
    fn nothing_is_sent_without_a_running_kanso_or_to_another_listener() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(7))),
            Err(ForwardError::NotRunning)
        );
        // A stale file whose port another program now holds: it doesn't
        // know the greeting, so the URL is never written to it.
        let impostor = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let endpoint = Endpoint::new(impostor.local_addr().unwrap().port());
        write_endpoint(&endpoint_path(dir.path()), &endpoint).unwrap();
        let received = std::thread::spawn(move || {
            let (mut stream, _) = impostor.accept().unwrap();
            stream.write_all(b"not-the-token\n").unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(500)))
                .unwrap();
            let mut seen = Vec::new();
            stream.read_to_end(&mut seen).ok();
            seen
        });
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(7))),
            Err(ForwardError::WrongPeer)
        );
        assert!(received.join().unwrap().is_empty());
    }

    #[test]
    fn a_forwarder_without_the_token_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(8);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        listen(dir.path(), router).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap();
        stream
            .write_all(format!("{}\n{}\n", "0".repeat(64), callback(&flow)).as_bytes())
            .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).ok();
        assert!(!answer.contains("ok"));
        assert!(waiting.try_recv().is_err());
    }

    proptest! {
        #[test]
        fn prop_the_endpoint_file_round_trips(port in 1u16..) {
            let endpoint = Endpoint::new(port);
            prop_assert!(Endpoint::parse(&endpoint.to_text()) == Some(endpoint));
        }

        #[test]
        fn prop_tokens_compare_equal_only_when_equal(a in "[0-9a-f]{64}", b in "[0-9a-f]{64}") {
            prop_assert_eq!(same(&a, &b), a == b);
            prop_assert!(same(&a, &a));
        }
    }
}
