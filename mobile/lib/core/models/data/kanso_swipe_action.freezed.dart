// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'kanso_swipe_action.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KansoSwipeAction {

 KansoIcons get icon;/// What it says, under the picture and to a screen reader.
 String get label;/// The colour behind it.
 Color get background;/// What a tap on it does.
 VoidCallback get onPressed;
/// Create a copy of KansoSwipeAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoSwipeActionCopyWith<KansoSwipeAction> get copyWith => _$KansoSwipeActionCopyWithImpl<KansoSwipeAction>(this as KansoSwipeAction, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoSwipeAction&&(identical(other.icon, icon) || other.icon == icon)&&(identical(other.label, label) || other.label == label)&&(identical(other.background, background) || other.background == background)&&(identical(other.onPressed, onPressed) || other.onPressed == onPressed));
}


@override
int get hashCode => Object.hash(runtimeType,icon,label,background,onPressed);

@override
String toString() {
  return 'KansoSwipeAction(icon: $icon, label: $label, background: $background, onPressed: $onPressed)';
}


}

/// @nodoc
abstract mixin class $KansoSwipeActionCopyWith<$Res>  {
  factory $KansoSwipeActionCopyWith(KansoSwipeAction value, $Res Function(KansoSwipeAction) _then) = _$KansoSwipeActionCopyWithImpl;
@useResult
$Res call({
 KansoIcons icon, String label, Color background, VoidCallback onPressed
});




}
/// @nodoc
class _$KansoSwipeActionCopyWithImpl<$Res>
    implements $KansoSwipeActionCopyWith<$Res> {
  _$KansoSwipeActionCopyWithImpl(this._self, this._then);

  final KansoSwipeAction _self;
  final $Res Function(KansoSwipeAction) _then;

/// Create a copy of KansoSwipeAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? icon = null,Object? label = null,Object? background = null,Object? onPressed = null,}) {
  return _then(_self.copyWith(
icon: null == icon ? _self.icon : icon // ignore: cast_nullable_to_non_nullable
as KansoIcons,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,background: null == background ? _self.background : background // ignore: cast_nullable_to_non_nullable
as Color,onPressed: null == onPressed ? _self.onPressed : onPressed // ignore: cast_nullable_to_non_nullable
as VoidCallback,
  ));
}

}


/// Adds pattern-matching-related methods to [KansoSwipeAction].
extension KansoSwipeActionPatterns on KansoSwipeAction {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KansoSwipeAction value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KansoSwipeAction() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KansoSwipeAction value)  $default,){
final _that = this;
switch (_that) {
case _KansoSwipeAction():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KansoSwipeAction value)?  $default,){
final _that = this;
switch (_that) {
case _KansoSwipeAction() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KansoIcons icon,  String label,  Color background,  VoidCallback onPressed)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoSwipeAction() when $default != null:
return $default(_that.icon,_that.label,_that.background,_that.onPressed);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KansoIcons icon,  String label,  Color background,  VoidCallback onPressed)  $default,) {final _that = this;
switch (_that) {
case _KansoSwipeAction():
return $default(_that.icon,_that.label,_that.background,_that.onPressed);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KansoIcons icon,  String label,  Color background,  VoidCallback onPressed)?  $default,) {final _that = this;
switch (_that) {
case _KansoSwipeAction() when $default != null:
return $default(_that.icon,_that.label,_that.background,_that.onPressed);case _:
  return null;

}
}

}

/// @nodoc


class _KansoSwipeAction implements KansoSwipeAction {
  const _KansoSwipeAction({required this.icon, required this.label, required this.background, required this.onPressed});
  

@override final  KansoIcons icon;
/// What it says, under the picture and to a screen reader.
@override final  String label;
/// The colour behind it.
@override final  Color background;
/// What a tap on it does.
@override final  VoidCallback onPressed;

/// Create a copy of KansoSwipeAction
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KansoSwipeActionCopyWith<_KansoSwipeAction> get copyWith => __$KansoSwipeActionCopyWithImpl<_KansoSwipeAction>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoSwipeAction&&(identical(other.icon, icon) || other.icon == icon)&&(identical(other.label, label) || other.label == label)&&(identical(other.background, background) || other.background == background)&&(identical(other.onPressed, onPressed) || other.onPressed == onPressed));
}


@override
int get hashCode => Object.hash(runtimeType,icon,label,background,onPressed);

@override
String toString() {
  return 'KansoSwipeAction(icon: $icon, label: $label, background: $background, onPressed: $onPressed)';
}


}

/// @nodoc
abstract mixin class _$KansoSwipeActionCopyWith<$Res> implements $KansoSwipeActionCopyWith<$Res> {
  factory _$KansoSwipeActionCopyWith(_KansoSwipeAction value, $Res Function(_KansoSwipeAction) _then) = __$KansoSwipeActionCopyWithImpl;
@override @useResult
$Res call({
 KansoIcons icon, String label, Color background, VoidCallback onPressed
});




}
/// @nodoc
class __$KansoSwipeActionCopyWithImpl<$Res>
    implements _$KansoSwipeActionCopyWith<$Res> {
  __$KansoSwipeActionCopyWithImpl(this._self, this._then);

  final _KansoSwipeAction _self;
  final $Res Function(_KansoSwipeAction) _then;

/// Create a copy of KansoSwipeAction
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? icon = null,Object? label = null,Object? background = null,Object? onPressed = null,}) {
  return _then(_KansoSwipeAction(
icon: null == icon ? _self.icon : icon // ignore: cast_nullable_to_non_nullable
as KansoIcons,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,background: null == background ? _self.background : background // ignore: cast_nullable_to_non_nullable
as Color,onPressed: null == onPressed ? _self.onPressed : onPressed // ignore: cast_nullable_to_non_nullable
as VoidCallback,
  ));
}


}

// dart format on
