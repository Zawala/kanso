// Kanso — preview_quote.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import 'markdown_inline_text.dart';

/// A block quote, rendered: a rule on the left and quieter, italic text.
class PreviewQuote extends StatelessWidget {
  const PreviewQuote({required this.text, super.key});

  final String text;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Container(
      padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
      decoration: BoxDecoration(
        border: BorderDirectional(
          start: BorderSide(color: colors.accent, width: 2),
        ),
      ),
      child: MarkdownInlineText(
        text,
        style: context.typography.body.copyWith(
          color: colors.textSecondary,
          fontStyle: FontStyle.italic,
        ),
      ),
    );
  }
}
