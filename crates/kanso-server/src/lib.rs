//! Kanso's local HTTP API.
//!
//! The REST endpoints speak a widely used local-server REST API, wire for
//! wire, so the MCP server `@inkdropapp/mcp-server` and scripts written for
//! that API work against Kanso unchanged. `/mcp` serves the same 17 MCP tools natively
//! on the same port. Every route requires HTTP Basic authentication.
//!
//! The server stays thin: validation, conflicts and search live in
//! [`kanso_store::Store`], which is synchronous and is called on tokio's
//! blocking pool.

mod api;
mod auth;
mod error;
mod mcp;
mod rest;

use std::fmt;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::post;
use kanso_store::Store;

pub use auth::basic_header;

/// Largest request body accepted. A 10 MB image is about 13.4 MB in base64.
pub const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// Where the server listens and the credentials it requires.
#[derive(Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    pub username: String,
    pub password: String,
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("bind", &self.bind)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub store: Arc<Store>,
    /// Whether MCP tools may read and write local files (`create-file` with
    /// `filePath`, `read-file` with `outputPath`). Only when the server is
    /// bound to a loopback address, so every client is on this machine.
    pub local_files: bool,
}

/// The full API: REST endpoints, `/mcp`, Basic auth and the body limit.
pub fn router(store: Arc<Store>, config: &ServerConfig) -> Router {
    let state = AppState {
        store,
        local_files: config.bind.ip().is_loopback(),
    };
    let credentials = Arc::new(auth::Credentials::new(&config.username, &config.password));
    rest::routes()
        .route(
            "/mcp",
            post(mcp::handle).get(mcp::no_stream).delete(mcp::no_stream),
        )
        .fallback(rest::not_found)
        .method_not_allowed_fallback(rest::method_not_allowed)
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(
            credentials,
            auth::require_auth,
        ))
}

/// How long in-flight requests may run on after shutdown is requested.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Binds `config.bind` and serves until `shutdown` resolves, then lets
/// in-flight requests finish for up to five seconds.
///
/// When the port is taken the error is an [`io::Error`] of kind
/// [`io::ErrorKind::AddrInUse`]; see [`is_addr_in_use`].
pub async fn serve(
    store: Arc<Store>,
    config: ServerConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    serve_on(listener, store, &config, shutdown).await
}

async fn serve_on(
    listener: tokio::net::TcpListener,
    store: Arc<Store>,
    config: &ServerConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let app = router(store, config);
    let (stopping_tx, stopping) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        shutdown.await;
        let _ = stopping_tx.send(true);
    });
    let stopped = |mut rx: tokio::sync::watch::Receiver<bool>| async move {
        let _ = rx.wait_for(|stopping| *stopping).await;
    };
    let graceful = axum::serve(listener, app).with_graceful_shutdown(stopped(stopping.clone()));
    let deadline = async {
        stopped(stopping).await;
        tokio::time::sleep(SHUTDOWN_GRACE).await;
    };
    tokio::select! {
        result = graceful => result?,
        // A client that never finishes its request can't hold the server open.
        () = deadline => {}
    }
    Ok(())
}

/// Whether a [`serve`] error means the port is already in use.
pub fn is_addr_in_use(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::AddrInUse)
    })
}

/// A server running on its own thread and tokio runtime, for callers that
/// are not async (the GPUI app). Dropping the handle asks the server to stop.
pub struct ServerHandle {
    bound: Result<SocketAddr, io::Error>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
}

/// Binds `config.bind` right away, then serves on a dedicated thread with
/// its own tokio runtime. The bind result is available immediately from the
/// handle, so the caller can show "API off: port in use".
pub fn spawn(store: Arc<Store>, config: ServerConfig) -> ServerHandle {
    let failed = |err| ServerHandle {
        bound: Err(err),
        stop: None,
        thread: None,
    };
    let listener = match std::net::TcpListener::bind(config.bind) {
        Ok(listener) => listener,
        Err(err) => return failed(err),
    };
    let addr = match listener
        .set_nonblocking(true)
        .and_then(|()| listener.local_addr())
    {
        Ok(addr) => addr,
        Err(err) => return failed(err),
    };
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let thread = std::thread::Builder::new()
        .name("kanso-server".into())
        .spawn(move || -> anyhow::Result<()> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("kanso-server-worker")
                .enable_all()
                .build()?;
            let result = runtime.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener)?;
                serve_on(listener, store, &config, async move {
                    let _ = stop_rx.await;
                })
                .await
            });
            runtime.shutdown_timeout(Duration::from_secs(5));
            result
        });
    match thread {
        Ok(thread) => ServerHandle {
            bound: Ok(addr),
            stop: Some(stop_tx),
            thread: Some(thread),
        },
        Err(err) => failed(err),
    }
}

impl ServerHandle {
    /// The bound address, or why binding failed.
    pub fn bind_result(&self) -> Result<SocketAddr, &io::Error> {
        self.bound.as_ref().copied()
    }

    /// The bound address, when binding succeeded.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.bound.as_ref().ok().copied()
    }

    /// Whether binding failed because the port is taken.
    pub fn is_addr_in_use(&self) -> bool {
        self.bound
            .as_ref()
            .is_err_and(|err| err.kind() == io::ErrorKind::AddrInUse)
    }

    /// Whether the server thread is still serving.
    pub fn is_running(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }

    /// Stops the server, waits for in-flight requests and returns how the
    /// server thread ended.
    pub fn stop(mut self) -> anyhow::Result<()> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        match self.thread.take() {
            Some(thread) => thread
                .join()
                .map_err(|_| anyhow::anyhow!("the server thread panicked"))?,
            None => Ok(()),
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

impl fmt::Debug for ServerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerHandle")
            .field("bound", &self.bound)
            .field("running", &self.is_running())
            .finish()
    }
}
