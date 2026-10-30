// Kanso — kanso_typography.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../models/data/kanso_font_category.dart';
import '../models/data/kanso_font_family.dart';
import '../models/data/kanso_fonts.dart';
import 'kanso_colors.dart';

/// Kanso's type scale.
///
/// Every style follows the person's choices in [KansoFonts]: the interface
/// font (Instrument Sans by default) for the interface, and the editor,
/// preview and code fonts for the text of notes, each a bundled family or a
/// downloaded Google Font. Note text has its own continuous scales.
///
/// The sizes are tuned for a phone, at the density of a compact notes app.
/// Nothing clamps the text scale: dynamic type is respected everywhere.
///
/// The styles are bound to the colours they are built with and handed down
/// through Kanso's own theme scope, not a framework theme extension.
@immutable
class KansoTypography {
  const KansoTypography({
    required this.largeTitle,
    required this.barTitle,
    required this.heading1,
    required this.heading2,
    required this.listTitle,
    required this.body,
    required this.secondary,
    required this.label,
    required this.meta,
    required this.sectionLabel,
    required this.editorTitle,
    required this.editorHeading1,
    required this.editorHeading2,
    required this.editorBody,
    required this.previewTitle,
    required this.previewHeading1,
    required this.previewHeading2,
    required this.previewBody,
    required this.mono,
    required this.monoSmall,
  });

  /// The styles for [colors]: text in `text`, secondary lines in
  /// `textSecondary`, section labels in `textMuted`.
  ///
  /// The note styles are set in [fonts] and scaled by [textSize]. The
  /// interface styles follow [interfaceTextSize].
  factory KansoTypography.fromColors(
    KansoColors colors, {
    KansoFonts fonts = const KansoFonts(),
    double textSize = 1,
    double? previewTextSize,
    double interfaceTextSize = 1,
  }) {
    final interfaceFamily =
        fonts.interfaceGoogleFont ?? fonts.interfaceFont.family ?? systemFamily;
    TextStyle ui(
      double size,
      FontWeight weight,
      double height,
      Color color, {
      double? letterSpacing,
    }) {
      return TextStyle(
        fontFamily: interfaceFamily,
        fontSize: size * interfaceTextSize,
        fontWeight: weight,
        height: height,
        letterSpacing: letterSpacing == null
            ? null
            : letterSpacing * interfaceTextSize,
        color: color,
      );
    }

    TextStyle note(
      KansoFontFamily font,
      String? googleFont,
      double scale,
      double size,
      FontWeight weight,
      double height,
    ) {
      return fontStyle(
        font,
        size: size * scale,
        weight: weight,
        height: height,
        color: colors.text,
      ).copyWith(fontFamily: googleFont);
    }

    final editor = fonts.editorFont;
    final preview = fonts.previewFont;
    return KansoTypography(
      largeTitle: ui(
        32,
        FontWeight.w700,
        1.15,
        colors.text,
        letterSpacing: -0.8,
      ),
      barTitle: ui(18, FontWeight.w600, 1.3, colors.text),
      heading1: ui(22, FontWeight.w700, 1.3, colors.text),
      heading2: ui(18, FontWeight.w600, 1.3, colors.text),
      listTitle: ui(16, FontWeight.w600, 1.3, colors.text),
      body: ui(16, FontWeight.w400, 1.45, colors.text),
      secondary: ui(14, FontWeight.w400, 1.35, colors.textSecondary),
      label: ui(14, FontWeight.w500, 1.3, colors.textSecondary),
      meta: ui(13, FontWeight.w400, 1.3, colors.textMuted),
      sectionLabel: ui(
        13,
        FontWeight.w500,
        1.3,
        colors.textMuted,
        letterSpacing: 0.2,
      ),
      editorTitle: note(
        editor,
        fonts.editorGoogleFont,
        textSize,
        28,
        FontWeight.w700,
        1.25,
      ),
      editorHeading1: note(
        editor,
        fonts.editorGoogleFont,
        textSize,
        23,
        FontWeight.w700,
        1.3,
      ),
      editorHeading2: note(
        editor,
        fonts.editorGoogleFont,
        textSize,
        20,
        FontWeight.w600,
        1.3,
      ),
      editorBody: note(
        editor,
        fonts.editorGoogleFont,
        textSize,
        18,
        FontWeight.w400,
        leading(editor),
      ),
      previewTitle: note(
        preview,
        fonts.previewGoogleFont,
        previewTextSize ?? textSize,
        28,
        FontWeight.w700,
        1.25,
      ),
      previewHeading1: note(
        preview,
        fonts.previewGoogleFont,
        previewTextSize ?? textSize,
        23,
        FontWeight.w700,
        1.3,
      ),
      previewHeading2: note(
        preview,
        fonts.previewGoogleFont,
        previewTextSize ?? textSize,
        20,
        FontWeight.w600,
        1.3,
      ),
      previewBody: note(
        preview,
        fonts.previewGoogleFont,
        previewTextSize ?? textSize,
        18,
        FontWeight.w400,
        leading(preview),
      ),
      mono: note(
        fonts.codeFont,
        fonts.codeGoogleFont,
        textSize,
        13.5,
        FontWeight.w400,
        1.5,
      ),
      // Theme previews are interface, not note text: not scaled.
      monoSmall: fontStyle(
        fonts.codeFont,
        size: 11.5 * interfaceTextSize,
        weight: FontWeight.w400,
        height: 1.55,
        color: colors.text,
      ).copyWith(fontFamily: fonts.codeGoogleFont),
    );
  }

