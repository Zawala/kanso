//! Published notebooks (docs/ACCOUNTS.md › Published notebooks): a
//! notebook's notes as a small site, published as one share.
//!
//! The site is one bundle, sealed and uploaded exactly as a note's page is
//! ([`crate::share::publish`]): the same 10 MiB, the same route, the same
//! link `<server>/p/<id>#<key>`. The share page opens it and finds JSON
//! (`{"kanso":"site",…}`) where a note's share is HTML, and shows the
//! index; `#<key>/<slug>` shows a page. Pages link to each other as
//! `kanso-page:<slug>` and carry their images as `src="kanso-file:<n>"`, an
//! index into [`SiteBundle::files`], so an image the notes share is in the
//! bundle once. The key stays in the fragment, so the server never sees the
//! site any more than it sees a note.
//!
//! What a notebook publishes is kept in the notebook, as
//! `book.extra.kansoPublish` (the same [`crate::share::KansoPublish`]), so
//! it syncs end to end encrypted and the account's other devices can update
//! or unpublish the same link.

use std::collections::BTreeMap;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// What a bundle says it is.
pub const SITE_KIND: &str = "site";
/// The only version so far.
pub const SITE_VERSION: u32 = 1;
/// A link from one page to another: this, then the page's slug (empty for
/// the index). The same as `kanso_render::html::PAGE_LINK_PREFIX`.
pub const PAGE_LINK_PREFIX: &str = "kanso-page:";
/// An image's source in a page: this, then its index in the files.
pub const FILE_SRC_PREFIX: &str = "kanso-file:";
/// The longest slug.
pub const MAX_SLUG: usize = 64;

/// A published notebook.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteBundle {
    /// Always [`SITE_KIND`].
    pub kanso: String,
    /// Always [`SITE_VERSION`].
    pub v: u32,
    /// The notebook's name.
    pub title: String,
    /// The first page: the notebook's notes, each linking to its page.
    pub index: String,
    /// The notes' pages by slug.
    pub pages: BTreeMap<String, SitePage>,
    /// The images and other attachments the pages use.
    pub files: Vec<SiteFile>,
}

/// A note's page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitePage {
    pub title: String,
    pub html: String,
    /// The attachments (other than images) the note links to, as indices
    /// into the files: the share page offers them to download.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downloads: Vec<usize>,
}

/// An attachment in the bundle.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SiteFile {
    pub name: String,
    #[serde(rename = "type")]
    pub content_type: String,
    /// The bytes as standard base64, as a `data:` URL carries them.
    pub data: String,
}

impl SiteFile {
    pub fn new(name: &str, content_type: &str, bytes: &[u8]) -> SiteFile {
        SiteFile {
            name: name.to_owned(),
            content_type: content_type.to_owned(),
            data: base64::engine::general_purpose::STANDARD.encode(bytes),
        }
    }

    /// The bytes, when the data is base64.
    pub fn bytes(&self) -> Option<Vec<u8>> {
        base64::engine::general_purpose::STANDARD
            .decode(&self.data)
            .ok()
    }

    /// `data:<type>;base64,<data>`: what a page's image is once the share
    /// page puts it back.
    pub fn data_uri(&self) -> String {
        format!("data:{};base64,{}", self.content_type, self.data)
    }
}

/// What can be wrong with a bundle.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SiteError {
    #[error("a page links to `{0}`, which isn't in the site")]
    MissingPage(String),
    #[error("a page uses file {0}, which isn't in the site")]
    MissingFile(usize),
    #[error("`{0}` can't be a page's address")]
    BadSlug(String),
}

impl SiteBundle {
    pub fn new(title: &str) -> SiteBundle {
        SiteBundle {
            kanso: SITE_KIND.into(),
            v: SITE_VERSION,
            title: title.to_owned(),
            index: String::new(),
            pages: BTreeMap::new(),
            files: Vec::new(),
        }
    }

