//! Imports Google Keep JSON exports and their local Takeout attachments.

use std::{io, path::Path};

use serde_json::Value;

use super::html;
use super::tree::{Tree, TreeFile, extension, file_name, parent_of};
use super::{Skipped, Source, SourceNote};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct GoogleKeepSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
}

/// Opens one Keep JSON note, a folder, or a Takeout ZIP.
pub fn open(path: &Path) -> anyhow::Result<GoogleKeepSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else if extension(&path.to_string_lossy()) == "zip" {
        Tree::zip(path)?
    } else {
        Tree::Single(path.to_owned())
    };
    let listed = tree.files()?;
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let has_json = listed.iter().any(|f| extension(&f.path) == "json");
    for file in listed {
        match extension(&file.path).as_str() {
            "json" if file.size <= MAX_NOTE => notes.push(file),
            "json" => skipped.push(Skipped::new(file.path, "too large for a Keep note")),
            "html" | "htm" | "txt" if has_json => skipped.push(Skipped::new(
                file.path,
                "Takeout HTML/TXT duplicate; JSON is canonical",
            )),
            "html" | "htm" | "txt" => {
                skipped.push(Skipped::new(file.path, "not a Google Keep JSON note"))
            }
            _ => files.push(file),
        }
    }
    Ok(GoogleKeepSource {
        tree,
        notes,
        files,
        skipped,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Google Keep")
            .to_owned(),
    })
}

