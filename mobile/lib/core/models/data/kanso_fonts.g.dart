// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'kanso_fonts.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_KansoFonts _$KansoFontsFromJson(Map<String, dynamic> json) => _KansoFonts(
  interfaceFont:
      $enumDecodeNullable(
        _$KansoFontFamilyEnumMap,
        json['interfaceFont'],
        unknownValue: KansoFontFamily.instrumentSans,
      ) ??
      KansoFontFamily.instrumentSans,
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
  interfaceGoogleFont: json['interfaceGoogleFont'] as String?,
  codeGoogleFont: json['codeGoogleFont'] as String?,
  downloadedGoogleFonts:
      (json['downloadedGoogleFonts'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const <String>[],
);

Map<String, dynamic> _$KansoFontsToJson(_KansoFonts instance) =>
    <String, dynamic>{
      'interfaceFont': _$KansoFontFamilyEnumMap[instance.interfaceFont]!,
      'editorFont': _$KansoFontFamilyEnumMap[instance.editorFont]!,
      'previewFont': _$KansoFontFamilyEnumMap[instance.previewFont]!,
      'codeFont': _$KansoFontFamilyEnumMap[instance.codeFont]!,
      'editorGoogleFont': ?instance.editorGoogleFont,
      'previewGoogleFont': ?instance.previewGoogleFont,
      'interfaceGoogleFont': ?instance.interfaceGoogleFont,
      'codeGoogleFont': ?instance.codeGoogleFont,
      'downloadedGoogleFonts': instance.downloadedGoogleFonts,
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
