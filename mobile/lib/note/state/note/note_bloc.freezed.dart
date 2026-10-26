// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteEvent()';
}


}

/// @nodoc
class $NoteEventCopyWith<$Res>  {
$NoteEventCopyWith(NoteEvent _, $Res Function(NoteEvent) __);
}


/// Adds pattern-matching-related methods to [NoteEvent].
extension NoteEventPatterns on NoteEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteStartedEvent value)?  started,TResult Function( NoteChangesReceivedEvent value)?  changesReceived,TResult Function( NoteSyntaxThemeChangedEvent value)?  syntaxThemeChanged,TResult Function( NoteRetriedEvent value)?  retried,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteStartedEvent() when started != null:
return started(_that);case NoteChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case NoteSyntaxThemeChangedEvent() when syntaxThemeChanged != null:
return syntaxThemeChanged(_that);case NoteRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteStartedEvent value)  started,required TResult Function( NoteChangesReceivedEvent value)  changesReceived,required TResult Function( NoteSyntaxThemeChangedEvent value)  syntaxThemeChanged,required TResult Function( NoteRetriedEvent value)  retried,}){
final _that = this;
switch (_that) {
case NoteStartedEvent():
return started(_that);case NoteChangesReceivedEvent():
return changesReceived(_that);case NoteSyntaxThemeChangedEvent():
return syntaxThemeChanged(_that);case NoteRetriedEvent():
return retried(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteStartedEvent value)?  started,TResult? Function( NoteChangesReceivedEvent value)?  changesReceived,TResult? Function( NoteSyntaxThemeChangedEvent value)?  syntaxThemeChanged,TResult? Function( NoteRetriedEvent value)?  retried,}){
final _that = this;
switch (_that) {
case NoteStartedEvent() when started != null:
return started(_that);case NoteChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case NoteSyntaxThemeChangedEvent() when syntaxThemeChanged != null:
return syntaxThemeChanged(_that);case NoteRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  String themeId)?  started,TResult Function( List<String> ids)?  changesReceived,TResult Function( String themeId)?  syntaxThemeChanged,TResult Function()?  retried,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteStartedEvent() when started != null:
return started(_that.id,_that.themeId);case NoteChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case NoteSyntaxThemeChangedEvent() when syntaxThemeChanged != null:
return syntaxThemeChanged(_that.themeId);case NoteRetriedEvent() when retried != null:
return retried();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  String themeId)  started,required TResult Function( List<String> ids)  changesReceived,required TResult Function( String themeId)  syntaxThemeChanged,required TResult Function()  retried,}) {final _that = this;
switch (_that) {
case NoteStartedEvent():
return started(_that.id,_that.themeId);case NoteChangesReceivedEvent():
return changesReceived(_that.ids);case NoteSyntaxThemeChangedEvent():
return syntaxThemeChanged(_that.themeId);case NoteRetriedEvent():
return retried();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  String themeId)?  started,TResult? Function( List<String> ids)?  changesReceived,TResult? Function( String themeId)?  syntaxThemeChanged,TResult? Function()?  retried,}) {final _that = this;
switch (_that) {
case NoteStartedEvent() when started != null:
return started(_that.id,_that.themeId);case NoteChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case NoteSyntaxThemeChangedEvent() when syntaxThemeChanged != null:
return syntaxThemeChanged(_that.themeId);case NoteRetriedEvent() when retried != null:
return retried();case _:
  return null;

}
}

}

/// @nodoc


class NoteStartedEvent implements NoteEvent {
  const NoteStartedEvent(this.id, this.themeId);
  

 final  String id;
 final  String themeId;

/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteStartedEventCopyWith<NoteStartedEvent> get copyWith => _$NoteStartedEventCopyWithImpl<NoteStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteStartedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.themeId, themeId) || other.themeId == themeId));
}


@override
int get hashCode => Object.hash(runtimeType,id,themeId);

@override
String toString() {
  return 'NoteEvent.started(id: $id, themeId: $themeId)';
}


}

