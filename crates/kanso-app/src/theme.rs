//! The colors the views draw with, and the component library's theme built
//! from them. Kanso's own two themes, warm graphite (dark, the primary one)
//! and soft paper (light), are [`DARK`] and [`LIGHT`]; `themes/kanso-dark.json`
//! and `themes/kanso-light.json` give the same colors, and every other theme
//! is read from a file the same way (see `theme_file.rs` and `themes.rs`).
//!
//! The palettes and the file format are `kanso_engine::theme`'s, shared with
//! the mobile app. This file keeps their drawing side: [`Rgb`] and
//! [`Colors`] here turn into the component library's colors (an engine type
//! can't, as neither it nor the library's is this crate's), so the views
//! read `colors.text` and hand it straight to `.text_color()`.
//!
//! The component library gets the theme in use as a theme file, so its
//! buttons, menus and inputs match. The views read [`Colors`] directly for
//! what the library can't express: three pane surfaces, secondary text, and
//! selected rows (the library caps a list's selected background at 20%
//! opacity).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use gpui_kit::assets::IconName;
use gpui_kit::component::Theme;
use gpui_kit::{App, Fill, FontWeight, Hsla, Styled, TextSystem, font};

use crate::zoom::px;
use kanso_core::NoteStatus;
use kanso_engine::theme as engine;
use serde_json::{Value, json};

use crate::theme_file::{CAPTURES, ResolvedTheme, TokenStyle};

pub use engine::{CURSOR_COLOURS, PALETTE_SELECTED_ALPHA, muted_tag_color};

/// An sRGB color, `0xRRGGBB`: the engine's [`engine::Rgb`] that GPUI can
/// draw with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u32);

impl Rgb {
    /// `#RRGGBB`.
    pub fn hex(self) -> String {
        engine::Rgb::from(self).hex()
    }

    /// `#RRGGBBAA`, with `alpha` out of 255.
    pub(crate) fn hex_alpha(self, alpha: u8) -> String {
        engine::Rgb::from(self).hex_alpha(alpha)
    }
}

impl From<engine::Rgb> for Rgb {
    fn from(color: engine::Rgb) -> Rgb {
        Rgb(color.0)
    }
}

impl From<Rgb> for engine::Rgb {
    fn from(color: Rgb) -> engine::Rgb {
        engine::Rgb(color.0)
    }
}

impl From<Rgb> for Hsla {
    fn from(color: Rgb) -> Hsla {
        gpui_kit::rgb(color.0).into()
    }
}

impl From<Rgb> for Fill {
    fn from(color: Rgb) -> Fill {
        Hsla::from(color).into()
    }
}

/// An engine color GPUI can draw with.
pub fn hsla(color: engine::Rgb) -> Hsla {
    Hsla::from(Rgb::from(color))
}

/// One theme's colors, as the engine has them, drawable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colors {
    /// Behind everything; the title bar.
    pub app: Rgb,
    pub sidebar: Rgb,
    pub list: Rgb,
    /// The editor and preview, and the theme's `background`.
    pub editor: Rgb,
    /// Menus and popovers.
    pub elevated: Rgb,
    /// Chips, banners and other quiet fills on the panes: a step off every
    /// pane, in both themes. (In light mode the elevated color is the
    /// lightest surface of all, so a fill in it would vanish.)
    pub fill: Rgb,
    pub text: Rgb,
    pub text_secondary: Rgb,
    pub text_muted: Rgb,
    pub divider: Rgb,
    pub accent: Rgb,
    /// Link text: the accent, shaded to stay readable on every surface.
    pub link: Rgb,
    /// The selected and hovered note row, on [`Self::list`].
    pub row_active: Rgb,
    pub row_hover: Rgb,
    /// The selected and hovered sidebar row, on [`Self::sidebar`].
    pub sidebar_active: Rgb,
    pub sidebar_hover: Rgb,
    /// Menu items and ghost buttons under the pointer.
    pub hover: Rgb,
    /// A selected ghost button, such as the current view mode: plainly
    /// stronger than a hover, in both themes.
    pub selected: Rgb,
    /// The row with the cursor, when line numbers show.
    pub active_line: Rgb,
    /// Primary buttons and the text on them.
    pub button: Rgb,
    pub button_text: Rgb,
    /// Code blocks in the preview.
    pub code: Rgb,
    /// Inline code, and literals in code blocks.
    pub code_text: Rgb,
    /// Keywords and strings in code blocks: present, but quiet.
    pub syntax_keyword: Rgb,
    pub syntax_string: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
}

