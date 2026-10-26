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
mixin _$AccountState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountState()';
}


}

/// @nodoc
class $AccountStateCopyWith<$Res>  {
$AccountStateCopyWith(AccountState _, $Res Function(AccountState) __);
}


/// Adds pattern-matching-related methods to [AccountState].
extension AccountStatePatterns on AccountState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AccountState_SignedOut value)?  signedOut,TResult Function( AccountState_SignedIn value)?  signedIn,TResult Function( AccountState_Locked value)?  locked,TResult Function( AccountState_Question value)?  question,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut(_that);case AccountState_SignedIn() when signedIn != null:
return signedIn(_that);case AccountState_Locked() when locked != null:
return locked(_that);case AccountState_Question() when question != null:
return question(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AccountState_SignedOut value)  signedOut,required TResult Function( AccountState_SignedIn value)  signedIn,required TResult Function( AccountState_Locked value)  locked,required TResult Function( AccountState_Question value)  question,}){
final _that = this;
switch (_that) {
case AccountState_SignedOut():
return signedOut(_that);case AccountState_SignedIn():
return signedIn(_that);case AccountState_Locked():
return locked(_that);case AccountState_Question():
return question(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AccountState_SignedOut value)?  signedOut,TResult? Function( AccountState_SignedIn value)?  signedIn,TResult? Function( AccountState_Locked value)?  locked,TResult? Function( AccountState_Question value)?  question,}){
final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut(_that);case AccountState_SignedIn() when signedIn != null:
return signedIn(_that);case AccountState_Locked() when locked != null:
return locked(_that);case AccountState_Question() when question != null:
return question(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  signedOut,TResult Function( String username,  String server)?  signedIn,TResult Function( LockedInfo field0)?  locked,TResult Function( BindingQuestion field0)?  question,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut();case AccountState_SignedIn() when signedIn != null:
return signedIn(_that.username,_that.server);case AccountState_Locked() when locked != null:
return locked(_that.field0);case AccountState_Question() when question != null:
return question(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  signedOut,required TResult Function( String username,  String server)  signedIn,required TResult Function( LockedInfo field0)  locked,required TResult Function( BindingQuestion field0)  question,}) {final _that = this;
switch (_that) {
case AccountState_SignedOut():
return signedOut();case AccountState_SignedIn():
return signedIn(_that.username,_that.server);case AccountState_Locked():
return locked(_that.field0);case AccountState_Question():
return question(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  signedOut,TResult? Function( String username,  String server)?  signedIn,TResult? Function( LockedInfo field0)?  locked,TResult? Function( BindingQuestion field0)?  question,}) {final _that = this;
switch (_that) {
case AccountState_SignedOut() when signedOut != null:
return signedOut();case AccountState_SignedIn() when signedIn != null:
return signedIn(_that.username,_that.server);case AccountState_Locked() when locked != null:
return locked(_that.field0);case AccountState_Question() when question != null:
return question(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class AccountState_SignedOut extends AccountState {
  const AccountState_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AccountState.signedOut()';
}


}




/// @nodoc


class AccountState_SignedIn extends AccountState {
  const AccountState_SignedIn({required this.username, required this.server}): super._();
  

 final  String username;
 final  String server;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_SignedInCopyWith<AccountState_SignedIn> get copyWith => _$AccountState_SignedInCopyWithImpl<AccountState_SignedIn>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_SignedIn&&(identical(other.username, username) || other.username == username)&&(identical(other.server, server) || other.server == server));
}


@override
int get hashCode => Object.hash(runtimeType,username,server);

@override
String toString() {
  return 'AccountState.signedIn(username: $username, server: $server)';
}


}

/// @nodoc
abstract mixin class $AccountState_SignedInCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_SignedInCopyWith(AccountState_SignedIn value, $Res Function(AccountState_SignedIn) _then) = _$AccountState_SignedInCopyWithImpl;
@useResult
$Res call({
 String username, String server
});




}
/// @nodoc
class _$AccountState_SignedInCopyWithImpl<$Res>
    implements $AccountState_SignedInCopyWith<$Res> {
  _$AccountState_SignedInCopyWithImpl(this._self, this._then);

  final AccountState_SignedIn _self;
  final $Res Function(AccountState_SignedIn) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? username = null,Object? server = null,}) {
  return _then(AccountState_SignedIn(
username: null == username ? _self.username : username // ignore: cast_nullable_to_non_nullable
as String,server: null == server ? _self.server : server // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AccountState_Locked extends AccountState {
  const AccountState_Locked(this.field0): super._();
  

 final  LockedInfo field0;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_LockedCopyWith<AccountState_Locked> get copyWith => _$AccountState_LockedCopyWithImpl<AccountState_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_Locked&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountState.locked(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountState_LockedCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_LockedCopyWith(AccountState_Locked value, $Res Function(AccountState_Locked) _then) = _$AccountState_LockedCopyWithImpl;
@useResult
$Res call({
 LockedInfo field0
});




}
/// @nodoc
class _$AccountState_LockedCopyWithImpl<$Res>
    implements $AccountState_LockedCopyWith<$Res> {
  _$AccountState_LockedCopyWithImpl(this._self, this._then);

  final AccountState_Locked _self;
  final $Res Function(AccountState_Locked) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountState_Locked(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as LockedInfo,
  ));
}


}

/// @nodoc


class AccountState_Question extends AccountState {
  const AccountState_Question(this.field0): super._();
  

 final  BindingQuestion field0;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AccountState_QuestionCopyWith<AccountState_Question> get copyWith => _$AccountState_QuestionCopyWithImpl<AccountState_Question>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AccountState_Question&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AccountState.question(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AccountState_QuestionCopyWith<$Res> implements $AccountStateCopyWith<$Res> {
  factory $AccountState_QuestionCopyWith(AccountState_Question value, $Res Function(AccountState_Question) _then) = _$AccountState_QuestionCopyWithImpl;
@useResult
$Res call({
 BindingQuestion field0
});




}
/// @nodoc
class _$AccountState_QuestionCopyWithImpl<$Res>
    implements $AccountState_QuestionCopyWith<$Res> {
  _$AccountState_QuestionCopyWithImpl(this._self, this._then);

  final AccountState_Question _self;
  final $Res Function(AccountState_Question) _then;

/// Create a copy of AccountState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AccountState_Question(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as BindingQuestion,
  ));
}


}

