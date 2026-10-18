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

/// The editor's typeface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorFont {
    /// Proportional for writing, monospace beside the preview.
    #[default]
    Auto,
    Sans,
    Mono,
}

/// How the editor sets its text in one view mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorFace {
    pub mono: bool,
    pub line_numbers: bool,
}

impl EditorFont {
    /// Line numbers come with the monospace face, where source lines and
    /// their numbers line up.
    pub fn face(self, mode: ViewMode) -> EditorFace {
        let mono = match self {
            EditorFont::Auto => mode == ViewMode::Split,
            EditorFont::Sans => false,
            EditorFont::Mono => true,
        };
        EditorFace {
            mono,
            line_numbers: mono,
        }
    }
}

impl EditorFace {
    /// The face with line numbers only where the setting allows them.
    pub fn with_line_numbers(self, allowed: bool) -> EditorFace {
        EditorFace {
            line_numbers: self.line_numbers && allowed,
            ..self
        }
    }
}

/// The editor text size before version 1, kept by anyone who never changed it.
const OLD_DEFAULT_FONT_SIZE: f32 = 14.0;
pub const DEFAULT_FONT_SIZE: f32 = 16.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub theme: ThemePreference,
    pub editor_font: EditorFont,
    pub editor_font_size: f32,
    /// Whether monospace text shows line numbers beside it.
    pub line_numbers: bool,
    /// Whether text keeps to a readable measure, or fills the pane.
    pub readable_width: bool,
    pub view_mode: ViewMode,
    /// The full sidebar, or (false) the icon rail it collapses to.
    pub show_sidebar: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            theme: ThemePreference::System,
            editor_font: EditorFont::Auto,
            editor_font_size: DEFAULT_FONT_SIZE,
            line_numbers: true,
            readable_width: true,
            view_mode: ViewMode::Split,
            show_sidebar: true,
        }
    }
}

/// The source side's share of the split before it is first dragged.
pub const DEFAULT_SPLIT_RATIO: f32 = 0.5;

/// How the window's panes were left: focus mode, where the split divides,
/// and whether the sidebar's Status group is open.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowLayout {
    pub focus_mode: bool,
    /// The source's share of the editor in Split mode, between 0 and 1.
    pub split_ratio: f32,
    /// Open or closed by the user; `None` until they choose.
    pub status_open: Option<bool>,
}

impl Default for WindowLayout {
    fn default() -> Self {
        WindowLayout {
            focus_mode: false,
            split_ratio: DEFAULT_SPLIT_RATIO,
            status_open: None,
        }
    }
}

/// The layout [`AppConfig::migrate`] brings older files up to.
pub const CONFIG_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    /// Files written before versions existed read as 0.
    #[serde(default)]
    pub version: u32,
    pub server: ServerSettings,
    pub sync: SyncSettings,
    pub appearance: Appearance,
    pub launch_at_login: bool,
    pub last_note: Option<String>,
    pub layout: WindowLayout,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: CONFIG_VERSION,
            server: ServerSettings::default(),
            sync: SyncSettings::default(),
            appearance: Appearance::default(),
            launch_at_login: false,
            last_note: None,
            layout: WindowLayout::default(),
        }
    }
}

impl AppConfig {
    /// Brings a config from an older version up to [`CONFIG_VERSION`], and
    /// says whether anything changed. Version 1 raised the default editor
    /// size from 14 to 16; a size someone chose stays.
    pub fn migrate(&mut self) -> bool {
        if self.version >= CONFIG_VERSION {
            return false;
        }
        if self.version < 1 && self.appearance.editor_font_size == OLD_DEFAULT_FONT_SIZE {
            self.appearance.editor_font_size = DEFAULT_FONT_SIZE;
        }
        self.version = CONFIG_VERSION;
        true
    }

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
    use proptest::test_runner::FileFailurePersistence;

    /// Any config, with the window layout too.
    fn gen_config() -> impl Strategy<Value = AppConfig> {
        (gen_settings(), gen_layout()).prop_map(|(config, layout)| AppConfig { layout, ..config })
    }

    fn gen_layout() -> impl Strategy<Value = WindowLayout> {
        (any::<bool>(), 0.0f32..=1.0, prop::option::of(any::<bool>())).prop_map(
            |(focus_mode, split_ratio, status_open)| WindowLayout {
                focus_mode,
                split_ratio,
                status_open,
            },
        )
    }

