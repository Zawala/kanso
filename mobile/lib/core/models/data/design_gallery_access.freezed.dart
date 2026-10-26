// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'design_gallery_access.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DesignGalleryAccess {

 bool get available;
/// Create a copy of DesignGalleryAccess
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DesignGalleryAccessCopyWith<DesignGalleryAccess> get copyWith => _$DesignGalleryAccessCopyWithImpl<DesignGalleryAccess>(this as DesignGalleryAccess, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DesignGalleryAccess&&(identical(other.available, available) || other.available == available));
}


@override
int get hashCode => Object.hash(runtimeType,available);

@override
String toString() {
  return 'DesignGalleryAccess(available: $available)';
}


}

/// @nodoc
abstract mixin class $DesignGalleryAccessCopyWith<$Res>  {
  factory $DesignGalleryAccessCopyWith(DesignGalleryAccess value, $Res Function(DesignGalleryAccess) _then) = _$DesignGalleryAccessCopyWithImpl;
@useResult
$Res call({
 bool available
});




}
/// @nodoc
class _$DesignGalleryAccessCopyWithImpl<$Res>
    implements $DesignGalleryAccessCopyWith<$Res> {
  _$DesignGalleryAccessCopyWithImpl(this._self, this._then);

  final DesignGalleryAccess _self;
  final $Res Function(DesignGalleryAccess) _then;

/// Create a copy of DesignGalleryAccess
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? available = null,}) {
  return _then(_self.copyWith(
available: null == available ? _self.available : available // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [DesignGalleryAccess].
extension DesignGalleryAccessPatterns on DesignGalleryAccess {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DesignGalleryAccess value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DesignGalleryAccess() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DesignGalleryAccess value)  $default,){
final _that = this;
switch (_that) {
case _DesignGalleryAccess():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DesignGalleryAccess value)?  $default,){
final _that = this;
switch (_that) {
case _DesignGalleryAccess() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool available)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DesignGalleryAccess() when $default != null:
return $default(_that.available);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool available)  $default,) {final _that = this;
switch (_that) {
case _DesignGalleryAccess():
return $default(_that.available);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool available)?  $default,) {final _that = this;
switch (_that) {
case _DesignGalleryAccess() when $default != null:
return $default(_that.available);case _:
  return null;

}
}

}

/// @nodoc


class _DesignGalleryAccess implements DesignGalleryAccess {
  const _DesignGalleryAccess({required this.available});
  

@override final  bool available;

/// Create a copy of DesignGalleryAccess
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DesignGalleryAccessCopyWith<_DesignGalleryAccess> get copyWith => __$DesignGalleryAccessCopyWithImpl<_DesignGalleryAccess>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DesignGalleryAccess&&(identical(other.available, available) || other.available == available));
}


@override
int get hashCode => Object.hash(runtimeType,available);

@override
String toString() {
  return 'DesignGalleryAccess(available: $available)';
}


}

/// @nodoc
abstract mixin class _$DesignGalleryAccessCopyWith<$Res> implements $DesignGalleryAccessCopyWith<$Res> {
  factory _$DesignGalleryAccessCopyWith(_DesignGalleryAccess value, $Res Function(_DesignGalleryAccess) _then) = __$DesignGalleryAccessCopyWithImpl;
@override @useResult
$Res call({
 bool available
});




}
/// @nodoc
class __$DesignGalleryAccessCopyWithImpl<$Res>
    implements _$DesignGalleryAccessCopyWith<$Res> {
  __$DesignGalleryAccessCopyWithImpl(this._self, this._then);

  final _DesignGalleryAccess _self;
  final $Res Function(_DesignGalleryAccess) _then;

/// Create a copy of DesignGalleryAccess
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? available = null,}) {
  return _then(_DesignGalleryAccess(
available: null == available ? _self.available : available // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
