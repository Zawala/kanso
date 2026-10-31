//! `kanso sticky "text"`: saves a note to the Stickies notebook (or the
//! one `--book` names) and shows it as a sticky note (`sticky.rs`). With no
//! text (or `-`) it reads standard input, so `docker ps | kanso sticky`
//! works.
//!
//! The running Kanso makes it, over the same loopback link that hands it
//! files (`url_callback.rs`), so the local API needn't be on. When Kanso
//! isn't running this starts it and hands it the note once it listens; if
//! it doesn't start, the note is saved straight to the store and opens as
//! a sticky the next time Kanso does.

use std::io::Read as _;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail};
use kanso_store::{Origin, Store, lock_data_dir};

use crate::capture::split_capture;
use crate::config::{NoteWindowFrame, StickyColor};
use crate::url_callback::{ForwardError, MAX_STICKY_TEXT, StickyRequest};

pub const USAGE: &str = "\
Usage: kanso sticky [--book <name or id>] [--color <colour>] [text…]

Saves a note to the Stickies notebook, or to the notebook --book names, and
shows it as a sticky note that stays above other windows. The first line is
its title. With no text, or `-`, the text is read from standard input.
Colours: yellow, blue, green, pink, purple, gray.";

/// How long a Kanso started here gets to start listening.
const START_WAIT: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(250);

/// `--color <name>` taken out of the arguments.
fn color_arg(rest: &[String]) -> Result<(Option<StickyColor>, Vec<String>)> {
    let mut color = None;
    let mut words = Vec::new();
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        let value = if arg == "--color" || arg == "--colour" {
            Some(args.next().context("--color needs a colour")?.clone())
        } else {
            arg.strip_prefix("--color=")
                .or_else(|| arg.strip_prefix("--colour="))
                .map(str::to_owned)
        };
        match value {
            Some(value) => {
                color = Some(
                    StickyColor::parse(&value)
                        .with_context(|| format!("“{value}” isn't a sticky colour\n\n{USAGE}"))?,
                );
            }
            None => words.push(arg.clone()),
        }
    }
    Ok((color, words))
}

/// What `kanso sticky` was asked for: the text read, the notebook and the
/// colour.
fn request(rest: &[String]) -> Result<StickyRequest> {
    let (book, rest) = crate::capture_cli::book_arg(rest)?;
    let (color, rest) = color_arg(&rest)?;
    let text = if rest.is_empty() || rest == ["-"] {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("reading standard input")?;
        text
    } else {
        rest.join(" ")
    };
    if split_capture(&text).is_none() {
        bail!("nothing to stick: the text is empty\n\n{USAGE}");
    }
    if text.len() > MAX_STICKY_TEXT {
        bail!(
            "that's more than a sticky note holds ({} KB at most); `kanso capture` saves longer notes",
            MAX_STICKY_TEXT / 1024
        );
    }
    Ok(StickyRequest { text, book, color })
}

pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    if rest.iter().any(|arg| arg == "help" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }
    let request = request(rest)?;
    let title = split_capture(&request.text)
        .map(|(title, _)| title)
        .unwrap_or_default();
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    // Not running, Kanso is started and handed the note once it listens;
    // starting, it is asked again until it makes notes.
    let started = Instant::now();
    let mut launched = false;
    while started.elapsed() < START_WAIT {
        match crate::url_callback::forward_sticky(data_dir, &request) {
            Ok(()) => {
                println!("Stuck “{title}” on the screen.");
                return Ok(());
            }
            Err(ForwardError::NotRunning) if !launched => {
                if start_kanso(data_dir).is_err() {
                    break;
                }
                launched = true;
            }
            Err(ForwardError::NotRunning | ForwardError::Refused) => {}
            Err(error) => bail!("Kanso didn't make the sticky note: {error}"),
        }
        std::thread::sleep(POLL);
    }
    // Kanso didn't start: saved here, to open with it next time.
    let place = save_for_later(data_dir, &request)?;
    println!("Saved “{title}” to {place}; it opens as a sticky note when Kanso starts.");
    Ok(())
}

/// Starts the app on `data_dir` in the background, on its own.
fn start_kanso(data_dir: &Path) -> Result<()> {
    let exe = std::env::current_exe().context("finding Kanso's program")?;
    std::process::Command::new(exe)
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("starting Kanso")?;
    Ok(())
}

