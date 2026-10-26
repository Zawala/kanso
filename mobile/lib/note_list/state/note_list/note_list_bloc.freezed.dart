// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_list_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteListEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEvent()';
}


}

/// @nodoc
class $NoteListEventCopyWith<$Res>  {
$NoteListEventCopyWith(NoteListEvent _, $Res Function(NoteListEvent) __);
}


/// Adds pattern-matching-related methods to [NoteListEvent].
extension NoteListEventPatterns on NoteListEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteListStartedEvent value)?  started,TResult Function( NoteListOptionsChangedEvent value)?  optionsChanged,TResult Function( NoteListChangesReceivedEvent value)?  changesReceived,TResult Function( NoteListRetriedEvent value)?  retried,TResult Function( NoteListSelectionStartedEvent value)?  selectionStarted,TResult Function( NoteListSelectionToggledEvent value)?  selectionToggled,TResult Function( NoteListSelectionClearedEvent value)?  selectionCleared,TResult Function( NoteListChangeRequestedEvent value)?  changeRequested,TResult Function( NoteListSwipeArmedEvent value)?  swipeArmed,TResult Function( NoteListErrorDismissedEvent value)?  errorDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteListStartedEvent() when started != null:
return started(_that);case NoteListOptionsChangedEvent() when optionsChanged != null:
return optionsChanged(_that);case NoteListChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case NoteListRetriedEvent() when retried != null:
return retried(_that);case NoteListSelectionStartedEvent() when selectionStarted != null:
return selectionStarted(_that);case NoteListSelectionToggledEvent() when selectionToggled != null:
return selectionToggled(_that);case NoteListSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case NoteListChangeRequestedEvent() when changeRequested != null:
return changeRequested(_that);case NoteListSwipeArmedEvent() when swipeArmed != null:
return swipeArmed(_that);case NoteListErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteListStartedEvent value)  started,required TResult Function( NoteListOptionsChangedEvent value)  optionsChanged,required TResult Function( NoteListChangesReceivedEvent value)  changesReceived,required TResult Function( NoteListRetriedEvent value)  retried,required TResult Function( NoteListSelectionStartedEvent value)  selectionStarted,required TResult Function( NoteListSelectionToggledEvent value)  selectionToggled,required TResult Function( NoteListSelectionClearedEvent value)  selectionCleared,required TResult Function( NoteListChangeRequestedEvent value)  changeRequested,required TResult Function( NoteListSwipeArmedEvent value)  swipeArmed,required TResult Function( NoteListErrorDismissedEvent value)  errorDismissed,}){
final _that = this;
switch (_that) {
case NoteListStartedEvent():
return started(_that);case NoteListOptionsChangedEvent():
return optionsChanged(_that);case NoteListChangesReceivedEvent():
return changesReceived(_that);case NoteListRetriedEvent():
return retried(_that);case NoteListSelectionStartedEvent():
return selectionStarted(_that);case NoteListSelectionToggledEvent():
return selectionToggled(_that);case NoteListSelectionClearedEvent():
return selectionCleared(_that);case NoteListChangeRequestedEvent():
return changeRequested(_that);case NoteListSwipeArmedEvent():
return swipeArmed(_that);case NoteListErrorDismissedEvent():
return errorDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteListStartedEvent value)?  started,TResult? Function( NoteListOptionsChangedEvent value)?  optionsChanged,TResult? Function( NoteListChangesReceivedEvent value)?  changesReceived,TResult? Function( NoteListRetriedEvent value)?  retried,TResult? Function( NoteListSelectionStartedEvent value)?  selectionStarted,TResult? Function( NoteListSelectionToggledEvent value)?  selectionToggled,TResult? Function( NoteListSelectionClearedEvent value)?  selectionCleared,TResult? Function( NoteListChangeRequestedEvent value)?  changeRequested,TResult? Function( NoteListSwipeArmedEvent value)?  swipeArmed,TResult? Function( NoteListErrorDismissedEvent value)?  errorDismissed,}){
final _that = this;
switch (_that) {
case NoteListStartedEvent() when started != null:
return started(_that);case NoteListOptionsChangedEvent() when optionsChanged != null:
return optionsChanged(_that);case NoteListChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case NoteListRetriedEvent() when retried != null:
return retried(_that);case NoteListSelectionStartedEvent() when selectionStarted != null:
return selectionStarted(_that);case NoteListSelectionToggledEvent() when selectionToggled != null:
return selectionToggled(_that);case NoteListSelectionClearedEvent() when selectionCleared != null:
return selectionCleared(_that);case NoteListChangeRequestedEvent() when changeRequested != null:
return changeRequested(_that);case NoteListSwipeArmedEvent() when swipeArmed != null:
return swipeArmed(_that);case NoteListErrorDismissedEvent() when errorDismissed != null:
return errorDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ListPlace place)?  started,TResult Function( ListOptions options)?  optionsChanged,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( String id)?  selectionStarted,TResult Function( String id)?  selectionToggled,TResult Function()?  selectionCleared,TResult Function( NoteChange change,  List<String> ids)?  changeRequested,TResult Function()?  swipeArmed,TResult Function()?  errorDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteListStartedEvent() when started != null:
return started(_that.place);case NoteListOptionsChangedEvent() when optionsChanged != null:
return optionsChanged(_that.options);case NoteListChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case NoteListRetriedEvent() when retried != null:
return retried();case NoteListSelectionStartedEvent() when selectionStarted != null:
return selectionStarted(_that.id);case NoteListSelectionToggledEvent() when selectionToggled != null:
return selectionToggled(_that.id);case NoteListSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case NoteListChangeRequestedEvent() when changeRequested != null:
return changeRequested(_that.change,_that.ids);case NoteListSwipeArmedEvent() when swipeArmed != null:
return swipeArmed();case NoteListErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ListPlace place)  started,required TResult Function( ListOptions options)  optionsChanged,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( String id)  selectionStarted,required TResult Function( String id)  selectionToggled,required TResult Function()  selectionCleared,required TResult Function( NoteChange change,  List<String> ids)  changeRequested,required TResult Function()  swipeArmed,required TResult Function()  errorDismissed,}) {final _that = this;
switch (_that) {
case NoteListStartedEvent():
return started(_that.place);case NoteListOptionsChangedEvent():
return optionsChanged(_that.options);case NoteListChangesReceivedEvent():
return changesReceived(_that.ids);case NoteListRetriedEvent():
return retried();case NoteListSelectionStartedEvent():
return selectionStarted(_that.id);case NoteListSelectionToggledEvent():
return selectionToggled(_that.id);case NoteListSelectionClearedEvent():
return selectionCleared();case NoteListChangeRequestedEvent():
return changeRequested(_that.change,_that.ids);case NoteListSwipeArmedEvent():
return swipeArmed();case NoteListErrorDismissedEvent():
return errorDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ListPlace place)?  started,TResult? Function( ListOptions options)?  optionsChanged,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( String id)?  selectionStarted,TResult? Function( String id)?  selectionToggled,TResult? Function()?  selectionCleared,TResult? Function( NoteChange change,  List<String> ids)?  changeRequested,TResult? Function()?  swipeArmed,TResult? Function()?  errorDismissed,}) {final _that = this;
switch (_that) {
case NoteListStartedEvent() when started != null:
return started(_that.place);case NoteListOptionsChangedEvent() when optionsChanged != null:
return optionsChanged(_that.options);case NoteListChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case NoteListRetriedEvent() when retried != null:
return retried();case NoteListSelectionStartedEvent() when selectionStarted != null:
return selectionStarted(_that.id);case NoteListSelectionToggledEvent() when selectionToggled != null:
return selectionToggled(_that.id);case NoteListSelectionClearedEvent() when selectionCleared != null:
return selectionCleared();case NoteListChangeRequestedEvent() when changeRequested != null:
return changeRequested(_that.change,_that.ids);case NoteListSwipeArmedEvent() when swipeArmed != null:
return swipeArmed();case NoteListErrorDismissedEvent() when errorDismissed != null:
return errorDismissed();case _:
  return null;

}
}

}

