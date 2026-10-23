//! Kanso's theme gallery: the signed catalog published by `kanso-themes`,
//! and the files it names (docs/RELEASING.md › Theme gallery).
//!
//! The catalog is `gallery/index.json`, a list of themes with a monotonic
//! serial, each theme's version and the SHA-256 and size of every file it
//! names. `gallery/index.json.sig` is an Ed25519 signature over the index's
//! exact bytes, checked against the themes keys compiled in
//! ([`trusted_keys`]) before the JSON is parsed. Nothing is installed until
//! its own SHA-256 matches the signed entry, and an installed copy that has
//! been edited is never overwritten.
//!
//! The fetching, redirect checks and signature checks are the updater's
//! ([`kanso_update`]); this crate adds the catalog's shape, its path rules
//! and caps, the plan an automatic check follows, the SVG sanitizer and the
//! installer.

pub mod index;
pub mod install;
pub mod plan;
pub mod policy;
pub mod sanitize;
pub mod state;

pub use index::{
    Appearance, FileRef, Floor, HeadlineColors, Index, IndexError, Kind, Revoked, Theme,
    check_index, clean_text, is_valid_name, is_valid_svg_path, sha256_hex,
};
pub use install::{
    IconCaps, InstallError, color_digest, install_color, install_icons, install_icons_with,
    prepare, remove_installed, theme_url,
};
pub use plan::{Action, Installed, Plan, PlanState, Skipped, plan};
pub use policy::{GalleryPolicy, PolicyError};
pub use sanitize::{MAX_SVG_BYTES, SanitizeError, sanitize_svg};
pub use state::{Meta, State};

use ed25519_dalek::VerifyingKey;

/// The repository every URL is built from. A signed index can never name
/// another host.
pub const REPOSITORY: &str = "iamngoni/kanso-themes";

/// The branch the app reads. `main` is mutable between publications; the
/// signature and per-file hashes make that fail closed.
pub const BRANCH: &str = "main";

/// Where the catalog and everything it names live.
pub const RAW_BASE: &str = "https://raw.githubusercontent.com/iamngoni/kanso-themes/main";

/// The signed catalog and its signature.
pub const INDEX_URL: &str =
    "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/index.json";
pub const SIGNATURE_URL: &str =
    "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/index.json.sig";

/// The largest index, signature, color file, icon bundle and preview SVG the
/// app reads.
pub const MAX_INDEX_BYTES: u64 = 1024 * 1024;
pub const MAX_SIGNATURE_BYTES: u64 = 1024;
pub const MAX_COLOR_BYTES: u64 = 256 * 1024;
pub const MAX_BUNDLE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PREVIEW_BYTES: u64 = 128 * 1024;
/// Most themes one catalog may list, and most previews one icon theme may
/// name.
pub const MAX_CATALOG_ENTRIES: usize = 512;
pub const MAX_PREVIEWS: usize = 8;
/// Most gallery color and icon themes that may be installed at once.
pub const MAX_GALLERY_COLOR_THEMES: usize = 256;
pub const MAX_GALLERY_ICON_THEMES: usize = 64;
/// The content-addressed download cache's budget.
pub const MAX_CACHE_BYTES: u64 = 64 * 1024 * 1024;

/// The URL of a file the index names, built from the fixed repository and a
/// path the index validator has already checked. Pure.
pub fn file_url(path: &str) -> String {
    format!("{RAW_BASE}/{path}")
}

/// The themes-only public keys compiled into this build, from
/// `keys/themes-trusted-keys.txt`. Empty until the owner adds one, in which
/// case the gallery says it isn't set up and never fetches.
pub fn trusted_keys() -> Vec<VerifyingKey> {
    kanso_update::feed::parse_key_list(include_str!("../keys/themes-trusted-keys.txt"))
}

/// Whether this build has a key to verify a catalog with. Pure.
pub fn is_configured() -> bool {
    !trusted_keys().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_key_list_is_well_formed_and_separate_from_updates() {
        let shipped = include_str!("../keys/themes-trusted-keys.txt");
        // Every line that isn't a comment parses: no half-added key.
        for line in shipped
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            assert!(
                kanso_update::feed::parse_public_key(line).is_ok(),
                "themes-trusted-keys.txt has a line that isn't a key"
            );
        }
        // The gallery never accepts the update key, and the updater never
        // accepts a themes key.
        let updates = kanso_update::trusted_keys();
        for key in trusted_keys() {
            assert!(
                !updates.contains(&key),
                "the themes key list must not reuse an update key"
            );
        }
    }

    #[test]
    fn urls_stay_in_the_fixed_repository() {
        assert!(INDEX_URL.starts_with(RAW_BASE));
        assert!(SIGNATURE_URL.starts_with(RAW_BASE));
        assert_eq!(
            file_url("nord.json"),
            "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/nord.json"
        );
    }
}
