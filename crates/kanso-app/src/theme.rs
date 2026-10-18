//! Kanso's two themes: warm graphite (dark, the primary one) and soft paper
//! (light), from the design spec's palette.
//!
//! The component library gets them as theme files, so its buttons, menus and
//! inputs match. The views read [`Colors`] directly for what the library
//! can't express: three pane surfaces, secondary text, and selected rows
//! (the library caps a list's selected background at 20% opacity).

use std::sync::OnceLock;

use gpui_kit::component::{Theme, ThemeRegistry};
use gpui_kit::{App, Fill, Hsla};
use serde_json::{Value, json};

pub const LIGHT_NAME: &str = "Kanso Light";
pub const DARK_NAME: &str = "Kanso Dark";

/// An sRGB color, `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u32);

impl Rgb {
    /// `#RRGGBB`.
    pub fn hex(self) -> String {
        format!("#{:06X}", self.0 & 0xFF_FFFF)
    }

    /// `#RRGGBBAA`, with `alpha` out of 255.
    fn hex_alpha(self, alpha: u8) -> String {
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
    /// Menus, popovers and muted fills.
    pub elevated: Rgb,
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
    active_line: Rgb(0xF6F3EB),
    button: Rgb(0x4D6684),
    button_text: Rgb(0xFFFDF8),
    code: Rgb(0xF4F1EA),
    code_text: Rgb(0x7D5B32),
    syntax_keyword: Rgb(0x4D6684),
    syntax_string: Rgb(0x55704A),
    success: Rgb(0x7BAA78),
    warning: Rgb(0xC49A5A),
    danger: Rgb(0xC96E63),
};

impl Colors {
    /// The colors of the theme in use.
    pub fn get(cx: &App) -> &'static Colors {
        if Theme::global(cx).is_dark() {
            &DARK
        } else {
            &LIGHT
        }
    }
}

/// Registers Kanso's themes and makes them the ones light and dark mode use.
/// Does nothing once they are in place.
pub fn ensure_installed(cx: &mut App) {
    let theme = Theme::global(cx);
    if theme.light_theme.name == LIGHT_NAME && theme.dark_theme.name == DARK_NAME {
        return;
    }
    let set = theme_set(mono_font(cx));
    let registry = ThemeRegistry::global_mut(cx);
    if let Err(err) = registry.load_themes_from_str(&set.to_string()) {
        eprintln!("kanso: couldn't load the theme: {err:#}");
        return;
    }
    let themes = registry.themes();
    let (Some(light), Some(dark)) = (
        themes.get(LIGHT_NAME).cloned(),
        themes.get(DARK_NAME).cloned(),
    ) else {
        return;
    };
    let theme = Theme::global_mut(cx);
    theme.light_theme = light;
    theme.dark_theme = dark;
}

/// SF Mono when the system has it. Laying out text in a missing family
/// panics, so it is looked for rather than assumed; without it the library
/// keeps its own installed default.
fn mono_font(cx: &App) -> Option<&'static str> {
    static FOUND: OnceLock<bool> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            cx.text_system()
                .all_font_names()
                .iter()
                .any(|name| name == "SF Mono")
        })
        .then_some("SF Mono")
}

// no-prop-test: a fixed document; `themes_parse` checks it loads as written.
fn theme_set(mono: Option<&str>) -> Value {
    json!({
        "name": "Kanso",
        "themes": [
            theme_config(LIGHT_NAME, false, &LIGHT, mono),
            theme_config(DARK_NAME, true, &DARK, mono),
        ],
    })
}

fn theme_config(name: &str, dark: bool, c: &Colors, mono: Option<&str>) -> Value {
    let mut config = json!({
        "name": name,
        "mode": if dark { "dark" } else { "light" },
        "radius": 6,
        "radius.lg": 10,
        "shadow": false,
        "colors": theme_colors(dark, c),
        "highlight": highlight(c),
    });
    if let Some(mono) = mono {
        config["mono_font.family"] = json!(mono);
    }
    config
}

