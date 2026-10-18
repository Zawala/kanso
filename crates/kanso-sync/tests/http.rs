//! `HttpBackend` against a loopback HTTP server (docs/ACCOUNTS.md › HTTP
//! protocol › Conventions): JSON bodies with their content type, bearer
//! tokens only on session routes, never an `Origin` header, file uploads
//! with `x-kanso-sha256`, no redirects followed, the response caps, and
//! error bodies mapped to their codes. Nothing leaves 127.0.0.1.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use kanso_sync::api::{PreloginRequest, WireKdf};
use kanso_sync::wire::{PushRequest, WirePushDoc};
use kanso_sync::{ApiError, Backend, HttpBackend, KdfParams, SessionToken, SignOutReason};
use sha2::Digest as _;

/// One request as the server saw it.
#[derive(Debug)]
struct Seen {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

/// Serves one canned answer per connection, in order, and reports each
/// request. Answers are raw HTTP/1.1 responses.
fn serve(answers: Vec<Vec<u8>>) -> (String, mpsc::Receiver<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for answer in answers {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut parts = line.split_whitespace();
            let method = parts.next().unwrap_or_default().to_owned();
            let path = parts.next().unwrap_or_default().to_owned();
            let mut headers = BTreeMap::new();
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                let header = header.trim_end();
                if header.is_empty() {
                    break;
                }
                let (name, value) = header.split_once(':').unwrap();
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
            }
            let length = headers
                .get("content-length")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let _ = tx.send(Seen {
                method,
                path,
                headers,
                body,
            });
            let mut stream = stream;
            let _ = stream.write_all(&answer);
            let _ = stream.flush();
        }
    });
    (url, rx)
}

