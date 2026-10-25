// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_notebook.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockNotebook {

 String get name; int get count; int get depth;
/// Create a copy of MockNotebook
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockNotebookCopyWith<MockNotebook> get copyWith => _$MockNotebookCopyWithImpl<MockNotebook>(this as MockNotebook, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockNotebook&&(identical(other.name, name) || other.name == name)&&(identical(other.count, count) || other.count == count)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,name,count,depth);

@override
String toString() {
  return 'MockNotebook(name: $name, count: $count, depth: $depth)';
}


}

/// @nodoc
abstract mixin class $MockNotebookCopyWith<$Res>  {
  factory $MockNotebookCopyWith(MockNotebook value, $Res Function(MockNotebook) _then) = _$MockNotebookCopyWithImpl;
@useResult
$Res call({
 String name, int count, int depth
});




}
/// @nodoc
class _$MockNotebookCopyWithImpl<$Res>
    implements $MockNotebookCopyWith<$Res> {
  _$MockNotebookCopyWithImpl(this._self, this._then);

  final MockNotebook _self;
  final $Res Function(MockNotebook) _then;

/// Create a copy of MockNotebook
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? count = null,Object? depth = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [MockNotebook].
extension MockNotebookPatterns on MockNotebook {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockNotebook value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockNotebook() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockNotebook value)  $default,){
final _that = this;
switch (_that) {
case _MockNotebook():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockNotebook value)?  $default,){
final _that = this;
switch (_that) {
case _MockNotebook() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String name,  int count,  int depth)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockNotebook() when $default != null:
return $default(_that.name,_that.count,_that.depth);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String name,  int count,  int depth)  $default,) {final _that = this;
switch (_that) {
case _MockNotebook():
return $default(_that.name,_that.count,_that.depth);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String name,  int count,  int depth)?  $default,) {final _that = this;
switch (_that) {
case _MockNotebook() when $default != null:
return $default(_that.name,_that.count,_that.depth);case _:
  return null;

}
}

}

/// @nodoc


class _MockNotebook implements MockNotebook {
  const _MockNotebook({required this.name, required this.count, this.depth = 0});
  

@override final  String name;
@override final  int count;
@override@JsonKey() final  int depth;

/// Create a copy of MockNotebook
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockNotebookCopyWith<_MockNotebook> get copyWith => __$MockNotebookCopyWithImpl<_MockNotebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockNotebook&&(identical(other.name, name) || other.name == name)&&(identical(other.count, count) || other.count == count)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,name,count,depth);

@override
String toString() {
  return 'MockNotebook(name: $name, count: $count, depth: $depth)';
}


}

/// @nodoc
abstract mixin class _$MockNotebookCopyWith<$Res> implements $MockNotebookCopyWith<$Res> {
  factory _$MockNotebookCopyWith(_MockNotebook value, $Res Function(_MockNotebook) _then) = __$MockNotebookCopyWithImpl;
@override @useResult
$Res call({
 String name, int count, int depth
});




}
/// @nodoc
class __$MockNotebookCopyWithImpl<$Res>
    implements _$MockNotebookCopyWith<$Res> {
  __$MockNotebookCopyWithImpl(this._self, this._then);

  final _MockNotebook _self;
  final $Res Function(_MockNotebook) _then;

/// Create a copy of MockNotebook
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? count = null,Object? depth = null,}) {
  return _then(_MockNotebook(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as int,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
