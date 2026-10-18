//! Runs the protocol cases shared with the Worker's tests against the
//! in-memory servers, so the servers the sync tests use have the Worker's
//! exact rules: the v1 cases (`cloud/test/fixtures/conformance-v1.json`)
//! against `MemoryRemote`, and the v2 cases (`conformance.json`, `auth.json`)
//! against `MemoryServer` through `kanso_sync::Api`.
// no-prop-test: fixed cases shared with the Worker; the server's rules have
// their own properties in tests/accounts.rs and tests/properties.rs.

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

// ---------------------------------------------------------------------------
// Protocol v2: auth.json and conformance.json over MemoryServer
// ---------------------------------------------------------------------------

mod v2 {
    use std::collections::BTreeMap;

    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use kanso_sync::api::{
        DeleteRequest, DeviceNameRequest, LoginRequest, PasswordRequest, PreloginRequest,
        RecoverFinishRequest, RecoverStartRequest, RecoveryKeyRequest, SignupRequest,
    };
    use kanso_sync::wire::PushRequest;
    use kanso_sync::{ApiError, Backend, KdfParams, MemoryServer, SessionToken};
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    fn load(name: &str) -> Value {
        let path = format!(
            "{}/../../cloud/test/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
    }

    /// One case's state: the server, the clock and the captured values.
    struct Runner<'a> {
        server: MemoryServer,
        values: &'a serde_json::Map<String, Value>,
        captured: BTreeMap<String, Value>,
        start: i64,
        t: i64,
    }

    impl Runner<'_> {
        fn lookup(&self, name: &str) -> Option<Value> {
            self.captured
                .get(name)
                .or_else(|| self.values.get(name))
                .cloned()
        }

        /// Replaces every string that is exactly `"$name"`.
        fn resolve(&self, value: &Value) -> Result<Value, String> {
            Ok(match value {
                Value::String(text) => match text.strip_prefix('$') {
                    Some(name) => self
                        .lookup(name)
                        .ok_or_else(|| format!("unknown ${name}"))?,
                    None => value.clone(),
                },
                Value::Array(items) => Value::Array(
                    items
                        .iter()
                        .map(|v| self.resolve(v))
                        .collect::<Result<_, _>>()?,
                ),
                Value::Object(map) => Value::Object(
                    map.iter()
                        .map(|(k, v)| Ok((k.clone(), self.resolve(v)?)))
                        .collect::<Result<_, String>>()?,
                ),
                _ => value.clone(),
            })
        }

        /// The fixtures' pattern rules (see their `about`).
        fn matches(
            &self,
            pattern: &Value,
            actual: Option<&Value>,
            path: &str,
        ) -> Result<(), String> {
            let fail = |why: &str| {
                Err(format!(
                    "{path}: {why}: expected {pattern}, got {}",
                    actual.map_or("nothing".to_owned(), Value::to_string)
                ))
            };
            match pattern {
                Value::Null => match actual {
                    None | Some(Value::Null) => Ok(()),
                    Some(_) => fail("not null"),
                },
                Value::String(text) if text == "*" => match actual {
                    Some(v) if !v.is_null() => Ok(()),
                    _ => fail("missing"),
                },
                Value::String(text) if text.starts_with("!$") => {
                    let other = self
                        .lookup(&text[2..])
                        .ok_or_else(|| format!("unknown {text}"))?;
                    if actual == Some(&other) {
                        fail("equal")
                    } else {
                        Ok(())
                    }
                }
                Value::String(text) if text.starts_with('$') => {
                    let value = self
                        .lookup(&text[1..])
                        .ok_or_else(|| format!("unknown {text}"))?;
                    if actual == Some(&value) {
                        Ok(())
                    } else {
                        fail("differs")
                    }
                }
                Value::Object(map) => {
                    let Some(Value::Object(got)) = actual else {
                        return fail("not an object");
                    };
                    for (key, sub) in map {
                        if key == "message" {
                            continue;
                        }
                        self.matches(sub, got.get(key), &format!("{path}.{key}"))?;
                    }
                    Ok(())
                }
                Value::Array(items) => {
                    let Some(Value::Array(got)) = actual else {
                        return fail("not an array");
                    };
                    if got.len() != items.len() {
                        return fail("length");
                    }
                    for (i, (sub, item)) in items.iter().zip(got).enumerate() {
                        self.matches(sub, Some(item), &format!("{path}[{i}]"))?;
                    }
                    Ok(())
                }
                _ => {
                    if actual == Some(pattern) {
                        Ok(())
                    } else {
                        fail("differs")
                    }
                }
            }
        }

