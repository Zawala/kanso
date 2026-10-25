//! Wiki-links in the app: the titles they resolve against, how the preview
//! shows them, and `[[` autocomplete in the editor.
//!
//! The preview is handed the note's Markdown with each wiki-link written as
//! an ordinary link ([`rewrite_for_preview`]): to an ID link
//! ([`kanso_core::links::NOTE_LINK_PREFIX`]`<id>`) when
//! its title names a note, so it opens like any note link, and to a
//! `kanso-new:` link when it names none. [`UnresolvedLink`] draws those
//! dashed, and clicking one creates the note.
//!
//! Autocomplete ([`WikiCompletion`]) writes `[Title](<ID link>)`,
//! which names the note by ID and so survives renames. What it offers is
//! decided in `kanso_engine::links`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use gpui_kit::base::input::{CompletionProvider, Rope, RopeExt as _};
use gpui_kit::base::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    markdown_ast,
};
use gpui_kit::*;
use lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit,
    TextEdit,
};

use crate::preview::LinkSender;

pub use kanso_engine::links::{
    NEW_NOTE_SCHEME, WikiIndex, new_note_title, new_note_url, wiki_choices,
};

/// The titles shared between the editor, its autocomplete and the preview's
/// rewriting, replaced whole when they change.
pub type SharedIndex = Rc<RefCell<Arc<WikiIndex>>>;

/// The note's Markdown with each wiki-link outside code written as an
/// ordinary link, for the preview: resolved ones to their note, others to a
/// `kanso-new:` link. Text without wiki-links comes back as it is, the same
/// shared string.
// no-prop-test: delegates to kanso_engine::links::rewrite_for_preview, tested there.
pub fn rewrite_for_preview(text: &SharedString, index: &WikiIndex) -> SharedString {
    match kanso_engine::links::rewrite_for_preview(text, index) {
        std::borrow::Cow::Borrowed(_) => text.clone(),
        std::borrow::Cow::Owned(rewritten) => rewritten.into(),
    }
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

// no-prop-test: delegates to kanso_engine::links::wiki_choices, tested there.
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
        let line_end = text.line_end_offset(position.line as usize);
        let line = text.slice(line_start..line_end).to_string();
        let index = self.index.borrow().clone();
        let Some(choices) = wiki_choices(&line, offset - line_start, &index) else {
            return empty();
        };
        self.active.set(true);
        let range = lsp_types::Range {
            start: text.offset_to_position(line_start + choices.replace.start),
            end: text.offset_to_position(line_start + choices.replace.end),
        };
        let items: Vec<CompletionItem> = choices
            .items
            .into_iter()
            .map(|choice| CompletionItem {
                label: choice.label,
                kind: Some(if choice.new_note {
                    CompletionItemKind::REFERENCE
                } else {
                    CompletionItemKind::FILE
                }),
                filter_text: Some(choice.highlight),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: choice.insert,
                })),
                ..Default::default()
            })
            .collect();
        Task::ready(Ok(CompletionResponse::Array(items)))
    }

    fn is_completion_trigger(&self, _: usize, new_text: &str, _: &mut App) -> bool {
        // A bracket may open `[[`; inside one, every keystroke refines the
        // list. Ordinary typing doesn't ask at all.
        !new_text.contains('\n') && (new_text.contains('[') || self.active.get())
    }
}
