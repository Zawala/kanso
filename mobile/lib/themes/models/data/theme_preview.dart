// Kanso — theme_preview.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

part 'theme_preview.freezed.dart';

/// What a theme card needs of a theme: its name and the colours that draw a
/// few lines of Markdown, taken from the theme's tokens
/// (`themePreviewFromBuiltin`).
///
/// [quote] is the quote colour of the theme's syntax, or its marker colour
/// when it has none; [strong] and [emphasis] are the colour of the bold and
/// italic text, or the text colour when the theme has none.
@freezed
abstract class ThemePreview with _$ThemePreview {
  const factory ThemePreview({
    /// The theme file's name without `.json`: `nord`.
    required String id,
    required String name,

    /// Whether the theme is for a dark interface.
    required bool dark,
    required Color editor,
    required Color text,
    required Color accent,
    required Color heading,
    required Color marker,
    required Color quote,
    required Color link,
    required Color url,
    required Color code,
    required Color strong,
    required Color emphasis,
    @Default(true) bool builtIn,
  }) = _ThemePreview;
}
