// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'backup_notice_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BackupNoticeEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupNoticeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupNoticeEvent()';
}


}

/// @nodoc
class $BackupNoticeEventCopyWith<$Res>  {
$BackupNoticeEventCopyWith(BackupNoticeEvent _, $Res Function(BackupNoticeEvent) __);
}


/// Adds pattern-matching-related methods to [BackupNoticeEvent].
extension BackupNoticeEventPatterns on BackupNoticeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BackupNoticeStartedEvent value)?  started,TResult Function( BackupNoticeDismissedEvent value)?  dismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BackupNoticeStartedEvent() when started != null:
return started(_that);case BackupNoticeDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BackupNoticeStartedEvent value)  started,required TResult Function( BackupNoticeDismissedEvent value)  dismissed,}){
final _that = this;
switch (_that) {
case BackupNoticeStartedEvent():
return started(_that);case BackupNoticeDismissedEvent():
return dismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BackupNoticeStartedEvent value)?  started,TResult? Function( BackupNoticeDismissedEvent value)?  dismissed,}){
final _that = this;
switch (_that) {
case BackupNoticeStartedEvent() when started != null:
return started(_that);case BackupNoticeDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  dismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BackupNoticeStartedEvent() when started != null:
return started();case BackupNoticeDismissedEvent() when dismissed != null:
return dismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  dismissed,}) {final _that = this;
switch (_that) {
case BackupNoticeStartedEvent():
return started();case BackupNoticeDismissedEvent():
return dismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  dismissed,}) {final _that = this;
switch (_that) {
case BackupNoticeStartedEvent() when started != null:
return started();case BackupNoticeDismissedEvent() when dismissed != null:
return dismissed();case _:
  return null;

}
}

}

/// @nodoc


class BackupNoticeStartedEvent implements BackupNoticeEvent {
  const BackupNoticeStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupNoticeStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupNoticeEvent.started()';
}


}




/// @nodoc


class BackupNoticeDismissedEvent implements BackupNoticeEvent {
  const BackupNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupNoticeEvent.dismissed()';
}


}




/// @nodoc
mixin _$BackupNoticeState {

/// Whether the notice is on screen.
 bool get visible;
/// Create a copy of BackupNoticeState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BackupNoticeStateCopyWith<BackupNoticeState> get copyWith => _$BackupNoticeStateCopyWithImpl<BackupNoticeState>(this as BackupNoticeState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupNoticeState&&(identical(other.visible, visible) || other.visible == visible));
}


@override
int get hashCode => Object.hash(runtimeType,visible);

@override
String toString() {
  return 'BackupNoticeState(visible: $visible)';
}


}

/// @nodoc
abstract mixin class $BackupNoticeStateCopyWith<$Res>  {
  factory $BackupNoticeStateCopyWith(BackupNoticeState value, $Res Function(BackupNoticeState) _then) = _$BackupNoticeStateCopyWithImpl;
@useResult
$Res call({
 bool visible
});




}
/// @nodoc
class _$BackupNoticeStateCopyWithImpl<$Res>
    implements $BackupNoticeStateCopyWith<$Res> {
  _$BackupNoticeStateCopyWithImpl(this._self, this._then);

  final BackupNoticeState _self;
  final $Res Function(BackupNoticeState) _then;

/// Create a copy of BackupNoticeState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? visible = null,}) {
  return _then(_self.copyWith(
visible: null == visible ? _self.visible : visible // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [BackupNoticeState].
extension BackupNoticeStatePatterns on BackupNoticeState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _BackupNoticeState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _BackupNoticeState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _BackupNoticeState value)  $default,){
final _that = this;
switch (_that) {
case _BackupNoticeState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _BackupNoticeState value)?  $default,){
final _that = this;
switch (_that) {
case _BackupNoticeState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool visible)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BackupNoticeState() when $default != null:
return $default(_that.visible);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool visible)  $default,) {final _that = this;
switch (_that) {
case _BackupNoticeState():
return $default(_that.visible);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool visible)?  $default,) {final _that = this;
switch (_that) {
case _BackupNoticeState() when $default != null:
return $default(_that.visible);case _:
  return null;

}
}

}

/// @nodoc


class _BackupNoticeState implements BackupNoticeState {
  const _BackupNoticeState({this.visible = false});
  

/// Whether the notice is on screen.
@override@JsonKey() final  bool visible;

/// Create a copy of BackupNoticeState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BackupNoticeStateCopyWith<_BackupNoticeState> get copyWith => __$BackupNoticeStateCopyWithImpl<_BackupNoticeState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BackupNoticeState&&(identical(other.visible, visible) || other.visible == visible));
}


@override
int get hashCode => Object.hash(runtimeType,visible);

@override
String toString() {
  return 'BackupNoticeState(visible: $visible)';
}


}

/// @nodoc
abstract mixin class _$BackupNoticeStateCopyWith<$Res> implements $BackupNoticeStateCopyWith<$Res> {
  factory _$BackupNoticeStateCopyWith(_BackupNoticeState value, $Res Function(_BackupNoticeState) _then) = __$BackupNoticeStateCopyWithImpl;
@override @useResult
$Res call({
 bool visible
});




}
/// @nodoc
class __$BackupNoticeStateCopyWithImpl<$Res>
    implements _$BackupNoticeStateCopyWith<$Res> {
  __$BackupNoticeStateCopyWithImpl(this._self, this._then);

  final _BackupNoticeState _self;
  final $Res Function(_BackupNoticeState) _then;

/// Create a copy of BackupNoticeState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? visible = null,}) {
  return _then(_BackupNoticeState(
visible: null == visible ? _self.visible : visible // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
