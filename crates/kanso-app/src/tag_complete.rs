//! `#` autocomplete in the editor: typing `#` and a few letters offers the
//! library's tags, nested ones (`work/clients`) by any part of their path.
//! The text editor has one completion provider, so [`NoteCompletion`] puts
//! this beside `[[` autocomplete ([`WikiCompletion`]): a `#tag` being typed
//! goes to the tags, anything else to the wiki-links.
//!
//! What counts as a tag is `kanso_core::tags::inline_tags`: not in code, a
//! link or a heading's `#`, and only names a `#tag` can spell are offered.
//!
//! With Settings › Editor › "Tags from #words" on, saving gives the note
//! the tags its text names ([`apply_inline_tags`]).

use std::cell::Cell;

use anyhow::Result;
use gpui_kit::base::input::{CompletionProvider, Rope, RopeExt as _};
use gpui_kit::*;
use kanso_core::tags::{SEPARATOR, inline_tags};
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    TextEdit,
};

use kanso_core::Note;
use kanso_store::{Origin, Store};
use serde_json::Value;

use crate::library::Library;
use crate::wikilinks::WikiCompletion;

/// The most tags offered at once.
const LIMIT: usize = 12;
/// How far back from the caret a `#tag` is looked for.
const LOOK_BACK: usize = 200;

/// Whether `ch` can be part of a `#tag`.
fn is_tag_char(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '/')
}

/// A `#tag` being typed at the end of `line_before` (the caret's line up
/// to the caret): the byte offset of its `#` and what follows it. `None`
/// when the caret isn't in one, and for a lone `#` that starts the line,
/// which is usually a heading on its way. Pure.
pub fn tag_query(line_before: &str) -> Option<(usize, &str)> {
    let start = line_before
        .char_indices()
        .rev()
        .take_while(|&(_, ch)| is_tag_char(ch))
        .last()
        .map_or(line_before.len(), |(ix, _)| ix);
    let hash = start.checked_sub(1)?;
    if line_before.as_bytes()[hash] != b'#' {
        return None;
    }
    let before = line_before[..hash].chars().next_back();
    if !before.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{')) {
        return None;
    }
    let query = &line_before[start..];
    if query.is_empty() && line_before[..hash].trim().is_empty() {
        return None;
    }
    Some((hash, query))
}

/// Tags matching `query`, best first: by the whole name, or by the part
/// after a `/`. Names a `#tag` can't spell (with spaces, say) are left
/// out. Pure.
pub fn candidates<'a>(names: &[&'a str], query: &str, limit: usize) -> Vec<&'a str> {
    let spellable = |name: &str| {
        !name.is_empty()
            && name.chars().all(is_tag_char)
            && inline_tags(&format!("#{name}"))
                .first()
                .is_some_and(|tag| tag.name == name)
    };
    let mut scored: Vec<(i64, &str)> = names
        .iter()
        .copied()
        .filter(|name| spellable(name))
        .filter_map(|name| {
            if query.is_empty() {
                return Some((0, name));
            }
            let whole = crate::fuzzy::score(query, name);
            let leaf = name
                .rsplit(SEPARATOR)
                .next()
                .and_then(|leaf| crate::fuzzy::score(query, leaf));
            whole.max(leaf).map(|score| (score, name))
        })
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
    });
    scored
        .into_iter()
        .take(limit)
        .map(|(_, name)| name)
        .collect()
}

/// The note's field listing the tags its text gave it, so a `#tag` taken
/// out of the text takes its tag off, and a tag added by hand never is.
pub const INLINE_TAGS_FIELD: &str = "kansoInlineTags";

