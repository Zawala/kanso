// Kanso — contrast_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, any;
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/utils/contrast.dart';

import '../../helpers/property.dart';

void main() {
  group('contrastRatio properties', () {
    Glados2<Color, Color>(
      any.opaqueColor,
      any.opaqueColor,
      kansoExploreConfig(),
    ).test('is symmetric', (a, b) {
      expect(contrastRatio(a, b), closeTo(contrastRatio(b, a), 1e-12));
    });

    Glados2<Color, Color>(
      any.opaqueColor,
      any.opaqueColor,
      kansoExploreConfig(),
    ).test('is between 1 and 21', (a, b) {
      final ratio = contrastRatio(a, b);
      expect(ratio, greaterThanOrEqualTo(1));
      expect(ratio, lessThanOrEqualTo(21 + 1e-9));
    });

    Glados<Color>(any.opaqueColor, kansoExploreConfig()).test(
      'a colour against itself is exactly 1',
      (color) {
        expect(contrastRatio(color, color), 1);
      },
    );
  });

  group('contrastRatio examples', () {
    test('black on white is 21', () {
      expect(
        contrastRatio(const Color(0xFF000000), const Color(0xFFFFFFFF)),
        closeTo(21, 1e-9),
      );
    });

    test('luminance of black is 0 and of white is 1', () {
      expect(relativeLuminance(const Color(0xFF000000)), 0);
      expect(relativeLuminance(const Color(0xFFFFFFFF)), closeTo(1, 1e-9));
    });
  });

  group('Kanso themes meet their contrast targets', () {
    final themes = <String, KansoColors>{
      'Dark': KansoColors.dark,
      'Light': KansoColors.light,
    };

    for (final entry in themes.entries) {
      final colors = entry.value;
      test('${entry.key}: text on editor is at least 7', () {
        expect(
          contrastRatio(colors.text, colors.editor),
          greaterThanOrEqualTo(7),
        );
      });

      test('${entry.key}: secondary text on list is at least 4.5', () {
        expect(
          contrastRatio(colors.textSecondary, colors.list),
          greaterThanOrEqualTo(4.5),
        );
      });

      test('${entry.key}: muted text on sidebar is at least 3', () {
        expect(
          contrastRatio(colors.textMuted, colors.sidebar),
          greaterThanOrEqualTo(3),
        );
      });
    }
  });
}
