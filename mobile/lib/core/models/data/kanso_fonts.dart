// Kanso — kanso_fonts.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'kanso_font_family.dart';

part 'kanso_fonts.freezed.dart';
part 'kanso_fonts.g.dart';

/// The three fonts of a note: the one it is written in, the one it is read
/// in and the one of its code. The interface keeps Kanso's own font.
@freezed
abstract class KansoFonts with _$KansoFonts {
  const factory KansoFonts({
    /// Writing in the editor.
    @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)
    @Default(KansoFontFamily.instrumentSans)
    KansoFontFamily editorFont,

    /// Reading a note.
    @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)
    @Default(KansoFontFamily.instrumentSans)
    KansoFontFamily previewFont,

    /// Code spans, code blocks and the theme previews.
    @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)
    @Default(KansoFontFamily.jetBrainsMono)
    KansoFontFamily codeFont,
  }) = _KansoFonts;

  const KansoFonts._();

  factory KansoFonts.fromJson(Map<String, dynamic> json) =>
      _$KansoFontsFromJson(json);

  /// A short line naming the choice, for a settings row: the editor font
  /// and the code font, with the preview font between them when it is a
  /// different one. [systemLabel] is how the device's font is called.
  ///
  /// It is never empty.
  String summary({String systemLabel = 'System'}) {
    String name(KansoFontFamily font) {
      return font == KansoFontFamily.system ? systemLabel : font.displayName;
    }

    final names = <String>[
      name(editorFont),
      if (previewFont != editorFont) name(previewFont),
      name(codeFont),
    ];
    return names.join(' · ');
  }
}
