// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'application_error.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ApplicationError {

 String get message;
/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<ApplicationError> get copyWith => _$ApplicationErrorCopyWithImpl<ApplicationError>(this as ApplicationError, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApplicationError&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApplicationError(message: $message)';
}


}

/// @nodoc
abstract mixin class $ApplicationErrorCopyWith<$Res>  {
  factory $ApplicationErrorCopyWith(ApplicationError value, $Res Function(ApplicationError) _then) = _$ApplicationErrorCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ApplicationErrorCopyWithImpl<$Res>
    implements $ApplicationErrorCopyWith<$Res> {
  _$ApplicationErrorCopyWithImpl(this._self, this._then);

  final ApplicationError _self;
  final $Res Function(ApplicationError) _then;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? message = null,}) {
  return _then(_self.copyWith(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [ApplicationError].
extension ApplicationErrorPatterns on ApplicationError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( UnexpectedError value)?  unexpected,TResult Function( NotFoundError value)?  notFound,TResult Function( LockedError value)?  locked,TResult Function( StorageError value)?  storage,required TResult orElse(),}){
final _that = this;
switch (_that) {
case UnexpectedError() when unexpected != null:
return unexpected(_that);case NotFoundError() when notFound != null:
return notFound(_that);case LockedError() when locked != null:
return locked(_that);case StorageError() when storage != null:
return storage(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( UnexpectedError value)  unexpected,required TResult Function( NotFoundError value)  notFound,required TResult Function( LockedError value)  locked,required TResult Function( StorageError value)  storage,}){
final _that = this;
switch (_that) {
case UnexpectedError():
return unexpected(_that);case NotFoundError():
return notFound(_that);case LockedError():
return locked(_that);case StorageError():
return storage(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( UnexpectedError value)?  unexpected,TResult? Function( NotFoundError value)?  notFound,TResult? Function( LockedError value)?  locked,TResult? Function( StorageError value)?  storage,}){
final _that = this;
switch (_that) {
case UnexpectedError() when unexpected != null:
return unexpected(_that);case NotFoundError() when notFound != null:
return notFound(_that);case LockedError() when locked != null:
return locked(_that);case StorageError() when storage != null:
return storage(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String message)?  unexpected,TResult Function( String message)?  notFound,TResult Function( String message)?  locked,TResult Function( String message)?  storage,required TResult orElse(),}) {final _that = this;
switch (_that) {
case UnexpectedError() when unexpected != null:
return unexpected(_that.message);case NotFoundError() when notFound != null:
return notFound(_that.message);case LockedError() when locked != null:
return locked(_that.message);case StorageError() when storage != null:
return storage(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String message)  unexpected,required TResult Function( String message)  notFound,required TResult Function( String message)  locked,required TResult Function( String message)  storage,}) {final _that = this;
switch (_that) {
case UnexpectedError():
return unexpected(_that.message);case NotFoundError():
return notFound(_that.message);case LockedError():
return locked(_that.message);case StorageError():
return storage(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String message)?  unexpected,TResult? Function( String message)?  notFound,TResult? Function( String message)?  locked,TResult? Function( String message)?  storage,}) {final _that = this;
switch (_that) {
case UnexpectedError() when unexpected != null:
return unexpected(_that.message);case NotFoundError() when notFound != null:
return notFound(_that.message);case LockedError() when locked != null:
return locked(_that.message);case StorageError() when storage != null:
return storage(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class UnexpectedError implements ApplicationError {
  const UnexpectedError({required this.message});
  

@override final  String message;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$UnexpectedErrorCopyWith<UnexpectedError> get copyWith => _$UnexpectedErrorCopyWithImpl<UnexpectedError>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is UnexpectedError&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApplicationError.unexpected(message: $message)';
}


}

/// @nodoc
abstract mixin class $UnexpectedErrorCopyWith<$Res> implements $ApplicationErrorCopyWith<$Res> {
  factory $UnexpectedErrorCopyWith(UnexpectedError value, $Res Function(UnexpectedError) _then) = _$UnexpectedErrorCopyWithImpl;
@override @useResult
$Res call({
 String message
});




}
/// @nodoc
class _$UnexpectedErrorCopyWithImpl<$Res>
    implements $UnexpectedErrorCopyWith<$Res> {
  _$UnexpectedErrorCopyWithImpl(this._self, this._then);

  final UnexpectedError _self;
  final $Res Function(UnexpectedError) _then;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(UnexpectedError(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NotFoundError implements ApplicationError {
  const NotFoundError({required this.message});
  

@override final  String message;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NotFoundErrorCopyWith<NotFoundError> get copyWith => _$NotFoundErrorCopyWithImpl<NotFoundError>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NotFoundError&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApplicationError.notFound(message: $message)';
}


}

/// @nodoc
abstract mixin class $NotFoundErrorCopyWith<$Res> implements $ApplicationErrorCopyWith<$Res> {
  factory $NotFoundErrorCopyWith(NotFoundError value, $Res Function(NotFoundError) _then) = _$NotFoundErrorCopyWithImpl;
@override @useResult
$Res call({
 String message
});




}
/// @nodoc
class _$NotFoundErrorCopyWithImpl<$Res>
    implements $NotFoundErrorCopyWith<$Res> {
  _$NotFoundErrorCopyWithImpl(this._self, this._then);

  final NotFoundError _self;
  final $Res Function(NotFoundError) _then;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(NotFoundError(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LockedError implements ApplicationError {
  const LockedError({required this.message});
  

@override final  String message;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockedErrorCopyWith<LockedError> get copyWith => _$LockedErrorCopyWithImpl<LockedError>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockedError&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApplicationError.locked(message: $message)';
}


}

/// @nodoc
abstract mixin class $LockedErrorCopyWith<$Res> implements $ApplicationErrorCopyWith<$Res> {
  factory $LockedErrorCopyWith(LockedError value, $Res Function(LockedError) _then) = _$LockedErrorCopyWithImpl;
@override @useResult
$Res call({
 String message
});




}
/// @nodoc
class _$LockedErrorCopyWithImpl<$Res>
    implements $LockedErrorCopyWith<$Res> {
  _$LockedErrorCopyWithImpl(this._self, this._then);

  final LockedError _self;
  final $Res Function(LockedError) _then;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(LockedError(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class StorageError implements ApplicationError {
  const StorageError({required this.message});
  

@override final  String message;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageErrorCopyWith<StorageError> get copyWith => _$StorageErrorCopyWithImpl<StorageError>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageError&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApplicationError.storage(message: $message)';
}


}

/// @nodoc
abstract mixin class $StorageErrorCopyWith<$Res> implements $ApplicationErrorCopyWith<$Res> {
  factory $StorageErrorCopyWith(StorageError value, $Res Function(StorageError) _then) = _$StorageErrorCopyWithImpl;
@override @useResult
$Res call({
 String message
});




}
/// @nodoc
class _$StorageErrorCopyWithImpl<$Res>
    implements $StorageErrorCopyWith<$Res> {
  _$StorageErrorCopyWithImpl(this._self, this._then);

  final StorageError _self;
  final $Res Function(StorageError) _then;

/// Create a copy of ApplicationError
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(StorageError(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
