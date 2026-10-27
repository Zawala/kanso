// Kanso — crash_send_outcome.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How sending a report ended.
enum CrashSendOutcome {
  /// The server stored it; the report is deleted.
  sent,

  /// The server said it is not a valid report (400, 413); it is kept.
  rejected,

  /// Too many reports from this address (429); it is kept for later.
  tryLater,

  /// The server could not be reached; it is kept.
  unreachable,
}
