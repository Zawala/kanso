// Kanso — note_list_entry.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';
import 'bucket.dart';

part 'note_list_entry.freezed.dart';

/// One line of the note list: a header over a group of notes, or a note.
@freezed
sealed class NoteListEntry with _$NoteListEntry {
  /// A header: what the notes under it have in common.
  const factory NoteListEntry.header(Bucket bucket) = HeaderEntry;

  /// A note.
  const factory NoteListEntry.note(NoteRow row) = NoteEntry;
}