  /// A style set in [font].
  ///
  /// Weights are plain `FontWeight`s: the bundled variable fonts draw the
  /// weight asked for. [KansoFontFamily.system] is the device's own font:
  /// San Francisco on iOS and the system font on Android.
  static TextStyle fontStyle(
    KansoFontFamily font, {
    required double size,
    required FontWeight weight,
    required double height,
    required Color color,
  }) {
    return TextStyle(
      fontFamily: font.family ?? systemFamily,
      fontSize: size,
      fontWeight: weight,
      height: height,
      color: color,
    );
  }

  /// The line height, as a multiple of the size, that suits a running text
  /// in [font]: serifs and monospaced families want a little more air.
  static double leading(KansoFontFamily font) {
    return switch (font.category) {
      KansoFontCategory.sans || KansoFontCategory.system => 1.45,
      KansoFontCategory.serif => 1.55,
      KansoFontCategory.mono => 1.6,
    };
  }

  /// The family name that makes Flutter use the device's own font.
  static String get systemFamily {
    return defaultTargetPlatform == TargetPlatform.iOS
        ? 'CupertinoSystemText'
        : 'sans-serif';
  }

  /// Kanso's own interface family, the default interface font.
  static const String sansFamily = 'Instrument Sans';

  /// A page's heading, at the top of what scrolls: 32, bold, set tight.
  final TextStyle largeTitle;

  /// A top bar's title: 18, semibold.
  final TextStyle barTitle;

  /// A first-level heading inside a note: 22, bold.
  final TextStyle heading1;

  /// A second-level heading inside a note: 18, semibold.
  final TextStyle heading2;

  /// A row's title in a list: 16, semibold.
  final TextStyle listTitle;

  /// Running text: 16.
  final TextStyle body;

  /// A row's second line, captions and hints: 14.
  final TextStyle secondary;

  /// A control's label (chips, buttons' small text): 14, medium.
  final TextStyle label;

  /// A quiet line of facts under a title: 13, muted.
  final TextStyle meta;

  /// A section's label above its rows: 13, medium, muted.
  final TextStyle sectionLabel;

  /// The title of a note being edited, in the editor font: 28, bold.
  final TextStyle editorTitle;

  /// A first-level heading in the editor: 23, bold.
  final TextStyle editorHeading1;

  /// A second-level heading in the editor: 20, semibold.
  final TextStyle editorHeading2;

  /// Text in the editor: 18.
  final TextStyle editorBody;

  /// The title of a note being read, in the preview font: 28, bold.
  final TextStyle previewTitle;

  /// A first-level heading in a rendered note: 23, bold.
  final TextStyle previewHeading1;

  /// A second-level heading in a rendered note: 20, semibold.
  final TextStyle previewHeading2;

  /// Running text of a rendered note: 18.
  final TextStyle previewBody;

  /// Code in a note, in the code font: 13.5.
  final TextStyle mono;

  /// Code in a small theme preview, in the code font: 11.5, scaled with the
  /// interface because it appears in interface chrome.
  final TextStyle monoSmall;

