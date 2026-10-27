// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_context.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CrashContext {

 CrashApp get app; CrashOs get os; String get device;
/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashContextCopyWith<CrashContext> get copyWith => _$CrashContextCopyWithImpl<CrashContext>(this as CrashContext, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashContext&&(identical(other.app, app) || other.app == app)&&(identical(other.os, os) || other.os == os)&&(identical(other.device, device) || other.device == device));
}


@override
int get hashCode => Object.hash(runtimeType,app,os,device);

@override
String toString() {
  return 'CrashContext(app: $app, os: $os, device: $device)';
}


}

/// @nodoc
abstract mixin class $CrashContextCopyWith<$Res>  {
  factory $CrashContextCopyWith(CrashContext value, $Res Function(CrashContext) _then) = _$CrashContextCopyWithImpl;
@useResult
$Res call({
 CrashApp app, CrashOs os, String device
});


$CrashAppCopyWith<$Res> get app;$CrashOsCopyWith<$Res> get os;

}
/// @nodoc
class _$CrashContextCopyWithImpl<$Res>
    implements $CrashContextCopyWith<$Res> {
  _$CrashContextCopyWithImpl(this._self, this._then);

  final CrashContext _self;
  final $Res Function(CrashContext) _then;

/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? app = null,Object? os = null,Object? device = null,}) {
  return _then(_self.copyWith(
app: null == app ? _self.app : app // ignore: cast_nullable_to_non_nullable
as CrashApp,os: null == os ? _self.os : os // ignore: cast_nullable_to_non_nullable
as CrashOs,device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String,
  ));
}
/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashAppCopyWith<$Res> get app {
  
  return $CrashAppCopyWith<$Res>(_self.app, (value) {
    return _then(_self.copyWith(app: value));
  });
}/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashOsCopyWith<$Res> get os {
  
  return $CrashOsCopyWith<$Res>(_self.os, (value) {
    return _then(_self.copyWith(os: value));
  });
}
}


/// Adds pattern-matching-related methods to [CrashContext].
extension CrashContextPatterns on CrashContext {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashContext value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashContext() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashContext value)  $default,){
final _that = this;
switch (_that) {
case _CrashContext():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashContext value)?  $default,){
final _that = this;
switch (_that) {
case _CrashContext() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( CrashApp app,  CrashOs os,  String device)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashContext() when $default != null:
return $default(_that.app,_that.os,_that.device);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( CrashApp app,  CrashOs os,  String device)  $default,) {final _that = this;
switch (_that) {
case _CrashContext():
return $default(_that.app,_that.os,_that.device);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( CrashApp app,  CrashOs os,  String device)?  $default,) {final _that = this;
switch (_that) {
case _CrashContext() when $default != null:
return $default(_that.app,_that.os,_that.device);case _:
  return null;

}
}

}

/// @nodoc


class _CrashContext implements CrashContext {
  const _CrashContext({required this.app, required this.os, required this.device});
  

@override final  CrashApp app;
@override final  CrashOs os;
@override final  String device;

/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashContextCopyWith<_CrashContext> get copyWith => __$CrashContextCopyWithImpl<_CrashContext>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashContext&&(identical(other.app, app) || other.app == app)&&(identical(other.os, os) || other.os == os)&&(identical(other.device, device) || other.device == device));
}


@override
int get hashCode => Object.hash(runtimeType,app,os,device);

@override
String toString() {
  return 'CrashContext(app: $app, os: $os, device: $device)';
}


}

/// @nodoc
abstract mixin class _$CrashContextCopyWith<$Res> implements $CrashContextCopyWith<$Res> {
  factory _$CrashContextCopyWith(_CrashContext value, $Res Function(_CrashContext) _then) = __$CrashContextCopyWithImpl;
@override @useResult
$Res call({
 CrashApp app, CrashOs os, String device
});


@override $CrashAppCopyWith<$Res> get app;@override $CrashOsCopyWith<$Res> get os;

}
/// @nodoc
class __$CrashContextCopyWithImpl<$Res>
    implements _$CrashContextCopyWith<$Res> {
  __$CrashContextCopyWithImpl(this._self, this._then);

  final _CrashContext _self;
  final $Res Function(_CrashContext) _then;

/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? app = null,Object? os = null,Object? device = null,}) {
  return _then(_CrashContext(
app: null == app ? _self.app : app // ignore: cast_nullable_to_non_nullable
as CrashApp,os: null == os ? _self.os : os // ignore: cast_nullable_to_non_nullable
as CrashOs,device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of CrashContext
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashAppCopyWith<$Res> get app {
  
  return $CrashAppCopyWith<$Res>(_self.app, (value) {
    return _then(_self.copyWith(app: value));
  });
}/// Create a copy of CrashContext
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
