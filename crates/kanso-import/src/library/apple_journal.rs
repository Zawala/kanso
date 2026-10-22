//! Imports Apple Journal's local HTML export without opening remote assets.

use std::{io, path::Path};

use chrono::NaiveDate;
use percent_encoding::{percent_decode_str, utf8_percent_encode};

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

/// The classes of an entry's text elements: its prompt and paragraphs.
const TEXT_CLASSES: [&str; 3] = ["reflectionPrompt", "p2", "p3"];

fn parse_entry(file: &TreeFile, html: &str) -> SourceNote {
    // In document order, so the prompt and paragraphs read as written.
    let body_html = elements_with_classes(html, &TEXT_CLASSES)
        .into_iter()
        .map(|(class, inner)| {
            if class.eq_ignore_ascii_case("reflectionPrompt") {
                inner
            } else {
                format!("<p>{inner}</p>")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = super::html::to_markdown(&body_html).unwrap_or_else(|| strip_tags(&body_html));
    let mut body = body.trim().to_owned();
    // Photos, videos and recordings, linked so they come along with the entry.
    for src in media_sources(html) {
        let src = percent_decode_str(&src).decode_utf8_lossy().into_owned();
        let target = utf8_percent_encode(&src, super::html::PATH);
        body.push_str(&format!("\n\n![]({target})"));
    }
    let created = elements_with_classes(html, &["pageHeader"])
        .into_iter()
        .next()
        .and_then(|(_, header)| parse_entry_date(&strip_tags(&header)))
        .and_then(|d| {
            d.and_hms_opt(12, 0, 0)
                .map(|dt| dt.and_utc().timestamp_millis())
        });
    // The asset grid's captions (a place, a workout, a song) are details of
    // the entry, kept whole: "San Francisco" is one place, not two tags.
    let mut details: Vec<String> = Vec::new();
    for (_, grid) in elements_with_classes(html, &["assetGrid"]) {
        for line in strip_tags_to_lines(&grid) {
            if !details.contains(&line) {
                details.push(line);
            }
        }
    }
    if !details.is_empty() {
        body.push_str(&format!("\n\nJournal details: {}", details.join(", ")));
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
        created,
        updated: created,
        ..SourceNote::default()
    }
}

const MONTHS: [&str; 12] = [
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
];

/// An entry's date, as "Monday, 1 January 2024" or "Monday, January 1,
/// 2024", with or without the weekday.
fn parse_entry_date(text: &str) -> Option<NaiveDate> {
    let text = text.replace(',', " ");
    let mut fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() == 4 {
        fields.remove(0);
    }
    let [first, second, year] = fields[..] else {
        return None;
    };
    let month = |name: &str| {
        MONTHS
            .iter()
            .position(|month| month.eq_ignore_ascii_case(name))
            .map(|at| at as u32 + 1)
    };
    let (day, month) = match (first.parse::<u32>(), month(second)) {
        (Ok(day), Some(month)) => (day, month),
        _ => (second.parse().ok()?, month(first)?),
    };
    NaiveDate::from_ymd_opt(year.parse().ok()?, month, day)
}

/// The opening tags in `html`: each one's name and the text inside `<…>`,
/// with where it ends.
fn opening_tags(html: &str) -> impl Iterator<Item = (String, &str, usize)> {
    let mut cursor = 0;
    std::iter::from_fn(move || {
        loop {
            let open = cursor + html[cursor..].find('<')?;
            let gt = open + html[open..].find('>')?;
            cursor = gt + 1;
            let inside = &html[open + 1..gt];
            let name: String = inside
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            if !name.is_empty() {
                return Some((name, inside, gt + 1));
            }
        }
    })
}

/// The value of `name` in a tag's attributes, if it has one.
fn attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find(name) {
        let at = from + rel;
        from = at + name.len();
        let before = lower[..at].chars().next_back();
        if !before.is_some_and(char::is_whitespace) {
            continue;
        }
        let rest = lower[from..].trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let start = lower.len() - rest.trim_start().len();
        let quote = tag[start..].chars().next()?;
        return Some(if quote == '"' || quote == '\'' {
            let value = &tag[start + 1..];
            value[..value.find(quote).unwrap_or(value.len())].to_owned()
        } else {
            let value = &tag[start..];
            value[..value.find(char::is_whitespace).unwrap_or(value.len())]
                .trim_end_matches('/')
                .to_owned()
        });
    }
    None
}

/// The inside of each element whose class list has one of `classes` (whole
/// names, any case), with the class matched, in document order.
fn elements_with_classes(html: &str, classes: &[&str]) -> Vec<(String, String)> {
    let lower = html.to_ascii_lowercase();
    let mut found = Vec::new();
    let mut skip_to = 0;
    for (name, tag, end) in opening_tags(html) {
        if end <= skip_to {
            continue;
        }
        let Some(list) = attribute(tag, "class") else {
            continue;
        };
        let Some(class) = list.split_whitespace().find_map(|token| {
            classes
                .iter()
                .find(|class| class.eq_ignore_ascii_case(token))
        }) else {
            continue;
        };
        let Some(close) = lower[end..].find(&format!("</{name}")) else {
            continue;
        };
        found.push(((*class).to_owned(), html[end..end + close].to_owned()));
        skip_to = end + close;
    }
    found
}

/// The local files an entry's images, videos and recordings point at.
fn media_sources(html: &str) -> Vec<String> {
    let mut sources = Vec::new();
    for (name, tag, _) in opening_tags(html) {
        if !matches!(name.as_str(), "img" | "video" | "audio" | "source") {
            continue;
        }
        let Some(src) = attribute(tag, "src") else {
            continue;
        };
        let src = src.trim();
        let remote = src.contains("://") || src.starts_with("data:") || src.starts_with('/');
        if !src.is_empty() && !remote && !sources.iter().any(|s| s == src) {
            sources.push(src.to_owned());
        }
    }
    sources
}

/// The text in `html`, one line per run of text between tags.
fn strip_tags_to_lines(html: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for run in html.split('<') {
        let text = run.split_once('>').map_or(run, |(_, text)| text);
        let text = strip_tags(text);
        let text = text.trim();
        if !text.is_empty() {
            lines.push(text.to_owned());
        }
    }
    lines
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

    fn file() -> TreeFile {
        TreeFile {
            path: "2024/Entry.html".into(),
            size: 0,
            modified: None,
            created: None,
        }
    }

    #[test]
    fn extracts_prompt_paragraphs_date_and_details() {
        let html = r#"<h2 class="pageHeader">Monday, 1 January 2024</h2><div class="reflectionPrompt">What mattered?</div><p class="p2">First paragraph.</p><p class="p3">Second paragraph.</p><div class="assetGrid"><span>San Francisco</span><span>Walk</span></div>"#;
        let note = parse_entry(&file(), html);
        assert_eq!(note.created, Some(1_704_110_400_000));
        assert!(note.body.contains("What mattered?"));
        assert!(note.body.contains("First paragraph."));
        assert!(note.body.ends_with("Journal details: San Francisco, Walk"));
        assert!(note.tags.is_empty());
    }

    #[test]
    fn keeps_paragraph_order_and_whole_class_names() {
        let html = r#"<p class="p3">One.</p><p class="p2 s1">Two.</p><p class="p20">Not text.</p><p class="x">p2</p><p class='p3'>Three.</p>"#;
        let body = parse_entry(&file(), html).body;
        let at = |text: &str| body.find(text).unwrap();
        assert!(at("One.") < at("Two.") && at("Two.") < at("Three."));
        assert!(!body.contains("Not text.") && !body.contains("p2"));
    }

    #[test]
    fn links_the_entrys_media() {
        let html = r#"<p class="p2">Look.</p><div class="assetGrid"><img src="../Resources/A B.jpeg"><video><source src='../Resources/clip.mov'></video><img src="https://e.com/x.png"></div>"#;
        let body = parse_entry(&file(), html).body;
        assert!(body.contains("![](../Resources/A%20B.jpeg)"));
        assert!(body.contains("![](../Resources/clip.mov)"));
        assert!(!body.contains("e.com"));
    }

    #[test]
    fn reads_dates_written_either_way() {
        let day = NaiveDate::from_ymd_opt(2024, 1, 1);
        for text in [
            "1 January 2024",
            "Monday, 1 January 2024",
            "January 1, 2024",
            "Monday, January 1, 2024",
        ] {
            assert_eq!(parse_entry_date(text), day, "{text}");
        }
        assert_eq!(parse_entry_date("Someday"), None);
    }
}
