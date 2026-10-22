//! Imports Roam Research JSON graph exports with stable page UIDs.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use serde_json::Value;

use super::tree::{Tree, TreeFile, extension};
use super::{Skipped, Source, SourceNote};

const MAX_GRAPH: u64 = 32 * 1024 * 1024;

pub struct RoamSource {
    tree: Tree,
    graph: TreeFile,
    files: Vec<TreeFile>,
    pages: Vec<Value>,
    titles: HashMap<String, String>,
    blocks: HashMap<String, Block>,
    skipped: Vec<Skipped>,
    name: String,
}

/// A block that `((uid))` references can point at.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Block {
    /// The UID of the page it's on.
    page: String,
    /// Its `string`, as Roam wrote it.
    text: String,
}

/// Opens a local Roam JSON graph export or a folder containing one.
pub fn open(path: &Path) -> anyhow::Result<RoamSource> {
    let mut tree = if path.is_dir() {
        Tree::dir(path)?
    } else {
        Tree::Single(path.to_owned())
    };
    let listed = tree.files()?;
    let graph = listed
        .iter()
        .find(|f| extension(&f.path) == "json")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no Roam JSON graph export found"))?;
    anyhow::ensure!(graph.size <= MAX_GRAPH, "Roam graph is too large");
    let bytes = tree.read(&graph.path, MAX_GRAPH)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| anyhow::anyhow!("invalid Roam graph JSON: {e}"))?;
    let pages = value
        .as_array()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Roam graph must be a JSON array"))?;
    let mut titles = HashMap::new();
    let mut blocks = HashMap::new();
    for page in &pages {
        if let Some(uid) = page.get("uid").and_then(Value::as_str) {
            let title = page
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Untitled");
            titles.insert(title.to_lowercase(), uid.to_owned());
            collect_blocks(page.get("children"), uid, &mut blocks);
        }
    }
    let files = listed
        .into_iter()
        .filter(|f| f.path != graph.path)
        .collect();
    Ok(RoamSource {
        tree,
        graph,
        files,
        pages,
        titles,
        blocks,
        skipped: Vec::new(),
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Roam")
            .to_owned(),
    })
}

fn collect_blocks(children: Option<&Value>, page_uid: &str, map: &mut HashMap<String, Block>) {
    if let Some(blocks) = children.and_then(Value::as_array) {
        for block in blocks {
            if let Some(uid) = block.get("uid").and_then(Value::as_str) {
                let text = block.get("string").and_then(Value::as_str).unwrap_or("");
                map.insert(
                    uid.to_owned(),
                    Block {
                        page: page_uid.to_owned(),
                        text: text.to_owned(),
                    },
                );
            }
            collect_blocks(block.get("children"), page_uid, map);
        }
    }
}

/// A page's `(created, updated)`: its own `create-time` and `edit-time`
/// (milliseconds since the epoch), else its blocks' earliest and latest.
fn page_times(page: &Value) -> (Option<i64>, Option<i64>) {
    fn walk(children: Option<&Value>, created: &mut Option<i64>, updated: &mut Option<i64>) {
        for block in children.and_then(Value::as_array).into_iter().flatten() {
            if let Some(at) = block.get("create-time").and_then(Value::as_i64) {
                *created = Some(created.map_or(at, |c| c.min(at)));
            }
            if let Some(at) = block.get("edit-time").and_then(Value::as_i64) {
                *updated = Some(updated.map_or(at, |u| u.max(at)));
            }
            walk(block.get("children"), created, updated);
        }
    }
    let (mut created, mut updated) = (None, None);
    walk(page.get("children"), &mut created, &mut updated);
    let own = |key: &str| page.get(key).and_then(Value::as_i64);
    (
        own("create-time").or(created),
        // A block edited later than the page's own time edited the page.
        own("edit-time").max(updated),
    )
}

