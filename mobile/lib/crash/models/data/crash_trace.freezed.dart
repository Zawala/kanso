// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_trace.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CrashTrace {

/// A type, signal or exception name, or `unknown`.
 String get type;/// Scrubbed frames, innermost first.
 List<String> get frames;/// The UTC day, `2026-10-26`, when the raw crash says it.
 String? get day;/// The app version the raw crash says it ran, when it says.
 String? get appVersion;/// The build the raw crash says it ran.
 String? get appBuild;/// The OS version the raw crash says it ran on.
 String? get osVersion;/// The device identifier the raw crash says it ran on.
 String? get device;
/// Create a copy of CrashTrace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashTraceCopyWith<CrashTrace> get copyWith => _$CrashTraceCopyWithImpl<CrashTrace>(this as CrashTrace, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashTrace&&(identical(other.type, type) || other.type == type)&&const DeepCollectionEquality().equals(other.frames, frames)&&(identical(other.day, day) || other.day == day)&&(identical(other.appVersion, appVersion) || other.appVersion == appVersion)&&(identical(other.appBuild, appBuild) || other.appBuild == appBuild)&&(identical(other.osVersion, osVersion) || other.osVersion == osVersion)&&(identical(other.device, device) || other.device == device));
}


@override
int get hashCode => Object.hash(runtimeType,type,const DeepCollectionEquality().hash(frames),day,appVersion,appBuild,osVersion,device);

@override
String toString() {
  return 'CrashTrace(type: $type, frames: $frames, day: $day, appVersion: $appVersion, appBuild: $appBuild, osVersion: $osVersion, device: $device)';
}


}

/// @nodoc
abstract mixin class $CrashTraceCopyWith<$Res>  {
  factory $CrashTraceCopyWith(CrashTrace value, $Res Function(CrashTrace) _then) = _$CrashTraceCopyWithImpl;
@useResult
$Res call({
 String type, List<String> frames, String? day, String? appVersion, String? appBuild, String? osVersion, String? device
});




}
/// @nodoc
class _$CrashTraceCopyWithImpl<$Res>
    implements $CrashTraceCopyWith<$Res> {
  _$CrashTraceCopyWithImpl(this._self, this._then);

  final CrashTrace _self;
  final $Res Function(CrashTrace) _then;

/// Create a copy of CrashTrace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? type = null,Object? frames = null,Object? day = freezed,Object? appVersion = freezed,Object? appBuild = freezed,Object? osVersion = freezed,Object? device = freezed,}) {
  return _then(_self.copyWith(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as List<String>,day: freezed == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as String?,appVersion: freezed == appVersion ? _self.appVersion : appVersion // ignore: cast_nullable_to_non_nullable
as String?,appBuild: freezed == appBuild ? _self.appBuild : appBuild // ignore: cast_nullable_to_non_nullable
as String?,osVersion: freezed == osVersion ? _self.osVersion : osVersion // ignore: cast_nullable_to_non_nullable
as String?,device: freezed == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [CrashTrace].
extension CrashTracePatterns on CrashTrace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashTrace value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashTrace() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashTrace value)  $default,){
final _that = this;
switch (_that) {
case _CrashTrace():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashTrace value)?  $default,){
final _that = this;
switch (_that) {
case _CrashTrace() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String type,  List<String> frames,  String? day,  String? appVersion,  String? appBuild,  String? osVersion,  String? device)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashTrace() when $default != null:
return $default(_that.type,_that.frames,_that.day,_that.appVersion,_that.appBuild,_that.osVersion,_that.device);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String type,  List<String> frames,  String? day,  String? appVersion,  String? appBuild,  String? osVersion,  String? device)  $default,) {final _that = this;
switch (_that) {
case _CrashTrace():
return $default(_that.type,_that.frames,_that.day,_that.appVersion,_that.appBuild,_that.osVersion,_that.device);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String type,  List<String> frames,  String? day,  String? appVersion,  String? appBuild,  String? osVersion,  String? device)?  $default,) {final _that = this;
switch (_that) {
case _CrashTrace() when $default != null:
return $default(_that.type,_that.frames,_that.day,_that.appVersion,_that.appBuild,_that.osVersion,_that.device);case _:
  return null;

}
}

}

/// @nodoc


class _CrashTrace implements CrashTrace {
  const _CrashTrace({required this.type, required final  List<String> frames, this.day, this.appVersion, this.appBuild, this.osVersion, this.device}): _frames = frames;
  

/// A type, signal or exception name, or `unknown`.
@override final  String type;
/// Scrubbed frames, innermost first.
 final  List<String> _frames;
/// Scrubbed frames, innermost first.
@override List<String> get frames {
  if (_frames is EqualUnmodifiableListView) return _frames;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_frames);
}

/// The UTC day, `2026-10-26`, when the raw crash says it.
@override final  String? day;
/// The app version the raw crash says it ran, when it says.
@override final  String? appVersion;
/// The build the raw crash says it ran.
@override final  String? appBuild;
/// The OS version the raw crash says it ran on.
@override final  String? osVersion;
/// The device identifier the raw crash says it ran on.
@override final  String? device;

/// Create a copy of CrashTrace
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashTraceCopyWith<_CrashTrace> get copyWith => __$CrashTraceCopyWithImpl<_CrashTrace>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashTrace&&(identical(other.type, type) || other.type == type)&&const DeepCollectionEquality().equals(other._frames, _frames)&&(identical(other.day, day) || other.day == day)&&(identical(other.appVersion, appVersion) || other.appVersion == appVersion)&&(identical(other.appBuild, appBuild) || other.appBuild == appBuild)&&(identical(other.osVersion, osVersion) || other.osVersion == osVersion)&&(identical(other.device, device) || other.device == device));
}


@override
int get hashCode => Object.hash(runtimeType,type,const DeepCollectionEquality().hash(_frames),day,appVersion,appBuild,osVersion,device);

@override
String toString() {
  return 'CrashTrace(type: $type, frames: $frames, day: $day, appVersion: $appVersion, appBuild: $appBuild, osVersion: $osVersion, device: $device)';
}


}

/// @nodoc
abstract mixin class _$CrashTraceCopyWith<$Res> implements $CrashTraceCopyWith<$Res> {
  factory _$CrashTraceCopyWith(_CrashTrace value, $Res Function(_CrashTrace) _then) = __$CrashTraceCopyWithImpl;
@override @useResult
$Res call({
 String type, List<String> frames, String? day, String? appVersion, String? appBuild, String? osVersion, String? device
});




}
/// @nodoc
class __$CrashTraceCopyWithImpl<$Res>
    implements _$CrashTraceCopyWith<$Res> {
  __$CrashTraceCopyWithImpl(this._self, this._then);

  final _CrashTrace _self;
  final $Res Function(_CrashTrace) _then;

/// Create a copy of CrashTrace
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? type = null,Object? frames = null,Object? day = freezed,Object? appVersion = freezed,Object? appBuild = freezed,Object? osVersion = freezed,Object? device = freezed,}) {
  return _then(_CrashTrace(
type: null == type ? _self.type : type // ignore: cast_nullable_to_non_nullable
as String,frames: null == frames ? _self._frames : frames // ignore: cast_nullable_to_non_nullable
as List<String>,day: freezed == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as String?,appVersion: freezed == appVersion ? _self.appVersion : appVersion // ignore: cast_nullable_to_non_nullable
as String?,appBuild: freezed == appBuild ? _self.appBuild : appBuild // ignore: cast_nullable_to_non_nullable
as String?,osVersion: freezed == osVersion ? _self.osVersion : osVersion // ignore: cast_nullable_to_non_nullable
as String?,device: freezed == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
