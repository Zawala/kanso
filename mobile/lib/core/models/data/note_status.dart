// Kanso — note_status.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a note stands, as the person set it.
enum NoteStatus {
  /// No status: nothing is shown.
  none,

  /// In progress.
  active,

  /// Paused.
  onHold,

  /// Done.
  completed,

  /// Given up.
  dropped,
}
