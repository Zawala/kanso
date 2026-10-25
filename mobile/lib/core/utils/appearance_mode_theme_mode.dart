// Kanso — appearance_mode_theme_mode.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../models/data/appearance_mode.dart';

/// Maps the app's appearance setting to Flutter's [ThemeMode].
extension AppearanceModeThemeMode on AppearanceMode {
  ThemeMode get themeMode => switch (this) {
    AppearanceMode.system => ThemeMode.system,
    AppearanceMode.light => ThemeMode.light,
    AppearanceMode.dark => ThemeMode.dark,
  };
}
