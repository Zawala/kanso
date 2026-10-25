//! Which plugins are installed, what each was granted, and this Mac's block
//! list: kept in the library's local `meta` table, which never syncs.
//! Trusting a plugin is a decision about this machine.

use std::collections::BTreeSet;

use kanso_store::Store;
use serde::{Deserialize, Serialize};

use crate::model::Installed;

/// The `meta` key the state is kept under.
pub const META_KEY: &str = "plugins.state";

/// Every installed plugin and the block list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    #[serde(default)]
    pub plugins: Vec<Installed>,
    /// Plugin IDs this Mac refuses, whatever the directory says.
    #[serde(default)]
    pub blocked: BTreeSet<String>,
    /// The serial of the newest directory index seen, so an older one is
    /// refused as a replay.
    #[serde(default)]
    pub last_serial: Option<u64>,
}

impl State {
    /// The state kept in `store`; empty when there is none, or when what is
    /// kept can't be read (so nothing runs).
    pub fn load(store: &Store) -> State {
        store
            .meta(META_KEY)
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, store: &Store) -> kanso_store::Result<()> {
        store.set_meta(META_KEY, &serde_json::to_string(self)?)
    }

    pub fn get(&self, id: &str) -> Option<&Installed> {
        self.plugins.iter().find(|p| p.id() == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Installed> {
        self.plugins.iter_mut().find(|p| p.id() == id)
    }

    /// Adds `installed`, or replaces the plugin with its ID.
    pub fn upsert(&mut self, installed: Installed) {
        match self.get_mut(installed.id()) {
            Some(slot) => *slot = installed,
            None => {
                self.plugins.push(installed);
                self.plugins
                    .sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
            }
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<Installed> {
        let at = self.plugins.iter().position(|p| p.id() == id)?;
        Some(self.plugins.remove(at))
    }

    /// The IDs installed, for the lookalike warning.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.plugins.iter().map(Installed::id)
    }
}
