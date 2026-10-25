//! Runs the protocol cases shared with the Worker's tests (`conformance.json`,
//! `auth.json`, and `sign_in.json` once the Worker ships it) against `MemoryServer` through `kanso_sync::Api`, so the
//! server the sync tests use has the Worker's exact rules. The v1 cases
//! (`conformance-v1.json`) check the rollback test's stand-in for the old
//! Worker, in tests/rollback.rs.
// no-prop-test: fixed cases shared with the Worker; the server's rules have
// their own properties in tests/accounts.rs and tests/properties.rs.

// ---------------------------------------------------------------------------
// Protocol v2: auth.json and conformance.json over MemoryServer
// ---------------------------------------------------------------------------

mod v2 {
    use std::collections::BTreeMap;

    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use kanso_sync::api::{
        AccountPasskeysFinishRequest, AccountPasskeysStartRequest, ApprovalApproveRequest,
        ApprovalChallengeRequest, ApprovalCreateRequest, ApprovalNameRequest, DeleteRequest,
        DeviceNameRequest, LoginRequest, PasskeyFinishRequest, PasskeyProofRequest,
        PasskeyProofStartRequest, PasskeySignupFinishRequest, PasskeySignupStartRequest,
        PasskeyStartRequest, PasswordRequest, PreloginRequest, RecoverFinishRequest,
        RecoverStartRequest, RecoveryKeyRequest, SignInPasswordRequest, SignupRequest,
    };
    use kanso_sync::provider::{
        IdentityStartRequest, LinkOutcome, ProviderPollRequest, ProviderStart,
        ProviderStartRequest, ProviderStep, SocialSignupRequest, UnlinkRequest,
    };
    use kanso_sync::wire::PushRequest;
    use kanso_sync::{ApiError, Backend, KdfParams, MemoryServer, Provider, SessionToken};
    use ring::signature::{Ed25519KeyPair, KeyPair as _};
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};
    use sha2::{Digest as _, Sha256};

    fn fixture_path(name: &str) -> String {
        format!(
            "{}/../../cloud/test/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn load(name: &str) -> Value {
        let path = fixture_path(name);
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
        /// sign_in.json's Ed25519 authenticators: seed and credential ID.
        authenticators: BTreeMap<String, (Vec<u8>, String)>,
        /// Flows `identity_start` began: their poll is a link or a proof.
        identity_flows: std::collections::BTreeSet<String>,
        /// Steps only the Worker can answer (the AASA file), skipped.
        skipped: usize,
    }

    fn b64(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// A CBOR head: major type and argument, in the shortest form.
    fn cbor_head(major: u8, n: usize, out: &mut Vec<u8>) {
        let m = major << 5;
        match n {
            0..=23 => out.push(m | n as u8),
            24..=0xff => out.extend([m | 24, n as u8]),
            0x100..=0xffff => {
                out.push(m | 25);
                out.extend((n as u16).to_be_bytes());
            }
            _ => {
                out.push(m | 26);
                out.extend((n as u32).to_be_bytes());
            }
        }
    }

    fn cbor_text(text: &str, out: &mut Vec<u8>) {
        cbor_head(3, text.len(), out);
        out.extend(text.as_bytes());
    }

    fn cbor_bytes(bytes: &[u8], out: &mut Vec<u8>) {
        cbor_head(2, bytes.len(), out);
        out.extend(bytes);
    }

    /// The fixture's `client_data_json`, byte for byte.
    fn client_data(kind: &str, challenge: &str, origin: &str) -> Vec<u8> {
        format!(
            r#"{{"type":"{kind}","challenge":"{challenge}","origin":"{origin}","crossOrigin":false}}"#
        )
        .into_bytes()
    }

    impl Runner<'_> {
        fn lookup(&self, name: &str) -> Option<Value> {
            self.captured
                .get(name)
                .or_else(|| self.values.get(name))
                .cloned()
        }

        /// `resolve`, then sign_in.json's `$register` and `$assert`.
        fn body(&self, value: &Value) -> Result<Value, String> {
            let resolved = self.resolve(value)?;
            self.expand(&resolved)
        }

        /// Replaces each object holding `$register` or `$assert` by the
        /// WebAuthn fields it stands for, merged with its other keys
        /// (sign_in.json's `about`).
        fn expand(&self, value: &Value) -> Result<Value, String> {
            Ok(match value {
                Value::Array(items) => Value::Array(
                    items
                        .iter()
                        .map(|v| self.expand(v))
                        .collect::<Result<_, _>>()?,
                ),
                Value::Object(map) => {
                    let mut out = serde_json::Map::new();
                    for (k, v) in map {
                        if k != "$register" && k != "$assert" {
                            out.insert(k.clone(), self.expand(v)?);
                        }
                    }
                    for kind in ["$register", "$assert"] {
                        if let Some(spec) = map.get(kind) {
                            for (k, v) in self.webauthn(kind, spec)? {
                                out.insert(k, v);
                            }
                        }
                    }
                    Value::Object(out)
                }
                _ => value.clone(),
            })
        }

        fn webauthn(
            &self,
            kind: &str,
            spec: &Value,
        ) -> Result<serde_json::Map<String, Value>, String> {
            let name = spec["authenticator"].as_str().ok_or("an authenticator")?;
            let (seed, credential_id) = self
                .authenticators
                .get(name)
                .ok_or_else(|| format!("no authenticator {name}"))?;
            let key = Ed25519KeyPair::from_seed_unchecked(seed).map_err(|e| e.to_string())?;
            let challenge = spec["challenge"].as_str().ok_or("a challenge")?;
            let origin = spec["origin"]
                .as_str()
                .unwrap_or("https://sync.kansoapp.app");
            let rp_id = spec["rp_id"].as_str().unwrap_or("sync.kansoapp.app");
            let count = spec["sign_count"].as_u64();
            let flags = spec["flags"].as_u64();
            let credential = URL_SAFE_NO_PAD
                .decode(credential_id)
                .map_err(|e| e.to_string())?;
            let mut data = Sha256::digest(rp_id.as_bytes()).to_vec();
            let mut out = serde_json::Map::new();
            out.insert("credential_id".into(), json!(credential_id));
            if kind == "$register" {
                data.push(flags.unwrap_or(0x45) as u8);
                data.extend((count.unwrap_or(0) as u32).to_be_bytes());
                data.extend([0x07; 16]);
                data.extend((credential.len() as u16).to_be_bytes());
                data.extend(&credential);
                // {1: 1, 3: -8, -1: 6, -2: x}
                data.extend([0xa4, 0x01, 0x01, 0x03, 0x27, 0x20, 0x06, 0x21]);
                cbor_bytes(key.public_key().as_ref(), &mut data);
                let mut attestation = vec![0xa3];
                cbor_text("fmt", &mut attestation);
                cbor_text("none", &mut attestation);
                cbor_text("attStmt", &mut attestation);
                attestation.push(0xa0);
                cbor_text("authData", &mut attestation);
                cbor_bytes(&data, &mut attestation);
                out.insert("attestation_object".into(), json!(b64(&attestation)));
                out.insert(
                    "client_data_json".into(),
                    json!(b64(&client_data("webauthn.create", challenge, origin))),
                );
            } else {
                data.push(flags.unwrap_or(0x05) as u8);
                data.extend((count.unwrap_or(1) as u32).to_be_bytes());
                let client = client_data("webauthn.get", challenge, origin);
                let mut signed = data.clone();
                signed.extend(Sha256::digest(&client));
                out.insert("authenticator_data".into(), json!(b64(&data)));
                out.insert("client_data_json".into(), json!(b64(&client)));
                out.insert("signature".into(), json!(b64(key.sign(&signed).as_ref())));
            }
            Ok(out)
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
            if let Some(n) = step.get("repeat").and_then(Value::as_u64) {
                for _ in 0..n {
                    for inner in step["steps"].as_array().ok_or("repeat steps")? {
                        self.run(inner)?;
                    }
                }
                return Ok(());
            }
            if let Some(answer) = step.get("github").and_then(Value::as_str) {
                self.server.push_github_answer(answer);
                return Ok(());
            }
            // A static file of the Worker's, not an API MemoryServer serves.
            if step.get("request").and_then(Value::as_str) == Some("aasa") {
                self.skipped += 1;
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
                                "writes_per_day" => &mut quota.writes_per_day,
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
            let body = self.body(step.get("body").unwrap_or(&Value::Null))?;
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
            if route == "identity_start"
                && let Some(flow) = answer.as_ref().and_then(|a| a["flow_id"].as_str())
            {
                self.identity_flows.insert(flow.to_owned());
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
            /// The three approval answers say `{"ok": true}` with 200.
            fn ok(result: Result<(), ApiError>) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(()) => Ok((200, Some(json!({ "ok": true })))),
                    Err(err) => answer::<()>(200, Err(err)),
                }
            }
            let path_part = |key: &str| -> Result<String, String> {
                path.get(key)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| format!("path.{key}"))
            };
            let provider_part = || -> Result<Provider, String> {
                serde_json::from_value(json!(path_part("provider")?)).map_err(|e| e.to_string())
            };
            /// The provider answers, as the Worker writes them.
            fn started(
                result: Result<ProviderStart, ApiError>,
            ) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(ProviderStart::Github(g)) => Ok((
                        200,
                        Some(json!({
                            "flow_id": g.flow_id,
                            "user_code": g.user_code,
                            "verification_uri": g.verification_uri,
                            "interval": g.interval,
                            "expires_in": g.expires_in,
                        })),
                    )),
                    Ok(other) => Err(format!("an answer the fixture has no case for: {other:?}")),
                    Err(err) => answer::<()>(200, Err(err)),
                }
            }
            fn stepped(
                result: Result<ProviderStep, ApiError>,
            ) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(ProviderStep::Pending { slow_down }) => {
                        let mut value = json!({ "status": "pending" });
                        if slow_down {
                            value["slow_down"] = json!(true);
                        }
                        Ok((200, Some(value)))
                    }
                    Ok(ProviderStep::NewAccount(ticket)) => {
                        let mut value = serde_json::to_value(ticket).map_err(|e| e.to_string())?;
                        value["kind"] = json!("new_account");
                        Ok((200, Some(value)))
                    }
                    Ok(ProviderStep::Session(session)) => {
                        let mut value = serde_json::to_value(session).map_err(|e| e.to_string())?;
                        value["kind"] = json!("session");
                        Ok((200, Some(value)))
                    }
                    Err(err) => answer::<()>(200, Err(err)),
                }
            }
            fn linked(
                result: Result<LinkOutcome, ApiError>,
            ) -> Result<(u16, Option<Value>), String> {
                match result {
                    Ok(LinkOutcome::Linked {
                        provider,
                        created_at,
                    }) => Ok((
                        200,
                        Some(
                            json!({ "kind": "linked", "provider": provider.as_str(), "created_at": created_at }),
                        ),
                    )),
                    Ok(LinkOutcome::Proof { provider }) => Ok((
                        200,
                        Some(json!({ "kind": "proof", "provider": provider.as_str() })),
                    )),
                    Err(err) => answer::<()>(200, Err(err)),
                }
            }
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
                "approval_create" => answer(
                    200,
                    api.create_approval(token, &parse::<ApprovalCreateRequest>(body)?),
                ),
                "approval_list" => answer(200, api.approvals(token)),
                "approval_poll" => answer(200, api.approval(token, &path_part("id")?)),
                "approval_challenge" => ok(api.challenge_approval(
                    token,
                    &path_part("id")?,
                    &parse::<ApprovalChallengeRequest>(body)?,
                )),
                "approval_name" => ok(api.name_approval(
                    token,
                    &path_part("id")?,
                    &parse::<ApprovalNameRequest>(body)?,
                )),
                "approval_approve" => ok(api.approve_approval(
                    token,
                    &path_part("id")?,
                    &parse::<ApprovalApproveRequest>(body)?,
                )),
                "approval_cancel" => empty(api.cancel_approval(token, &path_part("id")?)),
                // sign_in.json's routes, by its names.
                "passkey_start" => {
                    answer(200, api.passkey_start(&parse::<PasskeyStartRequest>(body)?))
                }
                "passkey_finish" => answer(
                    200,
                    api.passkey_finish(&parse::<PasskeyFinishRequest>(body)?),
                ),
                "passkey_signup_start" => answer(
                    200,
                    api.passkey_signup_start(&parse::<PasskeySignupStartRequest>(body)?),
                ),
                "passkey_signup_finish" => answer(
                    201,
                    api.passkey_signup_finish(&parse::<PasskeySignupFinishRequest>(body)?),
                ),
                "sign_in_methods" => answer(200, api.account_sign_in_methods(token)),
                "proof_start" => answer(
                    200,
                    api.passkey_proof_start(token, &parse::<PasskeyProofStartRequest>(body)?),
                ),
                "passkey_register_start" => answer(
                    200,
                    api.account_passkeys_start(token, &parse::<AccountPasskeysStartRequest>(body)?),
                ),
                "passkey_register_finish" => answer(
                    201,
                    api.account_passkeys_finish(
                        token,
                        &parse::<AccountPasskeysFinishRequest>(body)?,
                    ),
                ),
                "passkey_remove" => empty(api.account_passkey_remove(
                    token,
                    &path_part("credential_id")?,
                    &parse::<PasskeyProofRequest>(body)?,
                )),
                "set_password" => answer(
                    200,
                    api.set_first_password(token, &parse::<SignInPasswordRequest>(body)?),
                ),
                "github_start" => started(
                    api.provider_start(&Provider::Github, &parse::<ProviderStartRequest>(body)?),
                ),
                "github_poll" => {
                    let request = parse::<ProviderPollRequest>(body)?;
                    if self.identity_flows.contains(&request.flow_id) {
                        linked(api.identity_poll(token, &Provider::Github, &request))
                    } else {
                        stepped(api.provider_poll(&Provider::Github, &request))
                    }
                }
                "social_signup" => match api.social_signup(&parse::<SocialSignupRequest>(body)?) {
                    Ok(session) => {
                        let mut value = serde_json::to_value(session).map_err(|e| e.to_string())?;
                        value["kind"] = json!("session");
                        Ok((201, Some(value)))
                    }
                    Err(err) => answer::<()>(201, Err(err)),
                },
                "identity_start" => started(api.identity_start(
                    token,
                    &provider_part()?,
                    &parse::<IdentityStartRequest>(body)?,
                )),
                "identity_unlink" => empty(api.unlink_identity(
                    token,
                    &provider_part()?,
                    &parse::<UnlinkRequest>(body)?,
                )),
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
        // sign_in.json leaves `protocol` out; it is 2 wherever it is given.
        assert!(matches!(
            fixture.get("protocol"),
            None | Some(Value::Number(_))
        ));
        if let Some(protocol) = fixture.get("protocol") {
            assert_eq!(protocol, 2);
        }
        let start = fixture["start"].as_i64().unwrap();
        let values = fixture["values"].as_object().unwrap();
        let mut failures = Vec::new();
        let mut ran = 0;
        let mut skipped = 0;
        let authenticators: BTreeMap<String, (Vec<u8>, String)> = fixture
            .get("authenticators")
            .and_then(Value::as_object)
            .map(|all| {
                all.iter()
                    .map(|(name, a)| {
                        let seed = URL_SAFE_NO_PAD.decode(a["seed"].as_str().unwrap()).unwrap();
                        (
                            name.clone(),
                            (seed, a["credential_id"].as_str().unwrap().to_owned()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
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
                authenticators: authenticators.clone(),
                identity_flows: Default::default(),
                skipped: 0,
            };
            runner.server.set_now(start);
            if let Some(Value::Object(base)) = fixture.get("env") {
                // A case's env merges over the file's; null unsets.
                let mut env = base.clone();
                if let Some(Value::Object(over)) = case.get("env") {
                    for (k, v) in over {
                        env.insert(k.clone(), v.clone());
                    }
                }
                let var = |name: &str| env.get(name).and_then(Value::as_str);
                runner
                    .server
                    .set_passkey_env(var("PASSKEY_RP_ID"), var("PASSKEY_ORIGINS"));
                let github = var("GITHUB_CLIENT_ID").is_some_and(|id| !id.trim().is_empty());
                runner.server.set_providers(if github {
                    vec![Provider::Github]
                } else {
                    vec![]
                });
            }
            if let Some(github) = fixture.get("github") {
                runner.server.set_github_device_flow(
                    &github["id"].to_string(),
                    github["user_code"].as_str().unwrap(),
                    github["interval"].as_u64().unwrap(),
                );
            }
            for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
                if let Err(why) = runner.run(step) {
                    failures.push(format!("{name} › {case_name} › step {index}: {why}"));
                    break;
                }
            }
            skipped += runner.skipped;
        }
        if skipped > 0 {
            eprintln!(
                "{name}: {ran} cases ran; {skipped} Worker-only steps (the AASA file) skipped"
            );
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

    /// The sign-in cases (docs/ACCOUNTS.md › Tests › Sign-in conformance).
    /// The Worker builder owns `sign_in.json`; until it lands this skips, and
    /// it must be re-run once it does.
    #[test]
    fn memory_server_follows_the_shared_sign_in_cases() {
        if !std::path::Path::new(&fixture_path("sign_in.json")).exists() {
            eprintln!("skipped: cloud/test/fixtures/sign_in.json isn't in this checkout yet");
            return;
        }
        assert!(run_fixture("sign_in.json") >= 24);
    }
}
