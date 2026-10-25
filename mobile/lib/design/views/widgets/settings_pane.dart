// Kanso — settings_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/font_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_scroll_host.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/mock_data.dart';

/// Settings: a close button, a centred title and rows grouped in rounded
/// sections, each with an icon, a label and its current value.
class SettingsPane extends StatelessWidget {
  const SettingsPane({this.startAtEnd = false, super.key});

  final bool startAtEnd;

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: KansoIconButton(
              icon: KansoIcons.close,
              semanticLabel: l10n.settingsClose,
              onPressed: () {},
            ),
            title: l10n.settingsTitle,
          ),
          Expanded(
            child: KansoScrollHost(
              startAtEnd: startAtEnd,
              builder: (context, controller) {
                return BlocBuilder<AppearanceBloc, AppearanceState>(
                  builder: (context, appearance) {
                    final fonts = appearance.fonts;
                    final textSize = appearance.textSize;
                    return ListView(
                      controller: controller,
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      children: <Widget>[
                        KansoGroupedSection(
                          title: l10n.settingsGeneral,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.notebook,
                              label: l10n.settingsDefaultNotebook,
                              value: MockData.defaultNotebook,
                              chevron: true,
                              onTap: () {},
                            ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.settingsAppearance,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.palette,
                              label: l10n.settingsTheme,
                              value: MockData.currentThemeName,
                              chevron: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.typeface,
                              label: l10n.settingsFonts,
                              value: l10n.fontsSummary(fonts),
                              chevron: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.shapes,
                              label: l10n.settingsIcons,
                              value: MockData.iconSet,
                              chevron: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.textSize,
                              label: l10n.settingsTextSize,
                              value: l10n.textSizeName(textSize),
                              chevron: true,
                              onTap: () {},
                            ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.settingsSync,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.cloud,
                              label: l10n.settingsSyncAccount,
                              value: l10n.settingsSyncedJustNow,
                              chevron: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.bell,
                              label: l10n.settingsWakeDevice,
                              trailing: KansoSwitch(
                                value: true,
                                onChanged: (_) {},
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.settingsLockedNotes,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.scanFace,
                              label: l10n.settingsUnlockFaceId,
                              trailing: KansoSwitch(
                                value: true,
                                onChanged: (_) {},
                              ),
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.clock,
                              label: l10n.settingsLockAfter,
                              value: l10n.settingsTenMinutes,
                              chevron: true,
                              onTap: () {},
                            ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.settingsAccount,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.user,
                              label: l10n.settingsSignedInAs,
                              value: MockData.signedInAs,
                            ),
                            KansoSettingsRow(
                              label: l10n.settingsSignOut,
                              danger: true,
                              centered: true,
                              onTap: () {},
                            ),
                          ],
                        ),
                        const SizedBox(height: KansoSpacing.lg),
                        KansoGroupedSection(
                          title: l10n.settingsAbout,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.info,
                              label: l10n.settingsVersion,
                              value: MockData.version,
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.globe,
                              label: l10n.settingsWebsite,
                              external: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.fileText,
                              label: l10n.settingsReleaseNotes,
                              external: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.circleHelp,
                              label: l10n.settingsHelp,
                              external: true,
                              onTap: () {},
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.scale,
                              label: l10n.settingsLicences,
                              external: true,
                              onTap: () {},
                            ),
                          ],
                        ),
                      ],
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
