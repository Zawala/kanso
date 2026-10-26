//! Drawing: a note's preview, source highlighting, themes, math and
//! diagrams.
//!
//! The preview, math and diagrams are drawn by `kanso-render`; the
//! highlighter is the engine's tree-sitter one, the same that colors the
//! desktop editor. Text offsets cross this edge in UTF-16 code units, the
//! unit Flutter's text fields count in; the engine's are UTF-8 bytes.

use std::sync::Arc;

use kanso_engine::highlight;
use kanso_engine::text::{byte_to_utf16, utf16_to_byte};
use kanso_render::{Failure, Job, Kind, Outcome, Palette, Rgb, html, render_blocking};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{DiagramKind, PaletteDto, Range16, SpanRow, SvgRow, ThemeTokens};
use crate::theme_tokens;

/// Drawing for one library; the associated functions need no library.
#[derive(Clone)]
pub struct Renderer {
    pub(crate) inner: Arc<Inner>,
}

impl Renderer {
    /// A note as a standalone HTML page in `palette`'s colors, its math and
    /// diagrams drawn and its images inlined. [`KansoError::Locked`] while
    /// the note is locked and closed.
    pub fn preview_html(&self, id: &str, palette: PaletteDto) -> Result<String, KansoError> {
        guard(|| {
            let library = Library {
                inner: self.inner.clone(),
            };
            let (note, text) = library.readable_text(id)?.ok_or(KansoError::Locked)?;
            let store = &self.inner.store;
            let attachments = |file: &str| {
                store
                    .attachment(file)
                    .ok()
                    .flatten()
                    .map(|(content_type, blob)| (content_type, blob.data))
            };
            Ok(html::page_with(
                &note.title,
                &text,
                &palette_of(palette),
                &attachments,
                &html::PageOptions::default(),
            ))
        })
    }

    /// The runs of `text` between `start` and `end` (UTF-16 units) that the
    /// Markdown highlighter styles, with the theme keys that style them.
    /// Text between runs is plain.
    pub fn highlight(text: String, start: u32, end: u32) -> Result<Vec<SpanRow>, KansoError> {
        guard(|| {
            let from = utf16_to_byte(&text, start as usize);
            let to = utf16_to_byte(&text, end as usize).max(from);
            Ok(highlight::spans(&text, from..to)
                .into_iter()
                .map(|span| SpanRow {
                    range: Range16 {
                        start: byte_to_utf16(&text, span.range.start) as u32,
                        end: byte_to_utf16(&text, span.range.end) as u32,
                    },
                    tokens: span
                        .tokens
                        .iter()
                        .map(|token| token.key().to_owned())
                        .collect(),
                })
                .collect())
        })
    }

    /// A theme file (JSON) fitted for drawing: the interface colors, the
    /// syntax styles and the preview palette.
    pub fn theme_tokens(theme_json: String) -> Result<ThemeTokens, KansoError> {
        guard(|| theme_tokens::fit(&theme_json))
    }

    /// TeX math as an SVG in `palette`'s colors: `display` sets it on its own
    /// line, centred; otherwise in a line of text.
    pub fn math_svg(
        source: String,
        display: bool,
        palette: PaletteDto,
    ) -> Result<SvgRow, KansoError> {
        guard(|| {
            let kind = if display {
                Kind::DisplayMath
            } else {
                Kind::InlineMath
            };
            drawn(Job::new(kind, source, palette_of(palette)))
        })
    }

    /// A Mermaid diagram or an Excalidraw scene as an SVG in `palette`'s
    /// colors.
    pub fn diagram_svg(
        kind: DiagramKind,
        source: String,
        palette: PaletteDto,
    ) -> Result<SvgRow, KansoError> {
        guard(|| {
            let kind = match kind {
                DiagramKind::Mermaid => Kind::Mermaid,
                DiagramKind::Excalidraw => Kind::Excalidraw,
            };
            drawn(Job::new(kind, source, palette_of(palette)))
        })
    }

    /// Hands over the app's fonts (the bytes of font files) so diagrams are
    /// set in them. Only the first call before the first drawing takes
    /// effect; it says whether this one did.
    pub fn set_fonts(fonts: Vec<Vec<u8>>) -> bool {
        kanso_render::set_fonts(fonts)
    }
}

/// Draws `job` and turns a failure into the error the shell shows.
fn drawn(job: Job) -> Result<SvgRow, KansoError> {
    let outcome: Arc<Outcome> = render_blocking(&job);
    match outcome.as_ref() {
        Ok(svg) => Ok(SvgRow {
            markup: svg.markup.clone(),
            width: svg.width,
            height: svg.height,
            baseline: svg.baseline,
        }),
        Err(failure) => Err(KansoError::invalid(match failure {
            Failure::Invalid(message) if !message.is_empty() => {
                format!("{}: {message}", failure.badge())
            }
            _ => failure.badge(),
        })),
    }
}

pub(crate) fn palette_of(dto: PaletteDto) -> Palette {
    Palette {
        dark: dto.dark,
        background: Rgb(dto.background),
        text: Rgb(dto.text),
        muted: Rgb(dto.muted),
        accent: Rgb(dto.accent),
        border: Rgb(dto.border),
        surface: Rgb(dto.surface),
        heading: dto.heading.map(Rgb),
        quote: Rgb(dto.quote),
        task_done: Rgb(dto.task_done),
        code_text: dto.code_text.map(Rgb),
        inline_code_background: Rgb(dto.inline_code_background),
        inline_code_text: dto.inline_code_text.map(Rgb),
        table_header_background: dto.table_header_background.map(Rgb),
    }
}
