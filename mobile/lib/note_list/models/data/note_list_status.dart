// Kanso — note_list_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the notes of a list have been read.
enum NoteListStatus {
  /// Being read for the first time.
  loading,

  /// Read; kept up to date as the library changes.
  ready,

  /// The last reading failed.
  failed,
}
