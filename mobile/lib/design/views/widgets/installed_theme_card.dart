// Kanso — installed_theme_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/theme_preview.dart';
import 'theme_code_preview.dart';

/// A theme in the Installed row: a preview of Markdown in its colours and
/// its name. The theme in use is outlined in the accent colour with a check.
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
        border: Border.all(
          color: current ? colors.accent : colors.divider,
          width: current ? 2 : 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          ThemeCodePreview(theme: theme),
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
                if (current)
                  KansoIcon(
                    KansoIcons.check,
                    size: KansoSizes.iconSmall + 2,
                    color: colors.accent,
                    semanticLabel: l10n.themesCurrent,
                  )
                else
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