impl Colors {
    /// The engine's colors for these.
    pub const fn to_engine(self) -> engine::Colors {
        let Colors {
            app,
            sidebar,
            list,
            editor,
            elevated,
            fill,
            text,
            text_secondary,
            text_muted,
            divider,
            accent,
            link,
            row_active,
            row_hover,
            sidebar_active,
            sidebar_hover,
            hover,
            selected,
            active_line,
            button,
            button_text,
            code,
            code_text,
            syntax_keyword,
            syntax_string,
            success,
            warning,
            danger,
        } = self;
        engine::Colors {
            app: engine::Rgb(app.0),
            sidebar: engine::Rgb(sidebar.0),
            list: engine::Rgb(list.0),
            editor: engine::Rgb(editor.0),
            elevated: engine::Rgb(elevated.0),
            fill: engine::Rgb(fill.0),
            text: engine::Rgb(text.0),
            text_secondary: engine::Rgb(text_secondary.0),
            text_muted: engine::Rgb(text_muted.0),
            divider: engine::Rgb(divider.0),
            accent: engine::Rgb(accent.0),
            link: engine::Rgb(link.0),
            row_active: engine::Rgb(row_active.0),
            row_hover: engine::Rgb(row_hover.0),
            sidebar_active: engine::Rgb(sidebar_active.0),
            sidebar_hover: engine::Rgb(sidebar_hover.0),
            hover: engine::Rgb(hover.0),
            selected: engine::Rgb(selected.0),
            active_line: engine::Rgb(active_line.0),
            button: engine::Rgb(button.0),
            button_text: engine::Rgb(button_text.0),
            code: engine::Rgb(code.0),
            code_text: engine::Rgb(code_text.0),
            syntax_keyword: engine::Rgb(syntax_keyword.0),
            syntax_string: engine::Rgb(syntax_string.0),
            success: engine::Rgb(success.0),
            warning: engine::Rgb(warning.0),
            danger: engine::Rgb(danger.0),
        }
    }

    /// The app's colors for the engine's: every field, named, so that a color
    /// added to the engine fails to compile here until it is mirrored.
    pub const fn from_engine(colors: &engine::Colors) -> Colors {
        let engine::Colors {
            app,
            sidebar,
            list,
            editor,
            elevated,
            fill,
            text,
            text_secondary,
            text_muted,
            divider,
            accent,
            link,
            row_active,
            row_hover,
            sidebar_active,
            sidebar_hover,
            hover,
            selected,
            active_line,
            button,
            button_text,
            code,
            code_text,
            syntax_keyword,
            syntax_string,
            success,
            warning,
            danger,
        } = colors;
        Colors {
            app: Rgb(app.0),
            sidebar: Rgb(sidebar.0),
            list: Rgb(list.0),
            editor: Rgb(editor.0),
            elevated: Rgb(elevated.0),
            fill: Rgb(fill.0),
            text: Rgb(text.0),
            text_secondary: Rgb(text_secondary.0),
            text_muted: Rgb(text_muted.0),
            divider: Rgb(divider.0),
            accent: Rgb(accent.0),
            link: Rgb(link.0),
            row_active: Rgb(row_active.0),
            row_hover: Rgb(row_hover.0),
            sidebar_active: Rgb(sidebar_active.0),
            sidebar_hover: Rgb(sidebar_hover.0),
            hover: Rgb(hover.0),
            selected: Rgb(selected.0),
            active_line: Rgb(active_line.0),
            button: Rgb(button.0),
            button_text: Rgb(button_text.0),
            code: Rgb(code.0),
            code_text: Rgb(code_text.0),
            syntax_keyword: Rgb(syntax_keyword.0),
            syntax_string: Rgb(syntax_string.0),
            success: Rgb(success.0),
            warning: Rgb(warning.0),
            danger: Rgb(danger.0),
        }
    }
}

pub const DARK: Colors = Colors::from_engine(&engine::DARK);

pub const LIGHT: Colors = Colors::from_engine(&engine::LIGHT);

