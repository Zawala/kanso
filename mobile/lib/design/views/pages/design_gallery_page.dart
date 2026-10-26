// Kanso — design_gallery_page.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../core_check/views/pages/core_check_page.dart';
import '../../models/data/design_layout.dart';
import '../../models/data/design_screen.dart';
import '../../models/data/gallery_appearance.dart';
import '../../utils/page_for_screen.dart';
import '../../utils/screen_label.dart';

/// The page the app opens on while it is a design proposal: every mock
/// screen for a phone and for a tablet, and a control that switches the
/// whole app between light, dark and the device's setting.
class DesignGalleryPage extends StatelessWidget {
  const DesignGalleryPage({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return Scaffold(
      backgroundColor: colors.app,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            title: l10n.designGalleryTitle,
          ),
          Expanded(
            child: ListView(
              padding: EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.xs,
                KansoSpacing.md,
                bottom + KansoSpacing.lg,
              ),
              children: <Widget>[
                Text(
                  l10n.designGalleryIntro,
                  style: context.typography.secondary,
                ),
                const SizedBox(height: KansoSpacing.md),
                BlocBuilder<AppearanceBloc, AppearanceState>(
                  builder: (context, state) {
                    return KansoSegmented<GalleryAppearance>(
                      expanded: true,
                      selected: GalleryAppearance.of(state),
                      onChanged: (mode) {
                        final bloc = context.read<AppearanceBloc>();
                        mode.events.forEach(bloc.add);
                      },
                      segments: <KansoSegment<GalleryAppearance>>[
                        KansoSegment<GalleryAppearance>(
                          value: GalleryAppearance.system,
                          label: l10n.appearanceSystem,
                          icon: KansoIcons.monitor,
                        ),
                        KansoSegment<GalleryAppearance>(
                          value: GalleryAppearance.light,
                          label: l10n.appearanceLight,
                          icon: KansoIcons.sun,
                        ),
                        KansoSegment<GalleryAppearance>(
                          value: GalleryAppearance.dark,
                          label: l10n.appearanceDark,
                          icon: KansoIcons.moon,
                        ),
                      ],
                    );
                  },
                ),
                const SizedBox(height: KansoSpacing.lg),
                KansoGroupedSection(
                  title: l10n.designGalleryCoreTitle,
                  children: <Widget>[
                    KansoSettingsRow(
                      label: l10n.designGalleryCoreCheck,
                      chevron: true,
                      onTap: () {
                        Navigator.of(context).push(
                          MaterialPageRoute<void>(
                            builder: (_) => const CoreCheckPage(),
                          ),
                        );
                      },
                    ),
                  ],
                ),
                for (final layout in DesignLayout.values) ...<Widget>[
                  const SizedBox(height: KansoSpacing.lg),
                  KansoGroupedSection(
                    title: layout == DesignLayout.phone
                        ? l10n.designGalleryPhone
                        : l10n.designGalleryTablet,
                    children: <Widget>[
                      for (final screen in DesignScreen.values)
                        KansoSettingsRow(
                          label: l10n.screenName(screen),
                          chevron: true,
                          onTap: () {
                            Navigator.of(context).push(
                              MaterialPageRoute<void>(
                                builder: (_) => pageForScreen(screen, layout),
                              ),
                            );
                          },
                        ),
                    ],
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}
