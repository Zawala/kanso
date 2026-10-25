// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_completion.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockCompletion {

 String get title; String get notebook;
/// Create a copy of MockCompletion
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockCompletionCopyWith<MockCompletion> get copyWith => _$MockCompletionCopyWithImpl<MockCompletion>(this as MockCompletion, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockCompletion&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook));
}


@override
int get hashCode => Object.hash(runtimeType,title,notebook);

@override
String toString() {
  return 'MockCompletion(title: $title, notebook: $notebook)';
}


}

/// @nodoc
abstract mixin class $MockCompletionCopyWith<$Res>  {
  factory $MockCompletionCopyWith(MockCompletion value, $Res Function(MockCompletion) _then) = _$MockCompletionCopyWithImpl;
@useResult
$Res call({
 String title, String notebook
});




}
/// @nodoc
class _$MockCompletionCopyWithImpl<$Res>
    implements $MockCompletionCopyWith<$Res> {
  _$MockCompletionCopyWithImpl(this._self, this._then);

  final MockCompletion _self;
  final $Res Function(MockCompletion) _then;

/// Create a copy of MockCompletion
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? title = null,Object? notebook = null,}) {
  return _then(_self.copyWith(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [MockCompletion].
extension MockCompletionPatterns on MockCompletion {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockCompletion value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockCompletion() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockCompletion value)  $default,){
final _that = this;
switch (_that) {
case _MockCompletion():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockCompletion value)?  $default,){
final _that = this;
switch (_that) {
case _MockCompletion() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String title,  String notebook)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockCompletion() when $default != null:
return $default(_that.title,_that.notebook);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String title,  String notebook)  $default,) {final _that = this;
switch (_that) {
case _MockCompletion():
return $default(_that.title,_that.notebook);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String title,  String notebook)?  $default,) {final _that = this;
switch (_that) {
case _MockCompletion() when $default != null:
return $default(_that.title,_that.notebook);case _:
  return null;

}
}

}

/// @nodoc


class _MockCompletion implements MockCompletion {
  const _MockCompletion({required this.title, required this.notebook});
  

@override final  String title;
@override final  String notebook;

/// Create a copy of MockCompletion
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockCompletionCopyWith<_MockCompletion> get copyWith => __$MockCompletionCopyWithImpl<_MockCompletion>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockCompletion&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook));
}


@override
int get hashCode => Object.hash(runtimeType,title,notebook);

@override
String toString() {
  return 'MockCompletion(title: $title, notebook: $notebook)';
}


}

/// @nodoc
abstract mixin class _$MockCompletionCopyWith<$Res> implements $MockCompletionCopyWith<$Res> {
  factory _$MockCompletionCopyWith(_MockCompletion value, $Res Function(_MockCompletion) _then) = __$MockCompletionCopyWithImpl;
@override @useResult
$Res call({
 String title, String notebook
});




}
/// @nodoc
class __$MockCompletionCopyWithImpl<$Res>
    implements _$MockCompletionCopyWith<$Res> {
  __$MockCompletionCopyWithImpl(this._self, this._then);

  final _MockCompletion _self;
  final $Res Function(_MockCompletion) _then;

/// Create a copy of MockCompletion
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? title = null,Object? notebook = null,}) {
  return _then(_MockCompletion(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
