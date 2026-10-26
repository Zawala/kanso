// Kanso — muted_tag_color_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show BoolAny, Glados2, any;
import 'package:kanso/core/utils/muted_tag_color.dart';

import '../../helpers/property.dart';
import '../../helpers/tag_color.dart';

void main() {
  group('mutedTagColor properties', () {
    Glados2<Color, bool>(
      any.opaqueColor,
      any.bool,
      kansoExploreConfig(),
    ).test('lightness lands in the band of the surface', (color, dark) {
      final (_, _, lightness) = toHsl(mutedTagColor(color, dark: dark));
      final (low, high) = dark ? (0.50, 0.76) : (0.32, 0.50);
      expect(lightness, greaterThanOrEqualTo(low - 0.01));
      expect(lightness, lessThanOrEqualTo(high + 0.01));
    });

    Glados2<Color, bool>(
      any.opaqueColor,
      any.bool,
      kansoExploreConfig(),
    ).test('saturation never grows and is capped', (color, dark) {
      final (_, before, _) = toHsl(color);
      final (_, after, _) = toHsl(mutedTagColor(color, dark: dark));
      expect(after, lessThanOrEqualTo(before + 0.02));
      expect(after, lessThanOrEqualTo(0.42 + 0.02));
    });

    Glados2<Color, bool>(
      any.opaqueColor,
      any.bool,
      kansoExploreConfig(),
    ).test('the result is opaque', (color, dark) {
      expect(mutedTagColor(color, dark: dark).a, 1);
    });
  });

  group('mutedTagColor examples', () {
    test('a grey stays grey', () {
      final grey = mutedTagColor(TagColor.grey.color, dark: true);
      final (_, saturation, _) = toHsl(grey);
      expect(saturation, lessThan(0.02));
    });

    test('red stays red', () {
      final (hue, _, _) = toHsl(mutedTagColor(TagColor.red.color, dark: true));
      expect(hue, closeTo(0, 4));
    });

    test('every tag colour reads differently on dark and light', () {
      for (final tag in TagColor.values) {
        expect(
          mutedTagColor(tag.color, dark: true),
          isNot(mutedTagColor(tag.color, dark: false)),
        );
      }
    });
  });
}
