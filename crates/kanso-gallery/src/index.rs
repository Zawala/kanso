//! The signed catalog's shape, and every check before it is used.
//!
//! The index is verified as bytes first (`check_index`), then parsed with
//! `deny_unknown_fields`, then held to the rules the publisher promises:
//! valid ids and paths, positive serial and versions, a semver `min_kanso`,
//! an SPDX licence, unique ids and names, and no overlap between listed and
//! revoked themes. A failure means nothing is installed or removed.

use ed25519_dalek::VerifyingKey;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::{
    MAX_BUNDLE_BYTES, MAX_CATALOG_ENTRIES, MAX_COLOR_BYTES, MAX_INDEX_BYTES, MAX_PREVIEW_BYTES,
    MAX_PREVIEWS, MAX_SIGNATURE_BYTES,
};

/// The index layout this build reads. A newer one is "needs a newer Kanso",
/// never guessed at.
pub const SCHEMA: u32 = 1;

/// Whether a theme colors the app or draws its icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Color,
    Icons,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Color => "color",
            Kind::Icons => "icons",
        }
    }
}

/// A color theme's appearance, for pairing and the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}

impl Appearance {
    pub fn is_dark(self) -> bool {
        self == Appearance::Dark
    }
}

/// One file the index pins: a path inside the repository, its SHA-256 as
/// lowercase hex, and its size in bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    pub path: String,
    /// 64 lowercase hex digits.
    pub sha256: String,
    pub size: u64,
}

/// The five colors a card draws before its file has arrived.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeadlineColors {
    pub editor: String,
    pub text: String,
    pub accent: String,
    pub sidebar: String,
    pub list: String,
}

/// One theme in the catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// The file stem (color) or folder name (icons). Immutable.
    pub id: String,
    pub kind: Kind,
    /// The family a light and a dark theme pair under, when they do.
    #[serde(default)]
    pub family: Option<String>,
    /// Increases only when this theme's bytes change.
    pub version: u64,
    pub name: String,
    #[serde(default)]
    pub appearance: Option<Appearance>,
    pub author: String,
    /// An SPDX expression.
    pub license: String,
    /// Shown and opened in the browser; never fetched.
    pub source: String,
    pub description: String,
    /// A semver; an entry newer than the running app is shown and skipped.
    pub min_kanso: String,
    /// A color theme's one JSON file.
    #[serde(default)]
    pub file: Option<FileRef>,
    /// An icon theme's one deterministic `.tar.gz`.
    #[serde(default)]
    pub bundle: Option<FileRef>,
    /// Up to [`MAX_PREVIEWS`] real SVGs for an icon card.
    #[serde(default)]
    pub preview: Vec<FileRef>,
    /// Placeholder colors, used only before the file is fetched.
    #[serde(default)]
    pub colors: Option<HeadlineColors>,
}

impl Theme {
    /// The file this theme installs from, whichever kind it is.
    pub fn payload(&self) -> Option<&FileRef> {
        match self.kind {
            Kind::Color => self.file.as_ref(),
            Kind::Icons => self.bundle.as_ref(),
        }
    }

    /// Whether this build is new enough to install it. Pure.
    pub fn runs_on(&self, current: &Version) -> bool {
        Version::parse(&self.min_kanso).is_ok_and(|min| *current >= min)
    }
}

/// A theme the catalog owner withdrew. Signed, so a plain repository change
/// cannot withdraw anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revoked {
    pub id: String,
    pub reason: String,
    /// RFC 3339.
    pub since: String,
}

/// `gallery/index.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    pub schema: u32,
    /// A positive integer, one more for every publication.
    pub serial: u64,
    /// RFC 3339.
    pub generated_at: String,
    /// The commit the index was built from, for audit only.
    pub source_commit: String,
    /// Logged for diagnosis only; verification tries every trusted key.
    pub signing_key_id: String,
    pub themes: Vec<Theme>,
    #[serde(default)]
    pub revoked: Vec<Revoked>,
}

impl Index {
    pub fn theme(&self, id: &str) -> Option<&Theme> {
        self.themes.iter().find(|theme| theme.id == id)
    }

    pub fn revoked(&self, id: &str) -> Option<&Revoked> {
        self.revoked.iter().find(|entry| entry.id == id)
    }

    /// The serial and bytes a later index is compared with.
    pub fn floor(&self, bytes: &[u8]) -> Floor {
        Floor {
            serial: self.serial,
            digest: sha256_hex(bytes),
        }
    }
}

