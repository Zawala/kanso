//! Which addresses the gallery may fetch from, or open in the browser.
//!
//! Every file comes from `raw.githubusercontent.com/iamngoni/kanso-themes/`,
//! over HTTPS, with no credentials and no explicit port. The app builds each
//! URL itself from a validated relative path, so a signed index can never
//! point it at another server. Redirects are followed by
//! [`kanso_update::fetch::open`], which checks every hop with this policy
//! through [`kanso_update::fetch::FetchPolicy`].
//!
//! A debug build may point the index at `KANSO_GALLERY_INDEX=<url>` with
//! loopback allowed, so the end-to-end test can serve a signed catalog. The
//! index must still verify against a trusted key in every build.

use url::{Host, Url};

pub use kanso_update::PolicyError;

use crate::{INDEX_URL, REPOSITORY};

/// The gallery's rules are the updater's fetch-policy shape, so every
/// redirect hop is checked with [`GalleryPolicy`] rather than the updater's
/// narrower release-host list.
impl kanso_update::fetch::FetchPolicy for GalleryPolicy {
    fn check_fetch(&self, url: &str) -> Result<Url, PolicyError> {
        GalleryPolicy::check_fetch(self, url)
    }
}

/// The rules for one run of the gallery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GalleryPolicy {
    /// The fixed repository, as `owner/name`.
    pub repository: &'static str,
    /// Also allow `http://` on this machine's loopback address, for the
    /// end-to-end test and a debug build's override.
    pub allow_loopback: bool,
}

impl GalleryPolicy {
    /// What shipped builds use.
    pub const GALLERY: GalleryPolicy = GalleryPolicy {
        repository: REPOSITORY,
        allow_loopback: false,
    };

    /// Any file the index names, fetched: `raw.githubusercontent.com` under
    /// this repository, over HTTPS.
    pub fn check_fetch(&self, url: &str) -> Result<Url, PolicyError> {
        reject_encoded_separator(url)?;
        let parsed = Url::parse(url).map_err(|_| PolicyError::Invalid)?;
        if self.is_loopback(&parsed) {
            return Ok(parsed);
        }
        let host = https_host(&parsed)?;
        if host != "raw.githubusercontent.com" {
            return Err(PolicyError::WrongHost);
        }
        self.check_repository_path(&parsed)
    }

    /// The index override a debug build may name: any address the updater's
    /// policy accepts, or loopback, over HTTPS only.
    pub fn check_index(&self, url: &str) -> Result<Url, PolicyError> {
        reject_encoded_separator(url)?;
        let parsed = Url::parse(url).map_err(|_| PolicyError::Invalid)?;
        if self.is_loopback(&parsed) {
            return Ok(parsed);
        }
        if https_host(&parsed)? != "raw.githubusercontent.com" {
            return Err(PolicyError::WrongHost);
        }
        self.check_repository_path(&parsed)
    }

    /// A theme's `source`, opened in the browser and never fetched: HTTPS,
    /// with a host and no credentials.
    pub fn check_source(&self, url: &str) -> Result<Url, PolicyError> {
        // `url` may leave the host empty (`https:///x`), which `Url` then
        // reads as a path; a source is a website with an authority.
        let after = url.strip_prefix("https://").ok_or(PolicyError::NotHttps)?;
        if after.is_empty() || after.starts_with('/') {
            return Err(PolicyError::Invalid);
        }
        let parsed = Url::parse(url).map_err(|_| PolicyError::Invalid)?;
        if parsed.scheme() != "https" {
            return Err(PolicyError::NotHttps);
        }
        if parsed.host().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(PolicyError::Invalid);
        }
        Ok(parsed)
    }

    fn check_repository_path(&self, url: &Url) -> Result<Url, PolicyError> {
        let prefix = format!("/{}/", self.repository);
        let path = url.path();
        if !path.starts_with(&prefix)
            || path.len() == prefix.len()
            || path.contains("%2e")
            || path.contains("%2E")
            || path.contains("%2f")
            || path.contains("%2F")
        {
            return Err(PolicyError::WrongPath);
        }
        Ok(url.clone())
    }

    fn is_loopback(&self, url: &Url) -> bool {
        self.allow_loopback
            && url.scheme() == "http"
            && url.username().is_empty()
            && url.password().is_none()
            && match url.host() {
                Some(Host::Ipv4(ip)) => ip.is_loopback(),
                Some(Host::Ipv6(ip)) => ip.is_loopback(),
                Some(Host::Domain(name)) => name == "localhost",
                None => false,
            }
    }
}

