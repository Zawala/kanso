// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_code_token.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockCodeToken {

 String get text; MockTokenKind get kind;
/// Create a copy of MockCodeToken
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockCodeTokenCopyWith<MockCodeToken> get copyWith => _$MockCodeTokenCopyWithImpl<MockCodeToken>(this as MockCodeToken, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockCodeToken&&(identical(other.text, text) || other.text == text)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,text,kind);

@override
String toString() {
  return 'MockCodeToken(text: $text, kind: $kind)';
}


}

/// @nodoc
abstract mixin class $MockCodeTokenCopyWith<$Res>  {
  factory $MockCodeTokenCopyWith(MockCodeToken value, $Res Function(MockCodeToken) _then) = _$MockCodeTokenCopyWithImpl;
@useResult
$Res call({
 String text, MockTokenKind kind
});




}
/// @nodoc
class _$MockCodeTokenCopyWithImpl<$Res>
    implements $MockCodeTokenCopyWith<$Res> {
  _$MockCodeTokenCopyWithImpl(this._self, this._then);

  final MockCodeToken _self;
  final $Res Function(MockCodeToken) _then;

/// Create a copy of MockCodeToken
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? text = null,Object? kind = null,}) {
  return _then(_self.copyWith(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as MockTokenKind,
  ));
}

}


/// Adds pattern-matching-related methods to [MockCodeToken].
extension MockCodeTokenPatterns on MockCodeToken {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockCodeToken value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockCodeToken() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockCodeToken value)  $default,){
final _that = this;
switch (_that) {
case _MockCodeToken():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockCodeToken value)?  $default,){
final _that = this;
switch (_that) {
case _MockCodeToken() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String text,  MockTokenKind kind)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockCodeToken() when $default != null:
return $default(_that.text,_that.kind);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String text,  MockTokenKind kind)  $default,) {final _that = this;
switch (_that) {
case _MockCodeToken():
return $default(_that.text,_that.kind);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String text,  MockTokenKind kind)?  $default,) {final _that = this;
switch (_that) {
case _MockCodeToken() when $default != null:
return $default(_that.text,_that.kind);case _:
  return null;

}
}

}

/// @nodoc


class _MockCodeToken implements MockCodeToken {
  const _MockCodeToken(this.text, this.kind);
  

@override final  String text;
@override final  MockTokenKind kind;

/// Create a copy of MockCodeToken
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockCodeTokenCopyWith<_MockCodeToken> get copyWith => __$MockCodeTokenCopyWithImpl<_MockCodeToken>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockCodeToken&&(identical(other.text, text) || other.text == text)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,text,kind);

@override
String toString() {
  return 'MockCodeToken(text: $text, kind: $kind)';
}


}

/// @nodoc
abstract mixin class _$MockCodeTokenCopyWith<$Res> implements $MockCodeTokenCopyWith<$Res> {
  factory _$MockCodeTokenCopyWith(_MockCodeToken value, $Res Function(_MockCodeToken) _then) = __$MockCodeTokenCopyWithImpl;
@override @useResult
$Res call({
 String text, MockTokenKind kind
});




}
/// @nodoc
class __$MockCodeTokenCopyWithImpl<$Res>
    implements _$MockCodeTokenCopyWith<$Res> {
  __$MockCodeTokenCopyWithImpl(this._self, this._then);

  final _MockCodeToken _self;
  final $Res Function(_MockCodeToken) _then;

/// Create a copy of MockCodeToken
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? kind = null,}) {
  return _then(_MockCodeToken(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as MockTokenKind,
  ));
}


}

// dart format on
