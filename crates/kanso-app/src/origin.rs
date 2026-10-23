//! Where a theme came from: compiled in, the person's own, or the gallery.
//!
//! The distinction is shown as a badge and decides whether the gallery may
//! update a theme. A person's own files and built-ins are never touched by
//! the gallery; only a theme installed under `<data>/gallery/` is.

/// A theme's source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    /// Compiled in from `themes/`, reviewed with the release.
    #[default]
    Builtin,
    /// A file in the person's `themes/` or `icon-themes/`, imported or
    /// dropped there.
    Own,
    /// Installed by the gallery under `<data>/gallery/`.
    Gallery,
}

impl Origin {
    /// The detail line's first words.
    pub fn detail(self) -> &'static str {
        match self {
            Origin::Builtin => "Built in",
            Origin::Own => "Yours",
            Origin::Gallery => "From the gallery",
        }
    }
}
