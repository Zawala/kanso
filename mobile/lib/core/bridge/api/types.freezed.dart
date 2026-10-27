// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KansoError {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError()';
}


}

/// @nodoc
class $KansoErrorCopyWith<$Res>  {
$KansoErrorCopyWith(KansoError _, $Res Function(KansoError) __);
}


/// Adds pattern-matching-related methods to [KansoError].
extension KansoErrorPatterns on KansoError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( KansoError_NotFound value)?  notFound,TResult Function( KansoError_Locked value)?  locked,TResult Function( KansoError_Conflict value)?  conflict,TResult Function( KansoError_Invalid value)?  invalid,TResult Function( KansoError_Storage value)?  storage,TResult Function( KansoError_Network value)?  network,TResult Function( KansoError_SignedOut value)?  signedOut,TResult Function( KansoError_Auth value)?  auth,TResult Function( KansoError_Internal value)?  internal,required TResult orElse(),}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( KansoError_NotFound value)  notFound,required TResult Function( KansoError_Locked value)  locked,required TResult Function( KansoError_Conflict value)  conflict,required TResult Function( KansoError_Invalid value)  invalid,required TResult Function( KansoError_Storage value)  storage,required TResult Function( KansoError_Network value)  network,required TResult Function( KansoError_SignedOut value)  signedOut,required TResult Function( KansoError_Auth value)  auth,required TResult Function( KansoError_Internal value)  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound(_that);case KansoError_Locked():
return locked(_that);case KansoError_Conflict():
return conflict(_that);case KansoError_Invalid():
return invalid(_that);case KansoError_Storage():
return storage(_that);case KansoError_Network():
return network(_that);case KansoError_SignedOut():
return signedOut(_that);case KansoError_Auth():
return auth(_that);case KansoError_Internal():
return internal(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( KansoError_NotFound value)?  notFound,TResult? Function( KansoError_Locked value)?  locked,TResult? Function( KansoError_Conflict value)?  conflict,TResult? Function( KansoError_Invalid value)?  invalid,TResult? Function( KansoError_Storage value)?  storage,TResult? Function( KansoError_Network value)?  network,TResult? Function( KansoError_SignedOut value)?  signedOut,TResult? Function( KansoError_Auth value)?  auth,TResult? Function( KansoError_Internal value)?  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  notFound,TResult Function()?  locked,TResult Function( String currentRev)?  conflict,TResult Function( String message)?  invalid,TResult Function( String message)?  storage,TResult Function( String message)?  network,TResult Function()?  signedOut,TResult Function( String message)?  auth,TResult Function( String message)?  internal,required TResult orElse(),}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  notFound,required TResult Function()  locked,required TResult Function( String currentRev)  conflict,required TResult Function( String message)  invalid,required TResult Function( String message)  storage,required TResult Function( String message)  network,required TResult Function()  signedOut,required TResult Function( String message)  auth,required TResult Function( String message)  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound();case KansoError_Locked():
return locked();case KansoError_Conflict():
return conflict(_that.currentRev);case KansoError_Invalid():
return invalid(_that.message);case KansoError_Storage():
return storage(_that.message);case KansoError_Network():
return network(_that.message);case KansoError_SignedOut():
return signedOut();case KansoError_Auth():
return auth(_that.message);case KansoError_Internal():
return internal(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  notFound,TResult? Function()?  locked,TResult? Function( String currentRev)?  conflict,TResult? Function( String message)?  invalid,TResult? Function( String message)?  storage,TResult? Function( String message)?  network,TResult? Function()?  signedOut,TResult? Function( String message)?  auth,TResult? Function( String message)?  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class KansoError_NotFound extends KansoError {
  const KansoError_NotFound(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_NotFound);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.notFound()';
}


}




/// @nodoc


class KansoError_Locked extends KansoError {
  const KansoError_Locked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Locked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.locked()';
}


}




/// @nodoc


class KansoError_Conflict extends KansoError {
  const KansoError_Conflict({required this.currentRev}): super._();
  

 final  String currentRev;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_ConflictCopyWith<KansoError_Conflict> get copyWith => _$KansoError_ConflictCopyWithImpl<KansoError_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Conflict&&(identical(other.currentRev, currentRev) || other.currentRev == currentRev));
}


