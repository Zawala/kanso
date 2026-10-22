//! Wiki-links in the app: the titles they resolve against, how the preview
//! shows them, and `[[` autocomplete in the editor.
//!
//! The preview is handed the note's Markdown with each wiki-link written as
//! an ordinary link ([`rewrite_for_preview`]): to an ID link
//! ([`NOTE_LINK_PREFIX`]`<id>`) when
//! its title names a note, so it opens like any note link, and to a
//! `kanso-new:` link when it names none. [`UnresolvedLink`] draws those
//! dashed, and clicking one creates the note.
//!
//! Autocomplete ([`WikiCompletion`]) writes `[Title](<ID link>)`,
//! which names the note by ID and so survives renames.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::base::input::{CompletionProvider, Rope, RopeExt as _};
use gpui_kit::base::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    markdown_ast,
};
use gpui_kit::*;
use kanso_core::links::{NOTE_LINK_PREFIX, code_ranges, normalize_title, wiki_links};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    TextEdit,
};

use crate::preview::LinkSender;

/// The scheme of the links unresolved wiki-links become in the preview.
pub const NEW_NOTE_SCHEME: &str = "kanso-new:";
/// The most titles autocomplete offers at once.
const COMPLETION_LIMIT: usize = 12;

/// The titles wiki-links resolve against: every live note outside the
/// trash, the first created first.
#[derive(Clone, Debug, Default)]
pub struct WikiIndex {
    /// `(id, title)`, the first created first.
    pub titles: Vec<(String, String)>,
    /// Normalized title to the first note with it, as an index into
    /// `titles`.
    by_title: HashMap<String, usize>,
}

impl WikiIndex {
    /// The index of `titles`, given first created first: a title shared by
    /// several notes names the first.
    pub fn new(titles: Vec<(String, String)>) -> Self {
        let mut by_title = HashMap::with_capacity(titles.len());
        for (ix, (_, title)) in titles.iter().enumerate() {
            let key = normalize_title(title);
            if !key.is_empty() {
                by_title.entry(key).or_insert(ix);
            }
        }
        WikiIndex { titles, by_title }
    }

    /// The `(id, title)` of the note `title` names.
    pub fn resolve(&self, title: &str) -> Option<&(String, String)> {
        self.by_title
            .get(&normalize_title(title))
            .map(|&ix| &self.titles[ix])
    }
}

/// The titles shared between the editor, its autocomplete and the preview's
/// rewriting, replaced whole when they change.
pub type SharedIndex = Rc<RefCell<Arc<WikiIndex>>>;