/// @nodoc
mixin _$ApprovalPhase {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase()';
}


}

/// @nodoc
class $ApprovalPhaseCopyWith<$Res>  {
$ApprovalPhaseCopyWith(ApprovalPhase _, $Res Function(ApprovalPhase) __);
}


/// Adds pattern-matching-related methods to [ApprovalPhase].
extension ApprovalPhasePatterns on ApprovalPhase {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ApprovalPhase_Requesting value)?  requesting,TResult Function( ApprovalPhase_Waiting value)?  waiting,TResult Function( ApprovalPhase_Compare value)?  compare,TResult Function( ApprovalPhase_Unlocking value)?  unlocking,TResult Function( ApprovalPhase_Approved value)?  approved,TResult Function( ApprovalPhase_NoDevices value)?  noDevices,TResult Function( ApprovalPhase_Cancelled value)?  cancelled,TResult Function( ApprovalPhase_Expired value)?  expired,TResult Function( ApprovalPhase_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting(_that);case ApprovalPhase_Waiting() when waiting != null:
return waiting(_that);case ApprovalPhase_Compare() when compare != null:
return compare(_that);case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking(_that);case ApprovalPhase_Approved() when approved != null:
return approved(_that);case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices(_that);case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled(_that);case ApprovalPhase_Expired() when expired != null:
return expired(_that);case ApprovalPhase_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ApprovalPhase_Requesting value)  requesting,required TResult Function( ApprovalPhase_Waiting value)  waiting,required TResult Function( ApprovalPhase_Compare value)  compare,required TResult Function( ApprovalPhase_Unlocking value)  unlocking,required TResult Function( ApprovalPhase_Approved value)  approved,required TResult Function( ApprovalPhase_NoDevices value)  noDevices,required TResult Function( ApprovalPhase_Cancelled value)  cancelled,required TResult Function( ApprovalPhase_Expired value)  expired,required TResult Function( ApprovalPhase_Failed value)  failed,}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting():
return requesting(_that);case ApprovalPhase_Waiting():
return waiting(_that);case ApprovalPhase_Compare():
return compare(_that);case ApprovalPhase_Unlocking():
return unlocking(_that);case ApprovalPhase_Approved():
return approved(_that);case ApprovalPhase_NoDevices():
return noDevices(_that);case ApprovalPhase_Cancelled():
return cancelled(_that);case ApprovalPhase_Expired():
return expired(_that);case ApprovalPhase_Failed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ApprovalPhase_Requesting value)?  requesting,TResult? Function( ApprovalPhase_Waiting value)?  waiting,TResult? Function( ApprovalPhase_Compare value)?  compare,TResult? Function( ApprovalPhase_Unlocking value)?  unlocking,TResult? Function( ApprovalPhase_Approved value)?  approved,TResult? Function( ApprovalPhase_NoDevices value)?  noDevices,TResult? Function( ApprovalPhase_Cancelled value)?  cancelled,TResult? Function( ApprovalPhase_Expired value)?  expired,TResult? Function( ApprovalPhase_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting(_that);case ApprovalPhase_Waiting() when waiting != null:
return waiting(_that);case ApprovalPhase_Compare() when compare != null:
return compare(_that);case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking(_that);case ApprovalPhase_Approved() when approved != null:
return approved(_that);case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices(_that);case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled(_that);case ApprovalPhase_Expired() when expired != null:
return expired(_that);case ApprovalPhase_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  requesting,TResult Function()?  waiting,TResult Function()?  compare,TResult Function()?  unlocking,TResult Function()?  approved,TResult Function()?  noDevices,TResult Function()?  cancelled,TResult Function()?  expired,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting();case ApprovalPhase_Waiting() when waiting != null:
return waiting();case ApprovalPhase_Compare() when compare != null:
return compare();case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking();case ApprovalPhase_Approved() when approved != null:
return approved();case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices();case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled();case ApprovalPhase_Expired() when expired != null:
return expired();case ApprovalPhase_Failed() when failed != null:
return failed(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  requesting,required TResult Function()  waiting,required TResult Function()  compare,required TResult Function()  unlocking,required TResult Function()  approved,required TResult Function()  noDevices,required TResult Function()  cancelled,required TResult Function()  expired,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting():
return requesting();case ApprovalPhase_Waiting():
return waiting();case ApprovalPhase_Compare():
return compare();case ApprovalPhase_Unlocking():
return unlocking();case ApprovalPhase_Approved():
return approved();case ApprovalPhase_NoDevices():
return noDevices();case ApprovalPhase_Cancelled():
return cancelled();case ApprovalPhase_Expired():
return expired();case ApprovalPhase_Failed():
return failed(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  requesting,TResult? Function()?  waiting,TResult? Function()?  compare,TResult? Function()?  unlocking,TResult? Function()?  approved,TResult? Function()?  noDevices,TResult? Function()?  cancelled,TResult? Function()?  expired,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case ApprovalPhase_Requesting() when requesting != null:
return requesting();case ApprovalPhase_Waiting() when waiting != null:
return waiting();case ApprovalPhase_Compare() when compare != null:
return compare();case ApprovalPhase_Unlocking() when unlocking != null:
return unlocking();case ApprovalPhase_Approved() when approved != null:
return approved();case ApprovalPhase_NoDevices() when noDevices != null:
return noDevices();case ApprovalPhase_Cancelled() when cancelled != null:
return cancelled();case ApprovalPhase_Expired() when expired != null:
return expired();case ApprovalPhase_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class ApprovalPhase_Requesting extends ApprovalPhase {
  const ApprovalPhase_Requesting(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Requesting);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.requesting()';
}


}




/// @nodoc


class ApprovalPhase_Waiting extends ApprovalPhase {
  const ApprovalPhase_Waiting(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Waiting);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.waiting()';
}


}




/// @nodoc


class ApprovalPhase_Compare extends ApprovalPhase {
  const ApprovalPhase_Compare(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Compare);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.compare()';
}


}




