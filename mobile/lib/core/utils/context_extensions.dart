// Kanso — context_extensions.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../gen/l10n/app_localizations.dart';
import '../configs/kanso_colors.dart';
import '../configs/kanso_syntax_colors.dart';
import '../configs/kanso_theme_scope.dart';
import '../configs/kanso_typography.dart';

/// Short access to Kanso's tokens and strings from a [BuildContext].
extension KansoBuildContext on BuildContext {
  /// The interface colours of the current theme.
  KansoColors get colors => KansoThemeScope.of(this).colors;

  /// The Markdown and code colours of the current theme.
  KansoSyntaxColors get syntaxColors => KansoThemeScope.of(this).syntax;

  /// The surface of grouped rows (settings): lighter than the page by about
  /// the same step in light and dark. Light sits `elevated` on `app`; dark
  /// uses `list`, because `elevated` would stand out far more there.
  Color get groupedSurface {
    final palette = colors;
    return brightness == Brightness.dark ? palette.list : palette.elevated;
  }

  /// The type scale of the current theme.
  KansoTypography get typography => KansoThemeScope.of(this).typography;

  /// Whether the theme in force is a dark one.
  bool get isDark => KansoThemeScope.of(this).isDark;

  /// The theme's own light or dark.
  Brightness get brightness => KansoThemeScope.of(this).brightness;

  /// The platform the app is running on, which picks a few platform-shaped
  /// details: the height of a caret, the look of the keyboard.
  TargetPlatform get platform => defaultTargetPlatform;

  /// The app's strings, in the current language.
  AppLocalizations get l10n => AppLocalizations.of(this);
}
