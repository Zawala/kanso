// Kanso — licences_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/licences/licences_bloc.dart';

/// Settings › Licences: the packages the app is made of, each opening to its
/// licences. Drawn by Kanso, a page of its own rather than Flutter's.
class LicencesPane extends StatelessWidget {
  const LicencesPane({super.key});

  /// How many bars the page shows while the licences are read.
  static const int loadingRows = 10;

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
            leading: const ShellBackButton(),
            title: l10n.settingsLicences,
          ),
          Expanded(
            child: BlocBuilder<LicencesBloc, LicencesState>(
              builder: (context, state) {
                if (state.status == LoadStatus.loading) {
                  return Semantics(
                    label: l10n.licencesLoading,
                    container: true,
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: Column(
                        children: <Widget>[
                          for (var i = 0; i < loadingRows; i++)
                            const Padding(
                              padding: EdgeInsets.symmetric(
                                vertical: KansoSpacing.sm,
                              ),
                              child: KansoSkeleton(height: 14),
                            ),
                        ],
                      ),
                    ),
                  );
                }
                if (state.status == LoadStatus.failed) {
                  return Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        title: l10n.licencesFailedTitle,
                        message: l10n.licencesFailedMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => context.read<LicencesBloc>().add(
                              const LicencesEvent.started(),
                            ),
                          ),
                        ],
                      ),
                    ),
                  );
                }
                if (state.packages.isEmpty) {
                  return KansoEmptyState(
                    title: l10n.licencesEmptyTitle,
                    message: l10n.licencesEmptyMessage,
                  );
                }
                final packages = state.packages;
                return Padding(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    bottom + KansoSpacing.md,
                  ),
                  child: ClipRRect(
                    borderRadius: KansoRadii.largeBorder,
                    child: ColoredBox(
                      color: context.groupedSurface,
                      child: ListView.separated(
                        // The list is below the bar: the status bar's inset
                        // is not its to add.
                        padding: EdgeInsets.zero,
                        itemCount: packages.length,
                        separatorBuilder: (context, index) =>
                            const KansoDivider(indent: KansoSpacing.md),
                        itemBuilder: (context, index) {
                          final package = packages[index];
                          final open = state.expanded == package.name;
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: <Widget>[
                              KansoSettingsRow(
                                label: package.name,
                                value: l10n.licencesCount(
                                  package.licences.length,
                                ),
                                trailing: KansoIcon(
                                  open
                                      ? KansoIcons.chevronDown
                                      : KansoIcons.chevronRight,
                                  size: KansoSizes.iconSmall,
                                  color: colors.textMuted,
                                ),
                                onTap: () => context.read<LicencesBloc>().add(
                                  LicencesEvent.packageToggled(package.name),
                                ),
                              ),
                              if (open)
                                for (final licence in package.licences)
                                  Padding(
                                    padding: const EdgeInsets.fromLTRB(
                                      KansoSpacing.md,
                                      KansoSpacing.xs,
                                      KansoSpacing.md,
                                      KansoSpacing.md,
                                    ),
                                    child: Text(
                                      licence,
                                      style: context.typography.meta,
                                    ),
                                  ),
                            ],
                          );
                        },
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
