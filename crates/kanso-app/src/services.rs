//! Background services the window reports on: the local API and sync.

use gpui_kit::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ApiState {
    #[default]
    Off,
    Running(String),
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
}

impl Global for Services {}

impl Services {
    pub fn api_status(&self) -> String {
        match &self.api {
            ApiState::Off => "API off".into(),
            ApiState::Running(addr) => format!("API on {addr}"),
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
