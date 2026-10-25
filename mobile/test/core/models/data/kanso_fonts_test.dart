// Kanso — kanso_fonts_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, any;
import 'package:kanso/core/models/data/kanso_font_category.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';

import '../../../helpers/property.dart';

void main() {
  group('KansoFonts', () {
    test("defaults to Kanso's own sans for text and mono for code", () {
      const fonts = KansoFonts();
      expect(fonts.editorFont, KansoFontFamily.instrumentSans);
      expect(fonts.previewFont, KansoFontFamily.instrumentSans);
      expect(fonts.codeFont, KansoFontFamily.jetBrainsMono);
    });

    test('the default summary names the editor and code fonts', () {
      expect(
        const KansoFonts().summary(),
        'Instrument Sans \u00b7 JetBrains Mono',
      );
    });

    test('a different preview font joins the summary in the middle', () {
      const fonts = KansoFonts(previewFont: KansoFontFamily.sourceSerif4);
      expect(
        fonts.summary(),
        'Instrument Sans \u00b7 Source Serif 4 \u00b7 JetBrains Mono',
      );
    });

    test('the device font is named with the label it is given', () {
      const fonts = KansoFonts(
        editorFont: KansoFontFamily.system,
        previewFont: KansoFontFamily.system,
      );
      expect(
        fonts.summary(systemLabel: 'Device'),
        'Device \u00b7 JetBrains Mono',
      );
    });

    Glados<KansoFonts>(any.fonts, kansoExploreConfig()).test(
      'the summary is never empty and names every distinct choice',
      (fonts) {
        final summary = fonts.summary();
        expect(summary, isNotEmpty);
        expect(summary, contains(fonts.codeFont.displayName));
        expect(summary, contains(fonts.editorFont.displayName));
        expect(summary, contains(fonts.previewFont.displayName));
      },
    );
  });

  group('KansoFontFamily', () {
    test("only the device's font is not bundled", () {
      for (final font in KansoFontFamily.values) {
        expect(font.bundled, font != KansoFontFamily.system);
        expect(font.displayName, isNotEmpty);
      }
    });

    test('code choices are all monospaced', () {
      expect(KansoFontFamily.codeChoices, isNotEmpty);
      expect(
        KansoFontFamily.codeChoices.every((font) => font.monospaced),
        isTrue,
      );
    });

    test('preview choices are never monospaced; editor choices cover all', () {
      expect(
        KansoFontFamily.previewChoices.any((font) => font.monospaced),
        isFalse,
      );
      expect(
        KansoFontFamily.editorChoices.toSet(),
        KansoFontFamily.values.toSet(),
      );
    });

    test('has the families the settings promise, by category', () {
      Iterable<String> named(KansoFontCategory category) => KansoFontFamily
          .values
          .where((font) => font.category == category)
          .map((font) => font.displayName);
      expect(named(KansoFontCategory.sans), <String>[
        'Instrument Sans',
        'Inter',
      ]);
      expect(
        named(KansoFontCategory.serif),
        <String>['Source Serif 4', 'Literata'],
      );
      expect(
        named(KansoFontCategory.mono),
        <String>['JetBrains Mono', 'IBM Plex Mono'],
      );
    });
  });

  group('KansoTextSize', () {
    test('Default draws note text at its designed size', () {
      expect(KansoTextSize.standard.scaled(15), 15);
      expect(KansoTextSize.standard.withDeviceScale(1.3), 1.3);
    });

    test('the sizes are in increasing order', () {
      final scales = KansoTextSize.values.map((size) => size.scale).toList();
      expect(scales, orderedEquals(<double>[...scales]..sort()));
    });

    Glados2<KansoTextSize, KansoTextSize>(
      any.textSize,
      any.textSize,
      kansoExploreConfig(),
    ).test('a larger setting never draws smaller (monotonic)', (a, b) {
      if (a.index <= b.index) {
        expect(a.scaled(15), lessThanOrEqualTo(b.scaled(15)));
      }
    });

    Glados2<KansoTextSize, double>(
      any.textSize,
      any.deviceTextScale,
      kansoExploreConfig(),
    ).test('the device scale is applied in full, never clamped', (
      size,
      device,
    ) {
      final drawn = size.withDeviceScale(device);
      expect(drawn, closeTo(device * size.scale, 1e-9));
      expect(drawn, greaterThan(0));
      // From Default up the setting only adds to the device's own scale.
      if (size.index >= KansoTextSize.standard.index) {
        expect(drawn, greaterThanOrEqualTo(device));
      }
    });

    Glados2<double, double>(
      any.deviceTextScale,
      any.deviceTextScale,
      kansoExploreConfig(),
    ).test('a larger device scale gives larger text at every setting', (a, b) {
      for (final size in KansoTextSize.values) {
        if (a <= b) {
          expect(
            size.withDeviceScale(a),
            lessThanOrEqualTo(size.withDeviceScale(b)),
          );
        }
      }
    });
  });
}
