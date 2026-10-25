// Kanso — wiki_completion_popup.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_shadows.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../models/data/mock_completion.dart';
import 'highlighted_text.dart';

/// The notes offered while a `[[` link is typed: each one's title, with
/// what was typed highlighted, and its notebook. The first is the one
/// Return takes.
class WikiCompletionPopup extends StatelessWidget {
  const WikiCompletionPopup({
    required this.typed,
    required this.completions,
    super.key,
  });

  final String typed;
  final List<MockCompletion> completions;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(color: colors.divider),
        boxShadow: KansoShadows.floating(Theme.brightnessOf(context)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          for (var i = 0; i < completions.length; i++) ...<Widget>[
            if (i > 0) const KansoDivider(),
            Container(
              height: KansoSizes.touchTarget,
              color: i == 0 ? colors.rowActive : null,
              padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: HighlightedText(
                      completions[i].title,
                      query: typed,
                      style: typography.body,
                      maxLines: 1,
                    ),
                  ),
                  const SizedBox(width: KansoSpacing.xs),
                  ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 120),
                    child: Text(
                      completions[i].notebook,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}
