// Kanso — count_usage.dart
// Created 2026-11-01.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import '../../injection.dart';
import '../models/repos/abs/usage_metrics_repository.dart';

/// Counts one use of [feature], where an action happens that no repository
/// of the metrics' own sees. Best effort and fire-and-forget: a missing
/// registration (a test without the app's services) or a failed write never
/// reaches the action that was counted.
void countUsage(UsageFeature feature) {
  try {
    unawaited(getIt<UsageMetricsRepository>().record(feature));
  } on Object {
    // Counting never disturbs the app.
  }
}
