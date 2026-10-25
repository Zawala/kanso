// Kanso — mock_block.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'mock_code_token.dart';
import 'mock_task_item.dart';

part 'mock_block.freezed.dart';

/// One block of a note's Markdown. Inline marks (`**bold**`, `*italic*`,
/// `` `code` ``, `[text](url)`, `[[wiki link]]`) stay in the text and are
/// read by `parseInlineMarkdown`.
@freezed
sealed class MockBlock with _$MockBlock {
  const factory MockBlock.heading({
    required int level,
    required String text,
  }) = MockHeadingBlock;

  const factory MockBlock.paragraph(String text) = MockParagraphBlock;

  const factory MockBlock.bullets(List<String> items) = MockBulletsBlock;

  const factory MockBlock.tasks(List<MockTaskItem> items) = MockTasksBlock;

  const factory MockBlock.quote(String text) = MockQuoteBlock;

  const factory MockBlock.code({
    required String language,
    required List<List<MockCodeToken>> lines,
  }) = MockCodeBlock;
}
