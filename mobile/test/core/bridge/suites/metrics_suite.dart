// Kanso — metrics_suite.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The anonymous usage count, through the bridge: the notice state, the
/// closed feature list, and the consent-gated send.
void metricsSuite(OpenLibrary open) {
  group('anonymous usage metrics', () {
    test('the notice gates counts and sending', () async {
      final library = await open();
      final metrics = library.metrics();

      expect(await metrics.noticeSeen(), isFalse);

      // No notice, no send: this stays false without touching the network,
      // even when the native library was built in release mode for a test.
      expect(
        await metrics.sendDue(tablet: false, osVersion: 'unknown', ai: false),
        isFalse,
      );
      await metrics.record(feature: 'lock');
      await expectLater(
        metrics.record(feature: 'not_a_feature'),
        throwsA(isA<KansoError_Invalid>()),
      );

      await metrics.markNoticeSeen();
      expect(await metrics.noticeSeen(), isTrue);
    });
  });
}
