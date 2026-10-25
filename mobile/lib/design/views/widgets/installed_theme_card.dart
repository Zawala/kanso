// Kanso — installed_theme_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_check_badge.dart';
import '../../models/data/theme_preview.dart';
import 'theme_code_preview.dart';

/// A theme in the Installed row: a preview of Markdown in its colours and
/// its name and a badge that says where it came from. The theme in use is
/// outlined in the accent colour and wears a check mark badge on its
/// preview.
class InstalledThemeCard extends StatelessWidget {
  const InstalledThemeCard({
    required this.theme,
    this.current = false,
    super.key,
  });

  final ThemePreview theme;
  final bool current;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Container(
      width: KansoSizes.themeCardWidth,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
      ),
      // Drawn over the card, so the outline does not move its content.
      foregroundDecoration: current
          ? BoxDecoration(
              borderRadius: KansoRadii.largeBorder,
              border: Border.all(color: colors.accent, width: 2),
            )
          : null,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Stack(
            children: <Widget>[
              ThemeCodePreview(theme: theme),
              if (current)
                PositionedDirectional(
                  top: KansoSpacing.xs,
                  end: KansoSpacing.xs,
                  child: KansoCheckBadge(semanticLabel: l10n.themesCurrent),
                ),
            ],
          ),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Text(
                    theme.name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.listTitle,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
                KansoBadge(
                  label: theme.builtIn
                      ? l10n.themesBadgeBuiltIn
                      : l10n.themesBadgeInstalled,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
