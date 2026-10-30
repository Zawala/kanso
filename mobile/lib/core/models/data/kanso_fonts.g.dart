// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'kanso_fonts.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_KansoFonts _$KansoFontsFromJson(Map<String, dynamic> json) => _KansoFonts(
  editorFont:
      $enumDecodeNullable(
        _$KansoFontFamilyEnumMap,
        json['editorFont'],
        unknownValue: KansoFontFamily.instrumentSans,
      ) ??
      KansoFontFamily.instrumentSans,
  previewFont:
      $enumDecodeNullable(
        _$KansoFontFamilyEnumMap,
        json['previewFont'],
        unknownValue: KansoFontFamily.instrumentSans,
      ) ??
      KansoFontFamily.instrumentSans,
  codeFont:
      $enumDecodeNullable(
        _$KansoFontFamilyEnumMap,
        json['codeFont'],
        unknownValue: KansoFontFamily.jetBrainsMono,
      ) ??
      KansoFontFamily.jetBrainsMono,
  editorGoogleFont: json['editorGoogleFont'] as String?,
  previewGoogleFont: json['previewGoogleFont'] as String?,
);

Map<String, dynamic> _$KansoFontsToJson(_KansoFonts instance) =>
    <String, dynamic>{
      'editorFont': _$KansoFontFamilyEnumMap[instance.editorFont]!,
      'previewFont': _$KansoFontFamilyEnumMap[instance.previewFont]!,
      'codeFont': _$KansoFontFamilyEnumMap[instance.codeFont]!,
      'editorGoogleFont': ?instance.editorGoogleFont,
      'previewGoogleFont': ?instance.previewGoogleFont,
    };

const _$KansoFontFamilyEnumMap = {
  KansoFontFamily.instrumentSans: 'instrumentSans',
  KansoFontFamily.inter: 'inter',
  KansoFontFamily.sourceSerif4: 'sourceSerif4',
  KansoFontFamily.literata: 'literata',
  KansoFontFamily.jetBrainsMono: 'jetBrainsMono',
  KansoFontFamily.ibmPlexMono: 'ibmPlexMono',
  KansoFontFamily.system: 'system',
};