impl Source for RoamSource {
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
        for page in self.pages.clone() {
            let Some(uid) = page.get("uid").and_then(Value::as_str) else {
                let item = Skipped::new(&self.graph.path, "Roam page has no UID");
                if !self
                    .skipped
                    .iter()
                    .any(|existing| existing.path == item.path && existing.reason == item.reason)
                {
                    self.skipped.push(item);
                }
                continue;
            };
            let title = page
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Untitled")
                .to_owned();
            let mut tags = Vec::new();
            let body = render_blocks(
                page.get("children")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]),
                0,
                &self.titles,
                &self.blocks,
                &self.files,
                &mut tags,
            );
            let (created, updated) = page_times(&page);
            for link in external_local_refs(&body) {
                let item = Skipped::new(
                    link,
                    "Roam local attachment reference is unavailable in this export",
                );
                if !self
                    .skipped
                    .iter()
                    .any(|existing| existing.path == item.path && existing.reason == item.reason)
                {
                    self.skipped.push(item);
                }
            }
            each(SourceNote {
                key: uid.to_owned(),
                title,
                body: body.trim().to_owned(),
                tags,
                created,
                updated,
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

fn render_blocks(
    blocks: &[Value],
    depth: usize,
    titles: &HashMap<String, String>,
    ids: &HashMap<String, Block>,
    files: &[TreeFile],
    tags: &mut Vec<String>,
) -> String {
    let mut lines = Vec::new();
    for block in blocks {
        let raw = block.get("string").and_then(Value::as_str).unwrap_or("");
        if raw.contains("{{[[table]]}}") || raw.contains("{{table}}") {
            if let Some(table) = render_table(block, titles, ids, files, tags) {
                lines.extend(table.lines().map(str::to_owned));
            }
            continue;
        }
        let todo = raw.contains("{{[[TODO]]}}");
        let done = raw.contains("{{[[DONE]]}}");
        let raw = raw.replace("{{[[TODO]]}}", "").replace("{{[[DONE]]}}", "");
        let text = convert_refs(&raw, titles, ids, files, tags);
        let level = block
            .get("heading")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(6) as usize;
        let marker = if todo {
            "- [ ]"
        } else if done {
            "- [x]"
        } else {
            "-"
        };
        let text = text.trim();
        let (first, rest) = text.split_once('\n').unwrap_or((text, ""));
        let rendered = if level > 0 {
            format!("{} {}", "#".repeat(level), first.trim_end())
        } else {
            format!("{}{} {}", "  ".repeat(depth), marker, first.trim_end())
        };
        if !rendered.trim().is_empty() {
            lines.push(rendered);
        }
        // A block's later lines stay in its list item, under its text.
        let indent = if level > 0 {
            String::new()
        } else {
            "  ".repeat(depth + 1)
        };
        for line in rest.lines() {
            lines.push(if line.trim().is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            });
        }
        if let Some(children) = block.get("children").and_then(Value::as_array) {
            let nested = render_blocks(children, depth + 1, titles, ids, files, tags);
            if !nested.is_empty() {
                lines.extend(nested.lines().map(str::to_owned));
            }
        }
    }
    lines.join("\n")
}

fn render_table(
    block: &Value,
    titles: &HashMap<String, String>,
    ids: &HashMap<String, Block>,
    files: &[TreeFile],
    tags: &mut Vec<String>,
) -> Option<String> {
    let rows = block.get("children")?.as_array()?;
    if rows.is_empty() {
        return None;
    }
    let mut cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let mut values = Vec::new();
            let mut current = row;
            loop {
                values.push(
                    convert_refs(
                        current.get("string").and_then(Value::as_str).unwrap_or(""),
                        titles,
                        ids,
                        files,
                        tags,
                    )
                    .replace('|', "\\|")
                    .replace('\n', "<br>")
                    .trim()
                    .to_owned(),
                );
                let Some(child) = current
                    .get("children")
                    .and_then(Value::as_array)
                    .and_then(|children| children.first())
                else {
                    break;
                };
                current = child;
            }
            values
        })
        .collect();
    let columns = cells.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return None;
    }
    for row in &mut cells {
        row.resize(columns, String::new());
    }
    let mut rendered = Vec::new();
    rendered.push(format!("| {} |", cells[0].join(" | ")));
    rendered.push(format!("| {} |", vec!["---"; columns].join(" | ")));
    rendered.extend(
        cells
            .iter()
            .skip(1)
            .map(|row| format!("| {} |", row.join(" | "))),
    );
    Some(rendered.join("\n"))
}