/// Gives `note` the tags its text names (made when missing), keeping every
/// tag added by hand; a `#tag` the caret is in (byte `caret` of the body),
/// still being typed, is left for later.
pub fn apply_inline_tags(store: &Store, note: &mut Note, caret: Option<usize>) {
    let mut seen = std::collections::HashSet::new();
    let named: Vec<String> = inline_tags(&note.body)
        .into_iter()
        .filter(|tag| caret.is_none_or(|at| !(tag.range.start < at && at <= tag.range.end)))
        .filter(|tag| seen.insert(tag.name.to_lowercase()))
        .filter_map(|tag| store.tag_named(&tag.name, Origin::Ui).ok())
        .collect();
    let before: Vec<String> = note
        .extra
        .get(INLINE_TAGS_FIELD)
        .and_then(Value::as_array)
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let (tags, given) = kanso_core::tags::merge_inline(&note.tags, &before, &named);
    note.tags = tags;
    if given.is_empty() {
        note.extra.remove(INLINE_TAGS_FIELD);
    } else {
        note.extra.insert(INLINE_TAGS_FIELD.into(), given.into());
    }
}

/// `#` autocomplete beside `[[` autocomplete.
pub struct NoteCompletion {
    wiki: WikiCompletion,
    library: Entity<Library>,
    /// Whether the caret was in a `#tag` at the last keystroke, so the next
    /// one refines the list.
    in_tag: Cell<bool>,
    /// Whether the caret was after a task's `@` at the last keystroke.
    in_date: Cell<bool>,
}

impl NoteCompletion {
    pub fn new(wiki: WikiCompletion, library: Entity<Library>) -> Self {
        NoteCompletion {
            wiki,
            library,
            in_tag: Cell::new(false),
            in_date: Cell::new(false),
        }
    }

    /// The `#tag` at `offset` of `text`: where its `#` is and the query.
    fn tag_at(&self, text: &Rope, offset: usize) -> Option<(usize, String)> {
        let offset = offset.min(text.len());
        let position = text.offset_to_position(offset);
        let line_start = text.line_start_offset(position.line as usize);
        let from = line_start.max(offset.saturating_sub(LOOK_BACK));
        let from = (from..=offset)
            .find(|&at| text.is_char_boundary(at))
            .unwrap_or(offset);
        let before = text.slice(from..offset).to_string();
        let (hash, query) = tag_query(&before)?;
        let hash = from + hash;
        // Whether a tag would start there, given everything before it
        // (code, links, headings, front matter): the store's own rule.
        let mut probe = text.slice(0..hash).to_string();
        probe.push_str("#x");
        let found = inline_tags(&probe)
            .last()
            .is_some_and(|tag| tag.range.start == hash);
        found.then(|| (hash, query.to_owned()))
    }
}