/// The highest serial accepted, and the digest of the exact index bytes
/// that carried it. Stored beside the cached index and checked at every
/// start. A lower serial is a rollback; an equal serial with different
/// bytes is equivocation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Floor {
    pub serial: u64,
    /// 64 lowercase hex digits; empty before anything was accepted.
    pub digest: String,
}

impl Floor {
    pub fn is_empty(&self) -> bool {
        self.serial == 0 && self.digest.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum IndexError {
    #[error("the theme gallery isn't set up in this build")]
    NoTrustedKeys,
    #[error("the theme catalog's signature doesn't match")]
    BadSignature,
    #[error("the theme catalog is too large")]
    TooLarge,
    #[error("the theme catalog can't be read")]
    Malformed,
    #[error("the theme catalog needs a newer Kanso")]
    NewerSchema,
    #[error("the theme catalog is older than the one Kanso already checked")]
    RolledBack,
    #[error("the theme catalog was rewritten without a new serial")]
    Equivocation,
    #[error("the theme catalog has an entry Kanso can't use: {0}")]
    BadEntry(String),
}

/// Verifies `signature` over `bytes`, parses the catalog and validates it
/// against `current` and the stored `floor`. Nothing is accepted before all
/// of it checks out. Pure.
pub fn check_index(
    bytes: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
    floor: &Floor,
    current: &Version,
) -> Result<Index, IndexError> {
    if keys.is_empty() {
        return Err(IndexError::NoTrustedKeys);
    }
    kanso_update::verify_signature_with_cap(
        bytes,
        signature,
        keys,
        MAX_INDEX_BYTES,
        MAX_SIGNATURE_BYTES,
    )
    .map_err(|err| match err {
        kanso_update::FeedError::NoTrustedKeys => IndexError::NoTrustedKeys,
        kanso_update::FeedError::TooLarge => IndexError::TooLarge,
        _ => IndexError::BadSignature,
    })?;
    let index: Index = serde_json::from_slice(bytes).map_err(|_| IndexError::Malformed)?;
    index.check(current)?;
    // The floor is raised only after every check above has passed.
    if index.serial < floor.serial {
        return Err(IndexError::RolledBack);
    }
    if index.serial == floor.serial && !floor.digest.is_empty() && sha256_hex(bytes) != floor.digest
    {
        return Err(IndexError::Equivocation);
    }
    Ok(index)
}

impl Index {
    /// Every catalogue rule, in the order a person would read them. Pure.
    pub fn check(&self, current: &Version) -> Result<(), IndexError> {
        if self.schema > SCHEMA {
            return Err(IndexError::NewerSchema);
        }
        if self.schema != SCHEMA {
            return Err(IndexError::Malformed);
        }
        if self.serial == 0 {
            return Err(IndexError::BadEntry("the serial is zero".into()));
        }
        if self.source_commit.len() != 40
            || !self.source_commit.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(IndexError::BadEntry(
                "source_commit isn't a commit hash".into(),
            ));
        }
        if self.themes.len() > MAX_CATALOG_ENTRIES {
            return Err(IndexError::BadEntry(format!(
                "it lists more than {MAX_CATALOG_ENTRIES} themes"
            )));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for theme in &self.themes {
            theme.check(current)?;
            if !ids.insert(theme.id.as_str()) {
                return Err(IndexError::BadEntry(format!(
                    "two themes have the id {}",
                    theme.id
                )));
            }
            let name = theme.name.to_lowercase();
            if !names.insert(name.clone()) {
                return Err(IndexError::BadEntry(format!(
                    "two themes are named {}",
                    theme.name
                )));
            }
        }
        let mut revoked = std::collections::BTreeSet::new();
        for entry in &self.revoked {
            if !is_valid_name(&entry.id) {
                return Err(IndexError::BadEntry(format!(
                    "revoked id {} isn't a valid id",
                    entry.id
                )));
            }
            if !revoked.insert(entry.id.as_str()) {
                return Err(IndexError::BadEntry(format!(
                    "{} is revoked twice",
                    entry.id
                )));
            }
            if ids.contains(entry.id.as_str()) {
                return Err(IndexError::BadEntry(format!(
                    "{} is both listed and revoked",
                    entry.id
                )));
            }
            if clean_text(&entry.reason).is_empty() {
                return Err(IndexError::BadEntry(format!(
                    "{} is revoked without a reason",
                    entry.id
                )));
            }
            if !is_rfc3339(&entry.since) {
                return Err(IndexError::BadEntry(format!(
                    "{} has an invalid revocation date",
                    entry.id
                )));
            }
        }
        if !is_rfc3339(&self.generated_at) {
            return Err(IndexError::BadEntry(
                "generated_at isn't an RFC 3339 date".into(),
            ));
        }
        Ok(())
    }
}

impl Theme {
    fn check(&self, _current: &Version) -> Result<(), IndexError> {
        if !is_valid_name(&self.id) {
            return Err(IndexError::BadEntry(format!(
                "the id {:?} isn't lowercase letters, digits, - and _",
                self.id
            )));
        }
        if clean_text(&self.name).is_empty() || self.name.chars().count() > 128 {
            return Err(IndexError::BadEntry(format!(
                "{} has a name that's empty or too long",
                self.id
            )));
        }
        for (what, text, cap) in [
            ("author", &self.author, 200usize),
            ("description", &self.description, 1000),
            ("license", &self.license, 64),
            ("family", &self.family.clone().unwrap_or_default(), 64),
        ] {
            if text.chars().count() > cap {
                return Err(IndexError::BadEntry(format!(
                    "{} has a {what} longer than {cap} characters",
                    self.id
                )));
            }
        }
        if !is_spdx(&self.license) {
            return Err(IndexError::BadEntry(format!(
                "{} doesn't have an SPDX licence",
                self.id
            )));
        }
        if !is_https(&self.source) {
            return Err(IndexError::BadEntry(format!(
                "{} has a source that isn't HTTPS",
                self.id
            )));
        }
        if Version::parse(&self.min_kanso).is_err() {
            return Err(IndexError::BadEntry(format!(
                "{} has a min_kanso that isn't a version",
                self.id
            )));
        }
        if self.version == 0 {
            return Err(IndexError::BadEntry(format!("{} has version 0", self.id)));
        }
        match self.kind {
            Kind::Color => {
                if self.bundle.is_some() || !self.preview.is_empty() {
                    return Err(IndexError::BadEntry(format!(
                        "{} is a color theme with a bundle or previews",
                        self.id
                    )));
                }
                let file = self
                    .file
                    .as_ref()
                    .ok_or_else(|| IndexError::BadEntry(format!("{} has no file", self.id)))?;
                let want = format!("{}.json", self.id);
                check_pin(file, "the color theme", MAX_COLOR_BYTES)?;
                if file.path != want {
                    return Err(IndexError::BadEntry(format!(
                        "{} names {}, not {want}",
                        self.id, file.path
                    )));
                }
                if let Some(colors) = &self.colors {
                    for color in [
                        &colors.editor,
                        &colors.text,
                        &colors.accent,
                        &colors.sidebar,
                        &colors.list,
                    ] {
                        if !is_hex_color(color) {
                            return Err(IndexError::BadEntry(format!(
                                "{} has a headline color that isn't #hex",
                                self.id
                            )));
                        }
                    }
                }
                if self.appearance.is_none() {
                    return Err(IndexError::BadEntry(format!(
                        "{} is a color theme without an appearance",
                        self.id
                    )));
                }
            }
            Kind::Icons => {
                if self.file.is_some() || self.colors.is_some() {
                    return Err(IndexError::BadEntry(format!(
                        "{} is an icon theme with a file or colors",
                        self.id
                    )));
                }
                let bundle = self
                    .bundle
                    .as_ref()
                    .ok_or_else(|| IndexError::BadEntry(format!("{} has no bundle", self.id)))?;
                let want = format!("gallery/icon-themes/{}.tar.gz", self.id);
                check_pin(bundle, "the icon bundle", MAX_BUNDLE_BYTES)?;
                if bundle.path != want {
                    return Err(IndexError::BadEntry(format!(
                        "{} names {}, not {want}",
                        self.id, bundle.path
                    )));
                }
                if self.preview.len() > MAX_PREVIEWS {
                    return Err(IndexError::BadEntry(format!(
                        "{} names more than {MAX_PREVIEWS} previews",
                        self.id
                    )));
                }
                for preview in &self.preview {
                    check_pin(preview, "an icon preview", MAX_PREVIEW_BYTES)?;
                    if !preview
                        .path
                        .starts_with(&format!("icon-themes/{}/", self.id))
                        || !is_valid_svg_path(&preview.path)
                    {
                        return Err(IndexError::BadEntry(format!(
                            "{} names a preview outside its folder",
                            self.id
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

fn check_pin(file: &FileRef, what: &str, cap: u64) -> Result<(), IndexError> {
    if !is_gallery_path(&file.path) {
        return Err(IndexError::BadEntry(format!(
            "{} names a path that leaves the repository",
            what
        )));
    }
    if !is_sha256(&file.sha256) {
        return Err(IndexError::BadEntry(format!(
            "{what} doesn't have a SHA-256"
        )));
    }
    if file.size == 0 || file.size > cap {
        return Err(IndexError::BadEntry(format!(
            "{what} has a size Kanso won't take"
        )));
    }
    Ok(())
}

/// Whether `path` can be a path inside the repository: relative,
/// `/`-separated, no `..`, no backslash, no leading slash, no
/// percent-encoded separator, and no empty part. Pure.
pub fn is_gallery_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 200
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains("%2e")
        && !path.contains("%2E")
        && !path.contains("%2f")
        && !path.contains("%2F")
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '-' | '_'))
}

/// Whether `name` can name a theme or an icon theme folder: lowercase ASCII
/// letters, digits, dashes and underscores, starting with a letter or
/// digit, at most 64 long. Pure; the same rule as the app's
/// `icon_theme_file::is_valid_name`.
pub fn is_valid_name(name: &str) -> bool {
    name.len() <= 64
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Whether `path` can name an SVG in an icon theme's folder: relative,
/// `/` between parts, each part of ASCII letters, digits, `.`, `-` and `_`,
/// none empty, `.` or `..`, ending in `.svg`. Pure; the same rule as the
/// app's `icon_theme_file::is_valid_svg_path`.
pub fn is_valid_svg_path(path: &str) -> bool {
    path.len() <= 200
        && path.to_ascii_lowercase().ends_with(".svg")
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        })
}

/// Whether `text` looks like an SPDX license expression (`MIT`,
/// `MIT OR Apache-2.0`). Pure; the same rule as the app's
/// `icon_theme_import::is_spdx`.
pub fn is_spdx(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && text.len() <= 64
        && text.chars().any(|c| c.is_ascii_alphabetic())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || " .-+()".contains(c))
}

/// Text as it may be shown: control and bidi formatting characters removed
/// and trimmed. Pure; display code cuts it to the width it has.
pub fn clean_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() && !is_bidi(*c))
        .collect::<String>()
        .trim()
        .to_owned()
}

fn is_bidi(c: char) -> bool {
    matches!(
        c,
        '\u{061C}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn is_sha256(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_hex_color(text: &str) -> bool {
    let bytes = text.as_bytes();
    matches!(bytes.len(), 4 | 7 | 9)
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

fn is_https(text: &str) -> bool {
    url::Url::parse(text).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

/// `YYYY-MM-DDTHH:MM:SSZ`, optionally with fractional seconds and a numeric
/// offset, as chrono writes it.
fn is_rfc3339(text: &str) -> bool {
    let bytes = text.as_bytes();
    if text.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }
    // Past the seconds: `.digits`, then `Z` or `±HH:MM`.
    let Some(mut rest) = text.get(19..) else {
        return false;
    };
    if let Some(fraction) = rest.strip_prefix('.') {
        let digits = fraction.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 {
            return false;
        }
        rest = &fraction[digits..];
    }
    if rest == "Z" {
        return true;
    }
    let offset = rest.as_bytes();
    offset.len() == 6
        && (offset[0] == b'+' || offset[0] == b'-')
        && offset[1..3].iter().all(|b| b.is_ascii_digit())
        && offset[3] == b':'
        && offset[4..6].iter().all(|b| b.is_ascii_digit())
}

/// A SHA-256 as lowercase hex. Pure.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The catalog's JSON, for a test or the publisher to read back.
pub fn to_value(index: &Index) -> Value {
    serde_json::to_value(index).expect("an index serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use proptest::prelude::*;

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn pin(path: &str, size: u64) -> FileRef {
        FileRef {
            path: path.into(),
            sha256: "ab".repeat(32),
            size,
        }
    }

    fn theme(id: &str, kind: Kind) -> Theme {
        Theme {
            id: id.into(),
            kind,
            family: Some("family".into()),
            version: 1,
            name: id.replace('-', " "),
            appearance: (kind == Kind::Color).then_some(Appearance::Dark),
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com/source".into(),
            description: "A theme.".into(),
            min_kanso: "0.1.0".into(),
            file: (kind == Kind::Color).then(|| pin(&format!("{id}.json"), 100)),
            bundle: (kind == Kind::Icons)
                .then(|| pin(&format!("gallery/icon-themes/{id}.tar.gz"), 100)),
            preview: if kind == Kind::Icons {
                vec![pin(&format!("icon-themes/{id}/icon.svg"), 10)]
            } else {
                Vec::new()
            },
            colors: None,
        }
    }

    fn sample() -> Index {
        Index {
            schema: SCHEMA,
            serial: 7,
            generated_at: "2026-10-23T12:00:00Z".into(),
            source_commit: "8d96298839a6176210cd1e3b376b82ffefc70ab3".into(),
            signing_key_id: "7f3a91c2".into(),
            themes: vec![
                theme("nord", Kind::Color),
                theme("catppuccin-icons", Kind::Icons),
            ],
            revoked: vec![Revoked {
                id: "old".into(),
                reason: "Withdrawn by its author.".into(),
                since: "2026-10-01T00:00:00Z".into(),
            }],
        }
    }

    fn signed(index: &Index, key: &SigningKey) -> (Vec<u8>, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(index).unwrap();
        let sig = kanso_update::sign_feed(&bytes, key).into_bytes();
        (bytes, sig)
    }

    fn check(bytes: &[u8], sig: &[u8], floor: &Floor) -> Result<Index, IndexError> {
        check_index(
            bytes,
            sig,
            &[key(1).verifying_key()],
            floor,
            &Version::new(0, 9, 0),
        )
    }

    #[test]
    fn a_valid_catalog_is_accepted() {
        let (bytes, sig) = signed(&sample(), &key(1));
        let got = check(&bytes, &sig, &Floor::default()).unwrap();
        assert_eq!(got.serial, 7);
        assert_eq!(got.theme("nord").unwrap().kind, Kind::Color);
    }

    #[test]
    fn a_newer_schema_is_named_and_an_older_one_is_malformed() {
        let mut newer = sample();
        newer.schema = SCHEMA + 1;
        let (bytes, sig) = signed(&newer, &key(1));
        assert_eq!(
            check(&bytes, &sig, &Floor::default()),
            Err(IndexError::NewerSchema)
        );
        let mut older = sample();
        older.schema = 0;
        let (bytes, sig) = signed(&older, &key(1));
        assert_eq!(
            check(&bytes, &sig, &Floor::default()),
            Err(IndexError::Malformed)
        );
    }

    #[test]
    fn the_serial_floor_stops_rollback_and_equivocation() {
        let (bytes, sig) = signed(&sample(), &key(1));
        let accepted = check(&bytes, &sig, &Floor::default()).unwrap();
        let floor = accepted.floor(&bytes);
        assert_eq!(check(&bytes, &sig, &floor).unwrap().serial, 7);
        let mut older = sample();
        older.serial = 6;
        let (older, older_sig) = signed(&older, &key(1));
        assert_eq!(
            check(&older, &older_sig, &floor),
            Err(IndexError::RolledBack)
        );
        let mut changed = sample();
        changed.signing_key_id = "another".into();
        let (changed, changed_sig) = signed(&changed, &key(1));
        assert_eq!(
            check(&changed, &changed_sig, &floor),
            Err(IndexError::Equivocation)
        );
    }

    #[test]
    fn an_empty_key_list_never_fetches_or_parses() {
        assert_eq!(
            check_index(
                b"not json",
                b"",
                &[],
                &Floor::default(),
                &Version::new(0, 9, 0)
            ),
            Err(IndexError::NoTrustedKeys)
        );
    }

    #[test]
    fn every_path_rule_is_checked() {
        let mut index = sample();
        index.themes[0].file = Some(pin("../evil.json", 10));
        assert!(matches!(
            index.check(&Version::new(0, 9, 0)),
            Err(IndexError::BadEntry(_))
        ));
        let mut index = sample();
        index.themes[1].preview = vec![pin("icon-themes/other/icon.svg", 10)];
        assert!(matches!(
            index.check(&Version::new(0, 9, 0)),
            Err(IndexError::BadEntry(_))
        ));
        assert!(!is_gallery_path("themes/%2e%2e/x.json"));
        assert!(!is_gallery_path("/themes/x.json"));
        assert!(!is_gallery_path("themes\\x.json"));
        assert!(is_gallery_path("nord.json"));
    }

    #[test]
    fn duplicate_ids_and_names_are_refused() {
        let mut index = sample();
        index.themes.push(index.themes[0].clone());
        assert!(matches!(
            index.check(&Version::new(0, 9, 0)),
            Err(IndexError::BadEntry(_))
        ));
        let mut index = sample();
        let mut other = theme("other", Kind::Color);
        other.name = index.themes[0].name.to_uppercase();
        index.themes.push(other);
        assert!(matches!(
            index.check(&Version::new(0, 9, 0)),
            Err(IndexError::BadEntry(_))
        ));
    }

    #[test]
    fn a_theme_that_is_both_listed_and_revoked_is_refused() {
        let mut index = sample();
        index.revoked.push(Revoked {
            id: "nord".into(),
            reason: "No.".into(),
            since: "2026-10-01T00:00:00Z".into(),
        });
        assert!(matches!(
            index.check(&Version::new(0, 9, 0)),
            Err(IndexError::BadEntry(_))
        ));
    }

    #[test]
    fn text_is_cleaned_of_control_and_bidi_characters() {
        assert_eq!(clean_text("  Nord\u{202E} \n"), "Nord");
        assert_eq!(clean_text("a\u{0}b"), "ab");
    }

    fn kind() -> impl Strategy<Value = Kind> {
        prop::sample::select(vec![Kind::Color, Kind::Icons])
    }

    fn generated_id() -> impl Strategy<Value = String> {
        "[a-z0-9][a-z0-9_-]{0,12}".prop_map(String::from)
    }

    proptest! {
        /// Signing one catalog never verifies as another: a one-byte change
        /// is refused before parsing, so no index that fails the signature
        /// is ever returned.
        #[test]
        fn prop_a_tampered_catalog_is_never_accepted(
            at in any::<prop::sample::Index>(),
            bit in 0u8..8,
            serial in 1u64..1000,
        ) {
            let mut index = sample();
            index.serial = serial;
            let (mut bytes, sig) = signed(&index, &key(1));
            let ix = at.index(bytes.len());
            bytes[ix] ^= 1 << bit;
            prop_assert!(matches!(
                check(&bytes, &sig, &Floor::default()),
                Err(IndexError::BadSignature) | Err(IndexError::Malformed)
            ));
        }

        /// A serial below the floor is always a rollback, whatever else the
        /// catalog says.
        #[test]
        fn prop_a_lower_serial_is_never_accepted(
            floor_serial in 1u64..1000,
            delta in 1u64..100,
            kind in kind(),
        ) {
            let mut index = sample();
            index.themes = vec![theme("nord", kind)];
            index.revoked.clear();
            index.serial = floor_serial.saturating_sub(delta).max(1);
            let (bytes, sig) = signed(&index, &key(1));
            let floor = Floor { serial: floor_serial, digest: sha256_hex(b"earlier") };
            prop_assume!(index.serial < floor.serial);
            prop_assert_eq!(check(&bytes, &sig, &floor), Err(IndexError::RolledBack));
        }

        /// Any id that isn't lowercase letters, digits, `-` and `_` is
        /// refused; a valid one is accepted.
        #[test]
        fn prop_ids_are_held_to_the_folder_rule(id in generated_id(), kind in kind()) {
            let mut index = sample();
            index.themes = vec![theme(&id, kind)];
            index.revoked.clear();
            prop_assert!(index.check(&Version::new(0, 9, 0)).is_ok(), "{}", id);
        }

        /// min_kanso decides whether a theme runs on this build, and a
        /// version that isn't semver never parses.
        #[test]
        fn prop_min_kanso_is_a_semver_gate(
            a in 0u64..3, b in 0u64..20, c in 0u64..20,
            running in 0u64..3,
        ) {
            let min = Version::new(a, b, c);
            let mut one = theme("nord", Kind::Color);
            one.min_kanso = min.to_string();
            let current = Version::new(running, 0, 0);
            prop_assert_eq!(one.runs_on(&current), current >= min);
            one.min_kanso = "not-a-version".into();
            let mut index = sample();
            index.themes = vec![one];
            index.revoked.clear();
            prop_assert!(index.check(&current).is_err());
        }
    }
}
