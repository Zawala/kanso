//! Drawing: a note's preview, highlighting, themes, math and diagrams.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    BuiltinTheme, DiagramKind, DiagramRenderer, KansoError, LinkTarget, PaletteDto, PreviewOptions,
    Renderer, SpanRow, StyledRun, SvgRow, ThemeTokens,
};

#[frb(opaque)]
pub struct KansoRenderer(Renderer);

impl KansoRenderer {
    pub(super) fn new(renderer: Renderer) -> KansoRenderer {
        KansoRenderer(renderer)
    }

    pub fn preview_html(&self, id: String, palette: PaletteDto) -> Result<String, KansoError> {
        self.0.preview_html(&id, palette)
    }

    pub fn preview_html_with(
        &self,
        id: String,
        palette: PaletteDto,
        options: PreviewOptions,
    ) -> Result<String, KansoError> {
        self.0.preview_html_with(&id, palette, options)
    }

    pub fn link_target(&self, note_id: String, url: String) -> Result<LinkTarget, KansoError> {
        self.0.link_target(&note_id, &url)
    }

    pub fn code_block(&self, note_id: String, index: u32) -> Result<String, KansoError> {
        self.0.code_block(&note_id, index)
    }

    pub fn set_diagram_renderer(renderer: DiagramRenderer) -> Result<(), KansoError> {
        Renderer::set_diagram_renderer(renderer)
    }

    pub fn diagram_renderer() -> Result<DiagramRenderer, KansoError> {
        Renderer::diagram_renderer()
    }

    pub fn highlight(text: String, start: u32, end: u32) -> Result<Vec<SpanRow>, KansoError> {
        Renderer::highlight(text, start, end)
    }

    pub fn theme_tokens(theme_json: String) -> Result<ThemeTokens, KansoError> {
        Renderer::theme_tokens(theme_json)
    }

    pub fn builtin_themes() -> Result<Vec<BuiltinTheme>, KansoError> {
        Renderer::builtin_themes()
    }

    pub fn builtin_theme_tokens(id: String) -> Result<ThemeTokens, KansoError> {
        Renderer::builtin_theme_tokens(id)
    }

    pub fn styled_runs(
        text: String,
        start: u32,
        end: u32,
        theme_id: String,
    ) -> Result<Vec<StyledRun>, KansoError> {
        Renderer::styled_runs(text, start, end, theme_id)
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
