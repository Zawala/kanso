// Kanso — fake_usage_metrics.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/metrics/models/repos/abs/usage_metrics_repository.dart';

/// The anonymous count for widget and unit tests: records every call in
/// memory and never reaches the Rust core or the network.
class FakeUsageMetrics implements UsageMetricsRepository {
  FakeUsageMetrics({
    this.seen = false,
    this.markSeenResult = true,
    this.sendResult = false,
    this.noticeSeenThrows = false,
  });

  bool seen;
  bool markSeenResult;
  bool sendResult;
  bool noticeSeenThrows;

  int noticeSeenCalls = 0;
  int markSeenCalls = 0;
  int sendDueCalls = 0;
  final List<UsageFeature> recorded = <UsageFeature>[];
  final List<UsageSend> sends = <UsageSend>[];

  @override
  Future<bool> noticeSeen() async {
    noticeSeenCalls += 1;
    if (noticeSeenThrows) {
      throw StateError('notice read failed');
    }
    return seen;
  }

  @override
  Future<bool> markNoticeSeen() async {
    markSeenCalls += 1;
    if (markSeenResult) {
      seen = true;
    }
    return markSeenResult;
  }

  @override
  Future<void> record(UsageFeature feature) async {
    recorded.add(feature);
  }

  @override
  Future<bool> sendDue({
    required bool tablet,
    required String osVersion,
    required bool ai,
  }) async {
    sendDueCalls += 1;
    sends.add(UsageSend(tablet: tablet, osVersion: osVersion, ai: ai));
    return sendResult;
  }
}

/// One `sendDue` call's arguments.
class UsageSend {
  const UsageSend({
    required this.tablet,
    required this.osVersion,
    required this.ai,
  });

  final bool tablet;
  final String osVersion;
  final bool ai;
}