/// @nodoc


class ApprovalPhase_Unlocking extends ApprovalPhase {
  const ApprovalPhase_Unlocking(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Unlocking);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.unlocking()';
}


}




/// @nodoc


class ApprovalPhase_Approved extends ApprovalPhase {
  const ApprovalPhase_Approved(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Approved);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.approved()';
}


}




/// @nodoc


class ApprovalPhase_NoDevices extends ApprovalPhase {
  const ApprovalPhase_NoDevices(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_NoDevices);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.noDevices()';
}


}




/// @nodoc


class ApprovalPhase_Cancelled extends ApprovalPhase {
  const ApprovalPhase_Cancelled(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Cancelled);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.cancelled()';
}


}




/// @nodoc


class ApprovalPhase_Expired extends ApprovalPhase {
  const ApprovalPhase_Expired(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Expired);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ApprovalPhase.expired()';
}


}




/// @nodoc


class ApprovalPhase_Failed extends ApprovalPhase {
  const ApprovalPhase_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of ApprovalPhase
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ApprovalPhase_FailedCopyWith<ApprovalPhase_Failed> get copyWith => _$ApprovalPhase_FailedCopyWithImpl<ApprovalPhase_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ApprovalPhase_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ApprovalPhase.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $ApprovalPhase_FailedCopyWith<$Res> implements $ApprovalPhaseCopyWith<$Res> {
  factory $ApprovalPhase_FailedCopyWith(ApprovalPhase_Failed value, $Res Function(ApprovalPhase_Failed) _then) = _$ApprovalPhase_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ApprovalPhase_FailedCopyWithImpl<$Res>
    implements $ApprovalPhase_FailedCopyWith<$Res> {
  _$ApprovalPhase_FailedCopyWithImpl(this._self, this._then);

  final ApprovalPhase_Failed _self;
  final $Res Function(ApprovalPhase_Failed) _then;

/// Create a copy of ApprovalPhase
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ApprovalPhase_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

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
mixin _$PersonLookup {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PersonLookup()';
}


}

/// @nodoc
class $PersonLookupCopyWith<$Res>  {
$PersonLookupCopyWith(PersonLookup _, $Res Function(PersonLookup) __);
}


/// Adds pattern-matching-related methods to [PersonLookup].
extension PersonLookupPatterns on PersonLookup {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PersonLookup_Found value)?  found,TResult Function( PersonLookup_Changed value)?  changed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that);case PersonLookup_Changed() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PersonLookup_Found value)  found,required TResult Function( PersonLookup_Changed value)  changed,}){
final _that = this;
switch (_that) {
case PersonLookup_Found():
return found(_that);case PersonLookup_Changed():
return changed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PersonLookup_Found value)?  found,TResult? Function( PersonLookup_Changed value)?  changed,}){
final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that);case PersonLookup_Changed() when changed != null:
return changed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PersonRow field0)?  found,TResult Function( PersonRow? pinned,  PersonRow offered)?  changed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that.field0);case PersonLookup_Changed() when changed != null:
return changed(_that.pinned,_that.offered);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PersonRow field0)  found,required TResult Function( PersonRow? pinned,  PersonRow offered)  changed,}) {final _that = this;
switch (_that) {
case PersonLookup_Found():
return found(_that.field0);case PersonLookup_Changed():
return changed(_that.pinned,_that.offered);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PersonRow field0)?  found,TResult? Function( PersonRow? pinned,  PersonRow offered)?  changed,}) {final _that = this;
switch (_that) {
case PersonLookup_Found() when found != null:
return found(_that.field0);case PersonLookup_Changed() when changed != null:
return changed(_that.pinned,_that.offered);case _:
  return null;

}
}

}

