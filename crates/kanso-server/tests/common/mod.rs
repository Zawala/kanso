//! Helpers shared by the integration tests.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use kanso_server::{ServerConfig, basic_header, router};
use kanso_store::Store;
use serde_json::Value;
use tower::ServiceExt;

pub const USER: &str = "kanso-test";
pub const PASS: &str = "pa:ss word";

pub fn config() -> ServerConfig {
    ServerConfig {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
        username: USER.into(),
        password: PASS.into(),
    }
}

pub struct App {
    pub dir: tempfile::TempDir,
    pub store: Arc<Store>,
    pub router: Router,
}

pub fn app() -> App {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let router = router(store.clone(), &config());
    App { dir, store, router }
}

/// A runtime for property tests, which are synchronous.
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    })
}

pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: Vec<u8>,
}

impl Response {
    pub fn json(&self) -> Value {
        if self.bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&self.bytes)
                .unwrap_or_else(|err| panic!("{err}: {}", String::from_utf8_lossy(&self.bytes)))
        }
    }
}

pub async fn send(router: &Router, request: Request<Body>) -> Response {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Response {
        status,
        headers,
        bytes,
    }
}

pub fn request(method: Method, uri: &str, body: Option<&Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, basic_header(USER, PASS));
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// An authenticated request; returns the status and the JSON body.
pub async fn call(
    router: &Router,
    method: Method,
    uri: &str,
    body: Option<&Value>,
) -> (StatusCode, Value) {
    let response = send(router, request(method, uri, body)).await;
    (response.status, response.json())
}

pub async fn get(router: &Router, uri: &str) -> (StatusCode, Value) {
    call(router, Method::GET, uri, None).await
}

pub async fn post(router: &Router, uri: &str, body: Value) -> (StatusCode, Value) {
    call(router, Method::POST, uri, Some(&body)).await
}

/// POSTs and expects `{ok: true, id, rev}`; returns `(id, rev)`.
pub async fn write(router: &Router, uri: &str, body: Value) -> (String, String) {
    let (status, res) = post(router, uri, body.clone()).await;
    assert_eq!(status, StatusCode::OK, "POST {uri} {body}: {res}");
    assert_eq!(res["ok"], true, "{res}");
    (
        res["id"].as_str().unwrap().to_owned(),
        res["rev"].as_str().unwrap().to_owned(),
    )
}

/// A valid 1x1 PNG.
pub const PNG: [u8; 70] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xd0, 0xaa, 0xbf, 0xf2,
    0x1f, 0x00, 0x04, 0xd1, 0x02, 0x7d, 0xe2, 0x3a, 0xb3, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Percent-encodes a query-string value (everything but RFC 3986 unreserved).
pub fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
