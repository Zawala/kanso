// Kanso — note_preview_body.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/mock_block.dart';
import 'code_block.dart';
import 'markdown_inline_text.dart';
import 'preview_bullet_list.dart';
import 'preview_quote.dart';
import 'preview_task_list.dart';

/// A note's blocks, rendered: the look Kanso's preview is meant to have.
///
/// This is drawn with Flutter widgets for the mock; the real preview shows
/// Kanso's HTML in a web view.
class NotePreviewBody extends StatelessWidget {
  const NotePreviewBody({required this.blocks, super.key});

  final List<MockBlock> blocks;

  @override
  Widget build(BuildContext context) {
    final typography = context.typography;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        for (final block in blocks)
          Padding(
            padding: EdgeInsets.only(
              top: switch (block) {
                MockHeadingBlock() => KansoSpacing.xs,
                _ => 0,
              },
              bottom: switch (block) {
                MockHeadingBlock() => KansoSpacing.xs,
                _ => KansoSpacing.sm,
              },
            ),
            child: switch (block) {
              MockHeadingBlock(:final level, :final text) => Text(
                text,
                style: level == 1 ? typography.heading1 : typography.heading2,
              ),
              MockParagraphBlock(:final text) => MarkdownInlineText(
                text,
                style: typography.body,
              ),
              MockBulletsBlock(:final items) => PreviewBulletList(items: items),
              MockTasksBlock(:final items) => PreviewTaskList(items: items),
              MockQuoteBlock(:final text) => PreviewQuote(text: text),
              MockCodeBlock(:final lines, :final language) => CodeBlock(
                lines: lines,
                language: language,
              ),
            },
          ),
      ],
    );
  }
}