/// @nodoc


class PersonLookup_Found extends PersonLookup {
  const PersonLookup_Found(this.field0): super._();
  

 final  PersonRow field0;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PersonLookup_FoundCopyWith<PersonLookup_Found> get copyWith => _$PersonLookup_FoundCopyWithImpl<PersonLookup_Found>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup_Found&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'PersonLookup.found(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PersonLookup_FoundCopyWith<$Res> implements $PersonLookupCopyWith<$Res> {
  factory $PersonLookup_FoundCopyWith(PersonLookup_Found value, $Res Function(PersonLookup_Found) _then) = _$PersonLookup_FoundCopyWithImpl;
@useResult
$Res call({
 PersonRow field0
});




}
/// @nodoc
class _$PersonLookup_FoundCopyWithImpl<$Res>
    implements $PersonLookup_FoundCopyWith<$Res> {
  _$PersonLookup_FoundCopyWithImpl(this._self, this._then);

  final PersonLookup_Found _self;
  final $Res Function(PersonLookup_Found) _then;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PersonLookup_Found(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PersonRow,
  ));
}


}

/// @nodoc


class PersonLookup_Changed extends PersonLookup {
  const PersonLookup_Changed({this.pinned, required this.offered}): super._();
  

 final  PersonRow? pinned;
 final  PersonRow offered;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PersonLookup_ChangedCopyWith<PersonLookup_Changed> get copyWith => _$PersonLookup_ChangedCopyWithImpl<PersonLookup_Changed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PersonLookup_Changed&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.offered, offered) || other.offered == offered));
}


