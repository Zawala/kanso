// Kanso — store_sets_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados3, IntAnys, any;

import '../helpers/property.dart';
import '../helpers/rgb_png.dart';
import '../helpers/store_sets.dart';

void main() {
  group('store sets', () {
    const expected = <String, Size>{
      'ios-iphone-6.9': Size(1320, 2868),
      'ios-iphone-6.5': Size(1284, 2778),
      'ios-ipad-13': Size(2064, 2752),
      'android-phone': Size(1080, 2160),
      'android-tablet-7': Size(1200, 1920),
      'android-tablet-10': Size(1600, 2560),
    };

    test('the sizes are the stores', () {
      expect(<String, Size>{
        for (final s in storeSets) s.name: s.pixels,
      }, expected);
      expect(featureGraphicSize, const Size(1024, 500));
      expect(playIconSize, const Size(512, 512));
    });

    test('a size is a whole number of logical pixels', () {
      for (final set in storeSets) {
        final logical = set.logical;
        expect(logical.width * set.devicePixelRatio, set.pixels.width);
        expect(logical.height * set.devicePixelRatio, set.pixels.height);
      }
    });

    test('Play pictures obey its limits', () {
      for (final set in storeSets.where((s) => s.name.startsWith('android'))) {
        final p = set.pixels;
        expect(p.shortestSide, inInclusiveRange(320, 3840));
        expect(p.longestSide, inInclusiveRange(320, 3840));
        expect(p.longestSide / p.shortestSide, lessThanOrEqualTo(2));
        if (set.name.contains('tablet')) {
          expect(p.shortestSide, greaterThanOrEqualTo(1080));
        }
      }
    });

    test('iOS sets are drawn as iOS and Android sets as Android', () {
      for (final set in storeSets) {
        expect(
          set.platform.name,
          set.name.startsWith('ios') ? 'iOS' : 'android',
        );
      }
    });

    test('the sets are the ones mobile/store/README.md documents', () {
      final readme = File('store/README.md').readAsStringSync();
      final documented = RegExp(
        r'^\| `((?:ios|android)-[a-z0-9.\-]+)`',
        multiLine: true,
      ).allMatches(readme).map((m) => m.group(1)!).toSet().toList();
      expect(documented, storeSets.map((s) => s.name).toList());
    });
  });

  group('rgb png', () {
    Glados3<int, int, int>(
      any.intInRange(1, 12),
      any.intInRange(1, 12),
      any.intInRange(0, 1 << 30),
      kansoExploreConfig(),
    ).test('decodes back to the same pixels, without alpha', (w, h, seed) {
      final random = Random(seed);
      final rgba = Uint8List(w * h * 4);
      for (var i = 0; i < rgba.length; i += 4) {
        rgba[i] = random.nextInt(256);
        rgba[i + 1] = random.nextInt(256);
        rgba[i + 2] = random.nextInt(256);
        rgba[i + 3] = 255;
      }
      final png = encodeRgbPng(w, h, rgba);
      final header = readPngHeader(png);
      expect(header.colorType, pngColorTypeRgb);
      expect(header.hasAlpha, isFalse);
      expect((header.width, header.height), (w, h));
      final pixels = decodeRgbPng(png);
      for (var i = 0; i < w * h; i++) {
        expect(
          pixels.sublist(i * 3, i * 3 + 3),
          rgba.sublist(i * 4, i * 4 + 3),
        );
      }
    });

    Glados<int>(any.intInRange(0, 256), kansoExploreConfig()).test(
      'a translucent pixel is composited onto white',
      (alpha) {
        final png = encodeRgbPng(
          1,
          1,
          Uint8List.fromList(<int>[0, 0, 0, alpha]),
        );
        final expected = ((0 * alpha + 255 * (255 - alpha)) / 255).round();
        expect(decodeRgbPng(png), <int>[expected, expected, expected]);
      },
    );
  });
}
