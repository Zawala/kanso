//! Notion's "Markdown & CSV" export: a zip (or a zip of zips, for large
//! workspaces) of pages named `Title <32 hex>.md`, with each page's
//! sub-pages and files in a folder of the same name. Databases are
//! `Title <32 hex>.csv` (and `_all.csv`), their rows pages in the folder.
//!
//! The IDs leave names and titles; a page's sub-pages go into a notebook
//! named after it; a database becomes a note holding its table, whose first
//! column links its rows.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

use anyhow::Context as _;

use crate::library::resolve::is_note_path;
use crate::library::tree::{Tree, TreeFile, extension, file_name, parent_of, stem};
use crate::library::{Skipped, Source, SourceNote, dates, frontmatter};

pub struct NotionSource {
    tree: Tree,
    /// Stripped from the tree's paths (an export's single top folder).
    prefix: String,
    /// Inner zips of a split export, extracted for reading.
    _parts: Option<tempfile::TempDir>,
    pages: Vec<TreeFile>,
    files: Vec<TreeFile>,
    skipped: Vec<Skipped>,
    /// A page's or database's ID → its key, for `notion.so` links.
    ids: HashMap<String, String>,
    /// Lowercased cleaned titles of the pages in each folder, for linking
    /// a database's rows.
    titles_in: HashMap<String, HashSet<String>>,
}

/// Note files larger than this are skipped.
const MAX_PAGE_FILE: u64 = 8 * 1024 * 1024;

pub fn open(path: &Path) -> anyhow::Result<NotionSource> {
    NotionSource::open(path)
}

impl NotionSource {
    pub fn open(path: &Path) -> anyhow::Result<NotionSource> {
        let (mut tree, parts) = if path.is_dir() {
            (Tree::dir(path)?, None)
        } else {
            open_zip(path)?
        };
        let mut listed = tree.files()?;
        let mut prefix = String::new();
        if let Some(top) = single_top_folder(&listed) {
            let page = format!("{top}.md");
            if !listed.iter().any(|f| f.path == page) {
                for file in &mut listed {
                    file.path = file.path[top.len() + 1..].to_owned();
                }
                prefix = format!("{top}/");
            }
        }
        let paths: HashSet<&str> = listed.iter().map(|f| f.path.as_str()).collect();
        let mut pages = Vec::new();
        let mut files = Vec::new();
        let mut skipped = Vec::new();
        let mut ids = HashMap::new();
        let mut titles_in: HashMap<String, HashSet<String>> = HashMap::new();
        for file in &listed {
            let name = file_name(&file.path);
            let ext = extension(name);
            if ext == "csv" {
                // `X_all.csv` holds every row; `X.csv` only the view's.
                let all = format!("{}_all.csv", stem(&file.path));
                if paths.contains(all.as_str()) {
                    continue;
                }
            }
            if let Some(id) = notion_id(stem(name).trim_end_matches("_all")) {
                ids.insert(id.to_owned(), file.path.clone());
            }
            if is_note_path(name) || ext == "csv" {
                if file.size > MAX_PAGE_FILE {
                    skipped.push(Skipped::new(file.path.clone(), "too large for a note"));
                    continue;
                }
                if ext != "csv" {
                    titles_in
                        .entry(parent_of(&file.path).to_owned())
                        .or_default()
                        .insert(clean_name(stem(name)).to_lowercase());
                }
                pages.push(file.clone());
            } else {
                files.push(file.clone());
            }
        }
        Ok(NotionSource {
            tree,
            prefix,
            _parts: parts,
            pages,
            files,
            skipped,
            ids,
            titles_in,
        })
    }
}

