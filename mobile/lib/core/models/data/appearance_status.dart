// Kanso — appearance_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the saved appearance has been read.
enum AppearanceStatus {
  /// Not read yet: Kanso Light and Dark are shown.
  loading,

  /// The saved choices are in use.
  ready,

  /// The preferences or a chosen theme could not be read, so Kanso's own
  /// themes and the default fonts are shown; the app works, and the next
  /// choice is saved.
  failed,
}
