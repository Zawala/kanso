// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_task_item.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockTaskItem {

 String get text; bool get done;
/// Create a copy of MockTaskItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockTaskItemCopyWith<MockTaskItem> get copyWith => _$MockTaskItemCopyWithImpl<MockTaskItem>(this as MockTaskItem, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockTaskItem&&(identical(other.text, text) || other.text == text)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,text,done);

@override
String toString() {
  return 'MockTaskItem(text: $text, done: $done)';
}


}

/// @nodoc
abstract mixin class $MockTaskItemCopyWith<$Res>  {
  factory $MockTaskItemCopyWith(MockTaskItem value, $Res Function(MockTaskItem) _then) = _$MockTaskItemCopyWithImpl;
@useResult
$Res call({
 String text, bool done
});




}
/// @nodoc
class _$MockTaskItemCopyWithImpl<$Res>
    implements $MockTaskItemCopyWith<$Res> {
  _$MockTaskItemCopyWithImpl(this._self, this._then);

  final MockTaskItem _self;
  final $Res Function(MockTaskItem) _then;

/// Create a copy of MockTaskItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? text = null,Object? done = null,}) {
  return _then(_self.copyWith(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,done: null == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [MockTaskItem].
extension MockTaskItemPatterns on MockTaskItem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockTaskItem value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockTaskItem() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockTaskItem value)  $default,){
final _that = this;
switch (_that) {
case _MockTaskItem():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockTaskItem value)?  $default,){
final _that = this;
switch (_that) {
case _MockTaskItem() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String text,  bool done)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockTaskItem() when $default != null:
return $default(_that.text,_that.done);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String text,  bool done)  $default,) {final _that = this;
switch (_that) {
case _MockTaskItem():
return $default(_that.text,_that.done);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String text,  bool done)?  $default,) {final _that = this;
switch (_that) {
case _MockTaskItem() when $default != null:
return $default(_that.text,_that.done);case _:
  return null;

}
}

}

/// @nodoc


class _MockTaskItem implements MockTaskItem {
  const _MockTaskItem({required this.text, this.done = false});
  

@override final  String text;
@override@JsonKey() final  bool done;

/// Create a copy of MockTaskItem
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockTaskItemCopyWith<_MockTaskItem> get copyWith => __$MockTaskItemCopyWithImpl<_MockTaskItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockTaskItem&&(identical(other.text, text) || other.text == text)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,text,done);

@override
String toString() {
  return 'MockTaskItem(text: $text, done: $done)';
}


}

/// @nodoc
abstract mixin class _$MockTaskItemCopyWith<$Res> implements $MockTaskItemCopyWith<$Res> {
  factory _$MockTaskItemCopyWith(_MockTaskItem value, $Res Function(_MockTaskItem) _then) = __$MockTaskItemCopyWithImpl;
@override @useResult
$Res call({
 String text, bool done
});




}
/// @nodoc
class __$MockTaskItemCopyWithImpl<$Res>
    implements _$MockTaskItemCopyWith<$Res> {
  __$MockTaskItemCopyWithImpl(this._self, this._then);

  final _MockTaskItem _self;
  final $Res Function(_MockTaskItem) _then;

/// Create a copy of MockTaskItem
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? done = null,}) {
  return _then(_MockTaskItem(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,done: null == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
