// Kanso — markdown_inline_text.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../utils/parse_inline_markdown.dart';

/// A run of text with Markdown marks, drawn as a note shows it.
///
/// Rendered (the default) the marks and link addresses are hidden and bold,
/// italic, code and links are styled. With [showMarks], as in the editor,
/// the marks stay, in the muted `marker` colour, and the text they wrap is
/// styled the same way.
class MarkdownInlineText extends StatelessWidget {
  const MarkdownInlineText(
    this.text, {
    required this.style,
    this.showMarks = false,
    this.trailing,
    super.key,
  });

  final String text;
  final TextStyle style;
  final bool showMarks;

  /// Drawn right after the text, such as a caret.
  final InlineSpan? trailing;

  @override
  Widget build(BuildContext context) {
    final syntax = context.syntaxColors;
    final spans = <InlineSpan>[];
    for (final run in parseInlineMarkdown(text)) {
      final hidden =
          run.kind == InlineKind.marker || run.kind == InlineKind.url;
      if (hidden && !showMarks) {
        continue;
      }
      spans.add(
        TextSpan(
          text: run.text,
          style: switch (run.kind) {
            InlineKind.plain => null,
            InlineKind.bold => const TextStyle(fontWeight: FontWeight.w700),
            InlineKind.italic => const TextStyle(fontStyle: FontStyle.italic),
            InlineKind.code => context.typography.mono.copyWith(
              fontSize: style.fontSize == null ? null : style.fontSize! - 1,
              color: syntax.code,
              backgroundColor: showMarks ? null : syntax.codeBackground,
            ),
            InlineKind.link => TextStyle(
              color: syntax.link,
              decoration: showMarks ? null : TextDecoration.underline,
              decorationColor: syntax.link,
            ),
            InlineKind.url => TextStyle(color: syntax.url),
            InlineKind.wiki => TextStyle(
              color: syntax.link,
              fontWeight: FontWeight.w500,
            ),
            InlineKind.marker => TextStyle(color: syntax.marker),
          },
        ),
      );
    }
    final end = trailing;
    if (end != null) {
      spans.add(end);
    }
    return Text.rich(TextSpan(style: style, children: spans));
  }
}