@override
int get hashCode => Object.hash(runtimeType,currentRev);

@override
String toString() {
  return 'KansoError.conflict(currentRev: $currentRev)';
}


}

/// @nodoc
abstract mixin class $KansoError_ConflictCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_ConflictCopyWith(KansoError_Conflict value, $Res Function(KansoError_Conflict) _then) = _$KansoError_ConflictCopyWithImpl;
@useResult
$Res call({
 String currentRev
});




}
/// @nodoc
class _$KansoError_ConflictCopyWithImpl<$Res>
    implements $KansoError_ConflictCopyWith<$Res> {
  _$KansoError_ConflictCopyWithImpl(this._self, this._then);

  final KansoError_Conflict _self;
  final $Res Function(KansoError_Conflict) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? currentRev = null,}) {
  return _then(KansoError_Conflict(
currentRev: null == currentRev ? _self.currentRev : currentRev // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Invalid extends KansoError {
  const KansoError_Invalid({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InvalidCopyWith<KansoError_Invalid> get copyWith => _$KansoError_InvalidCopyWithImpl<KansoError_Invalid>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Invalid&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.invalid(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InvalidCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InvalidCopyWith(KansoError_Invalid value, $Res Function(KansoError_Invalid) _then) = _$KansoError_InvalidCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InvalidCopyWithImpl<$Res>
    implements $KansoError_InvalidCopyWith<$Res> {
  _$KansoError_InvalidCopyWithImpl(this._self, this._then);

  final KansoError_Invalid _self;
  final $Res Function(KansoError_Invalid) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Invalid(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Storage extends KansoError {
  const KansoError_Storage({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_StorageCopyWith<KansoError_Storage> get copyWith => _$KansoError_StorageCopyWithImpl<KansoError_Storage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Storage&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.storage(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_StorageCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_StorageCopyWith(KansoError_Storage value, $Res Function(KansoError_Storage) _then) = _$KansoError_StorageCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_StorageCopyWithImpl<$Res>
    implements $KansoError_StorageCopyWith<$Res> {
  _$KansoError_StorageCopyWithImpl(this._self, this._then);

  final KansoError_Storage _self;
  final $Res Function(KansoError_Storage) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Storage(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Network extends KansoError {
  const KansoError_Network({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_NetworkCopyWith<KansoError_Network> get copyWith => _$KansoError_NetworkCopyWithImpl<KansoError_Network>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Network&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.network(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_NetworkCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_NetworkCopyWith(KansoError_Network value, $Res Function(KansoError_Network) _then) = _$KansoError_NetworkCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_NetworkCopyWithImpl<$Res>
    implements $KansoError_NetworkCopyWith<$Res> {
  _$KansoError_NetworkCopyWithImpl(this._self, this._then);

  final KansoError_Network _self;
  final $Res Function(KansoError_Network) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Network(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_SignedOut extends KansoError {
  const KansoError_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.signedOut()';
}


}




/// @nodoc


class KansoError_Auth extends KansoError {
  const KansoError_Auth({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_AuthCopyWith<KansoError_Auth> get copyWith => _$KansoError_AuthCopyWithImpl<KansoError_Auth>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Auth&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.auth(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_AuthCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_AuthCopyWith(KansoError_Auth value, $Res Function(KansoError_Auth) _then) = _$KansoError_AuthCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_AuthCopyWithImpl<$Res>
    implements $KansoError_AuthCopyWith<$Res> {
  _$KansoError_AuthCopyWithImpl(this._self, this._then);

  final KansoError_Auth _self;
  final $Res Function(KansoError_Auth) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Auth(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Internal extends KansoError {
  const KansoError_Internal({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InternalCopyWith<KansoError_Internal> get copyWith => _$KansoError_InternalCopyWithImpl<KansoError_Internal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Internal&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.internal(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InternalCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InternalCopyWith(KansoError_Internal value, $Res Function(KansoError_Internal) _then) = _$KansoError_InternalCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InternalCopyWithImpl<$Res>
    implements $KansoError_InternalCopyWith<$Res> {
  _$KansoError_InternalCopyWithImpl(this._self, this._then);

  final KansoError_Internal _self;
  final $Res Function(KansoError_Internal) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Internal(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ListPlace {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace()';
}


}

/// @nodoc
class $ListPlaceCopyWith<$Res>  {
$ListPlaceCopyWith(ListPlace _, $Res Function(ListPlace) __);
}


/// Adds pattern-matching-related methods to [ListPlace].
extension ListPlacePatterns on ListPlace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ListPlace_All value)?  all,TResult Function( ListPlace_Pinned value)?  pinned,TResult Function( ListPlace_Recent value)?  recent,TResult Function( ListPlace_Notebook value)?  notebook,TResult Function( ListPlace_Tag value)?  tag,TResult Function( ListPlace_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ListPlace_All value)  all,required TResult Function( ListPlace_Pinned value)  pinned,required TResult Function( ListPlace_Recent value)  recent,required TResult Function( ListPlace_Notebook value)  notebook,required TResult Function( ListPlace_Tag value)  tag,required TResult Function( ListPlace_Trash value)  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All():
return all(_that);case ListPlace_Pinned():
return pinned(_that);case ListPlace_Recent():
return recent(_that);case ListPlace_Notebook():
return notebook(_that);case ListPlace_Tag():
return tag(_that);case ListPlace_Trash():
return trash(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ListPlace_All value)?  all,TResult? Function( ListPlace_Pinned value)?  pinned,TResult? Function( ListPlace_Recent value)?  recent,TResult? Function( ListPlace_Notebook value)?  notebook,TResult? Function( ListPlace_Tag value)?  tag,TResult? Function( ListPlace_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  pinned,TResult Function()?  recent,TResult Function( String id)?  notebook,TResult Function( String id)?  tag,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Trash() when trash != null:
return trash();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  pinned,required TResult Function()  recent,required TResult Function( String id)  notebook,required TResult Function( String id)  tag,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All():
return all();case ListPlace_Pinned():
return pinned();case ListPlace_Recent():
return recent();case ListPlace_Notebook():
return notebook(_that.id);case ListPlace_Tag():
return tag(_that.id);case ListPlace_Trash():
return trash();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  pinned,TResult? Function()?  recent,TResult? Function( String id)?  notebook,TResult? Function( String id)?  tag,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class ListPlace_All extends ListPlace {
  const ListPlace_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.all()';
}


}




/// @nodoc


class ListPlace_Pinned extends ListPlace {
  const ListPlace_Pinned(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Pinned);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.pinned()';
}


}




/// @nodoc


class ListPlace_Recent extends ListPlace {
  const ListPlace_Recent(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Recent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.recent()';
}


}




/// @nodoc


class ListPlace_Notebook extends ListPlace {
  const ListPlace_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_NotebookCopyWith<ListPlace_Notebook> get copyWith => _$ListPlace_NotebookCopyWithImpl<ListPlace_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_NotebookCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_NotebookCopyWith(ListPlace_Notebook value, $Res Function(ListPlace_Notebook) _then) = _$ListPlace_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_NotebookCopyWithImpl<$Res>
    implements $ListPlace_NotebookCopyWith<$Res> {
  _$ListPlace_NotebookCopyWithImpl(this._self, this._then);

  final ListPlace_Notebook _self;
  final $Res Function(ListPlace_Notebook) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Tag extends ListPlace {
  const ListPlace_Tag({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_TagCopyWith<ListPlace_Tag> get copyWith => _$ListPlace_TagCopyWithImpl<ListPlace_Tag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Tag&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.tag(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_TagCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_TagCopyWith(ListPlace_Tag value, $Res Function(ListPlace_Tag) _then) = _$ListPlace_TagCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_TagCopyWithImpl<$Res>
    implements $ListPlace_TagCopyWith<$Res> {
  _$ListPlace_TagCopyWithImpl(this._self, this._then);

  final ListPlace_Tag _self;
  final $Res Function(ListPlace_Tag) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Tag(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Trash extends ListPlace {
  const ListPlace_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.trash()';
}


}




/// @nodoc
mixin _$LockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent()';
}


}

/// @nodoc
class $LockEventCopyWith<$Res>  {
$LockEventCopyWith(LockEvent _, $Res Function(LockEvent) __);
}


/// Adds pattern-matching-related methods to [LockEvent].
extension LockEventPatterns on LockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockEvent_Unlocked value)?  unlocked,TResult Function( LockEvent_Locked value)?  locked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockEvent_Unlocked value)  unlocked,required TResult Function( LockEvent_Locked value)  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked(_that);case LockEvent_Locked():
return locked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockEvent_Unlocked value)?  unlocked,TResult? Function( LockEvent_Locked value)?  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  unlocked,TResult Function( LockReason reason)?  locked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  unlocked,required TResult Function( LockReason reason)  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked();case LockEvent_Locked():
return locked(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  unlocked,TResult? Function( LockReason reason)?  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockEvent_Unlocked extends LockEvent {
  const LockEvent_Unlocked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Unlocked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent.unlocked()';
}


}




/// @nodoc


class LockEvent_Locked extends LockEvent {
  const LockEvent_Locked({required this.reason}): super._();
  

 final  LockReason reason;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockEvent_LockedCopyWith<LockEvent_Locked> get copyWith => _$LockEvent_LockedCopyWithImpl<LockEvent_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Locked&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockEvent.locked(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockEvent_LockedCopyWith<$Res> implements $LockEventCopyWith<$Res> {
  factory $LockEvent_LockedCopyWith(LockEvent_Locked value, $Res Function(LockEvent_Locked) _then) = _$LockEvent_LockedCopyWithImpl;
@useResult
$Res call({
 LockReason reason
});




}
/// @nodoc
class _$LockEvent_LockedCopyWithImpl<$Res>
    implements $LockEvent_LockedCopyWith<$Res> {
  _$LockEvent_LockedCopyWithImpl(this._self, this._then);

  final LockEvent_Locked _self;
  final $Res Function(LockEvent_Locked) _then;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(LockEvent_Locked(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as LockReason,
  ));
}


}

/// @nodoc
mixin _$LockState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState()';
}


}

/// @nodoc
class $LockStateCopyWith<$Res>  {
$LockStateCopyWith(LockState _, $Res Function(LockState) __);
}


/// Adds pattern-matching-related methods to [LockState].
extension LockStatePatterns on LockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockState_Plain value)?  plain,TResult Function( LockState_Open value)?  open,TResult Function( LockState_Closed value)?  closed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockState_Plain value)  plain,required TResult Function( LockState_Open value)  open,required TResult Function( LockState_Closed value)  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain():
return plain(_that);case LockState_Open():
return open(_that);case LockState_Closed():
return closed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockState_Plain value)?  plain,TResult? Function( LockState_Open value)?  open,TResult? Function( LockState_Closed value)?  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  plain,TResult Function()?  open,TResult Function( String? reason)?  closed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  plain,required TResult Function()  open,required TResult Function( String? reason)  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain():
return plain();case LockState_Open():
return open();case LockState_Closed():
return closed(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  plain,TResult? Function()?  open,TResult? Function( String? reason)?  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockState_Plain extends LockState {
  const LockState_Plain(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Plain);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.plain()';
}


}




/// @nodoc


class LockState_Open extends LockState {
  const LockState_Open(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Open);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.open()';
}


}




/// @nodoc


class LockState_Closed extends LockState {
  const LockState_Closed({this.reason}): super._();
  

 final  String? reason;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockState_ClosedCopyWith<LockState_Closed> get copyWith => _$LockState_ClosedCopyWithImpl<LockState_Closed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Closed&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockState.closed(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockState_ClosedCopyWith<$Res> implements $LockStateCopyWith<$Res> {
  factory $LockState_ClosedCopyWith(LockState_Closed value, $Res Function(LockState_Closed) _then) = _$LockState_ClosedCopyWithImpl;
@useResult
$Res call({
 String? reason
});




}
/// @nodoc
class _$LockState_ClosedCopyWithImpl<$Res>
    implements $LockState_ClosedCopyWith<$Res> {
  _$LockState_ClosedCopyWithImpl(this._self, this._then);

  final LockState_Closed _self;
  final $Res Function(LockState_Closed) _then;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = freezed,}) {
  return _then(LockState_Closed(
reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$NoteChange {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange()';
}


}

/// @nodoc
class $NoteChangeCopyWith<$Res>  {
$NoteChangeCopyWith(NoteChange _, $Res Function(NoteChange) __);
}


/// Adds pattern-matching-related methods to [NoteChange].
extension NoteChangePatterns on NoteChange {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteChange_TogglePin value)?  togglePin,TResult Function( NoteChange_Move value)?  move,TResult Function( NoteChange_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteChange_TogglePin value)  togglePin,required TResult Function( NoteChange_Move value)  move,required TResult Function( NoteChange_Trash value)  trash,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin(_that);case NoteChange_Move():
return move(_that);case NoteChange_Trash():
return trash(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteChange_TogglePin value)?  togglePin,TResult? Function( NoteChange_Move value)?  move,TResult? Function( NoteChange_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  togglePin,TResult Function( String notebookId)?  move,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  togglePin,required TResult Function( String notebookId)  move,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin();case NoteChange_Move():
return move(_that.notebookId);case NoteChange_Trash():
return trash();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  togglePin,TResult? Function( String notebookId)?  move,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class NoteChange_TogglePin extends NoteChange {
  const NoteChange_TogglePin(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_TogglePin);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.togglePin()';
}


}




/// @nodoc


class NoteChange_Move extends NoteChange {
  const NoteChange_Move({required this.notebookId}): super._();
  

 final  String notebookId;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_MoveCopyWith<NoteChange_Move> get copyWith => _$NoteChange_MoveCopyWithImpl<NoteChange_Move>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Move&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'NoteChange.move(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $NoteChange_MoveCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_MoveCopyWith(NoteChange_Move value, $Res Function(NoteChange_Move) _then) = _$NoteChange_MoveCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$NoteChange_MoveCopyWithImpl<$Res>
    implements $NoteChange_MoveCopyWith<$Res> {
  _$NoteChange_MoveCopyWithImpl(this._self, this._then);

  final NoteChange_Move _self;
  final $Res Function(NoteChange_Move) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(NoteChange_Move(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_Trash extends NoteChange {
  const NoteChange_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.trash()';
}


}




/// @nodoc
mixin _$SearchScope {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope()';
}


}

/// @nodoc
class $SearchScopeCopyWith<$Res>  {
$SearchScopeCopyWith(SearchScope _, $Res Function(SearchScope) __);
}


/// Adds pattern-matching-related methods to [SearchScope].
extension SearchScopePatterns on SearchScope {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SearchScope_All value)?  all,TResult Function( SearchScope_Titles value)?  titles,TResult Function( SearchScope_Tags value)?  tags,TResult Function( SearchScope_Notebook value)?  notebook,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SearchScope_All value)  all,required TResult Function( SearchScope_Titles value)  titles,required TResult Function( SearchScope_Tags value)  tags,required TResult Function( SearchScope_Notebook value)  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All():
return all(_that);case SearchScope_Titles():
return titles(_that);case SearchScope_Tags():
return tags(_that);case SearchScope_Notebook():
return notebook(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SearchScope_All value)?  all,TResult? Function( SearchScope_Titles value)?  titles,TResult? Function( SearchScope_Tags value)?  tags,TResult? Function( SearchScope_Notebook value)?  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  titles,TResult Function()?  tags,TResult Function( String id)?  notebook,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  titles,required TResult Function()  tags,required TResult Function( String id)  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All():
return all();case SearchScope_Titles():
return titles();case SearchScope_Tags():
return tags();case SearchScope_Notebook():
return notebook(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  titles,TResult? Function()?  tags,TResult? Function( String id)?  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class SearchScope_All extends SearchScope {
  const SearchScope_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.all()';
}


}




/// @nodoc


class SearchScope_Titles extends SearchScope {
  const SearchScope_Titles(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Titles);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.titles()';
}


}




/// @nodoc


class SearchScope_Tags extends SearchScope {
  const SearchScope_Tags(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Tags);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.tags()';
}


}




/// @nodoc


class SearchScope_Notebook extends SearchScope {
  const SearchScope_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchScope_NotebookCopyWith<SearchScope_Notebook> get copyWith => _$SearchScope_NotebookCopyWithImpl<SearchScope_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'SearchScope.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $SearchScope_NotebookCopyWith<$Res> implements $SearchScopeCopyWith<$Res> {
  factory $SearchScope_NotebookCopyWith(SearchScope_Notebook value, $Res Function(SearchScope_Notebook) _then) = _$SearchScope_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$SearchScope_NotebookCopyWithImpl<$Res>
    implements $SearchScope_NotebookCopyWith<$Res> {
  _$SearchScope_NotebookCopyWithImpl(this._self, this._then);

  final SearchScope_Notebook _self;
  final $Res Function(SearchScope_Notebook) _then;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(SearchScope_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$Ticked {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked()';
}


}

/// @nodoc
class $TickedCopyWith<$Res>  {
$TickedCopyWith(Ticked _, $Res Function(Ticked) __);
}


/// Adds pattern-matching-related methods to [Ticked].
extension TickedPatterns on Ticked {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( Ticked_Changed value)?  changed,TResult Function( Ticked_Unchanged value)?  unchanged,TResult Function( Ticked_Missing value)?  missing,required TResult orElse(),}){
final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that);case Ticked_Unchanged() when unchanged != null:
return unchanged(_that);case Ticked_Missing() when missing != null:
return missing(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( Ticked_Changed value)  changed,required TResult Function( Ticked_Unchanged value)  unchanged,required TResult Function( Ticked_Missing value)  missing,}){
final _that = this;
switch (_that) {
case Ticked_Changed():
return changed(_that);case Ticked_Unchanged():
return unchanged(_that);case Ticked_Missing():
return missing(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( Ticked_Changed value)?  changed,TResult? Function( Ticked_Unchanged value)?  unchanged,TResult? Function( Ticked_Missing value)?  missing,}){
final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that);case Ticked_Unchanged() when unchanged != null:
return unchanged(_that);case Ticked_Missing() when missing != null:
return missing(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int undo,  String message)?  changed,TResult Function()?  unchanged,TResult Function()?  missing,required TResult orElse(),}) {final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged() when unchanged != null:
return unchanged();case Ticked_Missing() when missing != null:
return missing();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int undo,  String message)  changed,required TResult Function()  unchanged,required TResult Function()  missing,}) {final _that = this;
switch (_that) {
case Ticked_Changed():
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged():
return unchanged();case Ticked_Missing():
return missing();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int undo,  String message)?  changed,TResult? Function()?  unchanged,TResult? Function()?  missing,}) {final _that = this;
switch (_that) {
case Ticked_Changed() when changed != null:
return changed(_that.rev,_that.undo,_that.message);case Ticked_Unchanged() when unchanged != null:
return unchanged();case Ticked_Missing() when missing != null:
return missing();case _:
  return null;

}
}

}

/// @nodoc


class Ticked_Changed extends Ticked {
  const Ticked_Changed({required this.rev, required this.undo, required this.message}): super._();
  

 final  String rev;
 final  int undo;
 final  String message;

/// Create a copy of Ticked
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$Ticked_ChangedCopyWith<Ticked_Changed> get copyWith => _$Ticked_ChangedCopyWithImpl<Ticked_Changed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Changed&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.undo, undo) || other.undo == undo)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,rev,undo,message);

@override
String toString() {
  return 'Ticked.changed(rev: $rev, undo: $undo, message: $message)';
}


}

/// @nodoc
abstract mixin class $Ticked_ChangedCopyWith<$Res> implements $TickedCopyWith<$Res> {
  factory $Ticked_ChangedCopyWith(Ticked_Changed value, $Res Function(Ticked_Changed) _then) = _$Ticked_ChangedCopyWithImpl;
@useResult
$Res call({
 String rev, int undo, String message
});




}
/// @nodoc
class _$Ticked_ChangedCopyWithImpl<$Res>
    implements $Ticked_ChangedCopyWith<$Res> {
  _$Ticked_ChangedCopyWithImpl(this._self, this._then);

  final Ticked_Changed _self;
  final $Res Function(Ticked_Changed) _then;

/// Create a copy of Ticked
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? undo = null,Object? message = null,}) {
  return _then(Ticked_Changed(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,undo: null == undo ? _self.undo : undo // ignore: cast_nullable_to_non_nullable
as int,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class Ticked_Unchanged extends Ticked {
  const Ticked_Unchanged(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Unchanged);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked.unchanged()';
}


}




/// @nodoc


class Ticked_Missing extends Ticked {
  const Ticked_Missing(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Ticked_Missing);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Ticked.missing()';
}


}




// dart format on
