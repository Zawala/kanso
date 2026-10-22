//! A note in a window's back-and-forward history, and where the reader was
//! in it (`nav_history` keeps the history; the editor reads and restores
//! the place).

/// Where the reader was in a note: the caret, and how far the text and the
/// preview were scrolled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NotePlace {
    /// The caret's line, 0-based.
    pub caret_line: u32,
    /// The caret's column on its line, as the editor counts it.
    pub caret_column: u32,
    /// The text's scroll offset, in pixels (negative when scrolled).
    pub scroll_x: f32,
    pub scroll_y: f32,
    /// How far the preview is scrolled down, in pixels.
    pub preview_top: f32,
}

/// A note in the history and, once the reader left it, where they were.
#[derive(Clone, Debug, PartialEq)]
pub struct Visit {
    /// The note's ID (`note:…`).
    pub id: String,
    pub place: Option<NotePlace>,
}

impl Visit {
    pub fn new(id: impl Into<String>) -> Self {
        Visit {
            id: id.into(),
            place: None,
        }
    }
}
