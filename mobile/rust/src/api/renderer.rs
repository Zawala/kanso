//! Drawing: a note's preview, highlighting, themes, math and diagrams.

use flutter_rust_bridge::frb;
use kanso_mobile::{DiagramKind, KansoError, PaletteDto, Renderer, SpanRow, SvgRow, ThemeTokens};

#[frb(opaque)]
pub struct KansoRenderer(Renderer);

impl KansoRenderer {
    pub(super) fn new(renderer: Renderer) -> KansoRenderer {
        KansoRenderer(renderer)
    }

    pub fn preview_html(&self, id: String, palette: PaletteDto) -> Result<String, KansoError> {
        self.0.preview_html(&id, palette)
    }

    pub fn highlight(text: String, start: u32, end: u32) -> Result<Vec<SpanRow>, KansoError> {
        Renderer::highlight(text, start, end)
    }

    pub fn theme_tokens(theme_json: String) -> Result<ThemeTokens, KansoError> {
        Renderer::theme_tokens(theme_json)
    }

    pub fn math_svg(
        source: String,
        display: bool,
        palette: PaletteDto,
    ) -> Result<SvgRow, KansoError> {
        Renderer::math_svg(source, display, palette)
    }

    pub fn diagram_svg(
        kind: DiagramKind,
        source: String,
        palette: PaletteDto,
    ) -> Result<SvgRow, KansoError> {
        Renderer::diagram_svg(kind, source, palette)
    }

    pub fn set_fonts(fonts: Vec<Vec<u8>>) -> bool {
        Renderer::set_fonts(fonts)
    }
}
