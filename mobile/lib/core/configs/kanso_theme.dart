// Kanso — kanso_theme.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../models/data/kanso_fonts.dart';
import 'kanso_palette.dart';
import 'kanso_theme_scope.dart';
import 'kanso_typography.dart';

/// Builds the tokens a page draws with from a [KansoPalette].
///
/// There is no framework theme: Kanso hands its colours and type scale down
/// itself, so nothing falls back to a default chosen by a widget library,
/// and the app renders the same on iPhone, iPad and Android.
abstract final class KansoTheme {
  /// Kanso Light with the default fonts and text size.
  static final KansoThemeData light = build(KansoPalette.kansoLight);

  /// Kanso Dark with the default fonts and text size.
  static final KansoThemeData dark = build(KansoPalette.kansoDark);

  /// The theme drawn in [palette] with note text set in [fonts] at
  /// [textSize] and [interfaceTextSize]. Only the note styles of
  /// `KansoTypography` change with the fonts and note sizes; the interface
  /// keeps Kanso's own font and follows its own scale.
  static KansoThemeData build(
    KansoPalette palette, {
    KansoFonts fonts = const KansoFonts(),
    double textSize = 1,
    double? previewTextSize,
    double interfaceTextSize = 1,
  }) {
    return KansoThemeData(
      brightness: palette.dark ? Brightness.dark : Brightness.light,
      colors: palette.colors,
      syntax: palette.syntax,
      typography: KansoTypography.fromColors(
        palette.colors,
        fonts: fonts,
        textSize: textSize,
        previewTextSize: previewTextSize,
        interfaceTextSize: interfaceTextSize,
      ),
    );
  }
}
