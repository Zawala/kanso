// Kanso — status_label.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/note_status.dart';
import '../../gen/l10n/app_localizations.dart';

/// The words for a note's status.
extension NoteStatusLabel on AppLocalizations {
  /// The name of [status], or an empty string when it has none.
  String statusName(NoteStatus status) {
    return switch (status) {
      NoteStatus.none => '',
      NoteStatus.active => statusActive,
      NoteStatus.onHold => statusOnHold,
      NoteStatus.completed => statusCompleted,
      NoteStatus.dropped => statusDropped,
    };
  }
}
