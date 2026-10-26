// Kanso — design_screen.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The screens of the design proposal that are not built yet, in the order
/// the gallery lists them.
enum DesignScreen {
  /// A note, as Markdown source, with the keyboard up.
  noteEdit,

  /// Settings.
  settings,

  /// Settings, scrolled to their end.
  settingsScrolled,

  /// Choosing the fonts of notes and their text size.
  fonts,

  /// Choosing fonts, scrolled to the end.
  fontsScrolled,

  /// Choosing a theme.
  themePicker,

  /// Choosing a theme, scrolled to the end.
  themePickerScrolled,

  /// The signed-out error card.
  error,
}
