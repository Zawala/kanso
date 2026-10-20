//! `kanso capture "text"`: saves a note to the Inbox from a terminal, a
//! script or a desktop shortcut. With no text (or `-`) it reads standard
//! input, so `pbpaste | kanso capture` works.
//!
//! While Kanso is open it holds the data folder, so the note goes through
//! its local API and shows at once; otherwise it is written to the store
//! directly. The title is the text's first line (`capture::split_capture`).

use std::io::Read as _;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use kanso_core::roles::{ROLE_FIELD, Role};
use kanso_store::{Origin, Store, lock_data_dir};
use serde_json::{Value, json};

use crate::capture::{save_capture, split_capture};
use crate::config::ServerSettings;

pub const USAGE: &str = "\
Usage: kanso capture [text…]

Saves a note to the Inbox. The first line is its title. With no text, or
`-`, the text is read from standard input.";

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let text = if rest.is_empty() || rest == ["-"] {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("reading standard input")?;
        text
    } else {
        rest.join(" ")
    };
    let Some((title, _)) = split_capture(&text) else {
        bail!("nothing to capture: the text is empty\n\n{USAGE}");
    };
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    match lock_data_dir(data_dir)? {
        Some(_lock) => {
            let store = Store::open(crate::paths::db_path(data_dir))?;
            save_capture(&store, &text, None, Origin::Api)?;
        }
        None => {
            let config = crate::load_config(data_dir)?;
            if !config.server.enabled {
                bail!(
                    "Kanso is open and its local API is off. Turn it on in Settings › Local API, or quit Kanso, to capture from here."
                );
            }
            Api::new(&config.server)?.capture(&text)?;
        }
    }
    println!("Saved “{title}” to the Inbox.");
    Ok(())
}

/// A running Kanso's local API.
pub struct Api {
    base: String,
    auth: String,
    agent: ureq::Agent,
}

impl Api {
    pub fn new(settings: &ServerSettings) -> Result<Api> {
        let ip: IpAddr = settings
            .bind_address
            .trim()
            .parse()
            .with_context(|| format!("“{}” isn't an IP address", settings.bind_address))?;
        // Listening everywhere answers on this machine's own address too.
        let ip = match ip {
            IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
            ip => ip,
        };
        let host = match ip {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => format!("[{ip}]"),
        };
        Ok(Api::at(
            &format!("http://{host}:{}", settings.port),
            &settings.username,
            &settings.password,
        ))
    }

    pub fn at(base: &str, username: &str, password: &str) -> Api {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(20)))
            .build()
            .into();
        Api {
            base: base.trim_end_matches('/').to_owned(),
            auth: kanso_server::basic_header(username, password),
            agent,
        }
    }

    fn call(&self, method: &str, path: &str, body: Option<&Value>) -> Result<(u16, Value)> {
        let url = format!("{}{path}", self.base);
        let response = match body {
            None => self
                .agent
                .get(&url)
                .header("authorization", &self.auth)
                .call(),
            Some(body) => {
                debug_assert_eq!(method, "POST");
                self.agent
                    .post(&url)
                    .header("authorization", &self.auth)
                    .header("content-type", "application/json")
                    .send(serde_json::to_vec(body)?.as_slice())
            }
        };
        let mut response = response
            .with_context(|| format!("couldn't reach Kanso's local API at {}", self.base))?;
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap_or_default();
        if status == 401 {
            bail!("Kanso's local API didn't accept the username and password in its settings");
        }
        Ok((status, serde_json::from_str(&text).unwrap_or(Value::Null)))
    }

    /// The Inbox's ID, made through the API when there is none.
    fn inbox(&self) -> Result<String> {
        let (status, books) = self.call("GET", "/books", None)?;
        if status != 200 {
            bail!("Kanso's local API answered {status} for the notebooks");
        }
        let role = Role::Inbox;
        let found = books.as_array().and_then(|books| {
            let id = |book: &Value| book["_id"].as_str().map(str::to_owned);
            books
                .iter()
                .find(|b| b["_id"] == role.book_id())
                .or_else(|| books.iter().find(|b| b[ROLE_FIELD] == role.as_str()))
                .and_then(id)
        });
        if let Some(id) = found {
            return Ok(id);
        }
        let book = json!({
            "_id": role.book_id(),
            "name": role.default_name(),
            ROLE_FIELD: role.as_str(),
        });
        let (status, made) = self.call("POST", "/books", Some(&book))?;
        match made["id"].as_str() {
            Some(id) if status == 200 => Ok(id.to_owned()),
            _ => bail!("Kanso's local API couldn't make the Inbox ({status})"),
        }
    }

    /// Saves `text` as a note in the Inbox; returns its ID.
    pub fn capture(&self, text: &str) -> Result<String> {
        let Some((title, body)) = split_capture(text) else {
            bail!("nothing to capture");
        };
        let note = json!({"title": title, "body": body, "bookId": self.inbox()?});
        let (status, made) = self.call("POST", "/notes", Some(&note))?;
        match made["id"].as_str() {
            Some(id) if status == 200 => Ok(id.to_owned()),
            _ => bail!("Kanso's local API couldn't save the note ({status}): {made}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::Api;
    use kanso_store::Store;

    /// Through a running API on an ephemeral port: the Inbox is made once,
    /// and both notes land in it.
    #[test]
    fn captures_through_the_local_api() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let handle = kanso_server::spawn(
            store.clone(),
            kanso_server::ServerConfig {
                bind: "127.0.0.1:0".parse().unwrap(),
                username: "u".into(),
                password: "p".into(),
            },
        );
        let addr = handle.bind_result().unwrap();
        let api = Api::at(&format!("http://{addr}"), "u", "p");
        let first = api.capture("Buy milk\nand eggs").unwrap();
        let second = api.capture("Call Sam").unwrap();
        let note = store.note(&first).unwrap().unwrap();
        assert_eq!(
            (note.title.as_str(), note.body.as_str()),
            ("Buy milk", "and eggs")
        );
        assert_eq!(note.book_id, "book:inbox");
        assert_eq!(store.note(&second).unwrap().unwrap().book_id, "book:inbox");
        assert_eq!(store.books().unwrap().len(), 1);
        let wrong = Api::at(&format!("http://{addr}"), "u", "nope");
        assert!(
            wrong
                .capture("x")
                .unwrap_err()
                .to_string()
                .contains("didn't accept")
        );
    }

    /// Without the app: straight into the store, the Inbox made on the way.
    #[test]
    fn captures_into_the_store_without_the_app() {
        let dir = tempfile::tempdir().unwrap();
        super::run_cli(dir.path(), &["Idea:".into(), "kites".into()]).unwrap();
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        let books = store.books().unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "book:inbox");
        assert!(super::run_cli(dir.path(), &[" ".into()]).is_err());
    }
}
