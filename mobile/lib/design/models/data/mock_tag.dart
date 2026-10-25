// Kanso — mock_tag.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/tag_color.dart';

part 'mock_tag.freezed.dart';

/// A tag, with its colour and the number of notes that carry it.
@freezed
abstract class MockTag with _$MockTag {
  const factory MockTag({
    required String name,
    required TagColor color,
    @Default(0) int count,
  }) = _MockTag;
}
