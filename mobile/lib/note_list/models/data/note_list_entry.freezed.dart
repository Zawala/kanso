// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_list_entry.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteListEntry {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteListEntry);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteListEntry()';
}


}

/// @nodoc
class $NoteListEntryCopyWith<$Res>  {
$NoteListEntryCopyWith(NoteListEntry _, $Res Function(NoteListEntry) __);
}


/// Adds pattern-matching-related methods to [NoteListEntry].
extension NoteListEntryPatterns on NoteListEntry {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( HeaderEntry value)?  header,TResult Function( NoteEntry value)?  note,required TResult orElse(),}){
final _that = this;
switch (_that) {
case HeaderEntry() when header != null:
return header(_that);case NoteEntry() when note != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( HeaderEntry value)  header,required TResult Function( NoteEntry value)  note,}){
final _that = this;
switch (_that) {
case HeaderEntry():
return header(_that);case NoteEntry():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( HeaderEntry value)?  header,TResult? Function( NoteEntry value)?  note,}){
final _that = this;
switch (_that) {
case HeaderEntry() when header != null:
return header(_that);case NoteEntry() when note != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( Bucket bucket)?  header,TResult Function( NoteRow row)?  note,required TResult orElse(),}) {final _that = this;
switch (_that) {
case HeaderEntry() when header != null:
return header(_that.bucket);case NoteEntry() when note != null:
return note(_that.row);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( Bucket bucket)  header,required TResult Function( NoteRow row)  note,}) {final _that = this;
switch (_that) {
case HeaderEntry():
return header(_that.bucket);case NoteEntry():
return note(_that.row);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( Bucket bucket)?  header,TResult? Function( NoteRow row)?  note,}) {final _that = this;
switch (_that) {
case HeaderEntry() when header != null:
return header(_that.bucket);case NoteEntry() when note != null:
return note(_that.row);case _:
  return null;

}
}

}

/// @nodoc


class HeaderEntry implements NoteListEntry {
  const HeaderEntry(this.bucket);
  

 final  Bucket bucket;

/// Create a copy of NoteListEntry
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$HeaderEntryCopyWith<HeaderEntry> get copyWith => _$HeaderEntryCopyWithImpl<HeaderEntry>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is HeaderEntry&&(identical(other.bucket, bucket) || other.bucket == bucket));
}


@override
int get hashCode => Object.hash(runtimeType,bucket);

@override
String toString() {
  return 'NoteListEntry.header(bucket: $bucket)';
}


}

/// @nodoc
abstract mixin class $HeaderEntryCopyWith<$Res> implements $NoteListEntryCopyWith<$Res> {
  factory $HeaderEntryCopyWith(HeaderEntry value, $Res Function(HeaderEntry) _then) = _$HeaderEntryCopyWithImpl;
@useResult
$Res call({
 Bucket bucket
});


$BucketCopyWith<$Res> get bucket;

}
/// @nodoc
class _$HeaderEntryCopyWithImpl<$Res>
    implements $HeaderEntryCopyWith<$Res> {
  _$HeaderEntryCopyWithImpl(this._self, this._then);

  final HeaderEntry _self;
  final $Res Function(HeaderEntry) _then;

/// Create a copy of NoteListEntry
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? bucket = null,}) {
  return _then(HeaderEntry(
null == bucket ? _self.bucket : bucket // ignore: cast_nullable_to_non_nullable
as Bucket,
  ));
}

/// Create a copy of NoteListEntry
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BucketCopyWith<$Res> get bucket {
  
  return $BucketCopyWith<$Res>(_self.bucket, (value) {
    return _then(_self.copyWith(bucket: value));
  });
}
}

/// @nodoc


class NoteEntry implements NoteListEntry {
  const NoteEntry(this.row);
  

 final  NoteRow row;

/// Create a copy of NoteListEntry
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteEntryCopyWith<NoteEntry> get copyWith => _$NoteEntryCopyWithImpl<NoteEntry>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteEntry&&(identical(other.row, row) || other.row == row));
}


@override
int get hashCode => Object.hash(runtimeType,row);

@override
String toString() {
  return 'NoteListEntry.note(row: $row)';
}


}

/// @nodoc
abstract mixin class $NoteEntryCopyWith<$Res> implements $NoteListEntryCopyWith<$Res> {
  factory $NoteEntryCopyWith(NoteEntry value, $Res Function(NoteEntry) _then) = _$NoteEntryCopyWithImpl;
@useResult
$Res call({
 NoteRow row
});




}
/// @nodoc
class _$NoteEntryCopyWithImpl<$Res>
    implements $NoteEntryCopyWith<$Res> {
  _$NoteEntryCopyWithImpl(this._self, this._then);

  final NoteEntry _self;
  final $Res Function(NoteEntry) _then;

/// Create a copy of NoteListEntry
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? row = null,}) {
  return _then(NoteEntry(
null == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as NoteRow,
  ));
}


}

// dart format on