/// The component library's color keys.
fn theme_colors(dark: bool, c: &Colors) -> Value {
    // Text on the status colors.
    let on_status = if dark { c.app } else { c.text };
    let colors: [(&str, String); 57] = [
        ("background", c.editor.hex()),
        ("foreground", c.text.hex()),
        ("border", c.divider.hex()),
        ("input.border", c.divider.hex()),
        ("accent.background", c.hover.hex()),
        ("accent.foreground", c.text.hex()),
        ("secondary.background", c.hover.hex()),
        ("secondary.foreground", c.text.hex()),
        ("muted.background", c.elevated.hex()),
        ("muted.foreground", c.text_muted.hex()),
        ("popover.background", c.elevated.hex()),
        ("popover.foreground", c.text.hex()),
        ("primary.background", c.button.hex()),
        ("primary.foreground", c.button_text.hex()),
        ("ring", c.accent.hex()),
        ("caret", c.accent.hex()),
        ("selection.background", c.accent.hex_alpha(0x4D)),
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
        (
            "overlay",
            (if dark { "#00000059" } else { "#26241F26" }).into(),
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

/// Editor colors and syntax styles. Markdown source stays calm: headings are
/// bold in the text color, markers recede, and code is legible rather than
/// colorful.
fn highlight(c: &Colors) -> Value {
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
    });
    // Code in fenced blocks.
    let code = json!({
        "comment": { "color": c.text_muted.hex(), "font_style": "italic" },
        "comment.doc": { "color": c.text_muted.hex(), "font_style": "italic" },
        "keyword": color(c.syntax_keyword),
        "tag": color(c.syntax_keyword),
        "constructor": color(c.syntax_keyword),
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
    if let (Value::Object(syntax), Value::Object(code)) = (&mut syntax, code) {
        syntax.extend(code);
    }
    json!({
        "editor.background": c.editor.hex(),
        "editor.foreground": c.text.hex(),
        "editor.active_line.background": c.active_line.hex(),
        "editor.line_number": c.text_muted.hex(),
        "editor.active_line_number": c.text_secondary.hex(),
        "editor.invisible": c.text_muted.hex_alpha(0x66),
        "error": c.danger.hex(),
        "warning": c.warning.hex(),
        "info": c.accent.hex(),
        "success": c.success.hex(),
        "hint": c.text_muted.hex(),
        "syntax": syntax,
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
const TAG_SATURATION_MAX: f32 = 0.38;
/// The lightness band tag colors are kept in, on dark and light surfaces.
const TAG_LIGHTNESS_DARK: (f32, f32) = (0.58, 0.72);
const TAG_LIGHTNESS_LIGHT: (f32, f32) = (0.36, 0.50);

/// A tag's color calmed for display: the same hue, no more saturated than
/// it was or than [`TAG_SATURATION_MAX`], and light or dark enough to read on
/// the theme's surfaces. The stored color doesn't change.
pub fn muted_tag_color(rgb: u32, dark: bool) -> u32 {
    let (hue, saturation, lightness) = to_hsl(rgb);
    let (low, high) = if dark {
        TAG_LIGHTNESS_DARK
    } else {
        TAG_LIGHTNESS_LIGHT
    };
    from_hsl(
        hue,
        saturation.min(TAG_SATURATION_MAX),
        lightness.clamp(low, high),
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

        /// Idempotence: calming a calmed color changes it by at most one
        /// rounding step per channel.
        #[test]
        fn prop_muted_tag_color_is_idempotent(color in rgb(), dark in any::<bool>()) {
            let once = muted_tag_color(color, dark);
            let twice = muted_tag_color(once, dark);
            for shift in [16, 8, 0] {
                let (a, b) = ((once >> shift) & 0xFF, (twice >> shift) & 0xFF);
                prop_assert!(a.abs_diff(b) <= 1, "{once:06x} -> {twice:06x}");
            }
        }
    }

    #[test]
    fn contrast_examples() {
        assert_eq!(contrast_ratio(0, 0xFF_FFFF), 21.0);
        assert!((contrast_ratio(0x767676, 0xFF_FFFF) - 4.54).abs() < 0.01);
    }

    /// Every text color reads on every surface it can sit on, in both
    /// themes: primary text at 7:1, secondary at 4.5:1, muted at 3:1, links
    /// at 4.5:1. Muted text and links don't sit on the brief hover fill of a
    /// menu item, and links aren't drawn on sidebar rows.
    #[test]
    fn text_contrast_on_every_surface() {
        for (name, c) in [("dark", &DARK), ("light", &LIGHT)] {
            let surfaces = [
                ("app", c.app, true),
                ("sidebar", c.sidebar, false),
                ("list", c.list, true),
                ("editor", c.editor, true),
                ("elevated", c.elevated, true),
                ("row_active", c.row_active, true),
                ("row_hover", c.row_hover, true),
                ("sidebar_active", c.sidebar_active, false),
                ("sidebar_hover", c.sidebar_hover, false),
                ("code", c.code, true),
            ];
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
            // Selected rows are a subtle fill, but one that shows against
            // their pane.
            assert!(
                contrast_ratio(c.row_active.0, c.list.0) >= 1.08,
                "{name}: row"
            );
            assert!(
                contrast_ratio(c.sidebar_active.0, c.sidebar.0) >= 1.08,
                "{name}: sidebar row"
            );
        }
    }

    /// The spec's own light muted text and accent fall short as text, which
    /// is why [`LIGHT`] shades them.
    #[test]
    fn spec_colors_that_were_shaded() {
        let spec_muted = 0x948B7D;
        assert!(contrast_ratio(spec_muted, LIGHT.sidebar.0) < 3.0);
        let spec_accent = 0x5E7895;
        assert!(contrast_ratio(spec_accent, LIGHT.sidebar.0) < 4.5);
        assert!(contrast_ratio(DARK.accent.0, DARK.elevated.0) < 4.5);
    }

    #[test]
    fn tag_examples() {
        use kanso_core::TagColor;
        // Orange stays orange, and black becomes a readable grey.
        let (hue, saturation, _) = to_hsl(muted_tag_color(TagColor::Orange.rgb(), true));
        assert!(
            (20.0..30.0).contains(&hue) && saturation <= 0.39,
            "{hue} {saturation}"
        );
        let black = muted_tag_color(TagColor::Black.rgb(), true);
        assert!(contrast_ratio(black, DARK.sidebar.0) >= 3.0);
        assert_eq!(from_hsl(0., 0., 1.), 0xFF_FFFF);
        assert_eq!(from_hsl(120., 1., 0.5), 0x00_FF00);
    }

    /// The theme files load with the colors as written.
    #[test]
    fn themes_parse() {
        let set: ThemeSet = serde_json::from_value(theme_set(Some("SF Mono"))).unwrap();
        assert_eq!(set.themes.len(), 2);
        let dark = &set.themes[1];
        assert_eq!(dark.name.as_ref(), DARK_NAME);
        assert!(dark.mode.is_dark());
        assert_eq!(dark.mono_font_family.as_deref(), Some("SF Mono"));
        assert_eq!(dark.colors.background.as_deref(), Some("#282824"));
        assert_eq!(dark.colors.sidebar.as_deref(), Some("#1D1D1A"));
        let highlight = dark.highlight.as_ref().expect("highlight");
        assert_eq!(highlight.editor_background, Some(Hsla::from(DARK.editor)));
        assert!(highlight.syntax.title.is_some());
        assert!(!set.themes[0].mode.is_dark());
    }
}