    fn gen_settings() -> impl Strategy<Value = AppConfig> {
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
                gen_font(),
                prop_oneof![
                    Just(OLD_DEFAULT_FONT_SIZE),
                    Just(DEFAULT_FONT_SIZE),
                    8.0f32..32.0
                ],
                any::<bool>(),
                gen_mode(),
                any::<bool>(),
                any::<bool>(),
            ),
            any::<bool>(),
            prop::option::of("note:[a-zA-Z0-9]{8}"),
            0u32..=CONFIG_VERSION + 1,
        )
            .prop_map(
                |(
                    enabled,
                    port,
                    bind,
                    user,
                    pass,
                    (s_on, endpoint, interval),
                    (theme, font, size, numbers, mode, sidebar, readable),
                    login,
                    last,
                    version,
                )| AppConfig {
                    version,
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
                        editor_font: font,
                        editor_font_size: size,
                        line_numbers: numbers,
                        readable_width: readable,
                        view_mode: mode,
                        show_sidebar: sidebar,
                    },
                    launch_at_login: login,
                    last_note: last,
                    layout: WindowLayout::default(),
                },
            )
    }

    fn gen_font() -> impl Strategy<Value = EditorFont> {
        prop::sample::select(vec![EditorFont::Auto, EditorFont::Sans, EditorFont::Mono])
    }

    fn gen_mode() -> impl Strategy<Value = ViewMode> {
        prop::sample::select(vec![ViewMode::Edit, ViewMode::Split, ViewMode::Preview])
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Migration ends at the current version, is idempotent, changes
        /// nothing but the version and a never-changed old default size, and
        /// leaves current files alone.
        #[test]
        fn prop_migration_only_lifts_the_old_default_size(config in gen_config()) {
            let mut migrated = config.clone();
            let changed = migrated.migrate();
            prop_assert_eq!(migrated.version, CONFIG_VERSION.max(config.version));
            prop_assert_eq!(changed, config.version < CONFIG_VERSION);
            let expected_size = if config.version < 1 && config.appearance.editor_font_size == OLD_DEFAULT_FONT_SIZE {
                DEFAULT_FONT_SIZE
            } else {
                config.appearance.editor_font_size
            };
            prop_assert_eq!(migrated.appearance.editor_font_size, expected_size);
            let mut rest = migrated.clone();
            rest.version = config.version;
            rest.appearance.editor_font_size = config.appearance.editor_font_size;
            prop_assert_eq!(&rest, &config);
            let mut again = migrated.clone();
            prop_assert!(!again.migrate());
            prop_assert_eq!(again, migrated);
        }

        /// Line numbers only ever come with the monospace face, and turning
        /// the setting off removes them without changing the face.
        #[test]
        fn prop_line_numbers_need_mono_and_the_setting(font in gen_font(), mode in gen_mode(), allowed: bool) {
            let face = font.face(mode);
            let shown = face.with_line_numbers(allowed);
            prop_assert_eq!(shown.mono, face.mono);
            prop_assert_eq!(shown.line_numbers, face.mono && allowed);
            prop_assert_eq!(face.with_line_numbers(true), face);
        }

        /// A file without a version (written before versions existed) loads
        /// as version 0, so its default size is migrated.
        #[test]
        fn prop_unversioned_files_migrate(config in gen_config()) {
            let mut value = serde_json::to_value(&config).unwrap();
            value.as_object_mut().unwrap().remove("version");
            let mut loaded: AppConfig = serde_json::from_value(value).unwrap();
            prop_assert_eq!(loaded.version, 0);
            loaded.migrate();
            prop_assert_eq!(loaded.version, CONFIG_VERSION);
        }
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

    /// Auto writes in the proportional face and switches to monospace with
    /// line numbers beside the preview; the fixed choices hold in every mode.
    #[test]
    fn editor_faces() {
        use ViewMode::{Edit, Preview, Split};
        let face = |font: EditorFont, mode| {
            let face = font.face(mode);
            assert_eq!(face.line_numbers, face.mono);
            face.mono
        };
        assert!(!face(EditorFont::Auto, Edit));
        assert!(face(EditorFont::Auto, Split));
        assert!(!face(EditorFont::Auto, Preview));
        for mode in [Edit, Split, Preview] {
            assert!(!face(EditorFont::Sans, mode));
            assert!(face(EditorFont::Mono, mode));
        }
    }

    #[test]
    fn old_default_size_is_migrated_once() {
        let mut old: AppConfig =
            serde_json::from_str(r#"{"appearance": {"editorFontSize": 14}}"#).unwrap();
        assert!(old.migrate());
        assert_eq!(old.appearance.editor_font_size, 16.0);
        assert_eq!(old.appearance.editor_font, EditorFont::Auto);
        let mut chosen: AppConfig =
            serde_json::from_str(r#"{"appearance": {"editorFontSize": 15}}"#).unwrap();
        assert!(chosen.migrate());
        assert_eq!(chosen.appearance.editor_font_size, 15.0);
        // Someone who picks 14 after the migration keeps it.
        let mut current: AppConfig =
            serde_json::from_str(r#"{"version": 1, "appearance": {"editorFontSize": 14}}"#)
                .unwrap();
        assert!(!current.migrate());
        assert_eq!(current.appearance.editor_font_size, 14.0);
        assert_eq!(AppConfig::default().version, CONFIG_VERSION);
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
        // Files from before the setting keep their line numbers, and their
        // text to its readable measure.
        assert!(config.appearance.line_numbers);
        assert!(config.appearance.readable_width);
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

    /// Files from before the window layout was kept load with its defaults,
    /// and a partial one keeps what it has.
    #[test]
    fn older_files_take_the_default_layout() {
        let old: AppConfig =
            serde_json::from_str(r#"{"version": 1, "appearance": {"showSidebar": false}}"#)
                .unwrap();
        assert_eq!(old.layout, WindowLayout::default());
        assert!(!old.appearance.show_sidebar);
        assert_eq!(old.layout.split_ratio, DEFAULT_SPLIT_RATIO);
        let partial: AppConfig =
            serde_json::from_str(r#"{"layout": {"focusMode": true}}"#).unwrap();
        assert!(partial.layout.focus_mode);
        assert_eq!(partial.layout.split_ratio, DEFAULT_SPLIT_RATIO);
        assert_eq!(partial.layout.status_open, None);
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
