// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'frame_sample.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FrameSample {

 double get buildMs; double get rasterMs;
/// Create a copy of FrameSample
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FrameSampleCopyWith<FrameSample> get copyWith => _$FrameSampleCopyWithImpl<FrameSample>(this as FrameSample, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FrameSample&&(identical(other.buildMs, buildMs) || other.buildMs == buildMs)&&(identical(other.rasterMs, rasterMs) || other.rasterMs == rasterMs));
}


@override
int get hashCode => Object.hash(runtimeType,buildMs,rasterMs);

@override
String toString() {
  return 'FrameSample(buildMs: $buildMs, rasterMs: $rasterMs)';
}


}

/// @nodoc
abstract mixin class $FrameSampleCopyWith<$Res>  {
  factory $FrameSampleCopyWith(FrameSample value, $Res Function(FrameSample) _then) = _$FrameSampleCopyWithImpl;
@useResult
$Res call({
 double buildMs, double rasterMs
});




}
/// @nodoc
class _$FrameSampleCopyWithImpl<$Res>
    implements $FrameSampleCopyWith<$Res> {
  _$FrameSampleCopyWithImpl(this._self, this._then);

  final FrameSample _self;
  final $Res Function(FrameSample) _then;

/// Create a copy of FrameSample
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? buildMs = null,Object? rasterMs = null,}) {
  return _then(_self.copyWith(
buildMs: null == buildMs ? _self.buildMs : buildMs // ignore: cast_nullable_to_non_nullable
as double,rasterMs: null == rasterMs ? _self.rasterMs : rasterMs // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [FrameSample].
extension FrameSamplePatterns on FrameSample {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FrameSample value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FrameSample() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FrameSample value)  $default,){
final _that = this;
switch (_that) {
case _FrameSample():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FrameSample value)?  $default,){
final _that = this;
switch (_that) {
case _FrameSample() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( double buildMs,  double rasterMs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FrameSample() when $default != null:
return $default(_that.buildMs,_that.rasterMs);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( double buildMs,  double rasterMs)  $default,) {final _that = this;
switch (_that) {
case _FrameSample():
return $default(_that.buildMs,_that.rasterMs);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( double buildMs,  double rasterMs)?  $default,) {final _that = this;
switch (_that) {
case _FrameSample() when $default != null:
return $default(_that.buildMs,_that.rasterMs);case _:
  return null;

}
}

}

/// @nodoc


class _FrameSample implements FrameSample {
  const _FrameSample({required this.buildMs, required this.rasterMs});
  

@override final  double buildMs;
@override final  double rasterMs;

/// Create a copy of FrameSample
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FrameSampleCopyWith<_FrameSample> get copyWith => __$FrameSampleCopyWithImpl<_FrameSample>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FrameSample&&(identical(other.buildMs, buildMs) || other.buildMs == buildMs)&&(identical(other.rasterMs, rasterMs) || other.rasterMs == rasterMs));
}


@override
int get hashCode => Object.hash(runtimeType,buildMs,rasterMs);

@override
String toString() {
  return 'FrameSample(buildMs: $buildMs, rasterMs: $rasterMs)';
}


}

/// @nodoc
abstract mixin class _$FrameSampleCopyWith<$Res> implements $FrameSampleCopyWith<$Res> {
  factory _$FrameSampleCopyWith(_FrameSample value, $Res Function(_FrameSample) _then) = __$FrameSampleCopyWithImpl;
@override @useResult
$Res call({
 double buildMs, double rasterMs
});




}
/// @nodoc
class __$FrameSampleCopyWithImpl<$Res>
    implements _$FrameSampleCopyWith<$Res> {
  __$FrameSampleCopyWithImpl(this._self, this._then);

  final _FrameSample _self;
  final $Res Function(_FrameSample) _then;

/// Create a copy of FrameSample
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? buildMs = null,Object? rasterMs = null,}) {
  return _then(_FrameSample(
buildMs: null == buildMs ? _self.buildMs : buildMs // ignore: cast_nullable_to_non_nullable
as double,rasterMs: null == rasterMs ? _self.rasterMs : rasterMs // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

// dart format on
