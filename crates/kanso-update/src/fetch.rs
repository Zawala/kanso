//! Fetching over HTTPS, one hop at a time: redirects are followed here so
//! that each address is checked against the [`UrlPolicy`] before it is
//! asked for, and every body is read up to a cap.

use std::io::Read;
use std::time::Duration;

use crate::policy::{PolicyError, UrlPolicy};

/// How many redirects one fetch follows. GitHub's release downloads take
/// two: `latest/download` to the tag's download, then to storage.
const MAX_REDIRECTS: usize = 5;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("offline or the server couldn't be reached")]
    Offline,
    #[error("GitHub is limiting requests; Kanso tries again later")]
    RateLimited,
    #[error("not found")]
    NotFound,
    #[error("the server answered {0}")]
    Status(u16),
    #[error("too many redirects")]
    TooManyRedirects,
    #[error("an address isn't allowed: {0}")]
    Policy(PolicyError),
    #[error("the answer is larger than expected")]
    TooLarge,
    #[error("the connection failed: {0}")]
    Io(#[from] std::io::Error),
}

/// One answer, before redirects are followed.
pub struct Response {
    pub status: u16,
    /// The `Location` of a redirect.
    pub location: Option<String>,
    /// GitHub's `x-ratelimit-remaining`, when sent.
    pub rate_limit_remaining: Option<u64>,
    /// The answer's `ETag`, when sent, for a later conditional request.
    pub etag: Option<String>,
    pub body: Box<dyn Read + Send>,
}

/// One HTTP GET without following redirects: [`HttpTransport`] in the app,
/// fakes in tests.
pub trait Transport: Send + Sync {
    fn get(&self, url: &str) -> Result<Response, FetchError>;

    /// [`Transport::get`] with `If-None-Match`, so an unchanged answer costs
    /// a `304` with no body. A transport that can't send headers answers as
    /// [`Transport::get`] does.
    fn get_if_none_match(&self, url: &str, _etag: Option<&str>) -> Result<Response, FetchError> {
        self.get(url)
    }
}

/// The app's transport: blocking `ureq` with rustls, no redirects of its
/// own, and timeouts that suit a download in the background.
pub struct HttpTransport {
    agent: ureq::Agent,
}

impl HttpTransport {
    pub fn new(user_agent: &str) -> HttpTransport {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(Duration::from_secs(20)))
            // The body is read as it arrives; a slow line still finishes.
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .user_agent(user_agent)
            .build()
            .into();
        HttpTransport { agent }
    }
}

impl Transport for HttpTransport {
    fn get(&self, url: &str) -> Result<Response, FetchError> {
        let response = self.agent.get(url).call().map_err(transport_error)?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let location = header("location");
        let rate_limit_remaining = header("x-ratelimit-remaining").and_then(|v| v.parse().ok());
        let etag = header("etag");
        Ok(Response {
            status,
            location,
            rate_limit_remaining,
            etag,
            body: Box::new(response.into_body().into_reader()),
        })
    }

    fn get_if_none_match(&self, url: &str, etag: Option<&str>) -> Result<Response, FetchError> {
        let Some(etag) = etag else {
            return self.get(url);
        };
        let mut request = self.agent.get(url);
        request = request.header("If-None-Match", etag);
        let response = request.call().map_err(transport_error)?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let location = header("location");
        let rate_limit_remaining = header("x-ratelimit-remaining").and_then(|v| v.parse().ok());
        let etag = header("etag");
        Ok(Response {
            status,
            location,
            rate_limit_remaining,
            etag,
            body: Box::new(response.into_body().into_reader()),
        })
    }
}

fn transport_error(err: ureq::Error) -> FetchError {
    match err {
        ureq::Error::Io(io) if is_offline(&io) => FetchError::Offline,
        ureq::Error::Io(io) => FetchError::Io(io),
        ureq::Error::BadUri(_) => FetchError::Policy(PolicyError::Invalid),
        // Unreachable hosts, refused connections, timeouts and TLS
        // failures all read the same to a person: try again later.
        _ => FetchError::Offline,
    }
}