/// @nodoc
abstract mixin class $NoteStartedEventCopyWith<$Res> implements $NoteEventCopyWith<$Res> {
  factory $NoteStartedEventCopyWith(NoteStartedEvent value, $Res Function(NoteStartedEvent) _then) = _$NoteStartedEventCopyWithImpl;
@useResult
$Res call({
 String id, String themeId
});




}
/// @nodoc
class _$NoteStartedEventCopyWithImpl<$Res>
    implements $NoteStartedEventCopyWith<$Res> {
  _$NoteStartedEventCopyWithImpl(this._self, this._then);

  final NoteStartedEvent _self;
  final $Res Function(NoteStartedEvent) _then;

/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? themeId = null,}) {
  return _then(NoteStartedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChangesReceivedEvent implements NoteEvent {
  const NoteChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChangesReceivedEventCopyWith<NoteChangesReceivedEvent> get copyWith => _$NoteChangesReceivedEventCopyWithImpl<NoteChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'NoteEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $NoteChangesReceivedEventCopyWith<$Res> implements $NoteEventCopyWith<$Res> {
  factory $NoteChangesReceivedEventCopyWith(NoteChangesReceivedEvent value, $Res Function(NoteChangesReceivedEvent) _then) = _$NoteChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$NoteChangesReceivedEventCopyWithImpl<$Res>
    implements $NoteChangesReceivedEventCopyWith<$Res> {
  _$NoteChangesReceivedEventCopyWithImpl(this._self, this._then);

  final NoteChangesReceivedEvent _self;
  final $Res Function(NoteChangesReceivedEvent) _then;

/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(NoteChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class NoteSyntaxThemeChangedEvent implements NoteEvent {
  const NoteSyntaxThemeChangedEvent(this.themeId);
  

 final  String themeId;

/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteSyntaxThemeChangedEventCopyWith<NoteSyntaxThemeChangedEvent> get copyWith => _$NoteSyntaxThemeChangedEventCopyWithImpl<NoteSyntaxThemeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteSyntaxThemeChangedEvent&&(identical(other.themeId, themeId) || other.themeId == themeId));
}


@override
int get hashCode => Object.hash(runtimeType,themeId);

@override
String toString() {
  return 'NoteEvent.syntaxThemeChanged(themeId: $themeId)';
}


}

/// @nodoc
abstract mixin class $NoteSyntaxThemeChangedEventCopyWith<$Res> implements $NoteEventCopyWith<$Res> {
  factory $NoteSyntaxThemeChangedEventCopyWith(NoteSyntaxThemeChangedEvent value, $Res Function(NoteSyntaxThemeChangedEvent) _then) = _$NoteSyntaxThemeChangedEventCopyWithImpl;
@useResult
$Res call({
 String themeId
});




}
/// @nodoc
class _$NoteSyntaxThemeChangedEventCopyWithImpl<$Res>
    implements $NoteSyntaxThemeChangedEventCopyWith<$Res> {
  _$NoteSyntaxThemeChangedEventCopyWithImpl(this._self, this._then);

  final NoteSyntaxThemeChangedEvent _self;
  final $Res Function(NoteSyntaxThemeChangedEvent) _then;

/// Create a copy of NoteEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? themeId = null,}) {
  return _then(NoteSyntaxThemeChangedEvent(
null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteRetriedEvent implements NoteEvent {
  const NoteRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteEvent.retried()';
}


}




/// @nodoc
mixin _$NoteState {

/// The note being shown.
 String get noteId;/// The built-in theme its text is highlighted in.
 String get themeId; NoteScreenStatus get status;/// The note as read. While it is locked and closed it has its title and
/// its lock and no text, tags or status; null until it is read, and
/// when there is none.
 NoteView? get note;/// The text of the note, cut into lines with their styled runs; empty
/// unless [status] is ready.
 List<SourceLine> get lines;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;
/// Create a copy of NoteState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteStateCopyWith<NoteState> get copyWith => _$NoteStateCopyWithImpl<NoteState>(this as NoteState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themeId, themeId) || other.themeId == themeId)&&(identical(other.status, status) || other.status == status)&&(identical(other.note, note) || other.note == note)&&const DeepCollectionEquality().equals(other.lines, lines)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themeId,status,note,const DeepCollectionEquality().hash(lines),error);

@override
String toString() {
  return 'NoteState(noteId: $noteId, themeId: $themeId, status: $status, note: $note, lines: $lines, error: $error)';
}


}

/// @nodoc
abstract mixin class $NoteStateCopyWith<$Res>  {
  factory $NoteStateCopyWith(NoteState value, $Res Function(NoteState) _then) = _$NoteStateCopyWithImpl;
@useResult
$Res call({
 String noteId, String themeId, NoteScreenStatus status, NoteView? note, List<SourceLine> lines, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$NoteStateCopyWithImpl<$Res>
    implements $NoteStateCopyWith<$Res> {
  _$NoteStateCopyWithImpl(this._self, this._then);

  final NoteState _self;
  final $Res Function(NoteState) _then;

/// Create a copy of NoteState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? themeId = null,Object? status = null,Object? note = freezed,Object? lines = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themeId: null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteScreenStatus,note: freezed == note ? _self.note : note // ignore: cast_nullable_to_non_nullable
as NoteView?,lines: null == lines ? _self.lines : lines // ignore: cast_nullable_to_non_nullable
as List<SourceLine>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of NoteState
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


/// Adds pattern-matching-related methods to [NoteState].
extension NoteStatePatterns on NoteState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteState value)  $default,){
final _that = this;
switch (_that) {
case _NoteState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String themeId,  NoteScreenStatus status,  NoteView? note,  List<SourceLine> lines,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteState() when $default != null:
return $default(_that.noteId,_that.themeId,_that.status,_that.note,_that.lines,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String themeId,  NoteScreenStatus status,  NoteView? note,  List<SourceLine> lines,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _NoteState():
return $default(_that.noteId,_that.themeId,_that.status,_that.note,_that.lines,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String themeId,  NoteScreenStatus status,  NoteView? note,  List<SourceLine> lines,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _NoteState() when $default != null:
return $default(_that.noteId,_that.themeId,_that.status,_that.note,_that.lines,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _NoteState implements NoteState {
  const _NoteState({this.noteId = '', this.themeId = '', this.status = NoteScreenStatus.loading, this.note, final  List<SourceLine> lines = const <SourceLine>[], this.error}): _lines = lines;
  

/// The note being shown.
@override@JsonKey() final  String noteId;
/// The built-in theme its text is highlighted in.
@override@JsonKey() final  String themeId;
@override@JsonKey() final  NoteScreenStatus status;
/// The note as read. While it is locked and closed it has its title and
/// its lock and no text, tags or status; null until it is read, and
/// when there is none.
@override final  NoteView? note;
/// The text of the note, cut into lines with their styled runs; empty
/// unless [status] is ready.
 final  List<SourceLine> _lines;
/// The text of the note, cut into lines with their styled runs; empty
/// unless [status] is ready.
@override@JsonKey() List<SourceLine> get lines {
  if (_lines is EqualUnmodifiableListView) return _lines;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_lines);
}

/// Why reading failed, while [status] is failed.
@override final  ApplicationError? error;

/// Create a copy of NoteState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteStateCopyWith<_NoteState> get copyWith => __$NoteStateCopyWithImpl<_NoteState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themeId, themeId) || other.themeId == themeId)&&(identical(other.status, status) || other.status == status)&&(identical(other.note, note) || other.note == note)&&const DeepCollectionEquality().equals(other._lines, _lines)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themeId,status,note,const DeepCollectionEquality().hash(_lines),error);

@override
String toString() {
  return 'NoteState(noteId: $noteId, themeId: $themeId, status: $status, note: $note, lines: $lines, error: $error)';
}


}

/// @nodoc
abstract mixin class _$NoteStateCopyWith<$Res> implements $NoteStateCopyWith<$Res> {
  factory _$NoteStateCopyWith(_NoteState value, $Res Function(_NoteState) _then) = __$NoteStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String themeId, NoteScreenStatus status, NoteView? note, List<SourceLine> lines, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$NoteStateCopyWithImpl<$Res>
    implements _$NoteStateCopyWith<$Res> {
  __$NoteStateCopyWithImpl(this._self, this._then);

  final _NoteState _self;
  final $Res Function(_NoteState) _then;

/// Create a copy of NoteState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? themeId = null,Object? status = null,Object? note = freezed,Object? lines = null,Object? error = freezed,}) {
  return _then(_NoteState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themeId: null == themeId ? _self.themeId : themeId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteScreenStatus,note: freezed == note ? _self.note : note // ignore: cast_nullable_to_non_nullable
as NoteView?,lines: null == lines ? _self._lines : lines // ignore: cast_nullable_to_non_nullable
as List<SourceLine>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of NoteState
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
