// Kanso — rust_usage_metrics_repository.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/metrics.dart';
import '../abs/usage_metrics_repository.dart';

/// [UsageMetricsRepository] over the Rust side that owns the ledger, the
/// notice state and the release gate.
@LazySingleton(as: UsageMetricsRepository)
class RustUsageMetricsRepository implements UsageMetricsRepository {
  RustUsageMetricsRepository(this._metrics);

  final KansoMetrics _metrics;

  @override
  Future<bool> noticeSeen() async {
    try {
      return await _metrics.noticeSeen();
    } on Object {
      // A library that cannot answer must not put a notice over the app or
      // start a send it cannot finish.
      return true;
    }
  }

  @override
  Future<bool> markNoticeSeen() async {
    try {
      await _metrics.markNoticeSeen();
      return true;
    } on Object {
      return false;
    }
  }

  @override
  Future<void> record(UsageFeature feature) async {
    try {
      await _metrics.record(feature: feature.wireName);
    } on Object {
      // Counting is best effort; a failed metadata write never becomes an
      // error in the note flow that used the feature.
    }
  }

  @override
  Future<bool> sendDue({
    required bool tablet,
    required String osVersion,
    required bool ai,
  }) async {
    try {
      return await _metrics.sendDue(
        tablet: tablet,
        osVersion: osVersion,
        ai: ai,
      );
    } on Object {
      return false;
    }
  }
}
