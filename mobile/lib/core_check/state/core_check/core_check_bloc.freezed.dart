// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'core_check_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CoreCheckEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreCheckEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CoreCheckEvent()';
}


}

/// @nodoc
class $CoreCheckEventCopyWith<$Res>  {
$CoreCheckEventCopyWith(CoreCheckEvent _, $Res Function(CoreCheckEvent) __);
}


/// Adds pattern-matching-related methods to [CoreCheckEvent].
extension CoreCheckEventPatterns on CoreCheckEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CoreCheckStartedEvent value)?  started,TResult Function( CoreCheckNoteCreatedEvent value)?  noteCreated,TResult Function( CoreCheckChangeReceivedEvent value)?  changeReceived,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CoreCheckStartedEvent() when started != null:
return started(_that);case CoreCheckNoteCreatedEvent() when noteCreated != null:
return noteCreated(_that);case CoreCheckChangeReceivedEvent() when changeReceived != null:
return changeReceived(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CoreCheckStartedEvent value)  started,required TResult Function( CoreCheckNoteCreatedEvent value)  noteCreated,required TResult Function( CoreCheckChangeReceivedEvent value)  changeReceived,}){
final _that = this;
switch (_that) {
case CoreCheckStartedEvent():
return started(_that);case CoreCheckNoteCreatedEvent():
return noteCreated(_that);case CoreCheckChangeReceivedEvent():
return changeReceived(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CoreCheckStartedEvent value)?  started,TResult? Function( CoreCheckNoteCreatedEvent value)?  noteCreated,TResult? Function( CoreCheckChangeReceivedEvent value)?  changeReceived,}){
final _that = this;
switch (_that) {
case CoreCheckStartedEvent() when started != null:
return started(_that);case CoreCheckNoteCreatedEvent() when noteCreated != null:
return noteCreated(_that);case CoreCheckChangeReceivedEvent() when changeReceived != null:
return changeReceived(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  noteCreated,TResult Function( List<String> ids)?  changeReceived,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CoreCheckStartedEvent() when started != null:
return started();case CoreCheckNoteCreatedEvent() when noteCreated != null:
return noteCreated();case CoreCheckChangeReceivedEvent() when changeReceived != null:
return changeReceived(_that.ids);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  noteCreated,required TResult Function( List<String> ids)  changeReceived,}) {final _that = this;
switch (_that) {
case CoreCheckStartedEvent():
return started();case CoreCheckNoteCreatedEvent():
return noteCreated();case CoreCheckChangeReceivedEvent():
return changeReceived(_that.ids);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  noteCreated,TResult? Function( List<String> ids)?  changeReceived,}) {final _that = this;
switch (_that) {
case CoreCheckStartedEvent() when started != null:
return started();case CoreCheckNoteCreatedEvent() when noteCreated != null:
return noteCreated();case CoreCheckChangeReceivedEvent() when changeReceived != null:
return changeReceived(_that.ids);case _:
  return null;

}
}

}

/// @nodoc


class CoreCheckStartedEvent implements CoreCheckEvent {
  const CoreCheckStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreCheckStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CoreCheckEvent.started()';
}


}




/// @nodoc


class CoreCheckNoteCreatedEvent implements CoreCheckEvent {
  const CoreCheckNoteCreatedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreCheckNoteCreatedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CoreCheckEvent.noteCreated()';
}


}




/// @nodoc


