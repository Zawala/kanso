// Kanso — note_screen_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a note screen has to show.
enum NoteScreenStatus {
  /// The note is being read.
  loading,

  /// Its text is read and shown.
  ready,

  /// It is locked and closed: only its title and its lock are known.
  locked,

  /// There is no such note (any more).
  notFound,

  /// Reading it failed.
  failed,
}