@override
int get hashCode => Object.hash(runtimeType,pinned,offered);

@override
String toString() {
  return 'PersonLookup.changed(pinned: $pinned, offered: $offered)';
}


}

/// @nodoc
abstract mixin class $PersonLookup_ChangedCopyWith<$Res> implements $PersonLookupCopyWith<$Res> {
  factory $PersonLookup_ChangedCopyWith(PersonLookup_Changed value, $Res Function(PersonLookup_Changed) _then) = _$PersonLookup_ChangedCopyWithImpl;
@useResult
$Res call({
 PersonRow? pinned, PersonRow offered
});




}
/// @nodoc
class _$PersonLookup_ChangedCopyWithImpl<$Res>
    implements $PersonLookup_ChangedCopyWith<$Res> {
  _$PersonLookup_ChangedCopyWithImpl(this._self, this._then);

  final PersonLookup_Changed _self;
  final $Res Function(PersonLookup_Changed) _then;

/// Create a copy of PersonLookup
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? pinned = freezed,Object? offered = null,}) {
  return _then(PersonLookup_Changed(
pinned: freezed == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as PersonRow?,offered: null == offered ? _self.offered : offered // ignore: cast_nullable_to_non_nullable
as PersonRow,
  ));
}


}

/// @nodoc
mixin _$ProviderEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent()';
}


}

/// @nodoc
class $ProviderEventCopyWith<$Res>  {
$ProviderEventCopyWith(ProviderEvent _, $Res Function(ProviderEvent) __);
}


/// Adds pattern-matching-related methods to [ProviderEvent].
extension ProviderEventPatterns on ProviderEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ProviderEvent_OpenPage value)?  openPage,TResult Function( ProviderEvent_ShowCode value)?  showCode,TResult Function( ProviderEvent_ClosePage value)?  closePage,TResult Function( ProviderEvent_NeedsUsername value)?  needsUsername,TResult Function( ProviderEvent_Finished value)?  finished,TResult Function( ProviderEvent_Failed value)?  failed,TResult Function( ProviderEvent_Cancelled value)?  cancelled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that);case ProviderEvent_ClosePage() when closePage != null:
return closePage(_that);case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that);case ProviderEvent_Finished() when finished != null:
return finished(_that);case ProviderEvent_Failed() when failed != null:
return failed(_that);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ProviderEvent_OpenPage value)  openPage,required TResult Function( ProviderEvent_ShowCode value)  showCode,required TResult Function( ProviderEvent_ClosePage value)  closePage,required TResult Function( ProviderEvent_NeedsUsername value)  needsUsername,required TResult Function( ProviderEvent_Finished value)  finished,required TResult Function( ProviderEvent_Failed value)  failed,required TResult Function( ProviderEvent_Cancelled value)  cancelled,}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage():
return openPage(_that);case ProviderEvent_ShowCode():
return showCode(_that);case ProviderEvent_ClosePage():
return closePage(_that);case ProviderEvent_NeedsUsername():
return needsUsername(_that);case ProviderEvent_Finished():
return finished(_that);case ProviderEvent_Failed():
return failed(_that);case ProviderEvent_Cancelled():
return cancelled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ProviderEvent_OpenPage value)?  openPage,TResult? Function( ProviderEvent_ShowCode value)?  showCode,TResult? Function( ProviderEvent_ClosePage value)?  closePage,TResult? Function( ProviderEvent_NeedsUsername value)?  needsUsername,TResult? Function( ProviderEvent_Finished value)?  finished,TResult? Function( ProviderEvent_Failed value)?  failed,TResult? Function( ProviderEvent_Cancelled value)?  cancelled,}){
final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that);case ProviderEvent_ClosePage() when closePage != null:
return closePage(_that);case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that);case ProviderEvent_Finished() when finished != null:
return finished(_that);case ProviderEvent_Failed() when failed != null:
return failed(_that);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String url,  String? callbackScheme)?  openPage,TResult Function( String userCode,  String verificationUri)?  showCode,TResult Function()?  closePage,TResult Function( String suggested)?  needsUsername,TResult Function( AccountState field0)?  finished,TResult Function( String message)?  failed,TResult Function()?  cancelled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage() when closePage != null:
return closePage();case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that.suggested);case ProviderEvent_Finished() when finished != null:
return finished(_that.field0);case ProviderEvent_Failed() when failed != null:
return failed(_that.message);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String url,  String? callbackScheme)  openPage,required TResult Function( String userCode,  String verificationUri)  showCode,required TResult Function()  closePage,required TResult Function( String suggested)  needsUsername,required TResult Function( AccountState field0)  finished,required TResult Function( String message)  failed,required TResult Function()  cancelled,}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage():
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode():
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage():
return closePage();case ProviderEvent_NeedsUsername():
return needsUsername(_that.suggested);case ProviderEvent_Finished():
return finished(_that.field0);case ProviderEvent_Failed():
return failed(_that.message);case ProviderEvent_Cancelled():
return cancelled();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String url,  String? callbackScheme)?  openPage,TResult? Function( String userCode,  String verificationUri)?  showCode,TResult? Function()?  closePage,TResult? Function( String suggested)?  needsUsername,TResult? Function( AccountState field0)?  finished,TResult? Function( String message)?  failed,TResult? Function()?  cancelled,}) {final _that = this;
switch (_that) {
case ProviderEvent_OpenPage() when openPage != null:
return openPage(_that.url,_that.callbackScheme);case ProviderEvent_ShowCode() when showCode != null:
return showCode(_that.userCode,_that.verificationUri);case ProviderEvent_ClosePage() when closePage != null:
return closePage();case ProviderEvent_NeedsUsername() when needsUsername != null:
return needsUsername(_that.suggested);case ProviderEvent_Finished() when finished != null:
return finished(_that.field0);case ProviderEvent_Failed() when failed != null:
return failed(_that.message);case ProviderEvent_Cancelled() when cancelled != null:
return cancelled();case _:
  return null;

}
}

}