/// A zip, or the parts of one: Notion splits large exports into a zip
/// holding `…-Part-1.zip`, `…-Part-2.zip` and so on.
fn open_zip(path: &Path) -> anyhow::Result<(Tree, Option<tempfile::TempDir>)> {
    let mut outer = Tree::zip(path)?;
    let listed = outer.files()?;
    let inner: Vec<&TreeFile> = listed
        .iter()
        .filter(|f| extension(&f.path) == "zip")
        .collect();
    if inner.is_empty() || inner.len() != listed.len() {
        return Ok((outer, None));
    }
    let temp = tempfile::Builder::new()
        .prefix("kanso-notion-")
        .tempdir()
        .context("cannot create a temporary folder")?;
    let mut trees = Vec::new();
    for (ix, part) in inner.iter().enumerate() {
        let to = temp.path().join(format!("part-{ix}.zip"));
        outer
            .copy_to(&part.path, &to)
            .with_context(|| format!("cannot unpack {}", part.path))?;
        trees.push(Tree::zip(&to)?);
    }
    Ok((Tree::many(trees)?, Some(temp)))
}

/// The one folder every path is in, if there is one.
fn single_top_folder(files: &[TreeFile]) -> Option<String> {
    let (top, _) = files.first()?.path.split_once('/')?;
    files
        .iter()
        .all(|f| f.path.starts_with(&format!("{top}/")))
        .then(|| top.to_owned())
}

/// The 32-hex-digit ID at the end of a Notion name: `Plan 0f3c…` → `0f3c…`.
pub fn notion_id(name: &str) -> Option<&str> {
    let (_, id) = name.rsplit_once(' ')?;
    (id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())).then_some(id)
}

/// A Notion file or folder name without its ID: `Plan 0f3c….md` → `Plan.md`,
/// `Plan 0f3c…` → `Plan`. Names without one are left as they are.
pub fn clean_name(name: &str) -> String {
    let (base, ext) = match name.rsplit_once('.') {
        Some((base, ext)) if notion_id(base).is_some() => (base, Some(ext)),
        _ => (name, None),
    };
    let base = match notion_id(base) {
        Some(id) => base[..base.len() - id.len()].trim_end(),
        None => base,
    };
    let base = if base.is_empty() { "Untitled" } else { base };
    match ext {
        Some(ext) => format!("{base}.{ext}"),
        None => base.to_owned(),
    }
}

/// The notebook path a page at `path` belongs in: its folders, without IDs.
/// A page's sub-pages are in a folder named after it, so they land in a
/// notebook of that name.
pub fn folder_of(path: &str) -> Vec<String> {
    parent_of(path)
        .split('/')
        .filter(|p| !p.is_empty())
        .map(clean_name)
        .collect()
}

impl Source for NotionSource {
    fn name(&self) -> String {
        "Notion".to_owned()
    }

    fn files(&self) -> &[TreeFile] {
        &self.files
    }

    fn notes(
        &mut self,
        _full: bool,
        each: &mut dyn FnMut(SourceNote) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let pages = self.pages.clone();
        for page in pages {
            let full = format!("{}{}", self.prefix, page.path);
            let text = self
                .tree
                .read_text(&full, MAX_PAGE_FILE)
                .map_err(|err| anyhow::anyhow!("cannot read {}: {err}", page.path))?;
            let note = if extension(&page.path) == "csv" {
                let rows_dir = stem(&page.path).trim_end_matches("_all").to_owned();
                let rows = self.titles_in.get(&rows_dir);
                database_note(&page, &text, rows)
            } else {
                page_note(&page, &text, &self.ids)
            };
            each(note)?;
        }
        Ok(())
    }

    fn read(&mut self, path: &str, limit: u64) -> io::Result<Vec<u8>> {
        let full = format!("{}{path}", self.prefix);
        self.tree.read(&full, limit)
    }

    fn skipped(&mut self) -> Vec<Skipped> {
        std::mem::take(&mut self.skipped)
    }
}

