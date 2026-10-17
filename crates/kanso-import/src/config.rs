//! Inkdrop's local HTTP server settings from its `config.json`.
//!
//! The credentials are the user's real password. Nothing here prints, logs
//! or stores them; `Debug` redacts them.

use std::fmt;
use std::path::Path;

use anyhow::{Context as _, bail};
use base64::Engine as _;
use serde_json::Value;

/// Inkdrop's default local server port.
pub const DEFAULT_PORT: u16 = 19840;
/// Inkdrop's default bind address.
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// `core.server` from Inkdrop's `config.json`.
#[derive(Clone, PartialEq, Eq)]
pub struct InkdropServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub credentials: InkdropCredentials,
}

/// The `auth` value of Inkdrop's server settings.
#[derive(Clone, PartialEq, Eq)]
pub enum InkdropCredentials {
    /// No `auth` value.
    None,
    /// `"username:password"` (split at the first colon) or `{username, password}`.
    Basic { username: String, password: String },
    /// Electron `safeStorage` ciphertext: base64 of `v10`/`v11` followed by
    /// AES-CBC data. Only the macOS Keychain item "inkdrop Safe Storage" can
    /// decrypt it, so the username and password are not available here.
    Encrypted { ciphertext: String },
    /// A value in a shape this importer doesn't recognize.
    Unrecognized,
}

impl InkdropServerConfig {
    pub fn username(&self) -> Option<&str> {
        match &self.credentials {
            InkdropCredentials::Basic { username, .. } => Some(username),
            _ => None,
        }
    }

    pub fn password(&self) -> Option<&str> {
        match &self.credentials {
            InkdropCredentials::Basic { password, .. } => Some(password),
            _ => None,
        }
    }
}

impl fmt::Debug for InkdropServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InkdropServerConfig")
            .field("enabled", &self.enabled)
            .field("port", &self.port)
            .field("bind_address", &self.bind_address)
            .field("credentials", &self.credentials)
            .finish()
    }
}

impl fmt::Debug for InkdropCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InkdropCredentials::None => f.write_str("None"),
            InkdropCredentials::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            InkdropCredentials::Encrypted { .. } => f
                .debug_struct("Encrypted")
                .field("ciphertext", &"<redacted>")
                .finish(),
            InkdropCredentials::Unrecognized => f.write_str("Unrecognized"),
        }
    }
}

pub(crate) fn parse_credentials(auth: Option<&Value>) -> InkdropCredentials {
    match auth {
        None | Some(Value::Null) => InkdropCredentials::None,
        Some(Value::String(s)) if s.is_empty() => InkdropCredentials::None,
        Some(Value::String(s)) => match s.split_once(':') {
            Some((username, password)) => InkdropCredentials::Basic {
                username: username.to_owned(),
                password: password.to_owned(),
            },
            None => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(s.trim());
                match decoded {
                    Ok(bytes) if bytes.starts_with(b"v10") || bytes.starts_with(b"v11") => {
                        InkdropCredentials::Encrypted {
                            ciphertext: s.clone(),
                        }
                    }
                    _ => InkdropCredentials::Unrecognized,
                }
            }
        },
        Some(Value::Object(map)) => match (
            map.get("username").and_then(Value::as_str),
            map.get("password").and_then(Value::as_str),
        ) {
            (Some(username), Some(password)) => InkdropCredentials::Basic {
                username: username.to_owned(),
                password: password.to_owned(),
            },
            _ => InkdropCredentials::Unrecognized,
        },
        Some(_) => InkdropCredentials::Unrecognized,
    }
}

/// Parses the server settings from the text of Inkdrop's `config.json`.
/// `Ok(None)` when it has no `core.server` section.
pub fn parse_inkdrop_server_config(json: &str) -> anyhow::Result<Option<InkdropServerConfig>> {
    let config: Value = serde_json::from_str(json).context("config.json is not valid JSON")?;
    let Some(server) = config.pointer("/*/core/server") else {
        return Ok(None);
    };
    let Some(server) = server.as_object() else {
        bail!("core.server in config.json is not an object");
    };
    let enabled = match server.get("enabled") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => bail!("core.server.enabled in config.json is not a boolean"),
    };
    let port = match server.get("port") {
        None | Some(Value::Null) => DEFAULT_PORT,
        Some(v) => v
            .as_u64()
            .and_then(|p| u16::try_from(p).ok())
            .filter(|p| *p != 0)
            .context("core.server.port in config.json is not a valid port")?,
    };
    let bind_address = match server.get("bindAddress") {
        None | Some(Value::Null) => DEFAULT_BIND_ADDRESS.to_owned(),
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_owned(),
        Some(_) => bail!("core.server.bindAddress in config.json is not a valid address"),
    };
    Ok(Some(InkdropServerConfig {
        enabled,
        port,
        bind_address,
        credentials: parse_credentials(server.get("auth")),
    }))
}

/// Reads Inkdrop's local server settings from `<inkdrop_dir>/config.json`.
/// `Ok(None)` when the file doesn't exist or has no server section.
pub fn read_inkdrop_server_config(
    inkdrop_dir: &Path,
) -> anyhow::Result<Option<InkdropServerConfig>> {
    let path = inkdrop_dir.join("config.json");
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("cannot read {}", path.display())),
    };
    parse_inkdrop_server_config(&json).with_context(|| format!("in {}", path.display()))
}
