// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'list_navigation_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ListNavigationEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListNavigationEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListNavigationEvent()';
}


}

/// @nodoc
class $ListNavigationEventCopyWith<$Res>  {
$ListNavigationEventCopyWith(ListNavigationEvent _, $Res Function(ListNavigationEvent) __);
}


/// Adds pattern-matching-related methods to [ListNavigationEvent].
extension ListNavigationEventPatterns on ListNavigationEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ListNavigationListShownEvent value)?  listShown,TResult Function( ListNavigationPinRequestedEvent value)?  pinRequested,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ListNavigationListShownEvent() when listShown != null:
return listShown(_that);case ListNavigationPinRequestedEvent() when pinRequested != null:
return pinRequested(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ListNavigationListShownEvent value)  listShown,required TResult Function( ListNavigationPinRequestedEvent value)  pinRequested,}){
final _that = this;
switch (_that) {
case ListNavigationListShownEvent():
return listShown(_that);case ListNavigationPinRequestedEvent():
return pinRequested(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ListNavigationListShownEvent value)?  listShown,TResult? Function( ListNavigationPinRequestedEvent value)?  pinRequested,}){
final _that = this;
switch (_that) {
case ListNavigationListShownEvent() when listShown != null:
return listShown(_that);case ListNavigationPinRequestedEvent() when pinRequested != null:
return pinRequested(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<String> noteIds)?  listShown,TResult Function( String noteId)?  pinRequested,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ListNavigationListShownEvent() when listShown != null:
return listShown(_that.noteIds);case ListNavigationPinRequestedEvent() when pinRequested != null:
return pinRequested(_that.noteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<String> noteIds)  listShown,required TResult Function( String noteId)  pinRequested,}) {final _that = this;
switch (_that) {
case ListNavigationListShownEvent():
return listShown(_that.noteIds);case ListNavigationPinRequestedEvent():
return pinRequested(_that.noteId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<String> noteIds)?  listShown,TResult? Function( String noteId)?  pinRequested,}) {final _that = this;
switch (_that) {
case ListNavigationListShownEvent() when listShown != null:
return listShown(_that.noteIds);case ListNavigationPinRequestedEvent() when pinRequested != null:
return pinRequested(_that.noteId);case _:
  return null;

}
}

}

/// @nodoc


class ListNavigationListShownEvent implements ListNavigationEvent {
  const ListNavigationListShownEvent(final  List<String> noteIds): _noteIds = noteIds;
  

 final  List<String> _noteIds;
 List<String> get noteIds {
  if (_noteIds is EqualUnmodifiableListView) return _noteIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_noteIds);
}


/// Create a copy of ListNavigationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListNavigationListShownEventCopyWith<ListNavigationListShownEvent> get copyWith => _$ListNavigationListShownEventCopyWithImpl<ListNavigationListShownEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListNavigationListShownEvent&&const DeepCollectionEquality().equals(other._noteIds, _noteIds));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_noteIds));

@override
String toString() {
  return 'ListNavigationEvent.listShown(noteIds: $noteIds)';
}


}

