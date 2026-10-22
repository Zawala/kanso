//! Imports Tomboy and Gnote XML notes with stable file-based identities.

use std::collections::hash_map::Entry;
use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use quick_xml::{Reader, events::Event};

use super::tree::{Tree, TreeFile, extension, file_name, parent_of};
use super::{Skipped, Source, SourceNote, dates};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct TomboySource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
    titles: HashMap<String, String>,
}

/// Opens one `.note` XML document or a folder of Tomboy/Gnote notes.
pub fn open(path: &Path) -> anyhow::Result<TomboySource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else {
        Tree::Single(path.to_owned())
    };
    let listed = tree.files()?;
    let live: HashSet<String> = listed
        .iter()
        .filter(|file| extension(&file.path) == "note" && !in_backup(&file.path))
        .map(|file| file_name(&file.path).to_owned())
        .collect();
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    // Title -> (in the backup, path): a live note wins over a deleted one,
    // then the first path, whatever order the files come in.
    let mut titles: HashMap<String, (bool, String)> = HashMap::new();
    for file in listed {
        if extension(&file.path) != "note" {
            files.push(file);
            continue;
        }
        if file.size > MAX_NOTE {
            skipped.push(Skipped::new(file.path, "too large for a Tomboy note"));
            continue;
        }
        let backup = in_backup(&file.path);
        if backup && live.contains(file_name(&file.path)) {
            skipped.push(Skipped::new(
                file.path,
                "an older copy of a Tomboy note that is also in the export",
            ));
            continue;
        }
        let raw = tree.read_text(&file.path, MAX_NOTE)?;
        match parse_xml(&raw) {
            Ok(parsed) if parsed.template => {
                skipped.push(Skipped::new(file.path, "a Tomboy note template"));
            }
            Ok(parsed) => {
                let key = file.path.trim_end_matches(".note").to_owned();
                let rank = (backup, key);
                match titles.entry(parsed.title.to_lowercase()) {
                    Entry::Occupied(mut held) if rank < *held.get() => {
                        held.insert(rank);
                    }
                    Entry::Occupied(_) => {}
                    Entry::Vacant(slot) => {
                        slot.insert(rank);
                    }
                }
                notes.push(file);
            }
            Err(err) => skipped.push(Skipped::new(
                file.path,
                format!("invalid Tomboy XML: {err}"),
            )),
        }
    }
    let titles = titles
        .into_iter()
        .map(|(title, (_, key))| (title, key))
        .collect();
    Ok(TomboySource {
        tree,
        notes,
        files,
        skipped,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Tomboy")
            .to_owned(),
        titles,
    })
}

impl Source for TomboySource {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn files(&self) -> &[TreeFile] {
        &self.files
    }
    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for file in self.notes.clone() {
            let xml = self
                .tree
                .read_text(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let parsed = parse_xml(&xml)
                .map_err(|e| anyhow::anyhow!("invalid Tomboy XML in {}: {e}", file.path))?;
            let body = render_body(&parsed.body, &self.titles, &self.files);
            let body = body
                .strip_prefix(&parsed.title)
                .map(str::trim_start)
                .unwrap_or(&body)
                .to_owned();
            let trashed = in_backup(&file.path);
            // Tomboy's notebooks are flat and named by a tag; without one,
            // the note's folders (but not the backup's) stand in for it.
            let folder = match parsed.notebook {
                Some(notebook) => vec![notebook],
                None if trashed => Vec::new(),
                None => parent_of(&file.path)
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
            };
            each(SourceNote {
                key: file.path.trim_end_matches(".note").to_owned(),
                path: Some(file.path.clone()),
                folder,
                title: parsed.title,
                body,
                tags: parsed.tags,
                created: parsed.created,
                updated: parsed.updated,
                trashed,
                ..SourceNote::default()
            })?;
        }
        Ok(())
    }
    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        self.tree.read(path, limit)
    }
    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// Whether a note is in Tomboy's `Backup` folder, where deleted notes go.
