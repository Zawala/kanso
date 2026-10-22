//! Imports TextBundle and TextPack Markdown with bundle-local assets.

use std::{collections::BTreeSet, io, path::Path};

use serde_json::Value;

use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote, dates};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct TextBundleSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
}

/// Opens a `.textbundle`, `.textpack`, ZIP, or folder containing bundles.
pub fn open(path: &Path) -> anyhow::Result<TextBundleSource> {
    let is_bundle = extension(&path.to_string_lossy()) == "textbundle";
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else {
        Tree::zip(path)?
    };
    let listed = tree.files()?;
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let bundles: BTreeSet<String> = listed
        .iter()
        .filter_map(|f| bundle_prefix(&f.path))
        .collect();
    for bundle in &bundles {
        if !listed.iter().any(|f| {
            f.path == format!("{bundle}/text.md") || f.path == format!("{bundle}/text.text.md")
        }) {
            skipped.push(Skipped::new(
                format!("{bundle}/"),
                "TextBundle has no Markdown body",
            ));
        }
    }
    for file in listed {
        if bundles
            .iter()
            .any(|b| file.path == format!("{b}/info.json"))
        {
            continue;
        }
        if bundles.iter().any(|b| {
            file.path == format!("{b}/text.md") || file.path == format!("{b}/text.text.md")
        }) {
            if file.size > MAX_NOTE {
                skipped.push(Skipped::new(file.path, "too large for a TextBundle note"));
            } else {
                notes.push(file);
            }
        } else {
            files.push(file);
        }
    }
    if is_bundle && bundles.is_empty() {
        anyhow::bail!("{} is not a TextBundle", path.display());
    }
    Ok(TextBundleSource {
        tree,
        notes,
        files,
        skipped,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("TextBundle")
            .to_owned(),
    })
}

fn bundle_prefix(path: &str) -> Option<String> {
    let mut end = 0;
    for part in path.split('/') {
        end += part.len();
        if extension(part) == "textbundle" {
            return Some(path[..end].to_owned());
        }
        end += 1;
    }
    None
}

impl Source for TextBundleSource {
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
            let text = self
                .tree
                .read_text(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let bundle = bundle_prefix(&file.path).unwrap_or_default();
            let metadata = self
                .tree
                .read(&format!("{bundle}/info.json"), 1024 * 1024)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            each(parse_note(&file, &text, metadata.as_ref()))?;
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

fn parse_note(file: &TreeFile, text: &str, info: Option<&Value>) -> SourceNote {
    let bundle = bundle_prefix(&file.path).unwrap_or_else(|| parent_of(&file.path).to_owned());
    let meta = info.and_then(|v| v.get("net.shinyfrog.bear")).or(info);
    let title = meta
        .and_then(|v| v.get("title"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| stem(file_name(&bundle)).to_owned());
    let mut body = text.trim_start_matches('\n').to_owned();
    if body.starts_with("# ") {
        if let Some((_, rest)) = body.split_once('\n') {
            body = rest.trim_start_matches('\n').to_owned();
        } else {
            body.clear();
        }
    }
    let tags = meta
        .and_then(|v| v.get("tags"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let date = |key: &str| {
        meta.and_then(|v| v.get(key))
            .and_then(Value::as_str)
            .and_then(dates::parse)
    };
    let created = date("creationDate").or_else(|| date("created"));
    let updated = date("modificationDate").or_else(|| date("modified"));
    SourceNote {
        key: meta
            .and_then(|v| v.get("uniqueIdentifier"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| bundle.clone()),
        path: Some(file.path.clone()),
        folder: parent_of(&bundle)
            .split('/')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
        title,
        body,
        tags,
        created,
        updated,
        ..SourceNote::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_title_tags_dates_and_body() {
        let file = TreeFile {
            path: "Plan.textbundle/text.md".into(),
            size: 0,
            modified: None,
            created: None,
        };
        let info = serde_json::json!({"net.shinyfrog.bear":{"title":"Plan","tags":["work"],"creationDate":"2024-01-01T00:00:00Z","modificationDate":"2024-01-02T00:00:00Z"}});
        let note = parse_note(&file, "# Plan\n\nBody", Some(&info));
        assert_eq!(note.key, "Plan.textbundle");
        assert_eq!(note.tags, ["work"]);
        assert_eq!(note.body, "Body");
        assert!(note.created.is_some());
    }
}