/// A page: `# Title`, then (for a database's rows) `Property: value` lines,
/// then the content.
pub fn page_note(file: &TreeFile, text: &str, ids: &HashMap<String, String>) -> SourceNote {
    let mut body = text;
    let mut meta = frontmatter::Meta::default();
    if let Some((front, rest)) = frontmatter::split(text) {
        meta = frontmatter::meta(front);
        body = rest;
    }
    let body = body.trim_start_matches('\n');
    let (title, body) = match body.split_once('\n') {
        Some((first, rest)) if first.starts_with("# ") => (first[2..].trim().to_owned(), rest),
        None if body.starts_with("# ") => (body[2..].trim().to_owned(), ""),
        _ => (clean_name(stem(file_name(&file.path))), body),
    };
    let body = body.trim_start_matches('\n');
    let mut tags = meta.tags;
    let mut created = meta.created.as_deref().and_then(dates::parse);
    let mut updated = meta.updated.as_deref().and_then(dates::parse);
    for line in body.lines().take_while(|l| !l.trim().is_empty()) {
        let Some((key, value)) = line.split_once(": ") else {
            break;
        };
        match key.trim().to_lowercase().as_str() {
            "tags" | "tag" | "labels" => tags.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(str::to_owned),
            ),
            "created" | "created time" | "date created" => {
                created = created.or_else(|| parse_notion_date(value))
            }
            "last edited time" | "updated" | "last edited" => {
                updated = updated.or_else(|| parse_notion_date(value))
            }
            _ => {}
        }
    }
    let mut body = rewrite_notion_urls(body, ids);
    if !meta.rest.is_empty() {
        body = format!("---\n{}---\n\n{body}", meta.rest);
    }
    SourceNote {
        key: file.path.clone(),
        path: Some(file.path.clone()),
        folder: folder_of(&file.path),
        title,
        body,
        tags,
        created: created.or(file.created).or(file.modified),
        updated: updated.or(file.modified),
        ..SourceNote::default()
    }
}

/// `https://www.notion.so/Title-<id>` links to pages in the export →
/// links to their notes.
fn rewrite_notion_urls(body: &str, ids: &HashMap<String, String>) -> String {
    if !body.contains("notion.so/") {
        return body.to_owned();
    }
    let found = crate::library::links::scan(body);
    let edits = found
        .into_iter()
        .filter_map(|f| {
            let url = f.target.split(['?', '#']).next()?;
            if !url.contains("notion.so/") {
                return None;
            }
            let tail = url.rsplit(['/', '-']).next()?;
            let key = ids.get(tail)?;
            Some((f.range, format!("kanso-import:note/{key}")))
        })
        .collect();
    crate::library::links::apply(body, edits)
}

/// Notion's dates: `March 1, 2024 9:30 AM`, `March 1, 2024`, or ISO.
fn parse_notion_date(text: &str) -> Option<i64> {
    let text = text.trim();
    for pattern in ["%B %e, %Y %l:%M %p", "%B %d, %Y %I:%M %p"] {
        if let Ok(at) = chrono::NaiveDateTime::parse_from_str(text, pattern) {
            return dates::local(at);
        }
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(text, "%B %e, %Y") {
        return dates::local(date.and_hms_opt(0, 0, 0)?);
    }
    dates::parse(text)
}

/// A database: a note holding its table. A row's first cell links its page
/// when the export has one.
pub fn database_note(file: &TreeFile, csv: &str, rows: Option<&HashSet<String>>) -> SourceNote {
    let name = stem(file_name(&file.path)).trim_end_matches("_all");
    let records = parse_csv(csv);
    let body = markdown_table(&records, |cell| {
        rows.is_some_and(|titles| titles.contains(&cell.trim().to_lowercase()))
    });
    // Pages link the view's `X.csv`; the note is made from `X_all.csv`.
    let linked_as = match file.path.strip_suffix("_all.csv") {
        Some(base) => format!("{base}.csv"),
        None => file.path.clone(),
    };
    SourceNote {
        key: file.path.clone(),
        path: Some(linked_as),
        folder: folder_of(&file.path),
        title: clean_name(name),
        body,
        created: file.created.or(file.modified),
        updated: file.modified,
        ..SourceNote::default()
    }
}

/// Records of RFC 4180 CSV: quoted fields may hold commas, quotes (`""`)
/// and line breaks. A byte order mark is dropped.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    let mut any = false;
    while let Some(c) = chars.next() {
        any = true;
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => quoted = false,
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => quoted = true,
            ',' => record.push(std::mem::take(&mut field)),
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
                any = false;
            }
            _ => field.push(c),
        }
    }
    if any {
        record.push(field);
        records.push(record);
    }
    records
}

