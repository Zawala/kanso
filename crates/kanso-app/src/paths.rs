//! Where Kanso keeps its files.

use std::path::{Path, PathBuf};

/// `~/Library/Application Support/Kanso`, or `$KANSO_DATA_DIR` when set.
pub fn default_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("KANSO_DATA_DIR") {
        return PathBuf::from(dir);
    }
    home().join("Library/Application Support/Kanso")
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from)
}

pub fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("kanso.db")
}

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

pub fn backups_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

/// Inkdrop's data folder, the default import source.
pub fn inkdrop_dir() -> PathBuf {
    home().join("Library/Application Support/inkdrop")
}

/// The LaunchAgent that starts Kanso at login.
pub fn launch_agent_path() -> PathBuf {
    home().join("Library/LaunchAgents/app.kanso.Kanso.plist")
}
