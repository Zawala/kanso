// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'licences_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LicencesEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LicencesEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LicencesEvent()';
}


}

/// @nodoc
class $LicencesEventCopyWith<$Res>  {
$LicencesEventCopyWith(LicencesEvent _, $Res Function(LicencesEvent) __);
}


/// Adds pattern-matching-related methods to [LicencesEvent].
extension LicencesEventPatterns on LicencesEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LicencesStartedEvent value)?  started,TResult Function( LicencesPackageToggledEvent value)?  packageToggled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LicencesStartedEvent() when started != null:
return started(_that);case LicencesPackageToggledEvent() when packageToggled != null:
return packageToggled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LicencesStartedEvent value)  started,required TResult Function( LicencesPackageToggledEvent value)  packageToggled,}){
final _that = this;
switch (_that) {
case LicencesStartedEvent():
return started(_that);case LicencesPackageToggledEvent():
return packageToggled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LicencesStartedEvent value)?  started,TResult? Function( LicencesPackageToggledEvent value)?  packageToggled,}){
final _that = this;
switch (_that) {
case LicencesStartedEvent() when started != null:
return started(_that);case LicencesPackageToggledEvent() when packageToggled != null:
return packageToggled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String name)?  packageToggled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LicencesStartedEvent() when started != null:
return started();case LicencesPackageToggledEvent() when packageToggled != null:
return packageToggled(_that.name);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String name)  packageToggled,}) {final _that = this;
switch (_that) {
case LicencesStartedEvent():
return started();case LicencesPackageToggledEvent():
return packageToggled(_that.name);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String name)?  packageToggled,}) {final _that = this;
switch (_that) {
case LicencesStartedEvent() when started != null:
return started();case LicencesPackageToggledEvent() when packageToggled != null:
return packageToggled(_that.name);case _:
  return null;

}
}

}

/// @nodoc


class LicencesStartedEvent implements LicencesEvent {
  const LicencesStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LicencesStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LicencesEvent.started()';
}


}




/// @nodoc


class LicencesPackageToggledEvent implements LicencesEvent {
  const LicencesPackageToggledEvent(this.name);
  

 final  String name;

/// Create a copy of LicencesEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LicencesPackageToggledEventCopyWith<LicencesPackageToggledEvent> get copyWith => _$LicencesPackageToggledEventCopyWithImpl<LicencesPackageToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LicencesPackageToggledEvent&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'LicencesEvent.packageToggled(name: $name)';
}


}

/// @nodoc
abstract mixin class $LicencesPackageToggledEventCopyWith<$Res> implements $LicencesEventCopyWith<$Res> {
  factory $LicencesPackageToggledEventCopyWith(LicencesPackageToggledEvent value, $Res Function(LicencesPackageToggledEvent) _then) = _$LicencesPackageToggledEventCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$LicencesPackageToggledEventCopyWithImpl<$Res>
    implements $LicencesPackageToggledEventCopyWith<$Res> {
  _$LicencesPackageToggledEventCopyWithImpl(this._self, this._then);

  final LicencesPackageToggledEvent _self;
  final $Res Function(LicencesPackageToggledEvent) _then;

/// Create a copy of LicencesEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(LicencesPackageToggledEvent(
null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$LicencesState {

 LoadStatus get status; List<LicencePackage> get packages;/// The package whose licences are shown; null when none are.
 String? get expanded;
/// Create a copy of LicencesState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LicencesStateCopyWith<LicencesState> get copyWith => _$LicencesStateCopyWithImpl<LicencesState>(this as LicencesState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LicencesState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.packages, packages)&&(identical(other.expanded, expanded) || other.expanded == expanded));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(packages),expanded);

@override
String toString() {
  return 'LicencesState(status: $status, packages: $packages, expanded: $expanded)';
}


}

/// @nodoc
abstract mixin class $LicencesStateCopyWith<$Res>  {
  factory $LicencesStateCopyWith(LicencesState value, $Res Function(LicencesState) _then) = _$LicencesStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<LicencePackage> packages, String? expanded
});




}
/// @nodoc
class _$LicencesStateCopyWithImpl<$Res>
    implements $LicencesStateCopyWith<$Res> {
  _$LicencesStateCopyWithImpl(this._self, this._then);

  final LicencesState _self;
  final $Res Function(LicencesState) _then;

/// Create a copy of LicencesState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? packages = null,Object? expanded = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,packages: null == packages ? _self.packages : packages // ignore: cast_nullable_to_non_nullable
as List<LicencePackage>,expanded: freezed == expanded ? _self.expanded : expanded // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [LicencesState].
extension LicencesStatePatterns on LicencesState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LicencesState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LicencesState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LicencesState value)  $default,){
final _that = this;
switch (_that) {
case _LicencesState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LicencesState value)?  $default,){
final _that = this;
switch (_that) {
case _LicencesState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<LicencePackage> packages,  String? expanded)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LicencesState() when $default != null:
return $default(_that.status,_that.packages,_that.expanded);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<LicencePackage> packages,  String? expanded)  $default,) {final _that = this;
switch (_that) {
case _LicencesState():
return $default(_that.status,_that.packages,_that.expanded);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<LicencePackage> packages,  String? expanded)?  $default,) {final _that = this;
switch (_that) {
case _LicencesState() when $default != null:
return $default(_that.status,_that.packages,_that.expanded);case _:
  return null;

}
}

}

/// @nodoc


class _LicencesState implements LicencesState {
  const _LicencesState({this.status = LoadStatus.loading, final  List<LicencePackage> packages = const <LicencePackage>[], this.expanded}): _packages = packages;
  

@override@JsonKey() final  LoadStatus status;
 final  List<LicencePackage> _packages;
@override@JsonKey() List<LicencePackage> get packages {
  if (_packages is EqualUnmodifiableListView) return _packages;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_packages);
}

/// The package whose licences are shown; null when none are.
@override final  String? expanded;

/// Create a copy of LicencesState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LicencesStateCopyWith<_LicencesState> get copyWith => __$LicencesStateCopyWithImpl<_LicencesState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LicencesState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._packages, _packages)&&(identical(other.expanded, expanded) || other.expanded == expanded));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_packages),expanded);

@override
String toString() {
  return 'LicencesState(status: $status, packages: $packages, expanded: $expanded)';
}


}

/// @nodoc
abstract mixin class _$LicencesStateCopyWith<$Res> implements $LicencesStateCopyWith<$Res> {
  factory _$LicencesStateCopyWith(_LicencesState value, $Res Function(_LicencesState) _then) = __$LicencesStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<LicencePackage> packages, String? expanded
});




}
/// @nodoc
class __$LicencesStateCopyWithImpl<$Res>
    implements _$LicencesStateCopyWith<$Res> {
  __$LicencesStateCopyWithImpl(this._self, this._then);

  final _LicencesState _self;
  final $Res Function(_LicencesState) _then;

/// Create a copy of LicencesState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? packages = null,Object? expanded = freezed,}) {
  return _then(_LicencesState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,packages: null == packages ? _self._packages : packages // ignore: cast_nullable_to_non_nullable
as List<LicencePackage>,expanded: freezed == expanded ? _self.expanded : expanded // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on
