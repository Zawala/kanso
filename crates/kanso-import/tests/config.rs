//! Example tests for reading a PouchDB library's config and locating its
//! database.
//! All credentials here are fake.

use std::fs;

use base64::Engine as _;
use kanso_import::{
    DEFAULT_PORT, LocalServerCredentials, find_pouch_db, parse_local_server_config,
    read_local_server_config,
};
use serde_json::json;

fn config_with_auth(auth: serde_json::Value) -> String {
    json!({
        "*": {
            "core": {
                "server": {"enabled": true, "port": 19841, "bindAddress": "127.0.0.1", "auth": auth},
                "theme": "dark"
            },
            "editor": {"lineNumbers": true}
        }
    })
    .to_string()
}

#[test]
fn plain_auth_splits_at_the_first_colon_and_debug_redacts_it() {
    let config = parse_local_server_config(&config_with_auth(json!("fake-user:hunter2:x")))
        .unwrap()
        .unwrap();
    assert!(config.enabled);
    assert_eq!(config.port, 19841);
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.username(), Some("fake-user"));
    assert_eq!(config.password(), Some("hunter2:x"));
    let debug = format!("{config:?}");
    assert!(!debug.contains("hunter2"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
    assert!(debug.contains("19841"), "{debug}");
    let pretty = format!("{config:#?}");
    assert!(!pretty.contains("hunter2"), "{pretty}");
}

#[test]
fn object_auth_is_accepted() {
    let config = parse_local_server_config(&config_with_auth(
        json!({"username": "fake-user", "password": "s3cret-fake"}),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(config.username(), Some("fake-user"));
    assert_eq!(config.password(), Some("s3cret-fake"));
    assert!(!format!("{config:?}").contains("s3cret-fake"));
}

#[test]
fn electron_safe_storage_auth_is_recognized_as_encrypted() {
    // `v10` + 48 bytes of (fake) AES-CBC ciphertext, as Electron's safeStorage writes it.
    let mut blob = b"v10".to_vec();
    blob.extend((0..48u8).map(|b| b.wrapping_mul(37)));
    let encoded = base64::engine::general_purpose::STANDARD.encode(&blob);
    let config = parse_local_server_config(&config_with_auth(json!(encoded)))
        .unwrap()
        .unwrap();
    assert_eq!(
        config.credentials,
        LocalServerCredentials::Encrypted {
            ciphertext: encoded.clone()
        }
    );
    assert_eq!(config.username(), None);
    assert_eq!(config.password(), None);
    let debug = format!("{config:?}");
    assert!(!debug.contains(&encoded), "{debug}");
    assert!(debug.contains("Encrypted"), "{debug}");
}

#[test]
fn other_auth_shapes() {
    let parse = |auth| {
        parse_local_server_config(&config_with_auth(auth))
            .unwrap()
            .unwrap()
            .credentials
    };
    assert_eq!(parse(json!(null)), LocalServerCredentials::None);
    assert_eq!(parse(json!("")), LocalServerCredentials::None);
    assert_eq!(
        parse(json!("no-colon-not-base64!")),
        LocalServerCredentials::Unrecognized
    );
    assert_eq!(parse(json!(42)), LocalServerCredentials::Unrecognized);
    assert_eq!(
        parse(json!({"username": "u"})),
        LocalServerCredentials::Unrecognized
    );
}

#[test]
fn defaults_and_missing_sections() {
    let config = parse_local_server_config(r#"{"*": {"core": {"server": {}}}}"#)
        .unwrap()
        .unwrap();
    assert!(!config.enabled);
    assert_eq!(config.port, DEFAULT_PORT);
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.credentials, LocalServerCredentials::None);

    assert!(
        parse_local_server_config(r#"{"*": {"core": {}}}"#)
            .unwrap()
            .is_none()
    );
    assert!(parse_local_server_config("{}").unwrap().is_none());
}

#[test]
fn invalid_values_are_errors_that_never_echo_the_password() {
    let bad_port = json!({"*": {"core": {"server": {"port": 70000, "auth": "u:hunter2"}}}});
    let err = parse_local_server_config(&bad_port.to_string()).unwrap_err();
    assert!(format!("{err:#}").contains("port"));
    assert!(!format!("{err:#} {err:?}").contains("hunter2"));

    let broken = r#"{"*": {"core": {"server": {"auth": "u:hunter2""#;
    let err = parse_local_server_config(broken).unwrap_err();
    assert!(!format!("{err:#} {err:?}").contains("hunter2"));
}

#[test]
fn reads_config_json_from_the_library_dir() {
    let dir = tempfile::tempdir().unwrap();
    assert!(read_local_server_config(dir.path()).unwrap().is_none());
    fs::write(
        dir.path().join("config.json"),
        config_with_auth(json!("a:b")),
    )
    .unwrap();
    let config = read_local_server_config(dir.path()).unwrap().unwrap();
    assert_eq!(config.port, 19841);
    assert_eq!(config.username(), Some("a"));
}

#[test]
fn finds_the_main_database_among_library_files() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("db");
    let hash = "5c172db466294e6a0b7855cdba0b8d18";
    let view = "21f5a21417c6d244c49767ad979917ae";
    fs::create_dir_all(db.join(hash)).unwrap();
    fs::write(db.join(hash).join("fts.db"), b"").unwrap();
    fs::create_dir_all(db.join(format!("{hash}-mrview-{view}"))).unwrap();
    for name in [
        format!("{hash}-mrview-{view}.sqlite"),
        format!("{hash}-mrview-{view}.sqlite-wal"),
        format!("{hash}.sqlite-wal"),
        format!("{hash}.sqlite-shm"),
        "notes.sqlite".to_owned(),
    ] {
        fs::write(db.join(name), b"").unwrap();
    }
    let err = find_pouch_db(dir.path()).unwrap_err();
    assert!(err.to_string().contains("no PouchDB library"), "{err}");

    fs::write(db.join(format!("{hash}.sqlite")), b"").unwrap();
    assert_eq!(
        find_pouch_db(dir.path()).unwrap(),
        db.join(format!("{hash}.sqlite"))
    );

    fs::write(db.join("0123456789abcdef0123456789abcdef.sqlite"), b"").unwrap();
    let err = find_pouch_db(dir.path()).unwrap_err();
    assert!(
        err.to_string().contains("several PouchDB libraries"),
        "{err}"
    );

    let missing = find_pouch_db(&dir.path().join("nope")).unwrap_err();
    assert!(missing.to_string().contains("cannot list"), "{missing}");
}