/// `kanso-new:<title>`, with everything but letters, digits and a few safe
/// marks percent-encoded, so the title survives as a link target.
pub fn new_note_url(title: &str) -> String {
    let mut url = String::from(NEW_NOTE_SCHEME);
    for byte in title.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

/// The title a `kanso-new:` link asks to create.
pub fn new_note_title(url: &str) -> Option<String> {
    let encoded = url.strip_prefix(NEW_NOTE_SCHEME)?;
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = encoded.get(i + 1..i + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let title = String::from_utf8(out).ok()?;
    let title = title.trim();
    (!title.is_empty()).then(|| title.to_owned())
}

/// The note's Markdown with each wiki-link outside code written as an
/// ordinary link, for the preview: resolved ones to their note, others to a
/// `kanso-new:` link. Text without wiki-links comes back as it is. Pure.
pub fn rewrite_for_preview(text: &SharedString, index: &WikiIndex) -> SharedString {
    if !text.contains("[[") {
        return text.clone();
    }
    let links = wiki_links(text);
    if links.is_empty() {
        return text.clone();
    }
    let mut out = String::with_capacity(text.len() + links.len() * 32);
    let mut last = 0;
    for link in links {
        out.push_str(&text[last..link.range.start]);
        let label = escape_label(&link.label());
        let target = match index.resolve(&link.title) {
            Some((id, _)) => format!(
                "{NOTE_LINK_PREFIX}{}",
                id.strip_prefix("note:").unwrap_or(id)
            ),
            None => new_note_url(&link.title),
        };
        out.push('[');
        out.push_str(&label);
        out.push_str("](");
        out.push_str(&target);
        out.push(')');
        last = link.range.end;
    }
    out.push_str(&text[last..]);
    out.into()
}

/// Link text that stays one link: brackets and backslashes escaped.
fn escape_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if matches!(c, '[' | ']' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

// MARK: The preview

/// What an unresolved wiki-link shows and creates.
struct Unresolved {
    title: String,
    label: String,
}

/// Draws the links unresolved wiki-links become: the link's words with a
/// dashed underline, quieter than a link, and "Create this note" on hover.
/// Clicking one asks for the note to be created.
pub struct UnresolvedLink {
    links: LinkSender,
}

impl UnresolvedLink {
    pub const NAME: &'static str = "kanso-unresolved-link";

    pub fn new(links: LinkSender) -> Self {
        UnresolvedLink { links }
    }
}

impl MarkdownPlugin for UnresolvedLink {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else {
            return None;
        };
        let title = new_note_title(&link.url)?;
        let label = link
            .children
            .iter()
            .map(|child| child.to_string())
            .collect::<String>();
        let label = if label.trim().is_empty() {
            title.clone()
        } else {
            label
        };
        let source = cx.node_source(node).unwrap_or_default().to_owned();
        Some(
            MarkdownNode::new(
                Self::NAME,
                Unresolved {
                    title,
                    label: label.clone(),
                },
            )
            .text(label)
            .markdown(source),
        )
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        context: &InlineRenderContext,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        let unresolved = node.data::<Unresolved>()?;
        let link = crate::themes::preview_colors(cx).link;
        let style = context.text_style();
        let title = unresolved.title.clone();
        let links = self.links.clone();
        let at = node.source_range().map_or(0, |r| r.start);
        let element = div()
            .id(ElementId::NamedInteger(Self::NAME.into(), at as u64))
            .font_family(style.font_family.clone())
            .font_weight(style.font_weight)
            .text_size(context.font_size())
            .line_height(context.line_height())
            .text_color(Hsla::from(link).opacity(0.78))
            .border_b_1()
            .border_dashed()
            .border_color(Hsla::from(link).opacity(0.55))
            .cursor_pointer()
            .hover(|style| style.text_color(link))
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new("Create this note").build(window, cx)
            })
            .child(unresolved.label.clone())
            .on_click(move |_, _, cx| {
                cx.stop_propagation();
                links.try_send(new_note_url(&title)).ok();
            });
        Some(InlineElement::new(element))
    }
}

// MARK: Autocomplete

/// What `[[` autocomplete is completing, from the text of the caret's line
/// up to the caret: where the `[[` starts in it and what was typed after.
/// `None` once the brackets are closed, the line has none, or the typing
/// can't be a title (a bracket inside). Pure.
pub fn completion_query(before_caret: &str) -> Option<(usize, &str)> {
    let start = before_caret.rfind("[[")?;
    // `[[[`: the last two brackets open it.
    let query = &before_caret[start + 2..];
    if query.contains(['[', ']']) || query.contains('|') {
        return None;
    }
    if start > 0 && before_caret.as_bytes()[start - 1] == b'!' {
        return None;
    }
    if query.chars().count() > 80 {
        return None;
    }
    Some((start, query))
}

/// Kanso's own link to note `id` titled `title`, `[Title](<ID link>)`:
/// what autocomplete writes, which names the note by ID. Pure.
pub fn id_link(id: &str, title: &str) -> String {
    let suffix = id.strip_prefix("note:").unwrap_or(id);
    format!("[{}]({NOTE_LINK_PREFIX}{suffix})", link_text(title))
}

/// Link text for `title` in `[text](…)`: brackets escaped, and "Untitled"
/// for a note without one.
fn link_text(title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        "Untitled".to_owned()
    } else {
        escape_label(title)
    }
}

/// The titles autocomplete offers for `query`, best first, at most
/// `limit`: every title the query fits (fuzzily), or with nothing typed the
/// most recently created. Pure.
pub fn completion_candidates<'a>(
    index: &'a WikiIndex,
    query: &str,
    limit: usize,
) -> Vec<&'a (String, String)> {
    let query = query.trim();
    if query.is_empty() {
        return index.titles.iter().rev().take(limit).collect();
    }
    let mut scored: Vec<(i64, usize)> = index
        .titles
        .iter()
        .enumerate()
        .filter_map(|(ix, (_, title))| crate::fuzzy::score(query, title).map(|s| (s, ix)))
        .collect();
    // Best score first; among equals, the newer note.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, ix)| &index.titles[ix])
        .collect()
}

