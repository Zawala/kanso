// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'focus_mode_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FocusModeEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FocusModeEvent()';
}


}

/// @nodoc
class $FocusModeEventCopyWith<$Res>  {
$FocusModeEventCopyWith(FocusModeEvent _, $Res Function(FocusModeEvent) __);
}


/// Adds pattern-matching-related methods to [FocusModeEvent].
extension FocusModeEventPatterns on FocusModeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FocusModeStartedEvent value)?  started,TResult Function( FocusModeToggledEvent value)?  toggled,TResult Function( FocusModeExitedEvent value)?  exited,TResult Function( FocusModeLivePreviewChangedEvent value)?  livePreviewChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FocusModeStartedEvent() when started != null:
return started(_that);case FocusModeToggledEvent() when toggled != null:
return toggled(_that);case FocusModeExitedEvent() when exited != null:
return exited(_that);case FocusModeLivePreviewChangedEvent() when livePreviewChanged != null:
return livePreviewChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FocusModeStartedEvent value)  started,required TResult Function( FocusModeToggledEvent value)  toggled,required TResult Function( FocusModeExitedEvent value)  exited,required TResult Function( FocusModeLivePreviewChangedEvent value)  livePreviewChanged,}){
final _that = this;
switch (_that) {
case FocusModeStartedEvent():
return started(_that);case FocusModeToggledEvent():
return toggled(_that);case FocusModeExitedEvent():
return exited(_that);case FocusModeLivePreviewChangedEvent():
return livePreviewChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FocusModeStartedEvent value)?  started,TResult? Function( FocusModeToggledEvent value)?  toggled,TResult? Function( FocusModeExitedEvent value)?  exited,TResult? Function( FocusModeLivePreviewChangedEvent value)?  livePreviewChanged,}){
final _that = this;
switch (_that) {
case FocusModeStartedEvent() when started != null:
return started(_that);case FocusModeToggledEvent() when toggled != null:
return toggled(_that);case FocusModeExitedEvent() when exited != null:
return exited(_that);case FocusModeLivePreviewChangedEvent() when livePreviewChanged != null:
return livePreviewChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  toggled,TResult Function()?  exited,TResult Function( bool enabled)?  livePreviewChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FocusModeStartedEvent() when started != null:
return started();case FocusModeToggledEvent() when toggled != null:
return toggled();case FocusModeExitedEvent() when exited != null:
return exited();case FocusModeLivePreviewChangedEvent() when livePreviewChanged != null:
return livePreviewChanged(_that.enabled);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  toggled,required TResult Function()  exited,required TResult Function( bool enabled)  livePreviewChanged,}) {final _that = this;
switch (_that) {
case FocusModeStartedEvent():
return started();case FocusModeToggledEvent():
return toggled();case FocusModeExitedEvent():
return exited();case FocusModeLivePreviewChangedEvent():
return livePreviewChanged(_that.enabled);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  toggled,TResult? Function()?  exited,TResult? Function( bool enabled)?  livePreviewChanged,}) {final _that = this;
switch (_that) {
case FocusModeStartedEvent() when started != null:
return started();case FocusModeToggledEvent() when toggled != null:
return toggled();case FocusModeExitedEvent() when exited != null:
return exited();case FocusModeLivePreviewChangedEvent() when livePreviewChanged != null:
return livePreviewChanged(_that.enabled);case _:
  return null;

}
}

}

/// @nodoc


class FocusModeStartedEvent implements FocusModeEvent {
  const FocusModeStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FocusModeEvent.started()';
}


}




/// @nodoc


class FocusModeToggledEvent implements FocusModeEvent {
  const FocusModeToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FocusModeEvent.toggled()';
}


}




/// @nodoc


class FocusModeExitedEvent implements FocusModeEvent {
  const FocusModeExitedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeExitedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FocusModeEvent.exited()';
}


}




/// @nodoc


class FocusModeLivePreviewChangedEvent implements FocusModeEvent {
  const FocusModeLivePreviewChangedEvent({required this.enabled});
  

