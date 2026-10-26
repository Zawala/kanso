// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'kanso_syntax_style.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KansoSyntaxStyle {

/// Whether the token is italic; null leaves it as the text is.
 bool? get italic;/// The weight, 100 to 900; null leaves it as the text is.
 int? get weight;
/// Create a copy of KansoSyntaxStyle
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoSyntaxStyleCopyWith<KansoSyntaxStyle> get copyWith => _$KansoSyntaxStyleCopyWithImpl<KansoSyntaxStyle>(this as KansoSyntaxStyle, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoSyntaxStyle&&(identical(other.italic, italic) || other.italic == italic)&&(identical(other.weight, weight) || other.weight == weight));
}


@override
int get hashCode => Object.hash(runtimeType,italic,weight);

@override
String toString() {
  return 'KansoSyntaxStyle(italic: $italic, weight: $weight)';
}


}

/// @nodoc
abstract mixin class $KansoSyntaxStyleCopyWith<$Res>  {
  factory $KansoSyntaxStyleCopyWith(KansoSyntaxStyle value, $Res Function(KansoSyntaxStyle) _then) = _$KansoSyntaxStyleCopyWithImpl;
@useResult
$Res call({
 bool? italic, int? weight
});




}
/// @nodoc
class _$KansoSyntaxStyleCopyWithImpl<$Res>
    implements $KansoSyntaxStyleCopyWith<$Res> {
  _$KansoSyntaxStyleCopyWithImpl(this._self, this._then);

  final KansoSyntaxStyle _self;
  final $Res Function(KansoSyntaxStyle) _then;

/// Create a copy of KansoSyntaxStyle
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? italic = freezed,Object? weight = freezed,}) {
  return _then(_self.copyWith(
italic: freezed == italic ? _self.italic : italic // ignore: cast_nullable_to_non_nullable
as bool?,weight: freezed == weight ? _self.weight : weight // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}

}


/// Adds pattern-matching-related methods to [KansoSyntaxStyle].
extension KansoSyntaxStylePatterns on KansoSyntaxStyle {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KansoSyntaxStyle value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KansoSyntaxStyle() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KansoSyntaxStyle value)  $default,){
final _that = this;
switch (_that) {
case _KansoSyntaxStyle():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KansoSyntaxStyle value)?  $default,){
final _that = this;
switch (_that) {
case _KansoSyntaxStyle() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool? italic,  int? weight)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoSyntaxStyle() when $default != null:
return $default(_that.italic,_that.weight);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool? italic,  int? weight)  $default,) {final _that = this;
switch (_that) {
case _KansoSyntaxStyle():
return $default(_that.italic,_that.weight);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool? italic,  int? weight)?  $default,) {final _that = this;
switch (_that) {
case _KansoSyntaxStyle() when $default != null:
return $default(_that.italic,_that.weight);case _:
  return null;

}
}

}

/// @nodoc


class _KansoSyntaxStyle implements KansoSyntaxStyle {
  const _KansoSyntaxStyle({this.italic, this.weight});
  

/// Whether the token is italic; null leaves it as the text is.
@override final  bool? italic;
/// The weight, 100 to 900; null leaves it as the text is.
@override final  int? weight;

/// Create a copy of KansoSyntaxStyle
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KansoSyntaxStyleCopyWith<_KansoSyntaxStyle> get copyWith => __$KansoSyntaxStyleCopyWithImpl<_KansoSyntaxStyle>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoSyntaxStyle&&(identical(other.italic, italic) || other.italic == italic)&&(identical(other.weight, weight) || other.weight == weight));
}


@override
int get hashCode => Object.hash(runtimeType,italic,weight);

@override
String toString() {
  return 'KansoSyntaxStyle(italic: $italic, weight: $weight)';
}


}

/// @nodoc
abstract mixin class _$KansoSyntaxStyleCopyWith<$Res> implements $KansoSyntaxStyleCopyWith<$Res> {
  factory _$KansoSyntaxStyleCopyWith(_KansoSyntaxStyle value, $Res Function(_KansoSyntaxStyle) _then) = __$KansoSyntaxStyleCopyWithImpl;
@override @useResult
$Res call({
 bool? italic, int? weight
});




}
/// @nodoc
class __$KansoSyntaxStyleCopyWithImpl<$Res>
    implements _$KansoSyntaxStyleCopyWith<$Res> {
  __$KansoSyntaxStyleCopyWithImpl(this._self, this._then);

  final _KansoSyntaxStyle _self;
  final $Res Function(_KansoSyntaxStyle) _then;

/// Create a copy of KansoSyntaxStyle
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? italic = freezed,Object? weight = freezed,}) {
  return _then(_KansoSyntaxStyle(
italic: freezed == italic ? _self.italic : italic // ignore: cast_nullable_to_non_nullable
as bool?,weight: freezed == weight ? _self.weight : weight // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}

// dart format on
