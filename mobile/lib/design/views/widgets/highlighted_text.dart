// Kanso — highlighted_text.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/search_match_spans.dart';

/// [text] with the words of [query] on a soft accent wash, as a search
/// result shows what it matched.
class HighlightedText extends StatelessWidget {
  const HighlightedText(
    this.text, {
    required this.query,
    required this.style,
    this.maxLines,
    super.key,
  });

  final String text;
  final String query;
  final TextStyle style;
  final int? maxLines;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Text.rich(
      TextSpan(
        style: style,
        children: <InlineSpan>[
          for (final span in searchMatchSpans(text, query))
            TextSpan(
              text: span.text,
              style: span.matched
                  ? TextStyle(
                      color: colors.text,
                      backgroundColor: colors.accent.withValues(
                        alpha: KansoEffects.matchWash,
                      ),
                    )
                  : null,
            ),
        ],
      ),
      maxLines: maxLines,
      overflow: maxLines == null ? TextOverflow.clip : TextOverflow.ellipsis,
    );
  }
}
