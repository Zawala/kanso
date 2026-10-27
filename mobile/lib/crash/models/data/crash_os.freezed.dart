// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_os.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$CrashOs {

 CrashOsName get name;/// `18.1`, or `unknown`.
 String get version;
/// Create a copy of CrashOs
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashOsCopyWith<CrashOs> get copyWith => _$CrashOsCopyWithImpl<CrashOs>(this as CrashOs, _$identity);

  /// Serializes this CrashOs to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashOs&&(identical(other.name, name) || other.name == name)&&(identical(other.version, version) || other.version == version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,name,version);

@override
String toString() {
  return 'CrashOs(name: $name, version: $version)';
}


}

/// @nodoc
abstract mixin class $CrashOsCopyWith<$Res>  {
  factory $CrashOsCopyWith(CrashOs value, $Res Function(CrashOs) _then) = _$CrashOsCopyWithImpl;
@useResult
$Res call({
 CrashOsName name, String version
});




}
/// @nodoc
class _$CrashOsCopyWithImpl<$Res>
    implements $CrashOsCopyWith<$Res> {
  _$CrashOsCopyWithImpl(this._self, this._then);

  final CrashOs _self;
  final $Res Function(CrashOs) _then;

/// Create a copy of CrashOs
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? version = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as CrashOsName,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [CrashOs].
extension CrashOsPatterns on CrashOs {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashOs value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashOs() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashOs value)  $default,){
final _that = this;
switch (_that) {
case _CrashOs():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashOs value)?  $default,){
final _that = this;
switch (_that) {
case _CrashOs() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( CrashOsName name,  String version)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashOs() when $default != null:
return $default(_that.name,_that.version);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( CrashOsName name,  String version)  $default,) {final _that = this;
switch (_that) {
case _CrashOs():
return $default(_that.name,_that.version);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( CrashOsName name,  String version)?  $default,) {final _that = this;
switch (_that) {
case _CrashOs() when $default != null:
return $default(_that.name,_that.version);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _CrashOs implements CrashOs {
  const _CrashOs({required this.name, required this.version});
  factory _CrashOs.fromJson(Map<String, dynamic> json) => _$CrashOsFromJson(json);

@override final  CrashOsName name;
/// `18.1`, or `unknown`.
@override final  String version;

/// Create a copy of CrashOs
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashOsCopyWith<_CrashOs> get copyWith => __$CrashOsCopyWithImpl<_CrashOs>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$CrashOsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashOs&&(identical(other.name, name) || other.name == name)&&(identical(other.version, version) || other.version == version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,name,version);

@override
String toString() {
  return 'CrashOs(name: $name, version: $version)';
}


}

/// @nodoc
abstract mixin class _$CrashOsCopyWith<$Res> implements $CrashOsCopyWith<$Res> {
  factory _$CrashOsCopyWith(_CrashOs value, $Res Function(_CrashOs) _then) = __$CrashOsCopyWithImpl;
@override @useResult
$Res call({
 CrashOsName name, String version
});




}
/// @nodoc
class __$CrashOsCopyWithImpl<$Res>
    implements _$CrashOsCopyWith<$Res> {
  __$CrashOsCopyWithImpl(this._self, this._then);

  final _CrashOs _self;
  final $Res Function(_CrashOs) _then;

/// Create a copy of CrashOs
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? version = null,}) {
  return _then(_CrashOs(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as CrashOsName,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
