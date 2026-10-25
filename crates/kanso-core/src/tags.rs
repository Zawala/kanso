//! Nested tags and tags written in a note's text.
//!
//! A tag named `work/clients/acme` sits under `work` and `work/clients` in
//! the sidebar's tree, whether or not tags with those names exist. A
//! parent's notes are its own and its children's ([`TagNode::count`]), as
//! `tag:work` finds them in a search.
//!
//! `#tags` in a note's text ([`inline_tags`]) can become its tags on save
//! (an opt-in setting): `#idea`, `#work/clients/acme`. Code, links,
//! headings and front matter are skipped, and so are `#1` (an issue number)
//! and the `#` of a URL's fragment, which has no space before it.

use std::collections::{BTreeMap, HashSet};
use std::ops::Range;

/// The separator between a nested tag's levels.
pub const SEPARATOR: char = '/';

/// The tag whose notes never reach AI, in any case and with or without `#`.
pub const AI_EXCLUDE: &str = "ai-exclude";

/// Whether tag `name` is [`AI_EXCLUDE`], however it is written. Retrieval
/// and the agents' read-only MCP both decide `#ai-exclude` here.
pub fn is_ai_exclude(name: &str) -> bool {
    name.trim()
        .trim_start_matches('#')
        .eq_ignore_ascii_case(AI_EXCLUDE)
}

/// Whether tag `name` is `path` or inside it: `work/clients` is inside
/// `work`. Case is ignored for ASCII letters, as tag search ignores it.
pub fn is_within(name: &str, path: &str) -> bool {
    if name.len() < path.len() || !name.is_char_boundary(path.len()) {
        return false;
    }
    let (head, tail) = name.split_at(path.len());
    head.eq_ignore_ascii_case(path) && (tail.is_empty() || tail.starts_with(SEPARATOR))
}

/// The paths `name` is inside, outermost first, then `name`:
/// `a/b/c` → `a`, `a/b`, `a/b/c`. Empty levels (`a//b`, `/a`) are kept as
/// written, so every tag has a place.
pub fn ancestors(name: &str) -> impl Iterator<Item = &str> {
    name.match_indices(SEPARATOR)
        .map(move |(at, _)| &name[..at])
        .filter(|path| !path.is_empty())
        .chain(std::iter::once(name))
}

/// A tag as the tree needs it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    pub id: String,
    pub name: String,
}

/// A place in the tag tree: a tag, or a level that only other tags'
/// names make (`work` for `work/acme` when there is no `work` tag).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagNode {
    /// The full path, as the first tag that made it spelled it.
    pub path: String,
    /// The last level, shown in the tree.
    pub label: String,
    /// The tag with exactly this name (ignoring ASCII case), if any.
    pub tag_id: Option<String>,
    /// Notes with this tag or any tag inside it, each counted once.
    pub count: u32,
    pub children: Vec<TagNode>,
}

impl TagNode {
    /// Every node below this one, depth first.
    pub fn descendants(&self) -> impl Iterator<Item = &TagNode> {
        let mut stack: Vec<&TagNode> = self.children.iter().rev().collect();
        std::iter::from_fn(move || {
            let node = stack.pop()?;
            stack.extend(node.children.iter().rev());
            Some(node)
        })
    }
}

#[derive(Default)]
struct Building {
    path: String,
    tag_id: Option<String>,
    notes: HashSet<String>,
    children: BTreeMap<String, Building>,
}

impl Building {
    /// The finished node; its label is its path after `label_from` bytes
    /// (the parent's path and the separator). Each node's `notes` already
    /// holds its whole subtree's notes.
    fn finish(self, label_from: usize) -> TagNode {
        let child_from = self.path.len() + 1;
        let mut children: Vec<TagNode> = self
            .children
            .into_values()
            .map(|child| child.finish(child_from))
            .collect();
        children.sort_by_key(|c| c.label.to_lowercase());
        TagNode {
            label: self.path.get(label_from..).unwrap_or_default().to_owned(),
            path: self.path,
            tag_id: self.tag_id,
            count: u32::try_from(self.notes.len()).unwrap_or(u32::MAX),
            children,
        }
    }
}

