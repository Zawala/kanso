//! HTTP Basic authentication. Every route requires it, as in Inkdrop.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::{DecodePaddingMode, general_purpose};
use subtle::ConstantTimeEq;

use crate::error::ApiError;

/// Basic tokens with or without `=` padding.
const BASIC: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub(crate) struct Credentials {
    /// `username:password`, the decoded form of a valid Basic token.
    expected: Vec<u8>,
}

impl Credentials {
    pub fn new(username: &str, password: &str) -> Self {
        Credentials {
            expected: format!("{username}:{password}").into_bytes(),
        }
    }

    /// Whether an `Authorization` header value carries exactly these
    /// credentials. The comparison takes the same time for every token of a
    /// given length.
    pub fn accepts(&self, header: Option<&[u8]>) -> bool {
        let Some(given) = header.and_then(decode_basic) else {
            return false;
        };
        bool::from(given.ct_eq(&self.expected))
    }
}

fn decode_basic(header: &[u8]) -> Option<Vec<u8>> {
    let header = std::str::from_utf8(header).ok()?.trim();
    let (scheme, token) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    BASIC.decode(token.trim()).ok()
}

/// The `Authorization` header value for `username` and `password`.
pub fn basic_header(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        general_purpose::STANDARD.encode(format!("{username}:{password}"))
    )
}

pub(crate) async fn require_auth(
    State(credentials): State<Arc<Credentials>>,
    request: Request,
    next: Next,
) -> Response {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .map(|value| value.as_bytes());
    if credentials.accepts(header) {
        next.run(request).await
    } else {
        ApiError::unauthorized().into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_exact_credentials() {
        let creds = Credentials::new("user", "p:ss");
        let ok = basic_header("user", "p:ss");
        assert!(creds.accepts(Some(ok.as_bytes())));
        let unpadded = ok.trim_end_matches('=').to_owned();
        assert!(creds.accepts(Some(unpadded.as_bytes())));
        assert!(creds.accepts(Some(ok.replacen("Basic", "basic", 1).as_bytes())));
        for bad in [
            basic_header("user", "p:s"),
            basic_header("User", "p:ss"),
            basic_header("user", "p:ss "),
            "Bearer abc".to_owned(),
            "Basic !!!".to_owned(),
            String::new(),
        ] {
            assert!(!creds.accepts(Some(bad.as_bytes())), "{bad}");
        }
        assert!(!creds.accepts(None));
    }
}