/// The start of `title` that `query` spells, ignoring case, for the list
/// to highlight; empty when the title doesn't start with it (a fuzzy match
/// further in). The list highlights only a prefix. Pure.
pub fn matched_prefix<'a>(title: &'a str, query: &str) -> &'a str {
    let query = query.trim();
    let mut end = 0;
    let mut wanted = query.chars().flat_map(char::to_lowercase);
    let mut pending = wanted.next();
    for (ix, ch) in title.char_indices() {
        let Some(want) = pending else { break };
        let mut lower = ch.to_lowercase();
        if lower.next() != Some(want) || lower.next().is_some() {
            return "";
        }
        end = ix + ch.len_utf8();
        pending = wanted.next();
    }
    if pending.is_some() { "" } else { &title[..end] }
}

/// `[[` autocomplete: notes by title, inserted as `[Title](<ID link>)`.
/// When no title matches what was typed exactly, the last entry writes
/// `[[typed]]`, which the preview offers to create.
pub struct WikiCompletion {
    index: SharedIndex,
    /// Whether the caret was inside `[[…` at the last keystroke, so the
    /// next one refines the list. Other typing never asks.
    active: Cell<bool>,
}

impl WikiCompletion {
    pub fn new(index: SharedIndex) -> Self {
        WikiCompletion {
            index,
            active: Cell::new(false),
        }
    }
}

