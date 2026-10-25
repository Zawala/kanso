//! The step-1 contract between the Worker and the Rust client: every request
//! in `cloud/test/fixtures/auth.json`, `conformance.json` and, once the Worker
//! ships it, `sign_in.json` (protocol v2)
//! has the shape of `kanso_sync::api`, every error is one `ApiError` knows,
//! every setting is one D1 accepts and every value matches its wire pattern.
//! The fixtures' behaviour is checked by the runners (step 2: the Worker's
//! tests and `tests/conformance.rs` over `MemoryServer`); this test keeps the
//! shapes from drifting before then.
// no-prop-test: fixed fixtures checked case by case; the shapes they use have
// their own property tests in wire.rs and api.rs.

use std::collections::{BTreeMap, BTreeSet};

use kanso_sync::api::{
    AccountPasskeysFinishRequest, AccountPasskeysStartRequest, ApprovalApproveRequest,
    ApprovalChallengeRequest, ApprovalCreateRequest, ApprovalNameRequest, DeleteRequest,
    DeviceNameRequest, LoginRequest, PasskeyFinishRequest, PasskeyProofRequest,
    PasskeyProofStartRequest, PasskeySignupFinishRequest, PasskeySignupStartRequest,
    PasskeyStartRequest, PasswordRequest, PreloginRequest, RecoverFinishRequest,
    RecoverStartRequest, RecoveryKeyRequest, SignInPasswordRequest, SignupRequest, WireKdf,
};
use kanso_sync::provider::{
    IdentityStartRequest, ProviderPollRequest, ProviderStartRequest, SocialSignupRequest,
    UnlinkRequest,
};
use kanso_sync::wire::{self, PushRequest, PushResponse};
use kanso_sync::{ApiError, KdfParams, Route, is_valid_setting};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// The crate's folder: cargo's at run time (the phone runners set it to where
/// they copied the fixtures), else where it was built.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into())
}

fn load(name: &str) -> Value {
    let path = manifest_dir().join(format!("../../cloud/test/fixtures/{name}"));
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Replaces `"$name"` with the fixture value, or with a placeholder string for
/// a value captured at run time.
fn substitute(
    value: &Value,
    values: &BTreeMap<String, Value>,
    captured: &BTreeSet<String>,
) -> Value {
    match value {
        Value::String(text) => match text.strip_prefix('$') {
            Some(name) if values.contains_key(name) => values[name].clone(),
            Some(name) if captured.contains(name) => Value::String(format!("<{name}>")),
            Some(name) => panic!("unknown ${name}"),
            None => value.clone(),
        },
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|v| substitute(v, values, captured))
                .collect(),
        ),
        Value::Object(map) => {
            let mut out: serde_json::Map<String, Value> = map
                .iter()
                .filter(|(k, _)| *k != "$register" && *k != "$assert")
                .map(|(k, v)| (k.clone(), substitute(v, values, captured)))
                .collect();
            // sign_in.json's generators stand for these fields (the runners
            // make them); here they are base64url placeholders.
            let generated: &[&str] = if map.contains_key("$register") {
                &["credential_id", "attestation_object", "client_data_json"]
            } else if map.contains_key("$assert") {
                &[
                    "credential_id",
                    "authenticator_data",
                    "client_data_json",
                    "signature",
                ]
            } else {
                &[]
            };
            for field in generated {
                out.insert((*field).to_owned(), Value::String("AAAA".into()));
            }
            Value::Object(out)
        }
        _ => value.clone(),
    }
}

/// A step's route: a `Route` name, or one of sign_in.json's. `None` for the
/// AASA file, which is the Worker's static file, not an API route.
fn route_of(name: &str) -> Option<Route> {
    Some(match name {
        "aasa" => return None,
        "github_start" => Route::ProviderStart,
        "github_poll" => Route::ProviderPoll,
        "sign_in_methods" => Route::AccountSignInMethods,
        "proof_start" => Route::PasskeyProofStart,
        "set_password" => Route::SignInPassword,
        "passkey_register_start" => Route::AccountPasskeysStart,
        "passkey_register_finish" => Route::AccountPasskeysFinish,
        "passkey_remove" => Route::AccountPasskeyRemove,
        other => serde_json::from_value(Value::String(other.to_owned()))
            .unwrap_or_else(|e| panic!("route {other}: {e}")),
    })
}

fn shape<T: DeserializeOwned>(body: &Value, at: &str) {
    if let Err(e) = serde_json::from_value::<T>(body.clone()) {
        panic!("{at}: body doesn't fit {}: {e}", std::any::type_name::<T>());
    }
}

