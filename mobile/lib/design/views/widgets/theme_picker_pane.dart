// Kanso — theme_picker_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_chip.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_scroll_host.dart';
import '../../../core/views/widgets/kanso_search_field.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/mock_data.dart';
import '../../models/data/theme_preview.dart';
import 'gallery_theme_card.dart';
import 'installed_theme_card.dart';

/// The theme picker: the installed themes in a row, each a preview of
/// Markdown in its own colours with the one in use outlined, and below it
/// the gallery with its search field, filters and themes to install.
///
/// On a tablet the gallery has two columns.
class ThemePickerPane extends StatelessWidget {
  const ThemePickerPane({
    required this.layout,
    this.startAtEnd = false,
    super.key,
  });

  final DesignLayout layout;
  final bool startAtEnd;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final columns = layout == DesignLayout.tablet ? 2 : 1;
    const themes = MockData.galleryThemes;
    // The theme in use comes first, so its check is the first thing seen.
    final installed = <ThemePreview>[
      ...MockData.installedThemes.where(
        (theme) => theme.name == MockData.currentThemeName,
      ),
      ...MockData.installedThemes.where(
        (theme) => theme.name != MockData.currentThemeName,
      ),
    ];
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
            title: l10n.themesTitle,
          ),
          Expanded(
            child: KansoScrollHost(
              startAtEnd: startAtEnd,
              builder: (context, controller) {
                return ListView(
                  controller: controller,
                  padding: EdgeInsets.only(
                    top: KansoSpacing.xs,
                    bottom: bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    KansoSectionHeader(label: l10n.themesInstalled),
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                      ),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: <Widget>[
                          for (final theme in installed) ...<Widget>[
                            InstalledThemeCard(
                              theme: theme,
                              current: theme.name == MockData.currentThemeName,
                            ),
                            const SizedBox(width: KansoSpacing.sm),
                          ],
                        ],
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.md),
                    KansoSectionHeader(label: l10n.themesGallery),
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: <Widget>[
                          KansoSearchField(hint: l10n.themesSearchHint),
                          const SizedBox(height: KansoSpacing.sm),
                          Wrap(
                            spacing: KansoSpacing.xs,
                            runSpacing: KansoSpacing.xs,
                            children: <Widget>[
                              KansoChip(
                                label: l10n.themesFilterAll,
                                selected: true,
                              ),
                              KansoChip(label: l10n.themesFilterLight),
                              KansoChip(label: l10n.themesFilterDark),
                              KansoChip(label: l10n.themesFilterInstalled),
                              KansoChip(label: l10n.themesFilterUpdates),
                            ],
                          ),
                          const SizedBox(height: KansoSpacing.md),
                          for (var i = 0; i < themes.length; i += columns)
                            Padding(
                              padding: const EdgeInsets.only(
                                bottom: KansoSpacing.sm,
                              ),
                              child: Row(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: <Widget>[
                                  for (var j = 0; j < columns; j++) ...<Widget>[
                                    if (j > 0)
                                      const SizedBox(width: KansoSpacing.sm),
                                    Expanded(
                                      child: i + j < themes.length
                                          ? GalleryThemeCard(
                                              theme: themes[i + j],
                                            )
                                          : const SizedBox.shrink(),
                                    ),
                                  ],
                                ],
                              ),
                            ),
                        ],
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
