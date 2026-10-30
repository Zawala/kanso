// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'preview_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PreviewEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent()';
}


}

/// @nodoc
class $PreviewEventCopyWith<$Res>  {
$PreviewEventCopyWith(PreviewEvent _, $Res Function(PreviewEvent) __);
}


/// Adds pattern-matching-related methods to [PreviewEvent].
extension PreviewEventPatterns on PreviewEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PreviewStartedEvent value)?  started,TResult Function( PreviewThemeChangedEvent value)?  themeChanged,TResult Function( PreviewStyleChangedEvent value)?  styleChanged,TResult Function( PreviewChangesReceivedEvent value)?  changesReceived,TResult Function( PreviewLinkTappedEvent value)?  linkTapped,TResult Function( PreviewActionHandledEvent value)?  actionHandled,TResult Function( PreviewNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case PreviewStyleChangedEvent() when styleChanged != null:
return styleChanged(_that);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled(_that);case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PreviewStartedEvent value)  started,required TResult Function( PreviewThemeChangedEvent value)  themeChanged,required TResult Function( PreviewStyleChangedEvent value)  styleChanged,required TResult Function( PreviewChangesReceivedEvent value)  changesReceived,required TResult Function( PreviewLinkTappedEvent value)  linkTapped,required TResult Function( PreviewActionHandledEvent value)  actionHandled,required TResult Function( PreviewNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case PreviewStartedEvent():
return started(_that);case PreviewThemeChangedEvent():
return themeChanged(_that);case PreviewStyleChangedEvent():
return styleChanged(_that);case PreviewChangesReceivedEvent():
return changesReceived(_that);case PreviewLinkTappedEvent():
return linkTapped(_that);case PreviewActionHandledEvent():
return actionHandled(_that);case PreviewNoticeDismissedEvent():
return noticeDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PreviewStartedEvent value)?  started,TResult? Function( PreviewThemeChangedEvent value)?  themeChanged,TResult? Function( PreviewStyleChangedEvent value)?  styleChanged,TResult? Function( PreviewChangesReceivedEvent value)?  changesReceived,TResult? Function( PreviewLinkTappedEvent value)?  linkTapped,TResult? Function( PreviewActionHandledEvent value)?  actionHandled,TResult? Function( PreviewNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case PreviewStyleChangedEvent() when styleChanged != null:
return styleChanged(_that);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled(_that);case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId,  PreviewThemes themes,  double size)?  started,TResult Function( PreviewThemes themes)?  themeChanged,TResult Function( double size)?  styleChanged,TResult Function()?  changesReceived,TResult Function( String url)?  linkTapped,TResult Function()?  actionHandled,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that.noteId,_that.themes,_that.size);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.themes);case PreviewStyleChangedEvent() when styleChanged != null:
return styleChanged(_that.size);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived();case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that.url);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled();case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId,  PreviewThemes themes,  double size)  started,required TResult Function( PreviewThemes themes)  themeChanged,required TResult Function( double size)  styleChanged,required TResult Function()  changesReceived,required TResult Function( String url)  linkTapped,required TResult Function()  actionHandled,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case PreviewStartedEvent():
return started(_that.noteId,_that.themes,_that.size);case PreviewThemeChangedEvent():
return themeChanged(_that.themes);case PreviewStyleChangedEvent():
return styleChanged(_that.size);case PreviewChangesReceivedEvent():
return changesReceived();case PreviewLinkTappedEvent():
return linkTapped(_that.url);case PreviewActionHandledEvent():
return actionHandled();case PreviewNoticeDismissedEvent():
return noticeDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId,  PreviewThemes themes,  double size)?  started,TResult? Function( PreviewThemes themes)?  themeChanged,TResult? Function( double size)?  styleChanged,TResult? Function()?  changesReceived,TResult? Function( String url)?  linkTapped,TResult? Function()?  actionHandled,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case PreviewStartedEvent() when started != null:
return started(_that.noteId,_that.themes,_that.size);case PreviewThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.themes);case PreviewStyleChangedEvent() when styleChanged != null:
return styleChanged(_that.size);case PreviewChangesReceivedEvent() when changesReceived != null:
return changesReceived();case PreviewLinkTappedEvent() when linkTapped != null:
return linkTapped(_that.url);case PreviewActionHandledEvent() when actionHandled != null:
return actionHandled();case PreviewNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class PreviewStartedEvent implements PreviewEvent {
  const PreviewStartedEvent(this.noteId, this.themes, {this.size = previewBaseTextSize});
  

 final  String noteId;
 final  PreviewThemes themes;
@JsonKey() final  double size;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewStartedEventCopyWith<PreviewStartedEvent> get copyWith => _$PreviewStartedEventCopyWithImpl<PreviewStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themes, themes) || other.themes == themes)&&(identical(other.size, size) || other.size == size));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themes,size);

