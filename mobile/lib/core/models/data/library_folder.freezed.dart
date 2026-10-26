// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'library_folder.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LibraryFolder {

 String get path;
/// Create a copy of LibraryFolder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LibraryFolderCopyWith<LibraryFolder> get copyWith => _$LibraryFolderCopyWithImpl<LibraryFolder>(this as LibraryFolder, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LibraryFolder&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'LibraryFolder(path: $path)';
}


}

/// @nodoc
abstract mixin class $LibraryFolderCopyWith<$Res>  {
  factory $LibraryFolderCopyWith(LibraryFolder value, $Res Function(LibraryFolder) _then) = _$LibraryFolderCopyWithImpl;
@useResult
$Res call({
 String path
});




}
/// @nodoc
class _$LibraryFolderCopyWithImpl<$Res>
    implements $LibraryFolderCopyWith<$Res> {
  _$LibraryFolderCopyWithImpl(this._self, this._then);

  final LibraryFolder _self;
  final $Res Function(LibraryFolder) _then;

/// Create a copy of LibraryFolder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? path = null,}) {
  return _then(_self.copyWith(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [LibraryFolder].
extension LibraryFolderPatterns on LibraryFolder {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LibraryFolder value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LibraryFolder() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LibraryFolder value)  $default,){
final _that = this;
switch (_that) {
case _LibraryFolder():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LibraryFolder value)?  $default,){
final _that = this;
switch (_that) {
case _LibraryFolder() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String path)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LibraryFolder() when $default != null:
return $default(_that.path);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String path)  $default,) {final _that = this;
switch (_that) {
case _LibraryFolder():
return $default(_that.path);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String path)?  $default,) {final _that = this;
switch (_that) {
case _LibraryFolder() when $default != null:
return $default(_that.path);case _:
  return null;

}
}

}

/// @nodoc


class _LibraryFolder extends LibraryFolder {
  const _LibraryFolder(this.path): super._();
  

@override final  String path;

/// Create a copy of LibraryFolder
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LibraryFolderCopyWith<_LibraryFolder> get copyWith => __$LibraryFolderCopyWithImpl<_LibraryFolder>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LibraryFolder&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'LibraryFolder(path: $path)';
}


}

/// @nodoc
abstract mixin class _$LibraryFolderCopyWith<$Res> implements $LibraryFolderCopyWith<$Res> {
  factory _$LibraryFolderCopyWith(_LibraryFolder value, $Res Function(_LibraryFolder) _then) = __$LibraryFolderCopyWithImpl;
@override @useResult
$Res call({
 String path
});




}
/// @nodoc
class __$LibraryFolderCopyWithImpl<$Res>
    implements _$LibraryFolderCopyWith<$Res> {
  __$LibraryFolderCopyWithImpl(this._self, this._then);

  final _LibraryFolder _self;
  final $Res Function(_LibraryFolder) _then;

/// Create a copy of LibraryFolder
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? path = null,}) {
  return _then(_LibraryFolder(
null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
