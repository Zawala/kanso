// Kanso — mock_task_item.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'mock_task_item.freezed.dart';

/// One line of a task list.
@freezed
abstract class MockTaskItem with _$MockTaskItem {
  const factory MockTaskItem({
    required String text,
    @Default(false) bool done,
  }) = _MockTaskItem;
}
