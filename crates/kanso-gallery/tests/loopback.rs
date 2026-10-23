//! The whole gallery over a loopback server: fetch and verify a signed
//! catalog, install a theme, update it, see it revoked, and refuse a
//! replayed index. No network leaves this machine.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use kanso_gallery::index::{Appearance, FileRef, Index, Revoked, SCHEMA, Theme};
use kanso_gallery::{Action, Floor, check_index, install_color, plan, sha256_hex};
use kanso_update::HttpTransport;

type Routes = Arc<Mutex<HashMap<String, Vec<u8>>>>;

fn theme(version: u64, bytes: &[u8]) -> Theme {
    Theme {
        id: "loop".into(),
        kind: kanso_gallery::Kind::Color,
        family: Some("loop".into()),
        version,
        name: "Loop".into(),
        appearance: Some(Appearance::Dark),
        author: "Test".into(),
        license: "MIT".into(),
        source: "https://example.com".into(),
        description: "A loopback theme.".into(),
        min_kanso: "0.1.0".into(),
        file: Some(FileRef {
            path: "loop.json".into(),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        }),
        bundle: None,
        preview: Vec::new(),
        colors: None,
    }
}

fn catalog(serial: u64, version: u64, bytes: &[u8], revoked: bool) -> Index {
    Index {
        schema: SCHEMA,
        serial,
        generated_at: "2026-10-23T12:00:00Z".into(),
        source_commit: "8d96298839a6176210cd1e3b376b82ffefc70ab3".into(),
        signing_key_id: "test".into(),
        themes: if revoked {
            Vec::new()
        } else {
            vec![theme(version, bytes)]
        },
        revoked: if revoked {
            vec![Revoked {
                id: "loop".into(),
                reason: "Withdrawn.".into(),
                since: "2026-10-01T00:00:00Z".into(),
            }]
        } else {
            Vec::new()
        },
    }
}

fn publish(routes: &Routes, index: &Index, key: &ed25519_dalek::SigningKey) {
    let bytes = serde_json::to_vec_pretty(index).unwrap();
    let sig = kanso_update::sign_feed(&bytes, key).into_bytes();
    let mut routes = routes.lock().unwrap();
    routes.insert("/gallery/index.json".into(), bytes);
    routes.insert("/gallery/index.json.sig".into(), sig);
}

fn serve(routes: Routes) -> (String, Arc<tiny_http::Server>) {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let base = format!("http://127.0.0.1:{port}");
    let server = Arc::new(server);
    let server_thread = server.clone();
    thread::spawn(move || {
        while let Ok(request) = server_thread.recv_timeout(Duration::from_secs(30)) {
            let Some(request) = request else { break };
            let path = request
                .url()
                .split('?')
                .next()
                .unwrap_or_default()
                .to_owned();
            let body = routes.lock().unwrap().get(&path).cloned();
            match body {
                Some(body) => {
                    request.respond(tiny_http::Response::from_data(body)).ok();
                }
                None => {
                    request.respond(tiny_http::Response::empty(404)).ok();
                }
            }
        }
    });
    (base, server)
}

#[test]
fn a_signed_catalog_installs_updates_revokes_and_refuses_a_replay() {
    let (key, _) = kanso_update::feed::generate_signing_key();
    let keys = vec![key.verifying_key()];
    let routes: Routes = Arc::new(Mutex::new(HashMap::new()));
    let first = b"{\"name\":\"Loop\",\"appearance\":\"dark\"}";
    let second = b"{\"name\":\"Loop\",\"appearance\":\"dark\",\"ui\":{\"editor\":\"#101010\"}}";
    let (base, server) = serve(routes.clone());
    let transport = HttpTransport::new("Kanso/test");
    let current = semver::Version::new(0, 9, 0);

    // Serial 1: install.
    publish(&routes, &catalog(1, 1, first, false), &key);
    routes
        .lock()
        .unwrap()
        .insert("/loop.json".into(), first.to_vec());
    let fetch = |path: &str, cap: u64| {
        kanso_update::fetch::get_bytes(
            &transport,
            &kanso_update::UrlPolicy {
                repository: "iamngoni/kanso-themes",
                allow_loopback: true,
            },
            &format!("{base}{path}"),
            cap,
        )
        .unwrap()
    };
    let index_bytes = fetch("/gallery/index.json", kanso_gallery::MAX_INDEX_BYTES);
    let signature = fetch(
        "/gallery/index.json.sig",
        kanso_gallery::MAX_SIGNATURE_BYTES,
    );
    let index = check_index(&index_bytes, &signature, &keys, &Floor::default(), &current).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("gallery");
    let file = fetch("/loop.json", kanso_gallery::MAX_COLOR_BYTES);
    let digest = install_color(&root, &index.themes[0], &file).unwrap();
    assert_eq!(std::fs::read(root.join("themes/loop.json")).unwrap(), first);
    let floor = index.floor(&index_bytes);
    let installed = vec![kanso_gallery::Installed {
        id: "loop".into(),
        kind: kanso_gallery::Kind::Color,
        version: 1,
        name: "Loop".into(),
        family: Some("loop".into()),
        digest: digest.clone(),
        current_digest: Some(digest),
        installed_at: "2026-10-23T12:00:00Z".into(),
    }];

    // Serial 2: an update to version 2 is planned, fetched and installed.
    publish(&routes, &catalog(2, 2, second, false), &key);
    routes
        .lock()
        .unwrap()
        .insert("/loop.json".into(), second.to_vec());
    let bytes = fetch("/gallery/index.json", kanso_gallery::MAX_INDEX_BYTES);
    let sig = fetch(
        "/gallery/index.json.sig",
        kanso_gallery::MAX_SIGNATURE_BYTES,
    );
    let newer = check_index(&bytes, &sig, &keys, &floor, &current).unwrap();
    let floor2 = newer.floor(&bytes);
    let planned = plan(&newer, &installed, &current);
    let Action::Update(theme) = &planned.actions[0] else {
        panic!("expected an update: {planned:?}");
    };
    let file = fetch("/loop.json", kanso_gallery::MAX_COLOR_BYTES);
    install_color(&root, theme, &file).unwrap();
    assert_eq!(
        std::fs::read(root.join("themes/loop.json")).unwrap(),
        second
    );

    // Serial 3: a signed revocation is planned as a removal.
    publish(&routes, &catalog(3, 2, second, true), &key);
    let bytes = fetch("/gallery/index.json", kanso_gallery::MAX_INDEX_BYTES);
    let sig = fetch(
        "/gallery/index.json.sig",
        kanso_gallery::MAX_SIGNATURE_BYTES,
    );
    let revoked = check_index(&bytes, &sig, &keys, &floor2, &current).unwrap();
    let planned = plan(&revoked, &installed, &current);
    assert!(matches!(
        planned.actions.as_slice(),
        [Action::Remove { .. }]
    ));

    // A replay of serial 1 against the accepted floor is refused.
    publish(&routes, &catalog(1, 1, first, false), &key);
    let bytes = fetch("/gallery/index.json", kanso_gallery::MAX_INDEX_BYTES);
    let sig = fetch(
        "/gallery/index.json.sig",
        kanso_gallery::MAX_SIGNATURE_BYTES,
    );
    assert_eq!(
        check_index(
            &bytes,
            &sig,
            &keys,
            &Floor {
                serial: 3,
                digest: String::new()
            },
            &current
        ),
        Err(kanso_gallery::IndexError::RolledBack)
    );
    server.unblock();
}
