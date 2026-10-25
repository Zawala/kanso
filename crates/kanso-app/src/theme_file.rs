//! Kanso's theme files; `kanso_engine::theme::file` has the format, how a
//! theme resolves and how its colors are fitted. The preview's colors are
//! mirrored here as [`PreviewColors`], drawable by GPUI like [`Colors`]
//! (`theme.rs`), with conversions both ways.

use crate::theme::{Colors, Rgb};
use kanso_engine::theme::file as engine;

pub use engine::*;

/// The preview's colors. `None` leaves the part in the body text's color, or
/// (for a table's header) in the code background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewColors {
    pub text: Rgb,
    pub heading: Option<Rgb>,
    pub link: Rgb,
    pub quote: Rgb,
    /// Finished tasks and list markers, which step back from the text.
    pub task_done: Rgb,
    pub border: Rgb,
    pub code_background: Rgb,
    pub code_text: Option<Rgb>,
    pub inline_code_background: Rgb,
    pub inline_code_text: Option<Rgb>,
    pub table_header_background: Option<Rgb>,
    /// `None`: the accent at 30%.
    pub selection: Option<Color>,
}

impl From<engine::PreviewColors> for PreviewColors {
    fn from(colors: engine::PreviewColors) -> PreviewColors {
        let engine::PreviewColors {
            text,
            heading,
            link,
            quote,
            task_done,
            border,
            code_background,
            code_text,
            inline_code_background,
            inline_code_text,
            table_header_background,
            selection,
        } = colors;
        PreviewColors {
            text: text.into(),
            heading: heading.map(Rgb::from),
            link: link.into(),
            quote: quote.into(),
            task_done: task_done.into(),
            border: border.into(),
            code_background: code_background.into(),
            code_text: code_text.map(Rgb::from),
            inline_code_background: inline_code_background.into(),
            inline_code_text: inline_code_text.map(Rgb::from),
            table_header_background: table_header_background.map(Rgb::from),
            selection,
        }
    }
}

impl From<PreviewColors> for engine::PreviewColors {
    fn from(colors: PreviewColors) -> engine::PreviewColors {
        let PreviewColors {
            text,
            heading,
            link,
            quote,
            task_done,
            border,
            code_background,
            code_text,
            inline_code_background,
            inline_code_text,
            table_header_background,
            selection,
        } = colors;
        engine::PreviewColors {
            text: text.into(),
            heading: heading.map(Into::into),
            link: link.into(),
            quote: quote.into(),
            task_done: task_done.into(),
            border: border.into(),
            code_background: code_background.into(),
            code_text: code_text.map(Into::into),
            inline_code_background: inline_code_background.into(),
            inline_code_text: inline_code_text.map(Into::into),
            table_header_background: table_header_background.map(Into::into),
            selection,
        }
    }
}

/// Preview colors fitted to the pane they are drawn on.
// no-prop-test: converts to and from the engine's types around
// kanso_engine::theme::file::fit_preview, tested there.
pub fn fit_preview(preview: &PreviewColors, ui: &Colors) -> PreviewColors {
    engine::fit_preview(&(*preview).into(), &ui.to_engine()).into()
}
