// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_search_result.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockSearchResult {

 String get title; String get notebook; String get snippet;
/// Create a copy of MockSearchResult
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockSearchResultCopyWith<MockSearchResult> get copyWith => _$MockSearchResultCopyWithImpl<MockSearchResult>(this as MockSearchResult, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockSearchResult&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.snippet, snippet) || other.snippet == snippet));
}


@override
int get hashCode => Object.hash(runtimeType,title,notebook,snippet);

@override
String toString() {
  return 'MockSearchResult(title: $title, notebook: $notebook, snippet: $snippet)';
}


}

/// @nodoc
abstract mixin class $MockSearchResultCopyWith<$Res>  {
  factory $MockSearchResultCopyWith(MockSearchResult value, $Res Function(MockSearchResult) _then) = _$MockSearchResultCopyWithImpl;
@useResult
$Res call({
 String title, String notebook, String snippet
});




}
/// @nodoc
class _$MockSearchResultCopyWithImpl<$Res>
    implements $MockSearchResultCopyWith<$Res> {
  _$MockSearchResultCopyWithImpl(this._self, this._then);

  final MockSearchResult _self;
  final $Res Function(MockSearchResult) _then;

/// Create a copy of MockSearchResult
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? title = null,Object? notebook = null,Object? snippet = null,}) {
  return _then(_self.copyWith(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,snippet: null == snippet ? _self.snippet : snippet // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [MockSearchResult].
extension MockSearchResultPatterns on MockSearchResult {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockSearchResult value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockSearchResult() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockSearchResult value)  $default,){
final _that = this;
switch (_that) {
case _MockSearchResult():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockSearchResult value)?  $default,){
final _that = this;
switch (_that) {
case _MockSearchResult() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String title,  String notebook,  String snippet)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockSearchResult() when $default != null:
return $default(_that.title,_that.notebook,_that.snippet);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String title,  String notebook,  String snippet)  $default,) {final _that = this;
switch (_that) {
case _MockSearchResult():
return $default(_that.title,_that.notebook,_that.snippet);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String title,  String notebook,  String snippet)?  $default,) {final _that = this;
switch (_that) {
case _MockSearchResult() when $default != null:
return $default(_that.title,_that.notebook,_that.snippet);case _:
  return null;

}
}

}

/// @nodoc


class _MockSearchResult implements MockSearchResult {
  const _MockSearchResult({required this.title, required this.notebook, required this.snippet});
  

@override final  String title;
@override final  String notebook;
@override final  String snippet;

/// Create a copy of MockSearchResult
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockSearchResultCopyWith<_MockSearchResult> get copyWith => __$MockSearchResultCopyWithImpl<_MockSearchResult>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockSearchResult&&(identical(other.title, title) || other.title == title)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.snippet, snippet) || other.snippet == snippet));
}


@override
int get hashCode => Object.hash(runtimeType,title,notebook,snippet);

@override
String toString() {
  return 'MockSearchResult(title: $title, notebook: $notebook, snippet: $snippet)';
}


}

/// @nodoc
abstract mixin class _$MockSearchResultCopyWith<$Res> implements $MockSearchResultCopyWith<$Res> {
  factory _$MockSearchResultCopyWith(_MockSearchResult value, $Res Function(_MockSearchResult) _then) = __$MockSearchResultCopyWithImpl;
@override @useResult
$Res call({
 String title, String notebook, String snippet
});




}
/// @nodoc
class __$MockSearchResultCopyWithImpl<$Res>
    implements _$MockSearchResultCopyWith<$Res> {
  __$MockSearchResultCopyWithImpl(this._self, this._then);

  final _MockSearchResult _self;
  final $Res Function(_MockSearchResult) _then;

/// Create a copy of MockSearchResult
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? title = null,Object? notebook = null,Object? snippet = null,}) {
  return _then(_MockSearchResult(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,snippet: null == snippet ? _self.snippet : snippet // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