@override
String toString() {
  return 'PreviewEvent.started(noteId: $noteId, themes: $themes, size: $size)';
}


}

/// @nodoc
abstract mixin class $PreviewStartedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewStartedEventCopyWith(PreviewStartedEvent value, $Res Function(PreviewStartedEvent) _then) = _$PreviewStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId, PreviewThemes themes, double size
});




}
/// @nodoc
class _$PreviewStartedEventCopyWithImpl<$Res>
    implements $PreviewStartedEventCopyWith<$Res> {
  _$PreviewStartedEventCopyWithImpl(this._self, this._then);

  final PreviewStartedEvent _self;
  final $Res Function(PreviewStartedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? themes = null,Object? size = null,}) {
  return _then(PreviewStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as PreviewThemes,size: null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PreviewThemeChangedEvent implements PreviewEvent {
  const PreviewThemeChangedEvent(this.themes);
  

 final  PreviewThemes themes;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewThemeChangedEventCopyWith<PreviewThemeChangedEvent> get copyWith => _$PreviewThemeChangedEventCopyWithImpl<PreviewThemeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewThemeChangedEvent&&(identical(other.themes, themes) || other.themes == themes));
}


@override
int get hashCode => Object.hash(runtimeType,themes);

@override
String toString() {
  return 'PreviewEvent.themeChanged(themes: $themes)';
}


}

/// @nodoc
abstract mixin class $PreviewThemeChangedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewThemeChangedEventCopyWith(PreviewThemeChangedEvent value, $Res Function(PreviewThemeChangedEvent) _then) = _$PreviewThemeChangedEventCopyWithImpl;
@useResult
$Res call({
 PreviewThemes themes
});




}
/// @nodoc
class _$PreviewThemeChangedEventCopyWithImpl<$Res>
    implements $PreviewThemeChangedEventCopyWith<$Res> {
  _$PreviewThemeChangedEventCopyWithImpl(this._self, this._then);

  final PreviewThemeChangedEvent _self;
  final $Res Function(PreviewThemeChangedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? themes = null,}) {
  return _then(PreviewThemeChangedEvent(
null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as PreviewThemes,
  ));
}


}

/// @nodoc


class PreviewStyleChangedEvent implements PreviewEvent {
  const PreviewStyleChangedEvent({required this.size});
  

 final  double size;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewStyleChangedEventCopyWith<PreviewStyleChangedEvent> get copyWith => _$PreviewStyleChangedEventCopyWithImpl<PreviewStyleChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewStyleChangedEvent&&(identical(other.size, size) || other.size == size));
}


@override
int get hashCode => Object.hash(runtimeType,size);

@override
String toString() {
  return 'PreviewEvent.styleChanged(size: $size)';
}


}

