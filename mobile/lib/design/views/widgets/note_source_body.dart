// Kanso — note_source_body.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_caret.dart';
import '../../models/data/mock_block.dart';
import 'code_block.dart';
import 'markdown_inline_text.dart';

/// A note's blocks as Markdown source, as the editor shows them.
///
/// Prose is in Instrument Sans with the marks in the muted `marker` colour,
/// headings bold and code spans in the `code` colour. With [typing] a last
/// paragraph is drawn with the caret after it.
class NoteSourceBody extends StatelessWidget {
  const NoteSourceBody({required this.blocks, this.typing, super.key});

  final List<MockBlock> blocks;

  /// The text being typed after the last block.
  final String? typing;

  @override
  Widget build(BuildContext context) {
    final syntax = context.syntaxColors;
    final typography = context.typography;
    final body = typography.body;
    TextStyle marked(Color color) => body.copyWith(color: color);
    final typed = typing;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        for (final block in blocks)
          Padding(
            padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
            child: switch (block) {
              MockHeadingBlock(:final level, :final text) => Text.rich(
                TextSpan(
                  children: <InlineSpan>[
                    TextSpan(
                      text: '${'#' * level} ',
                      style: TextStyle(
                        color: syntax.blockMarker,
                        fontWeight: FontWeight.w400,
                      ),
                    ),
                    TextSpan(text: text),
                  ],
                  style:
                      (level == 1 ? typography.heading1 : typography.heading2)
                          .copyWith(color: syntax.heading),
                ),
              ),
              MockParagraphBlock(:final text) => MarkdownInlineText(
                text,
                style: body,
                showMarks: true,
              ),
              MockBulletsBlock(:final items) => Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  for (final item in items)
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: <Widget>[
                        Text('- ', style: marked(syntax.listMarker)),
                        Expanded(
                          child: MarkdownInlineText(
                            item,
                            style: body,
                            showMarks: true,
                          ),
                        ),
                      ],
                    ),
                ],
              ),
              MockTasksBlock(:final items) => Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  for (final item in items)
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: <Widget>[
                        Text('- ', style: marked(syntax.listMarker)),
                        Text(
                          item.done ? '[x] ' : '[ ] ',
                          style: marked(syntax.task),
                        ),
                        Expanded(
                          child: MarkdownInlineText(
                            item.text,
                            style: body,
                            showMarks: true,
                          ),
                        ),
                      ],
                    ),
                ],
              ),
              MockQuoteBlock(:final text) => Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text('> ', style: marked(syntax.blockMarker)),
                  Expanded(
                    child: MarkdownInlineText(
                      text,
                      style: body,
                      showMarks: true,
                    ),
                  ),
                ],
              ),
              MockCodeBlock(:final lines, :final language) => CodeBlock(
                lines: lines,
                language: language,
                fence: true,
              ),
            },
          ),
        if (typed != null)
          MarkdownInlineText(
            typed,
            style: body,
            showMarks: true,
            trailing: const WidgetSpan(
              alignment: PlaceholderAlignment.middle,
              child: KansoCaret(height: 19),
            ),
          ),
      ],
    );
  }
}
