// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_report.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CrashReport {

/// The version of the format; first, as on the page.
 int get v; CrashKind get kind;/// A type, signal or exception class name, or `unknown`.
 String get type; CrashApp get app; CrashOs get os;/// The device's model identifier, or `unknown`.
 String get device;/// The UTC day of the crash, `2026-10-26`.
 String get day;/// One to 128 frames, innermost first.
 List<String> get frames;
/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashReportCopyWith<CrashReport> get copyWith => _$CrashReportCopyWithImpl<CrashReport>(this as CrashReport, _$identity);

  /// Serializes this CrashReport to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReport&&(identical(other.v, v) || other.v == v)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.type, type) || other.type == type)&&(identical(other.app, app) || other.app == app)&&(identical(other.os, os) || other.os == os)&&(identical(other.device, device) || other.device == device)&&(identical(other.day, day) || other.day == day)&&const DeepCollectionEquality().equals(other.frames, frames));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,v,kind,type,app,os,device,day,const DeepCollectionEquality().hash(frames));

@override
String toString() {
  return 'CrashReport(v: $v, kind: $kind, type: $type, app: $app, os: $os, device: $device, day: $day, frames: $frames)';
}


}

/// @nodoc
abstract mixin class $CrashReportCopyWith<$Res>  {
  factory $CrashReportCopyWith(CrashReport value, $Res Function(CrashReport) _then) = _$CrashReportCopyWithImpl;
@useResult
$Res call({
 int v, CrashKind kind, String type, CrashApp app, CrashOs os, String device, String day, List<String> frames
});


$CrashAppCopyWith<$Res> get app;$CrashOsCopyWith<$Res> get os;

}
/// @nodoc
class _$CrashReportCopyWithImpl<$Res>
    implements $CrashReportCopyWith<$Res> {
  _$CrashReportCopyWithImpl(this._self, this._then);

  final CrashReport _self;
  final $Res Function(CrashReport) _then;

/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? v = null,Object? kind = null,Object? type = null,Object? app = null,Object? os = null,Object? device = null,Object? day = null,Object? frames = null,}) {
  return _then(_self.copyWith(
v: null == v ? _self.v : v // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as CrashKind,type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,app: null == app ? _self.app : app // ignore: cast_nullable_to_non_nullable
as CrashApp,os: null == os ? _self.os : os // ignore: cast_nullable_to_non_nullable
as CrashOs,device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String,day: null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as String,frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}
/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashAppCopyWith<$Res> get app {
  
  return $CrashAppCopyWith<$Res>(_self.app, (value) {
    return _then(_self.copyWith(app: value));
  });
}/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashOsCopyWith<$Res> get os {
  
  return $CrashOsCopyWith<$Res>(_self.os, (value) {
    return _then(_self.copyWith(os: value));
  });
}
}


/// Adds pattern-matching-related methods to [CrashReport].
extension CrashReportPatterns on CrashReport {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashReport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashReport() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashReport value)  $default,){
final _that = this;
switch (_that) {
case _CrashReport():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashReport value)?  $default,){
final _that = this;
switch (_that) {
case _CrashReport() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int v,  CrashKind kind,  String type,  CrashApp app,  CrashOs os,  String device,  String day,  List<String> frames)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashReport() when $default != null:
return $default(_that.v,_that.kind,_that.type,_that.app,_that.os,_that.device,_that.day,_that.frames);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int v,  CrashKind kind,  String type,  CrashApp app,  CrashOs os,  String device,  String day,  List<String> frames)  $default,) {final _that = this;
switch (_that) {
case _CrashReport():
return $default(_that.v,_that.kind,_that.type,_that.app,_that.os,_that.device,_that.day,_that.frames);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int v,  CrashKind kind,  String type,  CrashApp app,  CrashOs os,  String device,  String day,  List<String> frames)?  $default,) {final _that = this;
switch (_that) {
case _CrashReport() when $default != null:
return $default(_that.v,_that.kind,_that.type,_that.app,_that.os,_that.device,_that.day,_that.frames);case _:
  return null;

}
}

}

/// @nodoc

@JsonSerializable(explicitToJson: true)
class _CrashReport implements CrashReport {
  const _CrashReport({required this.v, required this.kind, required this.type, required this.app, required this.os, required this.device, required this.day, required final  List<String> frames}): _frames = frames;
  factory _CrashReport.fromJson(Map<String, dynamic> json) => _$CrashReportFromJson(json);

/// The version of the format; first, as on the page.
@override final  int v;
@override final  CrashKind kind;
/// A type, signal or exception class name, or `unknown`.
@override final  String type;
@override final  CrashApp app;
@override final  CrashOs os;
/// The device's model identifier, or `unknown`.
@override final  String device;
/// The UTC day of the crash, `2026-10-26`.
@override final  String day;
/// One to 128 frames, innermost first.
 final  List<String> _frames;
/// One to 128 frames, innermost first.
@override List<String> get frames {
  if (_frames is EqualUnmodifiableListView) return _frames;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_frames);
}


/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashReportCopyWith<_CrashReport> get copyWith => __$CrashReportCopyWithImpl<_CrashReport>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CrashReportToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashReport&&(identical(other.v, v) || other.v == v)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.type, type) || other.type == type)&&(identical(other.app, app) || other.app == app)&&(identical(other.os, os) || other.os == os)&&(identical(other.device, device) || other.device == device)&&(identical(other.day, day) || other.day == day)&&const DeepCollectionEquality().equals(other._frames, _frames));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,v,kind,type,app,os,device,day,const DeepCollectionEquality().hash(_frames));

@override
String toString() {
  return 'CrashReport(v: $v, kind: $kind, type: $type, app: $app, os: $os, device: $device, day: $day, frames: $frames)';
}


}

/// @nodoc
abstract mixin class _$CrashReportCopyWith<$Res> implements $CrashReportCopyWith<$Res> {
  factory _$CrashReportCopyWith(_CrashReport value, $Res Function(_CrashReport) _then) = __$CrashReportCopyWithImpl;
@override @useResult
$Res call({
 int v, CrashKind kind, String type, CrashApp app, CrashOs os, String device, String day, List<String> frames
});


@override $CrashAppCopyWith<$Res> get app;@override $CrashOsCopyWith<$Res> get os;

}
/// @nodoc
class __$CrashReportCopyWithImpl<$Res>
    implements _$CrashReportCopyWith<$Res> {
  __$CrashReportCopyWithImpl(this._self, this._then);

  final _CrashReport _self;
  final $Res Function(_CrashReport) _then;

/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? v = null,Object? kind = null,Object? type = null,Object? app = null,Object? os = null,Object? device = null,Object? day = null,Object? frames = null,}) {
  return _then(_CrashReport(
v: null == v ? _self.v : v // ignore: cast_nullable_to_non_nullable
as int,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as CrashKind,type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,app: null == app ? _self.app : app // ignore: cast_nullable_to_non_nullable
as CrashApp,os: null == os ? _self.os : os // ignore: cast_nullable_to_non_nullable
as CrashOs,device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String,day: null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as String,frames: null == frames ? _self._frames : frames // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashAppCopyWith<$Res> get app {
  
  return $CrashAppCopyWith<$Res>(_self.app, (value) {
    return _then(_self.copyWith(app: value));
  });
}/// Create a copy of CrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashOsCopyWith<$Res> get os {
  
  return $CrashOsCopyWith<$Res>(_self.os, (value) {
    return _then(_self.copyWith(os: value));
  });
}
}

// dart format on
