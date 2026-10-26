// Kanso — core_check_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/core_check_status.dart';
import '../../state/core_check/core_check_bloc.dart';

/// What the core check shows: the library's version, whether it can check
/// certificates, the notes it holds and the last change it reported, with a
/// button that makes a note.
class CoreCheckView extends StatelessWidget {
  const CoreCheckView({super.key});

  /// The widest the facts grow, so a tablet's lines stay short.
  static const double contentMaxWidth = 560;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return Scaffold(
      backgroundColor: colors.app,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            title: l10n.coreCheckTitle,
            leading: KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.noteBack,
              onPressed: () => Navigator.of(context).maybePop(),
            ),
          ),
          Expanded(
            child: BlocBuilder<CoreCheckBloc, CoreCheckState>(
              builder: (context, state) {
                return switch (state.status) {
                  CoreCheckStatus.loading => Center(
                    child: Text(
                      l10n.coreCheckLoading,
                      style: context.typography.secondary,
                    ),
                  ),
                  CoreCheckStatus.failed => const _Failed(),
                  CoreCheckStatus.ready => _Ready(state: state),
                };
              },
            ),
          ),
        ],
      ),
    );
  }
}

class _Failed extends StatelessWidget {
  const _Failed();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.lg),
        child: KansoMessageCard(
          icon: KansoIcons.circleX,
          title: l10n.coreCheckFailedTitle,
          message: l10n.coreCheckFailedMessage,
          actions: <Widget>[
            KansoButton(
              label: l10n.coreCheckRetry,
              onPressed: () => context.read<CoreCheckBloc>().add(
                const CoreCheckEvent.started(),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _Ready extends StatelessWidget {
  const _Ready({required this.state});

  final CoreCheckState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return Align(
      alignment: Alignment.topCenter,
      child: ConstrainedBox(
        constraints: const BoxConstraints(
          maxWidth: CoreCheckView.contentMaxWidth,
        ),
        child: ListView(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            bottom + KansoSpacing.lg,
          ),
          children: <Widget>[
            KansoGroupedSection(
              title: l10n.coreCheckLibrary,
              children: <Widget>[
                KansoSettingsRow(
                  label: l10n.coreCheckVersion,
                  value: state.version,
                ),
                KansoSettingsRow(
                  label: l10n.coreCheckCertificates,
                  value: state.tlsReady
                      ? l10n.coreCheckCertificatesReady
                      : l10n.coreCheckCertificatesNotReady,
                ),
                KansoSettingsRow(
                  label: l10n.coreCheckNotes,
                  value: '${state.noteCount}',
                ),
                KansoSettingsRow(
                  label: l10n.coreCheckLastChange,
                  value: state.lastChange.isEmpty
                      ? l10n.coreCheckNoChange
                      : l10n.coreCheckChanged(state.lastChange.length),
                ),
              ],
            ),
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: l10n.coreCheckNewNote,
              icon: KansoIcons.plus,
              onPressed: () => context.read<CoreCheckBloc>().add(
                const CoreCheckEvent.noteCreated(),
              ),
            ),
            if (state.error != null) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              Text(
                l10n.coreCheckActionFailed,
                textAlign: TextAlign.center,
                style: context.typography.secondary.copyWith(
                  color: context.colors.danger,
                ),
              ),
            ],
            if (state.latest.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.lg),
              KansoGroupedSection(
                title: l10n.coreCheckLatest,
                children: <Widget>[
                  for (final row in state.latest)
                    KansoSettingsRow(
                      label: row.title,
                      icon: row.locked ? KansoIcons.lock : KansoIcons.fileText,
                    ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }
}
