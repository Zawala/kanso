// Kanso — font_labels.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../bridge/api/types.dart' show DiagramRenderer;
import '../models/data/kanso_font_family.dart';
import '../models/data/kanso_fonts.dart';

/// The names of fonts and text sizes, in the current language.
extension FontLabels on AppLocalizations {
  /// What a settings row calls [font]: its own name, or "System" for the
  /// device's font.
  String fontName(KansoFontFamily font) {
    return font == KansoFontFamily.system ? fontsSystemShort : font.displayName;
  }

  /// A note size percentage shown by the control and settings summary.
  String textScaleName(double scale) => textSizePercent((scale * 100).round());

  /// What the settings call the diagram renderer [renderer].
  String diagramRendererName(DiagramRenderer renderer) {
    return switch (renderer) {
      DiagramRenderer.pele => diagramRendererPele,
      DiagramRenderer.kanso => diagramRendererKanso,
    };
  }

  /// The value of the Fonts row in settings: the choices in one line.
  String fontsSummary(KansoFonts fonts) {
    return fonts.summary(systemLabel: fontsSystemShort);
  }
}
