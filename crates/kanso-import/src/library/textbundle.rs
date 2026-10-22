//! Imports TextBundle and TextPack Markdown with bundle-local assets.

use std::{collections::BTreeSet, io, path::Path};

use serde_json::Value;

use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote, dates};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct TextBundleSource {
    tree: Tree,
    /// Each note's body and its bundle's path (empty for the opened bundle).
    notes: Vec<(TreeFile, String)>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
    /// The opened file's or folder's name, standing in for an empty bundle path.
    root: String,
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
    // Opened as the bundle itself (a `.textbundle` folder, or a `.textpack`
    // holding `text.md` at its root), paths are `text.md` and `assets/…`.
    let root = is_bundle || (!path.is_dir() && listed.iter().any(|f| is_body("", &f.path)));
    let bundle_of = |path: &str| {
        if root {
            Some(String::new())
        } else {
            bundle_prefix(path)
        }
    };
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let bundles: BTreeSet<String> = listed.iter().filter_map(|f| bundle_of(&f.path)).collect();
    let mut bodied = BTreeSet::new();
    for file in listed {
        match bundle_of(&file.path) {
            Some(bundle) if file.path == in_bundle(&bundle, "info.json") => {}
            // The first `text.*` is the body; any other is an attachment.
            Some(bundle) if is_body(&bundle, &file.path) && bodied.insert(bundle.clone()) => {
                if file.size > MAX_NOTE {
                    skipped.push(Skipped::new(file.path, "too large for a TextBundle note"));
                } else {
                    notes.push((file, bundle));
                }
            }
            _ => files.push(file),
        }
    }
    if is_bundle && bodied.is_empty() {
        anyhow::bail!("{} is not a TextBundle", path.display());
    }
    for bundle in bundles.difference(&bodied) {
        skipped.push(Skipped::new(
            format!("{bundle}/"),
            "TextBundle has no Markdown body",
        ));
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
        root: path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("TextBundle")
            .to_owned(),
    })
}

/// Whether `path` is the bundle's body: `text.md`, `text.markdown`,
/// `text.txt` or any other `text.*` directly inside it.
fn is_body(bundle: &str, path: &str) -> bool {
    parent_of(path) == bundle && file_name(path).starts_with("text.")
}

fn in_bundle(bundle: &str, name: &str) -> String {
    if bundle.is_empty() {
        name.to_owned()
    } else {
        format!("{bundle}/{name}")
    }
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
        for (file, bundle) in self.notes.clone() {
            let text = self
                .tree
                .read_text(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let metadata = self
                .tree
                .read(&in_bundle(&bundle, "info.json"), 1024 * 1024)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
            let bundle = if bundle.is_empty() {
                &self.root
            } else {
                &bundle
            };
            each(parse_note(&file, bundle, &text, metadata.as_ref()))?;
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

/// A note from its body, its bundle's path (or name) and its `info.json`.
fn parse_note(file: &TreeFile, bundle: &str, text: &str, info: Option<&Value>) -> SourceNote {
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
            .unwrap_or_else(|| bundle.to_owned()),
        path: Some(file.path.clone()),
        folder: parent_of(bundle)
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
        let note = parse_note(&file, "Plan.textbundle", "# Plan\n\nBody", Some(&info));
        assert_eq!(note.key, "Plan.textbundle");
        assert_eq!(note.tags, ["work"]);
        assert_eq!(note.body, "Body");
        assert!(note.created.is_some());
    }
}
