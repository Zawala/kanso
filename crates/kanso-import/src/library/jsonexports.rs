//! Exports that are one JSON file: Standard Notes' decrypted backups and
//! Simplenote's `notes.json`, either on its own or in the `.zip` it came
//! in. These are text, so the file is read whole and its notes kept in
//! memory; neither carries attachments.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::path::Path;

use anyhow::Context as _;
use serde::Deserialize;
use serde_json::Value;

use crate::library::tree::{Tree, TreeFile, extension, file_name, text_of};
use crate::library::{Skipped, Source, SourceNote, dates};

/// The largest JSON file read: backups are text, but can be large.
const MAX_JSON_FILE: u64 = 512 * 1024 * 1024;

/// The notebook archived Standard Notes notes go in.
pub const ARCHIVED: &str = "Archived";

const ENCRYPTED: &str =
    "This backup is encrypted; in Standard Notes choose Export › Decrypted backup.";

/// Notes read from a JSON export, given out in their order in the file.
pub struct JsonSource {
    name: String,
    notes: Vec<SourceNote>,
    skipped: Vec<Skipped>,
}

impl Source for JsonSource {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn files(&self) -> &[TreeFile] {
        &[]
    }

    fn notes(
        &mut self,
        full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for note in &self.notes {
            let note = if full {
                note.clone()
            } else {
                SourceNote {
                    key: note.key.clone(),
                    folder: note.folder.clone(),
                    title: note.title.clone(),
                    ..SourceNote::default()
                }
            };
            each(note)?;
        }
        Ok(())
    }