/// @nodoc


class NoteListStartedEvent implements NoteListEvent {
  const NoteListStartedEvent(this.place);
  

 final  ListPlace place;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListStartedEventCopyWith<NoteListStartedEvent> get copyWith => _$NoteListStartedEventCopyWithImpl<NoteListStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListStartedEvent&&(identical(other.place, place) || other.place == place));
}


@override
int get hashCode => Object.hash(runtimeType,place);

@override
String toString() {
  return 'NoteListEvent.started(place: $place)';
}


}

/// @nodoc
abstract mixin class $NoteListStartedEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListStartedEventCopyWith(NoteListStartedEvent value, $Res Function(NoteListStartedEvent) _then) = _$NoteListStartedEventCopyWithImpl;
@useResult
$Res call({
 ListPlace place
});


$ListPlaceCopyWith<$Res> get place;

}
/// @nodoc
class _$NoteListStartedEventCopyWithImpl<$Res>
    implements $NoteListStartedEventCopyWith<$Res> {
  _$NoteListStartedEventCopyWithImpl(this._self, this._then);

  final NoteListStartedEvent _self;
  final $Res Function(NoteListStartedEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? place = null,}) {
  return _then(NoteListStartedEvent(
null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,
  ));
}

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

/// @nodoc


class NoteListOptionsChangedEvent implements NoteListEvent {
  const NoteListOptionsChangedEvent(this.options);
  

