// Kanso — mock_notebook.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'mock_notebook.freezed.dart';

/// A notebook in the library, [depth] levels inside its parent.
@freezed
abstract class MockNotebook with _$MockNotebook {
  const factory MockNotebook({
    required String name,
    required int count,
    @Default(0) int depth,
  }) = _MockNotebook;
}
