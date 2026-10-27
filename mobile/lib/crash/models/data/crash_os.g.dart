// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'crash_os.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_CrashOs _$CrashOsFromJson(Map<String, dynamic> json) => _CrashOs(
  name: $enumDecode(_$CrashOsNameEnumMap, json['name']),
  version: json['version'] as String,
);

Map<String, dynamic> _$CrashOsToJson(_CrashOs instance) => <String, dynamic>{
  'name': _$CrashOsNameEnumMap[instance.name]!,
  'version': instance.version,
};

const _$CrashOsNameEnumMap = {
  CrashOsName.ios: 'ios',
  CrashOsName.android: 'android',
};
