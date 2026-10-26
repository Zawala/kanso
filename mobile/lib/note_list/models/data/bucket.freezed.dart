// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'bucket.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$Bucket {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is Bucket);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Bucket()';
}


}

/// @nodoc
class $BucketCopyWith<$Res>  {
$BucketCopyWith(Bucket _, $Res Function(Bucket) __);
}


/// Adds pattern-matching-related methods to [Bucket].
extension BucketPatterns on Bucket {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PinnedBucket value)?  pinned,TResult Function( DayBucket value)?  day,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PinnedBucket() when pinned != null:
return pinned(_that);case DayBucket() when day != null:
return day(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PinnedBucket value)  pinned,required TResult Function( DayBucket value)  day,}){
final _that = this;
switch (_that) {
case PinnedBucket():
return pinned(_that);case DayBucket():
return day(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PinnedBucket value)?  pinned,TResult? Function( DayBucket value)?  day,}){
final _that = this;
switch (_that) {
case PinnedBucket() when pinned != null:
return pinned(_that);case DayBucket() when day != null:
return day(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  pinned,TResult Function( int localDay)?  day,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PinnedBucket() when pinned != null:
return pinned();case DayBucket() when day != null:
return day(_that.localDay);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  pinned,required TResult Function( int localDay)  day,}) {final _that = this;
switch (_that) {
case PinnedBucket():
return pinned();case DayBucket():
return day(_that.localDay);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  pinned,TResult? Function( int localDay)?  day,}) {final _that = this;
switch (_that) {
case PinnedBucket() when pinned != null:
return pinned();case DayBucket() when day != null:
return day(_that.localDay);case _:
  return null;

}
}

}

/// @nodoc


class PinnedBucket implements Bucket {
  const PinnedBucket();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PinnedBucket);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'Bucket.pinned()';
}


}




/// @nodoc


class DayBucket implements Bucket {
  const DayBucket(this.localDay);
  

 final  int localDay;

/// Create a copy of Bucket
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DayBucketCopyWith<DayBucket> get copyWith => _$DayBucketCopyWithImpl<DayBucket>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DayBucket&&(identical(other.localDay, localDay) || other.localDay == localDay));
}


@override
int get hashCode => Object.hash(runtimeType,localDay);

@override
String toString() {
  return 'Bucket.day(localDay: $localDay)';
}


}

/// @nodoc
abstract mixin class $DayBucketCopyWith<$Res> implements $BucketCopyWith<$Res> {
  factory $DayBucketCopyWith(DayBucket value, $Res Function(DayBucket) _then) = _$DayBucketCopyWithImpl;
@useResult
$Res call({
 int localDay
});




}
/// @nodoc
class _$DayBucketCopyWithImpl<$Res>
    implements $DayBucketCopyWith<$Res> {
  _$DayBucketCopyWithImpl(this._self, this._then);

  final DayBucket _self;
  final $Res Function(DayBucket) _then;

/// Create a copy of Bucket
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? localDay = null,}) {
  return _then(DayBucket(
null == localDay ? _self.localDay : localDay // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
