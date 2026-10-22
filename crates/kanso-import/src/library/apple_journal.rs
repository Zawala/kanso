//! Imports Apple Journal's local HTML export without opening remote assets.

use std::{io, path::Path};

use chrono::NaiveDate;

use super::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use super::{Skipped, Source, SourceNote};

const MAX_NOTE: u64 = 8 * 1024 * 1024;

pub struct AppleJournalSource {
    tree: Tree,
    notes: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    name: String,
}

/// Opens an Apple Journal HTML file, export folder, or ZIP archive.
pub fn open(path: &Path) -> anyhow::Result<AppleJournalSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else if extension(&path.to_string_lossy()) == "html"
        || extension(&path.to_string_lossy()) == "htm"
    {
        Tree::Single(path.to_owned())
    } else {
        Tree::zip(path)?
    };
    let all = tree.files()?;
    let mut notes = Vec::new();
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for file in all {
        let ext = extension(&file.path);
        if ext == "html" || ext == "htm" {
            if file_name(&file.path).eq_ignore_ascii_case("index.html")
                || file_name(&file.path).eq_ignore_ascii_case("index.htm")
            {
                skipped.push(Skipped::new(file.path, "Apple Journal export index page"));
            } else if file.size > MAX_NOTE {
                skipped.push(Skipped::new(file.path, "too large for an HTML note"));
            } else {
                notes.push(file);
            }
        } else {
            files.push(file);
        }
    }
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Apple Journal")
        .to_owned();
    Ok(AppleJournalSource {
        tree,
        notes,
        files,
        skipped,
        name,
    })
}

impl Source for AppleJournalSource {
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
            let note = parse_entry(&file, &html);
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

fn parse_entry(file: &TreeFile, html: &str) -> SourceNote {
    let prompt = elements_with_class(html, "reflectionPrompt")
        .into_iter()
        .next()
        .unwrap_or_default();
    let paragraphs = elements_with_class(html, "p2")
        .into_iter()
        .chain(elements_with_class(html, "p3"))
        .map(|p| format!("<p>{p}</p>"))
        .collect::<Vec<_>>()
        .join("\n");
    let body_html = format!("{prompt}\n{paragraphs}");
    let body = htmd::convert(&body_html).unwrap_or_else(|_| strip_tags(&body_html));
    let header = elements_with_class(html, "pageHeader")
        .into_iter()
        .next()
        .unwrap_or_default();
    let header = strip_tags(&header);
    let created = header
        .trim()
        .split_once(", ")
        .and_then(|(_, date)| parse_english_date(date))
        .and_then(|d| {
            d.and_hms_opt(12, 0, 0)
                .map(|dt| dt.and_utc().timestamp_millis())
        });
    let tokens = elements_with_class(html, "assetGrid")
        .into_iter()
        .flat_map(|s| overlay_tokens(&s))
        .collect::<Vec<_>>();
    let tags = tokens
        .iter()
        .filter(|t| !t.starts_with("property:"))
        .cloned()
        .collect();
    let mut body = body.trim().to_owned();
    if !tokens.is_empty() {
        let props = tokens
            .iter()
            .filter_map(|t| t.strip_prefix("property:"))
            .collect::<Vec<_>>();
        if !props.is_empty() {
            body.push_str(&format!("\n\nJournal details: {}", props.join(", ")));
        }
    }
    SourceNote {
        key: file.path.clone(),
        path: Some(file.path.clone()),
        folder: parent_of(&file.path)
            .split('/')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
        title: stem(file_name(&file.path)).replace(['_', '-'], " "),
        body,
        tags,
        created,
        updated: created,
        ..SourceNote::default()
    }
}

fn parse_english_date(text: &str) -> Option<NaiveDate> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() != 3 {
        return None;
    }
    let day = fields[0].parse::<u32>().ok()?;
    let month = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ]
    .iter()
    .position(|month| month.eq_ignore_ascii_case(fields[1]))? as u32
        + 1;
    NaiveDate::from_ymd_opt(fields[2].parse().ok()?, month, day)
}

fn elements_with_class(html: &str, class: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let class = class.to_ascii_lowercase();
    let mut found = Vec::new();
    let mut cursor = 0;
    while let Some(rel) = lower[cursor..].find(&class) {
        let at = cursor + rel;
        let Some(open) = lower[..at].rfind('<') else {
            break;
        };
        let Some(gt_rel) = lower[at..].find('>') else {
            break;
        };
        let gt = at + gt_rel;
        if !lower[open..gt].contains("class=") {
            cursor = gt + 1;
            continue;
        }
        let tag = lower[open + 1..]
            .split(|c: char| c.is_whitespace() || c == '>')
            .next()
            .unwrap_or("div");
        let close = format!("</{tag}");
        if let Some(end_rel) = lower[gt + 1..].find(&close) {
            let end = gt + 1 + end_rel;
            found.push(html[gt + 1..end].to_owned());
            cursor = end + close.len();
        } else {
            cursor = gt + 1;
        }
    }
    found
}

fn overlay_tokens(html: &str) -> Vec<String> {
    let text = strip_tags(html);
    text.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != ':')
        .filter(|s| s.len() > 1)
        .map(str::to_owned)
        .collect()
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in s.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_prompt_paragraphs_date_and_overlay_tokens() {
        let file = TreeFile {
            path: "2024/Entry.html".into(),
            size: 0,
            modified: None,
            created: None,
        };
        assert_eq!(
            parse_english_date("1 January 2024"),
            NaiveDate::from_ymd_opt(2024, 1, 1)
        );
        let html = r#"<h2 class="pageHeader">Monday, 1 January 2024</h2><div class="reflectionPrompt">What mattered?</div><p class="p2">First paragraph.</p><p class="p3">Second paragraph.</p><div class="assetGrid"><span>Walk Nature</span></div>"#;
        assert_eq!(
            elements_with_class(html, "pageHeader"),
            ["Monday, 1 January 2024"]
        );
        let note = parse_entry(&file, html);
        assert_eq!(note.created, Some(1_704_110_400_000));
        assert!(note.body.contains("What mattered?"));
        assert!(note.body.contains("First paragraph."));
        assert_eq!(note.tags, ["Walk", "Nature"]);
    }
}
