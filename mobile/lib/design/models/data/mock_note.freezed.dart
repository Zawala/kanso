// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_note.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockNote {

 String get title; String get excerpt; String get notebook; String get time; NoteStatus get status; List<MockTag> get tags; bool get pinned; bool get locked;
/// Create a copy of MockNote
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockNoteCopyWith<MockNote> get copyWith => _$MockNoteCopyWithImpl<MockNote>(this as MockNote, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockNote&&(identical(other.title, title) || other.title == title)&&(identical(other.excerpt, excerpt) || other.excerpt == excerpt)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.time, time) || other.time == time)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.tags, tags)&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.locked, locked) || other.locked == locked));
}


@override
int get hashCode => Object.hash(runtimeType,title,excerpt,notebook,time,status,const DeepCollectionEquality().hash(tags),pinned,locked);

@override
String toString() {
  return 'MockNote(title: $title, excerpt: $excerpt, notebook: $notebook, time: $time, status: $status, tags: $tags, pinned: $pinned, locked: $locked)';
}


}

/// @nodoc
abstract mixin class $MockNoteCopyWith<$Res>  {
  factory $MockNoteCopyWith(MockNote value, $Res Function(MockNote) _then) = _$MockNoteCopyWithImpl;
@useResult
$Res call({
 String title, String excerpt, String notebook, String time, NoteStatus status, List<MockTag> tags, bool pinned, bool locked
});




}
/// @nodoc
class _$MockNoteCopyWithImpl<$Res>
    implements $MockNoteCopyWith<$Res> {
  _$MockNoteCopyWithImpl(this._self, this._then);

  final MockNote _self;
  final $Res Function(MockNote) _then;

/// Create a copy of MockNote
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? title = null,Object? excerpt = null,Object? notebook = null,Object? time = null,Object? status = null,Object? tags = null,Object? pinned = null,Object? locked = null,}) {
  return _then(_self.copyWith(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,excerpt: null == excerpt ? _self.excerpt : excerpt // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,time: null == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteStatus,tags: null == tags ? _self.tags : tags // ignore: cast_nullable_to_non_nullable
as List<MockTag>,pinned: null == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as bool,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [MockNote].
extension MockNotePatterns on MockNote {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MockNote value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MockNote() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MockNote value)  $default,){
final _that = this;
switch (_that) {
case _MockNote():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MockNote value)?  $default,){
final _that = this;
switch (_that) {
case _MockNote() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String title,  String excerpt,  String notebook,  String time,  NoteStatus status,  List<MockTag> tags,  bool pinned,  bool locked)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MockNote() when $default != null:
return $default(_that.title,_that.excerpt,_that.notebook,_that.time,_that.status,_that.tags,_that.pinned,_that.locked);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String title,  String excerpt,  String notebook,  String time,  NoteStatus status,  List<MockTag> tags,  bool pinned,  bool locked)  $default,) {final _that = this;
switch (_that) {
case _MockNote():
return $default(_that.title,_that.excerpt,_that.notebook,_that.time,_that.status,_that.tags,_that.pinned,_that.locked);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String title,  String excerpt,  String notebook,  String time,  NoteStatus status,  List<MockTag> tags,  bool pinned,  bool locked)?  $default,) {final _that = this;
switch (_that) {
case _MockNote() when $default != null:
return $default(_that.title,_that.excerpt,_that.notebook,_that.time,_that.status,_that.tags,_that.pinned,_that.locked);case _:
  return null;

}
}

}

/// @nodoc


class _MockNote implements MockNote {
  const _MockNote({required this.title, required this.excerpt, required this.notebook, required this.time, this.status = NoteStatus.none, final  List<MockTag> tags = const <MockTag>[], this.pinned = false, this.locked = false}): _tags = tags;
  

@override final  String title;
@override final  String excerpt;
@override final  String notebook;
@override final  String time;
@override@JsonKey() final  NoteStatus status;
 final  List<MockTag> _tags;
@override@JsonKey() List<MockTag> get tags {
  if (_tags is EqualUnmodifiableListView) return _tags;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_tags);
}

@override@JsonKey() final  bool pinned;
@override@JsonKey() final  bool locked;

/// Create a copy of MockNote
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MockNoteCopyWith<_MockNote> get copyWith => __$MockNoteCopyWithImpl<_MockNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MockNote&&(identical(other.title, title) || other.title == title)&&(identical(other.excerpt, excerpt) || other.excerpt == excerpt)&&(identical(other.notebook, notebook) || other.notebook == notebook)&&(identical(other.time, time) || other.time == time)&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._tags, _tags)&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.locked, locked) || other.locked == locked));
}


@override
int get hashCode => Object.hash(runtimeType,title,excerpt,notebook,time,status,const DeepCollectionEquality().hash(_tags),pinned,locked);

@override
String toString() {
  return 'MockNote(title: $title, excerpt: $excerpt, notebook: $notebook, time: $time, status: $status, tags: $tags, pinned: $pinned, locked: $locked)';
}


}

/// @nodoc
abstract mixin class _$MockNoteCopyWith<$Res> implements $MockNoteCopyWith<$Res> {
  factory _$MockNoteCopyWith(_MockNote value, $Res Function(_MockNote) _then) = __$MockNoteCopyWithImpl;
@override @useResult
$Res call({
 String title, String excerpt, String notebook, String time, NoteStatus status, List<MockTag> tags, bool pinned, bool locked
});




}
/// @nodoc
class __$MockNoteCopyWithImpl<$Res>
    implements _$MockNoteCopyWith<$Res> {
  __$MockNoteCopyWithImpl(this._self, this._then);

  final _MockNote _self;
  final $Res Function(_MockNote) _then;

/// Create a copy of MockNote
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? title = null,Object? excerpt = null,Object? notebook = null,Object? time = null,Object? status = null,Object? tags = null,Object? pinned = null,Object? locked = null,}) {
  return _then(_MockNote(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,excerpt: null == excerpt ? _self.excerpt : excerpt // ignore: cast_nullable_to_non_nullable
as String,notebook: null == notebook ? _self.notebook : notebook // ignore: cast_nullable_to_non_nullable
as String,time: null == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as NoteStatus,tags: null == tags ? _self._tags : tags // ignore: cast_nullable_to_non_nullable
as List<MockTag>,pinned: null == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as bool,locked: null == locked ? _self.locked : locked // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
