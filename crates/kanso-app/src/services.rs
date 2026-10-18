//! Background services the window reports on: the local API and sync.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use gpui_kit::*;
use kanso_server::{ServerConfig, ServerHandle};
use kanso_store::{Change, Origin};
use kanso_sync::{SyncConfig, SyncReport};

use crate::config::ServerSettings;
use crate::workspace::AppState;

/// How often to try again for a port another app holds.
const PORT_RETRY: Duration = Duration::from_secs(15);
/// How long after the last local edit to sync.
const SYNC_AFTER_EDIT: Duration = Duration::from_secs(20);

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
        /// Conflicts in that sync; the losing versions are in history.
        conflicts: usize,
        /// The documents that had them, to show and open the note.
        conflicted: Vec<String>,
    },
    Failed(String),
}

/// Whether a sync access token is in the Keychain. The token itself never
/// stays in memory here, only its masked form.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TokenState {
    /// Not looked up yet.
    #[default]
    Unknown,
    Missing,
    /// Saved; what Settings shows of it (see [`crate::settings::mask_secret`]).
    Saved(String),
    /// The Keychain wouldn't say: locked, or access was denied. A new token
    /// can still be set.
    Unreadable,
}

#[derive(Default)]
pub struct Services {
    pub api: ApiState,
    pub sync: SyncState,
    pub sync_token: TokenState,
    /// Whether the token may be looked up in the Keychain: only in the app,
    /// never in tests or snapshots, where macOS would ask for access.
    pub keychain: bool,
    /// A token lookup is on its way.
    token_lookup: bool,
    server: Option<ServerHandle>,
    port_retry: Option<Task<()>>,
    sync_tasks: Vec<Task<()>>,
    sync_after_edit: Option<Task<()>>,
    sync_running: bool,
    sync_again: bool,
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