/// @nodoc
abstract mixin class $PreviewStyleChangedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewStyleChangedEventCopyWith(PreviewStyleChangedEvent value, $Res Function(PreviewStyleChangedEvent) _then) = _$PreviewStyleChangedEventCopyWithImpl;
@useResult
$Res call({
 double size
});




}
/// @nodoc
class _$PreviewStyleChangedEventCopyWithImpl<$Res>
    implements $PreviewStyleChangedEventCopyWith<$Res> {
  _$PreviewStyleChangedEventCopyWithImpl(this._self, this._then);

  final PreviewStyleChangedEvent _self;
  final $Res Function(PreviewStyleChangedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? size = null,}) {
  return _then(PreviewStyleChangedEvent(
size: null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PreviewChangesReceivedEvent implements PreviewEvent {
  const PreviewChangesReceivedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewChangesReceivedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.changesReceived()';
}


}




/// @nodoc


class PreviewLinkTappedEvent implements PreviewEvent {
  const PreviewLinkTappedEvent(this.url);
  

 final  String url;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewLinkTappedEventCopyWith<PreviewLinkTappedEvent> get copyWith => _$PreviewLinkTappedEventCopyWithImpl<PreviewLinkTappedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewLinkTappedEvent&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,url);

@override
String toString() {
  return 'PreviewEvent.linkTapped(url: $url)';
}


}

/// @nodoc
abstract mixin class $PreviewLinkTappedEventCopyWith<$Res> implements $PreviewEventCopyWith<$Res> {
  factory $PreviewLinkTappedEventCopyWith(PreviewLinkTappedEvent value, $Res Function(PreviewLinkTappedEvent) _then) = _$PreviewLinkTappedEventCopyWithImpl;
@useResult
$Res call({
 String url
});




}
/// @nodoc
class _$PreviewLinkTappedEventCopyWithImpl<$Res>
    implements $PreviewLinkTappedEventCopyWith<$Res> {
  _$PreviewLinkTappedEventCopyWithImpl(this._self, this._then);

  final PreviewLinkTappedEvent _self;
  final $Res Function(PreviewLinkTappedEvent) _then;

/// Create a copy of PreviewEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,}) {
  return _then(PreviewLinkTappedEvent(
null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewActionHandledEvent implements PreviewEvent {
  const PreviewActionHandledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewActionHandledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.actionHandled()';
}


}




/// @nodoc


class PreviewNoticeDismissedEvent implements PreviewEvent {
  const PreviewNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$PreviewState {

/// The note drawn.
 String get noteId;/// The themes it is drawn in.
 PreviewThemes get themes;/// The note's body size in logical pixels.
 double get size;/// The note's blocks, once they are drawn; the last ones stay while they
/// are drawn again.
 List<PreviewBlock>? get blocks;/// The revision of the note the blocks were drawn from: a tick is made
/// against it.
 String get rev;/// Why the note could not be drawn, when it couldn't.
 ApplicationError? get error;/// What the screen is to do for the last tap, until it says it has.
 PreviewAction? get action;/// Why the last tap did nothing, until it is dismissed.
 ApplicationError? get notice;
/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewStateCopyWith<PreviewState> get copyWith => _$PreviewStateCopyWithImpl<PreviewState>(this as PreviewState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themes, themes) || other.themes == themes)&&(identical(other.size, size) || other.size == size)&&const DeepCollectionEquality().equals(other.blocks, blocks)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.error, error) || other.error == error)&&(identical(other.action, action) || other.action == action)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themes,size,const DeepCollectionEquality().hash(blocks),rev,error,action,notice);

@override
String toString() {
  return 'PreviewState(noteId: $noteId, themes: $themes, size: $size, blocks: $blocks, rev: $rev, error: $error, action: $action, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $PreviewStateCopyWith<$Res>  {
  factory $PreviewStateCopyWith(PreviewState value, $Res Function(PreviewState) _then) = _$PreviewStateCopyWithImpl;
@useResult
$Res call({
 String noteId, PreviewThemes themes, double size, List<PreviewBlock>? blocks, String rev, ApplicationError? error, PreviewAction? action, ApplicationError? notice
});


$ApplicationErrorCopyWith<$Res>? get error;$PreviewActionCopyWith<$Res>? get action;$ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class _$PreviewStateCopyWithImpl<$Res>
    implements $PreviewStateCopyWith<$Res> {
  _$PreviewStateCopyWithImpl(this._self, this._then);

  final PreviewState _self;
  final $Res Function(PreviewState) _then;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? themes = null,Object? size = null,Object? blocks = freezed,Object? rev = null,Object? error = freezed,Object? action = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themes: null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as PreviewThemes,size: null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as double,blocks: freezed == blocks ? _self.blocks : blocks // ignore: cast_nullable_to_non_nullable
as List<PreviewBlock>?,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as PreviewAction?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of PreviewState
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
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PreviewActionCopyWith<$Res>? get action {
    if (_self.action == null) {
    return null;
  }

  return $PreviewActionCopyWith<$Res>(_self.action!, (value) {
    return _then(_self.copyWith(action: value));
  });
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}


/// Adds pattern-matching-related methods to [PreviewState].
extension PreviewStatePatterns on PreviewState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PreviewState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PreviewState value)  $default,){
final _that = this;
switch (_that) {
case _PreviewState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PreviewState value)?  $default,){
final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  PreviewThemes themes,  double size,  List<PreviewBlock>? blocks,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
return $default(_that.noteId,_that.themes,_that.size,_that.blocks,_that.rev,_that.error,_that.action,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  PreviewThemes themes,  double size,  List<PreviewBlock>? blocks,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)  $default,) {final _that = this;
switch (_that) {
case _PreviewState():
return $default(_that.noteId,_that.themes,_that.size,_that.blocks,_that.rev,_that.error,_that.action,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  PreviewThemes themes,  double size,  List<PreviewBlock>? blocks,  String rev,  ApplicationError? error,  PreviewAction? action,  ApplicationError? notice)?  $default,) {final _that = this;
switch (_that) {
case _PreviewState() when $default != null:
return $default(_that.noteId,_that.themes,_that.size,_that.blocks,_that.rev,_that.error,_that.action,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _PreviewState implements PreviewState {
  const _PreviewState({this.noteId = '', this.themes = const (ui: 'kanso-light', syntax: 'kanso-light', preview: 'kanso-light', dark: false), this.size = previewBaseTextSize, final  List<PreviewBlock>? blocks, this.rev = '', this.error, this.action, this.notice}): _blocks = blocks;
  

/// The note drawn.
@override@JsonKey() final  String noteId;
/// The themes it is drawn in.
@override@JsonKey() final  PreviewThemes themes;
/// The note's body size in logical pixels.
@override@JsonKey() final  double size;
/// The note's blocks, once they are drawn; the last ones stay while they
/// are drawn again.
 final  List<PreviewBlock>? _blocks;
/// The note's blocks, once they are drawn; the last ones stay while they
/// are drawn again.
@override List<PreviewBlock>? get blocks {
  final value = _blocks;
  if (value == null) return null;
  if (_blocks is EqualUnmodifiableListView) return _blocks;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(value);
}

/// The revision of the note the blocks were drawn from: a tick is made
/// against it.
@override@JsonKey() final  String rev;
/// Why the note could not be drawn, when it couldn't.
@override final  ApplicationError? error;
/// What the screen is to do for the last tap, until it says it has.
@override final  PreviewAction? action;
/// Why the last tap did nothing, until it is dismissed.
@override final  ApplicationError? notice;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PreviewStateCopyWith<_PreviewState> get copyWith => __$PreviewStateCopyWithImpl<_PreviewState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PreviewState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.themes, themes) || other.themes == themes)&&(identical(other.size, size) || other.size == size)&&const DeepCollectionEquality().equals(other._blocks, _blocks)&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.error, error) || other.error == error)&&(identical(other.action, action) || other.action == action)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,themes,size,const DeepCollectionEquality().hash(_blocks),rev,error,action,notice);

@override
String toString() {
  return 'PreviewState(noteId: $noteId, themes: $themes, size: $size, blocks: $blocks, rev: $rev, error: $error, action: $action, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$PreviewStateCopyWith<$Res> implements $PreviewStateCopyWith<$Res> {
  factory _$PreviewStateCopyWith(_PreviewState value, $Res Function(_PreviewState) _then) = __$PreviewStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, PreviewThemes themes, double size, List<PreviewBlock>? blocks, String rev, ApplicationError? error, PreviewAction? action, ApplicationError? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;@override $PreviewActionCopyWith<$Res>? get action;@override $ApplicationErrorCopyWith<$Res>? get notice;

}
/// @nodoc
class __$PreviewStateCopyWithImpl<$Res>
    implements _$PreviewStateCopyWith<$Res> {
  __$PreviewStateCopyWithImpl(this._self, this._then);

  final _PreviewState _self;
  final $Res Function(_PreviewState) _then;

/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? themes = null,Object? size = null,Object? blocks = freezed,Object? rev = null,Object? error = freezed,Object? action = freezed,Object? notice = freezed,}) {
  return _then(_PreviewState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,themes: null == themes ? _self.themes : themes // ignore: cast_nullable_to_non_nullable
as PreviewThemes,size: null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as double,blocks: freezed == blocks ? _self._blocks : blocks // ignore: cast_nullable_to_non_nullable
as List<PreviewBlock>?,rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as PreviewAction?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of PreviewState
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
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PreviewActionCopyWith<$Res>? get action {
    if (_self.action == null) {
    return null;
  }

  return $PreviewActionCopyWith<$Res>(_self.action!, (value) {
    return _then(_self.copyWith(action: value));
  });
}/// Create a copy of PreviewState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}
}

// dart format on
