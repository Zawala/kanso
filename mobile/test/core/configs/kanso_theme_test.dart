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
      expect(theme.isDark, isTrue);
      expect(theme.colors, KansoColors.dark);
      expect(theme.syntax, KansoSyntaxColors.dark);
      expect(theme.colors.editor, KansoColors.dark.editor);
    });

    test('a light palette gives a light theme', () {
      final theme = KansoTheme.build(KansoPalette.kansoLight);
      expect(theme.brightness, Brightness.light);
      expect(theme.colors.button, KansoColors.light.button);
    });

    test('the shared themes are the two Kanso palettes', () {
      expect(KansoTheme.light.colors, KansoColors.light);
      expect(KansoTheme.dark.colors, KansoColors.dark);
    });

    test('the interface uses its raised type scale', () {
      final type = KansoTheme.light.typography;
      expect(type.largeTitle.fontSize, 32);
      expect(type.barTitle.fontSize, 18);
      expect(type.heading1.fontSize, 22);
      expect(type.heading2.fontSize, 18);
      expect(type.listTitle.fontSize, 16);
      expect(type.body.fontSize, 16);
      expect(type.secondary.fontSize, 14);
      expect(type.label.fontSize, 14);
      expect(type.meta.fontSize, 13);
      expect(type.sectionLabel.fontSize, 13);
      expect(type.body.fontFamily, 'Instrument Sans');
    });

    test('the interface scale leaves note text on its own scales', () {
      final type = KansoTheme.build(
        KansoPalette.kansoLight,
        textSize: 1.15,
        previewTextSize: 1.3,
        interfaceTextSize: 1.2,
      ).typography;
      expect(type.body.fontSize, closeTo(19.2, 1e-9));
      expect(type.largeTitle.fontSize, closeTo(38.4, 1e-9));
      expect(type.largeTitle.letterSpacing, closeTo(-0.96, 1e-9));
      expect(type.editorBody.fontSize, closeTo(20.7, 1e-9));
      expect(type.previewBody.fontSize, closeTo(23.4, 1e-9));
      expect(type.mono.fontSize, closeTo(15.525, 1e-9));
    });

    test('a theme blends its colours towards another', () {
      final half = KansoTheme.light.lerp(KansoTheme.dark, 0.5);
      expect(half.colors.app, isNot(KansoTheme.light.colors.app));
      expect(half.colors.app, isNot(KansoTheme.dark.colors.app));
      expect(
        KansoTheme.light.lerp(KansoTheme.dark, 1).colors,
        KansoColors.dark,
      );
    });
  });
}
