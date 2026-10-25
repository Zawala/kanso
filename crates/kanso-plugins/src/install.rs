//! Where plugins live on disk, and putting them there and taking them away.
//!
//! ```text
//! <data>/plugins/<id>/code/          the bundle's files (not for developer mode)
//! <data>/plugins/<id>/settings.json  the plugin's settings, owner-only
//! <data>/plugins/<id>/data/          its data store
//! <data>/plugins/<id>/egress.json    the last 100 network requests Kanso made for it
//! ```
//!
//! Installing follows the theme installer: the bundle is written into a
//! hidden folder beside the others and moved into place whole, so a plugin
//! is never half there. Nothing is written outside `plugins/`. Removing
//! deletes the plugin's folder; sweeping its data from notes is the
//! store's ([`kanso_store::Store::remove_plugin_data`]). Nothing here
//! syncs.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::manifest::is_valid_id;
use crate::registry_model::Bundle;

/// The folder all plugins live in.
pub fn plugins_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("plugins")
}

/// Plugin `id`'s own folder. `id` must already be valid.
pub fn plugin_dir(plugins: &Path, id: &str) -> PathBuf {
    debug_assert!(is_valid_id(id));
    plugins.join(id)
}

/// Where an installed bundle's files are.
pub fn code_dir(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join("code")
}

/// A random hidden name beside `parent`'s entries.
fn hidden(parent: &Path, what: &str) -> PathBuf {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("the system's random source");
    let tag: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    parent.join(format!(".{what}-{tag}"))
}

/// Writes `bundle`'s files as plugin `bundle.manifest.id`'s code, replacing
/// any it had, and returns the plugin's folder. The files go into a hidden
/// folder first and are moved into place in one rename.
pub fn install_bundle(plugins: &Path, bundle: &Bundle) -> std::io::Result<PathBuf> {
    let id = &bundle.manifest.id;
    if !is_valid_id(id) {
        return Err(std::io::Error::other("not a plugin ID"));
    }
    std::fs::create_dir_all(plugins)?;
    let dir = plugin_dir(plugins, id);
    std::fs::create_dir_all(&dir)?;
    let staging = hidden(&dir, "code");
    std::fs::create_dir(&staging)?;
    let written = (|| {
        let manifest = serde_json::to_vec_pretty(&bundle.manifest)?;
        std::fs::write(staging.join("manifest.json"), manifest)?;
        for (path, bytes) in &bundle.files {
            if !crate::manifest::is_valid_bundle_path(path) || path == "manifest.json" {
                return Err(std::io::Error::other("a bundle path escapes its folder"));
            }
            let target = staging.join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, bytes)?;
        }
        Ok::<_, std::io::Error>(())
    })();
    if let Err(err) = written {
        std::fs::remove_dir_all(&staging).ok();
        return Err(err);
    }
    let code = code_dir(&dir);
    let old = hidden(&dir, "old");
    let had = code.exists();
    if had {
        std::fs::rename(&code, &old)?;
    }
    if let Err(err) = std::fs::rename(&staging, &code) {
        if had {
            std::fs::rename(&old, &code).ok();
        }
        std::fs::remove_dir_all(&staging).ok();
        return Err(err);
    }
    if had {
        std::fs::remove_dir_all(&old).ok();
    }
    std::fs::create_dir_all(dir.join("data"))?;
    Ok(dir)
}

/// Deletes plugin `id`'s folder: code, settings, data and egress log.
pub fn remove(plugins: &Path, id: &str) -> std::io::Result<()> {
    if !is_valid_id(id) {
        return Err(std::io::Error::other("not a plugin ID"));
    }
    let dir = plugin_dir(plugins, id);
    match std::fs::remove_dir_all(&dir) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Writes `bytes` to `path` atomically, readable only by this user.
pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| std::io::Error::other("no folder"))?;
    std::fs::create_dir_all(dir)?;
    let temp = hidden(dir, "write");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        std::fs::remove_file(&temp).ok();
    }
    result
}

