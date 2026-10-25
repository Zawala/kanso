// Kanso — kanso_typography.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import 'kanso_colors.dart';

/// Kanso's type scale, in Instrument Sans (text) and JetBrains Mono (code).
///
/// The sizes are tuned for a phone, at the density of a compact notes app.
/// Nothing clamps the text scale: dynamic type is respected everywhere.
///
/// A [ThemeExtension] because the framework requires `copyWith` and `lerp`
/// overrides; the styles are bound to the colours they are built with.
@immutable
class KansoTypography extends ThemeExtension<KansoTypography> {
  const KansoTypography({
    required this.largeTitle,
    required this.noteTitle,
    required this.barTitle,
    required this.heading1,
    required this.heading2,
    required this.listTitle,
    required this.body,
    required this.secondary,
    required this.label,
    required this.meta,
    required this.sectionLabel,
    required this.mono,
    required this.monoSmall,
  });

  /// The styles for [colors]: text in `text`, secondary lines in
  /// `textSecondary`, section labels in `textMuted`.
  factory KansoTypography.fromColors(KansoColors colors) {
    return KansoTypography(
      largeTitle: TextStyle(
        fontFamily: sansFamily,
        fontSize: 22,
        fontWeight: FontWeight.w700,
        height: 1.25,
        color: colors.text,
      ),
      noteTitle: TextStyle(
        fontFamily: sansFamily,
        fontSize: 24,
        fontWeight: FontWeight.w700,
        height: 1.25,
        color: colors.text,
      ),
      barTitle: TextStyle(
        fontFamily: sansFamily,
        fontSize: 17,
        fontWeight: FontWeight.w600,
        height: 1.3,
        color: colors.text,
      ),
      heading1: TextStyle(
        fontFamily: sansFamily,
        fontSize: 20,
        fontWeight: FontWeight.w700,
        height: 1.3,
        color: colors.text,
      ),
      heading2: TextStyle(
        fontFamily: sansFamily,
        fontSize: 17,
        fontWeight: FontWeight.w600,
        height: 1.3,
        color: colors.text,
      ),
      listTitle: TextStyle(
        fontFamily: sansFamily,
        fontSize: 15,
        fontWeight: FontWeight.w600,
        height: 1.3,
        color: colors.text,
      ),
      body: TextStyle(
        fontFamily: sansFamily,
        fontSize: 15,
        fontWeight: FontWeight.w400,
        height: 1.45,
        color: colors.text,
      ),
      secondary: TextStyle(
        fontFamily: sansFamily,
        fontSize: 13,
        fontWeight: FontWeight.w400,
        height: 1.35,
        color: colors.textSecondary,
      ),
      label: TextStyle(
        fontFamily: sansFamily,
        fontSize: 13,
        fontWeight: FontWeight.w500,
        height: 1.3,
        color: colors.textSecondary,
      ),
      meta: TextStyle(
        fontFamily: sansFamily,
        fontSize: 12,
        fontWeight: FontWeight.w400,
        height: 1.3,
        color: colors.textMuted,
      ),
      sectionLabel: TextStyle(
        fontFamily: sansFamily,
        fontSize: 12,
        fontWeight: FontWeight.w500,
        height: 1.3,
        letterSpacing: 0.2,
        color: colors.textMuted,
      ),
      mono: TextStyle(
        fontFamily: monoFamily,
        fontSize: 13.5,
        fontWeight: FontWeight.w400,
        height: 1.5,
        color: colors.text,
      ),
      monoSmall: TextStyle(
        fontFamily: monoFamily,
        fontSize: 11.5,
        fontWeight: FontWeight.w400,
        height: 1.55,
        color: colors.text,
      ),
    );
  }

  /// The family of the interface and note text.
  static const String sansFamily = 'Instrument Sans';

  /// The family of code.
  static const String monoFamily = 'JetBrains Mono';

  /// A page's heading: 22, bold.
  final TextStyle largeTitle;

  /// The title of the open note: 24, bold.
  final TextStyle noteTitle;

  /// A top bar's title: 17, semibold.
  final TextStyle barTitle;

  /// A first-level heading inside a note: 20, bold.
  final TextStyle heading1;

  /// A second-level heading inside a note: 17, semibold.
  final TextStyle heading2;

  /// A row's title in a list: 15, semibold.
  final TextStyle listTitle;

  /// Running text: 15.
  final TextStyle body;

  /// A row's second line, captions and hints: 13.
  final TextStyle secondary;

  /// A control's label (chips, buttons' small text): 13, medium.
  final TextStyle label;

  /// A quiet line of facts under a title: 12, muted.
  final TextStyle meta;

  /// A section's label above its rows: 12, medium, muted.
  final TextStyle sectionLabel;

  /// Code, in the editor and in blocks: 13.5.
  final TextStyle mono;

  /// Code in a small preview: 11.5.
  final TextStyle monoSmall;

  /// A full Material [TextTheme] built from these styles, so no widget
  /// falls back to a platform font.
  TextTheme toTextTheme() {
    return TextTheme(
      displayLarge: largeTitle,
      displayMedium: largeTitle,
      displaySmall: largeTitle,
      headlineLarge: noteTitle,
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

  @override
  KansoTypography copyWith({
    TextStyle? largeTitle,
    TextStyle? noteTitle,
    TextStyle? barTitle,
    TextStyle? heading1,
    TextStyle? heading2,
    TextStyle? listTitle,
    TextStyle? body,
    TextStyle? secondary,
    TextStyle? label,
    TextStyle? meta,
    TextStyle? sectionLabel,
    TextStyle? mono,
    TextStyle? monoSmall,
  }) {
    return KansoTypography(
      largeTitle: largeTitle ?? this.largeTitle,
      noteTitle: noteTitle ?? this.noteTitle,
      barTitle: barTitle ?? this.barTitle,
      heading1: heading1 ?? this.heading1,
      heading2: heading2 ?? this.heading2,
      listTitle: listTitle ?? this.listTitle,
      body: body ?? this.body,
      secondary: secondary ?? this.secondary,
      label: label ?? this.label,
      meta: meta ?? this.meta,
      sectionLabel: sectionLabel ?? this.sectionLabel,
      mono: mono ?? this.mono,
      monoSmall: monoSmall ?? this.monoSmall,
    );
  }

  @override
  KansoTypography lerp(ThemeExtension<KansoTypography>? other, double t) {
    if (other is! KansoTypography) {
      return this;
    }
    return KansoTypography(
      largeTitle: TextStyle.lerp(largeTitle, other.largeTitle, t)!,
      noteTitle: TextStyle.lerp(noteTitle, other.noteTitle, t)!,
      barTitle: TextStyle.lerp(barTitle, other.barTitle, t)!,
      heading1: TextStyle.lerp(heading1, other.heading1, t)!,
      heading2: TextStyle.lerp(heading2, other.heading2, t)!,
      listTitle: TextStyle.lerp(listTitle, other.listTitle, t)!,
      body: TextStyle.lerp(body, other.body, t)!,
      secondary: TextStyle.lerp(secondary, other.secondary, t)!,
      label: TextStyle.lerp(label, other.label, t)!,
      meta: TextStyle.lerp(meta, other.meta, t)!,
      sectionLabel: TextStyle.lerp(sectionLabel, other.sectionLabel, t)!,
      mono: TextStyle.lerp(mono, other.mono, t)!,
      monoSmall: TextStyle.lerp(monoSmall, other.monoSmall, t)!,
    );
  }
}
