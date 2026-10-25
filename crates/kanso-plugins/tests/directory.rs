//! The `kanso-plugin build-directory` and `verify-directory` commands, run
//! as the directory's CI runs them, on two plugin folders.

use std::path::Path;
use std::process::{Command, Output};

use ed25519_dalek::SigningKey;
use kanso_plugins::bundle::{self, publisher_key_text};
use kanso_plugins::registry_model::Index;
use kanso_update::feed::{public_key_text, sign_feed};

const NOW: &str = "2026-10-20T10:00:00Z";

fn tool(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kanso-plugin"))
        .args(args)
        .output()
        .unwrap()
}

fn text(out: &[u8]) -> String {
    String::from_utf8_lossy(out).into_owned()
}

/// Writes `plugins/<id>/source/` and the author's signature over its pack.
fn plugin(plugins: &Path, id: &str, version: &str, seed: u8) {
    let key = SigningKey::from_bytes(&[seed; 32]);
    let source = plugins.join(id).join("source");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(
        source.join("manifest.json"),
        format!(
            r#"{{"schema":1,"apiVersion":1,"id":"{id}","name":"Test","version":"{version}",
"entry":"main.js","publisher":{{"name":"Tester","key":"{}"}},"minAppVersion":"0.1.0",
"permissions":{{"commands":true}}}}"#,
            publisher_key_text(&key.verifying_key())
        ),
    )
    .unwrap();
    std::fs::write(source.join("main.js"), format!("// {id} {version}\n")).unwrap();
    let hash = bundle::sha256_hex(&bundle::pack_dir(&source).unwrap());
    std::fs::write(
        plugins.join(id).join("author-signature.txt"),
        format!("{}\n", bundle::sign_hash(&hash, &key)),
    )
    .unwrap();
}

#[test]
fn builds_signs_and_verifies_a_directory() {
    let root = tempfile::tempdir().unwrap();
    let plugins = root.path().join("plugins");
    plugin(&plugins, "com.example.alpha", "1.0.0", 1);
    plugin(&plugins, "com.example.beta", "0.2.0", 2);
    let out = root.path().join("directory");
    let plugins_arg = plugins.to_str().unwrap();
    let out_arg = out.to_str().unwrap();

    let run = tool(&[
        "build-directory",
        "--plugins",
        plugins_arg,
        "--out",
        out_arg,
        "--published-at",
        NOW,
    ]);
    assert!(run.status.success(), "{}", text(&run.stderr));
    let said = text(&run.stdout);
    assert!(said.contains("Added com.example.alpha 1.0.0"), "{said}");
    assert!(said.contains("Added com.example.beta 0.2.0"), "{said}");
    assert!(said.contains("Serial is now 1"), "{said}");

    // Sign it with a test key, as the protected step would.
    let registry = SigningKey::from_bytes(&[90; 32]);
    let index_path = out.join("index.json");
    let bytes = std::fs::read(&index_path).unwrap();
    std::fs::write(out.join("index.json.sig"), sign_feed(&bytes, &registry)).unwrap();
    let key = public_key_text(&registry.verifying_key());

    let ok = tool(&["verify-directory", out_arg, "--key", &key]);
    assert!(ok.status.success(), "{}", text(&ok.stderr));
    assert!(text(&ok.stdout).contains("2 plugin versions"));

    // Building again from what was published changes nothing.
    let again = tool(&[
        "build-directory",
        "--plugins",
        plugins_arg,
        "--out",
        out_arg,
        "--published-at",
        "2026-10-21T10:00:00Z",
        "--previous",
        index_path.to_str().unwrap(),
    ]);
    assert!(again.status.success(), "{}", text(&again.stderr));
    assert!(text(&again.stdout).contains("The directory is unchanged."));

    // A new version raises the serial and keeps the old entry.
    plugin(&plugins, "com.example.alpha", "1.1.0", 1);
    let next = tool(&[
        "build-directory",
        "--plugins",
        plugins_arg,
        "--out",
        out_arg,
        "--published-at",
        "2026-10-21T10:00:00Z",
        "--previous",
        index_path.to_str().unwrap(),
    ]);
    assert!(next.status.success(), "{}", text(&next.stderr));
    assert!(text(&next.stdout).contains("Serial is now 2"));
    let index: Index = serde_json::from_slice(&std::fs::read(&index_path).unwrap()).unwrap();
    assert_eq!(index.plugins.len(), 3);

    // The index changed, so the old signature no longer matches.
    let stale = tool(&["verify-directory", out_arg, "--key", &key]);
    assert!(!stale.status.success());
    let bytes = std::fs::read(&index_path).unwrap();
    std::fs::write(out.join("index.json.sig"), sign_feed(&bytes, &registry)).unwrap();
    let ok = tool(&["verify-directory", out_arg, "--key", &key]);
    assert!(ok.status.success(), "{}", text(&ok.stderr));
}

#[test]
fn refuses_new_contents_at_a_published_version() {
    let root = tempfile::tempdir().unwrap();
    let plugins = root.path().join("plugins");
    plugin(&plugins, "com.example.alpha", "1.0.0", 1);
    let out = root.path().join("directory");
    let args = |extra: &[&str]| {
        let mut v = vec![
            "build-directory".to_owned(),
            "--plugins".to_owned(),
            plugins.to_str().unwrap().to_owned(),
            "--out".to_owned(),
            out.to_str().unwrap().to_owned(),
            "--published-at".to_owned(),
            NOW.to_owned(),
        ];
        v.extend(extra.iter().map(|s| (*s).to_owned()));
        v
    };
    let first = Command::new(env!("CARGO_BIN_EXE_kanso-plugin"))
        .args(args(&[]))
        .output()
        .unwrap();
    assert!(first.status.success());
    std::fs::write(
        plugins.join("com.example.alpha/source/main.js"),
        "// changed\n",
    )
    .unwrap();
    // Re-sign so only the republishing rule can refuse it.
    let key = SigningKey::from_bytes(&[1; 32]);
    let hash =
        bundle::sha256_hex(&bundle::pack_dir(&plugins.join("com.example.alpha/source")).unwrap());
    std::fs::write(
        plugins.join("com.example.alpha/author-signature.txt"),
        bundle::sign_hash(&hash, &key),
    )
    .unwrap();
    let index = out.join("index.json");
    let second = Command::new(env!("CARGO_BIN_EXE_kanso-plugin"))
        .args(args(&["--previous", index.to_str().unwrap()]))
        .output()
        .unwrap();
    assert!(!second.status.success());
    assert!(
        text(&second.stderr).contains("already published with other contents"),
        "{}",
        text(&second.stderr)
    );
}
