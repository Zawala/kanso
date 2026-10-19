//! Kanso and Kanso Dev (docs/DEVELOPING.md).
//!
//! A build is one or the other, chosen at compile time: `--features
//! dev-flavour` makes Kanso Dev, which `scripts/bundle-macos.sh --dev`
//! packs as `Kanso Dev.app`. Kanso Dev has its own name, bundle ID, data
//! folder, Keychain service and API port, so it never opens or writes the
//! library or the account of the Kanso installed from a release, and it
//! never updates itself or swaps a release in.

/// What tells one build apart from the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flavour {
    /// The app's name, in its window title and bundle.
    pub name: &'static str,
    /// The macOS bundle ID, and the login item's label.
    pub bundle_id: &'static str,
    /// The data folder's name on macOS and Windows.
    pub data_folder: &'static str,
    /// The data folder's name under `$XDG_DATA_HOME` on Linux, and the
    /// desktop entry's (and Wayland app ID's) name.
    pub xdg_name: &'static str,
    /// The Keychain (or Credential Manager, or Secret Service) service its
    /// secrets are kept under.
    pub keychain_service: &'static str,
    /// The local API's port in a new config.
    pub default_port: u16,
    /// Whether it checks for, downloads and installs releases.
    pub updates: bool,
    /// A word the sidebar shows beside the name, so the two are never
    /// mistaken for each other.
    pub badge: Option<&'static str>,
}

/// The build releases ship.
pub const RELEASE: Flavour = Flavour {
    name: "Kanso",
    bundle_id: "app.kanso.Kanso",
    data_folder: "Kanso",
    xdg_name: "kanso",
    keychain_service: kanso_sync::keychain::SERVICE,
    default_port: crate::config::DEFAULT_PORT,
    updates: true,
    badge: None,
};

/// The build made from a working copy to try changes on.
pub const DEV: Flavour = Flavour {
    name: "Kanso Dev",
    bundle_id: "app.kanso.Kanso.dev",
    data_folder: "Kanso Dev",
    xdg_name: "kanso-dev",
    keychain_service: "Kanso Dev",
    default_port: crate::config::DEFAULT_PORT + 1,
    updates: false,
    badge: Some("DEV"),
};

/// This build's flavour.
pub const CURRENT: Flavour = if cfg!(feature = "dev-flavour") {
    DEV
} else {
    RELEASE
};

impl Flavour {
    pub fn is_dev(&self) -> bool {
        self.badge.is_some()
    }

    /// Its Keychain, under its own service.
    pub fn keychain(&self) -> kanso_sync::Keychain {
        kanso_sync::Keychain::with_service(self.keychain_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn dev_shares_nothing_with_the_release() {
        assert_ne!(DEV.name, RELEASE.name);
        assert_ne!(DEV.bundle_id, RELEASE.bundle_id);
        assert_ne!(DEV.data_folder, RELEASE.data_folder);
        assert_ne!(DEV.xdg_name, RELEASE.xdg_name);
        assert_ne!(DEV.keychain_service, RELEASE.keychain_service);
        assert_ne!(DEV.default_port, RELEASE.default_port);
        assert_ne!(RELEASE.updates, DEV.updates);
        const { assert!(!DEV.updates) };
        assert!(DEV.is_dev() && !RELEASE.is_dev());
        assert_eq!(DEV.keychain().service(), "Kanso Dev");
        assert_eq!(RELEASE.keychain().service(), "Kanso");
    }

    #[test]
    fn the_release_keeps_its_names() {
        // What 0.1.0 used: changing any of these would lose people's
        // libraries and sign-ins.
        assert_eq!(RELEASE.bundle_id, "app.kanso.Kanso");
        assert_eq!(RELEASE.data_folder, "Kanso");
        assert_eq!(RELEASE.xdg_name, "kanso");
        assert_eq!(RELEASE.keychain_service, "Kanso");
        assert_eq!(RELEASE.default_port, 19840);
        assert_eq!(crate::launch::label_for(&RELEASE), "app.kanso.Kanso");
    }

    #[test]
    fn this_build_is_the_one_its_features_say() {
        assert_eq!(CURRENT == DEV, cfg!(feature = "dev-flavour"));
    }

    #[test]
    fn dev_updates_are_off_whatever_the_install() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("Kanso Dev.app/Contents/MacOS/kanso");
        let options = kanso_update::LocateOptions {
            dev_flavour: DEV.is_dev(),
            ..Default::default()
        };
        assert_eq!(
            kanso_update::install::locate(&exe, Some(kanso_update::Platform::MacosArm64), options),
            Err(kanso_update::NotInstallable::DevFlavour)
        );
        assert_eq!(
            kanso_update::NotInstallable::DevFlavour.reason(),
            "Development build — updates are off."
        );
    }

    proptest! {
        /// Whatever folder a flavour's data lives under, the other's is a
        /// different one: neither is inside the other.
        #[test]
        fn prop_data_folders_never_overlap(base in "[a-z]{1,8}(/[a-z]{1,8}){0,3}", dev_first in any::<bool>()) {
            let base = std::path::PathBuf::from("/").join(base);
            let (a, b) = if dev_first { (DEV, RELEASE) } else { (RELEASE, DEV) };
            for (x, y) in [(a.data_folder, b.data_folder), (a.xdg_name, b.xdg_name)] {
                let (x, y) = (base.join(x), base.join(y));
                prop_assert!(!x.starts_with(&y) && !y.starts_with(&x));
            }
        }
    }
}
