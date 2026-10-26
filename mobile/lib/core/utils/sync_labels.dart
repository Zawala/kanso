// Kanso — sync_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../bridge/api/types.dart';

/// The words for the state of the account and its sync.
extension SyncLabels on AppLocalizations {
  /// What Settings says of [status]: "Off" while no account is signed in,
  /// and nothing until the status is first read.
  String syncStatusName(SyncStatus? status) {
    if (status == null) {
      return '';
    }
    return switch (status.phase) {
      SyncPhase.signedOut => settingsSyncOff,
      SyncPhase.idle => settingsSyncOn,
      SyncPhase.syncing => settingsSyncSyncing,
      SyncPhase.synced => settingsSyncSynced,
      SyncPhase.failed => settingsSyncAttention,
    };
  }
}
