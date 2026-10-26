// Kanso — installed_theme_loading_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// Where the installed themes will be while they are read: still cards.
class InstalledThemeLoadingRow extends StatelessWidget {
  const InstalledThemeLoadingRow({super.key});

  /// How many cards are drawn.
  static const int cards = 3;

  /// How tall each is.
  static const double cardHeight = 150;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      label: context.l10n.themesLoading,
      container: true,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        physics: const NeverScrollableScrollPhysics(),
        padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
        child: Row(
          children: <Widget>[
            for (var i = 0; i < cards; i++) ...<Widget>[
              Container(
                width: KansoSizes.themeCardWidth,
                height: cardHeight,
                padding: const EdgeInsets.all(KansoSpacing.sm),
                decoration: BoxDecoration(
                  color: colors.elevated,
                  borderRadius: KansoRadii.largeBorder,
                  border: Border.all(color: colors.divider),
                ),
                child: const Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    KansoSkeleton(height: 10),
                    SizedBox(height: KansoSpacing.xs),
                    KansoSkeleton(width: 90, height: 10),
                    SizedBox(height: KansoSpacing.xs),
                    KansoSkeleton(width: 60, height: 10),
                    Spacer(),
                    KansoSkeleton(width: 80),
                  ],
                ),
              ),
              const SizedBox(width: KansoSpacing.sm),
            ],
          ],
        ),
      ),
    );
  }
}
