// Kanso — metrics_device_test.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/metrics/utils/metrics_device.dart';

void main() {
  group('metricsOsVersion', () {
    test('keeps only the numeric iOS version', () {
      expect(
        metricsOsVersion('Version 18.1 (Build 22B83)'),
        '18.1',
      );
    });

    test('keeps only the numeric Android version', () {
      expect(metricsOsVersion('Android 15 (API 35)'), '15');
    });

    test('keeps a patch version without its description', () {
      expect(
        metricsOsVersion('Version 14.6.1 (Build 23G93)'),
        '14.6.1',
      );
    });

    test('uses the unknown sentinel when there is no number', () {
      expect(metricsOsVersion('unavailable'), unknownMetricsOsVersion);
      expect(metricsOsVersion(''), unknownMetricsOsVersion);
    });
  });
}