 final  ListOptions options;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListOptionsChangedEventCopyWith<NoteListOptionsChangedEvent> get copyWith => _$NoteListOptionsChangedEventCopyWithImpl<NoteListOptionsChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListOptionsChangedEvent&&(identical(other.options, options) || other.options == options));
}


@override
int get hashCode => Object.hash(runtimeType,options);

@override
String toString() {
  return 'NoteListEvent.optionsChanged(options: $options)';
}


}

/// @nodoc
abstract mixin class $NoteListOptionsChangedEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListOptionsChangedEventCopyWith(NoteListOptionsChangedEvent value, $Res Function(NoteListOptionsChangedEvent) _then) = _$NoteListOptionsChangedEventCopyWithImpl;
@useResult
$Res call({
 ListOptions options
});


$ListOptionsCopyWith<$Res> get options;

}
/// @nodoc
class _$NoteListOptionsChangedEventCopyWithImpl<$Res>
    implements $NoteListOptionsChangedEventCopyWith<$Res> {
  _$NoteListOptionsChangedEventCopyWithImpl(this._self, this._then);

  final NoteListOptionsChangedEvent _self;
  final $Res Function(NoteListOptionsChangedEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? options = null,}) {
  return _then(NoteListOptionsChangedEvent(
null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as ListOptions,
  ));
}

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListOptionsCopyWith<$Res> get options {
  
  return $ListOptionsCopyWith<$Res>(_self.options, (value) {
    return _then(_self.copyWith(options: value));
  });
}
}

/// @nodoc


