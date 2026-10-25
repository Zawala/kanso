//! The systems releases are built for, and what each one's update is.

/// A system Kanso ships a build for, as the feed names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Platform {
    MacosArm64,
    WindowsX64,
    LinuxX64,
    LinuxArm64,
}

/// How an update is packed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveKind {
    /// A gzipped tar: the `.app` on macOS, the folder with `kanso` on Linux.
    TarGz,
    /// A zip with `kanso.exe`.
    Zip,
}

impl Platform {
    pub const ALL: [Platform; 4] = [
        Platform::MacosArm64,
        Platform::WindowsX64,
        Platform::LinuxX64,
        Platform::LinuxArm64,
    ];

    /// The feed's key for it, and the end of its downloads' names.
    pub fn key(self) -> &'static str {
        match self {
            Platform::MacosArm64 => "macos-arm64",
            Platform::WindowsX64 => "windows-x64",
            Platform::LinuxX64 => "linux-x64",
            Platform::LinuxArm64 => "linux-arm64",
        }
    }

    pub fn from_key(key: &str) -> Option<Platform> {
        Platform::ALL.into_iter().find(|p| p.key() == key)
    }

    /// The system this program was built for, when releases have a build
    /// for it.
    pub const fn current() -> Option<Platform> {
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            Some(Platform::MacosArm64)
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Some(Platform::WindowsX64)
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Some(Platform::LinuxX64)
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            Some(Platform::LinuxArm64)
        } else {
            None
        }
    }

    pub fn archive(self) -> ArchiveKind {
        match self {
            Platform::WindowsX64 => ArchiveKind::Zip,
            Platform::MacosArm64 | Platform::LinuxX64 | Platform::LinuxArm64 => ArchiveKind::TarGz,
        }
    }

    /// The release asset that updates this system to `version`: on macOS
    /// the `.app` archived beside the disk image, elsewhere the download
    /// itself.
    pub fn asset_name(self, version: &str) -> String {
        let stem = format!("Kanso-{version}-{}", self.key());
        match self {
            Platform::MacosArm64 => format!("{stem}.app.tar.gz"),
            Platform::WindowsX64 => format!("{stem}.zip"),
            Platform::LinuxX64 | Platform::LinuxArm64 => format!("{stem}.tar.gz"),
        }
    }

    /// The plugin host's file name, which ships beside `kanso`. On macOS it
    /// lives inside the bundle and the updater never looks for it.
    pub fn host_name(self) -> &'static str {
        match self {
            Platform::WindowsX64 => "kanso-plugin-host.exe",
            Platform::MacosArm64 | Platform::LinuxX64 | Platform::LinuxArm64 => "kanso-plugin-host",
        }
    }

    /// Whether an update replaces a whole `.app` bundle rather than one
    /// executable.
    pub fn is_bundle(self) -> bool {
        self == Platform::MacosArm64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn asset_names_match_the_release_workflow() {
        assert_eq!(
            Platform::MacosArm64.asset_name("0.2.0"),
            "Kanso-0.2.0-macos-arm64.app.tar.gz"
        );
        assert_eq!(
            Platform::WindowsX64.asset_name("0.2.0"),
            "Kanso-0.2.0-windows-x64.zip"
        );
        assert_eq!(
            Platform::LinuxArm64.asset_name("1.0.0"),
            "Kanso-1.0.0-linux-arm64.tar.gz"
        );
    }

    #[test]
    fn this_machine_is_a_release_platform() {
        // The tests run on macOS on Apple silicon, and CI's clippy on Linux
        // and Windows x64: all four are release platforms.
        if cfg!(any(target_arch = "aarch64", target_arch = "x86_64")) {
            assert!(Platform::current().is_some());
        }
    }

    proptest! {
        /// Keys name exactly one platform, and every name carries its key.
        #[test]
        fn prop_keys_round_trip(ix in 0usize..4, version in "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}") {
            let platform = Platform::ALL[ix];
            prop_assert_eq!(Platform::from_key(platform.key()), Some(platform));
            let name = platform.asset_name(&version);
            prop_assert!(name.contains(platform.key()));
            prop_assert!(name.contains(&version));
            for other in Platform::ALL {
                if other != platform {
                    prop_assert_ne!(other.asset_name(&version), name.clone());
                }
            }
        }

        /// Nothing but the four keys names a platform.
        #[test]
        fn prop_unknown_keys_name_nothing(key in "[a-z0-9-]{0,14}") {
            let known = Platform::ALL.iter().any(|p| p.key() == key);
            prop_assert_eq!(Platform::from_key(&key).is_some(), known);
        }
    }
}
