//! The Cloudflare Worker over HTTPS (blocking; the app has no async runtime).

use std::fmt;
use std::time::Duration;

use kanso_store::SyncDoc;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use ureq::http::Response;
use ureq::{Agent, Body};

use crate::remote::{
    ChangesPage, MAX_FILE_BYTES, PushResult, Remote, RemoteError, RemoteFile, encode_segment,
    md5_hex,
};

/// Largest JSON response read (a page of large notes can be big).
const MAX_JSON_RESPONSE: u64 = 512 * 1024 * 1024;

/// The sync Worker at `endpoint`, authenticated with a bearer token.
pub struct HttpRemote {
    endpoint: String,
    token: String,
    agent: Agent,
}

impl fmt::Debug for HttpRemote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRemote")
            .field("endpoint", &self.endpoint)
            .field("token", &"<redacted>")
            .finish()
    }
}

#[derive(serde::Deserialize)]
struct PushResponse {
    results: Vec<PushResult>,
}

impl HttpRemote {
    /// `endpoint` is the Worker's base URL, e.g. `https://kanso-sync.example.workers.dev`.
    pub fn new(endpoint: impl Into<String>, token: impl Into<String>) -> Self {
        let endpoint = endpoint.into().trim().trim_end_matches('/').to_owned();
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_global(Some(Duration::from_secs(300)))
            .user_agent(concat!("kanso-sync/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        HttpRemote {
            endpoint,
            token: token.into(),
            agent,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.endpoint)
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// `GET /v1/health`: whether the Worker answers (no token needed).
    pub fn health(&self) -> Result<(), RemoteError> {
        let mut response = self
            .agent
            .get(self.url("/v1/health"))
            .call()
            .map_err(transport)?;
        let value: Value = read_json(&mut response)?;
        if value.get("ok") == Some(&Value::Bool(true)) {
            Ok(())
        } else {
            Err(RemoteError::Protocol(
                "health check did not answer ok".into(),
            ))
        }
    }
}

fn transport(err: ureq::Error) -> RemoteError {
    RemoteError::Transport(err.to_string())
}

/// Turns a non-2xx response into an error, using the `{error}` message when present.
fn check(response: &mut Response<Body>) -> Result<(), RemoteError> {
    let status = response.status().as_u16();
    if (200..300).contains(&status) {
        return Ok(());
    }
    if status == 401 {
        return Err(RemoteError::Unauthorized);
    }
    let text = response
        .body_mut()
        .with_config()
        .limit(64 * 1024)
        .read_to_string()
        .unwrap_or_default();
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or(text);
    Err(RemoteError::Status { status, message })
}

fn read_json<T: DeserializeOwned>(response: &mut Response<Body>) -> Result<T, RemoteError> {
    check(response)?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_JSON_RESPONSE)
        .read_to_vec()
        .map_err(transport)?;
    serde_json::from_slice(&bytes).map_err(|e| RemoteError::Protocol(e.to_string()))
}

impl Remote for HttpRemote {
    fn push(&self, docs: &[SyncDoc]) -> Result<Vec<PushResult>, RemoteError> {
        let body = serde_json::to_vec(&json!({ "docs": docs }))
            .map_err(|e| RemoteError::Protocol(e.to_string()))?;
        let mut response = self
            .agent
            .post(self.url("/v1/push"))
            .header("authorization", self.auth())
            .header("content-type", "application/json")
            .send(&body[..])
            .map_err(transport)?;
        let parsed: PushResponse = read_json(&mut response)?;
        Ok(parsed.results)
    }

    fn changes(&self, since: i64, limit: usize) -> Result<ChangesPage, RemoteError> {
        let mut response = self
            .agent
            .get(self.url(&format!("/v1/changes?since={since}&limit={limit}")))
            .header("authorization", self.auth())
            .call()
            .map_err(transport)?;
        read_json(&mut response)
    }

    fn put_file(
        &self,
        id: &str,
        md5_hex: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<(), RemoteError> {
        let content_type = if content_type.is_empty() {
            "application/octet-stream"
        } else {
            content_type
        };
        let mut response = self
            .agent
            .put(self.url(&format!("/v1/files/{}", encode_segment(id))))
            .header("authorization", self.auth())
            .header("content-type", content_type)
            .header("x-kanso-md5", md5_hex)
            .send(bytes)
            .map_err(transport)?;
        check(&mut response)
    }

    fn get_file(&self, id: &str) -> Result<Option<RemoteFile>, RemoteError> {
        let mut response = self
            .agent
            .get(self.url(&format!("/v1/files/{}", encode_segment(id))))
            .header("authorization", self.auth())
            .call()
            .map_err(transport)?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        check(&mut response)?;
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let md5 = header("x-kanso-md5")
            .ok_or_else(|| RemoteError::Protocol("the file has no x-kanso-md5 header".into()))?
            .to_ascii_lowercase();
        let content_type =
            header("content-type").unwrap_or_else(|| "application/octet-stream".into());
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_FILE_BYTES as u64 + 1)
            .read_to_vec()
            .map_err(transport)?;
        if md5_hex(&bytes) != md5 {
            return Err(RemoteError::Protocol(format!(
                "the bytes of {id} do not match their MD5"
            )));
        }
        Ok(Some(RemoteFile {
            md5_hex: md5,
            content_type,
            bytes,
        }))
    }
}