fn check_body(route: Route, body: Option<&Value>, at: &str) {
    let needs_body = !matches!(
        route,
        Route::Health
            | Route::Logout
            | Route::Account
            | Route::Devices
            | Route::RevokeDevice
            | Route::Changes
            | Route::PutFile
            | Route::GetFile
            | Route::ApprovalList
            | Route::ApprovalPoll
            | Route::ApprovalCancel
            | Route::AccountSignInMethods
            | Route::SignInMethods
    );
    assert_eq!(
        body.is_some(),
        needs_body,
        "{at}: body present for {route:?}"
    );
    let Some(body) = body else { return };
    match route {
        Route::Prelogin => shape::<PreloginRequest>(body, at),
        Route::Signup => shape::<SignupRequest>(body, at),
        Route::Login => shape::<LoginRequest>(body, at),
        Route::RecoverStart => shape::<RecoverStartRequest>(body, at),
        Route::RecoverFinish => shape::<RecoverFinishRequest>(body, at),
        Route::Password => shape::<PasswordRequest>(body, at),
        Route::RecoveryKey => shape::<RecoveryKeyRequest>(body, at),
        Route::Delete => shape::<DeleteRequest>(body, at),
        Route::SetDevice => shape::<DeviceNameRequest>(body, at),
        Route::ApprovalCreate => shape::<ApprovalCreateRequest>(body, at),
        Route::ApprovalChallenge => shape::<ApprovalChallengeRequest>(body, at),
        Route::ApprovalName => shape::<ApprovalNameRequest>(body, at),
        Route::ApprovalApprove => shape::<ApprovalApproveRequest>(body, at),
        Route::Push => shape::<PushRequest>(body, at),
        Route::PasskeyStart => shape::<PasskeyStartRequest>(body, at),
        Route::PasskeyFinish => shape::<PasskeyFinishRequest>(body, at),
        Route::PasskeySignupStart => shape::<PasskeySignupStartRequest>(body, at),
        Route::PasskeySignupFinish => shape::<PasskeySignupFinishRequest>(body, at),
        Route::PasskeyProofStart => shape::<PasskeyProofStartRequest>(body, at),
        Route::AccountPasskeysStart => shape::<AccountPasskeysStartRequest>(body, at),
        Route::AccountPasskeysFinish => shape::<AccountPasskeysFinishRequest>(body, at),
        Route::AccountPasskeyRemove => shape::<PasskeyProofRequest>(body, at),
        Route::SignInPassword => shape::<SignInPasswordRequest>(body, at),
        Route::ProviderStart => shape::<ProviderStartRequest>(body, at),
        Route::ProviderPoll => shape::<ProviderPollRequest>(body, at),
        Route::SocialSignup => shape::<SocialSignupRequest>(body, at),
        Route::IdentityStart => shape::<IdentityStartRequest>(body, at),
        Route::IdentityUnlink => shape::<UnlinkRequest>(body, at),
        _ => {}
    }
}