    /// Sync's state in a few words, for a tooltip: the last sync's time as
    /// the sidebar's line says it, by the app's clock.
    pub fn sync_status(&self, cx: &App) -> String {
        match &self.sync {
            SyncState::Off => "Sync off".into(),
            SyncState::Syncing => "Syncing…".into(),
            SyncState::Synced {
                at_ms, conflicts, ..
            } => {
                let when = crate::clock::Clock::edited_label(cx, *at_ms);
                match conflicts {
                    0 => format!("Synced {when}"),
                    1 => format!("Synced {when}; 1 conflict kept in history"),
                    n => format!("Synced {when}; {n} conflicts kept in history"),
                }
            }
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

/// What the status bar says after a sync.
pub fn sync_state(result: Result<SyncReport, String>, now_ms: i64) -> SyncState {
    match result {
        Ok(report) => SyncState::Synced {
            at_ms: now_ms,
            conflicts: report.conflicts(),
            conflicted: report.conflicted,
        },
        Err(reason) => SyncState::Failed(reason),
    }
}

/// Whether a change should be sent to the server soon. Changes that came
/// from sync, and bulk imports, wait for the regular sync.
pub fn is_local_edit(change: &Change) -> bool {
    matches!(change.origin, Origin::Ui | Origin::Api)
}

/// Starts or stops background sync to match the settings.
pub fn apply_sync_settings(cx: &mut App) {
    let state = cx.global::<AppState>();
    let settings = state.config.sync.clone();
    let store = state.store.clone();
    let services = cx.global_mut::<Services>();
    services.sync_tasks.clear();
    services.sync_after_edit = None;
    if !settings.enabled || settings.endpoint.is_none() {
        services.sync = SyncState::Off;
        cx.refresh_windows();
        return;
    }
    let interval = Duration::from_secs(settings.interval_secs.max(60));
    let periodic = cx.spawn(async move |cx| {
        loop {
            cx.update(sync_now);
            cx.background_executor().timer(interval).await;
        }
    });
    let changes = store.subscribe();
    let after_edits = cx.spawn(async move |cx| {
        while let Ok(change) = changes.recv().await {
            if !is_local_edit(&change) {
                continue;
            }
            cx.update(|cx| {
                let delayed = cx.spawn(async move |cx| {
                    cx.background_executor().timer(SYNC_AFTER_EDIT).await;
                    cx.update(sync_now);
                });
                cx.global_mut::<Services>().sync_after_edit = Some(delayed);
            });
        }
    });
    let services = cx.global_mut::<Services>();
    services.sync_tasks = vec![periodic, after_edits];
}

/// Syncs once, now, on its own thread. A request during a sync runs right
/// after it.
pub fn sync_now(cx: &mut App) {
    let state = cx.global::<AppState>();
    let Some(endpoint) = state.config.sync.endpoint.clone() else {
        cx.global_mut::<Services>().sync =
            SyncState::Failed("set the Worker URL in Settings › Sync".into());
        cx.refresh_windows();
        return;
    };
    let store = state.store.clone();
    let services = cx.global_mut::<Services>();
    if services.sync_running {
        services.sync_again = true;
        return;
    }
    services.sync_running = true;
    services.sync = SyncState::Syncing;
    cx.refresh_windows();

    let (tx, rx) = async_channel::bounded(1);
    std::thread::Builder::new()
        .name("kanso-sync".into())
        .spawn(move || {
            let result = SyncConfig::new(&endpoint)
                .map_err(|err| format!("the Worker URL is refused: {err}"))
                .and_then(|config| {
                    config
                        .remote_from_keychain()
                        .map_err(|err| format!("couldn't read the access token: {err}"))
                })
                .and_then(|remote| {
                    remote.ok_or_else(|| "no access token; set one in Settings › Sync".to_owned())
                })
                .and_then(|remote| {
                    kanso_sync::sync_once(&store, &remote).map_err(|err| err.to_string())
                });
            tx.try_send(result).ok();
        })
        .ok();
    cx.spawn(async move |cx| {
        let result = rx
            .recv()
            .await
            .unwrap_or_else(|_| Err("the sync thread stopped".into()));
        cx.update(|cx| {
            let services = cx.global_mut::<Services>();
            services.sync_running = false;
            services.sync = sync_state(result, kanso_core::now_ms());
            let again = std::mem::take(&mut services.sync_again);
            cx.refresh_windows();
            if again {
                sync_now(cx);
            }
        });
    })
    .detach();
}

/// Keeps the sync access token in the Keychain, and notes that it's there.
pub fn save_sync_token(token: &str, cx: &mut App) -> anyhow::Result<()> {
    let token = token.trim();
    kanso_sync::credentials::save_token(token)?;
    cx.global_mut::<Services>().sync_token = TokenState::Saved(crate::settings::mask_secret(token));
    cx.refresh_windows();
    Ok(())
}

/// Takes the sync access token out of the Keychain; sync stops asking the
/// server until a new one is set.
pub fn remove_sync_token(cx: &mut App) -> anyhow::Result<()> {
    kanso_sync::credentials::delete_token()?;
    cx.global_mut::<Services>().sync_token = TokenState::Missing;
    cx.refresh_windows();
    Ok(())
}

/// Looks up whether a sync access token is saved, off the main thread, so
/// the Sync page can say so: only when it shows, only in the app (reading
/// the Keychain can make macOS ask for permission), and only until a lookup
/// has answered. One that failed is tried again the next time.
pub fn ensure_sync_token(cx: &mut App) {
    let Some(services) = cx.try_global::<Services>() else {
        return;
    };
    let wanted = matches!(
        services.sync_token,
        TokenState::Unknown | TokenState::Unreadable
    );
    if !services.keychain || services.token_lookup || !wanted {
        return;
    }
    cx.global_mut::<Services>().token_lookup = true;
    let lookup = cx.background_spawn(async move {
        kanso_sync::credentials::load_token()
            .map(|token| match token {
                Some(token) => TokenState::Saved(crate::settings::mask_secret(&token)),
                None => TokenState::Missing,
            })
            .map_err(|err| err.to_string())
    });
    cx.spawn(async move |cx| {
        let state = lookup.await;
        cx.update(|cx| {
            let state = state.unwrap_or_else(|err| {
                eprintln!("kanso: couldn't look up the sync token: {err}");
                TokenState::Unreadable
            });
            let services = cx.global_mut::<Services>();
            services.token_lookup = false;
            services.sync_token = state;
            cx.refresh_windows();
        });
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::{ApiState, SyncState, api_state, is_local_edit, server_config, sync_state};
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

    /// End to end through the Worker's rules (in memory): two libraries
    /// sync and the status reflects the result.
    #[test]
    fn syncing_two_libraries_reports_synced() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};
        let remote = kanso_sync::MemoryRemote::new();
        let (a_dir, b_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let a = Store::open(a_dir.path().join("k.db")).unwrap();
        let b = Store::open(b_dir.path().join("k.db")).unwrap();
        let book = a
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let note = a
            .put(
                DocType::Note,
                serde_json::json!({"title": "hello", "bookId": book.id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let state = sync_state(
            kanso_sync::sync_once(&a, &remote).map_err(|e| e.to_string()),
            7,
        );
        assert_eq!(
            state,
            SyncState::Synced {
                at_ms: 7,
                conflicts: 0,
                conflicted: Vec::new(),
            }
        );
        kanso_sync::sync_once(&b, &remote).unwrap();
        assert_eq!(b.note(&note.id).unwrap().unwrap().title, "hello");
        assert_eq!(
            sync_state(Err("offline".into()), 1),
            SyncState::Failed("offline".into())
        );
    }

    #[test]
    fn only_local_edits_trigger_a_quick_sync() {
        let change = |origin| kanso_store::Change {
            id: "note:a".into(),
            rev: "1-a".into(),
            seq: 1,
            deleted: false,
            origin,
        };
        assert!(is_local_edit(&change(kanso_store::Origin::Ui)));
        assert!(is_local_edit(&change(kanso_store::Origin::Api)));
        assert!(!is_local_edit(&change(kanso_store::Origin::Sync)));
        assert!(!is_local_edit(&change(kanso_store::Origin::Import)));
    }
}