impl Colors {
    /// The colors of the UI theme in use; Kanso's own light or dark before
    /// one is applied.
    pub fn get(cx: &App) -> &'static Colors {
        if let Some(active) = cx.try_global::<crate::themes::ActiveTheme>() {
            return active.colors;
        }
        if Theme::global(cx).is_dark() {
            &DARK
        } else {
            &LIGHT
        }
    }

    /// Quiet text that still says something (a row's time, a count, a
    /// hint, a shortcut): muted on a pane, and secondary on a `selected`
    /// row, whose fill takes muted text under 4.5:1.
    // no-prop-test: two cases per theme; the engine's `quiet` has
    // `quiet_text_reads_on_selected_rows`, and `the_mirror_draws_like_the_engine`
    // checks this one against it.
    pub fn quiet(&self, selected: bool) -> Rgb {
        if selected {
            self.text_secondary
        } else {
            self.text_muted
        }
    }
}

/// The colour of cursor `index` (see [`CURSOR_COLOURS`]) on a light or dark
/// editor.
pub fn cursor_colour(index: usize, dark: bool) -> Hsla {
    let (light, on_dark) = CURSOR_COLOURS[index % CURSOR_COLOURS.len()];
    hsla(if dark { on_dark } else { light })
}

/// The size of a section's label.
pub const SECTION_LABEL_SIZE: f32 = 12.;

/// A status's glyph and its color, the same in the note list, the editor,
/// the sidebar and the palette. The shape says which status it is, not
/// only the color: Active in the accent, On hold in warning, Completed in
/// success, and Dropped in `quiet`, the quiet color of wherever it sits. A
/// note without a status has no glyph.
// no-prop-test: a table of five values; `status_glyphs_differ_and_read`
// checks every one on every surface.
pub fn status_glyph(status: NoteStatus, colors: &Colors, quiet: Rgb) -> Option<(IconName, Rgb)> {
    match status {
        NoteStatus::Active => Some((IconName::CircleDot, colors.accent)),
        NoteStatus::OnHold => Some((IconName::CirclePause, colors.warning)),
        NoteStatus::Completed => Some((IconName::CircleCheck, colors.success)),
        NoteStatus::Dropped => Some((IconName::CircleX, quiet)),
        NoteStatus::None => None,
    }
}

/// A section's label, in one quiet style in every pane: the sidebar's
/// "Notebooks", the list's "Today", the palette's "Note actions".
// no-prop-test: a style; the snapshots show it.
pub fn section_label<E: Styled>(el: E, colors: &Colors) -> E {
    el.text_size(px(SECTION_LABEL_SIZE))
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors.text_muted)
}

/// SF Mono, the system's monospaced face, by the name Core Text knows it
/// by. It isn't installed as a family called "SF Mono", so it never shows in
/// the font list.
pub(crate) const SYSTEM_MONO: &str = ".AppleSystemUIFontMonospaced";

/// SF Mono when it loads. A family that doesn't load quietly becomes a
/// proportional fallback, so it is checked rather than assumed; without it
/// the library keeps its own default, Menlo.
pub(crate) fn mono_font(cx: &App) -> Option<&'static str> {
    static FOUND: OnceLock<bool> = OnceLock::new();
    FOUND
        .get_or_init(|| loads_monospaced(cx.text_system(), SYSTEM_MONO))
        .then_some(SYSTEM_MONO)
}

/// Whether `family` loads as itself, rather than as a fallback, and is
/// monospaced.
// no-prop-test: asks the platform's text system; `system_mono_font_loads`
// checks it against real fonts.
fn loads_monospaced(text_system: &TextSystem, family: &str) -> bool {
    let id = text_system.resolve_font(&font(family));
    let itself = text_system
        .get_font_for_id(id)
        .is_some_and(|loaded| loaded.family.as_ref() == family);
    let width = |c| text_system.advance(id, px(16.), c).ok().map(|s| s.width);
    itself && width('i').is_some() && width('i') == width('m')
}

/// Kanso's two themes as one theme set, as the component library reads
/// theme files.
#[cfg(test)]
pub(crate) fn theme_set(mono: Option<&str>) -> Value {
    let config = |ix: usize| {
        let theme = &crate::themes::builtins()[ix].resolved;
        theme_config(
            theme,
            &Colors::from_engine(&theme.colors),
            &theme.syntax,
            mono,
        )
    };
    json!({ "name": "Kanso", "themes": [config(0), config(1)] })
}

