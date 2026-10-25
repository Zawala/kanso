// Kanso — mock_completion.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'mock_completion.freezed.dart';

/// A note offered while a wiki link is typed.
@freezed
abstract class MockCompletion with _$MockCompletion {
  const factory MockCompletion({
    required String title,
    required String notebook,
  }) = _MockCompletion;
}
