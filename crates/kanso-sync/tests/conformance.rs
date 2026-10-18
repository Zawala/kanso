//! Runs the v1 protocol cases shared with the Worker's tests
//! (`cloud/test/fixtures/conformance-v1.json`) against `MemoryRemote`, so the
//! in-memory server used by the sync tests has the v1 Worker's exact rules.
//! The v2 cases (`conformance.json`, `auth.json`) run against `MemoryServer`
//! from step 2a.

use base64::Engine as _;
use kanso_sync::{ChangesPage, MemoryRemote, PushResult, Remote, RemoteError, SyncDoc};
use serde_json::Value;

fn fixture() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../cloud/test/fixtures/conformance-v1.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).expect("read the fixture"))
        .expect("parse the fixture")
}

fn expected_status(step: &Value) -> u16 {
    step.get("status").and_then(Value::as_u64).unwrap_or(200) as u16
}

fn status_of<T>(result: &Result<T, RemoteError>) -> u16 {
    match result {
        Ok(_) => 200,
        Err(err) => err
            .status()
            .unwrap_or_else(|| panic!("expected an HTTP status, got {err}")),
    }
}

fn run_step(remote: &MemoryRemote, case: &str, index: usize, step: &Value) {
    let at = format!("{case} / step {index}");
    if let Some(docs) = step.get("push") {
        let docs: Vec<SyncDoc> = match serde_json::from_value(docs.clone()) {
            Ok(docs) => docs,
            Err(_) => {
                // A shape the typed client cannot even express.
                assert_eq!(expected_status(step), 400, "{at}: unparseable push");
                return;
            }
        };
        let result = remote.push(&docs);
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
        if let Ok(results) = result {
            let expected: Vec<PushResult> =
                serde_json::from_value(step["results"].clone()).expect("results");
            assert_eq!(results, expected, "{at}: results");
        }
    } else if let Some(query) = step.get("changes") {
        let since = query["since"].as_i64().unwrap();
        let limit = query["limit"].as_u64().unwrap() as usize;
        let result = remote.changes(since, limit);
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
        if let Ok(page) = result {
            let expected: ChangesPage = serde_json::from_value(step["page"].clone()).expect("page");
            assert_eq!(page, expected, "{at}: page");
        }
    } else if let Some(file) = step.get("put_file") {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(file["base64"].as_str().unwrap())
            .unwrap();
        let result = remote.put_file(
            file["id"].as_str().unwrap(),
            file["md5"].as_str().unwrap(),
            "application/octet-stream",
            &bytes,
        );
        assert_eq!(status_of(&result), expected_status(step), "{at}: status");
    } else if let Some(id) = step.get("get_file") {
        let result = remote.get_file(id.as_str().unwrap()).map(|f| f.ok_or(()));
        let file = match result {
            Ok(Ok(file)) => file,
            Ok(Err(())) => {
                assert_eq!(expected_status(step), 404, "{at}: missing file");
                return;
            }
            Err(err) => panic!("{at}: {err}"),
        };
        assert_eq!(expected_status(step), 200, "{at}: status");
        assert_eq!(file.md5_hex, step["md5"].as_str().unwrap(), "{at}: md5");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(step["base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(file.bytes, bytes, "{at}: bytes");
    } else if step.get("push_body").is_some() || step.get("changes_query").is_some() {
        // Raw HTTP input; only the Worker's tests send it.
    } else {
        panic!("{at}: unknown step {step}");
    }
}

#[test]
fn memory_remote_follows_the_shared_protocol_cases() {
    let fixture = fixture();
    let cases = fixture["cases"].as_array().unwrap();
    assert!(cases.len() >= 10);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let remote = MemoryRemote::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            run_step(&remote, name, index, step);
        }
    }
}