/// An encoded dot-segment or separator never reaches the parser, which
/// would otherwise resolve it into an innocent-looking path.
fn reject_encoded_separator(url: &str) -> Result<(), PolicyError> {
    let low = url.to_ascii_lowercase();
    if low.contains("%2e") || low.contains("%2f") || low.contains("%5c") {
        return Err(PolicyError::WrongPath);
    }
    Ok(())
}

/// The host of an `https` URL on the default port with no credentials.
fn https_host(url: &Url) -> Result<String, PolicyError> {
    if url.scheme() != "https" {
        return Err(PolicyError::NotHttps);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(PolicyError::Invalid);
    }
    if url.port().is_some() {
        return Err(PolicyError::WrongHost);
    }
    match url.host() {
        Some(Host::Domain(name)) => Ok(name.to_ascii_lowercase()),
        _ => Err(PolicyError::WrongHost),
    }
}

/// The index's URL for this build: the fixed one, or a debug build's
/// `KANSO_GALLERY_INDEX`. Pure.
pub fn index_url() -> String {
    if cfg!(debug_assertions)
        && let Ok(url) = std::env::var("KANSO_GALLERY_INDEX")
        && !url.is_empty()
    {
        return url;
    }
    INDEX_URL.to_owned()
}

/// The policy this build uses, with loopback allowed only for a debug
/// build's override. Pure.
pub fn policy() -> GalleryPolicy {
    let overridden = cfg!(debug_assertions)
        && std::env::var("KANSO_GALLERY_INDEX").is_ok_and(|url| !url.is_empty());
    GalleryPolicy {
        allow_loopback: overridden,
        ..GalleryPolicy::GALLERY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const P: GalleryPolicy = GalleryPolicy::GALLERY;

    #[test]
    fn the_catalog_and_its_files_pass() {
        for url in [
            crate::INDEX_URL,
            crate::SIGNATURE_URL,
            "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/nord.json",
            "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/icon-themes/catppuccin-icons.tar.gz",
            "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/icon-themes/lucide/chevron.svg",
        ] {
            assert!(P.check_fetch(url).is_ok(), "{url}");
        }
    }

    #[test]
    fn everything_else_is_refused() {
        for (url, why) in [
            (
                "http://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/index.json",
                PolicyError::NotHttps,
            ),
            (
                "https://evil.example/iamngoni/kanso-themes/main/gallery/index.json",
                PolicyError::WrongHost,
            ),
            (
                "https://raw.githubusercontent.com/someone/kanso-themes/main/gallery/index.json",
                PolicyError::WrongPath,
            ),
            (
                "https://raw.githubusercontent.com:8443/iamngoni/kanso-themes/main/gallery/index.json",
                PolicyError::WrongHost,
            ),
            (
                "https://user@raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/index.json",
                PolicyError::Invalid,
            ),
            (
                "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/%2e%2e/x",
                PolicyError::WrongPath,
            ),
            ("file:///etc/passwd", PolicyError::NotHttps),
            ("not a url", PolicyError::Invalid),
        ] {
            assert_eq!(P.check_fetch(url).err(), Some(why.clone()), "{url}");
        }
        assert!(P.check_source("http://example.com").is_err());
        assert!(P.check_source("https://example.com/a").is_ok());
    }

    #[test]
    fn a_release_build_uses_the_gallery_policy_on_every_hop() {
        // The updater's policy is deliberately not widened: it refuses the
        // gallery's host, which is exactly why the gallery cannot borrow it.
        let raw = crate::INDEX_URL;
        assert!(P.check_fetch(raw).is_ok());
        assert_eq!(
            kanso_update::UrlPolicy::RELEASE.check_fetch(raw).err(),
            Some(PolicyError::WrongHost)
        );

        // A second allowed hop is followed, and the third, off-repository
        // address is refused after the transport is never asked for it.
        let next = "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/next.json";
        let evil = "https://evil.example/iamngoni/kanso-themes/main/next.json";
        let transport = HopTransport::new(vec![
            (raw, 302, Some(next)),
            (next, 302, Some(evil)),
            (evil, 200, None),
        ]);
        let err = kanso_update::fetch::get_bytes(&transport, &P, raw, crate::MAX_INDEX_BYTES)
            .unwrap_err();
        assert!(matches!(
            err,
            kanso_update::FetchError::Policy(PolicyError::WrongHost)
        ));
        assert_eq!(transport.asked(), vec![raw.to_owned(), next.to_owned()]);
    }

    #[test]
    fn a_third_hop_still_works_and_a_bad_path_is_refused() {
        let a = crate::INDEX_URL;
        let b = "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/a.json";
        let c = "https://raw.githubusercontent.com/iamngoni/kanso-themes/main/gallery/b.json";
        let t = HopTransport::new(vec![(a, 302, Some(b)), (b, 302, Some(c)), (c, 200, None)]);
        let mut body = Vec::new();
        use std::io::Read as _;
        kanso_update::fetch::open(&t, &P, a)
            .unwrap()
            .read_to_end(&mut body)
            .unwrap();
        assert_eq!(t.asked().len(), 3);
        assert_eq!(
            P.check_fetch("https://raw.githubusercontent.com/other/kanso-themes/main/x")
                .err(),
            Some(PolicyError::WrongPath)
        );
    }

    struct HopTransport {
        answers: Vec<(&'static str, u16, Option<&'static str>)>,
        asked: std::sync::Mutex<Vec<String>>,
    }

    impl HopTransport {
        fn new(answers: Vec<(&'static str, u16, Option<&'static str>)>) -> HopTransport {
            HopTransport {
                answers,
                asked: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }
    }

    impl kanso_update::Transport for HopTransport {
        fn get(
            &self,
            url: &str,
        ) -> Result<kanso_update::fetch::Response, kanso_update::FetchError> {
            self.asked.lock().unwrap().push(url.to_owned());
            let (_, status, location) = self
                .answers
                .iter()
                .find(|(known, _, _)| *known == url)
                .ok_or(kanso_update::FetchError::NotFound)?;
            Ok(kanso_update::fetch::Response {
                status: *status,
                location: location.map(str::to_owned),
                rate_limit_remaining: None,
                etag: None,
                body: Box::new(std::io::empty()),
            })
        }
    }

    #[test]
    fn loopback_only_when_allowed() {
        let local = GalleryPolicy {
            allow_loopback: true,
            ..P
        };
        assert!(local.check_fetch("http://127.0.0.1:4000/x").is_ok());
        assert!(local.check_fetch("http://[::1]:4000/x").is_ok());
        assert!(local.check_fetch("http://localhost:4000/x").is_ok());
        assert!(local.check_fetch("http://192.168.1.2:4000/x").is_err());
        assert!(P.check_fetch("http://127.0.0.1:4000/x").is_err());
    }

    proptest! {
        /// No host but the repository's raw host is ever fetched, and no
        /// path climbs out of it.
        #[test]
        fn prop_only_the_repository_is_fetched(
            scheme in prop::sample::select(vec!["https", "http", "ftp", "file"]),
            host in prop::sample::select(vec![
                "raw.githubusercontent.com",
                "github.com",
                "gist.github.com",
                "raw.githubusercontent.com.evil.example",
            ]),
            owner in prop::sample::select(vec!["iamngoni", "someone", "iamngonii"]),
            repo in prop::sample::select(vec!["kanso-themes", "kanso-themes-evil", "kanso"]),
            rest in "[a-zA-Z0-9._/-]{0,40}",
        ) {
            let url = format!("{scheme}://{host}/{owner}/{repo}/{rest}");
            let expect = scheme == "https"
                && host == "raw.githubusercontent.com"
                && owner == "iamngoni"
                && repo == "kanso-themes"
                && !rest.is_empty();
            prop_assert_eq!(P.check_fetch(&url).is_ok(), expect, "{}", url);
        }

        /// A source link is opened only over HTTPS with no credentials.
        #[test]
        fn prop_sources_are_https(
            scheme in prop::sample::select(vec!["https", "http", "git", "file"]),
            host in prop::sample::select(vec!["example.com", "github.com", ""]),
        ) {
            let url = format!("{scheme}://{host}/theme");
            let expect = scheme == "https" && !host.is_empty();
            prop_assert_eq!(P.check_source(&url).is_ok(), expect, "{}", url);
        }
    }
}
