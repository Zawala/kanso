// Kanso — highlighted_text.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_effects.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/search_match_spans.dart';

/// [text] with the words of [query] on a softly rounded wash of the accent
/// colour, as a search result shows what it matched.
///
/// The wash is the accent at low opacity, so it reads on a light theme and
/// on a dark one, and its corners are rounded: a matched word sits in a
/// small pill rather than a hard box.
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

  /// How round the wash's corners are.
  static const double washRadius = 4;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Text.rich(
      TextSpan(
        style: style,
        children: <InlineSpan>[
          for (final span in searchMatchSpans(text, query))
            if (span.matched)
              WidgetSpan(
                alignment: PlaceholderAlignment.baseline,
                baseline: TextBaseline.alphabetic,
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: colors.accent.withValues(
                      alpha: KansoEffects.highlight,
                    ),
                    borderRadius: BorderRadius.circular(washRadius),
                  ),
                  child: Text(
                    span.text,
                    style: style.copyWith(color: colors.text, height: 1.2),
                  ),
                ),
              )
            else
              TextSpan(text: span.text),
        ],
      ),
      maxLines: maxLines,
      overflow: maxLines == null ? TextOverflow.clip : TextOverflow.ellipsis,
    );
  }
}