/// The component library's theme file for the UI theme `ui`, drawn in
/// `colors` (its own, fitted), with the editor's syntax in `syntax`.
// no-prop-test: a mapping onto the library's keys; `themes_parse` and
// `kanso_themes_match_the_palettes` check what it writes.
pub(crate) fn theme_config(
    ui: &ResolvedTheme,
    c: &Colors,
    syntax: &BTreeMap<&'static str, TokenStyle>,
    mono: Option<&str>,
) -> Value {
    let dark = ui.dark;
    let mut config = json!({
        "name": ui.name,
        "mode": if dark { "dark" } else { "light" },
        "radius": 6,
        "radius.lg": 10,
        "shadow": false,
        "colors": theme_colors(dark, c, ui),
        "highlight": highlight_style(c, syntax),
    });
    if let Some(mono) = mono {
        config["mono_font.family"] = json!(mono);
    }
    config
}

/// The component library's color keys.
fn theme_colors(dark: bool, c: &Colors, ui: &ResolvedTheme) -> Value {
    // Text on the status colors: dark on the light dark-theme ones, paper
    // on the shaded light-theme ones.
    let on_status = if dark { c.app } else { c.button_text };
    let colors: [(&str, String); 62] = [
        ("background", c.editor.hex()),
        ("foreground", c.text.hex()),
        ("border", c.divider.hex()),
        ("input.border", c.divider.hex()),
        ("accent.background", c.hover.hex()),
        ("accent.foreground", c.text.hex()),
        ("secondary.background", c.hover.hex()),
        ("secondary.foreground", c.text.hex()),
        // A selected ghost button; the library would darken the secondary
        // color, which all but vanishes on the dark editor. Hover sits
        // between the surface and the selected fill.
        ("secondary.active.background", c.selected.hex()),
        (
            "secondary.hover.background",
            (if dark { c.elevated } else { c.hover }).hex(),
        ),
        ("button.active.background", c.selected.hex()),
        ("muted.background", c.fill.hex()),
        ("muted.foreground", c.text_muted.hex()),
        ("popover.background", c.elevated.hex()),
        ("popover.foreground", c.text.hex()),
        ("primary.background", c.button.hex()),
        ("primary.foreground", c.button_text.hex()),
        // A switch that is off still reads as a switch: a grey track with a
        // light thumb, as on macOS. On, the track takes the primary color.
        ("switch.background", c.text_muted.hex()),
        (
            "switch.thumb.background",
            (if dark { c.text } else { c.elevated }).hex(),
        ),
        ("ring", c.accent.hex()),
        ("caret", ui.caret.hex()),
        (
            "selection.background",
            ui.selection.rgb.hex_alpha(ui.selection.alpha),
        ),
        ("link", c.link.hex()),
        ("link.hover", c.link.hex()),
        ("link.active", c.link.hex()),
        ("list.background", c.list.hex()),
        ("list.even.background", c.list.hex()),
        ("list.head.background", c.list.hex()),
        ("list.active.background", c.text.hex_alpha(0x14)),
        ("list.active.border", c.accent.hex()),
        ("list.hover.background", c.text.hex_alpha(0x0A)),
        ("sidebar.background", c.sidebar.hex()),
        ("sidebar.foreground", c.text.hex()),
        ("sidebar.border", c.divider.hex()),
        ("sidebar.accent.background", c.sidebar_active.hex()),
        ("sidebar.accent.foreground", c.text.hex()),
        ("sidebar.primary.background", c.accent.hex()),
        ("sidebar.primary.foreground", c.app.hex()),
        ("title_bar.background", c.app.hex()),
        ("title_bar.border", c.divider.hex()),
        ("status_bar.background", c.app.hex()),
        ("status_bar.border", c.divider.hex()),
        ("window.border", c.divider.hex()),
        ("scrollbar.background", "#00000000".into()),
        ("scrollbar.thumb.background", c.text.hex_alpha(0x33)),
        ("scrollbar.thumb.hover.background", c.text.hex_alpha(0x55)),
        ("drag.border", c.accent.hex()),
        ("drop_target.background", c.accent.hex_alpha(0x33)),
        // A warm dim, never black.
        (
            "overlay",
            if dark {
                c.app.hex_alpha(0x99)
            } else {
                c.text.hex_alpha(0x26)
            },
        ),
        ("success.background", c.success.hex()),
        ("success.foreground", on_status.hex()),
        ("warning.background", c.warning.hex()),
        ("warning.foreground", on_status.hex()),
        ("danger.background", c.danger.hex()),
        ("danger.foreground", on_status.hex()),
        ("info.background", c.accent.hex()),
        ("info.foreground", on_status.hex()),
        ("base.blue", c.accent.hex()),
        ("base.cyan", c.accent.hex()),
        ("base.green", c.success.hex()),
        ("base.red", c.danger.hex()),
        ("base.yellow", c.warning.hex()),
    ];
    Value::Object(
        colors
            .into_iter()
            .map(|(key, value)| (key.to_owned(), Value::String(value)))
            .collect(),
    )
}

