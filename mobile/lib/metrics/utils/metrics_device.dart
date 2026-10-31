// Kanso — metrics_device.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

/// The value sent when the platform does not expose a usable version.
const String unknownMetricsOsVersion = 'unknown';

/// A numeric OS version from the platform's descriptive string.
///
/// iOS says `Version 18.1 (Build 22B83)` and Android says
/// `Android 15 (API 35)`; the metrics contract takes just `18.1` or `15`,
/// never spaces or parentheses.
String metricsOsVersion([String? platformVersion]) {
  final text = platformVersion ?? Platform.operatingSystemVersion;
  final match = RegExp(r'\d+(?:\.\d+)*').firstMatch(text);
  return match?.group(0) ?? unknownMetricsOsVersion;
}
