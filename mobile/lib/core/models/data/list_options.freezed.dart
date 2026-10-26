// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'list_options.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ListOptions {

/// What the list is sorted by.
 SortKey get sort;/// Whether it runs from the largest value to the smallest: the newest
/// first, Z to A.
 bool get descending;/// The one status to list, as the library writes it (`onHold`); null
/// lists any.
 String? get statusOnly;/// Whether to list pinned notes only.
 bool get pinnedOnly;
/// Create a copy of ListOptions
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListOptionsCopyWith<ListOptions> get copyWith => _$ListOptionsCopyWithImpl<ListOptions>(this as ListOptions, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListOptions&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.statusOnly, statusOnly) || other.statusOnly == statusOnly)&&(identical(other.pinnedOnly, pinnedOnly) || other.pinnedOnly == pinnedOnly));
}


@override
int get hashCode => Object.hash(runtimeType,sort,descending,statusOnly,pinnedOnly);

@override
String toString() {
  return 'ListOptions(sort: $sort, descending: $descending, statusOnly: $statusOnly, pinnedOnly: $pinnedOnly)';
}


}

/// @nodoc
abstract mixin class $ListOptionsCopyWith<$Res>  {
  factory $ListOptionsCopyWith(ListOptions value, $Res Function(ListOptions) _then) = _$ListOptionsCopyWithImpl;
@useResult
$Res call({
 SortKey sort, bool descending, String? statusOnly, bool pinnedOnly
});




}
/// @nodoc
class _$ListOptionsCopyWithImpl<$Res>
    implements $ListOptionsCopyWith<$Res> {
  _$ListOptionsCopyWithImpl(this._self, this._then);

  final ListOptions _self;
  final $Res Function(ListOptions) _then;

/// Create a copy of ListOptions
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? sort = null,Object? descending = null,Object? statusOnly = freezed,Object? pinnedOnly = null,}) {
  return _then(_self.copyWith(
sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,statusOnly: freezed == statusOnly ? _self.statusOnly : statusOnly // ignore: cast_nullable_to_non_nullable
as String?,pinnedOnly: null == pinnedOnly ? _self.pinnedOnly : pinnedOnly // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [ListOptions].
extension ListOptionsPatterns on ListOptions {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ListOptions value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ListOptions() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ListOptions value)  $default,){
final _that = this;
switch (_that) {
case _ListOptions():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ListOptions value)?  $default,){
final _that = this;
switch (_that) {
case _ListOptions() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( SortKey sort,  bool descending,  String? statusOnly,  bool pinnedOnly)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ListOptions() when $default != null:
return $default(_that.sort,_that.descending,_that.statusOnly,_that.pinnedOnly);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( SortKey sort,  bool descending,  String? statusOnly,  bool pinnedOnly)  $default,) {final _that = this;
switch (_that) {
case _ListOptions():
return $default(_that.sort,_that.descending,_that.statusOnly,_that.pinnedOnly);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( SortKey sort,  bool descending,  String? statusOnly,  bool pinnedOnly)?  $default,) {final _that = this;
switch (_that) {
case _ListOptions() when $default != null:
return $default(_that.sort,_that.descending,_that.statusOnly,_that.pinnedOnly);case _:
  return null;

}
}

}

/// @nodoc


class _ListOptions extends ListOptions {
  const _ListOptions({this.sort = SortKey.updated, this.descending = true, this.statusOnly, this.pinnedOnly = false}): super._();
  

/// What the list is sorted by.
@override@JsonKey() final  SortKey sort;
/// Whether it runs from the largest value to the smallest: the newest
/// first, Z to A.
@override@JsonKey() final  bool descending;
/// The one status to list, as the library writes it (`onHold`); null
/// lists any.
@override final  String? statusOnly;
/// Whether to list pinned notes only.
@override@JsonKey() final  bool pinnedOnly;

/// Create a copy of ListOptions
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ListOptionsCopyWith<_ListOptions> get copyWith => __$ListOptionsCopyWithImpl<_ListOptions>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ListOptions&&(identical(other.sort, sort) || other.sort == sort)&&(identical(other.descending, descending) || other.descending == descending)&&(identical(other.statusOnly, statusOnly) || other.statusOnly == statusOnly)&&(identical(other.pinnedOnly, pinnedOnly) || other.pinnedOnly == pinnedOnly));
}


@override
int get hashCode => Object.hash(runtimeType,sort,descending,statusOnly,pinnedOnly);

@override
String toString() {
  return 'ListOptions(sort: $sort, descending: $descending, statusOnly: $statusOnly, pinnedOnly: $pinnedOnly)';
}


}

/// @nodoc
abstract mixin class _$ListOptionsCopyWith<$Res> implements $ListOptionsCopyWith<$Res> {
  factory _$ListOptionsCopyWith(_ListOptions value, $Res Function(_ListOptions) _then) = __$ListOptionsCopyWithImpl;
@override @useResult
$Res call({
 SortKey sort, bool descending, String? statusOnly, bool pinnedOnly
});




}
/// @nodoc
class __$ListOptionsCopyWithImpl<$Res>
    implements _$ListOptionsCopyWith<$Res> {
  __$ListOptionsCopyWithImpl(this._self, this._then);

  final _ListOptions _self;
  final $Res Function(_ListOptions) _then;

/// Create a copy of ListOptions
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? sort = null,Object? descending = null,Object? statusOnly = freezed,Object? pinnedOnly = null,}) {
  return _then(_ListOptions(
sort: null == sort ? _self.sort : sort // ignore: cast_nullable_to_non_nullable
as SortKey,descending: null == descending ? _self.descending : descending // ignore: cast_nullable_to_non_nullable
as bool,statusOnly: freezed == statusOnly ? _self.statusOnly : statusOnly // ignore: cast_nullable_to_non_nullable
as String?,pinnedOnly: null == pinnedOnly ? _self.pinnedOnly : pinnedOnly // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
