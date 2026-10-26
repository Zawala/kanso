// Kanso — installed_theme_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/theme_selection.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../models/data/theme_preview.dart';
import 'installed_theme_card.dart';

/// The installed themes side by side, in a row that scrolls sideways. The
/// ones [selection] has in use are outlined; choosing a card picks its
/// theme.
class InstalledThemeRow extends StatelessWidget {
  const InstalledThemeRow({
    required this.themes,
    required this.selection,
    super.key,
  });

  final List<ThemePreview> themes;
  final ThemeSelection selection;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          for (final theme in themes) ...<Widget>[
            InstalledThemeCard(
              key: ValueKey<String>(theme.id),
              theme: theme,
              current: selection.isSelected(theme.id),
              onTap: () => context.read<AppearanceBloc>().add(
                AppearanceEvent.themePicked(theme.id, dark: theme.dark),
              ),
            ),
            const SizedBox(width: KansoSpacing.sm),
          ],
        ],
      ),
    );
  }
}