fn answer(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

fn json(status: &str, body: serde_json::Value) -> Vec<u8> {
    answer(status, "application/json", body.to_string().as_bytes())
}

fn token() -> SessionToken {
    SessionToken::new(format!("kanso_{}", "A".repeat(43)))
}

#[test]
fn auth_requests_are_json_without_a_token_or_an_origin() {
    let (url, seen) = serve(vec![json(
        "200 OK",
        serde_json::json!({"kdf": {"alg": "argon2id", "version": 19, "m_kib": 1048576, "t": 3, "p": 4},
                           "salt": "AAAAAAAAAAAAAAAAAAAAAA", "account_id": format!("a_{}", "0".repeat(32))}),
    )]);
    let api = HttpBackend.api(&format!("{url}/"));
    let request = PreloginRequest {
        username: "alice".into(),
    };
    let answer = api.prelogin(&request).unwrap();
    assert_eq!(answer.kdf, WireKdf::from(KdfParams::DEFAULT));
    let seen = seen.recv().unwrap();
    assert_eq!(
        (seen.method.as_str(), seen.path.as_str()),
        ("POST", "/v2/auth/prelogin")
    );
    assert_eq!(seen.headers["content-type"], "application/json");
    assert!(!seen.headers.contains_key("origin"));
    assert!(!seen.headers.contains_key("authorization"));
    assert!(seen.headers["user-agent"].starts_with("kanso-sync/"));
    let body: serde_json::Value = serde_json::from_slice(&seen.body).unwrap();
    assert_eq!(body, serde_json::json!({"username": "alice"}));
}

#[test]
fn session_routes_send_the_bearer_token() {
    let page = serde_json::json!({"docs": [], "last_seq": 7, "more": false, "head_seq": 7,
                                  "epoch": "0123456789abcdef", "key_id": "fedcba9876543210"});
    let (url, seen) = serve(vec![
        json("200 OK", page),
        answer("204 No Content", "text/plain", b""),
        json(
            "200 OK",
            serde_json::json!({"results": [{"id": "note:AAAAAAAAAAAAAAAAAAAAAA", "status": "behind"}]}),
        ),
    ]);
    let api = HttpBackend.api(&url);
    let page = api.changes(&token(), Some(7), Some(100)).unwrap();
    assert_eq!((page.last_seq, page.head_seq), (7, 7));
    let seen_page = seen.recv().unwrap();
    assert_eq!(seen_page.path, "/v2/changes?since=7&limit=100");
    assert_eq!(
        seen_page.headers["authorization"],
        format!("Bearer kanso_{}", "A".repeat(43))
    );
    assert!(!seen_page.headers.contains_key("origin"));

    api.revoke_device(&token(), &format!("d_{}", "1".repeat(32)))
        .unwrap();
    let revoke = seen.recv().unwrap();
    assert_eq!(revoke.method, "DELETE");
    assert_eq!(
        revoke.path,
        format!("/v2/account/devices/d_{}", "1".repeat(32))
    );

    let request = PushRequest {
        docs: vec![WirePushDoc {
            id: "note:AAAAAAAAAAAAAAAAAAAAAA".into(),
            rev: "0".repeat(32),
            deleted: false,
            sealed: "A".repeat(60),
            base_rev: None,
        }],
    };
    let results = api.push(&token(), &request).unwrap().results;
    assert_eq!(results.len(), 1);
    let push = seen.recv().unwrap();
    assert_eq!(push.headers["content-type"], "application/json");
    let body: serde_json::Value = serde_json::from_slice(&push.body).unwrap();
    assert_eq!(body["docs"][0]["base_rev"], serde_json::Value::Null);
}

#[test]
fn file_uploads_are_octet_streams_with_their_sha256() {
    let sealed = vec![1u8; 64];
    let (url, seen) = serve(vec![
        json("200 OK", serde_json::json!({"ok": true, "size": 64})),
        answer("200 OK", "application/octet-stream", &sealed),
        json(
            "404 Not Found",
            serde_json::json!({"error": "not_found", "message": "No such file."}),
        ),
    ]);
    let api = HttpBackend.api(&url);
    let wire_id = "file:AAAAAAAAAAAAAAAAAAAAAA";
    assert_eq!(api.put_file(&token(), wire_id, &sealed).unwrap().size, 64);
    let put = seen.recv().unwrap();
    assert_eq!(
        (put.method.as_str(), put.path.as_str()),
        ("PUT", "/v2/files/file%3AAAAAAAAAAAAAAAAAAAAAAA")
    );
    assert_eq!(put.headers["content-type"], "application/octet-stream");
    let digest: String = sha2::Sha256::digest(&sealed)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(put.headers["x-kanso-sha256"], digest);
    assert_eq!(put.body, sealed);

    assert_eq!(api.get_file(&token(), wire_id).unwrap(), Some(sealed));
    assert_eq!(api.get_file(&token(), wire_id).unwrap(), None);
}

#[test]
fn a_redirect_is_never_followed() {
    let redirect = b"HTTP/1.1 307 Temporary Redirect\r\nlocation: /v2/health\r\ncontent-length: 0\r\nconnection: close\r\n\r\n".to_vec();
    let (url, seen) = serve(vec![
        redirect,
        json(
            "200 OK",
            serde_json::json!({"ok": true, "protocol": 2, "signup": "open"}),
        ),
    ]);
    let api = HttpBackend.api(&url);
    assert!(matches!(api.health(), Err(ApiError::Protocol(_))));
    assert_eq!(seen.recv().unwrap().path, "/v2/health");
    assert!(
        seen.recv_timeout(Duration::from_millis(300)).is_err(),
        "the redirect was followed"
    );
}

#[test]
fn answers_over_the_cap_are_refused() {
    let big = vec![b' '; 70 * 1024];
    let (url, _seen) = serve(vec![answer("200 OK", "application/json", &big)]);
    let api = HttpBackend.api(&url);
    assert!(matches!(api.health(), Err(ApiError::Protocol(message)) if message.contains("cap")));
}

#[test]
fn error_bodies_map_to_their_codes() {
    let (url, _seen) = serve(vec![
        json(
            "401 Unauthorized",
            serde_json::json!({"error": "unauthorized", "reason": "evicted", "message": "x"}),
        ),
        json(
            "429 Too Many Requests",
            serde_json::json!({"error": "rate_limited", "retry_after": 30, "limit": "writes-account"}),
        ),
        json(
            "413 Payload Too Large",
            serde_json::json!({"error": "quota_exceeded", "quota": "files", "used": 3, "limit": 3}),
        ),
        answer("502 Bad Gateway", "text/html", b"<html>"),
    ]);
    let api = HttpBackend.api(&url);
    assert_eq!(
        api.account(&token()).unwrap_err(),
        ApiError::Unauthorized {
            reason: SignOutReason::Evicted
        }
    );
    assert_eq!(
        api.devices(&token()).unwrap_err(),
        ApiError::RateLimited {
            retry_after: 30,
            limit: "writes-account".into()
        }
    );
    assert!(matches!(
        api.put_file(&token(), "file:AAAAAAAAAAAAAAAAAAAAAA", &[0; 41])
            .unwrap_err(),
        ApiError::QuotaExceeded {
            quota: kanso_sync::QuotaKind::Files,
            used: 3,
            limit: 3
        }
    ));
    assert!(matches!(api.health().unwrap_err(), ApiError::Protocol(_)));
}

#[test]
fn no_answer_is_a_transport_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let api = HttpBackend.api(&url);
    assert!(matches!(api.health(), Err(ApiError::Transport(_))));
}

#[test]
fn a_logout_gives_up_after_five_seconds() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    // Accept and never answer.
    let hold = thread::spawn(move || {
        let held = listener.accept();
        thread::sleep(Duration::from_secs(8));
        drop(held);
    });
    let api = HttpBackend.api(&url);
    let start = Instant::now();
    assert!(matches!(api.logout(&token()), Err(ApiError::Transport(_))));
    let took = start.elapsed();
    assert!(
        took >= Duration::from_secs(4) && took < Duration::from_secs(7),
        "{took:?}"
    );
    drop(hold);
}

#[test]
fn the_backend_uses_the_production_kdf_floor() {
    assert_eq!(HttpBackend.kdf_floor(), KdfParams::FLOOR);
    assert_eq!(HttpBackend.kdf_default(), KdfParams::DEFAULT);
}