impl CompletionProvider for NoteCompletion {
    fn completions(
        &self,
        text: &Rope,
        offset: usize,
        trigger: CompletionContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<CompletionResponse>> {
        // A task's due date after `@` (see `crate::due`).
        let dates = crate::due::completions(text, offset, cx);
        self.in_date.set(dates.is_some());
        if let Some(items) = dates {
            self.in_tag.set(false);
            return Task::ready(Ok(CompletionResponse::Array(items)));
        }
        let Some((hash, query)) = self.tag_at(text, offset) else {
            self.in_tag.set(false);
            return self.wiki.completions(text, offset, trigger, window, cx);
        };
        self.in_tag.set(true);
        let offset = offset.min(text.len());
        let range = lsp_types::Range {
            start: text.offset_to_position(hash),
            end: text.offset_to_position(offset),
        };
        let library = self.library.read(cx);
        let names: Vec<&str> = library.tags.iter().map(|tag| tag.name.as_str()).collect();
        let items = candidates(&names, &query, LIMIT)
            .into_iter()
            .filter(|name| *name != query)
            .map(|name| CompletionItem {
                label: format!("#{name}"),
                kind: Some(CompletionItemKind::KEYWORD),
                filter_text: Some(String::new()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: format!("#{name}"),
                })),
                ..Default::default()
            })
            .collect();
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, offset: usize, new_text: &str, cx: &mut App) -> bool {
        if new_text.contains('\n') {
            self.in_tag.set(false);
            self.in_date.set(false);
        }
        (!new_text.contains('\n')
            && (new_text.contains(['#', '@']) || self.in_tag.get() || self.in_date.get()))
            || self.wiki.is_completion_trigger(offset, new_text, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::{INLINE_TAGS_FIELD, apply_inline_tags, candidates, tag_query};
    use kanso_core::{DocType, Note};
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use serde_json::json;

    /// Tags from the text come and go with it; a tag added by hand stays,
    /// and the `#tag` being typed waits.
    #[test]
    fn inline_tags_follow_the_text() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let hand = store.tag_named("pinned-by-hand", Origin::Ui).unwrap();
        let mut note: Note = serde_json::from_value(json!({
            "_id": "note:x", "title": "T", "bookId": book, "tags": [hand],
            "body": "Notes on #work/acme and `#code` and #rea",
        }))
        .unwrap();
        let caret = note.body.len();
        apply_inline_tags(&store, &mut note, Some(caret));
        let names = |note: &Note| {
            let mut names: Vec<String> = note
                .tags
                .iter()
                .map(|id| {
                    store.get(id).unwrap().unwrap().fields["name"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                })
                .collect();
            names.sort();
            names
        };
        assert_eq!(names(&note), ["pinned-by-hand", "work/acme"]);
        apply_inline_tags(&store, &mut note, None);
        assert_eq!(names(&note), ["pinned-by-hand", "rea", "work/acme"]);
        assert_eq!(note.extra[INLINE_TAGS_FIELD].as_array().unwrap().len(), 2);
        note.body = "Nothing now; #pinned-by-hand".into();
        apply_inline_tags(&store, &mut note, None);
        assert_eq!(names(&note), ["pinned-by-hand"]);
        note.body = String::new();
        apply_inline_tags(&store, &mut note, None);
        // Added by hand before the text named it: still there.
        assert_eq!(names(&note), ["pinned-by-hand"]);
        assert!(!note.extra.contains_key(INLINE_TAGS_FIELD));
    }

    #[test]
    fn queries() {
        assert_eq!(tag_query("see #wor"), Some((4, "wor")));
        assert_eq!(tag_query("see #"), Some((4, "")));
        assert_eq!(tag_query("(#work/cl"), Some((1, "work/cl")));
        assert_eq!(tag_query("#work"), Some((0, "work")));
        // A heading on its way.
        assert_eq!(tag_query("#"), None);
        assert_eq!(tag_query("  #"), None);
        assert_eq!(tag_query("## Head"), None);
        assert_eq!(tag_query("issue#4"), None);
        assert_eq!(tag_query("see #work done"), None);
        assert_eq!(tag_query("plain"), None);
    }

    #[test]
    fn nested_tags_match_by_any_part() {
        let names = ["work", "work/clients", "home", "read later", "Writing"];
        assert_eq!(candidates(&names, "cli", 5), vec!["work/clients"]);
        assert_eq!(candidates(&names, "wor", 5)[0], "work");
        // Not spellable as a #tag.
        assert!(!candidates(&names, "", 10).contains(&"read later"));
        assert_eq!(candidates(&names, "", 2).len(), 2);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariants: a query found ends at the caret, starts after a `#`,
        /// and is made of tag characters only.
        #[test]
        fn prop_query_ends_at_the_caret(line in "[a-z #/(\\-]{0,30}") {
            if let Some((hash, query)) = tag_query(&line) {
                prop_assert_eq!(&line[hash..hash + 1], "#");
                prop_assert_eq!(&line[hash + 1..], query);
                prop_assert!(query.chars().all(|c| c.is_alphanumeric() || c == '/' || c == '-' || c == '_'));
            }
        }

        /// Every candidate matches the query and comes from the names.
        #[test]
        fn prop_candidates_come_from_names(
            names in prop::collection::vec("[a-z]{1,6}(/[a-z]{1,6})?", 0..12),
            query in "[a-z]{0,3}",
        ) {
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let found = candidates(&names, &query, 50);
            for name in &found {
                prop_assert!(names.contains(name));
            }
            if query.is_empty() {
                let mut unique = names.clone();
                unique.sort_unstable();
                unique.dedup();
                prop_assert!(found.len() >= unique.len().min(1).min(names.len()));
            }
        }
    }
}
