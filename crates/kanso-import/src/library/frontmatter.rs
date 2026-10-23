//! YAML front matter, as far as notes use it: `key: value` lines with plain
//! or quoted scalars, `[inline, lists]`, block lists (`- item`), and `|`/`>`
//! block scalars. Anything else (nested maps, anchors) is kept as written
//! and not interpreted.
//!
//! Kanso keeps a note's title, tags and dates in fields of their own, so
//! those keys leave the front matter on import; every other line stays in
//! the body exactly as it was.

/// A front-matter value as far as it is understood.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Scalar(String),
    List(Vec<String>),
    /// A nested map or anything else: not interpreted.
    Other,
}

/// One top-level key and the lines it spans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    /// The entry's lines as written, each ending in `\n`.
    pub raw: String,
}

/// A body split at its front matter: the text between the `---` lines,
/// and what follows the closing line. `None` without front matter.
pub fn split(text: &str) -> Option<(&str, &str)> {
    // The same detection the properties editor uses: a BOM and trailing
    // whitespace on the marker lines are allowed.
    let front = kanso_core::properties::split(text)?;
    Some((
        &text[front.front_start..front.close_start],
        &text[front.body_start..],
    ))
}

/// The entries of front matter (the text between the `---` lines). Lines
/// before the first key (comments, blank lines) belong to no entry and
/// are dropped from the result; lines after an entry belong to it.
pub fn parse(front: &str) -> Vec<Entry> {
    let mut entries: Vec<(String, Vec<&str>)> = Vec::new();
    let mut leading: Vec<&str> = Vec::new();
    for line in front.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        // Core's entry parsing decides what a key is, so `-k: v` is a key
        // and `- item` is not.
        let starts_key = !content.starts_with([' ', '\t', '#'])
            && !content.is_empty()
            && key_of(content).is_some();
        if starts_key {
            let key = key_of(content).unwrap_or_default();
            entries.push((key, vec![line]));
        } else if let Some((_, lines)) = entries.last_mut() {
            lines.push(line);
        } else {
            leading.push(line);
        }
    }
    entries
        .into_iter()
        .map(|(key, lines)| {
            let mut raw: String = lines.concat();
            if !raw.ends_with('\n') {
                raw.push('\n');
            }
            Entry {
                value: value_of(&lines),
                key,
                raw,
            }
        })
        .collect()
}

/// The key of a `key: value` line, unquoted.
fn key_of(line: &str) -> Option<String> {
    split_key(line).map(|(key, _)| key)
}

/// `key: rest` → (key, rest). The properties editor and the importer agree
/// on what a key is.
fn split_key(line: &str) -> Option<(String, &str)> {
    kanso_core::properties::split_key(line)
}

fn value_of(lines: &[&str]) -> Value {
    let first = lines[0].trim_end_matches(['\n', '\r']);
    let Some((_, rest)) = split_key(first) else {
        return Value::Other;
    };
    let rest = strip_comment(rest.trim());
    let more: Vec<&str> = lines[1..]
        .iter()
        .map(|l| l.trim_end_matches(['\n', '\r']))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();
    if rest.starts_with('|') || rest.starts_with('>') {
        let folded = rest.starts_with('>');
        let text: Vec<&str> = lines[1..]
            .iter()
            .map(|l| l.trim_end_matches(['\n', '\r']).trim())
            .collect();
        let text = if folded {
            text.iter()
                .filter(|l| !l.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            text.join("\n").trim_end().to_owned()
        };
        return Value::Scalar(text);
    }
    if rest.is_empty() {
        if more.is_empty() {
            return Value::Scalar(String::new());
        }
        if more.iter().all(|l| l.trim_start().starts_with('-')) {
            return Value::List(
                more.iter()
                    .map(|l| {
                        let item = l.trim_start().strip_prefix('-').unwrap_or("");
                        unquote(strip_comment(item.trim()))
                    })
                    .collect(),
            );
        }
        return Value::Other;
    }
    if !more.is_empty() {
        // A plain scalar continued on the next lines.
        if rest.starts_with(['"', '\'', '[', '{']) {
            return Value::Other;
        }
        let mut text = rest.to_owned();
        for line in more {
            text.push(' ');
            text.push_str(line.trim());
        }
        return Value::Scalar(text);
    }
    if let Some(inner) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        return Value::List(split_flow(inner).into_iter().map(|s| unquote(&s)).collect());
    }
    if rest.starts_with('{') {
        return Value::Other;
    }
    Value::Scalar(unquote(rest))
}

