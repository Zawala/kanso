// Kanso — search_result_tile.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/mock_search_result.dart';
import 'highlighted_text.dart';

/// A search result: the note's title, its notebook and the text around the
/// match, with the matched words highlighted.
class SearchResultTile extends StatelessWidget {
  const SearchResultTile({
    required this.result,
    required this.query,
    this.onTap,
    super.key,
  });

  final MockSearchResult result;
  final String query;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return KansoPressable(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            HighlightedText(
              result.title,
              query: query,
              style: typography.listTitle,
              maxLines: 1,
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Row(
              children: <Widget>[
                KansoIcon(
                  KansoIcons.notebook,
                  size: KansoSizes.glyph,
                  color: colors.textMuted,
                ),
                const SizedBox(width: KansoSpacing.xxs),
                Flexible(
                  child: Text(
                    result.notebook,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.meta,
                  ),
                ),
              ],
            ),
            const SizedBox(height: KansoSpacing.xs),
            HighlightedText(
              result.snippet,
              query: query,
              style: typography.secondary,
              maxLines: 2,
            ),
          ],
        ),
      ),
    );
  }
}
