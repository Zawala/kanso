// Kanso — gallery_theme_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/theme_appearance.dart';
import '../../models/data/theme_preview.dart';
import 'theme_code_preview.dart';

/// A theme of the gallery: a preview of Markdown in its colours, its name,
/// who made it, its licence and appearance, where its source is, and a
/// button to install it (or the Installed badge).
class GalleryThemeCard extends StatelessWidget {
  const GalleryThemeCard({
    required this.theme,
    this.installed = false,
    super.key,
  });

  final ThemePreview theme;
  final bool installed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final appearance = theme.appearance == ThemeAppearance.light
        ? l10n.themesAppearanceLight
        : l10n.themesAppearanceDark;
    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          ThemeCodePreview(theme: theme),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(
                  theme.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: typography.listTitle,
                ),
                const SizedBox(height: KansoSpacing.xxs),
                Text(
                  '${theme.author} · ${theme.license} · $appearance',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: typography.meta,
                ),
                const SizedBox(height: KansoSpacing.xs),
                Row(
                  children: <Widget>[
                    KansoIcon(
                      KansoIcons.github,
                      size: KansoSizes.iconSmall,
                      color: colors.link,
                    ),
                    const SizedBox(width: KansoSpacing.xxs),
                    Expanded(
                      child: Text(
                        theme.repository,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.meta.copyWith(color: colors.link),
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    if (installed)
                      KansoBadge(label: l10n.themesBadgeInstalled)
                    else
                      KansoButton(
                        label: l10n.themesInstall,
                        compact: true,
                        expanded: false,
                        onPressed: () {},
                      ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
