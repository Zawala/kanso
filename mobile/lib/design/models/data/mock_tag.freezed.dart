// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_tag.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockTag {

 String get name; TagColor get color; int get count;
/// Create a copy of MockTag
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockTagCopyWith<MockTag> get copyWith => _$MockTagCopyWithImpl<MockTag>(this as MockTag, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockTag&&(identical(other.name, name) || other.name == name)&&(identical(other.color, color) || other.color == color)&&(identical(other.count, count) || other.count == count));
}


@override
int get hashCode => Object.hash(runtimeType,name,color,count);

@override
String toString() {
  return 'MockTag(name: $name, color: $color, count: $count)';
}


}

/// @nodoc
abstract mixin class $MockTagCopyWith<$Res>  {
  factory $MockTagCopyWith(MockTag value, $Res Function(MockTag) _then) = _$MockTagCopyWithImpl;
@useResult
$Res call({
 String name, TagColor color, int count
});




}
/// @nodoc
class _$MockTagCopyWithImpl<$Res>
    implements $MockTagCopyWith<$Res> {
  _$MockTagCopyWithImpl(this._self, this._then);

  final MockTag _self;
  final $Res Function(MockTag) _then;

/// Create a copy of MockTag
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? color = null,Object? count = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as TagColor,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [MockTag].
extension MockTagPatterns on MockTag {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockTag value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockTag() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockTag value)  $default,){
final _that = this;
switch (_that) {
case _MockTag():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockTag value)?  $default,){
final _that = this;
switch (_that) {
case _MockTag() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String name,  TagColor color,  int count)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockTag() when $default != null:
return $default(_that.name,_that.color,_that.count);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String name,  TagColor color,  int count)  $default,) {final _that = this;
switch (_that) {
case _MockTag():
return $default(_that.name,_that.color,_that.count);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String name,  TagColor color,  int count)?  $default,) {final _that = this;
switch (_that) {
case _MockTag() when $default != null:
return $default(_that.name,_that.color,_that.count);case _:
  return null;

}
}

}

/// @nodoc


class _MockTag implements MockTag {
  const _MockTag({required this.name, required this.color, this.count = 0});
  

@override final  String name;
@override final  TagColor color;
@override@JsonKey() final  int count;

/// Create a copy of MockTag
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockTagCopyWith<_MockTag> get copyWith => __$MockTagCopyWithImpl<_MockTag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockTag&&(identical(other.name, name) || other.name == name)&&(identical(other.color, color) || other.color == color)&&(identical(other.count, count) || other.count == count));
}


@override
int get hashCode => Object.hash(runtimeType,name,color,count);

@override
String toString() {
  return 'MockTag(name: $name, color: $color, count: $count)';
}


}

/// @nodoc
abstract mixin class _$MockTagCopyWith<$Res> implements $MockTagCopyWith<$Res> {
  factory _$MockTagCopyWith(_MockTag value, $Res Function(_MockTag) _then) = __$MockTagCopyWithImpl;
@override @useResult
$Res call({
 String name, TagColor color, int count
});




}
/// @nodoc
class __$MockTagCopyWithImpl<$Res>
    implements _$MockTagCopyWith<$Res> {
  __$MockTagCopyWithImpl(this._self, this._then);

  final _MockTag _self;
  final $Res Function(_MockTag) _then;

/// Create a copy of MockTag
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? color = null,Object? count = null,}) {
  return _then(_MockTag(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as TagColor,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