class NoteListChangesReceivedEvent implements NoteListEvent {
  const NoteListChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListChangesReceivedEventCopyWith<NoteListChangesReceivedEvent> get copyWith => _$NoteListChangesReceivedEventCopyWithImpl<NoteListChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'NoteListEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $NoteListChangesReceivedEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListChangesReceivedEventCopyWith(NoteListChangesReceivedEvent value, $Res Function(NoteListChangesReceivedEvent) _then) = _$NoteListChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$NoteListChangesReceivedEventCopyWithImpl<$Res>
    implements $NoteListChangesReceivedEventCopyWith<$Res> {
  _$NoteListChangesReceivedEventCopyWithImpl(this._self, this._then);

  final NoteListChangesReceivedEvent _self;
  final $Res Function(NoteListChangesReceivedEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(NoteListChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class NoteListRetriedEvent implements NoteListEvent {
  const NoteListRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEvent.retried()';
}


}




/// @nodoc


class NoteListSelectionStartedEvent implements NoteListEvent {
  const NoteListSelectionStartedEvent(this.id);
  

 final  String id;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListSelectionStartedEventCopyWith<NoteListSelectionStartedEvent> get copyWith => _$NoteListSelectionStartedEventCopyWithImpl<NoteListSelectionStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListSelectionStartedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'NoteListEvent.selectionStarted(id: $id)';
}


}

/// @nodoc
abstract mixin class $NoteListSelectionStartedEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListSelectionStartedEventCopyWith(NoteListSelectionStartedEvent value, $Res Function(NoteListSelectionStartedEvent) _then) = _$NoteListSelectionStartedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$NoteListSelectionStartedEventCopyWithImpl<$Res>
    implements $NoteListSelectionStartedEventCopyWith<$Res> {
  _$NoteListSelectionStartedEventCopyWithImpl(this._self, this._then);

  final NoteListSelectionStartedEvent _self;
  final $Res Function(NoteListSelectionStartedEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(NoteListSelectionStartedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteListSelectionToggledEvent implements NoteListEvent {
  const NoteListSelectionToggledEvent(this.id);
  

 final  String id;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListSelectionToggledEventCopyWith<NoteListSelectionToggledEvent> get copyWith => _$NoteListSelectionToggledEventCopyWithImpl<NoteListSelectionToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListSelectionToggledEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'NoteListEvent.selectionToggled(id: $id)';
}


}

/// @nodoc
abstract mixin class $NoteListSelectionToggledEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListSelectionToggledEventCopyWith(NoteListSelectionToggledEvent value, $Res Function(NoteListSelectionToggledEvent) _then) = _$NoteListSelectionToggledEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$NoteListSelectionToggledEventCopyWithImpl<$Res>
    implements $NoteListSelectionToggledEventCopyWith<$Res> {
  _$NoteListSelectionToggledEventCopyWithImpl(this._self, this._then);

  final NoteListSelectionToggledEvent _self;
  final $Res Function(NoteListSelectionToggledEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(NoteListSelectionToggledEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteListSelectionClearedEvent implements NoteListEvent {
  const NoteListSelectionClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListSelectionClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEvent.selectionCleared()';
}


}




/// @nodoc


class NoteListChangeRequestedEvent implements NoteListEvent {
  const NoteListChangeRequestedEvent(this.change, final  List<String> ids): _ids = ids;
  

 final  NoteChange change;
 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListChangeRequestedEventCopyWith<NoteListChangeRequestedEvent> get copyWith => _$NoteListChangeRequestedEventCopyWithImpl<NoteListChangeRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListChangeRequestedEvent&&(identical(other.change, change) || other.change == change)&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,change,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'NoteListEvent.changeRequested(change: $change, ids: $ids)';
}


}

/// @nodoc
abstract mixin class $NoteListChangeRequestedEventCopyWith<$Res> implements $NoteListEventCopyWith<$Res> {
  factory $NoteListChangeRequestedEventCopyWith(NoteListChangeRequestedEvent value, $Res Function(NoteListChangeRequestedEvent) _then) = _$NoteListChangeRequestedEventCopyWithImpl;
@useResult
$Res call({
 NoteChange change, List<String> ids
});


$NoteChangeCopyWith<$Res> get change;

}
/// @nodoc
class _$NoteListChangeRequestedEventCopyWithImpl<$Res>
    implements $NoteListChangeRequestedEventCopyWith<$Res> {
  _$NoteListChangeRequestedEventCopyWithImpl(this._self, this._then);

  final NoteListChangeRequestedEvent _self;
  final $Res Function(NoteListChangeRequestedEvent) _then;

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? change = null,Object? ids = null,}) {
  return _then(NoteListChangeRequestedEvent(
null == change ? _self.change : change // ignore: cast_nullable_to_non_nullable
as NoteChange,null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

/// Create a copy of NoteListEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NoteChangeCopyWith<$Res> get change {
  
  return $NoteChangeCopyWith<$Res>(_self.change, (value) {
    return _then(_self.copyWith(change: value));
  });
}
}

/// @nodoc


class NoteListSwipeArmedEvent implements NoteListEvent {
  const NoteListSwipeArmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListSwipeArmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEvent.swipeArmed()';
}


}




/// @nodoc


class NoteListErrorDismissedEvent implements NoteListEvent {
  const NoteListErrorDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListErrorDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEvent.errorDismissed()';
}


}




/// @nodoc
mixin _$NoteListState {

/// The place being listed.
 ListPlace get place;/// How it is sorted and narrowed.
 ListOptions get options; NoteListStatus get status;/// The notes, in list order.
 List<NoteRow> get rows;/// What the library says about laying [rows] out; null until read.
 ListShape? get shape;/// [rows] under their headers.
 List<NoteListEntry> get entries;/// The ids of the notes chosen, while the person is choosing several.
 Set<String> get selected;/// Why reading the list failed, while [status] is failed.
 ApplicationError? get error;/// Why the last change to notes failed, until it is dismissed.
 ApplicationError? get actionError;
/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteListStateCopyWith<NoteListState> get copyWith => _$NoteListStateCopyWithImpl<NoteListState>(this as NoteListState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListState&&(identical(other.place, place) || other.place == place)&&(identical(other.options, options) || other.options == options)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.rows, rows)&&(identical(other.shape, shape) || other.shape == shape)&&const DeepCollectionEquality().equals(other.entries, entries)&&const DeepCollectionEquality().equals(other.selected, selected)&&(identical(other.error, error) || other.error == error)&&(identical(other.actionError, actionError) || other.actionError == actionError));
}


@override
int get hashCode => Object.hash(runtimeType,place,options,status,const DeepCollectionEquality().hash(rows),shape,const DeepCollectionEquality().hash(entries),const DeepCollectionEquality().hash(selected),error,actionError);

@override
String toString() {
  return 'NoteListState(place: $place, options: $options, status: $status, rows: $rows, shape: $shape, entries: $entries, selected: $selected, error: $error, actionError: $actionError)';
}


}

/// @nodoc
abstract mixin class $NoteListStateCopyWith<$Res>  {
  factory $NoteListStateCopyWith(NoteListState value, $Res Function(NoteListState) _then) = _$NoteListStateCopyWithImpl;
@useResult
$Res call({
 ListPlace place, ListOptions options, NoteListStatus status, List<NoteRow> rows, ListShape? shape, List<NoteListEntry> entries, Set<String> selected, ApplicationError? error, ApplicationError? actionError
});


$ListPlaceCopyWith<$Res> get place;$ListOptionsCopyWith<$Res> get options;$ApplicationErrorCopyWith<$Res>? get error;$ApplicationErrorCopyWith<$Res>? get actionError;

}
/// @nodoc
class _$NoteListStateCopyWithImpl<$Res>
    implements $NoteListStateCopyWith<$Res> {
  _$NoteListStateCopyWithImpl(this._self, this._then);

  final NoteListState _self;
  final $Res Function(NoteListState) _then;

/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? place = null,Object? options = null,Object? status = null,Object? rows = null,Object? shape = freezed,Object? entries = null,Object? selected = null,Object? error = freezed,Object? actionError = freezed,}) {
  return _then(_self.copyWith(
place: null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as ListOptions,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteListStatus,rows: null == rows ? _self.rows : rows // ignore: cast_nullable_to_non_nullable
as List<NoteRow>,shape: freezed == shape ? _self.shape : shape // ignore: cast_nullable_to_non_nullable
as ListShape?,entries: null == entries ? _self.entries : entries // ignore: cast_nullable_to_non_nullable
as List<NoteListEntry>,selected: null == selected ? _self.selected : selected // ignore: cast_nullable_to_non_nullable
as Set<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,actionError: freezed == actionError ? _self.actionError : actionError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListOptionsCopyWith<$Res> get options {
  
  return $ListOptionsCopyWith<$Res>(_self.options, (value) {
    return _then(_self.copyWith(options: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get actionError {
    if (_self.actionError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.actionError!, (value) {
    return _then(_self.copyWith(actionError: value));
  });
}
}


/// Adds pattern-matching-related methods to [NoteListState].
extension NoteListStatePatterns on NoteListState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteListState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteListState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteListState value)  $default,){
final _that = this;
switch (_that) {
case _NoteListState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteListState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteListState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ListPlace place,  ListOptions options,  NoteListStatus status,  List<NoteRow> rows,  ListShape? shape,  List<NoteListEntry> entries,  Set<String> selected,  ApplicationError? error,  ApplicationError? actionError)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteListState() when $default != null:
return $default(_that.place,_that.options,_that.status,_that.rows,_that.shape,_that.entries,_that.selected,_that.error,_that.actionError);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ListPlace place,  ListOptions options,  NoteListStatus status,  List<NoteRow> rows,  ListShape? shape,  List<NoteListEntry> entries,  Set<String> selected,  ApplicationError? error,  ApplicationError? actionError)  $default,) {final _that = this;
switch (_that) {
case _NoteListState():
return $default(_that.place,_that.options,_that.status,_that.rows,_that.shape,_that.entries,_that.selected,_that.error,_that.actionError);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ListPlace place,  ListOptions options,  NoteListStatus status,  List<NoteRow> rows,  ListShape? shape,  List<NoteListEntry> entries,  Set<String> selected,  ApplicationError? error,  ApplicationError? actionError)?  $default,) {final _that = this;
switch (_that) {
case _NoteListState() when $default != null:
return $default(_that.place,_that.options,_that.status,_that.rows,_that.shape,_that.entries,_that.selected,_that.error,_that.actionError);case _:
  return null;

}
}

}

/// @nodoc


class _NoteListState extends NoteListState {
  const _NoteListState({this.place = const ListPlace.all(), this.options = const ListOptions(), this.status = NoteListStatus.loading, final  List<NoteRow> rows = const <NoteRow>[], this.shape, final  List<NoteListEntry> entries = const <NoteListEntry>[], final  Set<String> selected = const <String>{}, this.error, this.actionError}): _rows = rows,_entries = entries,_selected = selected,super._();
  

/// The place being listed.
@override@JsonKey() final  ListPlace place;
/// How it is sorted and narrowed.
@override@JsonKey() final  ListOptions options;
@override@JsonKey() final  NoteListStatus status;
/// The notes, in list order.
 final  List<NoteRow> _rows;
/// The notes, in list order.
@override@JsonKey() List<NoteRow> get rows {
  if (_rows is EqualUnmodifiableListView) return _rows;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_rows);
}

/// What the library says about laying [rows] out; null until read.
@override final  ListShape? shape;
/// [rows] under their headers.
 final  List<NoteListEntry> _entries;
/// [rows] under their headers.
@override@JsonKey() List<NoteListEntry> get entries {
  if (_entries is EqualUnmodifiableListView) return _entries;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_entries);
}

/// The ids of the notes chosen, while the person is choosing several.
 final  Set<String> _selected;
/// The ids of the notes chosen, while the person is choosing several.
@override@JsonKey() Set<String> get selected {
  if (_selected is EqualUnmodifiableSetView) return _selected;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_selected);
}

/// Why reading the list failed, while [status] is failed.
@override final  ApplicationError? error;
/// Why the last change to notes failed, until it is dismissed.
@override final  ApplicationError? actionError;

/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteListStateCopyWith<_NoteListState> get copyWith => __$NoteListStateCopyWithImpl<_NoteListState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteListState&&(identical(other.place, place) || other.place == place)&&(identical(other.options, options) || other.options == options)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._rows, _rows)&&(identical(other.shape, shape) || other.shape == shape)&&const DeepCollectionEquality().equals(other._entries, _entries)&&const DeepCollectionEquality().equals(other._selected, _selected)&&(identical(other.error, error) || other.error == error)&&(identical(other.actionError, actionError) || other.actionError == actionError));
}