/// A plain scalar without a trailing ` # comment`. Quoted text keeps its `#`.
fn strip_comment(text: &str) -> &str {
    if text.starts_with(['"', '\'']) {
        return text;
    }
    if text.starts_with('#') {
        return "";
    }
    match text.find(" #").or_else(|| text.find("\t#")) {
        Some(at) => text[..at].trim_end(),
        None => text,
    }
}

/// Splits `a, "b, c", d` at the commas outside quotes.
fn split_flow(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) if c == q => {
                if q == '\'' && chars.peek() == Some(&'\'') {
                    current.push('\'');
                    current.push(chars.next().unwrap_or('\''));
                    continue;
                }
                quote = None;
                current.push(c);
            }
            Some('"') if c == '\\' => {
                current.push(c);
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            Some(_) => current.push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                current.push(c);
            }
            None if c == ',' => {
                items.push(current.trim().to_owned());
                current.clear();
            }
            None => current.push(c),
        }
    }
    if !current.trim().is_empty() || !items.is_empty() {
        items.push(current.trim().to_owned());
    }
    items.retain(|item| !item.is_empty());
    items
}

/// A scalar without its quotes: `"a \"b\""` → `a "b"`, `'it''s'` → `it's`.
pub fn unquote(text: &str) -> String {
    let text = text.trim();
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        let inner = &text[1..text.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c != '\\' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        }
        out
    } else if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
        text[1..text.len() - 1].replace("''", "'")
    } else {
        text.to_owned()
    }
}

/// Front-matter text for `entries` as [`parse`] reads them back. Used by
/// tests and to rebuild what remains of a note's front matter.
pub fn render(entries: &[Entry]) -> String {
    entries.iter().map(|e| e.raw.as_str()).collect()
}

/// Quotes `text` for a scalar when it would not read back as itself.
pub fn quote(text: &str) -> String {
    let plain = !text.is_empty()
        && text.trim() == text
        && !text.starts_with([
            '"', '\'', '[', '{', '|', '>', '-', '#', '&', '*', '!', '%', '@',
        ])
        && !text.contains(": ")
        && !text.contains(" #")
        && !text.contains('\n')
        && !text.ends_with(':');
    if plain {
        text.to_owned()
    } else {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{escaped}\"")
    }
}

/// What Kanso takes from a note's front matter.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub status: Option<kanso_core::NoteStatus>,
    pub pinned: bool,
    /// The front matter that remains: every entry Kanso has no field for,
    /// as written. Empty when nothing remains.
    pub rest: String,
}

const TITLE_KEYS: &[&str] = &["title"];
const TAG_KEYS: &[&str] = &["tags", "tag"];
const ALIAS_KEYS: &[&str] = &["aliases", "alias"];
const CREATED_KEYS: &[&str] = &[
    "created",
    "date",
    "created_at",
    "createdat",
    "creation date",
    "date created",
    "creation_date",
];
const STATUS_KEYS: &[&str] = &["status"];
const PINNED_KEYS: &[&str] = &["pinned"];
const UPDATED_KEYS: &[&str] = &[
    "updated",
    "modified",
    "updated_at",
    "updatedat",
    "last_modified",
    "last modified",
    "lastmod",
    "date modified",
    "modification date",
];