impl Source for GoogleKeepSource {
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
            let raw = self
                .tree
                .read(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let value: Value = match serde_json::from_slice(&raw) {
                Ok(v) => v,
                Err(e) => {
                    push_skipped_once(
                        &mut self.skipped,
                        Skipped::new(file.path, format!("invalid Keep JSON: {e}")),
                    );
                    continue;
                }
            };
            let note = match parse_note(&file, &value, &self.files) {
                Ok(note) => note,
                Err(reason) => {
                    push_skipped_once(&mut self.skipped, Skipped::new(file.path, reason));
                    continue;
                }
            };
            each(note)?;
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

fn push_skipped_once(skipped: &mut Vec<Skipped>, item: Skipped) {
    if !skipped.iter().any(|existing| existing.path == item.path) {
        skipped.push(item);
    }
}

/// The note in one Keep JSON file, or why it can't be imported.
fn parse_note(
    file: &TreeFile,
    value: &Value,
    files: &[TreeFile],
) -> Result<SourceNote, &'static str> {
    const BAD_TIME: &str = "missing or invalid Keep timestamps";
    let created = value
        .get("createdTimestampUsec")
        .and_then(|t| keep_time(t, false))
        .ok_or(BAD_TIME)?;
    let updated = value
        .get("userEditedTimestampUsec")
        .and_then(|t| keep_time(t, true))
        .filter(|updated| *updated > 0)
        .unwrap_or(created);
    let raw_title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_owned();
    let mut title_lines = raw_title
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let first_title = title_lines.next().unwrap_or("");
    let aliases: Vec<String> = title_lines.map(str::to_owned).collect();
    let title = if first_title.is_empty() {
        file_name(&file.path).trim_end_matches(".json").to_owned()
    } else {
        first_title.to_owned()
    };
    let text = value
        .get("textContent")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty());
    let mut body = match (text, value.get("textContentHtml").and_then(Value::as_str)) {
        (Some(text), _) => text.to_owned(),
        // The body is all there is: rather than drop it, skip the note.
        (None, Some(html)) => {
            html::to_markdown(html).ok_or("note text nested too deeply to convert")?
        }
        (None, None) => String::new(),
    };
    let mut list = Vec::new();
    if let Some(items) = value.get("listContent").and_then(Value::as_array) {
        for item in items {
            if let Some(text) = item.get("text").and_then(Value::as_str) {
                // An item's formatting is optional: its plain text will do.
                let text = item
                    .get("textHtml")
                    .and_then(Value::as_str)
                    .and_then(html::to_markdown)
                    .unwrap_or_else(|| text.to_owned());
                let checked = item
                    .get("isChecked")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                list.push(format!(
                    "- [{}] {}",
                    if checked { "x" } else { " " },
                    text.trim()
                ));
            }
        }
    }
    if !list.is_empty() {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&list.join("\n"));
    }
    let labels: Vec<String> = value
        .get("labels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|l| l.get("name").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let mut attachments = Vec::new();
    if let Some(items) = value.get("attachments").and_then(Value::as_array) {
        for item in items {
            let Some(raw) = item.get("filePath").and_then(Value::as_str) else {
                continue;
            };
            let rel = resolve_attachment(&file.path, raw, files)
                .unwrap_or_else(|| raw.replace('\\', "/"));
            let label = item
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| file_name(&rel).to_owned());
            let image = item
                .get("mimetype")
                .and_then(Value::as_str)
                .is_some_and(|s| s.starts_with("image/"));
            attachments.push(if image {
                format!("![{label}](kanso-import:path/{rel})")
            } else {
                format!("[{label}](kanso-import:path/{rel})")
            });
        }
    }
    if !attachments.is_empty() {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&attachments.join("\n\n"));
    }
    if let Some(annotations) = value.get("annotations").and_then(Value::as_array) {
        let rendered: Vec<String> = annotations.iter().filter_map(format_annotation).collect();
        if !rendered.is_empty() {
            if !body.is_empty() {
                body.push_str("\n\n");
            }
            body.push_str("## Annotations\n\n");
            body.push_str(
                &rendered
                    .into_iter()
                    .map(|item| format!("- {item}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
    }
    if let Some(tasks) = value
        .get("tasks")
        .and_then(Value::as_array)
        .filter(|tasks| !tasks.is_empty())
    {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str("## Tasks\n\n");
        body.push_str(
            &tasks
                .iter()
                .filter_map(|task| task.get("id").and_then(Value::as_str))
                .map(|id| format!("- {id}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    Ok(SourceNote {
        key: file.path.clone(),
        path: Some(file.path.clone()),
        folder: {
            let mut folders: Vec<String> = parent_of(&file.path)
                .split('/')
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect();
            if value
                .get("isArchived")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                folders.push("Archived".to_owned());
            }
            folders
        },
        title,
        body,
        tags: labels,
        aliases,
        created: Some(created),
        updated: Some(updated),
        pinned: value
            .get("isPinned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        trashed: value
            .get("isTrashed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        ..SourceNote::default()
    })
}

fn keep_time(value: &Value, allow_zero: bool) -> Option<i64> {
    let micros = value
        .as_i64()
        .or_else(|| value.as_str()?.parse::<i64>().ok())?;
    ((micros > 0 || (allow_zero && micros == 0)) && micros <= 253_402_300_799_999_999)
        .then_some(micros / 1_000)
}

fn format_annotation(value: &Value) -> Option<String> {
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let source = value
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let primary = match (title.is_empty(), url.is_empty()) {
        (false, false) => format!("[{title}]({url})"),
        (false, true) => title.to_owned(),
        (true, false) => format!("<{url}>"),
        (true, true) if !description.is_empty() => description.to_owned(),
        (true, true) => source.to_owned(),
    };
    if primary.is_empty() {
        return None;
    }
    Some(
        if !description.is_empty() && description != title && description != primary {
            format!("{primary} — {description}")
        } else {
            primary
        },
    )
}

fn resolve_attachment(note_path: &str, raw: &str, files: &[TreeFile]) -> Option<String> {
    let clean = raw.replace('\\', "/").trim_start_matches("./").to_owned();
    let local = format!("{}/{}", parent_of(note_path), clean);
    files
        .iter()
        .find(|f| f.path == clean || f.path == local || f.path.ends_with(&format!("/{clean}")))
        .map(|f| f.path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_list_labels_flags_dates_and_local_files() {
        let file = TreeFile {
            path: "Keep/Note.json".into(),
            size: 0,
            modified: None,
            created: None,
        };
        let asset = TreeFile {
            path: "Keep/photo.jpg".into(),
            size: 2,
            modified: None,
            created: None,
        };
        let data = serde_json::json!({"title":"Trip\nHoliday","textContent":"Pack","listContent":[{"text":"Passport","isChecked":true}],"labels":[{"name":"travel"}],"isPinned":true,"isTrashed":false,"createdTimestampUsec":1700000000000000i64,"userEditedTimestampUsec":1700000001000000i64,"attachments":[{"filePath":"photo.jpg","mimetype":"image/jpeg"}]});
        let note = parse_note(&file, &data, &[asset]).unwrap();
        assert!(note.body.contains("- [x] Passport"));
        assert!(note.body.contains("kanso-import:path/Keep/photo.jpg"));
        assert_eq!(note.tags, ["travel"]);
        assert!(note.pinned);
        assert_eq!(note.title, "Trip");
        assert_eq!(note.aliases, ["Holiday"]);
        assert_eq!(note.created, Some(1_700_000_000_000));
    }
    #[test]
    fn over_deep_html_skips_the_body_but_not_a_list_item() {
        let file = TreeFile {
            path: "Note.json".into(),
            size: 0,
            modified: None,
            created: None,
        };
        let deep = "<div>".repeat(10_000);
        let times = serde_json::json!({"createdTimestampUsec":1, "userEditedTimestampUsec":1});
        let mut item = times.clone();
        item["listContent"] = serde_json::json!([{"text":"Milk","textHtml":deep}]);
        let note = parse_note(&file, &item, &[]).unwrap();
        assert_eq!(note.body, "- [ ] Milk");
        let mut body = times.clone();
        body["textContentHtml"] = deep.into();
        assert_eq!(
            parse_note(&file, &body, &[]).unwrap_err(),
            "note text nested too deeply to convert"
        );
        body["textContent"] = "Plain".into();
        assert_eq!(parse_note(&file, &body, &[]).unwrap().body, "Plain");
    }
    #[test]
    fn rejects_unusable_timestamps() {
        assert!(keep_time(&Value::from(0), false).is_none());
        assert_eq!(keep_time(&Value::from(0), true), Some(0));
    }
}