/// Saves the sticky straight to the store, and puts it among the windows
/// Kanso opens at launch. Returns where it went.
fn save_for_later(data_dir: &Path, request: &StickyRequest) -> Result<String> {
    let Some(_lock) = lock_data_dir(data_dir)? else {
        bail!("Kanso is starting but isn't answering yet; try again in a moment");
    };
    let store = Store::open(crate::paths::db_path(data_dir))?;
    let (id, place) = save(&store, request, Origin::Api)?;
    let path = crate::paths::config_path(data_dir);
    let mut config = crate::load_config(data_dir)?;
    config.layout.note_windows.push(NoteWindowFrame {
        note_id: id,
        // Off every display: the sticky is placed on the main one.
        x: -100_000.,
        y: -100_000.,
        width: crate::sticky::SIZE.0,
        height: crate::sticky::SIZE.1,
        sticky: true,
        color: request.color.unwrap_or_default(),
    });
    config.save(&path)?;
    Ok(place)
}

/// Saves `request` as a note in its notebook (by ID, else by name), or in
/// the Stickies notebook. Returns the note's ID and where it went.
pub fn save(store: &Store, request: &StickyRequest, origin: Origin) -> Result<(String, String)> {
    let (book, place) = match &request.book {
        Some(wanted) => {
            let books: Vec<(String, String)> =
                store.books()?.into_iter().map(|b| (b.id, b.name)).collect();
            let (id, name) =
                crate::capture_cli::pick_book(&books, wanted).map_err(anyhow::Error::msg)?;
            (Some(id.clone()), name.clone())
        }
        None => (None, crate::sticky::BOOK_NAME.to_owned()),
    };
    let id = crate::sticky::save_sticky(store, &request.text, book.as_deref(), origin)?
        .context("nothing to stick: the text is empty")?;
    Ok((id, place))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn reads_the_book_colour_and_text() {
        let asked = request(&args(&["--book", "Work", "--color=pink", "docker", "ps"])).unwrap();
        assert_eq!(
            asked,
            StickyRequest {
                text: "docker ps".into(),
                book: Some("Work".into()),
                color: Some(StickyColor::Pink),
            }
        );
        assert!(request(&args(&["--color", "teal", "x"])).is_err());
        assert!(request(&args(&["   "])).is_err());
        let long = "x".repeat(MAX_STICKY_TEXT + 1);
        assert!(request(&args(&[long.as_str()])).is_err());
    }

    #[test]
    fn saves_to_stickies_or_the_book_named() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let sticky = |book: Option<&str>| StickyRequest {
            text: "Deploy\nmake ship".into(),
            book: book.map(str::to_owned),
            color: None,
        };
        let (id, place) = save(&store, &sticky(None), Origin::Api).unwrap();
        assert_eq!(place, "Stickies");
        let note = store.note(&id).unwrap().unwrap();
        assert_eq!(
            (note.title.as_str(), note.body.as_str()),
            ("Deploy", "make ship")
        );
        assert!(save(&store, &sticky(Some("Nope")), Origin::Api).is_err());
        let (other, place) = save(&store, &sticky(Some("stickies")), Origin::Api).unwrap();
        assert_eq!(place, "Stickies");
        assert_eq!(store.note(&other).unwrap().unwrap().book_id, note.book_id);
    }

    /// Kanso closed and unable to start (the test's program isn't Kanso):
    /// the note waits in the store, and its sticky among the windows to
    /// open.
    #[test]
    fn without_kanso_the_sticky_waits_for_the_next_launch() {
        let dir = tempfile::tempdir().unwrap();
        let place = save_for_later(
            dir.path(),
            &StickyRequest {
                text: "Remember the milk".into(),
                book: None,
                color: Some(StickyColor::Green),
            },
        )
        .unwrap();
        assert_eq!(place, "Stickies");
        let config = crate::load_config(dir.path()).unwrap();
        let [frame] = config.layout.note_windows.as_slice() else {
            panic!("one sticky to open: {:?}", config.layout.note_windows);
        };
        assert!(frame.sticky);
        assert_eq!(frame.color, StickyColor::Green);
        let store = Store::open(crate::paths::db_path(dir.path())).unwrap();
        assert!(store.note(&frame.note_id).unwrap().is_some());
    }
}