/// The tag tree: `tags` sorted into levels by their names, each node with
/// how many notes have it or a tag inside it. `links` are `(note ID, tag
/// ID)` pairs; tags without notes still show, with 0. Levels match without
/// regard to ASCII case and are sorted by name.
pub fn tag_tree(tags: &[TagInfo], links: &[(String, String)]) -> Vec<TagNode> {
    let mut root = Building::default();
    let key = |s: &str| s.to_ascii_lowercase();
    let mut path_of_tag: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for tag in tags {
        let mut node = &mut root;
        let mut keys = Vec::new();
        for path in ancestors(&tag.name) {
            let level_key = key(&path[path.rfind(SEPARATOR).map_or(0, |at| at + 1)..]);
            let level_key = if level_key.is_empty() {
                // An empty level keeps a place of its own.
                format!("\u{0}{}", path.len())
            } else {
                level_key
            };
            keys.push(level_key.clone());
            node = node.children.entry(level_key).or_insert_with(|| Building {
                path: path.to_owned(),
                ..Building::default()
            });
        }
        if node.tag_id.is_none() {
            node.tag_id = Some(tag.id.clone());
        }
        path_of_tag.entry(tag.id.as_str()).or_insert(keys);
    }
    for (note, tag) in links {
        let Some(keys) = path_of_tag.get(tag.as_str()) else {
            continue;
        };
        let mut node = &mut root;
        for k in keys {
            let Some(next) = node.children.get_mut(k) else {
                break;
            };
            next.notes.insert(note.clone());
            node = next;
        }
    }
    let mut nodes: Vec<TagNode> = root
        .children
        .into_values()
        .map(|child| child.finish(0))
        .collect();
    nodes.sort_by_key(|n| n.label.to_lowercase());
    nodes
}

/// Whether any tag name makes a tree deeper than one level.
pub fn has_nesting<'a>(names: impl IntoIterator<Item = &'a str>) -> bool {
    names.into_iter().any(|name| name.contains(SEPARATOR))
}

// MARK: Tags in the text

/// A `#tag` in a note's text: where it is (the `#` included) and its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineTag {
    pub range: Range<usize>,
    pub name: String,
}

/// Longest tag name taken from the text, the store's limit for names.
const MAX_INLINE_LEN: usize = crate::model::MAX_NAME_LEN;

fn is_tag_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | SEPARATOR)
}

/// The `#tags` in `body`, in order. A tag starts at a `#` at the start of a
/// line or after a space or an opening bracket, and runs over letters,
/// digits, `_`, `-` and `/`; it needs a letter, and a trailing `/` isn't
/// part of it. Fenced and inline code, links (`[text](url)`,
/// `<https://…>` and wiki-links), ATX headings and YAML front matter are
/// skipped.
pub fn inline_tags(body: &str) -> Vec<InlineTag> {
    let code = crate::links::code_ranges(body);
    let in_code = |at: usize| {
        let ix = code.partition_point(|r| r.end <= at);
        code.get(ix).is_some_and(|r| r.start <= at)
    };
    let mut tags = Vec::new();
    let mut offset = 0;
    let mut front_matter = body.starts_with("---\n") || body.starts_with("---\r\n");
    for (ix, line) in body.split_inclusive('\n').enumerate() {
        let start = offset;
        offset += line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        if front_matter {
            if ix > 0 && (content == "---" || content == "...") {
                front_matter = false;
            }
            continue;
        }
        if is_heading(content) || !content.contains('#') {
            continue;
        }
        let covered = crate::links::covered_ranges(content);
        let autolinks = autolink_ranges(content);
        let mut chars = content.char_indices().peekable();
        let mut prev: Option<char> = None;
        while let Some((at, c)) = chars.next() {
            if c != '#' || !prev.is_none_or(|p| p.is_whitespace() || matches!(p, '(' | '[' | '{')) {
                prev = Some(c);
                continue;
            }
            let rest = &content[at + 1..];
            let len: usize = rest
                .chars()
                .take_while(|&c| is_tag_char(c))
                .map(char::len_utf8)
                .sum();
            let name = rest[..len].trim_end_matches(SEPARATOR);
            let inside = |ranges: &[Range<usize>]| ranges.iter().any(|r| r.contains(&at));
            let valid = !name.is_empty()
                && name.chars().any(char::is_alphabetic)
                && !name.starts_with(SEPARATOR)
                && !name.contains("//")
                && name.chars().count() <= MAX_INLINE_LEN
                && !in_code(start + at)
                && !inside(&covered)
                && !inside(&autolinks);
            if valid {
                tags.push(InlineTag {
                    range: start + at..start + at + 1 + name.len(),
                    name: name.to_owned(),
                });
            }
            // Skip what was read; a `#` inside a run of tag characters can't
            // start another tag.
            while chars.peek().is_some_and(|&(i, _)| i <= at + len) {
                chars.next();
            }
            prev = content[..at + 1 + len].chars().next_back();
        }
    }
    tags
}

/// The distinct tag names in `body`, in the order they first appear,
/// ignoring ASCII case.
pub fn inline_tag_names(body: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    inline_tags(body)
        .into_iter()
        .filter(|t| seen.insert(t.name.to_ascii_lowercase()))
        .map(|t| t.name)
        .collect()
}

