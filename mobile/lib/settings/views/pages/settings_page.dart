// Kanso — settings_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../shell/views/widgets/shell_page.dart';

/// Settings: where the appearance and the rest are changed.
class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ShellPage(
      title: l10n.settingsTitle,
      backgroundColor: context.colors.app,
      trailing: <Widget>[
        KansoIconButton(
          icon: KansoIcons.close,
          semanticLabel: l10n.settingsClose,
          onPressed: () => context.pop(),
        ),
      ],
      child: ListView(
        padding: const EdgeInsets.all(KansoSpacing.md),
        children: <Widget>[
          KansoGroupedSection(
            title: l10n.settingsAppearance,
            children: <Widget>[
              KansoSettingsRow(
                label: l10n.settingsTheme,
                chevron: true,
                onTap: () => context.push(KansoRoutes.settingsTheme),
              ),
              KansoSettingsRow(
                label: l10n.settingsFonts,
                chevron: true,
                onTap: () => context.push(KansoRoutes.fonts()),
              ),
              KansoSettingsRow(
                label: l10n.settingsTextSize,
                chevron: true,
                onTap: () => context.push(KansoRoutes.fonts(atSize: true)),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.lg),
          KansoGroupedSection(
            title: l10n.settingsAbout,
            children: <Widget>[
              KansoSettingsRow(
                label: l10n.settingsLicences,
                chevron: true,
                onTap: () => context.push(KansoRoutes.settingsLicences),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
