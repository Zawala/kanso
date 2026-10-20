//! Bear: a `.bear2bk` backup (a zip of TextBundles), or a folder from
//! Export Notes as TextBundle or Markdown.
//!
//! A TextBundle is a folder `Title.textbundle` holding `text.md` (or
//! `.markdown`, `.txt`), `info.json` with Bear's metadata, and `assets/`.
//! Bear has no notebooks, so notes go into one called Bear; archived notes
//! into a notebook inside it. Tags are Bear's `#tag`, `#nested/tag` and
//! `#multi word tag#`.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use kanso_core::tags::inline_tag_names;
use serde_json::Value;

use crate::library::resolve::is_note_path;
use crate::library::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use crate::library::{Skipped, Source, SourceNote, dates};

/// Note files larger than this are skipped.
const MAX_NOTE_FILE: u64 = 8 * 1024 * 1024;

pub struct BearSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
}

pub fn open(path: &Path) -> anyhow::Result<BearSource> {
    BearSource::open(path)
}

impl BearSource {
    pub fn open(path: &Path) -> anyhow::Result<BearSource> {
        let mut tree = if path.is_dir() {
            Tree::dir(path)?
        } else {
            Tree::zip(path)?
        };
        let mut notes = Vec::new();
        let mut files = Vec::new();
        let mut skipped = Vec::new();
        for file in tree.files()? {
            let name = file_name(&file.path);
            let bundle = bundle_of(&file.path);
            if bundle.is_some() && name == "info.json" {
                continue;
            }
            let is_text = match bundle {
                Some(bundle) => parent_of(&file.path) == bundle && stem(name) == "text",
                None => is_note_path(name),
            };
            if is_text {
                if file.size > MAX_NOTE_FILE {
                    skipped.push(Skipped::new(file.path, "too large for a note"));
                } else {
                    notes.push(file);
                }
            } else {
                files.push(file);
            }
        }
        Ok(BearSource {
            tree,
            notes,
            files,
            skipped,
        })
    }
}

/// The `….textbundle` folder a path is in.
fn bundle_of(path: &str) -> Option<&str> {
    let mut end = 0;
    for part in path.split('/') {
        end += part.len();
        if extension(part) == "textbundle" {
            return Some(&path[..end]);
        }
        end += 1;
    }
    None
}

impl Source for BearSource {
    fn name(&self) -> String {
        "Bear".to_owned()
    }