 final  bool enabled;

/// Create a copy of FocusModeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FocusModeLivePreviewChangedEventCopyWith<FocusModeLivePreviewChangedEvent> get copyWith => _$FocusModeLivePreviewChangedEventCopyWithImpl<FocusModeLivePreviewChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeLivePreviewChangedEvent&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,enabled);

@override
String toString() {
  return 'FocusModeEvent.livePreviewChanged(enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $FocusModeLivePreviewChangedEventCopyWith<$Res> implements $FocusModeEventCopyWith<$Res> {
  factory $FocusModeLivePreviewChangedEventCopyWith(FocusModeLivePreviewChangedEvent value, $Res Function(FocusModeLivePreviewChangedEvent) _then) = _$FocusModeLivePreviewChangedEventCopyWithImpl;
@useResult
$Res call({
 bool enabled
});




}
/// @nodoc
class _$FocusModeLivePreviewChangedEventCopyWithImpl<$Res>
    implements $FocusModeLivePreviewChangedEventCopyWith<$Res> {
  _$FocusModeLivePreviewChangedEventCopyWithImpl(this._self, this._then);

  final FocusModeLivePreviewChangedEvent _self;
  final $Res Function(FocusModeLivePreviewChangedEvent) _then;

/// Create a copy of FocusModeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? enabled = null,}) {
  return _then(FocusModeLivePreviewChangedEvent(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$FocusModeState {

/// Whether the open note is shown by itself.
 bool get active;/// Whether the saved choice has been read. A choice made before it was
/// read is not overwritten by the file.
 bool get ready;/// Whether entering focus mode shows the open note in Live, and leaving
/// it goes back to the mode the note was in.
 bool get livePreview;
/// Create a copy of FocusModeState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FocusModeStateCopyWith<FocusModeState> get copyWith => _$FocusModeStateCopyWithImpl<FocusModeState>(this as FocusModeState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FocusModeState&&(identical(other.active, active) || other.active == active)&&(identical(other.ready, ready) || other.ready == ready)&&(identical(other.livePreview, livePreview) || other.livePreview == livePreview));
}


@override
int get hashCode => Object.hash(runtimeType,active,ready,livePreview);

@override
String toString() {
  return 'FocusModeState(active: $active, ready: $ready, livePreview: $livePreview)';
}


}

/// @nodoc
abstract mixin class $FocusModeStateCopyWith<$Res>  {
  factory $FocusModeStateCopyWith(FocusModeState value, $Res Function(FocusModeState) _then) = _$FocusModeStateCopyWithImpl;
@useResult
$Res call({
 bool active, bool ready, bool livePreview
});




}
/// @nodoc
class _$FocusModeStateCopyWithImpl<$Res>
    implements $FocusModeStateCopyWith<$Res> {
  _$FocusModeStateCopyWithImpl(this._self, this._then);

  final FocusModeState _self;
  final $Res Function(FocusModeState) _then;

/// Create a copy of FocusModeState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? active = null,Object? ready = null,Object? livePreview = null,}) {
  return _then(_self.copyWith(
active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,ready: null == ready ? _self.ready : ready // ignore: cast_nullable_to_non_nullable
as bool,livePreview: null == livePreview ? _self.livePreview : livePreview // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [FocusModeState].
extension FocusModeStatePatterns on FocusModeState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FocusModeState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FocusModeState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FocusModeState value)  $default,){
final _that = this;
switch (_that) {
case _FocusModeState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FocusModeState value)?  $default,){
final _that = this;
switch (_that) {
case _FocusModeState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool active,  bool ready,  bool livePreview)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FocusModeState() when $default != null:
return $default(_that.active,_that.ready,_that.livePreview);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool active,  bool ready,  bool livePreview)  $default,) {final _that = this;
switch (_that) {
case _FocusModeState():
return $default(_that.active,_that.ready,_that.livePreview);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool active,  bool ready,  bool livePreview)?  $default,) {final _that = this;
switch (_that) {
case _FocusModeState() when $default != null:
return $default(_that.active,_that.ready,_that.livePreview);case _:
  return null;

}
}

}

/// @nodoc


class _FocusModeState implements FocusModeState {
  const _FocusModeState({this.active = false, this.ready = false, this.livePreview = false});
  

/// Whether the open note is shown by itself.
@override@JsonKey() final  bool active;
/// Whether the saved choice has been read. A choice made before it was
/// read is not overwritten by the file.
@override@JsonKey() final  bool ready;
/// Whether entering focus mode shows the open note in Live, and leaving
/// it goes back to the mode the note was in.
@override@JsonKey() final  bool livePreview;

/// Create a copy of FocusModeState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FocusModeStateCopyWith<_FocusModeState> get copyWith => __$FocusModeStateCopyWithImpl<_FocusModeState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FocusModeState&&(identical(other.active, active) || other.active == active)&&(identical(other.ready, ready) || other.ready == ready)&&(identical(other.livePreview, livePreview) || other.livePreview == livePreview));
}


@override
int get hashCode => Object.hash(runtimeType,active,ready,livePreview);

@override
String toString() {
  return 'FocusModeState(active: $active, ready: $ready, livePreview: $livePreview)';
}


}

/// @nodoc
abstract mixin class _$FocusModeStateCopyWith<$Res> implements $FocusModeStateCopyWith<$Res> {
  factory _$FocusModeStateCopyWith(_FocusModeState value, $Res Function(_FocusModeState) _then) = __$FocusModeStateCopyWithImpl;
@override @useResult
$Res call({
 bool active, bool ready, bool livePreview
});




}
/// @nodoc
class __$FocusModeStateCopyWithImpl<$Res>
    implements _$FocusModeStateCopyWith<$Res> {
  __$FocusModeStateCopyWithImpl(this._self, this._then);

  final _FocusModeState _self;
  final $Res Function(_FocusModeState) _then;

/// Create a copy of FocusModeState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? active = null,Object? ready = null,Object? livePreview = null,}) {
  return _then(_FocusModeState(
active: null == active ? _self.active : active // ignore: cast_nullable_to_non_nullable
as bool,ready: null == ready ? _self.ready : ready // ignore: cast_nullable_to_non_nullable
as bool,livePreview: null == livePreview ? _self.livePreview : livePreview // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