fn in_backup(path: &str) -> bool {
    parent_of(path).split('/').any(|folder| folder == "Backup")
}

struct Parsed {
    title: String,
    body: String,
    tags: Vec<String>,
    /// From the `system:notebook:NAME` tag.
    notebook: Option<String>,
    /// Tagged `system:template`: the note new notes start from.
    template: bool,
    created: Option<i64>,
    updated: Option<i64>,
}

fn parse_xml(xml: &str) -> anyhow::Result<Parsed> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut stack: Vec<String> = Vec::new();
    let mut title = String::new();
    let mut body = String::new();
    let mut tags = Vec::new();
    let mut notebook = None;
    let mut template = false;
    let mut created = None;
    let mut updated = None;
    let mut capture = String::new();
    let mut captured_text = String::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let name = local_name(e.name().as_ref());
                if [
                    "title",
                    "note-content",
                    "create-date",
                    "last-change-date",
                    "tag",
                ]
                .contains(&name.as_str())
                {
                    capture = name.clone();
                    captured_text.clear();
                }
                if name == "list-item" {
                    let depth = stack.iter().filter(|tag| tag.as_str() == "list").count();
                    body.push_str(&format!("\n{}- ", "  ".repeat(depth.saturating_sub(1))));
                }
                if name == "list" && !body.ends_with('\n') {
                    body.push('\n');
                }
                if capture == "note-content" {
                    format_open(&mut body, &name, &e);
                }
                stack.push(name);
            }
            Event::Empty(e) => {
                let name = local_name(e.name().as_ref());
                if name == "list-item" {
                    body.push_str("\n- ");
                }
            }
            Event::End(e) => {
                let name = local_name(e.name().as_ref());
                if capture == "note-content" {
                    format_close(&mut body, &name, e.name().as_ref());
                }
                if name == "title" {
                    capture.clear();
                }
                if name == "note-content" {
                    capture.clear();
                }
                if name == "create-date" {
                    created = dates::parse(&captured_text);
                    capture.clear();
                }
                if name == "last-change-date" {
                    updated = dates::parse(&captured_text);
                    capture.clear();
                }
                if name == "tag" {
                    let tag = captured_text.trim();
                    if let Some(name) = tag.strip_prefix("system:notebook:") {
                        if !name.trim().is_empty() && notebook.is_none() {
                            notebook = Some(name.trim().to_owned());
                        }
                    } else if tag == "system:template" {
                        template = true;
                    } else if !tag.is_empty() && !tag.starts_with("system:") {
                        tags.push(tag.to_owned());
                    }
                    capture.clear();
                }
                let _ = stack.pop();
            }
            Event::Text(e) => {
                let text = e.decode()?.into_owned();
                match capture.as_str() {
                    "title" => title.push_str(&text),
                    "note-content" => body.push_str(&text),
                    _ => captured_text.push_str(&text),
                }
            }
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref());
                if capture == "note-content" {
                    body.push_str(&text);
                } else if capture == "title" {
                    title.push_str(&text);
                } else {
                    captured_text.push_str(&text);
                }
            }
            Event::GeneralRef(e) => {
                let name = e.as_ref();
                let character = match name {
                    b"amp" => Some('&'),
                    b"lt" => Some('<'),
                    b"gt" => Some('>'),
                    b"apos" => Some('\''),
                    b"quot" => Some('"'),
                    _ => e.resolve_char_ref()?,
                };
                if let Some(character) = character {
                    match capture.as_str() {
                        "title" => title.push(character),
                        "note-content" => body.push(character),
                        _ => captured_text.push(character),
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    anyhow::ensure!(!title.trim().is_empty(), "missing note title");
    Ok(Parsed {
        title: title.trim().to_owned(),
        body: body.trim().to_owned(),
        tags,
        notebook,
        template,
        created,
        updated,
    })
}

fn local_name(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .rsplit(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn format_open(body: &mut String, name: &str, _event: &quick_xml::events::BytesStart<'_>) {
    match name {
        "bold" => body.push_str("**"),
        "italic" => body.push('*'),
        "strikethrough" => body.push_str("~~"),
        "highlight" => body.push_str("=="),
        "monospace" => body.push('`'),
        "internal" => body.push_str("<link:internal>"),
        "url" => body.push_str("<link:url>"),
        "list-item" => {}
        _ => {}
    }
}
fn format_close(body: &mut String, name: &str, qualified: &[u8]) {
    match name {
        "bold" => body.push_str("**"),
        "italic" => body.push('*'),
        "strikethrough" => body.push_str("~~"),
        "highlight" => body.push_str("=="),
        "monospace" => body.push('`'),
        "internal" if qualified == b"link:internal" => body.push_str("</link:internal>"),
        "url" if qualified == b"link:url" => body.push_str("</link:url>"),
        "list-item" => body.push('\n'),
        _ => {}
    }
}

fn render_body(raw: &str, titles: &HashMap<String, String>, files: &[TreeFile]) -> String {
    let mut out = raw.to_owned();
    let mut cursor = 0;
    while let Some(rel) = out[cursor..].find("<link:internal>") {
        let at = cursor + rel;
        let Some(gt_rel) = out[at..].find('>') else {
            break;
        };
        let gt = at + gt_rel;
        let Some(end_rel) = out[gt..].find("</link:internal>") else {
            break;
        };
        let end = gt + end_rel;
        let title = out[gt + 1..end].trim().to_owned();
        let replacement = titles
            .get(&title.to_lowercase())
            .map(|key| format!("[{title}](kanso-import:note/{key})"))
            .unwrap_or(title);
        out.replace_range(at..end + 16, &replacement);
        cursor = at + replacement.len();
    }
    cursor = 0;
    while let Some(rel) = out[cursor..].find("<link:url>") {
        let at = cursor + rel;
        let Some(start) = out[at..].find('>').map(|offset| at + offset + 1) else {
            break;
        };
        let Some(end) = out[start..]
            .find("</link:url>")
            .map(|offset| start + offset)
        else {
            break;
        };
        let url = out[start..end].trim().to_owned();
        let local = url.strip_prefix("file://").unwrap_or(&url);
        let basename = local.rsplit(['/', '\\']).next().unwrap_or(local);
        let replacement = files
            .iter()
            .find(|file| file.path == local || super::tree::file_name(&file.path) == basename)
            .map(|file| format!("[{}](kanso-import:path/{})", basename, file.path))
            .unwrap_or(url);
        out.replace_range(at..end + "</link:url>".len(), &replacement);
        cursor = at + replacement.len();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_tomboy_formatting_dates_and_notebook() {
        let xml = r#"<note><title>Example</title><text><note-content>Line <bold>strong</bold>, <italic>em</italic>, <strikethrough>gone</strikethrough>, <highlight>mark</highlight>, <monospace>code</monospace><list><list-item>one</list-item></list></note-content></text><create-date>2024-01-01T00:00:00Z</create-date><last-change-date>2024-01-02T00:00:00Z</last-change-date><tags><tag>system:notebook:Work</tag></tags></note>"#;
        let note = parse_xml(xml).unwrap();
        assert!(note.body.contains("**strong**"));
        assert!(note.body.contains("==mark=="));
        assert_eq!(note.notebook.as_deref(), Some("Work"));
        assert!(note.tags.is_empty() && !note.template);
        assert!(note.created.is_some());
    }
    #[test]
    fn malformed_xml_is_rejected() {
        assert!(parse_xml("<note>").is_err());
    }

    #[test]
    fn resolves_internal_links_by_title() {
        let titles = HashMap::from([("other".to_owned(), "id-2".to_owned())]);
        assert_eq!(
            render_body("See <link:internal>Other</link:internal>", &titles, &[]),
            "See [Other](kanso-import:note/id-2)"
        );
    }
}