    fn files(&self) -> &[TreeFile] {
        &self.files
    }

    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let notes = self.notes.clone();
        for file in notes {
            let text = self
                .tree
                .read_text(&file.path, MAX_NOTE_FILE)
                .map_err(|err| anyhow::anyhow!("cannot read {}: {err}", file.path))?;
            let info = match bundle_of(&file.path) {
                Some(bundle) => self
                    .tree
                    .read(&format!("{bundle}/info.json"), 1024 * 1024)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()),
                None => None,
            };
            each(bear_note(&file, &text, info.as_ref()))?;
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

/// A Bear note from its text and its bundle's `info.json`.
pub fn bear_note(file: &TreeFile, text: &str, info: Option<&Value>) -> SourceNote {
    let bear = info.and_then(|i| i.get("net.shinyfrog.bear"));
    let flag = |key: &str| {
        bear.and_then(|b| b.get(key))
            .is_some_and(|v| v.as_i64() == Some(1) || v.as_bool() == Some(true))
    };
    let date = |key: &str| {
        bear.and_then(|b| b.get(key))
            .and_then(Value::as_str)
            .and_then(dates::parse)
    };
    let fallback = match bundle_of(&file.path) {
        Some(bundle) => stem(file_name(bundle)).to_owned(),
        None => stem(file_name(&file.path)).to_owned(),
    };
    let trimmed = text.trim_start_matches('\n');
    let (title, body) = match trimmed.split_once('\n') {
        Some((first, rest)) if first.starts_with("# ") => (first[2..].trim().to_owned(), rest),
        None if trimmed.starts_with("# ") => (trimmed[2..].trim().to_owned(), ""),
        _ => (fallback, trimmed),
    };
    let body = body.trim_start_matches('\n').to_owned();
    let closed = closed_tags(&body);
    // `#road trip#` isn't also the tag `road`.
    let mut masked = body.clone();
    for (range, _) in closed.iter().rev() {
        masked.replace_range(range.clone(), &" ".repeat(range.len()));
    }
    let mut tags: Vec<String> = closed.into_iter().map(|(_, name)| name).collect();
    tags.extend(inline_tag_names(&masked));
    let mut seen = BTreeSet::new();
    tags.retain(|t| seen.insert(t.to_lowercase()));
    let key = bear
        .and_then(|b| b.get("uniqueIdentifier"))
        .and_then(Value::as_str)
        .map_or_else(|| file.path.clone(), str::to_owned);
    SourceNote {
        key,
        path: Some(file.path.clone()),
        folder: if flag("archived") {
            vec!["Archived".to_owned()]
        } else {
            Vec::new()
        },
        title,
        body,
        tags,
        created: date("creationDate").or(file.created).or(file.modified),
        updated: date("modificationDate").or(file.modified),
        pinned: flag("pinned"),
        trashed: flag("trashed"),
        ..SourceNote::default()
    }
}

/// Bear's multi-word tags, `#like this#`, outside code: a `#` at the start
/// of a line or after a space, words, and a closing `#` on the same line.
/// Returns each tag's bytes (both `#`s included) and name.
pub fn closed_tags(body: &str) -> Vec<(std::ops::Range<usize>, String)> {
    let code = kanso_core::links::code_ranges(body);
    let in_code = |at: usize| code.iter().any(|r| r.contains(&at));
    let mut tags = Vec::new();
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let content = line.trim_end_matches('\n');
        let mut from = 0;
        while let Some(at) = content[from..].find('#') {
            let at = from + at;
            from = at + 1;
            let before = content[..at].chars().next_back();
            if !before.is_none_or(char::is_whitespace) || in_code(start + at) {
                continue;
            }
            let rest = &content[at + 1..];
            let Some(close) = rest.find('#') else {
                break;
            };
            let inner = &rest[..close];
            let after = rest[close + 1..].chars().next();
            let valid = inner.contains(' ')
                && !inner.starts_with(' ')
                && !inner.ends_with(' ')
                && inner.chars().any(char::is_alphabetic)
                && after.is_none_or(|c| c.is_whitespace() || c.is_ascii_punctuation());
            if valid {
                tags.push((start + at..start + at + close + 2, inner.to_owned()));
                from = at + 1 + close + 1;
            }
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn file(path: &str) -> TreeFile {
        TreeFile {
            path: path.to_owned(),
            size: 0,
            modified: Some(5),
            created: None,
        }
    }

    #[test]
    fn a_textbundle_note() {
        let info = json!({"net.shinyfrog.bear": {
            "uniqueIdentifier": "ABC-1",
            "creationDate": "2024-03-01T09:30:00Z",
            "modificationDate": "2024-03-02T09:30:00Z",
            "pinned": 1, "archived": 1, "trashed": 0
        }});
        let note = bear_note(
            &file("Bear/Plan.textbundle/text.md"),
            "# Plan\n\nWork #work/acme and #road trip# ![](assets/pic.png)\n",
            Some(&info),
        );
        assert_eq!(note.key, "ABC-1");
        assert_eq!(note.title, "Plan");
        assert_eq!(note.tags, ["road trip", "work/acme"]);
        assert_eq!(note.folder, ["Archived"]);
        assert!(note.pinned && !note.trashed);
        assert_eq!(note.created, Some(1_709_285_400_000));
        assert!(note.body.starts_with("Work #work"));
    }

    #[test]
    fn closed_tags_need_words_and_a_close() {
        let names = |body| {
            closed_tags(body)
                .into_iter()
                .map(|(_, n)| n)
                .collect::<Vec<_>>()
        };
        assert_eq!(names("#a b# and #c d#."), ["a b", "c d"]);
        assert_eq!(closed_tags("x #a b#")[0].0, 2..7);
        assert!(closed_tags("#single# and #open tag").is_empty());
        assert!(closed_tags("`#in code#` x#a b#").is_empty());
    }

    #[test]
    fn bundles() {
        assert_eq!(
            bundle_of("x/A.textbundle/assets/p.png"),
            Some("x/A.textbundle")
        );
        assert_eq!(bundle_of("A.md"), None);
    }
}
