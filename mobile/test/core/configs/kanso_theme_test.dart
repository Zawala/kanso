// Kanso — kanso_theme_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_syntax_colors.dart';
import 'package:kanso/core/configs/kanso_theme.dart';

void main() {
  group('KansoTheme.build', () {
    test("draws a palette's colours, syntax and brightness", () {
      final theme = KansoTheme.build(KansoPalette.kansoDark);
      expect(theme.brightness, Brightness.dark);
      expect(theme.extension<KansoColors>(), KansoColors.dark);
      expect(theme.extension<KansoSyntaxColors>(), KansoSyntaxColors.dark);
      expect(theme.scaffoldBackgroundColor, KansoColors.dark.editor);
    });

    test('a light palette gives a light theme', () {
      final theme = KansoTheme.build(KansoPalette.kansoLight);
      expect(theme.brightness, Brightness.light);
      expect(theme.colorScheme.primary, KansoColors.light.button);
    });

    test('the shared themes are the two Kanso palettes', () {
      expect(KansoTheme.light.extension<KansoColors>(), KansoColors.light);
      expect(KansoTheme.dark.extension<KansoColors>(), KansoColors.dark);
    });
  });
}
