// Kanso — mock_note_group.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'mock_note.dart';

part 'mock_note_group.freezed.dart';

/// The notes of one date group of a list: "Today", "Yesterday".
@freezed
abstract class MockNoteGroup with _$MockNoteGroup {
  const factory MockNoteGroup({
    required String label,
    required List<MockNote> notes,
  }) = _MockNoteGroup;
}
