//! `kanso-plugin`: what a plugin author and the directory's CI use to pack,
//! sign and publish plugins.
//!
//! ```text
//! kanso-plugin keygen --secret-out <file>
//! kanso-plugin pack <dir> [-o <file>]
//! kanso-plugin sign <bundle>
//! kanso-plugin index-add <index.json> <bundle> --signature <b64> --published-at <rfc3339>
//! kanso-plugin index-sign <index.json> [--allow-untrusted]
//! kanso-plugin verify <index.json> <index.json.sig> [--key <b64>]...
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kanso_plugins::bundle::{self, publisher_key_text};
use kanso_plugins::registry::{self, check_index, validate_index};
use kanso_plugins::registry_model::{Entry, INDEX_SCHEMA, Index};
use kanso_update::feed::{generate_signing_key, parse_public_key, parse_signing_key, sign_feed};

const AUTHOR_KEY_ENV: &str = "KANSO_PLUGIN_AUTHOR_KEY";
const REGISTRY_KEY_ENV: &str = "KANSO_PLUGIN_REGISTRY_KEY";

const USAGE: &str = "\
Usage:
  kanso-plugin keygen --secret-out <file>
      Makes a signing key. Writes its secret (base64) to <file>, readable
      only by you, and prints the public keys: the `ed25519:` form for a
      manifest's publisher.key and the base64 form for the directory's
      keys/registry-keys.txt.
  kanso-plugin pack <dir> [-o <file>]
      Checks the folder as the app would, then writes the deterministic
      .kansoplugin (the same files always give the same bytes) and prints
      its SHA-256.
  kanso-plugin sign <bundle>
      Signs the bundle's SHA-256 with the base64 secret in
      $KANSO_PLUGIN_AUTHOR_KEY and prints the signature. The key must be the
      manifest's publisher.key.
  kanso-plugin index-add <index.json> <bundle> --signature <b64>
                         --published-at <rfc3339>
      Adds (or replaces) the bundle's entry, built from its manifest, and
      raises the index's serial. Makes the index if there isn't one.
  kanso-plugin index-sign <index.json> [--allow-untrusted]
      Signs the index with the base64 secret in $KANSO_PLUGIN_REGISTRY_KEY,
      writing <index.json>.sig. Refuses a key the app doesn't trust.
  kanso-plugin verify <index.json> <index.json.sig> [--key <base64>]...
      Checks the signature and every entry against the given keys, or the
      app's own.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = Args::parse(args.get(1..).unwrap_or_default());
    let result = match args.first().map(String::as_str) {
        Some("keygen") => keygen(&rest),
        Some("pack") => pack(&rest),
        Some("sign") => sign(&rest),
        Some("index-add") => index_add(&rest),
        Some("index-sign") => index_sign(&rest),
        Some("verify") => verify(&rest),
        Some("help" | "-h" | "--help") => {
            print!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("no command given\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("kanso-plugin: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Words that aren't flags, `--name value` pairs and lone `--switch`es.
struct Args {
    positional: Vec<String>,
    values: BTreeMap<String, Vec<String>>,
}

impl Args {
    fn parse(args: &[String]) -> Args {
        let mut positional = Vec::new();
        let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut i = 0;
        while i < args.len() {
            let name = match args[i].as_str() {
                "-o" => Some("out"),
                other => other.strip_prefix("--"),
            };
            let Some(name) = name else {
                positional.push(args[i].clone());
                i += 1;
                continue;
            };
            match args.get(i + 1).filter(|v| !v.starts_with('-')) {
                Some(value) => {
                    values
                        .entry(name.to_owned())
                        .or_default()
                        .push(value.clone());
                    i += 2;
                }
                None => {
                    values.entry(name.to_owned()).or_default();
                    i += 1;
                }
            }
        }
        Args { positional, values }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .and_then(|v| v.last())
            .map(String::as_str)
    }

    fn need(&self, name: &str) -> Result<&str, String> {
        self.get(name)
            .ok_or_else(|| format!("--{name} is needed\n\n{USAGE}"))
    }

    fn all(&self, name: &str) -> &[String] {
        self.values.get(name).map_or(&[], Vec::as_slice)
    }

    fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    fn positional(&self, at: usize, what: &str) -> Result<&str, String> {
        self.positional
            .get(at)
            .map(String::as_str)
            .ok_or_else(|| format!("{what} is needed\n\n{USAGE}"))
    }
}

fn keygen(args: &Args) -> Result<(), String> {
    let out = PathBuf::from(args.need("secret-out")?);
    let (key, secret) = generate_signing_key();
    write_secret(&out, &secret).map_err(|e| format!("writing {}: {e}", out.display()))?;
    let public = key.verifying_key();
    println!("Secret key written to {} (keep it private).", out.display());
    println!("Author key, for a manifest's publisher.key:");
    println!("{}", publisher_key_text(&public));
    println!("Directory key, for keys/registry-keys.txt:");
    println!("{}", kanso_update::feed::public_key_text(&public));
    Ok(())
}

/// Writes a new file only its owner can read; never overwrites one.
fn write_secret(path: &Path, secret: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(secret.as_bytes())?;
    file.write_all(b"\n")
}

fn pack(args: &Args) -> Result<(), String> {
    let dir = PathBuf::from(args.positional(0, "the folder to pack")?);
    let opened = bundle::open_dir(&dir).map_err(|e| e.to_string())?;
    let bytes = bundle::pack_dir(&dir).map_err(|e| e.to_string())?;
    let out = args.get("out").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "{}-{}.kansoplugin",
            opened.manifest.id, opened.manifest.version
        ))
    });
    std::fs::write(&out, &bytes).map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("Wrote {}", out.display());
    println!("sha256 {}", bundle::sha256_hex(&bytes));
    Ok(())
}

