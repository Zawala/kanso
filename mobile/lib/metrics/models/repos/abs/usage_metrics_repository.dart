// Kanso — usage_metrics_repository.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A feature the app's product code may count. The names are the closed
/// list `kanso-metrics` accepts; Rust owns the rule that only one of each is
/// counted per day.
enum UsageFeature {
  noteCreated('note_created'),
  voiceInput('voice_input'),
  publish('publish'),
  lock('lock'),
  ai('ai'),
  quickCapture('quick_capture'),
  sharedNote('shared_note'),
  export('export');

  const UsageFeature(this.wireName);

  /// The name sent across the bridge.
  final String wireName;
}

/// The anonymous usage count as the Flutter shell sees it: the notice state,
/// feature counts, and the daily batch. Implementations must never make a
/// normal note action fail because metrics could not be written or sent.
abstract interface class UsageMetricsRepository {
  /// Whether the one-time notice was acknowledged.
  Future<bool> noticeSeen();

  /// Keeps the acknowledgement. Returns whether it was saved.
  Future<bool> markNoticeSeen();

  /// Counts [feature] once for today, if the notice was seen.
  Future<void> record(UsageFeature feature);

  /// Sends what is due. Returns whether the Worker accepted a batch.
  Future<bool> sendDue({
    required bool tablet,
    required String osVersion,
    required bool ai,
  });
}