    /// The bundle as published (before sealing): compact JSON.
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("a bundle serializes")
    }

    /// A bundle from published bytes; `None` for anything else, a note's
    /// page among them.
    pub fn from_bytes(bytes: &[u8]) -> Option<SiteBundle> {
        let bundle: SiteBundle = serde_json::from_slice(bytes).ok()?;
        (bundle.kanso == SITE_KIND && bundle.v == SITE_VERSION).then_some(bundle)
    }

    /// Every page link and image source resolves within the bundle, and
    /// every slug can be an address. What the share page relies on.
    pub fn check(&self) -> Result<(), SiteError> {
        if let Some(slug) = self.pages.keys().find(|slug| !is_slug(slug)) {
            return Err(SiteError::BadSlug(slug.clone()));
        }
        for html in std::iter::once(&self.index).chain(self.pages.values().map(|p| &p.html)) {
            for slug in page_links(html) {
                if !slug.is_empty() && !self.pages.contains_key(&slug) {
                    return Err(SiteError::MissingPage(slug));
                }
            }
            for file in file_sources(html) {
                if file >= self.files.len() {
                    return Err(SiteError::MissingFile(file));
                }
            }
        }
        for page in self.pages.values() {
            if let Some(&file) = page.downloads.iter().find(|&&f| f >= self.files.len()) {
                return Err(SiteError::MissingFile(file));
            }
        }
        Ok(())
    }

    /// Puts each of `files` in the bundle once, and in every page and the
    /// index swaps the image's `data:` URL for its `kanso-file:<n>`. Returns
    /// each file's index, in the order given.
    pub fn share_images(&mut self, files: Vec<SiteFile>) -> Vec<usize> {
        let mut indices = Vec::with_capacity(files.len());
        for file in files {
            let at = match self.files.iter().position(|f| *f == file) {
                Some(at) => at,
                None => {
                    self.files.push(file);
                    self.files.len() - 1
                }
            };
            let from = format!("src=\"{}\"", self.files[at].data_uri());
            let to = format!("src=\"{FILE_SRC_PREFIX}{at}\"");
            for page in self.pages.values_mut() {
                if page.html.contains(&from) {
                    page.html = page.html.replace(&from, &to);
                }
            }
            if self.index.contains(&from) {
                self.index = self.index.replace(&from, &to);
            }
            indices.push(at);
        }
        indices
    }

    /// The page `slug` with its images put back as `data:` URLs, as the
    /// share page shows it; the index for the empty slug.
    pub fn page_html(&self, slug: &str) -> Option<String> {
        let html = if slug.is_empty() {
            &self.index
        } else {
            &self.pages.get(slug)?.html
        };
        let mut out = html.clone();
        for file in file_sources(html) {
            if let Some(f) = self.files.get(file) {
                out = out.replace(
                    &format!("src=\"{FILE_SRC_PREFIX}{file}\""),
                    &format!("src=\"{}\"", f.data_uri()),
                );
            }
        }
        Some(out)
    }
}

/// Whether `slug` can name a page: 1 to [`MAX_SLUG`] of `a-z`, `0-9` and
/// `-`. Pure.
pub fn is_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= MAX_SLUG
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A slug from a title: its letters and digits in ASCII lowercase, runs of
/// anything else as one `-`, at most [`MAX_SLUG`] long, `note` when nothing
/// is left; then `-2`, `-3`… until it isn't one of `taken`. Pure.
pub fn slug_for(title: &str, taken: &dyn Fn(&str) -> bool) -> String {
    let mut base = String::new();
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            base.push(c);
        } else if !base.ends_with('-') && !base.is_empty() {
            base.push('-');
        }
    }
    let mut base: String = base.chars().take(MAX_SLUG - 4).collect();
    while base.ends_with('-') {
        base.pop();
    }
    if base.is_empty() {
        base = "note".into();
    }
    if !taken(&base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|slug| !taken(slug))
        .expect("some suffix is free")
}

/// The slugs of the `kanso-page:` links in `html`, as written. Pure.
pub fn page_links(html: &str) -> Vec<String> {
    let needle = format!("href=\"{PAGE_LINK_PREFIX}");
    html.match_indices(&needle)
        .filter_map(|(at, _)| {
            let rest = &html[at + needle.len()..];
            rest.find('"').map(|end| rest[..end].to_owned())
        })
        .collect()
}

/// The files the `src="kanso-file:<n>"` images in `html` use. Pure.
pub fn file_sources(html: &str) -> Vec<usize> {
    let needle = format!("src=\"{FILE_SRC_PREFIX}");
    html.match_indices(&needle)
        .filter_map(|(at, _)| {
            let rest = &html[at + needle.len()..];
            rest.find('"').and_then(|end| rest[..end].parse().ok())
        })
        .collect()
}