/// Plugin settings, or `null` when there are none or they can't be read.
pub fn read_settings(plugin_dir: &Path) -> Value {
    std::fs::read(plugin_dir.join("settings.json"))
        .ok()
        .filter(|bytes| bytes.len() <= crate::router::MAX_SETTINGS_BYTES)
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

/// Saves plugin settings, owner-only.
pub fn write_settings(plugin_dir: &Path, value: &Value) -> std::io::Result<()> {
    write_private(
        &plugin_dir.join("settings.json"),
        &serde_json::to_vec(value)?,
    )
}

/// The egress log Kanso keeps for a plugin.
pub fn read_egress(plugin_dir: &Path) -> crate::net::EgressLog {
    let entries: Vec<crate::net::Egress> = std::fs::read(plugin_dir.join("egress.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    let mut log = crate::net::EgressLog::new();
    // Stored newest first; pushed oldest first.
    for entry in entries.into_iter().rev() {
        log.push(entry);
    }
    log
}

/// Saves the egress log.
pub fn write_egress(plugin_dir: &Path, log: &crate::net::EgressLog) -> std::io::Result<()> {
    let entries = log.entries();
    write_private(
        &plugin_dir.join("egress.json"),
        &serde_json::to_vec(&entries)?,
    )
}

/// The entry script of a plugin: from its installed code, or, in developer
/// mode, read and checked from its folder again.
pub fn entry_script(
    plugin_dir: &Path,
    installed: &crate::model::Installed,
) -> Result<String, String> {
    let bundle_files = match &installed.source {
        crate::model::Source::Developer { path } => {
            let bundle = crate::bundle::open_dir(Path::new(path)).map_err(|e| e.to_string())?;
            if bundle.manifest.id != installed.manifest.id {
                return Err("the folder's manifest has another ID now".into());
            }
            bundle.files
        }
        _ => {
            let code = code_dir(plugin_dir);
            let path = code.join(&installed.manifest.entry);
            // The path was checked at install; keep to the folder anyway.
            if !crate::manifest::is_valid_bundle_path(&installed.manifest.entry) {
                return Err("the entry isn't inside the plugin".into());
            }
            let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if !meta.is_file() || meta.len() > crate::manifest::MAX_ENTRY_SCRIPT_BYTES {
                return Err("the entry script is missing or too large".into());
            }
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            return String::from_utf8(bytes).map_err(|_| "the entry script isn't UTF-8".into());
        }
    };
    let bytes = bundle_files
        .into_iter()
        .find(|(path, _)| *path == installed.manifest.entry)
        .map(|(_, bytes)| bytes)
        .ok_or("the entry script is missing")?;
    String::from_utf8(bytes).map_err(|_| "the entry script isn't UTF-8".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Manifest, Permissions, Publisher};

    fn bundle(files: Vec<(&str, &str)>) -> Bundle {
        Bundle {
            manifest: Manifest {
                schema: 1,
                api_version: 1,
                id: "com.example.wordcount".into(),
                name: "Word count".into(),
                version: "1.0.0".into(),
                entry: "main.js".into(),
                publisher: Publisher {
                    name: "Sam".into(),
                    key: format!("ed25519:{}", "ab".repeat(32)),
                },
                min_app_version: "0.1.0".into(),
                icon: None,
                description: None,
                permissions: Permissions::default(),
            },
            files: files
                .into_iter()
                .map(|(p, b)| (p.to_owned(), b.as_bytes().to_vec()))
                .collect(),
            sha256: String::new(),
        }
    }

    #[test]
    fn install_replaces_whole_and_remove_deletes_everything() {
        let data = tempfile::tempdir().unwrap();
        let plugins = plugins_dir(data.path());
        let dir =
            install_bundle(&plugins, &bundle(vec![("main.js", "1"), ("lib/a.js", "a")])).unwrap();
        write_settings(&dir, &serde_json::json!({ "x": 1 })).unwrap();
        assert_eq!(read_settings(&dir)["x"], 1);
        install_bundle(&plugins, &bundle(vec![("main.js", "2")])).unwrap();
        let code = code_dir(&dir);
        assert_eq!(std::fs::read_to_string(code.join("main.js")).unwrap(), "2");
        assert!(!code.join("lib/a.js").exists());
        // Settings survive an update.
        assert_eq!(read_settings(&dir)["x"], 1);
        // No hidden leftovers.
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().all(|n| !n.starts_with('.')), "{names:?}");
        remove(&plugins, "com.example.wordcount").unwrap();
        assert!(!dir.exists());
        remove(&plugins, "com.example.wordcount").unwrap();
        assert!(remove(&plugins, "../evil").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn settings_are_owner_only() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        write_settings(dir.path(), &serde_json::json!(1)).unwrap();
        let mode = std::fs::metadata(dir.path().join("settings.json"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn a_bad_path_writes_nothing() {
        let data = tempfile::tempdir().unwrap();
        let plugins = plugins_dir(data.path());
        assert!(install_bundle(&plugins, &bundle(vec![("../x.js", "1")])).is_err());
        assert!(!data.path().join("x.js").exists());
        let dir = plugin_dir(&plugins, "com.example.wordcount");
        assert!(!code_dir(&dir).exists());
    }
}
