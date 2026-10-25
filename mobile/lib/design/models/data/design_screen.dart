// Kanso — design_screen.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The screens of the design proposal, in the order the gallery lists them.
enum DesignScreen {
  /// The library, as a drawer on a phone.
  library(hasTablet: false),

  /// The library, scrolled to its end.
  libraryScrolled(hasTablet: false),

  /// The list of notes.
  noteList(hasTablet: false),

  /// The list of notes while it loads.
  loadingList(),

  /// A notebook with no notes.
  emptyNotebook(),

  /// A note, rendered.
  notePreview(),

  /// A note, as Markdown source, with the keyboard up.
  noteEdit(),

  /// Search results.
  search(),

  /// Settings.
  settings(),

  /// Settings, scrolled to their end.
  settingsScrolled(),

  /// Choosing the fonts of notes and their text size.
  fonts(),

  /// Choosing fonts, scrolled to the end.
  fontsScrolled(),

  /// Choosing a theme.
  themePicker(),

  /// Choosing a theme, scrolled to the end.
  themePickerScrolled(),

  /// A locked note.
  lockedNote(),

  /// The signed-out error card.
  error();

  const DesignScreen({this.hasTablet = true});

  /// Whether the proposal has this screen for a tablet. The library and the
  /// list are panes of every tablet screen, so they have no screen of their
  /// own there.
  final bool hasTablet;
}