fn convert_refs(
    input: &str,
    titles: &HashMap<String, String>,
    ids: &HashMap<String, Block>,
    files: &[TreeFile],
    tags: &mut Vec<String>,
) -> String {
    let input = expand_block_refs(input, ids);
    let mut out = String::new();
    let mut cursor = 0;
    while let Some(rel) = input[cursor..].find("[[") {
        let start = cursor + rel;
        out.push_str(&input[cursor..start]);
        let Some(close_rel) = input[start + 2..].find("]]") else {
            out.push_str(&input[start..]);
            return out;
        };
        let end = start + 2 + close_rel;
        let label = &input[start + 2..end];
        let is_tag = start > 0 && input[..start].ends_with('#');
        // `[label]([[Page]])`: Roam's alias, a link to the page.
        let is_alias = out.ends_with("](") && input[end + 2..].starts_with(')');
        let target = titles
            .get(&label.to_lowercase())
            .map(|uid| format!("kanso-import:note/{uid}"))
            .or_else(|| {
                files
                    .iter()
                    .find(|file| file.path == label || super::tree::file_name(&file.path) == label)
                    .map(|file| format!("kanso-import:path/{}", file.path))
            });
        cursor = end + 2;
        if is_tag {
            if !tags.iter().any(|t| t.eq_ignore_ascii_case(label)) {
                tags.push(label.to_owned());
            }
            out.push_str(label);
        } else if is_alias {
            if let Some(target) = target {
                out.push_str(&target);
            } else if let Some(open) = out[..out.len() - 2].rfind('[') {
                // A page the export doesn't have: just the alias's text.
                let text = out[open + 1..out.len() - 2].to_owned();
                out.truncate(open);
                out.push_str(&text);
                cursor += 1;
            } else {
                out.push_str(label);
            }
        } else if let Some(target) = target {
            out.push_str(&format!("[{label}]({target})"));
        } else {
            out.push_str(label);
        }
    }
    out.push_str(&input[cursor..]);
    out.replace("{{[[TODO]]}}", "- [ ]")
        .replace("{{[[DONE]]}}", "- [x]")
}

/// `((uid))` block references as the referenced block's text, or, as a
/// link's destination (`[label](((uid)))`), as a link to the block's page.
/// Only one level is expanded: references in the inserted text, and to
/// blocks the export doesn't have, stay as written.
fn expand_block_refs(input: &str, blocks: &HashMap<String, Block>) -> String {
    let mut out = String::new();
    let mut at = 0;
    while let Some(rel) = input[at..].find("((") {
        let mut start = at + rel;
        // In `(((uid)))` the first `(` is the link's.
        while input[start + 2..].starts_with('(') {
            start += 1;
        }
        out.push_str(&input[at..start]);
        let Some(end_rel) = input[start + 2..].find("))") else {
            at = start;
            break;
        };
        let end = start + 2 + end_rel;
        match blocks.get(&input[start + 2..end]) {
            Some(block) if out.ends_with("](") && input[end + 2..].starts_with(')') => {
                out.push_str(&format!("kanso-import:note/{}", block.page));
            }
            Some(block) => {
                let text = block
                    .text
                    .replace("{{[[TODO]]}}", "")
                    .replace("{{[[DONE]]}}", "");
                out.push_str(&text.trim().replace('\n', " "));
            }
            None => out.push_str(&input[start..end + 2]),
        }
        at = end + 2;
    }
    out.push_str(&input[at..]);
    out
}

fn external_local_refs(body: &str) -> HashSet<String> {
    body.split_whitespace()
        .filter(|s| {
            s.starts_with("file://") || s.starts_with("https://firebasestorage.googleapis.com/")
        })
        .map(|s| s.trim_matches(['(', ')', ']']).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_nested_todos_page_refs_and_tags() {
        let titles = HashMap::from([("target".into(), "page-2".into())]);
        let ids = HashMap::new();
        let mut tags = Vec::new();
        let blocks = vec![
            serde_json::json!({"string":"{{[[TODO]]}} Read [[Target]] #[[work]]","children":[{"string":"child"}]}),
        ];
        let md = render_blocks(&blocks, 0, &titles, &ids, &[], &mut tags);
        assert!(md.contains("- [ ] Read [Target](kanso-import:note/page-2)"));
        assert!(md.contains("  - child"));
        assert_eq!(tags, ["work"]);
    }

    #[test]
    fn renders_roam_table_rows_as_markdown() {
        let block = serde_json::json!({"string":"{{[[table]]}}","children":[{"string":"Name","children":[{"string":"Colour"}]},{"string":"Apple","children":[{"string":"Red"}]}]});
        let mut tags = Vec::new();
        assert_eq!(
            render_table(&block, &HashMap::new(), &HashMap::new(), &[], &mut tags).as_deref(),
            Some("| Name | Colour |\n| --- | --- |\n| Apple | Red |")
        );
    }

    #[test]
    fn turns_local_file_page_references_into_attachment_links() {
        let files = [TreeFile {
            path: "assets/reading.pdf".into(),
            size: 4,
            modified: None,
            created: None,
        }];
        let mut tags = Vec::new();
        assert_eq!(
            convert_refs(
                "Open [[reading.pdf]]",
                &HashMap::new(),
                &HashMap::new(),
                &files,
                &mut tags,
            ),
            "Open [reading.pdf](kanso-import:path/assets/reading.pdf)"
        );
    }
}
