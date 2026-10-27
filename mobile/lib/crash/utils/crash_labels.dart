// Kanso — crash_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/crash_kind.dart';
import '../models/data/crash_send_outcome.dart';

/// The words for where a crash report came from and how sending it ended.
extension CrashLabels on AppLocalizations {
  /// Where a report came from, for its row.
  String crashKindName(CrashKind kind) => switch (kind) {
    CrashKind.dart => crashKindDart,
    CrashKind.iosNative => crashKindIosNative,
    CrashKind.androidNative => crashKindAndroidNative,
    CrashKind.androidAnr => crashKindAndroidAnr,
    CrashKind.androidJvm => crashKindAndroidJvm,
  };

  /// What to tell the person once a send has ended.
  String crashOutcomeMessage(CrashSendOutcome outcome) => switch (outcome) {
    CrashSendOutcome.sent => crashOutcomeSent,
    CrashSendOutcome.rejected => crashOutcomeRejected,
    CrashSendOutcome.tryLater => crashOutcomeTryLater,
    CrashSendOutcome.unreachable => crashOutcomeUnreachable,
  };
}
