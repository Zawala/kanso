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
    required this.listTitle,
    required this.body,
    required this.secondary,
    required this.sectionLabel,
    required this.mono,
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

  /// A row's title in a list: 15, semibold.
  final TextStyle listTitle;

  /// Running text: 15.
  final TextStyle body;

  /// A row's second line, captions and hints: 13.
  final TextStyle secondary;

  /// A section's label above its rows: 12, medium, muted.
  final TextStyle sectionLabel;

  /// Code, in the editor and in blocks: 13.5.
  final TextStyle mono;

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
      titleLarge: largeTitle,
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
    TextStyle? listTitle,
    TextStyle? body,
    TextStyle? secondary,
    TextStyle? sectionLabel,
    TextStyle? mono,
  }) {
    return KansoTypography(
      largeTitle: largeTitle ?? this.largeTitle,
      noteTitle: noteTitle ?? this.noteTitle,
      listTitle: listTitle ?? this.listTitle,
      body: body ?? this.body,
      secondary: secondary ?? this.secondary,
      sectionLabel: sectionLabel ?? this.sectionLabel,
      mono: mono ?? this.mono,
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
      listTitle: TextStyle.lerp(listTitle, other.listTitle, t)!,
      body: TextStyle.lerp(body, other.body, t)!,
      secondary: TextStyle.lerp(secondary, other.secondary, t)!,
      sectionLabel: TextStyle.lerp(sectionLabel, other.sectionLabel, t)!,
      mono: TextStyle.lerp(mono, other.mono, t)!,
    );
  }
}