        fn run(&mut self, step: &Value) -> Result<(), String> {
            if step.get("runner").and_then(Value::as_str) == Some("worker")
                || step.get("raw").is_some()
            {
                return Ok(());
            }
            if let Some(Value::Object(settings)) = step.get("settings") {
                for (name, value) in settings {
                    self.server
                        .set_setting(name, value.as_str().ok_or("a setting")?);
                }
                return Ok(());
            }
            if let Some(admin) = step.get("admin").and_then(Value::as_str) {
                let username = step["username"].as_str().ok_or("admin username")?;
                match admin {
                    "disable" => self.server.disable(username),
                    "enable" => self.server.enable(username),
                    "quota" => {
                        let mut quota = self.server.quota(username);
                        let given = step["quota"].as_object().ok_or("admin quota")?;
                        for (column, value) in given {
                            let slot = match column.as_str() {
                                "docs" => &mut quota.docs,
                                "doc_bytes" => &mut quota.doc_bytes,
                                "files" => &mut quota.files,
                                "file_bytes" => &mut quota.file_bytes,
                                other => return Err(format!("unknown quota column {other}")),
                            };
                            *slot = value.as_u64();
                        }
                        self.server.set_quota(username, quota);
                    }
                    other => return Err(format!("unknown admin step {other}")),
                }
                return Ok(());
            }
            let route = step["request"].as_str().ok_or("a step of no known kind")?;
            if let Some(t) = step.get("t").and_then(Value::as_i64) {
                if t < self.t {
                    return Err("time went backwards".into());
                }
                self.t = t;
            }
            self.server.set_now(self.start + self.t);
            let token = match (step.get("as"), step.get("token")) {
                (Some(Value::String(name)), _) => self
                    .captured
                    .get(name)
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("no captured token {name}"))?
                    .to_owned(),
                (_, Some(token)) => self
                    .resolve(token)?
                    .as_str()
                    .ok_or("a literal token")?
                    .to_owned(),
                _ => String::new(),
            };
            let token = SessionToken::new(token);
            let body = self.resolve(step.get("body").unwrap_or(&Value::Null))?;
            let path = self.resolve(step.get("path").unwrap_or(&Value::Null))?;
            let (status, answer) = self.send(route, &token, body, &path, step)?;
            let expected = step["status"].as_u64().ok_or("a status")? as u16;
            if status != expected {
                return Err(format!(
                    "{route}: status {status}, expected {expected}; body {}",
                    answer.as_ref().map_or("none".to_owned(), Value::to_string)
                ));
            }
            if let Some(pattern) = step.get("response") {
                self.matches(pattern, answer.as_ref(), route)?;
            }
            if let Some(Value::Object(capture)) = step.get("capture") {
                for (name, at) in capture {
                    let mut value = answer.as_ref().ok_or("nothing to capture from")?;
                    for key in at.as_str().ok_or("a capture path")?.split('.') {
                        value = value
                            .get(key)
                            .ok_or_else(|| format!("no {at} to capture"))?;
                    }
                    self.captured.insert(name.clone(), value.clone());
                }
            }
            Ok(())
        }

        fn send(
            &self,
            route: &str,
            token: &SessionToken,
            body: Value,
            path: &Value,
            step: &Value,
        ) -> Result<(u16, Option<Value>), String> {
            fn parse<T: DeserializeOwned>(body: Value) -> Result<T, String> {
                serde_json::from_value(body)
                    .map_err(|e| format!("a body the typed client can't send: {e}"))
            }
            fn answer<T: Serialize>(
                status: u16,
                result: Result<T, ApiError>,
            ) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(value) => Ok((
                        status,
                        Some(serde_json::to_value(value).map_err(|e| e.to_string())?),
                    )),
                    Err(err) => match err.status() {
                        Some(status) => Ok((status, err.body())),
                        None => Err(format!("no HTTP answer: {err}")),
                    },
                }
            }
            fn empty(result: Result<(), ApiError>) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(()) => Ok((204, None)),
                    Err(err) => answer::<()>(204, Err(err)),
                }
            }
            let path_part = |key: &str| -> Result<String, String> {
                path.get(key)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| format!("path.{key}"))
            };
            let api = self.server.api("http://127.0.0.1:8787");
            match route {
                "health" => answer(200, api.health()),
                "prelogin" => answer(200, api.prelogin(&parse::<PreloginRequest>(body)?)),
                "signup" => answer(201, api.signup(&parse::<SignupRequest>(body)?)),
                "login" => answer(200, api.login(&parse::<LoginRequest>(body)?)),
                "recover_start" => {
                    answer(200, api.recover_start(&parse::<RecoverStartRequest>(body)?))
                }
                "recover_finish" => answer(
                    200,
                    api.recover_finish(&parse::<RecoverFinishRequest>(body)?),
                ),
                "logout" => empty(api.logout(token)),
                "account" => answer(200, api.account(token)),
                "password" => answer(
                    200,
                    api.change_password(token, &parse::<PasswordRequest>(body)?),
                ),
                "recovery_key" => answer(
                    200,
                    api.replace_recovery_key(token, &parse::<RecoveryKeyRequest>(body)?),
                ),
                "delete" => answer(
                    200,
                    api.delete_account(token, &parse::<DeleteRequest>(body)?),
                ),
                "set_device" => {
                    empty(api.set_device_name(token, &parse::<DeviceNameRequest>(body)?))
                }
                "devices" => answer(200, api.devices(token)),
                "revoke_device" => empty(api.revoke_device(token, &path_part("device_id")?)),
                "push" => answer(200, api.push(token, &parse::<PushRequest>(body)?)),
                "changes" => {
                    let query = step.get("query");
                    let since = query.and_then(|q| q.get("since")).and_then(Value::as_i64);
                    let limit = query
                        .and_then(|q| q.get("limit"))
                        .and_then(Value::as_u64)
                        .map(|limit| limit as usize);
                    answer(200, api.changes(token, since, limit))
                }
                "put_file" => {
                    let file = self.resolve(step.get("file").ok_or("put_file needs a file")?)?;
                    let bytes = URL_SAFE_NO_PAD
                        .decode(file.as_str().ok_or("a base64url file")?)
                        .map_err(|e| e.to_string())?;
                    answer(200, api.put_file(token, &path_part("wire_id")?, &bytes))
                }
                "get_file" => match api.get_file(token, &path_part("wire_id")?) {
                    Ok(Some(bytes)) => {
                        Ok((200, Some(json!({ "bytes": URL_SAFE_NO_PAD.encode(bytes) }))))
                    }
                    Ok(None) => Ok((404, Some(json!({ "error": "not_found" })))),
                    Err(err) => answer::<()>(200, Err(err)),
                },
                other => Err(format!("unknown route {other}")),
            }
        }
    }

    /// Runs every case MemoryServer runs; returns how many ran.
    fn run_fixture(name: &str) -> usize {
        let fixture = load(name);
        assert_eq!(fixture["protocol"], 2);
        let start = fixture["start"].as_i64().unwrap();
        let values = fixture["values"].as_object().unwrap();
        let mut failures = Vec::new();
        let mut ran = 0;
        for case in fixture["cases"].as_array().unwrap() {
            if case.get("runner").and_then(Value::as_str) == Some("worker") {
                continue;
            }
            ran += 1;
            let case_name = case["name"].as_str().unwrap();
            // The fixtures assume the Worker's production KDF floor and default.
            let mut runner = Runner {
                server: MemoryServer::with_kdf(KdfParams::FLOOR, KdfParams::DEFAULT),
                values,
                captured: BTreeMap::new(),
                start,
                t: 0,
            };
            runner.server.set_now(start);
            for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                if let Err(why) = runner.run(step) {
                    failures.push(format!("{name} › {case_name} › step {index}: {why}"));
                    break;
                }
            }
        }
        assert!(
            failures.is_empty(),
            "{} failing cases:\n{}",
            failures.len(),
            failures.join("\n")
        );
        ran
    }

    #[test]
    fn memory_server_follows_the_shared_auth_cases() {
        assert!(run_fixture("auth.json") >= 20);
    }

    #[test]
    fn memory_server_follows_the_shared_sync_cases() {
        assert!(run_fixture("conformance.json") >= 15);
    }
}
