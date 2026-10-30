// Kanso — kanso_fonts.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'kanso_font_family.dart';
import 'kanso_font_slot.dart';

part 'kanso_fonts.freezed.dart';
part 'kanso_fonts.g.dart';

/// The four fonts of the app: the interface's, the one a note is written
/// in, the one it is read in and the one of its code. Each is a bundled
/// family or, when its Google Font is set, one downloaded from Google
/// Fonts; [downloadedGoogleFonts] lists every downloaded family, so each
/// can be chosen for any of the four.
@freezed
abstract class KansoFonts with _$KansoFonts {
  const factory KansoFonts({
    /// Menus, lists, labels and buttons.
    @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)
    @Default(KansoFontFamily.instrumentSans)
    KansoFontFamily interfaceFont,

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

    /// A downloaded Google Font for the editor, when selected.
    String? editorGoogleFont,

    /// A downloaded Google Font for the preview, when selected.
    String? previewGoogleFont,

    /// A downloaded Google Font for the interface, when selected.
    String? interfaceGoogleFont,

    /// A downloaded Google Font for code, when selected.
    String? codeGoogleFont,

    /// The Google Fonts families downloaded to this device, in the order
    /// they were downloaded.
    @Default(<String>[]) List<String> downloadedGoogleFonts,
  }) = _KansoFonts;

  const KansoFonts._();

  factory KansoFonts.fromJson(Map<String, dynamic> json) =>
      _$KansoFontsFromJson(json);

  /// The downloaded Google Font set for [slot], or null when the slot uses
  /// a bundled family.
  String? googleFontFor(KansoFontSlot slot) {
    return switch (slot) {
      KansoFontSlot.interface => interfaceGoogleFont,
      KansoFontSlot.editor => editorGoogleFont,
      KansoFontSlot.preview => previewGoogleFont,
      KansoFontSlot.code => codeGoogleFont,
    };
  }

  /// These fonts with [family] set as the Google Font of [slot].
  KansoFonts withGoogleFont(KansoFontSlot slot, String family) {
    return switch (slot) {
      KansoFontSlot.interface => copyWith(interfaceGoogleFont: family),
      KansoFontSlot.editor => copyWith(editorGoogleFont: family),
      KansoFontSlot.preview => copyWith(previewGoogleFont: family),
      KansoFontSlot.code => copyWith(codeGoogleFont: family),
    };
  }

  /// Every Google Font these fonts know: the downloaded ones, then any set
  /// in a slot but missing from the list (saved by a build that kept no
  /// list), without repeats.
  List<String> get googleFonts {
    return <String>{
      ...downloadedGoogleFonts,
      ?editorGoogleFont,
      ?previewGoogleFont,
      ?interfaceGoogleFont,
      ?codeGoogleFont,
    }.toList();
  }

  /// These fonts with [family] downloaded: added once to
  /// [downloadedGoogleFonts].
  KansoFonts withDownloaded(String family) {
    return downloadedGoogleFonts.contains(family)
        ? this
        : copyWith(
            downloadedGoogleFonts: <String>[...downloadedGoogleFonts, family],
          );
  }

  /// These fonts without [family]: gone from the list, and every slot that
  /// used it back on its bundled family.
  KansoFonts withoutGoogleFont(String family) {
    String? keep(String? value) => value == family ? null : value;
    return copyWith(
      downloadedGoogleFonts: <String>[
        for (final name in downloadedGoogleFonts)
          if (name != family) name,
      ],
      editorGoogleFont: keep(editorGoogleFont),
      previewGoogleFont: keep(previewGoogleFont),
      interfaceGoogleFont: keep(interfaceGoogleFont),
      codeGoogleFont: keep(codeGoogleFont),
    );
  }

  /// A short line naming the choice, for a settings row: the editor font
  /// and the code font, with the preview font between them when it is a
  /// different one. [systemLabel] is how the device's font is called.
  ///
  /// It is never empty.
  String summary({String systemLabel = 'System'}) {
    String name(KansoFontFamily font) {
      return font == KansoFontFamily.system ? systemLabel : font.displayName;
    }

    final editor = editorGoogleFont ?? name(editorFont);
    final preview = previewGoogleFont ?? name(previewFont);
    final names = <String>[
      editor,
      if (preview != editor) preview,
      codeGoogleFont ?? name(codeFont),
    ];
    return names.join(' · ');
  }
}
