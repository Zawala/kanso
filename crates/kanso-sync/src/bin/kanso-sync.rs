//! Manual sync runs.
//!
//! ```text
//! kanso-sync --data-dir DIR --endpoint URL [--full]   sync once
//! kanso-sync check --endpoint URL                     check the endpoint and token
//! kanso-sync token set                                store a token read from stdin in the Keychain
//! kanso-sync token delete                             remove it
//! ```
//!
//! The token comes from `KANSO_SYNC_TOKEN` or, failing that, the Keychain.

// no-prop-test: argument plumbing around `sync_once`, which has its own tests.

use std::io::Read as _;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use kanso_store::Store;
use kanso_sync::{HttpRemote, Remote, SyncConfig, credentials, reset_cursor, sync_once};

const USAGE: &str = "usage:
  kanso-sync --data-dir DIR --endpoint URL [--full]
  kanso-sync check --endpoint URL
  kanso-sync token set        (reads the token from stdin)
  kanso-sync token delete";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("kanso-sync: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("token") => match args.get(1).map(String::as_str) {
            Some("set") => {
                let mut token = String::new();
                std::io::stdin().read_to_string(&mut token)?;
                let token = token.trim();
                if token.is_empty() {
                    bail!("no token on stdin");
                }
                credentials::save_token(token)?;
                println!("Saved the sync token in the Keychain.");
                Ok(())
            }
            Some("delete") => {
                credentials::delete_token()?;
                println!("Removed the sync token from the Keychain.");
                Ok(())
            }
            _ => bail!("{USAGE}"),
        },
        Some("check") => {
            let flags = Flags::parse(&args[1..])?;
            let remote = remote(&flags)?;
            remote.health().context("health check")?;
            remote.changes(0, 1).context("authenticated request")?;
            println!("{} is reachable and accepts the token.", remote.endpoint());
            Ok(())
        }
        Some("-h" | "--help") | None => {
            println!("{USAGE}");
            Ok(())
        }
        Some(_) => {
            let flags = Flags::parse(&args)?;
            let data_dir = flags
                .data_dir
                .clone()
                .ok_or_else(|| anyhow!("--data-dir is required\n{USAGE}"))?;
            let remote = remote(&flags)?;
            let _lock = kanso_store::lock_data_dir(&data_dir)?
                .ok_or_else(|| anyhow!("{} is in use by Kanso", data_dir.display()))?;
            let store = Store::open(data_dir.join("kanso.db"))?;
            if flags.full {
                reset_cursor(&store)?;
            }
            let report = sync_once(&store, &remote)?;
            println!("{report:#?}");
            Ok(())
        }
    }
}

#[derive(Default)]
struct Flags {
    data_dir: Option<PathBuf>,
    endpoint: Option<String>,
    full: bool,
}

impl Flags {
    fn parse(args: &[String]) -> Result<Flags> {
        let mut flags = Flags::default();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--data-dir" => {
                    flags.data_dir = Some(iter.next().context("--data-dir needs a value")?.into())
                }
                "--endpoint" => {
                    flags.endpoint = Some(iter.next().context("--endpoint needs a value")?.clone())
                }
                "--full" => flags.full = true,
                other => bail!("unknown argument {other}\n{USAGE}"),
            }
        }
        Ok(flags)
    }
}

fn remote(flags: &Flags) -> Result<HttpRemote> {
    let endpoint = flags
        .endpoint
        .as_deref()
        .ok_or_else(|| anyhow!("--endpoint is required\n{USAGE}"))?;
    let token = match std::env::var("KANSO_SYNC_TOKEN") {
        Ok(token) if !token.trim().is_empty() => token.trim().to_owned(),
        _ => credentials::load_token()?.ok_or_else(|| {
            anyhow!("no sync token: set KANSO_SYNC_TOKEN or run `kanso-sync token set`")
        })?,
    };
    Ok(SyncConfig::new(endpoint)
        .map_err(|err| anyhow!("--endpoint: {err}"))?
        .remote(token))
}
