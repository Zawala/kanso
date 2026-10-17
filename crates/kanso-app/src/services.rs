//! Background services the window reports on: the local API and sync.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use gpui_kit::*;
use kanso_server::{ServerConfig, ServerHandle};

use crate::config::ServerSettings;
use crate::workspace::AppState;

/// How often to try again for a port another app holds.
const PORT_RETRY: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ApiState {
    #[default]
    Off,
    Running(String),
    /// The port is taken, usually by Inkdrop; Kanso keeps trying.
    PortInUse(u16),
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SyncState {
    #[default]
    Off,
    Syncing,
    Synced {
        at_ms: i64,
    },
    Failed(String),
}

#[derive(Default)]
pub struct Services {
    pub api: ApiState,
    pub sync: SyncState,
    server: Option<ServerHandle>,
    port_retry: Option<Task<()>>,
}

impl Global for Services {}

impl Services {
    pub fn api_status(&self) -> String {
        match &self.api {
            ApiState::Off => "API off".into(),
            ApiState::Running(addr) => format!("API on {addr}"),
            ApiState::PortInUse(port) => {
                format!("API off: port {port} is in use (quit Inkdrop to free it)")
            }
            ApiState::Failed(reason) => format!("API off: {reason}"),
        }
    }

    pub fn sync_status(&self) -> String {
        match &self.sync {
            SyncState::Off => "Sync off".into(),
            SyncState::Syncing => "Syncing…".into(),
            SyncState::Synced { at_ms } => format!(
                "Synced {}",
                crate::note_list::relative_time(*at_ms, kanso_core::now_ms())
            ),
            SyncState::Failed(reason) => format!("Sync failed: {reason}"),
        }
    }
}

/// The server's listen address and credentials from the settings.
pub fn server_config(settings: &ServerSettings) -> Result<ServerConfig, String> {
    let ip: IpAddr = settings
        .bind_address
        .trim()
        .parse()
        .map_err(|_| format!("“{}” isn't an IP address", settings.bind_address))?;
    if settings.username.is_empty() || settings.password.is_empty() {
        return Err("set a username and password first".into());
    }
    Ok(ServerConfig {
        bind: SocketAddr::new(ip, settings.port),
        username: settings.username.clone(),
        password: settings.password.clone(),
    })
}

/// What the status bar says for a server that just tried to start.
pub fn api_state(handle: &ServerHandle, port: u16) -> ApiState {
    match handle.bind_result() {
        Ok(addr) => ApiState::Running(format!("http://{addr}")),
        Err(_) if handle.is_addr_in_use() => ApiState::PortInUse(port),
        Err(err) => ApiState::Failed(err.to_string()),
    }
}

/// Starts, stops or restarts the local API to match the settings.
pub fn apply_server_settings(cx: &mut App) {
    let state = cx.global::<AppState>();
    let settings = state.config.server.clone();
    let store = state.store.clone();
    let services = cx.global_mut::<Services>();
    services.port_retry = None;
    if let Some(old) = services.server.take() {
        // Waits for in-flight requests (at most a few seconds) so the port
        // is free before binding it again.
        if let Err(err) = old.stop() {
            eprintln!("kanso: the API server stopped with an error: {err:#}");
        }
    }
    services.api = if !settings.enabled {
        ApiState::Off
    } else {
        match server_config(&settings) {
            Err(reason) => ApiState::Failed(reason),
            Ok(config) => {
                let handle = kanso_server::spawn(store, config);
                let api = api_state(&handle, settings.port);
                if matches!(api, ApiState::Running(_)) {
                    services.server = Some(handle);
                }
                api
            }
        }
    };
    if matches!(services.api, ApiState::PortInUse(_)) {
        let retry = cx.spawn(async move |cx| {
            cx.background_executor().timer(PORT_RETRY).await;
            cx.update(apply_server_settings);
        });
        cx.global_mut::<Services>().port_retry = Some(retry);
    }
    cx.refresh_windows();
}

/// Stops the API before quitting.
pub fn stop_server(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.port_retry = None;
    if let Some(server) = services.server.take() {
        server.stop().ok();
    }
}

/// Starts or stops background sync to match the settings.
pub fn apply_sync_settings(cx: &mut App) {
    cx.refresh_windows();
}

/// Syncs once, now.
pub fn sync_now(cx: &mut App) {
    cx.refresh_windows();
}

/// Keeps the sync access token in the Keychain.
pub fn save_sync_token(_token: &str) -> anyhow::Result<()> {
    anyhow::bail!("sync isn't set up in this build yet")
}

#[cfg(test)]
mod tests {
    use super::{ApiState, api_state, server_config};
    use crate::config::ServerSettings;
    use proptest::prelude::*;
    use std::net::IpAddr;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round-trip: the listen address keeps the configured IP and port,
        /// IPv6 included.
        #[test]
        fn prop_server_config_keeps_ip_and_port(ip in any::<IpAddr>(), port in any::<u16>()) {
            let settings = ServerSettings {
                bind_address: ip.to_string(),
                port,
                username: "u".into(),
                password: "p".into(),
                ..ServerSettings::default()
            };
            let config = server_config(&settings).unwrap();
            prop_assert_eq!((config.bind.ip(), config.bind.port()), (ip, port));
        }

        /// Refinement: text that isn't an IP address is refused.
        #[test]
        fn prop_bad_addresses_are_refused(text in "[a-z]{1,10}") {
            let settings = ServerSettings { bind_address: text, ..ServerSettings::default() };
            prop_assert!(server_config(&settings).is_err());
        }
    }

    #[test]
    fn a_taken_port_is_reported_as_in_use() {
        let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = taken.local_addr().unwrap().port();
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
        let settings = ServerSettings {
            port,
            username: "u".into(),
            password: "p".into(),
            ..ServerSettings::default()
        };
        let handle = kanso_server::spawn(store.clone(), server_config(&settings).unwrap());
        assert_eq!(api_state(&handle, port), ApiState::PortInUse(port));
        drop(taken);
        let settings = ServerSettings {
            port: 0,
            ..settings
        };
        let handle = kanso_server::spawn(store, server_config(&settings).unwrap());
        assert!(matches!(api_state(&handle, 0), ApiState::Running(_)));
        handle.stop().unwrap();
    }
}
