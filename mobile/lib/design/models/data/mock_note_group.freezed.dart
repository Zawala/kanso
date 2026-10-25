// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_note_group.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockNoteGroup {

 String get label; List<MockNote> get notes;
/// Create a copy of MockNoteGroup
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockNoteGroupCopyWith<MockNoteGroup> get copyWith => _$MockNoteGroupCopyWithImpl<MockNoteGroup>(this as MockNoteGroup, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockNoteGroup&&(identical(other.label, label) || other.label == label)&&const DeepCollectionEquality().equals(other.notes, notes));
}


@override
int get hashCode => Object.hash(runtimeType,label,const DeepCollectionEquality().hash(notes));

@override
String toString() {
  return 'MockNoteGroup(label: $label, notes: $notes)';
}


}

/// @nodoc
abstract mixin class $MockNoteGroupCopyWith<$Res>  {
  factory $MockNoteGroupCopyWith(MockNoteGroup value, $Res Function(MockNoteGroup) _then) = _$MockNoteGroupCopyWithImpl;
@useResult
$Res call({
 String label, List<MockNote> notes
});




}
/// @nodoc
class _$MockNoteGroupCopyWithImpl<$Res>
    implements $MockNoteGroupCopyWith<$Res> {
  _$MockNoteGroupCopyWithImpl(this._self, this._then);

  final MockNoteGroup _self;
  final $Res Function(MockNoteGroup) _then;

/// Create a copy of MockNoteGroup
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? label = null,Object? notes = null,}) {
  return _then(_self.copyWith(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,notes: null == notes ? _self.notes : notes // ignore: cast_nullable_to_non_nullable
as List<MockNote>,
  ));
}

}


/// Adds pattern-matching-related methods to [MockNoteGroup].
extension MockNoteGroupPatterns on MockNoteGroup {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockNoteGroup value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockNoteGroup() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockNoteGroup value)  $default,){
final _that = this;
switch (_that) {
case _MockNoteGroup():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockNoteGroup value)?  $default,){
final _that = this;
switch (_that) {
case _MockNoteGroup() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String label,  List<MockNote> notes)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockNoteGroup() when $default != null:
return $default(_that.label,_that.notes);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String label,  List<MockNote> notes)  $default,) {final _that = this;
switch (_that) {
case _MockNoteGroup():
return $default(_that.label,_that.notes);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String label,  List<MockNote> notes)?  $default,) {final _that = this;
switch (_that) {
case _MockNoteGroup() when $default != null:
return $default(_that.label,_that.notes);case _:
  return null;

}
}

}

/// @nodoc


class _MockNoteGroup implements MockNoteGroup {
  const _MockNoteGroup({required this.label, required final  List<MockNote> notes}): _notes = notes;
  

@override final  String label;
 final  List<MockNote> _notes;
@override List<MockNote> get notes {
  if (_notes is EqualUnmodifiableListView) return _notes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notes);
}


/// Create a copy of MockNoteGroup
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockNoteGroupCopyWith<_MockNoteGroup> get copyWith => __$MockNoteGroupCopyWithImpl<_MockNoteGroup>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockNoteGroup&&(identical(other.label, label) || other.label == label)&&const DeepCollectionEquality().equals(other._notes, _notes));
}


@override
int get hashCode => Object.hash(runtimeType,label,const DeepCollectionEquality().hash(_notes));

@override
String toString() {
  return 'MockNoteGroup(label: $label, notes: $notes)';
}


}

/// @nodoc
abstract mixin class _$MockNoteGroupCopyWith<$Res> implements $MockNoteGroupCopyWith<$Res> {
  factory _$MockNoteGroupCopyWith(_MockNoteGroup value, $Res Function(_MockNoteGroup) _then) = __$MockNoteGroupCopyWithImpl;
@override @useResult
$Res call({
 String label, List<MockNote> notes
});




}
/// @nodoc
class __$MockNoteGroupCopyWithImpl<$Res>
    implements _$MockNoteGroupCopyWith<$Res> {
  __$MockNoteGroupCopyWithImpl(this._self, this._then);

  final _MockNoteGroup _self;
  final $Res Function(_MockNoteGroup) _then;

/// Create a copy of MockNoteGroup
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? label = null,Object? notes = null,}) {
  return _then(_MockNoteGroup(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,notes: null == notes ? _self._notes : notes // ignore: cast_nullable_to_non_nullable
as List<MockNote>,
  ));
}


}

// dart format on