/// Editor colors and syntax styles: each highlight capture in its syntax
/// key's style (see [`CAPTURES`]). In Kanso's own themes Markdown source
/// stays calm: headings are bold in the text color, markers recede, and code
/// is legible rather than colorful.
pub(crate) fn highlight_style(c: &Colors, syntax: &BTreeMap<&'static str, TokenStyle>) -> Value {
    let styles: serde_json::Map<String, Value> = CAPTURES
        .iter()
        .filter_map(|(capture, key)| {
            let style = syntax.get(key)?;
            (!style.is_empty()).then(|| ((*capture).to_owned(), style.to_highlight_json()))
        })
        .collect();
    let foreground = syntax
        .get("text")
        .and_then(|s| s.color)
        .map_or_else(|| c.text.hex(), |color| color.hex());
    // There are no line number keys: the gutter draws numbers in the
    // theme's `muted.foreground`, and the current line's in `foreground`.
    json!({
        "editor.background": c.editor.hex(),
        "editor.foreground": foreground,
        "editor.active_line.background": c.active_line.hex(),
        "editor.invisible": c.text_muted.hex_alpha(0x66),
        "error": c.danger.hex(),
        "warning": c.warning.hex(),
        "info": c.accent.hex(),
        "success": c.success.hex(),
        "hint": c.text_muted.hex(),
        "syntax": styles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::component::ThemeSet;

    /// The WCAG contrast ratio of two colors.
    fn contrast_ratio(a: u32, b: u32) -> f64 {
        engine::file::contrast(engine::Rgb(a), engine::Rgb(b))
    }

    /// `top` over `bottom` at `alpha`.
    fn mix(top: Rgb, bottom: Rgb, alpha: f64) -> Rgb {
        engine::file::mix(top.into(), bottom.into(), alpha).into()
    }

    /// The app's palettes are the engine's, field for field, and quiet text
    /// is chosen the same way.
    #[test]
    fn the_mirror_draws_like_the_engine() {
        for (app, engine) in [(&DARK, &engine::DARK), (&LIGHT, &engine::LIGHT)] {
            assert_eq!(app.text, Rgb::from(engine.text));
            for selected in [false, true] {
                assert_eq!(app.quiet(selected), Rgb::from(engine.quiet(selected)));
            }
        }
        assert_eq!(Rgb(0x123456).hex(), "#123456");
        assert_eq!(Rgb(0x123456).hex_alpha(0x80), "#12345680");
    }

    /// A status reads by its shape and its color alike: each has its own
    /// glyph, none for no status, and the colored ones read at 3:1 on every
    /// surface a glyph sits on, the palette's selected row included; Dropped
    /// takes the quiet color it is given.
    #[test]
    fn status_glyphs_differ_and_read() {
        for (name, c) in [("dark", &DARK), ("light", &LIGHT)] {
            let quiet = Rgb(0x123456);
            let glyphs: Vec<_> = NoteStatus::ALL
                .into_iter()
                .filter_map(|status| status_glyph(status, c, quiet).map(|g| (status, g)))
                .collect();
            assert_eq!(glyphs.len(), NoteStatus::ALL.len() - 1, "{name}");
            assert_eq!(status_glyph(NoteStatus::None, c, quiet), None);
            for (ix, (status, (icon, _))) in glyphs.iter().enumerate() {
                assert!(
                    glyphs[ix + 1..].iter().all(|(_, (other, _))| other != icon),
                    "{name}: {status:?} shares its glyph"
                );
            }
            let alpha = if name == "dark" {
                PALETTE_SELECTED_ALPHA.0
            } else {
                PALETTE_SELECTED_ALPHA.1
            };
            let surfaces = [
                ("sidebar", c.sidebar),
                ("sidebar_active", c.sidebar_active),
                ("list", c.list),
                ("row_active", c.row_active),
                ("fill", c.fill),
                ("elevated", c.elevated),
                (
                    "palette_selected",
                    mix(c.accent, c.elevated, f64::from(alpha)),
                ),
            ];
            for (status, (_, tint)) in glyphs {
                if status == NoteStatus::Dropped {
                    assert_eq!(tint, quiet, "{name}");
                    continue;
                }
                for (surface, bg) in surfaces {
                    let ratio = contrast_ratio(tint.0, bg.0);
                    assert!(
                        ratio >= 3.0,
                        "{name}: {status:?} on {surface} is {ratio:.2}, wants 3"
                    );
                }
            }
        }
    }

    /// SF Mono loads under its system name, and the check tells a
    /// monospaced face from a proportional one or a missing family.
    #[test]
    fn system_mono_font_loads() {
        let text_system = TextSystem::new(crate::snapshots::text_system());
        assert!(loads_monospaced(&text_system, SYSTEM_MONO));
        assert!(loads_monospaced(&text_system, "Menlo"));
        assert!(!loads_monospaced(&text_system, "Helvetica"));
        assert!(!loads_monospaced(&text_system, "No Such Family Kanso"));
    }

    /// The editor colors and syntax styles Kanso drew before themes were
    /// files, kept to check that its own theme files draw the same, except
    /// for the code colors, which are the ones the theme files now give:
    /// `[keyword, string, number, function, type, attribute, punctuation,
    /// property]`.
    fn legacy_highlight(c: &Colors) -> Value {
        let color = |rgb: Rgb| json!({ "color": rgb.hex() });
        let [
            keyword,
            string,
            number,
            function,
            ty,
            attribute,
            punctuation,
            property,
        ] = if c.editor == LIGHT.editor {
            [
                "#3F6496", "#4A7A35", "#A0522D", "#8A6416", "#2F7A6E", "#7A4F8F", "#6F695E",
                "#3A362F",
            ]
        } else {
            [
                "#8FB0D9", "#A8C98A", "#E0A882", "#E6C68E", "#8EC5B8", "#C3A6D6", "#A29E94",
                "#DAD6CB",
            ]
        };
        let color_hex = |hex: &str| json!({ "color": hex });
        let muted = color(c.text_muted);
        let secondary = color(c.text_secondary);
        let mut syntax = json!({
                "title": { "color": c.text.hex(), "font_weight": 700 },
                "punctuation.special": muted,
                "punctuation.list_marker": muted,
                "punctuation.delimiter": muted,
                "text.literal": secondary,
                "text.code.span": color(c.code_text),
                "link_text": color(c.link),
                "link_uri": muted,
                "emphasis": { "font_style": "italic" },
                "emphasis.strong": { "font_weight": 700 },
                "string.escape": muted,
                "comment": { "color": c.text_muted.hex(), "font_style": "italic" },
                "comment.doc": { "color": c.text_muted.hex(), "font_style": "italic" },
                "keyword": color_hex(keyword),
                "tag": color_hex(keyword),
                "constructor": color_hex(keyword),
        });
        let rest = json!({
                "string": color_hex(string),
                "string.special": color_hex(string),
                "number": color_hex(number),
                "boolean": color_hex(number),
                "constant": color_hex(number),
                "variable.special": color_hex(number),
                "type": color_hex(ty),
                "attribute": color_hex(attribute),
                "label": color_hex(attribute),
                "operator": color_hex(punctuation),
                "punctuation": color_hex(punctuation),
                "punctuation.bracket": color_hex(punctuation),
                "function": color_hex(function),
                "property": color_hex(property),
                "variable": color(c.text),
                "embedded": color(c.text),

        });
        if let (Value::Object(syntax), Value::Object(rest)) = (&mut syntax, rest) {
            syntax.extend(rest);
        }
        json!({
            "editor.background": c.editor.hex(),
            "editor.foreground": c.text.hex(),
            "editor.active_line.background": c.active_line.hex(),
            "editor.invisible": c.text_muted.hex_alpha(0x66),
            "error": c.danger.hex(),
            "warning": c.warning.hex(),
            "info": c.accent.hex(),
            "success": c.success.hex(),
            "hint": c.text_muted.hex(),
            "syntax": syntax,
        })
    }

    /// Kanso's own theme files give exactly the palettes above, the same
    /// component theme, and the same syntax styles, plus the two new
    /// captures for rules and task boxes, which take the list markers'
    /// color they had before, and two for code that take the special
    /// strings'. Selection and caret are the accent's.
    #[test]
    fn kanso_themes_match_the_palettes() {
        use gpui_kit::component::highlighter::HighlightThemeStyle;
        for (ix, expected) in [(0, &LIGHT), (1, &DARK)] {
            let theme = &crate::themes::builtins()[ix].resolved;
            assert_eq!(
                Colors::from_engine(&theme.colors),
                *expected,
                "{}",
                theme.name
            );
            assert_eq!(Rgb::from(theme.caret), expected.accent);
            assert_eq!(Rgb::from(theme.selection.rgb), expected.accent);
            assert_eq!(theme.selection.alpha, 0x4D);
            // Fitting to its own surface changes nothing.
            let fitted = crate::theme_file::fit_syntax(
                &theme.syntax,
                expected.editor.into(),
                expected.text.into(),
            );
            assert_eq!(fitted, theme.syntax, "{}", theme.name);
            let new: HighlightThemeStyle =
                serde_json::from_value(highlight_style(expected, &fitted)).unwrap();
            let mut old: HighlightThemeStyle =
                serde_json::from_value(legacy_highlight(expected)).unwrap();
            assert_eq!(new.syntax.variant, old.syntax.punctuation_list_marker);
            assert_eq!(new.syntax.enum_, old.syntax.punctuation_list_marker);
            old.syntax.variant = new.syntax.variant;
            old.syntax.enum_ = new.syntax.enum_;
            // Regular expressions and symbols in code take the special
            // strings' style rather than falling back to plain strings.
            assert_eq!(new.syntax.string_regex, new.syntax.string_special);
            assert_eq!(new.syntax.string_special_symbol, new.syntax.string_special);
            old.syntax.string_regex = new.syntax.string_regex;
            old.syntax.string_special_symbol = new.syntax.string_special_symbol;
            assert_eq!(new, old, "{}", theme.name);
            let preview = crate::theme_file::fit_preview(&theme.preview.into(), expected);
            assert_eq!(preview, theme.preview.into());
            assert_eq!(preview.code_background, expected.code);
            assert_eq!(preview.text, expected.text);
            assert_eq!(preview.quote, expected.text_secondary);
            assert_eq!(preview.border, expected.divider);
            assert_eq!(preview.link, expected.link);
            assert_eq!(preview.inline_code_background, expected.code);
            assert_eq!(
                (preview.heading, preview.code_text, preview.inline_code_text),
                (None, None, None)
            );
            assert_eq!(preview.table_header_background, None);
            assert_eq!(preview.selection, None);
        }
    }

    /// The theme files load with the colors as written.
    #[test]
    fn themes_parse() {
        let set: ThemeSet = serde_json::from_value(theme_set(Some(SYSTEM_MONO))).unwrap();
        assert_eq!(set.themes.len(), 2);
        let dark = &set.themes[1];
        assert_eq!(dark.name.as_ref(), crate::config::KANSO_DARK);
        assert!(dark.mode.is_dark());
        assert_eq!(dark.mono_font_family.as_deref(), Some(SYSTEM_MONO));
        assert_eq!(dark.colors.background.as_deref(), Some("#282824"));
        assert_eq!(dark.colors.sidebar.as_deref(), Some("#1D1D1A"));
        assert_eq!(dark.colors.secondary_active.as_deref(), Some("#3A3933"));
        assert_eq!(dark.colors.secondary_hover.as_deref(), Some("#30302B"));
        assert_eq!(dark.colors.button_active.as_deref(), Some("#3A3933"));
        assert_eq!(dark.colors.overlay.as_deref(), Some("#18181699"));
        assert_eq!(dark.colors.switch.as_deref(), Some("#7E7A70"));
        assert_eq!(dark.colors.switch_thumb.as_deref(), Some("#F1EFE7"));
        assert_eq!(dark.colors.muted.as_deref(), Some("#30302B"));
        let light = &set.themes[0].colors;
        assert_eq!(light.muted.as_deref(), Some("#E6E0D4"));
        assert_eq!(light.popover.as_deref(), Some("#FFFDF8"));
        assert_eq!(light.danger_foreground.as_deref(), Some("#FFFDF8"));
        assert_eq!(light.secondary_active.as_deref(), Some("#E6E0D4"));
        assert_eq!(light.secondary_hover.as_deref(), Some("#F0ECE3"));
        let highlight = dark.highlight.as_ref().expect("highlight");
        assert_eq!(highlight.editor_background, Some(Hsla::from(DARK.editor)));
        assert!(highlight.syntax.title.is_some());
        assert!(!set.themes[0].mode.is_dark());
    }
}
