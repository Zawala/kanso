// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'navigation_history.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NavigationHistory {

/// The visits behind the open note, the nearest last.
 List<String> get back;/// The note of the current visit.
 String? get current;/// The visits ahead of the open note, the nearest last.
 List<String> get forward;
/// Create a copy of NavigationHistory
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NavigationHistoryCopyWith<NavigationHistory> get copyWith => _$NavigationHistoryCopyWithImpl<NavigationHistory>(this as NavigationHistory, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NavigationHistory&&const DeepCollectionEquality().equals(other.back, back)&&(identical(other.current, current) || other.current == current)&&const DeepCollectionEquality().equals(other.forward, forward));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(back),current,const DeepCollectionEquality().hash(forward));

@override
String toString() {
  return 'NavigationHistory(back: $back, current: $current, forward: $forward)';
}


}

/// @nodoc
abstract mixin class $NavigationHistoryCopyWith<$Res>  {
  factory $NavigationHistoryCopyWith(NavigationHistory value, $Res Function(NavigationHistory) _then) = _$NavigationHistoryCopyWithImpl;
@useResult
$Res call({
 List<String> back, String? current, List<String> forward
});




}
/// @nodoc
class _$NavigationHistoryCopyWithImpl<$Res>
    implements $NavigationHistoryCopyWith<$Res> {
  _$NavigationHistoryCopyWithImpl(this._self, this._then);

  final NavigationHistory _self;
  final $Res Function(NavigationHistory) _then;

/// Create a copy of NavigationHistory
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? back = null,Object? current = freezed,Object? forward = null,}) {
  return _then(_self.copyWith(
back: null == back ? _self.back : back // ignore: cast_nullable_to_non_nullable
as List<String>,current: freezed == current ? _self.current : current // ignore: cast_nullable_to_non_nullable
as String?,forward: null == forward ? _self.forward : forward // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [NavigationHistory].
extension NavigationHistoryPatterns on NavigationHistory {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NavigationHistory value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NavigationHistory() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NavigationHistory value)  $default,){
final _that = this;
switch (_that) {
case _NavigationHistory():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NavigationHistory value)?  $default,){
final _that = this;
switch (_that) {
case _NavigationHistory() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<String> back,  String? current,  List<String> forward)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NavigationHistory() when $default != null:
return $default(_that.back,_that.current,_that.forward);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<String> back,  String? current,  List<String> forward)  $default,) {final _that = this;
switch (_that) {
case _NavigationHistory():
return $default(_that.back,_that.current,_that.forward);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<String> back,  String? current,  List<String> forward)?  $default,) {final _that = this;
switch (_that) {
case _NavigationHistory() when $default != null:
return $default(_that.back,_that.current,_that.forward);case _:
  return null;

}
}

}

/// @nodoc


class _NavigationHistory extends NavigationHistory {
  const _NavigationHistory({final  List<String> back = const <String>[], this.current, final  List<String> forward = const <String>[]}): _back = back,_forward = forward,super._();
  

/// The visits behind the open note, the nearest last.
 final  List<String> _back;
/// The visits behind the open note, the nearest last.
@override@JsonKey() List<String> get back {
  if (_back is EqualUnmodifiableListView) return _back;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_back);
}

/// The note of the current visit.
@override final  String? current;
/// The visits ahead of the open note, the nearest last.
 final  List<String> _forward;
/// The visits ahead of the open note, the nearest last.
@override@JsonKey() List<String> get forward {
  if (_forward is EqualUnmodifiableListView) return _forward;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_forward);
}


/// Create a copy of NavigationHistory
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NavigationHistoryCopyWith<_NavigationHistory> get copyWith => __$NavigationHistoryCopyWithImpl<_NavigationHistory>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NavigationHistory&&const DeepCollectionEquality().equals(other._back, _back)&&(identical(other.current, current) || other.current == current)&&const DeepCollectionEquality().equals(other._forward, _forward));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_back),current,const DeepCollectionEquality().hash(_forward));

@override
String toString() {
  return 'NavigationHistory(back: $back, current: $current, forward: $forward)';
}


}

/// @nodoc
abstract mixin class _$NavigationHistoryCopyWith<$Res> implements $NavigationHistoryCopyWith<$Res> {
  factory _$NavigationHistoryCopyWith(_NavigationHistory value, $Res Function(_NavigationHistory) _then) = __$NavigationHistoryCopyWithImpl;
@override @useResult
$Res call({
 List<String> back, String? current, List<String> forward
});




}
/// @nodoc
class __$NavigationHistoryCopyWithImpl<$Res>
    implements _$NavigationHistoryCopyWith<$Res> {
  __$NavigationHistoryCopyWithImpl(this._self, this._then);

  final _NavigationHistory _self;
  final $Res Function(_NavigationHistory) _then;

/// Create a copy of NavigationHistory
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? back = null,Object? current = freezed,Object? forward = null,}) {
  return _then(_NavigationHistory(
back: null == back ? _self._back : back // ignore: cast_nullable_to_non_nullable
as List<String>,current: freezed == current ? _self.current : current // ignore: cast_nullable_to_non_nullable
as String?,forward: null == forward ? _self._forward : forward // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
