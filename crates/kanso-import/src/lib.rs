//! Imports an Inkdrop database into Kanso's store.
//!
//! Inkdrop keeps its documents in PouchDB on SQLite at
//! `~/Library/Application Support/inkdrop/db/<32 hex>.sqlite` on macOS (see
//! [`default_inkdrop_dir`] for Windows and Linux). The importer
//! snapshots that file read-only (it may be open in Inkdrop), then copies the
//! winning revision of every note, notebook, tag and file, keeping their IDs
//! and revisions. Attachment bytes are checked against their MD5 digests;
//! Inkdrop's `activity:` log is archived verbatim; design documents, `config`
//! and `_local` documents are skipped.

mod config;
mod import;
pub mod pouch;
mod report;
pub mod safe_storage;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

pub use config::{
    DEFAULT_BIND_ADDRESS, DEFAULT_PORT, InkdropCredentials, InkdropServerConfig,
    parse_inkdrop_server_config, read_inkdrop_server_config,
};
pub use import::{Phase, Progress, import};
pub use report::{ImportReport, SourceCounts};

/// Inkdrop's data directory on macOS: `~/Library/Application Support/inkdrop`.
/// `None` when `HOME` is not set.
#[cfg(target_os = "macos")]
pub fn default_inkdrop_dir() -> Option<PathBuf> {
    non_empty_var("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/inkdrop"))
}

/// Inkdrop's data directory on Windows, Electron's `userData`:
/// `%APPDATA%\inkdrop`. `None` when `APPDATA` is not set.
#[cfg(target_os = "windows")]
pub fn default_inkdrop_dir() -> Option<PathBuf> {
    non_empty_var("APPDATA").map(|dir| PathBuf::from(dir).join("inkdrop"))
}

/// Inkdrop's data directory on Linux, Electron's `userData`:
/// `$XDG_CONFIG_HOME/inkdrop` or `~/.config/inkdrop`. `None` when neither
/// is set.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn default_inkdrop_dir() -> Option<PathBuf> {
    non_empty_var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| non_empty_var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config| config.join("inkdrop"))
}

fn non_empty_var(name: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

/// Whether `name` is a main database file name: 32 hex digits and `.sqlite`.
/// Excludes `*-mrview-*` view databases and `-wal`/`-shm` companions.
fn is_main_db_name(name: &str) -> bool {
    name.strip_suffix(".sqlite")
        .is_some_and(|stem| stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The main database in `<inkdrop_dir>/db`: the single `<32 hex>.sqlite` file.
pub fn find_inkdrop_db(inkdrop_dir: &Path) -> anyhow::Result<PathBuf> {
    let db_dir = inkdrop_dir.join("db");
    let entries =
        std::fs::read_dir(&db_dir).with_context(|| format!("cannot list {}", db_dir.display()))?;
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if entry.file_name().to_str().is_some_and(is_main_db_name) && path.is_file() {
            found.push(path);
        }
    }
    found.sort();
    match found.len() {
        0 => bail!(
            "no Inkdrop database (<32 hex digits>.sqlite) in {}",
            db_dir.display()
        ),
        1 => Ok(found.remove(0)),
        _ => bail!(
            "several Inkdrop databases in {}: {}",
            db_dir.display(),
            found
                .iter()
                .filter_map(|p| p.file_name()?.to_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_db_names() {
        assert!(is_main_db_name("5c172db466294e6a0b7855cdba0b8d18.sqlite"));
        assert!(!is_main_db_name(
            "5c172db466294e6a0b7855cdba0b8d18-mrview-21f5a21417c6d244c49767ad979917ae.sqlite"
        ));
        assert!(!is_main_db_name(
            "5c172db466294e6a0b7855cdba0b8d18.sqlite-wal"
        ));
        assert!(!is_main_db_name("5c172db466294e6a0b7855cdba0b8d18"));
        assert!(!is_main_db_name("fts.db"));
    }
}