  /// A full Material [TextTheme] built from these styles, so no widget
  /// falls back to a platform font.
  TextTheme toTextTheme() {
    return TextTheme(
      displayLarge: largeTitle,
      displayMedium: largeTitle,
      displaySmall: largeTitle,
      headlineLarge: largeTitle,
      headlineMedium: largeTitle,
      headlineSmall: largeTitle,
      titleLarge: barTitle,
      titleMedium: listTitle,
      titleSmall: listTitle,
      bodyLarge: body,
      bodyMedium: body,
      bodySmall: secondary,
      labelLarge: listTitle,
      labelMedium: secondary.copyWith(fontWeight: FontWeight.w500),
      labelSmall: sectionLabel,
    );
  }

  KansoTypography copyWith({
    TextStyle? largeTitle,
    TextStyle? barTitle,
    TextStyle? heading1,
    TextStyle? heading2,
    TextStyle? listTitle,
    TextStyle? body,
    TextStyle? secondary,
    TextStyle? label,
    TextStyle? meta,
    TextStyle? sectionLabel,
    TextStyle? editorTitle,
    TextStyle? editorHeading1,
    TextStyle? editorHeading2,
    TextStyle? editorBody,
    TextStyle? previewTitle,
    TextStyle? previewHeading1,
    TextStyle? previewHeading2,
    TextStyle? previewBody,
    TextStyle? mono,
    TextStyle? monoSmall,
  }) {
    return KansoTypography(
      largeTitle: largeTitle ?? this.largeTitle,
      barTitle: barTitle ?? this.barTitle,
      heading1: heading1 ?? this.heading1,
      heading2: heading2 ?? this.heading2,
      listTitle: listTitle ?? this.listTitle,
      body: body ?? this.body,
      secondary: secondary ?? this.secondary,
      label: label ?? this.label,
      meta: meta ?? this.meta,
      sectionLabel: sectionLabel ?? this.sectionLabel,
      editorTitle: editorTitle ?? this.editorTitle,
      editorHeading1: editorHeading1 ?? this.editorHeading1,
      editorHeading2: editorHeading2 ?? this.editorHeading2,
      editorBody: editorBody ?? this.editorBody,
      previewTitle: previewTitle ?? this.previewTitle,
      previewHeading1: previewHeading1 ?? this.previewHeading1,
      previewHeading2: previewHeading2 ?? this.previewHeading2,
      previewBody: previewBody ?? this.previewBody,
      mono: mono ?? this.mono,
      monoSmall: monoSmall ?? this.monoSmall,
    );
  }

  KansoTypography lerp(KansoTypography other, double t) {
    return KansoTypography(
      largeTitle: TextStyle.lerp(largeTitle, other.largeTitle, t)!,
      barTitle: TextStyle.lerp(barTitle, other.barTitle, t)!,
      heading1: TextStyle.lerp(heading1, other.heading1, t)!,
      heading2: TextStyle.lerp(heading2, other.heading2, t)!,
      listTitle: TextStyle.lerp(listTitle, other.listTitle, t)!,
      body: TextStyle.lerp(body, other.body, t)!,
      secondary: TextStyle.lerp(secondary, other.secondary, t)!,
      label: TextStyle.lerp(label, other.label, t)!,
      meta: TextStyle.lerp(meta, other.meta, t)!,
      sectionLabel: TextStyle.lerp(sectionLabel, other.sectionLabel, t)!,
      editorTitle: TextStyle.lerp(editorTitle, other.editorTitle, t)!,
      editorHeading1: TextStyle.lerp(editorHeading1, other.editorHeading1, t)!,
      editorHeading2: TextStyle.lerp(editorHeading2, other.editorHeading2, t)!,
      editorBody: TextStyle.lerp(editorBody, other.editorBody, t)!,
      previewTitle: TextStyle.lerp(previewTitle, other.previewTitle, t)!,
      previewHeading1: TextStyle.lerp(
        previewHeading1,
        other.previewHeading1,
        t,
      )!,
      previewHeading2: TextStyle.lerp(
        previewHeading2,
        other.previewHeading2,
        t,
      )!,
      previewBody: TextStyle.lerp(previewBody, other.previewBody, t)!,
      mono: TextStyle.lerp(mono, other.mono, t)!,
      monoSmall: TextStyle.lerp(monoSmall, other.monoSmall, t)!,
    );
  }
}