/// A secret key from an environment variable, never from an argument.
fn secret_from_env(var: &str) -> Result<ed25519_dalek::SigningKey, String> {
    let secret = std::env::var(var).unwrap_or_default();
    if secret.trim().is_empty() {
        return Err(format!("{var} is empty or not set"));
    }
    parse_signing_key(&secret).map_err(|_| format!("{var} isn't a base64 Ed25519 secret key"))
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))
}

fn sign(args: &Args) -> Result<(), String> {
    let path = PathBuf::from(args.positional(0, "the bundle")?);
    let bytes = read(&path)?;
    let opened = bundle::open(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let key = secret_from_env(AUTHOR_KEY_ENV)?;
    if publisher_key_text(&key.verifying_key()) != opened.manifest.publisher.key {
        return Err(format!(
            "the key in {AUTHOR_KEY_ENV} isn't the manifest's publisher.key, so the app would refuse this signature"
        ));
    }
    println!("{}", bundle::sign_hash(&opened.sha256, &key));
    Ok(())
}

fn index_add(args: &Args) -> Result<(), String> {
    let index_path = PathBuf::from(args.positional(0, "the index file")?);
    let bundle_path = PathBuf::from(args.positional(1, "the bundle")?);
    let signature = args.need("signature")?.to_owned();
    let published_at = args.need("published-at")?.to_owned();
    let bytes = read(&bundle_path)?;
    let opened = bundle::open(&bytes).map_err(|e| format!("{}: {e}", bundle_path.display()))?;
    let m = &opened.manifest;
    bundle::verify_hash(&opened.sha256, &signature, &m.publisher.key)
        .map_err(|_| "the signature isn't the manifest's author's, over this bundle".to_owned())?;
    let mut index = if index_path.exists() {
        serde_json::from_slice::<Index>(&read(&index_path)?)
            .map_err(|e| format!("{} isn't an index: {e}", index_path.display()))?
    } else {
        Index {
            schema: INDEX_SCHEMA,
            serial: 0,
            plugins: Vec::new(),
            revoked: Vec::new(),
        }
    };
    let entry = Entry {
        id: m.id.clone(),
        name: m.name.clone(),
        publisher: m.publisher.name.clone(),
        description: m.description.clone().unwrap_or_default(),
        version: m.version.clone(),
        sha256: opened.sha256.clone(),
        size: bytes.len() as u64,
        signer: m.publisher.key.clone(),
        author_signature: signature,
        min_app_version: m.min_app_version.clone(),
        permissions: m.permissions.clone(),
        published_at,
        revoked: false,
    };
    index
        .plugins
        .retain(|e| !(e.id == entry.id && e.version == entry.version));
    println!("Added {} {}", entry.id, entry.version);
    index.plugins.push(entry);
    index
        .plugins
        .sort_by(|a, b| (&a.id, &a.version).cmp(&(&b.id, &b.version)));
    index.serial += 1;
    validate_index(&index).map_err(|e| e.to_string())?;
    let mut out = serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?;
    out.push(b'\n');
    std::fs::write(&index_path, out)
        .map_err(|e| format!("writing {}: {e}", index_path.display()))?;
    println!("Serial is now {}", index.serial);
    Ok(())
}

fn sig_path(index: &Path) -> PathBuf {
    let mut name = index.file_name().unwrap_or_default().to_os_string();
    name.push(".sig");
    index.with_file_name(name)
}

fn index_sign(args: &Args) -> Result<(), String> {
    let path = PathBuf::from(args.positional(0, "the index file")?);
    let key = secret_from_env(REGISTRY_KEY_ENV)?;
    let public = key.verifying_key();
    if !args.has("allow-untrusted") && !registry::trusted_keys().contains(&public) {
        return Err(format!(
            "the key in {REGISTRY_KEY_ENV} (public {}) isn't in crates/kanso-plugins/keys/registry-keys.txt, so this build's app would refuse the directory",
            kanso_update::feed::public_key_text(&public)
        ));
    }
    let bytes = read(&path)?;
    let index = serde_json::from_slice::<Index>(&bytes)
        .map_err(|e| format!("{} isn't an index: {e}", path.display()))?;
    validate_index(&index).map_err(|e| e.to_string())?;
    let out = sig_path(&path);
    std::fs::write(&out, sign_feed(&bytes, &key))
        .map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("Wrote {}", out.display());
    Ok(())
}

fn verify(args: &Args) -> Result<(), String> {
    let path = PathBuf::from(args.positional(0, "the index file")?);
    let sig = args
        .positional
        .get(1)
        .map_or_else(|| sig_path(&path), PathBuf::from);
    let keys = if args.all("key").is_empty() {
        registry::trusted_keys()
    } else {
        args.all("key")
            .iter()
            .map(|k| parse_public_key(k).map_err(|_| format!("--key {k} isn't a public key")))
            .collect::<Result<Vec<_>, _>>()?
    };
    let index = check_index(&read(&path)?, &read(&sig)?, &keys, None).map_err(|e| e.to_string())?;
    println!(
        "Signature OK: serial {}, {} plugin versions, {} revocations",
        index.serial,
        index.plugins.len(),
        index.revoked.len()
    );
    Ok(())
}