/// A Markdown table of `records`, the first being the header. Cells are
/// one line (`<br>` for breaks), with `|` escaped; `link` says whether a
/// first-column cell names a page to link with `[[…]]`.
pub fn markdown_table(records: &[Vec<String>], link: impl Fn(&str) -> bool) -> String {
    let Some(header) = records.first() else {
        return String::new();
    };
    let width = records.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let cell = |text: &str| {
        let text = text.trim().replace('|', "\\|").replace("\r\n", "\n");
        text.split('\n').collect::<Vec<_>>().join("<br>")
    };
    let row = |cells: &[String], first_links: bool| {
        let mut line = String::from("|");
        for ix in 0..width {
            let text = cells.get(ix).map_or(String::new(), |c| cell(c));
            let text = if ix == 0
                && first_links
                && !text.is_empty()
                && link(&cells[0])
                && !text.contains(['[', ']', '#'])
            {
                format!("[[{text}]]")
            } else {
                text
            };
            line.push(' ');
            line.push_str(&text);
            line.push_str(" |");
        }
        line
    };
    let mut out = vec![row(header, false)];
    out.push(format!("|{}", " --- |".repeat(width)));
    for record in &records[1..] {
        out.push(row(record, true));
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "0f3c5e2a9b8d4c1e8f7a6b5c4d3e2f1a";

    #[test]
    fn names_lose_their_ids() {
        assert_eq!(clean_name(&format!("Plan {ID}.md")), "Plan.md");
        assert_eq!(clean_name(&format!("Plan {ID}")), "Plan");
        assert_eq!(
            clean_name(&format!("Q3 roadmap v2 {ID}.csv")),
            "Q3 roadmap v2.csv"
        );
        assert_eq!(clean_name("photo.png"), "photo.png");
        assert_eq!(clean_name(ID), ID);
        assert_eq!(clean_name("Plan abc.md"), "Plan abc.md");
        assert_eq!(
            folder_of(&format!("Work {ID}/Plan {ID}/Sub {ID}.md")),
            ["Work", "Plan"]
        );
    }

    #[test]
    fn a_page() {
        let file = TreeFile {
            path: format!("Work {ID}/Task {ID}.md"),
            size: 0,
            modified: None,
            created: None,
        };
        let text = "# Task\n\nTags: urgent, home\nCreated: March 1, 2024 9:30 AM\n\nDo it.\n";
        let note = page_note(&file, text, &HashMap::new());
        assert_eq!(note.title, "Task");
        assert_eq!(note.folder, ["Work"]);
        assert_eq!(note.tags, ["urgent", "home"]);
        // Notion writes local times.
        let local = chrono::NaiveDate::from_ymd_opt(2024, 3, 1)
            .and_then(|d| d.and_hms_opt(9, 30, 0))
            .and_then(dates::local);
        assert_eq!(note.created, local);
        assert!(note.body.starts_with("Tags: urgent"));
    }

    #[test]
    fn notion_urls_become_note_links() {
        let mut ids = HashMap::new();
        ids.insert(ID.to_owned(), format!("Plan {ID}.md"));
        let body =
            format!("[Plan](https://www.notion.so/Plan-{ID}?pvs=4) [x](https://example.com)");
        assert_eq!(
            rewrite_notion_urls(&body, &ids),
            format!("[Plan](kanso-import:note/Plan {ID}.md) [x](https://example.com)")
        );
    }

    #[test]
    fn csv_and_tables() {
        let records =
            parse_csv("Name,Notes\r\n\"Task, one\",\"say \"\"hi\"\"\nthen go\"\nTwo,a|b\n");
        assert_eq!(
            records,
            [
                vec!["Name".to_owned(), "Notes".to_owned()],
                vec!["Task, one".to_owned(), "say \"hi\"\nthen go".to_owned()],
                vec!["Two".to_owned(), "a|b".to_owned()],
            ]
        );
        assert_eq!(
            markdown_table(&records, |c| c == "Two"),
            "| Name | Notes |\n| --- | --- |\n| Task, one | say \"hi\"<br>then go |\n| [[Two]] | a\\|b |\n"
        );
    }
}