/// What a notebook's site was made from, so any device can tell when it
/// changed since: the notebook's name and each published note's ID, title
/// and text, in ID order, as one string to [`crate::share::content_hash`].
/// Pure.
pub fn fingerprint<'a>(notes: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>) -> String {
    let mut notes: Vec<(&str, &str, &str)> = notes.into_iter().collect();
    notes.sort_unstable();
    let mut hasher = Sha256::new();
    for (id, title, body) in notes {
        for part in [id, title, body] {
            hasher.update((part.len() as u64).to_be_bytes());
            hasher.update(part.as_bytes());
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::share::{ShareKey, open_share, seal_share};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use std::collections::HashSet;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn image(n: u8) -> SiteFile {
        SiteFile::new(&format!("i{n}.png"), "image/png", &[n; 5])
    }

    #[test]
    fn examples() {
        let none = |_: &str| false;
        assert_eq!(
            slug_for("Meeting notes: Q3 & Q4!", &none),
            "meeting-notes-q3-q4"
        );
        assert_eq!(slug_for("Café", &none), "caf");
        assert_eq!(slug_for("—", &none), "note");
        assert_eq!(
            slug_for("Plan", &|s| s == "plan" || s == "plan-2"),
            "plan-3"
        );
        assert!(is_slug("a-1") && !is_slug("") && !is_slug("A") && !is_slug("a/b"));

        let mut bundle = SiteBundle::new("Work");
        let png = image(1);
        bundle.index = r#"<a href="kanso-page:plan">Plan</a>"#.into();
        bundle.pages.insert(
            "plan".into(),
            SitePage {
                title: "Plan".into(),
                html: format!(
                    r#"<a href="kanso-page:">back</a><img src="{0}"><img src="{0}">"#,
                    png.data_uri()
                ),
                downloads: Vec::new(),
            },
        );
        assert_eq!(bundle.share_images(vec![png.clone(), png.clone()]), [0, 0]);
        assert_eq!(bundle.files.len(), 1);
        assert_eq!(file_sources(&bundle.pages["plan"].html), [0, 0]);
        assert!(bundle.check().is_ok());
        assert!(bundle.page_html("plan").unwrap().contains(&png.data_uri()));
        assert_eq!(bundle.page_html("gone"), None);
        // A link to a page that isn't there, or a file that isn't, fails.
        let mut broken = bundle.clone();
        broken.index.push_str(r#"<a href="kanso-page:secret">"#);
        assert_eq!(broken.check(), Err(SiteError::MissingPage("secret".into())));
        let mut broken = bundle.clone();
        broken.pages.get_mut("plan").unwrap().downloads.push(3);
        assert_eq!(broken.check(), Err(SiteError::MissingFile(3)));
        // A note's page is not a bundle, and neither is another version.
        assert_eq!(SiteBundle::from_bytes(b"<!doctype html><p>x</p>"), None);
        let mut later = bundle.clone();
        later.v = 2;
        assert_eq!(SiteBundle::from_bytes(&later.to_bytes()), None);
        assert_eq!(SiteBundle::from_bytes(&bundle.to_bytes()), Some(bundle));
        // The fingerprint ignores order and tells notes apart.
        let a = fingerprint([("note:a", "A", "x"), ("note:b", "B", "y")]);
        assert_eq!(a, fingerprint([("note:b", "B", "y"), ("note:a", "A", "x")]));
        assert_ne!(a, fingerprint([("note:a", "A", "xy"), ("note:b", "B", "")]));
    }

    fn gen_bundle() -> impl Strategy<Value = SiteBundle> {
        (
            "\\PC{0,20}",
            prop::collection::vec(("\\PC{0,16}", "\\PC{0,60}", 0u8..4), 0..6),
        )
            .prop_map(|(title, notes)| {
                let mut bundle = SiteBundle::new(&title);
                let mut slugs: Vec<String> = Vec::new();
                for (title, _, _) in &notes {
                    let taken: HashSet<String> = slugs.iter().cloned().collect();
                    slugs.push(slug_for(title, &|s| taken.contains(s)));
                }
                let mut images = Vec::new();
                for (ix, (title, text, img)) in notes.iter().enumerate() {
                    let next = &slugs[(ix + 1) % slugs.len()];
                    let file = image(*img);
                    bundle.pages.insert(
                        slugs[ix].clone(),
                        SitePage {
                            title: title.clone(),
                            html: format!(
                                r#"<p>{text}</p><a href="kanso-page:{next}">n</a><img src="{}">"#,
                                file.data_uri()
                            ),
                            downloads: Vec::new(),
                        },
                    );
                    images.push(file);
                }
                bundle.index = slugs
                    .iter()
                    .map(|s| format!(r#"<a href="kanso-page:{s}">x</a>"#))
                    .collect();
                bundle.share_images(images);
                bundle
            })
    }

    proptest! {
        #![proptest_config(config(128))]

        /// Round trip: a bundle sealed, opened and read back is the same
        /// bundle, and every link and image in it resolves.
        #[test]
        fn prop_bundles_seal_and_open(bundle in gen_bundle(), key in any::<[u8; 32]>(), id in any::<[u8; 16]>()) {
            prop_assert!(bundle.check().is_ok(), "{:?}", bundle.check());
            let key = ShareKey::from_bytes(key);
            let id = crate::crypto::b64(&id);
            let sealed = seal_share(&key, &id, &bundle.to_bytes()).unwrap();
            let opened = open_share(&key, &id, &sealed).unwrap();
            prop_assert_eq!(SiteBundle::from_bytes(&opened), Some(bundle.clone()));
            // Each image is in the bundle once, and every page shows them
            // back as data: URLs.
            let unique: HashSet<&SiteFile> = bundle.files.iter().collect();
            prop_assert_eq!(unique.len(), bundle.files.len());
            for slug in bundle.pages.keys() {
                let html = bundle.page_html(slug).unwrap();
                prop_assert!(!html.contains("kanso-file:"));
            }
        }

        /// Slugs are addresses, and never one already taken.
        #[test]
        fn prop_slugs_are_unique_addresses(titles in prop::collection::vec("\\PC{0,80}", 0..12)) {
            let mut taken: HashSet<String> = HashSet::new();
            for title in titles {
                let slug = slug_for(&title, &|s| taken.contains(s));
                prop_assert!(is_slug(&slug), "{}", slug);
                prop_assert!(taken.insert(slug));
            }
        }
    }
}
