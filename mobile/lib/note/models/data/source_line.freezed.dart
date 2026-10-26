// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'source_line.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SourceLine {

/// Where the line starts in the note's text, in UTF-16 units.
 int get start;/// Where it ends: before the line break.
 int get end;/// The runs on this line, in order and not overlapping, each cut to the
/// line; text between them is plain.
 List<StyledRun> get runs;
/// Create a copy of SourceLine
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SourceLineCopyWith<SourceLine> get copyWith => _$SourceLineCopyWithImpl<SourceLine>(this as SourceLine, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SourceLine&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&const DeepCollectionEquality().equals(other.runs, runs));
}


@override
int get hashCode => Object.hash(runtimeType,start,end,const DeepCollectionEquality().hash(runs));

@override
String toString() {
  return 'SourceLine(start: $start, end: $end, runs: $runs)';
}


}

/// @nodoc
abstract mixin class $SourceLineCopyWith<$Res>  {
  factory $SourceLineCopyWith(SourceLine value, $Res Function(SourceLine) _then) = _$SourceLineCopyWithImpl;
@useResult
$Res call({
 int start, int end, List<StyledRun> runs
});




}
/// @nodoc
class _$SourceLineCopyWithImpl<$Res>
    implements $SourceLineCopyWith<$Res> {
  _$SourceLineCopyWithImpl(this._self, this._then);

  final SourceLine _self;
  final $Res Function(SourceLine) _then;

/// Create a copy of SourceLine
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? start = null,Object? end = null,Object? runs = null,}) {
  return _then(_self.copyWith(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,runs: null == runs ? _self.runs : runs // ignore: cast_nullable_to_non_nullable
as List<StyledRun>,
  ));
}

}


/// Adds pattern-matching-related methods to [SourceLine].
extension SourceLinePatterns on SourceLine {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SourceLine value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SourceLine() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SourceLine value)  $default,){
final _that = this;
switch (_that) {
case _SourceLine():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SourceLine value)?  $default,){
final _that = this;
switch (_that) {
case _SourceLine() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int start,  int end,  List<StyledRun> runs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SourceLine() when $default != null:
return $default(_that.start,_that.end,_that.runs);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int start,  int end,  List<StyledRun> runs)  $default,) {final _that = this;
switch (_that) {
case _SourceLine():
return $default(_that.start,_that.end,_that.runs);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int start,  int end,  List<StyledRun> runs)?  $default,) {final _that = this;
switch (_that) {
case _SourceLine() when $default != null:
return $default(_that.start,_that.end,_that.runs);case _:
  return null;

}
}

}

/// @nodoc


class _SourceLine implements SourceLine {
  const _SourceLine({required this.start, required this.end, final  List<StyledRun> runs = const <StyledRun>[]}): _runs = runs;
  

/// Where the line starts in the note's text, in UTF-16 units.
@override final  int start;
/// Where it ends: before the line break.
@override final  int end;
/// The runs on this line, in order and not overlapping, each cut to the
/// line; text between them is plain.
 final  List<StyledRun> _runs;
/// The runs on this line, in order and not overlapping, each cut to the
/// line; text between them is plain.
@override@JsonKey() List<StyledRun> get runs {
  if (_runs is EqualUnmodifiableListView) return _runs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_runs);
}


/// Create a copy of SourceLine
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SourceLineCopyWith<_SourceLine> get copyWith => __$SourceLineCopyWithImpl<_SourceLine>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SourceLine&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&const DeepCollectionEquality().equals(other._runs, _runs));
}


@override
int get hashCode => Object.hash(runtimeType,start,end,const DeepCollectionEquality().hash(_runs));

@override
String toString() {
  return 'SourceLine(start: $start, end: $end, runs: $runs)';
}


}

/// @nodoc
abstract mixin class _$SourceLineCopyWith<$Res> implements $SourceLineCopyWith<$Res> {
  factory _$SourceLineCopyWith(_SourceLine value, $Res Function(_SourceLine) _then) = __$SourceLineCopyWithImpl;
@override @useResult
$Res call({
 int start, int end, List<StyledRun> runs
});




}
/// @nodoc
class __$SourceLineCopyWithImpl<$Res>
    implements _$SourceLineCopyWith<$Res> {
  __$SourceLineCopyWithImpl(this._self, this._then);

  final _SourceLine _self;
  final $Res Function(_SourceLine) _then;

/// Create a copy of SourceLine
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? start = null,Object? end = null,Object? runs = null,}) {
  return _then(_SourceLine(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,runs: null == runs ? _self._runs : runs // ignore: cast_nullable_to_non_nullable
as List<StyledRun>,
  ));
}


}

// dart format on
