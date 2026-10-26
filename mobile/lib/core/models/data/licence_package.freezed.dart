// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'licence_package.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LicencePackage {

/// The package's name.
 String get name;/// The text of each licence, its paragraphs joined by blank lines.
 List<String> get licences;
/// Create a copy of LicencePackage
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LicencePackageCopyWith<LicencePackage> get copyWith => _$LicencePackageCopyWithImpl<LicencePackage>(this as LicencePackage, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LicencePackage&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other.licences, licences));
}


@override
int get hashCode => Object.hash(runtimeType,name,const DeepCollectionEquality().hash(licences));

@override
String toString() {
  return 'LicencePackage(name: $name, licences: $licences)';
}


}

/// @nodoc
abstract mixin class $LicencePackageCopyWith<$Res>  {
  factory $LicencePackageCopyWith(LicencePackage value, $Res Function(LicencePackage) _then) = _$LicencePackageCopyWithImpl;
@useResult
$Res call({
 String name, List<String> licences
});




}
/// @nodoc
class _$LicencePackageCopyWithImpl<$Res>
    implements $LicencePackageCopyWith<$Res> {
  _$LicencePackageCopyWithImpl(this._self, this._then);

  final LicencePackage _self;
  final $Res Function(LicencePackage) _then;

/// Create a copy of LicencePackage
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? licences = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,licences: null == licences ? _self.licences : licences // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [LicencePackage].
extension LicencePackagePatterns on LicencePackage {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LicencePackage value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LicencePackage() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LicencePackage value)  $default,){
final _that = this;
switch (_that) {
case _LicencePackage():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LicencePackage value)?  $default,){
final _that = this;
switch (_that) {
case _LicencePackage() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String name,  List<String> licences)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LicencePackage() when $default != null:
return $default(_that.name,_that.licences);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String name,  List<String> licences)  $default,) {final _that = this;
switch (_that) {
case _LicencePackage():
return $default(_that.name,_that.licences);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String name,  List<String> licences)?  $default,) {final _that = this;
switch (_that) {
case _LicencePackage() when $default != null:
return $default(_that.name,_that.licences);case _:
  return null;

}
}

}

/// @nodoc


class _LicencePackage implements LicencePackage {
  const _LicencePackage({required this.name, required final  List<String> licences}): _licences = licences;
  

/// The package's name.
@override final  String name;
/// The text of each licence, its paragraphs joined by blank lines.
 final  List<String> _licences;
/// The text of each licence, its paragraphs joined by blank lines.
@override List<String> get licences {
  if (_licences is EqualUnmodifiableListView) return _licences;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_licences);
}


/// Create a copy of LicencePackage
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LicencePackageCopyWith<_LicencePackage> get copyWith => __$LicencePackageCopyWithImpl<_LicencePackage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LicencePackage&&(identical(other.name, name) || other.name == name)&&const DeepCollectionEquality().equals(other._licences, _licences));
}


@override
int get hashCode => Object.hash(runtimeType,name,const DeepCollectionEquality().hash(_licences));

@override
String toString() {
  return 'LicencePackage(name: $name, licences: $licences)';
}


}

/// @nodoc
abstract mixin class _$LicencePackageCopyWith<$Res> implements $LicencePackageCopyWith<$Res> {
  factory _$LicencePackageCopyWith(_LicencePackage value, $Res Function(_LicencePackage) _then) = __$LicencePackageCopyWithImpl;
@override @useResult
$Res call({
 String name, List<String> licences
});




}
/// @nodoc
class __$LicencePackageCopyWithImpl<$Res>
    implements _$LicencePackageCopyWith<$Res> {
  __$LicencePackageCopyWithImpl(this._self, this._then);

  final _LicencePackage _self;
  final $Res Function(_LicencePackage) _then;

/// Create a copy of LicencePackage
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? licences = null,}) {
  return _then(_LicencePackage(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,licences: null == licences ? _self._licences : licences // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