/// @nodoc


class ProviderEvent_OpenPage extends ProviderEvent {
  const ProviderEvent_OpenPage({required this.url, this.callbackScheme}): super._();
  

 final  String url;
 final  String? callbackScheme;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_OpenPageCopyWith<ProviderEvent_OpenPage> get copyWith => _$ProviderEvent_OpenPageCopyWithImpl<ProviderEvent_OpenPage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_OpenPage&&(identical(other.url, url) || other.url == url)&&(identical(other.callbackScheme, callbackScheme) || other.callbackScheme == callbackScheme));
}


@override
int get hashCode => Object.hash(runtimeType,url,callbackScheme);

@override
String toString() {
  return 'ProviderEvent.openPage(url: $url, callbackScheme: $callbackScheme)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_OpenPageCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_OpenPageCopyWith(ProviderEvent_OpenPage value, $Res Function(ProviderEvent_OpenPage) _then) = _$ProviderEvent_OpenPageCopyWithImpl;
@useResult
$Res call({
 String url, String? callbackScheme
});




}
/// @nodoc
class _$ProviderEvent_OpenPageCopyWithImpl<$Res>
    implements $ProviderEvent_OpenPageCopyWith<$Res> {
  _$ProviderEvent_OpenPageCopyWithImpl(this._self, this._then);

  final ProviderEvent_OpenPage _self;
  final $Res Function(ProviderEvent_OpenPage) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,Object? callbackScheme = freezed,}) {
  return _then(ProviderEvent_OpenPage(
url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,callbackScheme: freezed == callbackScheme ? _self.callbackScheme : callbackScheme // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class ProviderEvent_ShowCode extends ProviderEvent {
  const ProviderEvent_ShowCode({required this.userCode, required this.verificationUri}): super._();
  

 final  String userCode;
 final  String verificationUri;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_ShowCodeCopyWith<ProviderEvent_ShowCode> get copyWith => _$ProviderEvent_ShowCodeCopyWithImpl<ProviderEvent_ShowCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_ShowCode&&(identical(other.userCode, userCode) || other.userCode == userCode)&&(identical(other.verificationUri, verificationUri) || other.verificationUri == verificationUri));
}


@override
int get hashCode => Object.hash(runtimeType,userCode,verificationUri);

@override
String toString() {
  return 'ProviderEvent.showCode(userCode: $userCode, verificationUri: $verificationUri)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_ShowCodeCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_ShowCodeCopyWith(ProviderEvent_ShowCode value, $Res Function(ProviderEvent_ShowCode) _then) = _$ProviderEvent_ShowCodeCopyWithImpl;
@useResult
$Res call({
 String userCode, String verificationUri
});




}
/// @nodoc
class _$ProviderEvent_ShowCodeCopyWithImpl<$Res>
    implements $ProviderEvent_ShowCodeCopyWith<$Res> {
  _$ProviderEvent_ShowCodeCopyWithImpl(this._self, this._then);

  final ProviderEvent_ShowCode _self;
  final $Res Function(ProviderEvent_ShowCode) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? userCode = null,Object? verificationUri = null,}) {
  return _then(ProviderEvent_ShowCode(
userCode: null == userCode ? _self.userCode : userCode // ignore: cast_nullable_to_non_nullable
as String,verificationUri: null == verificationUri ? _self.verificationUri : verificationUri // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_ClosePage extends ProviderEvent {
  const ProviderEvent_ClosePage(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_ClosePage);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent.closePage()';
}


}




/// @nodoc


class ProviderEvent_NeedsUsername extends ProviderEvent {
  const ProviderEvent_NeedsUsername({required this.suggested}): super._();
  

 final  String suggested;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_NeedsUsernameCopyWith<ProviderEvent_NeedsUsername> get copyWith => _$ProviderEvent_NeedsUsernameCopyWithImpl<ProviderEvent_NeedsUsername>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_NeedsUsername&&(identical(other.suggested, suggested) || other.suggested == suggested));
}


