//! Which addresses the updater may fetch from, or open in the browser.
//!
//! The feed's downloads and release page must be this repository's on
//! `https://github.com`. A download then redirects to GitHub's storage
//! hosts, and every hop of a redirect is checked here too. The signature
//! and the SHA-256 already pin the bytes; this keeps the updater from
//! talking to any other server at all.

use url::{Host, Url};

/// Hosts a GitHub release download redirects to.
const STORAGE_HOSTS: [&str; 2] = [
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("not a valid address")]
    Invalid,
    #[error("not HTTPS")]
    NotHttps,
    #[error("not on GitHub")]
    WrongHost,
    #[error("not this repository's releases")]
    WrongPath,
}

/// The rules for one run of the updater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UrlPolicy {
    /// `owner/name` on GitHub.
    pub repository: &'static str,
    /// Also allow `http://` on this machine's loopback address, for the
    /// end-to-end test and a debug build's `KANSO_UPDATE_FEED`. The feed
    /// must still be signed by a trusted key.
    pub allow_loopback: bool,
}

impl UrlPolicy {
    /// What shipped builds use.
    pub const RELEASE: UrlPolicy = UrlPolicy {
        repository: crate::REPOSITORY,
        allow_loopback: false,
    };

    /// A download the feed names: an asset of one of this repository's
    /// releases.
    pub fn check_asset(&self, url: &str) -> Result<Url, PolicyError> {
        self.check_github_path(url, "releases/download/")
    }

    /// The release page the feed names, which "Download" opens.
    pub fn check_release_page(&self, url: &str) -> Result<Url, PolicyError> {
        self.check_github_path(url, "releases/")
    }

    /// Any address fetched, including each hop of a redirect: GitHub or
    /// its release storage, over HTTPS.
    pub fn check_fetch(&self, url: &str) -> Result<Url, PolicyError> {
        let parsed = Url::parse(url).map_err(|_| PolicyError::Invalid)?;
        if self.is_loopback(&parsed) {
            return Ok(parsed);
        }
        let host = https_host(&parsed)?;
        if host == "github.com" || STORAGE_HOSTS.contains(&host.as_str()) {
            Ok(parsed)
        } else {
            Err(PolicyError::WrongHost)
        }
    }