impl CompletionProvider for WikiCompletion {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        _: CompletionContext,
        _: &mut Window,
        _: &mut App,
    ) -> Task<Result<CompletionResponse>> {
        let empty = || {
            self.active.set(false);
            Task::ready(Ok(CompletionResponse::Array(Vec::new())))
        };
        let offset = offset.min(text.len());
        let position = text.offset_to_position(offset);
        let line_start = text.line_start_offset(position.line as usize);
        // Only the end of a long line can hold the query.
        let from = line_start.max(offset.saturating_sub(400));
        let from = (from..=offset)
            .find(|&at| text.is_char_boundary(at))
            .unwrap_or(offset);
        let before = text.slice(from..offset).to_string();
        let Some((start, query)) = completion_query(&before) else {
            return empty();
        };
        // Inside a code span or block, `[[` is text.
        let line_end = text.line_end_offset(position.line as usize);
        let line = text.slice(line_start..line_end).to_string();
        let in_line = from - line_start + start;
        if code_ranges(&line)
            .iter()
            .any(|r| r.start <= in_line && in_line < r.end)
        {
            return empty();
        }
        self.active.set(true);
        let start = from + start;
        // Brackets the pairing closed after the caret go too.
        let after = text.slice(offset..line_end.min(offset + 2)).to_string();
        let end = if after == "]]" { offset + 2 } else { offset };
        let range = lsp_types::Range {
            start: text.offset_to_position(start),
            end: text.offset_to_position(end),
        };
        let index = self.index.borrow().clone();
        let query = query.to_owned();
        let edit = |new_text: String| Some(CompletionTextEdit::Edit(TextEdit { range, new_text }));
        let mut items: Vec<CompletionItem> =
            completion_candidates(&index, &query, COMPLETION_LIMIT)
                .into_iter()
                .map(|(id, title)| {
                    let shown = if title.trim().is_empty() {
                        "Untitled".to_owned()
                    } else {
                        title.clone()
                    };
                    CompletionItem {
                        filter_text: Some(matched_prefix(&shown, &query).to_owned()),
                        label: shown,
                        kind: Some(CompletionItemKind::FILE),
                        text_edit: edit(id_link(id, title)),
                        ..Default::default()
                    }
                })
                .collect();
        let typed = query.trim();
        if !typed.is_empty() && index.resolve(typed).is_none() {
            items.push(CompletionItem {
                label: format!("New note “{typed}”"),
                kind: Some(CompletionItemKind::REFERENCE),
                filter_text: Some(String::new()),
                text_edit: edit(format!("[[{typed}]]")),
                ..Default::default()
            });
        }
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, _: usize, new_text: &str, _: &mut App) -> bool {
        // A bracket may open `[[`; inside one, every keystroke refines the
        // list. Ordinary typing doesn't ask at all.
        !new_text.contains('\n') && (new_text.contains('[') || self.active.get())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NEW_NOTE_SCHEME, WikiIndex, completion_candidates, completion_query, matched_prefix,
        new_note_title, new_note_url, rewrite_for_preview,
    };
    use gpui_kit::SharedString;
    use kanso_core::links::wiki_links;
    use proptest::prelude::*;

    fn index() -> WikiIndex {
        WikiIndex::new(vec![
            ("note:a".into(), "Project Plan".into()),
            ("note:b".into(), "Budget".into()),
            ("note:c".into(), "project plan".into()),
        ])
    }

    #[test]
    fn only_a_spelled_prefix_is_highlighted() {
        assert_eq!(matched_prefix("Sync design", "sy"), "Sy");
        assert_eq!(matched_prefix("MCP support model", "s"), "");
        assert_eq!(matched_prefix("Café crème", "CAFÉ"), "Café");
        assert_eq!(matched_prefix("Ab", "abc"), "");
        assert_eq!(matched_prefix("Ab", ""), "");
    }

    #[test]
    fn the_first_created_note_wins_a_shared_title() {
        assert_eq!(index().resolve("PROJECT  plan").unwrap().0, "note:a");
        assert!(index().resolve("Nope").is_none());
    }

    #[test]
    fn the_preview_gets_ordinary_links() {
        let text: SharedString =
            "See [[project plan#Goals|the plan]], [[Missing Note]] and `[[Budget]]`.".into();
        assert_eq!(
            rewrite_for_preview(&text, &index()).as_ref(),
            "See [the plan](inkdrop://note/a), [Missing Note](kanso-new:Missing%20Note) and `[[Budget]]`."
        );
        let plain: SharedString = "No links here".into();
        assert_eq!(rewrite_for_preview(&plain, &index()), plain);
    }

    #[test]
    fn completion_queries() {
        assert_eq!(completion_query("see [[Pro"), Some((4, "Pro")));
        assert_eq!(completion_query("[["), Some((0, "")));
        assert_eq!(completion_query("[[done]] x"), None);
        assert_eq!(completion_query("![[img"), None);
        assert_eq!(completion_query("plain"), None);
        let index = index();
        let found: Vec<&str> = completion_candidates(&index, "bud", 5)
            .into_iter()
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(found, ["note:b"]);
        assert_eq!(completion_candidates(&index, "", 2).len(), 2);
    }

    proptest! {
        /// The highlight is always the title's own start, as long as the
        /// query or nothing.
        #[test]
        fn the_highlight_is_a_prefix(title in "\\PC{0,12}", query in "\\PC{0,4}") {
            let got = matched_prefix(&title, &query);
            prop_assert!(title.starts_with(got));
            prop_assert!(got.is_empty() || got.to_lowercase() == query.trim().to_lowercase());
        }

        /// Round trip: any title survives the `kanso-new:` link.
        #[test]
        fn prop_new_note_urls_round_trip(title in "[^\\s]([^\\n]{0,30}[^\\s])?") {
            let url = new_note_url(&title);
            prop_assert!(!url[NEW_NOTE_SCHEME.len()..].contains([' ', '(', ')', '[', ']', '<', '>']));
            prop_assert_eq!(new_note_title(&url), Some(title));
        }

        /// Invariant: rewriting leaves no wiki-link outside code, keeps the
        /// text around them, and every link it writes points at a note the
        /// index resolves or at `kanso-new:`.
        #[test]
        fn prop_rewritten_text_has_no_wiki_links(parts in prop::collection::vec(prop_oneof!["[a-z ]{0,6}", Just("[[Budget]]".to_owned()), Just("[[Nope|x]]".to_owned()), Just("`[[Budget]]`".to_owned())], 0..8)) {
            let text: SharedString = parts.concat().into();
            let out = rewrite_for_preview(&text, &index());
            prop_assert!(wiki_links(&out).is_empty());
            let code_kept = parts.iter().filter(|p| p.starts_with('`')).count();
            prop_assert_eq!(out.matches("`[[Budget]]`").count(), code_kept);
            let resolved = parts.iter().filter(|p| p.as_str() == "[[Budget]]").count();
            prop_assert_eq!(out.matches("](inkdrop://note/b)").count(), resolved);
        }

        /// The query is what follows the last `[[` on the line, when it can
        /// still be a title.
        #[test]
        fn prop_completion_query_is_the_text_after_the_brackets(pre in "[a-z ]{0,8}", typed in "[A-Za-z0-9 ]{0,12}") {
            let line = format!("{pre}[[{typed}");
            prop_assert_eq!(completion_query(&line), Some((pre.len(), typed.as_str())));
            let closed = format!("{line}]]");
            prop_assert_eq!(completion_query(&closed), None);
        }
    }
}