/// @nodoc
abstract mixin class $ListNavigationListShownEventCopyWith<$Res> implements $ListNavigationEventCopyWith<$Res> {
  factory $ListNavigationListShownEventCopyWith(ListNavigationListShownEvent value, $Res Function(ListNavigationListShownEvent) _then) = _$ListNavigationListShownEventCopyWithImpl;
@useResult
$Res call({
 List<String> noteIds
});




}
/// @nodoc
class _$ListNavigationListShownEventCopyWithImpl<$Res>
    implements $ListNavigationListShownEventCopyWith<$Res> {
  _$ListNavigationListShownEventCopyWithImpl(this._self, this._then);

  final ListNavigationListShownEvent _self;
  final $Res Function(ListNavigationListShownEvent) _then;

/// Create a copy of ListNavigationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteIds = null,}) {
  return _then(ListNavigationListShownEvent(
null == noteIds ? _self._noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class ListNavigationPinRequestedEvent implements ListNavigationEvent {
  const ListNavigationPinRequestedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of ListNavigationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListNavigationPinRequestedEventCopyWith<ListNavigationPinRequestedEvent> get copyWith => _$ListNavigationPinRequestedEventCopyWithImpl<ListNavigationPinRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListNavigationPinRequestedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'ListNavigationEvent.pinRequested(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $ListNavigationPinRequestedEventCopyWith<$Res> implements $ListNavigationEventCopyWith<$Res> {
  factory $ListNavigationPinRequestedEventCopyWith(ListNavigationPinRequestedEvent value, $Res Function(ListNavigationPinRequestedEvent) _then) = _$ListNavigationPinRequestedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$ListNavigationPinRequestedEventCopyWithImpl<$Res>
    implements $ListNavigationPinRequestedEventCopyWith<$Res> {
  _$ListNavigationPinRequestedEventCopyWithImpl(this._self, this._then);

  final ListNavigationPinRequestedEvent _self;
  final $Res Function(ListNavigationPinRequestedEvent) _then;

/// Create a copy of ListNavigationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(ListNavigationPinRequestedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ListNavigationState {

/// The ids of the notes of the list on screen, in order.
 List<String> get noteIds;
/// Create a copy of ListNavigationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListNavigationStateCopyWith<ListNavigationState> get copyWith => _$ListNavigationStateCopyWithImpl<ListNavigationState>(this as ListNavigationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListNavigationState&&const DeepCollectionEquality().equals(other.noteIds, noteIds));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(noteIds));

@override
String toString() {
  return 'ListNavigationState(noteIds: $noteIds)';
}


}

/// @nodoc
abstract mixin class $ListNavigationStateCopyWith<$Res>  {
  factory $ListNavigationStateCopyWith(ListNavigationState value, $Res Function(ListNavigationState) _then) = _$ListNavigationStateCopyWithImpl;
@useResult
$Res call({
 List<String> noteIds
});




}
/// @nodoc
class _$ListNavigationStateCopyWithImpl<$Res>
    implements $ListNavigationStateCopyWith<$Res> {
  _$ListNavigationStateCopyWithImpl(this._self, this._then);

  final ListNavigationState _self;
  final $Res Function(ListNavigationState) _then;

/// Create a copy of ListNavigationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteIds = null,}) {
  return _then(_self.copyWith(
noteIds: null == noteIds ? _self.noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

}


/// Adds pattern-matching-related methods to [ListNavigationState].
extension ListNavigationStatePatterns on ListNavigationState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ListNavigationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ListNavigationState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ListNavigationState value)  $default,){
final _that = this;
switch (_that) {
case _ListNavigationState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ListNavigationState value)?  $default,){
final _that = this;
switch (_that) {
case _ListNavigationState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<String> noteIds)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ListNavigationState() when $default != null:
return $default(_that.noteIds);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<String> noteIds)  $default,) {final _that = this;
switch (_that) {
case _ListNavigationState():
return $default(_that.noteIds);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<String> noteIds)?  $default,) {final _that = this;
switch (_that) {
case _ListNavigationState() when $default != null:
return $default(_that.noteIds);case _:
  return null;

}
}

}

/// @nodoc


class _ListNavigationState implements ListNavigationState {
  const _ListNavigationState({final  List<String> noteIds = const <String>[]}): _noteIds = noteIds;
  

/// The ids of the notes of the list on screen, in order.
 final  List<String> _noteIds;
/// The ids of the notes of the list on screen, in order.
@override@JsonKey() List<String> get noteIds {
  if (_noteIds is EqualUnmodifiableListView) return _noteIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_noteIds);
}


/// Create a copy of ListNavigationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ListNavigationStateCopyWith<_ListNavigationState> get copyWith => __$ListNavigationStateCopyWithImpl<_ListNavigationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ListNavigationState&&const DeepCollectionEquality().equals(other._noteIds, _noteIds));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_noteIds));

@override
String toString() {
  return 'ListNavigationState(noteIds: $noteIds)';
}


}

/// @nodoc
abstract mixin class _$ListNavigationStateCopyWith<$Res> implements $ListNavigationStateCopyWith<$Res> {
  factory _$ListNavigationStateCopyWith(_ListNavigationState value, $Res Function(_ListNavigationState) _then) = __$ListNavigationStateCopyWithImpl;
@override @useResult
$Res call({
 List<String> noteIds
});




}
/// @nodoc
class __$ListNavigationStateCopyWithImpl<$Res>
    implements _$ListNavigationStateCopyWith<$Res> {
  __$ListNavigationStateCopyWithImpl(this._self, this._then);

  final _ListNavigationState _self;
  final $Res Function(_ListNavigationState) _then;

/// Create a copy of ListNavigationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteIds = null,}) {
  return _then(_ListNavigationState(
noteIds: null == noteIds ? _self._noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
