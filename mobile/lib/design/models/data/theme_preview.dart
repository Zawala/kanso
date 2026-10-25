// Kanso — theme_preview.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:freezed_annotation/freezed_annotation.dart';

import 'theme_appearance.dart';

part 'theme_preview.freezed.dart';

/// What a theme card needs of a Kanso theme file: who made it and the
/// colours that draw a few lines of Markdown.
///
/// The values are copied from the theme files in `themes/`, and a test
/// reads the files to keep them equal. [quote] is the quote colour of the
/// file's syntax, or its `marker` colour when it has none; [strong] and
/// [emphasis] are the colour of the bold and italic text, or its `text`
/// colour when they have none.
@freezed
abstract class ThemePreview with _$ThemePreview {
  const factory ThemePreview({
    required String file,
    required String name,
    required String author,
    required String license,
    required String source,
    required ThemeAppearance appearance,
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
    @Default(false) bool builtIn,
  }) = _ThemePreview;

  const ThemePreview._();

  /// The `owner/repo` of the theme's source, from its GitHub address.
  String get repository {
    final segments = Uri.parse(source).pathSegments;
    return segments.take(2).join('/');
  }
}
