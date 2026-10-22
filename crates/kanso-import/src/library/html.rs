//! Imports local HTML documents as Markdown, preserving their tree paths.

use std::{collections::HashMap, io, path::Path};

use percent_encoding::percent_decode_str;

use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct HtmlSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
    targets: HashMap<String, String>,
}

/// Opens one HTML document, a local folder, or a ZIP archive.
pub fn open(path: &Path) -> anyhow::Result<HtmlSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else if matches!(extension(&path.to_string_lossy()).as_str(), "html" | "htm") {
        Tree::Single(path.to_owned())
    } else {
        Tree::zip(path)?
    };
    let listed = tree.files()?;
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for file in listed {
        if matches!(extension(&file.path).as_str(), "html" | "htm") {
            if file.size > MAX_NOTE {
                skipped.push(Skipped::new(file.path, "too large for an HTML note"));
            } else {
                notes.push(file);
            }
        } else {
            files.push(file);
        }
    }
    let targets = notes
        .iter()
        .filter_map(|f| normalize(&f.path).map(|path| (path, f.path.clone())))
        .collect();
    Ok(HtmlSource {
        tree,
        notes,
        files,
        skipped,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("HTML")
            .to_owned(),
        targets,
    })
}

impl Source for HtmlSource {
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
            let html = self
                .tree
                .read_text(&file.path, MAX_NOTE)
                .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", file.path))?;
            let rewritten = rewrite_doc_links(&html, &file.path, &self.targets);
            let cleaned = remove_blocks(&rewritten, &["script", "style", "noscript"]);
            let body = htmd::convert(&cleaned)
                .map_err(|e| anyhow::anyhow!("cannot convert {}: {e}", file.path))?;
            let title = html_title(&html)
                .unwrap_or_else(|| stem(file_name(&file.path)).replace(['_', '-'], " "));
            each(SourceNote {
                key: file.path.clone(),
                path: Some(file.path.clone()),
                folder: parent_of(&file.path)
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect(),
                title,
                body: body.trim().to_owned(),
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

fn html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    for tag in ["title", "h1"] {
        let Some(start) = lower.find(&format!("<{tag}")) else {
            continue;
        };
        let Some(gt) = lower[start..].find('>').map(|offset| offset + start + 1) else {
            continue;
        };
        let Some(end) = lower[gt..]
            .find(&format!("</{tag}"))
            .map(|offset| offset + gt)
        else {
            continue;
        };
        let value = plain_text(&html[gt..end]);
        if !value.trim().is_empty() {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn rewrite_doc_links(html: &str, path: &str, targets: &HashMap<String, String>) -> String {
    let lower = html.to_ascii_lowercase();
    let mut result = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find("href=") {
        let at = cursor + rel;
        result.push_str(&html[cursor..at + 5]);
        let quote = html.as_bytes().get(at + 5).copied().unwrap_or(b' ');
        if quote != b'\'' && quote != b'"' {
            cursor = at + 5;
            continue;
        }
        let value_start = at + 6;
        let Some(end_rel) = html[value_start..].find(char::from(quote)) else {
            result.push_str(&html[value_start..]);
            return result;
        };
        let end = value_start + end_rel;
        let raw = &html[value_start..end];
        let decoded = raw.split(['?', '#']).next().unwrap_or(raw);
        let decoded = percent_decode_str(decoded).decode_utf8_lossy();
        if !decoded.contains("://") && !decoded.starts_with(['/', '#', '?']) {
            let dest = normalize(&format!("{}/{}", parent_of(path), decoded));
            if let Some(target) = dest.as_ref().and_then(|dest| targets.get(dest)) {
                result.push_str(&format!("kanso-import:note/{target}"));
            } else {
                result.push_str(raw);
            }
        } else {
            result.push_str(raw);
        }
        result.push(char::from(quote));
        cursor = end + 1;
    }
    result.push_str(&html[cursor..]);
    result
}

fn normalize(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    let normalized = path.replace('\\', "/");
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

fn remove_blocks(input: &str, tags: &[&str]) -> String {
    let mut out = input.to_owned();
    for tag in tags {
        loop {
            let lower = out.to_ascii_lowercase();
            let Some(start) = lower.find(&format!("<{tag}")) else {
                break;
            };
            let Some(end_start) = lower[start..].find(&format!("</{tag}")) else {
                out.replace_range(start.., "");
                break;
            };
            let end_start = start + end_start;
            let Some(end) = lower[end_start..].find('>') else {
                out.replace_range(start.., "");
                break;
            };
            out.replace_range(start..end_start + end + 1, "");
        }
    }
    out
}

fn plain_text(value: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in value.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn links_local_pages_and_ignores_scripts() {
        let targets = HashMap::from([("dir/next.html".into(), "dir/next.html".into())]);
        let converted = rewrite_doc_links(
            "<a href='next.html'>Next</a><script>bad()</script>",
            "dir/start.html",
            &targets,
        );
        assert!(converted.contains("kanso-import:note/dir/next.html"));
        assert!(!remove_blocks(&converted, &["script"]).contains("bad()"));
    }
    #[test]
    fn derives_title_and_normalizes_paths() {
        assert_eq!(
            html_title("<title> A &amp; B </title>"),
            Some("A & B".into())
        );
        assert_eq!(normalize("a/b/../c.html"), Some("a/c.html".into()));
        assert_eq!(normalize("../c.html"), None);
    }
}