/// Whether an I/O error means the network or the server isn't there.
fn is_offline(err: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(
        err.kind(),
        ConnectionRefused
            | ConnectionReset
            | ConnectionAborted
            | NotConnected
            | TimedOut
            | HostUnreachable
            | NetworkUnreachable
            | NetworkDown
            | AddrNotAvailable
    )
}

/// GETs `url`, following up to [`MAX_REDIRECTS`] redirects, each checked
/// against `policy`, and returns the final answer's body.
pub fn open(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    url: &str,
) -> Result<Box<dyn Read + Send>, FetchError> {
    let mut current = policy.check_fetch(url).map_err(FetchError::Policy)?;
    for _ in 0..=MAX_REDIRECTS {
        let response = transport.get(current.as_str())?;
        match response.status {
            200 => return Ok(response.body),
            301 | 302 | 303 | 307 | 308 => {
                let location = response
                    .location
                    .ok_or(FetchError::Status(response.status))?;
                // Relative locations resolve against the address asked.
                let next = current
                    .join(&location)
                    .map_err(|_| FetchError::Policy(PolicyError::Invalid))?;
                current = policy
                    .check_fetch(next.as_str())
                    .map_err(FetchError::Policy)?;
            }
            404 => return Err(FetchError::NotFound),
            429 => return Err(FetchError::RateLimited),
            403 if response.rate_limit_remaining == Some(0) => return Err(FetchError::RateLimited),
            status => return Err(FetchError::Status(status)),
        }
    }
    Err(FetchError::TooManyRedirects)
}

/// GETs `url` (as [`open`]) and reads the whole body, refusing more than
/// `cap` bytes.
pub fn get_bytes(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    url: &str,
    cap: u64,
) -> Result<Vec<u8>, FetchError> {
    let body = open(transport, policy, url)?;
    let mut bytes = Vec::new();
    body.take(cap + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > cap {
        return Err(FetchError::TooLarge);
    }
    Ok(bytes)
}

/// The answer to a conditional GET: the bytes and their `ETag`, or nothing
/// new because the cached copy is still current.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetched {
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
    NotModified,
}

/// [`get_bytes`] with `If-None-Match`. A `304` (or `200` with the same body)
/// leaves the cached copy in place; a redirect is followed as [`open`] does,
/// but the header is only sent on the first request.
pub fn get_bytes_if_none_match(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    url: &str,
    cap: u64,
    etag: Option<&str>,
) -> Result<Fetched, FetchError> {
    let mut current = policy.check_fetch(url).map_err(FetchError::Policy)?;
    let mut conditional = etag;
    for _ in 0..=MAX_REDIRECTS {
        let response = transport.get_if_none_match(current.as_str(), conditional)?;
        conditional = None;
        match response.status {
            304 => return Ok(Fetched::NotModified),
            200 => {
                let new_etag = response.etag;
                let mut bytes = Vec::new();
                response.body.take(cap + 1).read_to_end(&mut bytes)?;
                if bytes.len() as u64 > cap {
                    return Err(FetchError::TooLarge);
                }
                return Ok(Fetched::Body {
                    bytes,
                    etag: new_etag,
                });
            }
            301 | 302 | 303 | 307 | 308 => {
                let location = response
                    .location
                    .ok_or(FetchError::Status(response.status))?;
                let next = current
                    .join(&location)
                    .map_err(|_| FetchError::Policy(PolicyError::Invalid))?;
                current = policy
                    .check_fetch(next.as_str())
                    .map_err(FetchError::Policy)?;
            }
            404 => return Err(FetchError::NotFound),
            429 => return Err(FetchError::RateLimited),
            403 if response.rate_limit_remaining == Some(0) => return Err(FetchError::RateLimited),
            status => return Err(FetchError::Status(status)),
        }
    }
    Err(FetchError::TooManyRedirects)
}

#[cfg(test)]
pub(crate) mod fake {
    //! A transport that answers from a table, for tests.

    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;

    #[derive(Clone)]
    pub enum Answer {
        Body(Vec<u8>),
        Redirect(String),
        Status(u16, Option<u64>),
        Offline,
    }

    #[derive(Default)]
    pub struct FakeTransport {
        pub answers: HashMap<String, Answer>,
        pub asked: Mutex<Vec<String>>,
    }

