// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'crash_report.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_CrashReport _$CrashReportFromJson(Map<String, dynamic> json) => _CrashReport(
  v: (json['v'] as num).toInt(),
  kind: $enumDecode(_$CrashKindEnumMap, json['kind']),
  type: json['type'] as String,
  app: CrashApp.fromJson(json['app'] as Map<String, dynamic>),
  os: CrashOs.fromJson(json['os'] as Map<String, dynamic>),
  device: json['device'] as String,
  day: json['day'] as String,
  frames: (json['frames'] as List<dynamic>).map((e) => e as String).toList(),
);

Map<String, dynamic> _$CrashReportToJson(_CrashReport instance) =>
    <String, dynamic>{
      'v': instance.v,
      'kind': _$CrashKindEnumMap[instance.kind]!,
      'type': instance.type,
      'app': instance.app.toJson(),
      'os': instance.os.toJson(),
      'device': instance.device,
      'day': instance.day,
      'frames': instance.frames,
    };

const _$CrashKindEnumMap = {
  CrashKind.dart: 'dart',
  CrashKind.iosNative: 'ios-native',
  CrashKind.androidNative: 'android-native',
  CrashKind.androidAnr: 'android-anr',
  CrashKind.androidJvm: 'android-jvm',
};
