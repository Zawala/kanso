// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'editor_destination.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$EditorDestination {

 String get noteId;
/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorDestinationCopyWith<EditorDestination> get copyWith => _$EditorDestinationCopyWithImpl<EditorDestination>(this as EditorDestination, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorDestination&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'EditorDestination(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $EditorDestinationCopyWith<$Res>  {
  factory $EditorDestinationCopyWith(EditorDestination value, $Res Function(EditorDestination) _then) = _$EditorDestinationCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$EditorDestinationCopyWithImpl<$Res>
    implements $EditorDestinationCopyWith<$Res> {
  _$EditorDestinationCopyWithImpl(this._self, this._then);

  final EditorDestination _self;
  final $Res Function(EditorDestination) _then;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [EditorDestination].
extension EditorDestinationPatterns on EditorDestination {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteDestination value)?  note,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteDestination value)  note,}){
final _that = this;
switch (_that) {
case NoteDestination():
return note(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteDestination value)?  note,}){
final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  note,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that.noteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  note,}) {final _that = this;
switch (_that) {
case NoteDestination():
return note(_that.noteId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  note,}) {final _that = this;
switch (_that) {
case NoteDestination() when note != null:
return note(_that.noteId);case _:
  return null;

}
}

}

/// @nodoc


class NoteDestination implements EditorDestination {
  const NoteDestination(this.noteId);
  

@override final  String noteId;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteDestinationCopyWith<NoteDestination> get copyWith => _$NoteDestinationCopyWithImpl<NoteDestination>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteDestination&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'EditorDestination.note(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $NoteDestinationCopyWith<$Res> implements $EditorDestinationCopyWith<$Res> {
  factory $NoteDestinationCopyWith(NoteDestination value, $Res Function(NoteDestination) _then) = _$NoteDestinationCopyWithImpl;
@override @useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$NoteDestinationCopyWithImpl<$Res>
    implements $NoteDestinationCopyWith<$Res> {
  _$NoteDestinationCopyWithImpl(this._self, this._then);

  final NoteDestination _self;
  final $Res Function(NoteDestination) _then;

/// Create a copy of EditorDestination
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(NoteDestination(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