@override
int get hashCode => Object.hash(runtimeType,place,options,status,const DeepCollectionEquality().hash(_rows),shape,const DeepCollectionEquality().hash(_entries),const DeepCollectionEquality().hash(_selected),error,actionError);

@override
String toString() {
  return 'NoteListState(place: $place, options: $options, status: $status, rows: $rows, shape: $shape, entries: $entries, selected: $selected, error: $error, actionError: $actionError)';
}


}

/// @nodoc
abstract mixin class _$NoteListStateCopyWith<$Res> implements $NoteListStateCopyWith<$Res> {
  factory _$NoteListStateCopyWith(_NoteListState value, $Res Function(_NoteListState) _then) = __$NoteListStateCopyWithImpl;
@override @useResult
$Res call({
 ListPlace place, ListOptions options, NoteListStatus status, List<NoteRow> rows, ListShape? shape, List<NoteListEntry> entries, Set<String> selected, ApplicationError? error, ApplicationError? actionError
});


@override $ListPlaceCopyWith<$Res> get place;@override $ListOptionsCopyWith<$Res> get options;@override $ApplicationErrorCopyWith<$Res>? get error;@override $ApplicationErrorCopyWith<$Res>? get actionError;

}
/// @nodoc
class __$NoteListStateCopyWithImpl<$Res>
    implements _$NoteListStateCopyWith<$Res> {
  __$NoteListStateCopyWithImpl(this._self, this._then);

  final _NoteListState _self;
  final $Res Function(_NoteListState) _then;

/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? place = null,Object? options = null,Object? status = null,Object? rows = null,Object? shape = freezed,Object? entries = null,Object? selected = null,Object? error = freezed,Object? actionError = freezed,}) {
  return _then(_NoteListState(
place: null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,options: null == options ? _self.options : options // ignore: cast_nullable_to_non_nullable
as ListOptions,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteListStatus,rows: null == rows ? _self._rows : rows // ignore: cast_nullable_to_non_nullable
as List<NoteRow>,shape: freezed == shape ? _self.shape : shape // ignore: cast_nullable_to_non_nullable
as ListShape?,entries: null == entries ? _self._entries : entries // ignore: cast_nullable_to_non_nullable
as List<NoteListEntry>,selected: null == selected ? _self._selected : selected // ignore: cast_nullable_to_non_nullable
as Set<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,actionError: freezed == actionError ? _self.actionError : actionError // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListOptionsCopyWith<$Res> get options {
  
  return $ListOptionsCopyWith<$Res>(_self.options, (value) {
    return _then(_self.copyWith(options: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}/// Create a copy of NoteListState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get actionError {
    if (_self.actionError == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.actionError!, (value) {
    return _then(_self.copyWith(actionError: value));
  });
}
}

// dart format on
