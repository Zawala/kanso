//! `config.json` in the data directory. It holds the API credentials, so it is
//! written with owner-only permissions and never logged.

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 19840;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ServerSettings {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub username: String,
    pub password: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        ServerSettings {
            enabled: true,
            port: DEFAULT_PORT,
            bind_address: "127.0.0.1".into(),
            username: String::new(),
            password: String::new(),
        }
    }
}

impl fmt::Debug for ServerSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerSettings")
            .field("enabled", &self.enabled)
            .field("port", &self.port)
            .field("bind_address", &self.bind_address)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SyncSettings {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub interval_secs: u64,
}

impl Default for SyncSettings {
    fn default() -> Self {
        SyncSettings {
            enabled: false,
            endpoint: None,
            interval_secs: 300,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewMode {
    Edit,
    #[default]
    Split,
    Preview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub theme: ThemePreference,
    pub editor_font_size: f32,
    pub view_mode: ViewMode,
    pub show_sidebar: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            theme: ThemePreference::System,
            editor_font_size: 14.0,
            view_mode: ViewMode::Split,
            show_sidebar: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub server: ServerSettings,
    pub sync: SyncSettings,
    pub appearance: Appearance,
    pub launch_at_login: bool,
    pub last_note: Option<String>,
}

impl AppConfig {
    /// Reads the config, falling back to defaults for a missing file or field.
    pub fn load(path: &Path) -> anyhow::Result<AppConfig> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(err) => Err(err.into()),
        }
    }

    /// Writes the config atomically with owner-only permissions.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        use std::io::Write as _;
        use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)?;
        // `mode` only applies to a new file; a leftover one keeps its own.
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Fills in API credentials on first run so the server never runs open.
    pub fn ensure_credentials(&mut self) -> bool {
        let mut changed = false;
        if self.server.username.is_empty() {
            self.server.username = "kanso".into();
            changed = true;
        }
        if self.server.password.is_empty() {
            self.server.password = (0..24).map(|_| fastrand::alphanumeric()).collect();
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn gen_config() -> impl Strategy<Value = AppConfig> {
        (
            any::<bool>(),
            any::<u16>(),
            "[a-z0-9.:]{0,15}",
            "\\PC{0,12}",
            "\\PC{0,24}",
            (
                any::<bool>(),
                prop::option::of("https://[a-z.]{1,20}"),
                1u64..100_000,
            ),
            (
                prop::sample::select(vec![
                    ThemePreference::System,
                    ThemePreference::Light,
                    ThemePreference::Dark,
                ]),
                8.0f32..32.0,
                prop::sample::select(vec![ViewMode::Edit, ViewMode::Split, ViewMode::Preview]),
                any::<bool>(),
            ),
            any::<bool>(),
            prop::option::of("note:[a-zA-Z0-9]{8}"),
        )
            .prop_map(
                |(
                    enabled,
                    port,
                    bind,
                    user,
                    pass,
                    (s_on, endpoint, interval),
                    (theme, size, mode, sidebar),
                    login,
                    last,
                )| AppConfig {
                    server: ServerSettings {
                        enabled,
                        port,
                        bind_address: bind,
                        username: user,
                        password: pass,
                    },
                    sync: SyncSettings {
                        enabled: s_on,
                        endpoint,
                        interval_secs: interval,
                    },
                    appearance: Appearance {
                        theme,
                        editor_font_size: size,
                        view_mode: mode,
                        show_sidebar: sidebar,
                    },
                    launch_at_login: login,
                    last_note: last,
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Round-trip: saving then loading gives the same config.
        #[test]
        fn prop_config_save_load_roundtrip(config in gen_config()) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("config.json");
            config.save(&path).unwrap();
            prop_assert_eq!(AppConfig::load(&path).unwrap(), config);
        }

        /// Refinement: debug output never contains the password.
        #[test]
        fn prop_debug_redacts_password(password in "[a-zA-Z0-9]{8,24}") {
            let settings = ServerSettings { password: password.clone(), ..ServerSettings::default() };
            let debug = format!("{settings:?}");
            prop_assert!(!debug.contains(&password));
        }
    }

    #[test]
    fn missing_fields_take_defaults_and_file_is_private() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"server": {"port": 1234}}"#).unwrap();
        let mut config = AppConfig::load(&path).unwrap();
        assert_eq!(config.server.port, 1234);
        assert_eq!(config.server.bind_address, "127.0.0.1");
        assert_eq!(config.appearance.view_mode, ViewMode::Split);
        assert!(config.ensure_credentials());
        assert!(!config.ensure_credentials());
        config.save(&path).unwrap();
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            AppConfig::load(&dir.path().join("missing.json")).unwrap(),
            AppConfig::default()
        );
    }

    #[test]
    fn a_leftover_temporary_file_does_not_widen_permissions() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, "{}").unwrap();
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644)).unwrap();
        AppConfig::default().save(&path).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