/// Reads the keys Kanso uses out of front matter.
pub fn meta(front: &str) -> Meta {
    let mut meta = Meta::default();
    let mut rest = Vec::new();
    // `kanso-*` keys carry the values the user's own keys would hide, and
    // win over them however the front matter is ordered.
    let mut kanso_title: Option<String> = None;
    let mut kanso_tags: Vec<String> = Vec::new();
    let mut kanso_created: Option<String> = None;
    let mut kanso_updated: Option<String> = None;
    let mut kanso_status: Option<kanso_core::NoteStatus> = None;
    let mut kanso_pinned: Option<bool> = None;
    for entry in parse(front) {
        let key = entry.key.to_lowercase();
        let key = key.as_str();
        let used = if key == "kanso-title" {
            match scalar_text(&entry.value) {
                Some(title) => {
                    kanso_title = Some(title);
                    true
                }
                None => false,
            }
        } else if key == "kanso-tags" {
            kanso_tags.extend(tags_of(&entry.value));
            true
        } else if key == "kanso-status" {
            scalar_text(&entry.value)
                .and_then(|text| kanso_core::NoteStatus::parse(&text))
                .map(|status| kanso_status = Some(status))
                .is_some()
        } else if key == "kanso-pinned" {
            bool_of(&entry.value)
                .map(|pinned| kanso_pinned = Some(pinned))
                .is_some()
        } else if key == "kanso-created" {
            date_of(&entry.value)
                .map(|date| kanso_created = Some(date))
                .is_some()
        } else if key == "kanso-updated" {
            date_of(&entry.value)
                .map(|date| kanso_updated = Some(date))
                .is_some()
        } else if TITLE_KEYS.contains(&key) {
            match &entry.value {
                Value::Scalar(title) if !title.trim().is_empty() => {
                    meta.title = Some(title.trim().to_owned());
                    true
                }
                _ => false,
            }
        } else if TAG_KEYS.contains(&key) {
            let tags = tags_of(&entry.value);
            let used = !tags.is_empty() || !matches!(entry.value, Value::Other);
            meta.tags.extend(tags);
            used
        } else if ALIAS_KEYS.contains(&key) {
            match &entry.value {
                Value::Scalar(text) => {
                    meta.aliases.extend(
                        text.split(',')
                            .map(str::trim)
                            .filter(|a| !a.is_empty())
                            .map(str::to_owned),
                    );
                }
                Value::List(items) => meta.aliases.extend(
                    items
                        .iter()
                        .map(|a| a.trim().to_owned())
                        .filter(|a| !a.is_empty()),
                ),
                _ => {}
            }
            // Kanso has no aliases of its own: they stay in the front matter.
            false
        } else if CREATED_KEYS.contains(&key) || UPDATED_KEYS.contains(&key) {
            let slot = if CREATED_KEYS.contains(&key) {
                &mut meta.created
            } else {
                &mut meta.updated
            };
            match date_of(&entry.value) {
                Some(date) => {
                    if slot.is_none() {
                        *slot = Some(date);
                    }
                    true
                }
                None => false,
            }
        } else if STATUS_KEYS.contains(&key) {
            match scalar_text(&entry.value)
                .and_then(|text| kanso_core::NoteStatus::parse(&text))
            {
                Some(status) => {
                    meta.status = Some(status);
                    true
                }
                None => false,
            }
        } else if PINNED_KEYS.contains(&key) {
            match bool_of(&entry.value) {
                Some(pinned) => {
                    meta.pinned = pinned;
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if !used {
            rest.push(entry);
        }
    }
    if let Some(title) = kanso_title {
        meta.title = Some(title);
    }
    meta.tags.extend(kanso_tags);
    if let Some(date) = kanso_created {
        meta.created = Some(date);
    }
    if let Some(date) = kanso_updated {
        meta.updated = Some(date);
    }
    if let Some(status) = kanso_status {
        meta.status = Some(status);
    }
    if let Some(pinned) = kanso_pinned {
        meta.pinned = pinned;
    }
    meta.rest = render(&rest);
    meta
}

/// The text of a scalar value, if it has any.
fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::Scalar(text) if !text.trim().is_empty() => Some(text.trim().to_owned()),
        _ => None,
    }
}

/// The tags a value names: a comma- or whitespace-separated scalar, or a
/// list of them.
fn tags_of(value: &Value) -> Vec<String> {
    match value {
        Value::Scalar(text) => split_tags(text),
        Value::List(items) => items.iter().flat_map(|item| split_tags(item)).collect(),
        Value::Other => Vec::new(),
    }
}

fn bool_of(value: &Value) -> Option<bool> {
    match scalar_text(value)?.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn date_of(value: &Value) -> Option<String> {
    let text = scalar_text(value)?;
    crate::library::dates::parse(&text).map(|_| text)
}

/// Tag names in a scalar: separated by commas or whitespace, without a
/// leading `#`.
pub fn split_tags(text: &str) -> Vec<String> {
    text.split(|c: char| c == ',' || c.is_whitespace())
        .map(|t| t.trim().trim_start_matches('#').trim_end_matches('/'))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_front_matter() {
        assert_eq!(
            split("---\ntitle: A\n---\nBody"),
            Some(("title: A\n", "Body"))
        );
        assert_eq!(split("---\n---\n"), Some(("", "")));
        assert_eq!(split("No front matter"), None);
        assert_eq!(split("---\nunterminated"), None);
    }

    #[test]
    fn reads_obsidian_front_matter() {
        let front = "title: \"Plan: v2\"\ntags:\n  - work\n  - '#project/acme'\naliases: [Roadmap, \"The plan\"]\ncreated: 2024-03-01\ncssclass: wide\nnested:\n  a: 1\n";
        let meta = meta(front);
        assert_eq!(meta.title.as_deref(), Some("Plan: v2"));
        assert_eq!(meta.tags, ["work", "project/acme"]);
        assert_eq!(meta.aliases, ["Roadmap", "The plan"]);
        assert_eq!(meta.created.as_deref(), Some("2024-03-01"));
        assert_eq!(
            meta.rest,
            "aliases: [Roadmap, \"The plan\"]\ncssclass: wide\nnested:\n  a: 1\n"
        );
    }

    #[test]
    fn tags_as_a_scalar() {
        assert_eq!(meta("tags: a, b c #d\n").tags, ["a", "b", "c"]);
        assert_eq!(meta("tags: \"#x, y\"\n").tags, ["x", "y"]);
    }

    #[test]
    fn block_scalars_and_comments() {
        assert_eq!(
            parse("title: >\n  A long\n  title\n")[0].value,
            Value::Scalar("A long title".into())
        );
        assert_eq!(
            parse("note: |\n  one\n  two\n")[0].value,
            Value::Scalar("one\ntwo".into())
        );
        assert_eq!(
            parse("a: b # comment\n")[0].value,
            Value::Scalar("b".into())
        );
        assert_eq!(
            parse("url: http://x.com/a#b\n")[0].value,
            Value::Scalar("http://x.com/a#b".into())
        );
    }

    #[test]
    fn a_dash_key_is_a_key_and_not_a_list_item() {
        let meta = meta("title: A\ntags:\n  - x\n-k: v\nstatus: active\n");
        assert_eq!(meta.title.as_deref(), Some("A"));
        assert_eq!(meta.tags, ["x"]);
        assert_eq!(meta.status, Some(kanso_core::NoteStatus::Active));
        assert_eq!(meta.rest, "-k: v\n");
    }

    #[test]
    fn status_and_pinned_leave_the_body() {
        let found = meta("status: onHold\npinned: true\ncssclass: wide\n");
        assert_eq!(found.status, Some(kanso_core::NoteStatus::OnHold));
        assert!(found.pinned);
        assert_eq!(found.rest, "cssclass: wide\n");
        // Values that aren't Kanso's stay where the user wrote them.
        assert_eq!(meta("status: someday\n").status, None);
        assert_eq!(meta("status: someday\n").rest, "status: someday\n");
        assert_eq!(meta("pinned: maybe\n").rest, "pinned: maybe\n");
    }

    #[test]
    fn kanso_keys_win_over_a_users_same_key() {
        let found = meta("title: Mine\ntags: [project]\nkanso-title: Kanso\nkanso-tags: [work]\n");
        assert_eq!(found.title.as_deref(), Some("Kanso"));
        assert_eq!(found.tags, ["project", "work"]);
        assert_eq!(found.rest, "");
    }

    #[test]
    fn an_unparseable_date_stays() {
        let meta = meta("date: someday\n");
        assert_eq!(meta.created, None);
        assert_eq!(meta.rest, "date: someday\n");
    }
}