    fn read(&mut self, _path: &str, _limit: u64) -> io::Result<Vec<u8>> {
        Err(io::ErrorKind::NotFound.into())
    }

    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// The text of the JSON file at `path`, or of the first file in the zip at
/// `path` that `pick` chooses among its paths (sorted).
fn read_json(path: &Path, pick: impl Fn(&[String]) -> Vec<String>) -> anyhow::Result<String> {
    anyhow::ensure!(
        !path.is_dir(),
        "{} is a folder; pick the export file",
        path.display()
    );
    let is_zip = extension(&path.to_string_lossy()) == "zip" || looks_like_zip(path);
    if !is_zip {
        let meta = std::fs::metadata(path)?;
        anyhow::ensure!(
            meta.len() <= MAX_JSON_FILE,
            "{} is too large to read",
            path.display()
        );
        let bytes =
            std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
        return Ok(text_of(&bytes));
    }
    let mut tree = Tree::zip(path)?;
    let paths: Vec<String> = tree.files()?.into_iter().map(|f| f.path).collect();
    let candidates = pick(&paths);
    let first = candidates
        .first()
        .with_context(|| format!("{} holds no export file", path.display()))?;
    tree.read_text(first, MAX_JSON_FILE)
        .with_context(|| format!("cannot read {first} in {}", path.display()))
}

fn looks_like_zip(path: &Path) -> bool {
    use std::io::Read as _;
    let mut magic = [0u8; 4];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .is_ok()
        && magic == *b"PK\x03\x04"
}

// ---------------------------------------------------------------------------
// Standard Notes

#[derive(Deserialize)]
struct SnBackup {
    items: Option<Vec<SnItem>>,
}

#[derive(Deserialize)]
struct SnItem {
    #[serde(default)]
    uuid: String,
    #[serde(default)]
    content_type: String,
    #[serde(default)]
    content: Option<SnContent>,
    created_at: Option<String>,
    updated_at: Option<String>,
    #[serde(default)]
    deleted: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SnContent {
    /// An encrypted payload (`004:…`); only that it is one matters.
    Encrypted(#[allow(dead_code)] String),
    Fields(Box<SnFields>),
    Other(serde::de::IgnoredAny),
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SnFields {
    title: Option<String>,
    text: Option<String>,
    trashed: Option<bool>,
    archived: Option<bool>,
    pinned: Option<bool>,
    note_type: Option<String>,
    #[serde(default)]
    references: Vec<SnReference>,
    app_data: Option<SnAppData>,
}

#[derive(Deserialize, Default)]
struct SnAppData {
    #[serde(rename = "org.standardnotes.sn")]
    sn: Option<SnAppFields>,
}

#[derive(Deserialize, Default)]
struct SnAppFields {
    pinned: Option<bool>,
    archived: Option<bool>,
    client_updated_at: Option<String>,
}

#[derive(Deserialize)]
struct SnReference {
    #[serde(default)]
    uuid: String,
    content_type: Option<String>,
    reference_type: Option<String>,
}

/// An item's content when it is fields (not encrypted, not odd).
fn fields_of(item: &SnItem) -> Option<&SnFields> {
    match &item.content {
        Some(SnContent::Fields(fields)) => Some(fields.as_ref()),
        _ => None,
    }
}

/// Whether an item is still there (not a deletion's record).
fn live(item: &SnItem) -> bool {
    item.deleted != Some(true)
}

/// Opens a Standard Notes decrypted backup: the `.txt`/`.json` file, or a
/// `.zip` holding one.
pub fn standard_notes(path: &Path) -> anyhow::Result<JsonSource> {
    let text = read_json(path, |paths| {
        let mut found: Vec<String> = paths
            .iter()
            .filter(|p| matches!(extension(p).as_str(), "txt" | "json"))
            .cloned()
            .collect();
        // The backup itself before anything else in the zip.
        found.sort_by_key(|p| !file_name(p).to_lowercase().contains("backup"));
        found
    })?;
    parse_standard_notes(&text)
}

/// Reads a Standard Notes backup's JSON.
pub fn parse_standard_notes(text: &str) -> anyhow::Result<JsonSource> {
    let backup: SnBackup = serde_json::from_str(text)
        .context("this is not a Standard Notes backup (it isn't JSON with an items list)")?;
    let items = backup
        .items
        .context("this is not a Standard Notes backup (it has no items)")?;

    let mut skipped_types: BTreeMap<String, usize> = BTreeMap::new();
    for item in items.iter().filter(|item| live(item)) {
        match item.content_type.as_str() {
            "Note" | "Tag" => {
                if matches!(item.content, Some(SnContent::Encrypted(_))) {
                    anyhow::bail!(ENCRYPTED);
                }
            }
            other => *skipped_types.entry(other.to_owned()).or_default() += 1,
        }
    }

    // Tags: names with their parents', and the notes each is on.
    let mut tag_titles: HashMap<&str, (&str, Option<&str>)> = HashMap::new();
    for item in items.iter().filter(|item| live(item)) {
        if item.content_type != "Tag" {
            continue;
        }
        let Some(fields) = fields_of(item) else {
            continue;
        };
        let parent = fields
            .references
            .iter()
            .find(|r| {
                r.content_type.as_deref() == Some("Tag")
                    && r.reference_type.as_deref() == Some("TagToParentTag")
            })
            .map(|r| r.uuid.as_str());
        tag_titles.insert(
            item.uuid.as_str(),
            (fields.title.as_deref().unwrap_or_default(), parent),
        );
    }
    let mut note_tags: HashMap<&str, Vec<String>> = HashMap::new();
    for item in items.iter().filter(|item| live(item)) {
        if item.content_type != "Tag" {
            continue;
        }
        let Some(fields) = fields_of(item) else {
            continue;
        };
        let name = tag_path(&tag_titles, &item.uuid);
        if name.is_empty() {
            continue;
        }
        for reference in &fields.references {
            if matches!(reference.content_type.as_deref(), Some("Note") | None) {
                let names = note_tags.entry(reference.uuid.as_str()).or_default();
                if !names.contains(&name) {
                    names.push(name.clone());
                }
            }
        }
    }

    let mut notes = Vec::new();
    for item in items.iter().filter(|item| live(item)) {
        if item.content_type != "Note" {
            continue;
        }
        let empty = SnFields::default();
        let fields = fields_of(item).unwrap_or(&empty);
        let app = fields.app_data.as_ref().and_then(|a| a.sn.as_ref());
        let text = fields.text.clone().unwrap_or_default();
        let body = if fields.note_type.as_deref() == Some("super") {
            super_to_markdown(&text).unwrap_or(text)
        } else {
            text
        };
        let archived = fields.archived.or(app.and_then(|a| a.archived)) == Some(true);
        let updated = app
            .and_then(|a| a.client_updated_at.as_deref())
            .and_then(dates::parse)
            .or_else(|| item.updated_at.as_deref().and_then(dates::parse));
        notes.push(SourceNote {
            key: item.uuid.clone(),
            folder: if archived {
                vec![ARCHIVED.to_owned()]
            } else {
                Vec::new()
            },
            title: fields.title.clone().unwrap_or_default(),
            body,
            tags: note_tags.remove(item.uuid.as_str()).unwrap_or_default(),
            created: item.created_at.as_deref().and_then(dates::parse),
            updated,
            pinned: fields.pinned.or(app.and_then(|a| a.pinned)) == Some(true),
            trashed: fields.trashed == Some(true),
            ..SourceNote::default()
        });
    }

    let skipped = skipped_types
        .into_iter()
        .map(|(kind, count)| {
            let kind = if kind.is_empty() { "untyped" } else { &kind };
            Skipped::new(
                format!("{count} {kind} item(s)"),
                "Standard Notes' own data (settings, extensions, files), not notes or tags",
            )
        })
        .collect();
    Ok(JsonSource {
        name: "Standard Notes".to_owned(),
        notes,
        skipped,
    })
}

/// A tag's name with its parents' before it: `parent/child`. A missing
/// parent ends the walk, as does a loop.
fn tag_path(tags: &HashMap<&str, (&str, Option<&str>)>, uuid: &str) -> String {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    let mut at = Some(uuid);
    while let Some(current) = at {
        if !seen.insert(current) {
            break;
        }
        let Some((title, parent)) = tags.get(current) else {
            break;
        };
        let title = title.trim();
        if !title.is_empty() {
            names.push(title);
        }
        at = *parent;
    }
    names.reverse();
    names.join("/")
}

/// A Super note's text (Lexical's editor state as JSON) as Markdown:
/// headings, paragraphs, quotes, lists and checklists, code, links, rules
/// and tables, with bold, italic, strikethrough and inline code. `None`
/// when the text isn't such a state.
pub fn super_to_markdown(text: &str) -> Option<String> {
    let state: Value = serde_json::from_str(text).ok()?;
    let root = state.get("root")?;
    let mut out = Vec::new();
    blocks(children(root), &mut out);
    Some(out.join("\n\n"))
}

fn children(node: &Value) -> &[Value] {
    node.get("children")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn node_type(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or_default()
}

fn blocks(nodes: &[Value], out: &mut Vec<String>) {
    for node in nodes {
        match node_type(node) {
            "paragraph" => out.push(inline(children(node))),
            "heading" => {
                let level = node
                    .get("tag")
                    .and_then(Value::as_str)
                    .and_then(|t| t.strip_prefix('h'))
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(1)
                    .clamp(1, 6);
                out.push(format!("{} {}", "#".repeat(level), inline(children(node))));
            }
            "quote" => {
                let text = inline(children(node));
                let quoted: Vec<String> = text.lines().map(|l| format!("> {l}")).collect();
                out.push(quoted.join("\n"));
            }
            "list" => {
                let mut lines = Vec::new();
                list(node, 0, &mut lines);
                out.push(lines.join("\n"));
            }
            "code" => {
                let language = node
                    .get("language")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                out.push(format!("```{language}\n{}\n```", plain(children(node))));
            }
            "horizontalrule" => out.push("---".to_owned()),
            "table" => out.push(table(node)),
            "text" | "link" | "autolink" | "linebreak" => {
                out.push(inline(std::slice::from_ref(node)))
            }
            _ => blocks(children(node), out),
        }
    }
}

fn list(node: &Value, depth: usize, lines: &mut Vec<String>) {
    let kind = node
        .get("listType")
        .and_then(Value::as_str)
        .unwrap_or("bullet");
    let mut number = node.get("start").and_then(Value::as_u64).unwrap_or(1);
    let indent = "    ".repeat(depth);
    for item in children(node) {
        let nested: Vec<&Value> = children(item)
            .iter()
            .filter(|c| node_type(c) == "list")
            .collect();
        if !nested.is_empty() {
            for sub in nested {
                list(sub, depth + 1, lines);
            }
            continue;
        }
        let marker = match kind {
            "number" => {
                let marker = format!("{number}.");
                number += 1;
                marker
            }
            "check" => {
                let checked = item.get("checked").and_then(Value::as_bool) == Some(true);
                if checked { "- [x]" } else { "- [ ]" }.to_owned()
            }
            _ => "-".to_owned(),
        };
        lines.push(format!("{indent}{marker} {}", inline(children(item))));
    }
}

fn table(node: &Value) -> String {
    let mut lines = Vec::new();
    for (ix, row) in children(node).iter().enumerate() {
        let cells: Vec<String> = children(row)
            .iter()
            .map(|cell| {
                let mut parts = Vec::new();
                blocks(children(cell), &mut parts);
                parts.join(" ").replace('\n', " ").replace('|', "\\|")
            })
            .collect();
        lines.push(format!("| {} |", cells.join(" | ")));
        if ix == 0 {
            lines.push(format!("|{}", " --- |".repeat(cells.len().max(1))));
        }
    }
    lines.join("\n")
}

/// The text of inline nodes, as written in code.
fn plain(nodes: &[Value]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node_type(node) {
            "linebreak" => out.push('\n'),
            "tab" => out.push('\t'),
            _ => {
                if let Some(text) = node.get("text").and_then(Value::as_str) {
                    out.push_str(text);
                }
                out.push_str(&plain(children(node)));
            }
        }
    }
    out
}

fn inline(nodes: &[Value]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node_type(node) {
            "linebreak" => out.push('\n'),
            "tab" => out.push('\t'),
            "link" | "autolink" => {
                let url = node.get("url").and_then(Value::as_str).unwrap_or_default();
                let text = inline(children(node));
                if url.is_empty() {
                    out.push_str(&text);
                } else {
                    out.push_str(&format!("[{text}]({url})"));
                }
            }
            _ => match node.get("text").and_then(Value::as_str) {
                Some(text) => {
                    let format = node.get("format").and_then(Value::as_u64).unwrap_or(0);
                    out.push_str(&styled(text, format));
                }
                None => out.push_str(&inline(children(node))),
            },
        }
    }
    out
}

/// Text with Lexical's format bits as Markdown: 1 bold, 2 italic,
/// 4 strikethrough, 16 code.
fn styled(text: &str, format: u64) -> String {
    if format == 0 || text.trim().is_empty() {
        return text.to_owned();
    }
    if format & 16 != 0 {
        return format!("`{text}`");
    }
    // Markers go inside the spaces: `** bold**` isn't bold.
    let start = text.len() - text.trim_start().len();
    let end = text.trim_end().len();
    let mut marks = String::new();
    if format & 1 != 0 {
        marks.push_str("**");
    }
    if format & 2 != 0 {
        marks.push('*');
    }
    if format & 4 != 0 {
        marks.push_str("~~");
    }
    let closing: String = marks.chars().rev().collect();
    format!(
        "{}{marks}{}{closing}{}",
        &text[..start],
        &text[start..end],
        &text[end..]
    )
}

// ---------------------------------------------------------------------------
// Simplenote

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimplenoteExport {
    active_notes: Option<Vec<SimplenoteNote>>,
    trashed_notes: Option<Vec<SimplenoteNote>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SimplenoteNote {
    #[serde(default)]
    id: String,
    #[serde(default)]
    content: String,
    creation_date: Option<When>,
    last_modified: Option<When>,
    #[serde(default)]
    tags: Vec<String>,
    pinned: Option<bool>,
    #[serde(default)]
    system_tags: Vec<String>,
}

/// A date as Simplenote writes it: ISO text, or seconds since the epoch.
#[derive(Deserialize)]
#[serde(untagged)]
enum When {
    Text(String),
    Number(f64),
}

impl When {
    fn ms(&self) -> Option<i64> {
        match self {
            When::Text(text) => dates::parse(text),
            When::Number(n) if n.is_finite() && *n > 0.0 => {
                // Seconds (with a fraction) until the year 5138; else ms.
                let ms = if *n < 100_000_000_000.0 {
                    n * 1000.0
                } else {
                    *n
                };
                Some(ms as i64)
            }
            When::Number(_) => None,
        }
    }
}

/// Opens Simplenote's export: the `.zip`, or the `notes.json` in it.
pub fn simplenote(path: &Path) -> anyhow::Result<JsonSource> {
    let text = read_json(path, |paths| {
        let mut found: Vec<String> = paths
            .iter()
            .filter(|p| file_name(p).eq_ignore_ascii_case("notes.json"))
            .cloned()
            .collect();
        found.sort_by_key(|p| p != "source/notes.json");
        found
    })?;
    parse_simplenote(&text)
}

/// Reads Simplenote's `notes.json`.
pub fn parse_simplenote(text: &str) -> anyhow::Result<JsonSource> {
    let export: SimplenoteExport = serde_json::from_str(text)
        .context("this is not Simplenote's notes.json (it isn't JSON with activeNotes)")?;
    anyhow::ensure!(
        export.active_notes.is_some() || export.trashed_notes.is_some(),
        "this is not Simplenote's notes.json (it has no activeNotes or trashedNotes)"
    );
    let active = export.active_notes.unwrap_or_default();
    let trashed = export.trashed_notes.unwrap_or_default();
    let notes = active
        .into_iter()
        .map(|n| (n, false))
        .chain(trashed.into_iter().map(|n| (n, true)))
        .map(|(note, trashed)| {
            let content = text_of(note.content.as_bytes());
            let (title, body) = split_title(&content);
            SourceNote {
                key: note.id,
                title,
                body,
                tags: note.tags,
                created: note.creation_date.as_ref().and_then(When::ms),
                updated: note.last_modified.as_ref().and_then(When::ms),
                pinned: note.pinned == Some(true) || note.system_tags.iter().any(|t| t == "pinned"),
                trashed,
                ..SourceNote::default()
            }
        })
        .collect();
    Ok(JsonSource {
        name: "Simplenote".to_owned(),
        notes,
        skipped: Vec::new(),
    })
}

/// A Simplenote note's title and body: the first non-blank line (trimmed,
/// without leading `#`s) and what follows it, without leading blank lines.
pub fn split_title(content: &str) -> (String, String) {
    let mut rest = skip_blank_lines(content);
    let (line, after) = rest.split_once('\n').unwrap_or((rest, ""));
    let title = line.trim().trim_start_matches('#').trim().to_owned();
    rest = skip_blank_lines(after);
    (title, rest.to_owned())
}

/// `text` from its first line that has more than whitespace.
fn skip_blank_lines(mut text: &str) -> &str {
    loop {
        match text.split_once('\n') {
            Some((line, rest)) if line.trim().is_empty() => text = rest,
            Some(_) => return text,
            None if text.trim().is_empty() => return "",
            None => return text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_titles() {
        assert_eq!(
            split_title("\n# Shopping \n\n\nmilk\neggs\n"),
            ("Shopping".to_owned(), "milk\neggs\n".to_owned())
        );
        assert_eq!(split_title("Only"), ("Only".to_owned(), String::new()));
        assert_eq!(split_title("  \n \n"), (String::new(), String::new()));
    }

    #[test]
    fn super_notes_become_markdown() {
        let state = r#"{"root":{"type":"root","children":[
            {"type":"heading","tag":"h2","children":[{"type":"text","text":"Plan","format":0}]},
            {"type":"paragraph","children":[{"type":"text","text":"bold ","format":1},{"type":"link","url":"https://x.y","children":[{"type":"text","text":"site","format":0}]}]},
            {"type":"list","listType":"check","children":[
                {"type":"listitem","checked":true,"children":[{"type":"text","text":"done","format":0}]},
                {"type":"listitem","checked":false,"children":[{"type":"text","text":"todo","format":0}]}]}
        ]}}"#;
        assert_eq!(
            super_to_markdown(state).unwrap(),
            "## Plan\n\n**bold** [site](https://x.y)\n\n- [x] done\n- [ ] todo"
        );
        assert_eq!(super_to_markdown("plain text"), None);
    }
}