    fn check_github_path(&self, url: &str, under: &str) -> Result<Url, PolicyError> {
        let parsed = Url::parse(url).map_err(|_| PolicyError::Invalid)?;
        if self.is_loopback(&parsed) {
            return Ok(parsed);
        }
        if https_host(&parsed)? != "github.com" {
            return Err(PolicyError::WrongHost);
        }
        let prefix = format!("/{}/{under}", self.repository);
        let path = parsed.path();
        // `..` never survives `Url::parse`, but an encoded one would.
        if !path.starts_with(&prefix)
            || path.len() == prefix.len()
            || path.contains("%2e")
            || path.contains("%2E")
        {
            return Err(PolicyError::WrongPath);
        }
        Ok(parsed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const P: UrlPolicy = UrlPolicy::RELEASE;

    #[test]
    fn release_downloads_and_their_redirects_pass() {
        let asset = "https://github.com/iamngoni/kanso/releases/download/v0.2.0/Kanso-0.2.0-macos-arm64.app.tar.gz";
        assert!(P.check_asset(asset).is_ok());
        assert!(P.check_fetch(asset).is_ok());
        assert!(P.check_fetch(crate::FEED_URL).is_ok());
        assert!(
            P.check_fetch(
                "https://objects.githubusercontent.com/github-production-release-asset/1/2?x=y"
            )
            .is_ok()
        );
        assert!(
            P.check_fetch(
                "https://release-assets.githubusercontent.com/github-production-release-asset/1/2"
            )
            .is_ok()
        );
        assert!(
            P.check_release_page("https://github.com/iamngoni/kanso/releases/tag/v0.2.0")
                .is_ok()
        );
    }

    #[test]
    fn everything_else_is_refused() {
        for (url, why) in [
            (
                "http://github.com/iamngoni/kanso/releases/download/v1/a",
                PolicyError::NotHttps,
            ),
            (
                "https://github.com.evil.example/iamngoni/kanso/releases/download/v1/a",
                PolicyError::WrongHost,
            ),
            (
                "https://evil.example/iamngoni/kanso/releases/download/v1/a",
                PolicyError::WrongHost,
            ),
            (
                "https://github.com:8443/iamngoni/kanso/releases/download/v1/a",
                PolicyError::WrongHost,
            ),
            (
                "https://user@github.com/iamngoni/kanso/releases/download/v1/a",
                PolicyError::Invalid,
            ),
            (
                "https://github.com/someone/else/releases/download/v1/a",
                PolicyError::WrongPath,
            ),
            (
                "https://github.com/iamngoni/kanso/archive/main.zip",
                PolicyError::WrongPath,
            ),
            (
                "https://github.com/iamngoni/kanso/releases/download/",
                PolicyError::WrongPath,
            ),
            (
                "https://github.com/iamngoni/kanso/releases/download/%2e%2e/x",
                PolicyError::WrongPath,
            ),
            (
                "https://objects.githubusercontent.com/x",
                PolicyError::WrongHost,
            ),
            ("http://127.0.0.1:8080/a", PolicyError::NotHttps),
            ("file:///etc/passwd", PolicyError::NotHttps),
            ("not a url", PolicyError::Invalid),
        ] {
            assert_eq!(P.check_asset(url).err(), Some(why.clone()), "{url}");
        }
    }

    #[test]
    fn loopback_only_when_allowed() {
        let local = UrlPolicy {
            allow_loopback: true,
            ..P
        };
        assert!(local.check_asset("http://127.0.0.1:4000/x").is_ok());
        assert!(local.check_fetch("http://[::1]:4000/x").is_ok());
        assert!(local.check_fetch("http://localhost:4000/x").is_ok());
        assert!(local.check_fetch("http://192.168.1.2:4000/x").is_err());
        assert!(local.check_fetch("http://example.com/x").is_err());
        assert!(P.check_fetch("http://127.0.0.1:4000/x").is_err());
    }

    proptest! {
        /// No host but GitHub's is ever fetched, whatever the address.
        #[test]
        fn prop_only_github_hosts_are_fetched(
            scheme in prop::sample::select(vec!["https", "http", "ftp", "file"]),
            host in "[a-z]{1,10}(\\.[a-z]{2,8}){0,3}",
            path in "(/[a-zA-Z0-9._-]{0,12}){0,4}",
        ) {
            let url = format!("{scheme}://{host}{path}");
            if let Ok(parsed) = P.check_fetch(&url) {
                prop_assert_eq!(parsed.scheme(), "https");
                let host = parsed.host_str().unwrap();
                prop_assert!(host == "github.com" || STORAGE_HOSTS.contains(&host));
            }
        }

        /// A download is under this repository's release downloads, on
        /// github.com over HTTPS, or it is refused.
        #[test]
        fn prop_assets_stay_in_the_repository(
            owner in prop::sample::select(vec!["iamngoni", "someone", "iamngonii"]),
            repo in prop::sample::select(vec!["kanso", "kanso-evil", "other"]),
            kind in prop::sample::select(vec!["releases/download", "releases/tag", "raw", "archive"]),
            rest in "[a-zA-Z0-9][a-zA-Z0-9._-]{0,19}",
            host in prop::sample::select(vec!["github.com", "gist.github.com", "objects.githubusercontent.com"]),
        ) {
            let url = format!("https://{host}/{owner}/{repo}/{kind}/{rest}");
            let expect = host == "github.com" && owner == "iamngoni" && repo == "kanso" && kind == "releases/download";
            prop_assert_eq!(P.check_asset(&url).is_ok(), expect, "{}", url);
        }
    }
}
