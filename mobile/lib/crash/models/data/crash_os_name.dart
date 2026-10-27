// Kanso — crash_os_name.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:json_annotation/json_annotation.dart';

/// The operating system a report came from (`os.name`).
enum CrashOsName {
  @JsonValue('ios')
  ios,

  @JsonValue('android')
  android,
}