/// Checks one fixture file and returns how many request steps it has.
fn check_fixture(name: &str) -> usize {
    let fixture = load(name);
    // sign_in.json leaves `protocol` out.
    if name != "sign_in.json" {
        assert_eq!(fixture["protocol"], 2, "{name}: protocol");
    }
    let start = fixture["start"].as_i64().expect("start");
    assert_eq!(start % 86_400_000, 0, "{name}: start is a UTC midnight");
    let values: BTreeMap<String, Value> =
        serde_json::from_value(fixture["values"].clone()).expect("values");
    check_values(name, &values);

    let mut requests = 0;
    for case in fixture["cases"].as_array().expect("cases") {
        let case_name = case["name"].as_str().expect("case name");
        let worker_case = case.get("runner").is_some_and(|r| r == "worker");
        let mut captured = BTreeSet::new();
        let mut t = 0;
        fn flatten(steps: &[Value], out: &mut Vec<Value>) {
            for step in steps {
                match step.get("repeat") {
                    Some(n) => {
                        let inner = step["steps"].as_array().expect("repeat steps");
                        for _ in 0..n.as_u64().expect("repeat count") {
                            flatten(inner, out);
                        }
                    }
                    None => out.push(step.clone()),
                }
            }
        }
        let mut steps = Vec::new();
        flatten(case["steps"].as_array().expect("steps"), &mut steps);
        for (i, step) in steps.iter().enumerate() {
            let at = format!("{name} › {case_name} › step {}", i + 1);
            if let Some(answer) = step.get("github") {
                assert!(
                    matches!(answer.as_str(), Some("authorized" | "denied" | "slow_down")),
                    "{at}: github"
                );
                continue;
            }
            let kinds: Vec<_> = ["settings", "admin", "request", "raw"]
                .into_iter()
                .filter(|k| step.get(*k).is_some())
                .collect();
            assert_eq!(kinds.len(), 1, "{at}: one kind, got {kinds:?}");
            if let Some(next) = step.get("t") {
                let next = next.as_i64().expect("t");
                assert!(next >= t, "{at}: t goes back");
                t = next;
            }
            if let Some(settings) = step.get("settings") {
                for (setting, value) in settings.as_object().expect("settings") {
                    let value = value.as_str().expect("settings are strings");
                    assert!(
                        is_valid_setting(setting, value),
                        "{at}: D1 refuses {setting} = {value}"
                    );
                }
                continue;
            }
            if let Some(action) = step.get("admin") {
                assert!(
                    matches!(action.as_str(), Some("disable" | "enable" | "quota")),
                    "{at}: admin"
                );
                assert!(step["username"].is_string(), "{at}: admin username");
                continue;
            }
            let status = step["status"].as_u64().expect("status") as u16;
            let worker_only = worker_case || step.get("runner").is_some_and(|r| r == "worker");
            if step.get("raw").is_some() {
                assert!(worker_only, "{at}: raw steps are Worker-only");
            } else {
                requests += 1;
                let Some(route) = route_of(step["request"].as_str().expect("request")) else {
                    continue;
                };
                if let Some(token) = step.get("as") {
                    let token = token.as_str().expect("as");
                    assert!(
                        captured.contains(token),
                        "{at}: token {token} not captured yet"
                    );
                    assert!(route.needs_session(), "{at}: {route:?} takes no session");
                }
                let body = step.get("body").map(|b| substitute(b, &values, &captured));
                if !worker_only {
                    check_body(route, body.as_ref(), &at);
                }
                if route == Route::Push && status == 200 {
                    let response = substitute(&step["response"], &values, &captured);
                    let parsed: PushResponse = serde_json::from_value(response)
                        .unwrap_or_else(|e| panic!("{at}: push results: {e}"));
                    let sent = body.as_ref().expect("push body")["docs"]
                        .as_array()
                        .expect("docs")
                        .len();
                    assert_eq!(parsed.results.len(), sent, "{at}: one result per document");
                }
            }
            // A step may expect the status alone; the runners check it.
            if status >= 400 && step.get("response").is_some() {
                let mut response = step["response"].clone();
                response["message"] = Value::String("fixture".into());
                let error = ApiError::from_response(status, response.to_string().as_bytes());
                assert!(
                    !matches!(error, ApiError::Unrecognized { .. } | ApiError::Protocol(_)),
                    "{at}: {error:?}"
                );
                assert_eq!(
                    error.code(),
                    step["response"]["error"].as_str(),
                    "{at}: code"
                );
                if let ApiError::RateLimited { retry_after, limit } = &error {
                    // sign_in.json's patterns leave the wait out.
                    assert!(
                        *retry_after > 0 || step["response"].get("retry_after").is_none(),
                        "{at}: retry_after"
                    );
                    assert!(
                        [
                            "signup-ip",
                            "login-user",
                            "login-ip",
                            "recover-ip",
                            "password-account",
                            "writes-account",
                            "writes-global",
                            "uploads-account",
                            "uploads-global",
                            "approval-create",
                            "approval-poll",
                            "approval-respond",
                            "provider-poll",
                            "passkey-challenge",
                            "proof-account",
                            "proof-total",
                            "password-total",
                            "passkey-manage"
                        ]
                        .contains(&limit.as_str()),
                        "{at}: limit {limit}"
                    );
                }
            }
            if let Some(capture) = step.get("capture") {
                captured.extend(capture.as_object().expect("capture").keys().cloned());
            }
        }
    }
    requests
}