@override
int get hashCode => Object.hash(runtimeType,suggested);

@override
String toString() {
  return 'ProviderEvent.needsUsername(suggested: $suggested)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_NeedsUsernameCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_NeedsUsernameCopyWith(ProviderEvent_NeedsUsername value, $Res Function(ProviderEvent_NeedsUsername) _then) = _$ProviderEvent_NeedsUsernameCopyWithImpl;
@useResult
$Res call({
 String suggested
});




}
/// @nodoc
class _$ProviderEvent_NeedsUsernameCopyWithImpl<$Res>
    implements $ProviderEvent_NeedsUsernameCopyWith<$Res> {
  _$ProviderEvent_NeedsUsernameCopyWithImpl(this._self, this._then);

  final ProviderEvent_NeedsUsername _self;
  final $Res Function(ProviderEvent_NeedsUsername) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? suggested = null,}) {
  return _then(ProviderEvent_NeedsUsername(
suggested: null == suggested ? _self.suggested : suggested // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_Finished extends ProviderEvent {
  const ProviderEvent_Finished(this.field0): super._();
  

 final  AccountState field0;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_FinishedCopyWith<ProviderEvent_Finished> get copyWith => _$ProviderEvent_FinishedCopyWithImpl<ProviderEvent_Finished>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Finished&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ProviderEvent.finished(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_FinishedCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_FinishedCopyWith(ProviderEvent_Finished value, $Res Function(ProviderEvent_Finished) _then) = _$ProviderEvent_FinishedCopyWithImpl;
@useResult
$Res call({
 AccountState field0
});


$AccountStateCopyWith<$Res> get field0;

}
/// @nodoc
class _$ProviderEvent_FinishedCopyWithImpl<$Res>
    implements $ProviderEvent_FinishedCopyWith<$Res> {
  _$ProviderEvent_FinishedCopyWithImpl(this._self, this._then);

  final ProviderEvent_Finished _self;
  final $Res Function(ProviderEvent_Finished) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ProviderEvent_Finished(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as AccountState,
  ));
}

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AccountStateCopyWith<$Res> get field0 {
  
  return $AccountStateCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class ProviderEvent_Failed extends ProviderEvent {
  const ProviderEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ProviderEvent_FailedCopyWith<ProviderEvent_Failed> get copyWith => _$ProviderEvent_FailedCopyWithImpl<ProviderEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'ProviderEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $ProviderEvent_FailedCopyWith<$Res> implements $ProviderEventCopyWith<$Res> {
  factory $ProviderEvent_FailedCopyWith(ProviderEvent_Failed value, $Res Function(ProviderEvent_Failed) _then) = _$ProviderEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ProviderEvent_FailedCopyWithImpl<$Res>
    implements $ProviderEvent_FailedCopyWith<$Res> {
  _$ProviderEvent_FailedCopyWithImpl(this._self, this._then);

  final ProviderEvent_Failed _self;
  final $Res Function(ProviderEvent_Failed) _then;

/// Create a copy of ProviderEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ProviderEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ProviderEvent_Cancelled extends ProviderEvent {
  const ProviderEvent_Cancelled(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ProviderEvent_Cancelled);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ProviderEvent.cancelled()';
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

// dart format on
