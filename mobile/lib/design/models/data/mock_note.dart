// Kanso — mock_note.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/note_status.dart';
import 'mock_tag.dart';

part 'mock_note.freezed.dart';

/// A note as a list shows it.
@freezed
abstract class MockNote with _$MockNote {
  const factory MockNote({
    required String title,
    required String excerpt,
    required String notebook,
    required String time,
    @Default(NoteStatus.none) NoteStatus status,
    @Default(<MockTag>[]) List<MockTag> tags,
    @Default(false) bool pinned,
    @Default(false) bool locked,
  }) = _MockNote;
}
