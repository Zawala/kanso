// Kanso — kanso_typography_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, any;
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_typography.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';

import '../../helpers/property.dart';

void main() {
  KansoTypography build({
    KansoFonts fonts = const KansoFonts(),
    KansoTextSize size = KansoTextSize.standard,
  }) {
    return KansoTypography.fromColors(
      KansoColors.light,
      fonts: fonts,
      textSize: size,
    );
  }

  List<TextStyle> interfaceStyles(KansoTypography t) => <TextStyle>[
    t.largeTitle,
    t.barTitle,
    t.heading1,
    t.heading2,
    t.listTitle,
    t.body,
    t.secondary,
    t.label,
    t.meta,
    t.sectionLabel,
  ];

  group('KansoTypography fonts', () {
    test(
      'the default note styles are in Instrument Sans and JetBrains Mono',
      () {
        final t = build();
        expect(t.editorBody.fontFamily, 'Instrument Sans');
        expect(t.previewBody.fontFamily, 'Instrument Sans');
        expect(t.mono.fontFamily, 'JetBrains Mono');
      },
    );

    test('each note style follows its own font', () {
      final t = build(
        fonts: const KansoFonts(
          editorFont: KansoFontFamily.inter,
          previewFont: KansoFontFamily.sourceSerif4,
          codeFont: KansoFontFamily.ibmPlexMono,
        ),
      );
      expect(t.editorTitle.fontFamily, 'Inter');
      expect(t.editorHeading1.fontFamily, 'Inter');
      expect(t.editorBody.fontFamily, 'Inter');
      expect(t.previewTitle.fontFamily, 'Source Serif 4');
      expect(t.previewHeading2.fontFamily, 'Source Serif 4');
      expect(t.previewBody.fontFamily, 'Source Serif 4');
      expect(t.mono.fontFamily, 'IBM Plex Mono');
      expect(t.monoSmall.fontFamily, 'IBM Plex Mono');
    });

    test('weights are plain font weights, which the variable fonts draw', () {
      final t = build(
        fonts: const KansoFonts(previewFont: KansoFontFamily.literata),
      );
      expect(t.previewTitle.fontWeight, FontWeight.w700);
      expect(t.previewHeading2.fontWeight, FontWeight.w600);
      expect(t.previewBody.fontWeight, FontWeight.w400);
      expect(t.previewBody.fontVariations, isNull);
    });

    test(
      "the device font is a family the platform resolves, never Kanso's",
      () {
        final t = build(
          fonts: const KansoFonts(
            editorFont: KansoFontFamily.system,
            previewFont: KansoFontFamily.system,
          ),
        );
        expect(t.editorBody.fontFamily, KansoTypography.systemFamily);
        expect(t.previewBody.fontFamily, KansoTypography.systemFamily);
        expect(t.previewBody.fontFamily, isNot('Instrument Sans'));
      },
    );

    test('serifs and monospaced fonts get more line height than sans', () {
      expect(
        KansoTypography.leading(KansoFontFamily.sourceSerif4),
        greaterThan(KansoTypography.leading(KansoFontFamily.inter)),
      );
      expect(
        KansoTypography.leading(KansoFontFamily.jetBrainsMono),
        greaterThan(KansoTypography.leading(KansoFontFamily.sourceSerif4)),
      );
    });

    Glados2<KansoFonts, KansoTextSize>(
      any.fonts,
      any.textSize,
      kansoExploreConfig(),
    ).test('the interface styles never change with fonts or text size', (
      fonts,
      size,
    ) {
      final reference = interfaceStyles(build());
      final styles = interfaceStyles(build(fonts: fonts, size: size));
      expect(styles, orderedEquals(reference));
    });
  });

  group('KansoTypography text size', () {
    test('scales every note style and nothing else', () {
      final base = build();
      final large = build(size: KansoTextSize.large);
      final pairs = <(TextStyle, TextStyle)>[
        (base.editorTitle, large.editorTitle),
        (base.editorHeading1, large.editorHeading1),
        (base.editorHeading2, large.editorHeading2),
        (base.editorBody, large.editorBody),
        (base.previewTitle, large.previewTitle),
        (base.previewHeading1, large.previewHeading1),
        (base.previewHeading2, large.previewHeading2),
        (base.previewBody, large.previewBody),
        (base.mono, large.mono),
      ];
      for (final (a, b) in pairs) {
        expect(b.fontSize, closeTo(a.fontSize! * 1.15, 1e-9));
      }
      // Theme previews are interface, not note text.
      expect(large.monoSmall.fontSize, base.monoSmall.fontSize);
    });

    test('Default leaves the designed sizes', () {
      final t = build();
      expect(t.editorBody.fontSize, 15);
      expect(t.previewTitle.fontSize, 24);
      expect(t.mono.fontSize, 13.5);
    });

    test('never touches the text scaler: no clamp is set on any style', () {
      final t = build(size: KansoTextSize.larger);
      // A TextStyle has no scale cap; sizes grow by the multiplier only.
      expect(t.previewBody.fontSize, closeTo(15 * 1.3, 1e-9));
    });
  });
}
