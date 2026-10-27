// Kanso — settings_sync_section.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/sync_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';

/// Settings › Sync: with no account a row that signs in; with an account
/// that is signed in but locked a row that opens its notes on this device;
/// with one signed in the account's row (its name, to its page), the status
/// of the sync and Sync now. It says under it that the notes are not in
/// device backups.
///
/// It needs the app's `AccountBloc` and `SyncStatusBloc`.
class SettingsSyncSection extends StatelessWidget {
  const SettingsSyncSection({required this.iconInset, super.key});

  /// How far a row's hairline is inset: past its icon.
  final double iconInset;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<AccountBloc, AccountBlocState>(
      builder: (context, account) {
        return BlocBuilder<SyncStatusBloc, SyncStatusState>(
          builder: (context, sync) {
            final locked = account.locked;
            final username = account.username;
            return KansoGroupedSection(
              title: l10n.settingsSync,
              footer: l10n.settingsSyncBackupFooter,
              dividerIndent: iconInset,
              children: <Widget>[
                if (locked != null)
                  KansoSettingsRow(
                    icon: KansoIcons.lock,
                    label: l10n.settingsSyncOpenNotes,
                    chevron: true,
                    onTap: () => context.push(KansoRoutes.settingsUnlock),
                  )
                else if (account.signedIn && username != null) ...<Widget>[
                  KansoSettingsRow(
                    icon: KansoIcons.user,
                    label: username,
                    value: l10n.settingsSyncAccount,
                    chevron: true,
                    onTap: () => context.push(KansoRoutes.settingsAccount),
                  ),
                  KansoSettingsRow(
                    icon: KansoIcons.cloud,
                    label: l10n.settingsSyncStatus,
                    value: l10n.syncStatusName(sync.status),
                  ),
                  KansoSettingsRow(
                    icon: KansoIcons.refreshCw,
                    label: l10n.settingsSyncNow,
                    value: sync.busy ? l10n.settingsSyncSyncing : null,
                    onTap: sync.busy
                        ? null
                        : () => context.read<SyncStatusBloc>().add(
                            const SyncStatusEvent.syncRequested(),
                          ),
                  ),
                ] else
                  KansoSettingsRow(
                    icon: KansoIcons.cloud,
                    label: l10n.settingsSyncSignIn,
                    chevron: true,
                    onTap: () => context.push(KansoRoutes.settingsSignIn),
                  ),
              ],
            );
          },
        );
      },
    );
  }
}
