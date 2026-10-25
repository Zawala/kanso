//! Which certificates Kanso trusts when it reaches a server.
//!
//! On a computer that is the web's roots (Mozilla's list, built in). On iOS
//! and Android it is the phone's own store, through the system: a root the
//! person's company or school installed works, and a revoked certificate
//! stops. [`kanso-sync`](https://docs.rs/kanso-sync) (HTTP and the live
//! socket), `kanso-collab` and `kanso-ai` all take their choice from here, so
//! it is made once.
//!
//! The Android verifier reaches the system through the app's JVM, which
//! only the app can hand over ([`init_android`]). Before that, asking it to
//! check a certificate panics, so every request over TLS calls [`ready`] (or [`ready_for`]) first and
//! turns "not yet" into an error.

use std::sync::Arc;

#[cfg(target_os = "android")]
use std::sync::atomic::{AtomicBool, Ordering};

/// Why the trust choice can't be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TlsError {
    /// The Android app hasn't called [`init_android`] yet.
    #[error(
        "the phone's certificate store isn't set up yet (the app must call init_android first)"
    )]
    NotReady,
    /// The system's verifier or the TLS setup refused to start.
    #[error("TLS setup failed: {0}")]
    Setup(String),
}

/// Whether servers' certificates are checked by the phone (iOS, Android)
/// rather than against the web's roots built into the program.
pub const PLATFORM_VERIFIER: bool = cfg!(any(target_os = "ios", target_os = "android"));

/// Set once the Android app has handed over its `Context`.
#[cfg(target_os = "android")]
static ANDROID_READY: AtomicBool = AtomicBool::new(false);

/// Hands the Android verifier the app's `Context`. Call it once, on a thread
/// attached to the JVM, before any request; later calls do nothing.
#[cfg(target_os = "android")]
pub fn init_android(env: &mut jni::Env, context: jni::objects::JObject) -> Result<(), TlsError> {
    rustls_platform_verifier::android::init_with_env(env, context)
        .map_err(|err| TlsError::Setup(err.to_string()))?;
    ANDROID_READY.store(true, Ordering::Release);
    Ok(())
}

/// Ok when certificates can be checked. Always on a computer and on iOS; on
/// Android, once [`init_android`] has run. Cheap: call it before every
/// request over TLS.
pub fn ready() -> Result<(), TlsError> {
    #[cfg(target_os = "android")]
    if !ANDROID_READY.load(Ordering::Acquire) {
        return Err(TlsError::NotReady);
    }
    Ok(())
}

/// [`ready`] for a request to `url`: only an `https` address needs the
/// phone's certificate store. A plain `http` one (a server on this network
/// or the tests' own) checks no certificate.
pub fn ready_for(url: &str) -> Result<(), TlsError> {
    if url
        .get(..6)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https:"))
    {
        ready()
    } else {
        Ok(())
    }
}

/// A rustls client configuration that trusts what Kanso trusts, for the
/// live socket and for HTTP clients that take a rustls configuration.
pub fn client_config() -> Result<Arc<rustls::ClientConfig>, TlsError> {
    ready()?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|err| TlsError::Setup(err.to_string()))?;
    #[cfg(any(target_os = "ios", target_os = "android"))]
    let builder = builder
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(
            rustls_platform_verifier::Verifier::new(provider)
                .map_err(|err| TlsError::Setup(err.to_string()))?,
        ));
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    let builder = {
        let _ = provider;
        builder.with_root_certificates(rustls::RootCertStore::from_iter(
            webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
        ))
    };
    Ok(Arc::new(builder.with_no_client_auth()))
}

/// The same choice for `ureq`. It doesn't check [`ready`]: the agent is
/// built before the Android app may have set up, so the request does.
#[cfg(feature = "ureq")]
pub fn ureq_tls() -> ureq::tls::TlsConfig {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        ureq::tls::TlsConfig::builder()
            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
            .build()
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        ureq::tls::TlsConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // no-prop-test: the answer depends on a platform and, on Android, on one
    // call that the app makes.
    #[test]
    fn a_computer_and_iphone_are_ready_but_an_android_app_waits_for_its_context() {
        let ready = ready();
        if cfg!(target_os = "android") {
            assert_eq!(ready, Err(TlsError::NotReady));
        } else {
            assert_eq!(ready, Ok(()));
        }
    }

    // no-prop-test: one fixed configuration, compared with the platform.
    #[test]
    fn a_configuration_is_built_when_ready_and_refused_when_not() {
        match client_config() {
            Ok(config) => {
                assert!(ready().is_ok());
                assert!(config.alpn_protocols.is_empty());
            }
            Err(err) => {
                assert_eq!(err, TlsError::NotReady);
                assert!(ready().is_err());
            }
        }
    }

    proptest::proptest! {
        /// A plain `http` address never waits for the certificate store, on
        /// any platform; an `https` one waits exactly as `ready` does.
        #[test]
        fn prop_only_https_addresses_need_the_store(rest in "[a-z0-9./:?=&-]{0,40}") {
            proptest::prop_assert_eq!(ready_for(&format!("http://{rest}")), Ok(()));
            proptest::prop_assert_eq!(ready_for(&format!("HTTPS://{rest}")), ready());
            proptest::prop_assert_eq!(ready_for(&format!("https://{rest}")), ready());
        }
    }

    // no-prop-test: nothing but a constant.
    #[test]
    fn only_the_phones_use_the_platform_verifier() {
        assert_eq!(
            PLATFORM_VERIFIER,
            cfg!(any(target_os = "ios", target_os = "android"))
        );
    }

    #[cfg(feature = "ureq")]
    #[test]
    fn the_http_choice_matches_the_socket_choice() {
        let config = ureq_tls();
        let platform = matches!(config.root_certs(), ureq::tls::RootCerts::PlatformVerifier);
        assert_eq!(platform, PLATFORM_VERIFIER);
    }
}

/// What the iPhone's verifier does with bytes that are not a certificate
/// someone trusts: refuse them, and never panic.
#[cfg(all(test, target_os = "ios"))]
mod ios {
    use super::*;
    use proptest::prelude::*;
    use rustls::client::danger::ServerCertVerifier as _;
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};

    fn verifier() -> rustls_platform_verifier::Verifier {
        rustls_platform_verifier::Verifier::new(Arc::new(rustls::crypto::ring::default_provider()))
            .unwrap()
    }

    fn check(der: &[u8]) -> Result<(), rustls::Error> {
        verifier()
            .verify_server_cert(
                &CertificateDer::from(der.to_vec()),
                &[],
                &ServerName::try_from("example.com").unwrap(),
                &[],
                UnixTime::now(),
            )
            .map(|_| ())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::Off
            )),
            ..ProptestConfig::default()
        })]

        /// Whatever bytes a server sends as its certificate, they are not
        /// trusted.
        #[test]
        fn prop_bytes_that_are_no_trusted_certificate_are_refused(
            der in proptest::collection::vec(any::<u8>(), 0..600)
        ) {
            prop_assert!(check(&der).is_err());
        }
    }

    // no-prop-test: a fixed, well-formed but self-signed certificate.
    #[test]
    fn a_self_signed_certificate_is_refused() {
        assert!(check(SELF_SIGNED).is_err());
    }

    /// A certificate nobody trusts: self-signed for example.com.
    const SELF_SIGNED: &[u8] = include_bytes!("../tests/self-signed.der");
}
