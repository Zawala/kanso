// Kanso — app_environment.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The entry point the app was started from.
///
/// There are no native flavours: the same binary ships to every device, so
/// the environment only selects Dart-side registrations and diagnostics.
enum AppEnvironment {
  development,
  production;

  /// The name Injectable registrations use to select an environment.
  String get injectableName => name;
}
