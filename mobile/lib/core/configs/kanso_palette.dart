// Kanso — kanso_palette.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../bridge/api/types.dart';
import '../models/data/kanso_syntax_style.dart';
import 'kanso_colors.dart';
import 'kanso_syntax_colors.dart';

part 'kanso_palette.freezed.dart';

/// Everything a Kanso theme file decides, as the app draws it: the
/// interface colours, the syntax colours and styles, and the colours the
/// preview and drawings are made in.
///
/// Palettes of the built-in themes come from `paletteFromTokens`, the one
/// mapping of the Rust core's theme tokens. [kansoLight] and [kansoDark] are
/// the same two palettes written out, for the first frame and for tests; a
/// test proves them equal to the mapping.
@freezed
abstract class KansoPalette with _$KansoPalette {
  const factory KansoPalette({
    /// The theme file's name without `.json`: `nord`, `kanso-light`.
    required String id,

    /// The name people see: `Kanso Light`.
    required String name,

    /// Whether the theme is a dark one.
    required bool dark,

    /// The interface colours.
    required KansoColors colors,

    /// The Markdown and code colours.
    required KansoSyntaxColors syntax,

    /// The syntax tokens that are italic or bold, by the theme file's token
    /// name.
    required Map<String, KansoSyntaxStyle> syntaxStyles,

    /// The colours the preview and drawings are made in.
    required PaletteDto preview,
  }) = _KansoPalette;

  const KansoPalette._();

  /// Kanso Light, from `themes/kanso-light.json`.
  static const KansoPalette kansoLight = KansoPalette(
    id: 'kanso-light',
    name: 'Kanso Light',
    dark: false,
    colors: KansoColors.light,
    syntax: KansoSyntaxColors.light,
    syntaxStyles: <String, KansoSyntaxStyle>{
      'comment': KansoSyntaxStyle(italic: true),
      'comment_doc': KansoSyntaxStyle(italic: true),
      'emphasis': KansoSyntaxStyle(italic: true),
      'heading': KansoSyntaxStyle(weight: 700),
      'strong': KansoSyntaxStyle(weight: 700),
    },
    preview: PaletteDto(
      dark: false,
      background: 0xFCFAF4,
      text: 0x26241F,
      muted: 0x857C6E,
      accent: 0x4D6684,
      border: 0xDDD6C9,
      surface: 0xF4F1EA,
      quote: 0x655F55,
      taskDone: 0x655F55,
      inlineCodeBackground: 0xF4F1EA,
    ),
  );

  /// Kanso Dark, from `themes/kanso-dark.json`.
  static const KansoPalette kansoDark = KansoPalette(
    id: 'kanso-dark',
    name: 'Kanso Dark',
    dark: true,
    colors: KansoColors.dark,
    syntax: KansoSyntaxColors.dark,
    syntaxStyles: <String, KansoSyntaxStyle>{
      'comment': KansoSyntaxStyle(italic: true),
      'comment_doc': KansoSyntaxStyle(italic: true),
      'emphasis': KansoSyntaxStyle(italic: true),
      'heading': KansoSyntaxStyle(weight: 700),
      'strong': KansoSyntaxStyle(weight: 700),
    },
    preview: PaletteDto(
      dark: true,
      background: 0x282824,
      text: 0xF1EFE7,
      muted: 0x7E7A70,
      accent: 0x869DB9,
      border: 0x3A3933,
      surface: 0x30302B,
      quote: 0xB7B3A8,
      taskDone: 0xB7B3A8,
      inlineCodeBackground: 0x30302B,
    ),
  );
}
