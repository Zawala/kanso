// Kanso — palette_from_tokens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/kanso_syntax_style.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';

import '../../helpers/theme_tokens.dart';

void main() {
  group('paletteFromTokens', () {
    test('reads back the tokens made from a palette', () {
      expect(
        paletteFromTokens('kanso-light', tokensFor(KansoPalette.kansoLight)),
        KansoPalette.kansoLight,
      );
      expect(
        paletteFromTokens('kanso-dark', tokensFor(KansoPalette.kansoDark)),
        KansoPalette.kansoDark,
      );
    });

    test('a syntax token without a colour is drawn in the text colour', () {
      final tokens = tokensFor(KansoPalette.kansoDark);
      final stripped = ThemeTokens(
        name: tokens.name,
        dark: tokens.dark,
        ui: tokens.ui,
        caret: tokens.caret,
        selection: tokens.selection,
        selectionAlpha: tokens.selectionAlpha,
        palette: tokens.palette,
        syntax: <SyntaxStyleRow>[
          for (final row in tokens.syntax)
            if (row.token == 'keyword')
              const SyntaxStyleRow(token: 'keyword')
            else
              row,
        ],
      );
      final palette = paletteFromTokens('x', stripped);
      expect(palette.syntax.keyword, palette.syntax.text);
      expect(palette.syntax.string, KansoPalette.kansoDark.syntax.string);
    });

    test('a missing token keeps the Kanso colour for the appearance', () {
      final tokens = tokensFor(KansoPalette.kansoLight);
      final without = ThemeTokens(
        name: 'Bare',
        dark: false,
        ui: const <ColorRow>[],
        caret: 0,
        selection: 0,
        selectionAlpha: 0,
        syntax: const <SyntaxStyleRow>[],
        palette: tokens.palette,
      );
      final palette = paletteFromTokens('bare', without);
      expect(palette.colors, KansoPalette.kansoLight.colors);
      expect(palette.name, 'Bare');
      expect(palette.syntaxStyles, isEmpty);
    });

    test('italic and weight become styles, only where there is one', () {
      final palette = paletteFromTokens(
        'x',
        tokensFor(KansoPalette.kansoLight),
      );
      expect(
        palette.syntaxStyles['heading'],
        const KansoSyntaxStyle(weight: 700),
      );
      expect(
        palette.syntaxStyles['comment'],
        const KansoSyntaxStyle(italic: true),
      );
      expect(palette.syntaxStyles.containsKey('keyword'), isFalse);
    });

    test(
      'colours are opaque and the code background is the preview surface',
      () {
        final tokens = tokensFor(fakePalette('x'));
        final palette = paletteFromTokens('x', tokens);
        expect(palette.colors.accent.a, 1);
        expect(
          palette.syntax.codeBackground,
          Color(0xFF000000 | tokens.palette.surface),
        );
      },
    );
  });
}