/// `# Heading` to `###### Heading`, up to three spaces in.
fn is_heading(line: &str) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let hashes = rest.bytes().take_while(|&b| b == b'#').count();
    (1..=6).contains(&hashes)
        && rest[hashes..]
            .chars()
            .next()
            .is_none_or(|c| c == ' ' || c == '\t')
}

/// `<scheme:…>` autolinks in a line.
fn autolink_ranges(line: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(open) = line[from..].find('<') {
        let open = from + open;
        let Some(close) = line[open..].find('>') else {
            break;
        };
        let close = open + close;
        if line[open..close].contains(':') && !line[open..close].contains(' ') {
            ranges.push(open..close + 1);
        }
        from = close + 1;
    }
    ranges
}

/// What the note's tags become when its text's `#tags` are added on save:
/// `current` (tag IDs) keeps every tag added by hand, gains the tags named
/// in the text, and loses those that only the text gave it before
/// (`from_text`, IDs) and that it no longer names. Returns the new tags and
/// the IDs the text gives now, to remember for next time.
pub fn merge_inline(
    current: &[String],
    from_text_before: &[String],
    named_now: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut tags: Vec<String> = current
        .iter()
        .filter(|id| !from_text_before.contains(id) || named_now.contains(id))
        .cloned()
        .collect();
    let mut given = Vec::new();
    for id in named_now {
        if !tags.contains(id) {
            tags.push(id.clone());
            given.push(id.clone());
        } else if from_text_before.contains(id) {
            given.push(id.clone());
        }
    }
    (tags, given)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn info(id: &str, name: &str) -> TagInfo {
        TagInfo {
            id: id.into(),
            name: name.into(),
        }
    }

    fn link(note: &str, tag: &str) -> (String, String) {
        (note.into(), tag.into())
    }

    #[test]
    fn within_and_ancestors() {
        assert!(is_within("work/acme", "work"));
        assert!(is_within("Work/acme", "work"));
        assert!(is_within("work", "WORK"));
        assert!(!is_within("workshop", "work"));
        assert!(!is_within("wor", "work"));
        assert_eq!(
            ancestors("a/b/c").collect::<Vec<_>>(),
            ["a", "a/b", "a/b/c"]
        );
        assert_eq!(ancestors("plain").collect::<Vec<_>>(), ["plain"]);
    }

    #[test]
    fn tree_example() {
        let tags = [
            info("tag:w", "work"),
            info("tag:a", "work/clients/acme"),
            info("tag:b", "work/clients/beta"),
            info("tag:r", "rust"),
        ];
        let links = [
            link("n1", "tag:a"),
            link("n1", "tag:b"),
            link("n2", "tag:b"),
            link("n3", "tag:w"),
            link("n4", "tag:r"),
        ];
        let tree = tag_tree(&tags, &links);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].label, "rust");
        let work = &tree[1];
        assert_eq!((work.label.as_str(), work.count), ("work", 3));
        assert_eq!(work.tag_id.as_deref(), Some("tag:w"));
        let clients = &work.children[0];
        assert_eq!(
            (
                clients.path.as_str(),
                clients.tag_id.as_deref(),
                clients.count
            ),
            ("work/clients", None, 2)
        );
        let names: Vec<_> = clients
            .children
            .iter()
            .map(|c| (c.label.as_str(), c.count))
            .collect();
        assert_eq!(names, [("acme", 1), ("beta", 2)]);
    }

    #[test]
    fn inline_examples() {
        let body = "---\ntags: #nope\n---\n# Heading #nope\n#idea and #work/acme, not#this or #1 or #/x.\n`#code` [#link](https://x.y/#frag) <https://a.b/#c> [[Note#Head]]\n```\n#fenced\n```\n(#paren) #trailing/ #über #a//b\n##not-heading-hash";
        let names: Vec<String> = inline_tags(body).into_iter().map(|t| t.name).collect();
        assert_eq!(names, ["idea", "work/acme", "paren", "trailing", "über"]);
        for tag in inline_tags(body) {
            assert_eq!(&body[tag.range.clone()], format!("#{}", tag.name));
        }
        assert_eq!(inline_tag_names("#a #A #b"), ["a", "b"]);
    }

    #[test]
    fn merge_examples() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        // A hand tag stays even when the text drops it; a text tag goes.
        let (tags, given) = merge_inline(&s(&["hand", "old"]), &s(&["old"]), &s(&["new"]));
        assert_eq!(tags, s(&["hand", "new"]));
        assert_eq!(given, s(&["new"]));
        // A hand tag the text also names stays a hand tag.
        let (tags, given) = merge_inline(&s(&["hand"]), &s(&[]), &s(&["hand"]));
        assert_eq!(tags, s(&["hand"]));
        assert!(given.is_empty());
    }

    fn tag_name() -> impl Strategy<Value = String> {
        prop::collection::vec(prop_oneof!["[a-c]{1,2}", "[A-C]{1}"], 1..4)
            .prop_map(|levels| levels.join("/"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariants: every tag appears in the tree exactly once, and a
        /// parent counts at least as many notes as each of its children.
        #[test]
        fn prop_tag_tree_places_every_tag_once(
            names in prop::collection::vec(tag_name(), 0..12),
            links in prop::collection::vec((0usize..6, 0usize..12), 0..30),
        ) {
            let tags: Vec<TagInfo> = names.iter().enumerate()
                .map(|(i, n)| info(&format!("tag:{i}"), n))
                .collect();
            let links: Vec<(String, String)> = links.iter()
                .filter(|(_, t)| *t < tags.len())
                .map(|(n, t)| (format!("n{n}"), format!("tag:{t}")))
                .collect();
            let tree = tag_tree(&tags, &links);
            let mut ids: Vec<&str> = Vec::new();
            fn walk<'a>(nodes: &'a [TagNode], ids: &mut Vec<&'a str>, parent: Option<&TagNode>) -> Result<(), TestCaseError> {
                for node in nodes {
                    if let Some(id) = &node.tag_id { ids.push(id); }
                    if let Some(parent) = parent {
                        prop_assert!(parent.count >= node.count);
                        prop_assert!(is_within(&node.path, &parent.path));
                    }
                    walk(&node.children, ids, Some(node))?;
                }
                Ok(())
            }
            walk(&tree, &mut ids, None)?;
            // A tag whose name another tag has (ignoring case) shares its
            // node: each name has one place.
            let mut distinct: Vec<String> = names.iter().map(|n| n.to_ascii_lowercase()).collect();
            distinct.sort();
            distinct.dedup();
            prop_assert_eq!(ids.len(), distinct.len());
            let mut unique = ids.clone();
            unique.sort();
            unique.dedup();
            prop_assert_eq!(unique.len(), ids.len());
            // A node's count is the notes with its tag or one inside it.
            for node in tree.iter().flat_map(|n| std::iter::once(n).chain(n.descendants())) {
                let expected: HashSet<&str> = links.iter()
                    .filter(|(_, t)| tags.iter().any(|tag| &tag.id == t && is_within(&tag.name, &node.path)))
                    .map(|(n, _)| n.as_str())
                    .collect();
                prop_assert_eq!(node.count as usize, expected.len(), "{}", node.path);
            }
        }

        /// Invariant: a tag is never taken from code, a link or a heading,
        /// and each one found is `#` and its name in the text.
        #[test]
        fn prop_inline_tags_skip_code_links_and_headings(
            parts in prop::collection::vec(prop_oneof![
                "[a-z ]{0,6}",
                Just(" #ok".to_owned()),
                Just("\n# #head".to_owned()),
                Just("`#code`".to_owned()),
                Just("[#link](u)".to_owned()),
                Just("[t](u#frag)".to_owned()),
                Just("\n```\n#fence\n```\n".to_owned()),
                Just("<https://x/#auto>".to_owned()),
                Just("\n".to_owned()),
            ], 0..12),
        ) {
            let body = parts.concat();
            for tag in inline_tags(&body) {
                prop_assert_eq!(&body[tag.range.clone()], format!("#{}", tag.name));
                prop_assert!(!["head", "code", "link", "frag", "fence", "auto"].contains(&tag.name.as_str()), "{} in {:?}", tag.name, body);
            }
        }

        /// Invariant: merging never drops a tag added by hand, always
        /// includes every tag the text names, and is stable when nothing
        /// changes.
        #[test]
        fn prop_merge_keeps_hand_tags(
            current in prop::collection::btree_set(0u8..8, 0..6),
            before in prop::collection::btree_set(0u8..8, 0..6),
            now in prop::collection::btree_set(0u8..8, 0..6),
        ) {
            let s = |set: &std::collections::BTreeSet<u8>| set.iter().map(|x| x.to_string()).collect::<Vec<_>>();
            let (current, before, now) = (s(&current), s(&before), s(&now));
            let (tags, given) = merge_inline(&current, &before, &now);
            for id in &current {
                if !before.contains(id) {
                    prop_assert!(tags.contains(id), "hand tag {} dropped", id);
                }
            }
            for id in &now {
                prop_assert!(tags.contains(id));
            }
            for id in &given {
                prop_assert!(now.contains(id));
            }
            let (again, given_again) = merge_inline(&tags, &given, &now);
            prop_assert_eq!(again, tags);
            prop_assert_eq!(given_again, given);
        }
    }
}
