// Kanso — kanso_theme.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import 'kanso_colors.dart';
import 'kanso_radii.dart';
import 'kanso_syntax_colors.dart';
import 'kanso_typography.dart';

/// The light and dark [ThemeData], built from Kanso's tokens.
///
/// Everything that Material would otherwise choose by [TargetPlatform] is
/// fixed here (fonts, density, tap targets, page transitions, ripples), so
/// the app renders identically on iPhone, iPad and Android.
abstract final class KansoTheme {
  static final ThemeData light = _build(
    brightness: Brightness.light,
    colors: KansoColors.light,
    syntax: KansoSyntaxColors.light,
  );

  static final ThemeData dark = _build(
    brightness: Brightness.dark,
    colors: KansoColors.dark,
    syntax: KansoSyntaxColors.dark,
  );

  /// The same transition on every platform: a slide with an edge swipe
  /// back, so the back gesture works the same way on iPhone and Android.
  static const PageTransitionsTheme _pageTransitions = PageTransitionsTheme(
    builders: <TargetPlatform, PageTransitionsBuilder>{
      TargetPlatform.android: CupertinoPageTransitionsBuilder(),
      TargetPlatform.iOS: CupertinoPageTransitionsBuilder(),
      TargetPlatform.fuchsia: CupertinoPageTransitionsBuilder(),
      TargetPlatform.linux: CupertinoPageTransitionsBuilder(),
      TargetPlatform.macOS: CupertinoPageTransitionsBuilder(),
      TargetPlatform.windows: CupertinoPageTransitionsBuilder(),
    },
  );

  static ThemeData _build({
    required Brightness brightness,
    required KansoColors colors,
    required KansoSyntaxColors syntax,
  }) {
    final typography = KansoTypography.fromColors(colors);
    final textTheme = typography.toTextTheme();
    final scheme = ColorScheme(
      brightness: brightness,
      primary: colors.button,
      onPrimary: colors.buttonText,
      secondary: colors.accent,
      onSecondary: colors.buttonText,
      error: colors.danger,
      onError: colors.buttonText,
      surface: colors.editor,
      onSurface: colors.text,
      onSurfaceVariant: colors.textSecondary,
      surfaceContainerHighest: colors.fill,
      outline: colors.divider,
      outlineVariant: colors.divider,
      surfaceTint: Colors.transparent,
    );

    return ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      extensions: <ThemeExtension<dynamic>>[colors, syntax, typography],
      fontFamily: KansoTypography.sansFamily,
      textTheme: textTheme,
      primaryTextTheme: textTheme,
      typography: Typography.material2021(
        // Fixed, not the running platform's, so no platform font leaks.
        // ignore: avoid_redundant_argument_values
        platform: TargetPlatform.android,
        colorScheme: scheme,
      ),
      scaffoldBackgroundColor: colors.editor,
      canvasColor: colors.editor,
      cardColor: colors.elevated,
      dividerColor: colors.divider,
      hoverColor: colors.hover,
      focusColor: colors.hover,
      highlightColor: Colors.transparent,
      splashColor: Colors.transparent,
      splashFactory: NoSplash.splashFactory,
      visualDensity: VisualDensity.standard,
      materialTapTargetSize: MaterialTapTargetSize.padded,
      pageTransitionsTheme: _pageTransitions,
      iconTheme: IconThemeData(color: colors.text, size: 20),
      dividerTheme: DividerThemeData(color: colors.divider, thickness: 1),
      appBarTheme: AppBarTheme(
        backgroundColor: colors.editor,
        foregroundColor: colors.text,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        scrolledUnderElevation: 0,
        titleTextStyle: typography.listTitle,
      ),
      scrollbarTheme: ScrollbarThemeData(
        thumbVisibility: const WidgetStatePropertyAll<bool>(false),
        radius: const Radius.circular(KansoRadii.small),
        thumbColor: WidgetStatePropertyAll<Color>(colors.textMuted),
      ),
    );
  }
}
