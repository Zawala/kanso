//! The colors the views draw with, and the component library's theme built
//! from them. Kanso's own two themes, warm graphite (dark, the primary one)
//! and soft paper (light), are [`DARK`] and [`LIGHT`]; `themes/kanso-dark.json`
//! and `themes/kanso-light.json` give the same colors, and every other theme
//! is read from a file the same way (see `theme_file.rs` and `themes.rs`).
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
use gpui_kit::{App, Fill, FontWeight, Hsla, Styled, TextSystem, font, px};
use kanso_core::NoteStatus;
use serde_json::{Value, json};

use crate::theme_file::{CAPTURES, ResolvedTheme, TokenStyle};

/// An sRGB color, `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u32);

impl Rgb {
    /// `#RRGGBB`.
    pub fn hex(self) -> String {
        format!("#{:06X}", self.0 & 0xFF_FFFF)
    }

    /// `#RRGGBBAA`, with `alpha` out of 255.
    pub(crate) fn hex_alpha(self, alpha: u8) -> String {
        format!("{}{alpha:02X}", self.hex())
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

/// One theme's colors.
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

pub const DARK: Colors = Colors {
    app: Rgb(0x181816),
    sidebar: Rgb(0x1D1D1A),
    list: Rgb(0x232320),
    editor: Rgb(0x282824),
    elevated: Rgb(0x30302B),
    fill: Rgb(0x30302B),
    text: Rgb(0xF1EFE7),
    text_secondary: Rgb(0xB7B3A8),
    text_muted: Rgb(0x7E7A70),
    divider: Rgb(0x3A3933),
    accent: Rgb(0x7E96B3),
    link: Rgb(0x869DB9),
    row_active: Rgb(0x30302B),
    row_hover: Rgb(0x2A2A26),
    sidebar_active: Rgb(0x2E2E2A),
    sidebar_hover: Rgb(0x252522),
    hover: Rgb(0x3A3933),
    selected: Rgb(0x3A3933),
    active_line: Rgb(0x2D2D29),
    button: Rgb(0x4A6385),
    button_text: Rgb(0xF1EFE7),
    code: Rgb(0x30302B),
    code_text: Rgb(0xCDB892),
    syntax_keyword: Rgb(0x9AACC4),
    syntax_string: Rgb(0xA9BC98),
    success: Rgb(0x7BAA78),
    warning: Rgb(0xC49A5A),
    danger: Rgb(0xC96E63),
};

pub const LIGHT: Colors = Colors {
    app: Rgb(0xF4F1EA),
    sidebar: Rgb(0xEBE7DD),
    list: Rgb(0xF8F5EF),
    editor: Rgb(0xFCFAF4),
    elevated: Rgb(0xFFFDF8),
    fill: Rgb(0xE6E0D4),
    text: Rgb(0x26241F),
    text_secondary: Rgb(0x655F55),
    // The spec's #948B7D reads below 3:1 on most surfaces.
    text_muted: Rgb(0x857C6E),
    divider: Rgb(0xDDD6C9),
    accent: Rgb(0x5E7895),
    // The accent itself is under 4.5:1 for text.
    link: Rgb(0x4D6684),
    row_active: Rgb(0xECE7DC),
    row_hover: Rgb(0xF2EEE6),
    sidebar_active: Rgb(0xE2DDD0),
    sidebar_hover: Rgb(0xE5E0D5),
    hover: Rgb(0xF0ECE3),
    selected: Rgb(0xE6E0D4),
    active_line: Rgb(0xF6F3EB),
    button: Rgb(0x4D6684),
    button_text: Rgb(0xFFFDF8),
    code: Rgb(0xF4F1EA),
    code_text: Rgb(0x7D5B32),
    syntax_keyword: Rgb(0x4D6684),
    syntax_string: Rgb(0x55704A),
    // The spec's status colors are for the dark theme; on paper they fall
    // under 3:1, so they are shaded the way muted text and links are.
    success: Rgb(0x4F7F4C),
    warning: Rgb(0x8A6424),
    danger: Rgb(0xA6493F),
};

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
    // no-prop-test: two cases per theme; `quiet_text_reads_on_selected_rows`
    // checks every one.
    pub fn quiet(&self, selected: bool) -> Rgb {
        if selected {
            self.text_secondary
        } else {
            self.text_muted
        }
    }
}

/// How strongly the accent tints the palette's selected row over the
/// elevated panel, in the dark theme and the light one: enough to show, and
/// little enough that secondary text on it still reads at 4.5:1.
pub const PALETTE_SELECTED_ALPHA: (f32, f32) = (0.22, 0.18);

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
        theme_config(theme, &theme.colors, &theme.syntax, mono)
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
    let foreground = syntax.get("text").and_then(|s| s.color).unwrap_or(c.text);
    // There are no line number keys: the gutter draws numbers in the
    // theme's `muted.foreground`, and the current line's in `foreground`.
    json!({
        "editor.background": c.editor.hex(),
        "editor.foreground": foreground.hex(),
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

/// WCAG relative luminance of an sRGB color, 0 (black) to 1 (white).
#[cfg(test)]
fn luminance(rgb: u32) -> f64 {
    let channel = |shift: u32| {
        let c = f64::from((rgb >> shift) & 0xFF) / 255.;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}

/// The WCAG contrast ratio of two colors: 1 for the same color, 21 for black
/// on white. Body text wants 4.5 or more, large or incidental text 3. The
/// palette is checked with it in the tests below.
#[cfg(test)]
pub fn contrast_ratio(a: u32, b: u32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    ((light + 0.05) / (dark + 0.05)).clamp(1., 21.)
}

/// The most saturated a tag color gets.
const TAG_SATURATION_MAX: f32 = 0.42;
/// The lightness band tag colors are kept in, on dark and light surfaces.
const TAG_LIGHTNESS_DARK: (f32, f32) = (0.50, 0.76);
const TAG_LIGHTNESS_LIGHT: (f32, f32) = (0.32, 0.50);
/// The input lightness that is spread across the band. Tag palettes sit in
/// it; lighter and darker colors take the band's ends.
const TAG_LIGHTNESS_SPREAD: (f32, f32) = (0.40, 0.60);

/// A tag's color calmed for display: the same hue, no more saturated than
/// it was or than [`TAG_SATURATION_MAX`], and light or dark enough to read on
/// the theme's surfaces. Differences between colors survive it, so Orange
/// and Brown, or Violet and Purple, stay apart. The stored color doesn't
/// change.
pub fn muted_tag_color(rgb: u32, dark: bool) -> u32 {
    let (hue, saturation, lightness) = to_hsl(rgb);
    let (saturation, lightness) = calm(saturation, lightness, dark);
    from_hsl(hue, saturation, lightness)
}

/// The saturation and lightness [`muted_tag_color`] gives a color. Both keep
/// their order instead of being cut off: saturation is compressed toward the
/// cap by a square root, so a stronger color stays stronger, and lightness is
/// stretched across the band, so a darker color stays darker.
fn calm(saturation: f32, lightness: f32, dark: bool) -> (f32, f32) {
    let (low, high) = if dark {
        TAG_LIGHTNESS_DARK
    } else {
        TAG_LIGHTNESS_LIGHT
    };
    let (from, to) = TAG_LIGHTNESS_SPREAD;
    let spread = ((lightness - from) / (to - from)).clamp(0., 1.);
    (
        saturation.min(TAG_SATURATION_MAX * saturation.sqrt()),
        low + (high - low) * spread,
    )
}

/// Hue in degrees `0..360`, saturation and lightness in `0..=1`.
fn to_hsl(rgb: u32) -> (f32, f32, f32) {
    let [r, g, b] = [16, 8, 0].map(|shift| ((rgb >> shift) & 0xFF) as f32 / 255.);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.;
    let chroma = max - min;
    if chroma == 0. {
        return (0., 0., lightness);
    }
    let saturation = chroma / (1. - (2. * lightness - 1.).abs());
    let hue = if max == r {
        60. * ((g - b) / chroma).rem_euclid(6.)
    } else if max == g {
        60. * ((b - r) / chroma + 2.)
    } else {
        60. * ((r - g) / chroma + 4.)
    };
    (hue, saturation.min(1.), lightness)
}

fn from_hsl(hue: f32, saturation: f32, lightness: f32) -> u32 {
    let chroma = (1. - (2. * lightness - 1.).abs()) * saturation;
    let sector = hue.rem_euclid(360.) / 60.;
    let x = chroma * (1. - (sector.rem_euclid(2.) - 1.).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.),
        1 => (x, chroma, 0.),
        2 => (0., chroma, x),
        3 => (0., x, chroma),
        4 => (x, 0., chroma),
        _ => (chroma, 0., x),
    };
    let m = lightness - chroma / 2.;
    let byte = |v: f32| ((v + m).clamp(0., 1.) * 255.).round() as u32;
    (byte(r) << 16) | (byte(g) << 8) | byte(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::component::ThemeSet;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn rgb() -> impl Strategy<Value = u32> {
        0u32..=0xFF_FFFF
    }

    proptest! {
        #![proptest_config(config())]

        /// Symmetry and range: order doesn't matter, and the ratio is
        /// between 1 and 21, exactly 1 for a color against itself.
        #[test]
        fn prop_contrast_is_symmetric_and_bounded(a in rgb(), b in rgb()) {
            let ratio = contrast_ratio(a, b);
            prop_assert_eq!(ratio, contrast_ratio(b, a));
            prop_assert!((1.0..=21.0).contains(&ratio), "{ratio}");
            prop_assert_eq!(contrast_ratio(a, a), 1.0);
        }

        /// Monotonicity: a lighter grey stands out more against black and
        /// less against white.
        #[test]
        fn prop_contrast_grows_with_distance(a in 0u32..=255, b in 0u32..=255) {
            let (dark, light) = (a.min(b), a.max(b));
            let grey = |v: u32| v * 0x01_0101;
            prop_assert!(contrast_ratio(grey(light), 0) >= contrast_ratio(grey(dark), 0));
            prop_assert!(contrast_ratio(grey(light), 0xFF_FFFF) <= contrast_ratio(grey(dark), 0xFF_FFFF));
        }

        /// Invariant: a tag color keeps its hue, is never more saturated than
        /// it was or than the cap, and sits in its theme's lightness band.
        /// One step of 8-bit rounding is allowed for.
        #[test]
        fn prop_muted_tag_colors_keep_hue_and_calm_down(color in rgb(), dark in any::<bool>()) {
            let (hue, saturation, _) = to_hsl(color);
            let out = muted_tag_color(color, dark);
            let (out_hue, out_saturation, out_lightness) = to_hsl(out);
            let step = 1.5 / 255.;
            let (low, high) = if dark { TAG_LIGHTNESS_DARK } else { TAG_LIGHTNESS_LIGHT };
            prop_assert!(out_lightness >= low - step && out_lightness <= high + step, "{out_lightness}");
            // Saturation is measured against the band's own chroma limit, so
            // rounding near the band's edges costs a little more.
            let slack = 4. * step / (1. - (2. * out_lightness - 1.).abs());
            prop_assert!(out_saturation <= saturation.min(TAG_SATURATION_MAX) + slack,
                "{saturation} -> {out_saturation}");
            let chroma = (1. - (2. * out_lightness - 1.).abs()) * out_saturation;
            if chroma >= 0.15 {
                let diff = (out_hue - hue).abs();
                prop_assert!(diff.min(360. - diff) <= 3., "{hue} -> {out_hue}");
            }
        }

        /// Order preservation: a more saturated or lighter color never
        /// comes out less saturated or darker, and saturation doesn't depend
        /// on lightness or the other way round.
        #[test]
        fn prop_calming_keeps_order(a in unit(), b in unit(), other in unit(), dark in any::<bool>()) {
            let (less, more) = (a.min(b), a.max(b));
            let (s_less, l_less) = calm(less, other, dark);
            let (s_more, l_more) = calm(more, other, dark);
            prop_assert!(s_less <= s_more, "{less} -> {s_less}, {more} -> {s_more}");
            prop_assert_eq!(l_less, l_more);
            let (s_less, l_less) = calm(other, less, dark);
            let (s_more, l_more) = calm(other, more, dark);
            prop_assert!(l_less <= l_more, "{less} -> {l_less}, {more} -> {l_more}");
            prop_assert_eq!(s_less, s_more);
        }

        /// Strictness: different saturations stay different all the way up,
        /// and so do lightnesses across the spread, rather than meeting at
        /// the cap or the band's edge.
        #[test]
        fn prop_calming_keeps_colors_apart(a in unit(), gap in 0.01f32..1., dark in any::<bool>()) {
            let b = (a + gap).min(1.);
            prop_assume!(b - a >= 0.01);
            prop_assert!(calm(a, 0.5, dark).0 < calm(b, 0.5, dark).0, "{a} {b}");
            let (from, to) = TAG_LIGHTNESS_SPREAD;
            let (l1, l2) = (from + (to - from) * a, from + (to - from) * b);
            prop_assert!(calm(0.5, l1, dark).1 < calm(0.5, l2, dark).1, "{l1} {l2}");
        }
    }

    fn unit() -> impl Strategy<Value = f32> {
        0f32..=1.
    }

    /// CIE L*a*b* of an sRGB color (D65), for color differences.
    fn lab(rgb: u32) -> [f64; 3] {
        let [r, g, b] = [16, 8, 0].map(|shift| {
            let c = f64::from((rgb >> shift) & 0xFF) / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        });
        let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
        let f = |t: f64| {
            if t > 0.008856 {
                t.cbrt()
            } else {
                7.787 * t + 16. / 116.
            }
        };
        let (fx, fy, fz) = (f(x), f(y), f(z));
        [116. * fy - 16., 500. * (fx - fy), 200. * (fy - fz)]
    }

    /// The CIE76 difference of two colors: about 2 is just noticeable, 10
    /// tells two small dots apart at a glance.
    fn delta_e(a: u32, b: u32) -> f64 {
        let (a, b) = (lab(a), lab(b));
        (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
    }

    /// Every tag color stays distinct from every other in both themes, and
    /// readable as a dot on the surfaces tags sit on. The chromatic colors
    /// stay clearly apart; the three neutrals only differ in lightness, so
    /// they get a lower bar.
    #[test]
    fn tag_colors_stay_apart() {
        use kanso_core::TagColor;
        let neutral =
            |c: TagColor| matches!(c, TagColor::Default | TagColor::Grey | TagColor::Black);
        for (name, dark, c) in [("dark", true, &DARK), ("light", false, &LIGHT)] {
            for (ix, a) in TagColor::ALL.iter().enumerate() {
                let shown = muted_tag_color(a.rgb(), dark);
                for bg in [c.sidebar, c.list, c.editor, c.elevated, c.fill] {
                    let ratio = contrast_ratio(shown, bg.0);
                    assert!(ratio >= 3.0, "{name}: {a:?} on {bg:?} is {ratio:.2}");
                }
                for b in &TagColor::ALL[ix + 1..] {
                    let min = if neutral(*a) && neutral(*b) {
                        5.0
                    } else {
                        10.0
                    };
                    let diff = delta_e(shown, muted_tag_color(b.rgb(), dark));
                    assert!(diff >= min, "{name}: {a:?} and {b:?} are {diff:.1} apart");
                }
            }
        }
        // The pair that used to come out the same.
        let (orange, brown) = (TagColor::Orange.rgb(), TagColor::Brown.rgb());
        assert!(delta_e(muted_tag_color(orange, true), muted_tag_color(brown, true)) > 10.);
    }

    #[test]
    fn contrast_examples() {
        assert_eq!(contrast_ratio(0, 0xFF_FFFF), 21.0);
        assert!((contrast_ratio(0x767676, 0xFF_FFFF) - 4.54).abs() < 0.01);
    }

    /// Every text color reads on every surface it can sit on, in both
    /// themes: primary text at 7:1, secondary at 4.5:1, muted at 3:1, links
    /// at 4.5:1. Muted text and links don't sit on the brief hover fill of a
    /// menu item, and links aren't drawn on sidebar rows. Status colors,
    /// drawn as icons and dots, read at 3:1 on every surface.
    #[test]
    fn text_contrast_on_every_surface() {
        for (name, c) in [("dark", &DARK), ("light", &LIGHT)] {
            let surfaces = [
                ("app", c.app, true),
                ("sidebar", c.sidebar, false),
                ("list", c.list, true),
                ("editor", c.editor, true),
                ("elevated", c.elevated, true),
                ("fill", c.fill, true),
                ("row_active", c.row_active, true),
                ("row_hover", c.row_hover, true),
                ("sidebar_active", c.sidebar_active, false),
                ("sidebar_hover", c.sidebar_hover, false),
                ("code", c.code, true),
            ];
            let all_surfaces = surfaces
                .iter()
                .map(|&(surface, bg, _)| (surface, bg))
                .chain([("hover", c.hover), ("selected", c.selected)]);
            for (surface, bg) in all_surfaces {
                for (status, fg) in [
                    ("success", c.success),
                    ("warning", c.warning),
                    ("danger", c.danger),
                ] {
                    let ratio = contrast_ratio(fg.0, bg.0);
                    assert!(
                        ratio >= 3.0,
                        "{name}: {status} on {surface} is {ratio:.2}, wants 3"
                    );
                }
            }
            // A chip or banner shows on the panes it sits on.
            for (surface, bg) in [("list", c.list), ("editor", c.editor)] {
                let ratio = contrast_ratio(c.fill.0, bg.0);
                assert!(ratio >= 1.1, "{name}: fill on {surface} is {ratio:.2}");
            }
            for (surface, bg, links) in surfaces {
                for (text, fg, min) in [
                    ("text", c.text, 7.0),
                    ("text_secondary", c.text_secondary, 4.5),
                    ("text_muted", c.text_muted, 3.0),
                    ("link", if links { c.link } else { c.text }, 4.5),
                ] {
                    let ratio = contrast_ratio(fg.0, bg.0);
                    assert!(
                        ratio >= min,
                        "{name}: {text} on {surface} is {ratio:.2}, wants {min}"
                    );
                }
            }
            for fg in [c.text, c.text_secondary] {
                assert!(
                    contrast_ratio(fg.0, c.hover.0) >= 4.5,
                    "{name}: {fg:?} on hover"
                );
            }
            // Code text sits on the code background and the editor.
            for bg in [c.code, c.editor] {
                for fg in [c.code_text, c.syntax_keyword, c.syntax_string] {
                    assert!(
                        contrast_ratio(fg.0, bg.0) >= 4.5,
                        "{name}: {fg:?} on {bg:?}"
                    );
                }
            }
            assert!(
                contrast_ratio(c.button_text.0, c.button.0) >= 4.5,
                "{name}: button"
            );
            // A switch's thumb shows on its track, off and on, and an off
            // track shows on the panes (it is the muted text color, checked
            // on every surface above).
            let thumb = if name == "dark" { c.text } else { c.elevated };
            for track in [c.text_muted, c.button] {
                assert!(
                    contrast_ratio(thumb.0, track.0) >= 3.0,
                    "{name}: switch thumb on {track:?}"
                );
            }
            // Text on a status color, as in a destructive button.
            let on_status = if name == "dark" { c.app } else { c.button_text };
            for fill in [c.success, c.warning, c.danger] {
                assert!(
                    contrast_ratio(on_status.0, fill.0) >= 4.5,
                    "{name}: text on {fill:?}"
                );
            }
            // Selected rows are a subtle fill, but one that shows against
            // their pane. A chip on a selected row takes the pane's color,
            // so it shows by the same step: in the dark theme no fill can be
            // a step off the selected row and still carry muted text at 3:1.
            assert!(
                contrast_ratio(c.row_active.0, c.list.0) >= 1.08,
                "{name}: row"
            );
            assert!(
                contrast_ratio(c.sidebar_active.0, c.sidebar.0) >= 1.08,
                "{name}: sidebar row"
            );
            // A selected ghost button shows on the editor, more than one
            // under the pointer does (the library hovers a ghost button with
            // the accent fill, at half strength in dark mode).
            let ghost_hover = if name == "dark" {
                mix(c.hover, c.editor, 0.5)
            } else {
                c.hover
            };
            let selected = contrast_ratio(c.selected.0, c.editor.0);
            assert!(selected >= 1.2, "{name}: selected is {selected:.2}");
            for hover in [
                ghost_hover,
                if name == "dark" { c.elevated } else { c.hover },
            ] {
                assert!(
                    selected > contrast_ratio(hover.0, c.editor.0),
                    "{name}: selected vs {hover:?}"
                );
            }
            assert!(contrast_ratio(c.text.0, c.selected.0) >= 7.0, "{name}");
            assert!(
                contrast_ratio(c.text_secondary.0, c.selected.0) >= 4.5,
                "{name}"
            );
            // The palette's selected row: the accent over the elevated
            // panel. Its label is primary text; its icon, hint and keys
            // turn secondary on it, and it shows against the panel.
            let alpha = if name == "dark" {
                PALETTE_SELECTED_ALPHA.0
            } else {
                PALETTE_SELECTED_ALPHA.1
            };
            let palette_selected = mix(c.accent, c.elevated, f64::from(alpha));
            for (text, fg, min) in [
                ("text", c.text, 7.0),
                ("text_secondary", c.text_secondary, 4.5),
            ] {
                let ratio = contrast_ratio(fg.0, palette_selected.0);
                assert!(
                    ratio >= min,
                    "{name}: {text} on the palette's selected row is {ratio:.2}, wants {min}"
                );
            }
            assert!(
                contrast_ratio(palette_selected.0, c.elevated.0) >= 1.2,
                "{name}: palette selected row"
            );
        }
    }

    /// Times, counts and hints are small (11.5 to 13 px), so on a selected
    /// row, note or sidebar or palette, they take the 4.5:1 that small
    /// text needs; muted text falls short of it on every one of those
    /// fills, which is why they don't stay muted there.
    #[test]
    fn quiet_text_reads_on_selected_rows() {
        for (name, c, alpha) in [
            ("dark", &DARK, PALETTE_SELECTED_ALPHA.0),
            ("light", &LIGHT, PALETTE_SELECTED_ALPHA.1),
        ] {
            assert_eq!(c.quiet(false), c.text_muted, "{name}");
            let palette_selected = mix(c.accent, c.elevated, f64::from(alpha));
            for (fill, bg) in [
                ("row_active", c.row_active),
                ("sidebar_active", c.sidebar_active),
                ("palette", palette_selected),
            ] {
                let ratio = contrast_ratio(c.quiet(true).0, bg.0);
                assert!(ratio >= 4.5, "{name}: quiet text on {fill} is {ratio:.2}");
                assert!(
                    contrast_ratio(c.text_muted.0, bg.0) < 4.5,
                    "{name}: muted text would do on {fill}"
                );
            }
        }
    }

    /// `top` over `bottom` at `alpha`.
    fn mix(top: Rgb, bottom: Rgb, alpha: f64) -> Rgb {
        let channel = |shift: u32| {
            let (t, b) = ((top.0 >> shift) & 0xFF, (bottom.0 >> shift) & 0xFF);
            (f64::from(t) * alpha + f64::from(b) * (1. - alpha)).round() as u32
        };
        Rgb((channel(16) << 16) | (channel(8) << 8) | channel(0))
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

    /// The spec's own light muted text and accent fall short as text, and
    /// its (dark) status colors fall short on paper, which is why [`LIGHT`]
    /// shades them. Its elevated color is the lightest light surface, so it
    /// can't be a fill there.
    #[test]
    fn spec_colors_that_were_shaded() {
        let spec_muted = 0x948B7D;
        assert!(contrast_ratio(spec_muted, LIGHT.sidebar.0) < 3.0);
        let spec_accent = 0x5E7895;
        assert!(contrast_ratio(spec_accent, LIGHT.sidebar.0) < 4.5);
        assert!(contrast_ratio(DARK.accent.0, DARK.elevated.0) < 4.5);
        for spec_status in [DARK.success, DARK.warning, DARK.danger] {
            assert!(contrast_ratio(spec_status.0, LIGHT.sidebar.0) < 3.0);
        }
        assert!(contrast_ratio(LIGHT.elevated.0, LIGHT.list.0) < 1.1);
    }

    #[test]
    fn tag_examples() {
        use kanso_core::TagColor;
        // Orange stays orange, and black becomes a readable grey.
        let (hue, saturation, _) = to_hsl(muted_tag_color(TagColor::Orange.rgb(), true));
        assert!(
            (20.0..30.0).contains(&hue) && saturation <= TAG_SATURATION_MAX + 0.01,
            "{hue} {saturation}"
        );
        // A grey keeps no tint, and a weak color isn't calmed further.
        assert_eq!(calm(0., 0.5, true).0, 0.);
        assert_eq!(calm(0.1, 0.5, true).0, 0.1);
        let black = muted_tag_color(TagColor::Black.rgb(), true);
        assert!(contrast_ratio(black, DARK.sidebar.0) >= 3.0);
        assert_eq!(from_hsl(0., 0., 1.), 0xFF_FFFF);
        assert_eq!(from_hsl(120., 1., 0.5), 0x00_FF00);
    }

    /// SF Mono loads under its system name, and the check tells a
    /// monospaced face from a proportional one or a missing family.
    #[test]
    fn system_mono_font_loads() {
        let text_system = TextSystem::new(std::sync::Arc::new(gpui_apple::AppleTextSystem::new()));
        assert!(loads_monospaced(&text_system, SYSTEM_MONO));
        assert!(loads_monospaced(&text_system, "Menlo"));
        assert!(!loads_monospaced(&text_system, "Helvetica"));
        assert!(!loads_monospaced(&text_system, "No Such Family Kanso"));
    }

    /// The editor colors and syntax styles Kanso drew before themes were
    /// files, kept to check that its own theme files draw the same.
    fn legacy_highlight(c: &Colors) -> Value {
        let color = |rgb: Rgb| json!({ "color": rgb.hex() });
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
                "keyword": color(c.syntax_keyword),
                "tag": color(c.syntax_keyword),
                "constructor": color(c.syntax_keyword),
        });
        let rest = json!({
                "string": color(c.syntax_string),
                "string.special": color(c.syntax_string),
                "number": color(c.code_text),
                "boolean": color(c.code_text),
                "constant": color(c.code_text),
                "variable.special": color(c.code_text),
                "type": secondary,
                "attribute": secondary,
                "label": secondary,
                "operator": secondary,
                "punctuation": secondary,
                "punctuation.bracket": secondary,
                "function": color(c.text),
                "property": color(c.text),
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
            assert_eq!(&theme.colors, expected, "{}", theme.name);
            assert_eq!(theme.caret, expected.accent);
            assert_eq!(theme.selection.rgb, expected.accent);
            assert_eq!(theme.selection.alpha, 0x4D);
            // Fitting to its own surface changes nothing.
            let fitted =
                crate::theme_file::fit_syntax(&theme.syntax, expected.editor, expected.text);
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
            let preview = crate::theme_file::fit_preview(&theme.preview, expected);
            assert_eq!(preview, theme.preview);
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