class CoreCheckChangeReceivedEvent implements CoreCheckEvent {
  const CoreCheckChangeReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of CoreCheckEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CoreCheckChangeReceivedEventCopyWith<CoreCheckChangeReceivedEvent> get copyWith => _$CoreCheckChangeReceivedEventCopyWithImpl<CoreCheckChangeReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreCheckChangeReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'CoreCheckEvent.changeReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $CoreCheckChangeReceivedEventCopyWith<$Res> implements $CoreCheckEventCopyWith<$Res> {
  factory $CoreCheckChangeReceivedEventCopyWith(CoreCheckChangeReceivedEvent value, $Res Function(CoreCheckChangeReceivedEvent) _then) = _$CoreCheckChangeReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$CoreCheckChangeReceivedEventCopyWithImpl<$Res>
    implements $CoreCheckChangeReceivedEventCopyWith<$Res> {
  _$CoreCheckChangeReceivedEventCopyWithImpl(this._self, this._then);

  final CoreCheckChangeReceivedEvent _self;
  final $Res Function(CoreCheckChangeReceivedEvent) _then;

/// Create a copy of CoreCheckEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(CoreCheckChangeReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc
mixin _$CoreCheckState {

 CoreCheckStatus get status;/// The Rust library's version.
 String get version;/// Whether the library can check a server's certificate.
 bool get tlsReady;/// The notes outside the trash.
 int get noteCount;/// The newest few notes: their titles show even when locked.
 List<NoteRow> get latest;/// The IDs in the last batch of changes the library reported.
 List<String> get lastChange;/// Why the status is failed, or why the last action did not work.
 ApplicationError? get error;
/// Create a copy of CoreCheckState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CoreCheckStateCopyWith<CoreCheckState> get copyWith => _$CoreCheckStateCopyWithImpl<CoreCheckState>(this as CoreCheckState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CoreCheckState&&(identical(other.status, status) || other.status == status)&&(identical(other.version, version) || other.version == version)&&(identical(other.tlsReady, tlsReady) || other.tlsReady == tlsReady)&&(identical(other.noteCount, noteCount) || other.noteCount == noteCount)&&const DeepCollectionEquality().equals(other.latest, latest)&&const DeepCollectionEquality().equals(other.lastChange, lastChange)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,version,tlsReady,noteCount,const DeepCollectionEquality().hash(latest),const DeepCollectionEquality().hash(lastChange),error);

@override
String toString() {
  return 'CoreCheckState(status: $status, version: $version, tlsReady: $tlsReady, noteCount: $noteCount, latest: $latest, lastChange: $lastChange, error: $error)';
}


}

/// @nodoc
abstract mixin class $CoreCheckStateCopyWith<$Res>  {
  factory $CoreCheckStateCopyWith(CoreCheckState value, $Res Function(CoreCheckState) _then) = _$CoreCheckStateCopyWithImpl;
@useResult
$Res call({
 CoreCheckStatus status, String version, bool tlsReady, int noteCount, List<NoteRow> latest, List<String> lastChange, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$CoreCheckStateCopyWithImpl<$Res>
    implements $CoreCheckStateCopyWith<$Res> {
  _$CoreCheckStateCopyWithImpl(this._self, this._then);

  final CoreCheckState _self;
  final $Res Function(CoreCheckState) _then;

/// Create a copy of CoreCheckState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? version = null,Object? tlsReady = null,Object? noteCount = null,Object? latest = null,Object? lastChange = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CoreCheckStatus,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,tlsReady: null == tlsReady ? _self.tlsReady : tlsReady // ignore: cast_nullable_to_non_nullable
as bool,noteCount: null == noteCount ? _self.noteCount : noteCount // ignore: cast_nullable_to_non_nullable
as int,latest: null == latest ? _self.latest : latest // ignore: cast_nullable_to_non_nullable
as List<NoteRow>,lastChange: null == lastChange ? _self.lastChange : lastChange // ignore: cast_nullable_to_non_nullable
as List<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of CoreCheckState
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
}
}


/// Adds pattern-matching-related methods to [CoreCheckState].
extension CoreCheckStatePatterns on CoreCheckState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CoreCheckState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CoreCheckState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CoreCheckState value)  $default,){
final _that = this;
switch (_that) {
case _CoreCheckState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CoreCheckState value)?  $default,){
final _that = this;
switch (_that) {
case _CoreCheckState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( CoreCheckStatus status,  String version,  bool tlsReady,  int noteCount,  List<NoteRow> latest,  List<String> lastChange,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CoreCheckState() when $default != null:
return $default(_that.status,_that.version,_that.tlsReady,_that.noteCount,_that.latest,_that.lastChange,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( CoreCheckStatus status,  String version,  bool tlsReady,  int noteCount,  List<NoteRow> latest,  List<String> lastChange,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _CoreCheckState():
return $default(_that.status,_that.version,_that.tlsReady,_that.noteCount,_that.latest,_that.lastChange,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( CoreCheckStatus status,  String version,  bool tlsReady,  int noteCount,  List<NoteRow> latest,  List<String> lastChange,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _CoreCheckState() when $default != null:
return $default(_that.status,_that.version,_that.tlsReady,_that.noteCount,_that.latest,_that.lastChange,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _CoreCheckState implements CoreCheckState {
  const _CoreCheckState({this.status = CoreCheckStatus.loading, this.version = '', this.tlsReady = false, this.noteCount = 0, final  List<NoteRow> latest = const <NoteRow>[], final  List<String> lastChange = const <String>[], this.error}): _latest = latest,_lastChange = lastChange;
  

@override@JsonKey() final  CoreCheckStatus status;
/// The Rust library's version.
@override@JsonKey() final  String version;
/// Whether the library can check a server's certificate.
@override@JsonKey() final  bool tlsReady;
/// The notes outside the trash.
@override@JsonKey() final  int noteCount;
/// The newest few notes: their titles show even when locked.
 final  List<NoteRow> _latest;
/// The newest few notes: their titles show even when locked.
@override@JsonKey() List<NoteRow> get latest {
  if (_latest is EqualUnmodifiableListView) return _latest;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_latest);
}

/// The IDs in the last batch of changes the library reported.
 final  List<String> _lastChange;
/// The IDs in the last batch of changes the library reported.
@override@JsonKey() List<String> get lastChange {
  if (_lastChange is EqualUnmodifiableListView) return _lastChange;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_lastChange);
}

/// Why the status is failed, or why the last action did not work.
@override final  ApplicationError? error;

/// Create a copy of CoreCheckState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CoreCheckStateCopyWith<_CoreCheckState> get copyWith => __$CoreCheckStateCopyWithImpl<_CoreCheckState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CoreCheckState&&(identical(other.status, status) || other.status == status)&&(identical(other.version, version) || other.version == version)&&(identical(other.tlsReady, tlsReady) || other.tlsReady == tlsReady)&&(identical(other.noteCount, noteCount) || other.noteCount == noteCount)&&const DeepCollectionEquality().equals(other._latest, _latest)&&const DeepCollectionEquality().equals(other._lastChange, _lastChange)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,version,tlsReady,noteCount,const DeepCollectionEquality().hash(_latest),const DeepCollectionEquality().hash(_lastChange),error);

@override
String toString() {
  return 'CoreCheckState(status: $status, version: $version, tlsReady: $tlsReady, noteCount: $noteCount, latest: $latest, lastChange: $lastChange, error: $error)';
}


}

/// @nodoc
abstract mixin class _$CoreCheckStateCopyWith<$Res> implements $CoreCheckStateCopyWith<$Res> {
  factory _$CoreCheckStateCopyWith(_CoreCheckState value, $Res Function(_CoreCheckState) _then) = __$CoreCheckStateCopyWithImpl;
@override @useResult
$Res call({
 CoreCheckStatus status, String version, bool tlsReady, int noteCount, List<NoteRow> latest, List<String> lastChange, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$CoreCheckStateCopyWithImpl<$Res>
    implements _$CoreCheckStateCopyWith<$Res> {
  __$CoreCheckStateCopyWithImpl(this._self, this._then);

  final _CoreCheckState _self;
  final $Res Function(_CoreCheckState) _then;

/// Create a copy of CoreCheckState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? version = null,Object? tlsReady = null,Object? noteCount = null,Object? latest = null,Object? lastChange = null,Object? error = freezed,}) {
  return _then(_CoreCheckState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as CoreCheckStatus,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String,tlsReady: null == tlsReady ? _self.tlsReady : tlsReady // ignore: cast_nullable_to_non_nullable
as bool,noteCount: null == noteCount ? _self.noteCount : noteCount // ignore: cast_nullable_to_non_nullable
as int,latest: null == latest ? _self._latest : latest // ignore: cast_nullable_to_non_nullable
as List<NoteRow>,lastChange: null == lastChange ? _self._lastChange : lastChange // ignore: cast_nullable_to_non_nullable
as List<String>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of CoreCheckState
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
}
}

// dart format on
