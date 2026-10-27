// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_folder.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CrashFolder {

 String get path;
/// Create a copy of CrashFolder
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashFolderCopyWith<CrashFolder> get copyWith => _$CrashFolderCopyWithImpl<CrashFolder>(this as CrashFolder, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashFolder&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'CrashFolder(path: $path)';
}


}

/// @nodoc
abstract mixin class $CrashFolderCopyWith<$Res>  {
  factory $CrashFolderCopyWith(CrashFolder value, $Res Function(CrashFolder) _then) = _$CrashFolderCopyWithImpl;
@useResult
$Res call({
 String path
});




}
/// @nodoc
class _$CrashFolderCopyWithImpl<$Res>
    implements $CrashFolderCopyWith<$Res> {
  _$CrashFolderCopyWithImpl(this._self, this._then);

  final CrashFolder _self;
  final $Res Function(CrashFolder) _then;

/// Create a copy of CrashFolder
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? path = null,}) {
  return _then(_self.copyWith(
path: null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [CrashFolder].
extension CrashFolderPatterns on CrashFolder {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashFolder value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashFolder() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashFolder value)  $default,){
final _that = this;
switch (_that) {
case _CrashFolder():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashFolder value)?  $default,){
final _that = this;
switch (_that) {
case _CrashFolder() when $default != null:
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
case _CrashFolder() when $default != null:
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
case _CrashFolder():
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
case _CrashFolder() when $default != null:
return $default(_that.path);case _:
  return null;

}
}

}

/// @nodoc


class _CrashFolder extends CrashFolder {
  const _CrashFolder(this.path): super._();
  

@override final  String path;

/// Create a copy of CrashFolder
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashFolderCopyWith<_CrashFolder> get copyWith => __$CrashFolderCopyWithImpl<_CrashFolder>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashFolder&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'CrashFolder(path: $path)';
}


}

/// @nodoc
abstract mixin class _$CrashFolderCopyWith<$Res> implements $CrashFolderCopyWith<$Res> {
  factory _$CrashFolderCopyWith(_CrashFolder value, $Res Function(_CrashFolder) _then) = __$CrashFolderCopyWithImpl;
@override @useResult
$Res call({
 String path
});




}
/// @nodoc
class __$CrashFolderCopyWithImpl<$Res>
    implements _$CrashFolderCopyWith<$Res> {
  __$CrashFolderCopyWithImpl(this._self, this._then);

  final _CrashFolder _self;
  final $Res Function(_CrashFolder) _then;

/// Create a copy of CrashFolder
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? path = null,}) {
  return _then(_CrashFolder(
null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