/// Every named value matches the wire pattern its prefix stands for.
fn check_values(name: &str, values: &BTreeMap<String, Value>) {
    let floor = KdfParams::FLOOR;
    for (key, value) in values {
        let at = format!("{name} › values.{key}");
        if key == "unknown_flow" {
            let text = value.as_str().expect("a flow ID");
            assert!(text.len() == 43 && wire::is_base64url(text), "{at}");
            continue;
        }
        if key == "kdf" {
            let kdf: WireKdf = serde_json::from_value(value.clone()).expect("kdf");
            assert!(kdf.params().is_some_and(|p| p.is_within(&floor)), "{at}");
            continue;
        }
        let text = value
            .as_str()
            .unwrap_or_else(|| panic!("{at}: not a string"));
        let prefix = key.split('_').next().unwrap();
        let ok = match prefix {
            "acct" => wire::is_account_id(text),
            "dev" => wire::is_device_id(text),
            "kid" => wire::is_hex16(text),
            "w" => wire::is_wire_id(text),
            "r" => wire::is_wire_rev(text),
            "s" => wire::is_sealed(text, wire::MIN_SEALED_LEN, wire::MAX_SEALED_LEN),
            "name" => wire::is_sealed_name(text),
            "salt" => text.len() == 22 && wire::is_base64url(text),
            "auth" | "rauth" | "proof" | "commit" | "epk" | "request" => {
                text.len() == 43 && wire::is_base64url(text)
            }
            "wrap" | "rwrap" => text.len() == 98 && wire::is_base64url(text),
            "f" => wire::is_base64url(text),
            "cred" => text.len() == 43 && wire::is_base64url(text),
            "github" => !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()),
            "unknown" => wire::is_token(text),
            other => panic!("{at}: unknown prefix {other}"),
        };
        assert!(ok, "{at}: {text:?} doesn't match its pattern");
    }
}

#[test]
fn auth_fixture_has_the_api_shapes() {
    assert!(check_fixture("auth.json") >= 250);
}

/// `sign_in.json` is the Worker builder's; until it lands this skips, and it
/// must be re-run once it does.
#[test]
fn sign_in_fixture_has_the_api_shapes() {
    let path = manifest_dir().join("../../cloud/test/fixtures/sign_in.json");
    if !path.exists() {
        eprintln!("skipped: cloud/test/fixtures/sign_in.json isn't in this checkout yet");
        return;
    }
    assert!(check_fixture("sign_in.json") >= 150);
}

#[test]
fn conformance_fixture_has_the_api_shapes() {
    assert!(check_fixture("conformance.json") >= 120);
}

/// Every route and every error code the protocol has appears somewhere in the
/// two fixtures, except the ones only a broken deployment produces.
#[test]
fn the_fixtures_cover_every_route_and_error() {
    let mut routes = BTreeSet::new();
    let mut codes = BTreeSet::new();
    for name in ["auth.json", "conformance.json"] {
        let fixture = load(name);
        for case in fixture["cases"].as_array().unwrap() {
            for step in case["steps"].as_array().unwrap() {
                if let Some(route) = step["request"].as_str() {
                    routes.insert(route.to_owned());
                }
                if let Some(code) = step["response"]["error"].as_str() {
                    codes.insert(code.to_owned());
                }
            }
        }
    }
    for route in Route::ALL {
        let name = serde_json::to_value(route).unwrap();
        assert!(
            routes.contains(name.as_str().unwrap()),
            "no step uses {route:?}"
        );
    }
    let untested = ["internal", "misconfigured"];
    let all = [
        "bad_request",
        "invalid_username",
        "reserved_username",
        "unsupported_kdf",
        "weak_kdf",
        "username_mismatch",
        "checksum_mismatch",
        "unauthorized",
        "invalid_credentials",
        "forbidden_origin",
        "account_disabled",
        "signup_closed",
        "not_found",
        "method_not_allowed",
        "username_taken",
        "account_id_taken",
        "self_approval",
        "approval_used",
        "approval_expired",
        "too_large",
        "quota_exceeded",
        "unsupported_media_type",
        "rate_limited",
        "internal",
        "misconfigured",
    ];
    for code in all.iter().filter(|c| !untested.contains(c)) {
        assert!(codes.contains(*code), "no step expects {code}");
    }
}

/// sign_in.json uses every passkey, proof and first-password route.
#[test]
fn the_sign_in_fixture_covers_every_passkey_route() {
    let fixture = load("sign_in.json");
    let mut routes = BTreeSet::new();
    fn walk(steps: &[Value], routes: &mut BTreeSet<Route>) {
        for step in steps {
            if let Some(inner) = step.get("steps").and_then(Value::as_array) {
                walk(inner, routes);
            }
            if let Some(route) = step["request"].as_str().and_then(route_of) {
                routes.insert(route);
            }
        }
    }
    for case in fixture["cases"].as_array().unwrap() {
        walk(case["steps"].as_array().unwrap(), &mut routes);
    }
    for route in Route::PASSKEYS {
        assert!(routes.contains(&route), "no step uses {route:?}");
    }
}