    impl FakeTransport {
        pub fn with(mut self, url: &str, answer: Answer) -> Self {
            self.answers.insert(url.to_owned(), answer);
            self
        }
    }

    impl Transport for FakeTransport {
        fn get(&self, url: &str) -> Result<Response, FetchError> {
            self.asked.lock().unwrap().push(url.to_owned());
            let empty = || Box::new(std::io::empty()) as Box<dyn Read + Send>;
            match self.answers.get(url).cloned() {
                Some(Answer::Body(bytes)) => Ok(Response {
                    status: 200,
                    location: None,
                    rate_limit_remaining: None,
                    etag: None,
                    body: Box::new(std::io::Cursor::new(bytes)),
                }),
                Some(Answer::Redirect(to)) => Ok(Response {
                    status: 302,
                    location: Some(to),
                    rate_limit_remaining: None,
                    etag: None,
                    body: empty(),
                }),
                Some(Answer::Status(status, remaining)) => Ok(Response {
                    status,
                    location: None,
                    rate_limit_remaining: remaining,
                    etag: None,
                    body: empty(),
                }),
                Some(Answer::Offline) => Err(FetchError::Offline),
                None => Ok(Response {
                    status: 404,
                    location: None,
                    rate_limit_remaining: None,
                    etag: None,
                    body: empty(),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::{Answer, FakeTransport};
    use super::*;

    const FEED: &str = crate::FEED_URL;
    const TAG: &str = "https://github.com/iamngoni/kanso/releases/download/v0.2.0/latest.json";
    const STORE: &str = "https://release-assets.githubusercontent.com/x/latest.json?sig=1";

    #[test]
    fn follows_github_redirects() {
        let t = FakeTransport::default()
            .with(FEED, Answer::Redirect(TAG.into()))
            .with(TAG, Answer::Redirect(STORE.into()))
            .with(STORE, Answer::Body(b"{}".to_vec()));
        assert_eq!(get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10).unwrap(), b"{}");
        assert_eq!(t.asked.lock().unwrap().len(), 3);
    }

    #[test]
    fn a_redirect_off_github_is_never_asked() {
        let t = FakeTransport::default()
            .with(
                FEED,
                Answer::Redirect("https://evil.example/latest.json".into()),
            )
            .with(
                "https://evil.example/latest.json",
                Answer::Body(b"{}".to_vec()),
            );
        assert!(matches!(
            get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10),
            Err(FetchError::Policy(PolicyError::WrongHost))
        ));
        assert_eq!(t.asked.lock().unwrap().as_slice(), [FEED]);
        // A downgrade to plain HTTP is refused the same way.
        let t = FakeTransport::default().with(FEED, Answer::Redirect("http://github.com/x".into()));
        assert!(matches!(
            get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10),
            Err(FetchError::Policy(PolicyError::NotHttps))
        ));
    }

    #[test]
    fn failures_are_told_apart() {
        type Expect = fn(&FetchError) -> bool;
        let cases: [(Answer, Expect); 5] = [
            (Answer::Offline, |e| matches!(e, FetchError::Offline)),
            (Answer::Status(429, None), |e| {
                matches!(e, FetchError::RateLimited)
            }),
            (Answer::Status(403, Some(0)), |e| {
                matches!(e, FetchError::RateLimited)
            }),
            (Answer::Status(500, None), |e| {
                matches!(e, FetchError::Status(500))
            }),
            (Answer::Status(404, None), |e| {
                matches!(e, FetchError::NotFound)
            }),
        ];
        for (answer, expected) in cases {
            let t = FakeTransport::default().with(FEED, answer);
            let err = get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10).unwrap_err();
            assert!(expected(&err), "{err:?}");
        }
    }

    #[test]
    fn loops_and_big_answers_stop() {
        let t = FakeTransport::default().with(FEED, Answer::Redirect(FEED.into()));
        assert!(matches!(
            get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10),
            Err(FetchError::TooManyRedirects)
        ));
        let t = FakeTransport::default().with(FEED, Answer::Body(vec![0; 11]));
        assert!(matches!(
            get_bytes(&t, &UrlPolicy::RELEASE, FEED, 10),
            Err(FetchError::TooLarge)
        ));
    }
}
