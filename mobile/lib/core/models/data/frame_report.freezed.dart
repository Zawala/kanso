// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'frame_report.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FrameReport {

/// How many frames were drawn.
 int get frames;/// The display's refresh rate in hertz, as the app reported it.
 double get refreshRate;/// How long a frame may take on each thread to keep up with the display:
/// `1000 / refreshRate`, in milliseconds.
 double get budgetMs;/// The median, 90th and 99th percentile of the build times, in
/// milliseconds.
 double get buildP50; double get buildP90; double get buildP99;/// The same for the raster times.
 double get rasterP50; double get rasterP90; double get rasterP99;/// How many frames took longer than [budgetMs] on either thread.
 int get framesOverBudget;
/// Create a copy of FrameReport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FrameReportCopyWith<FrameReport> get copyWith => _$FrameReportCopyWithImpl<FrameReport>(this as FrameReport, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FrameReport&&(identical(other.frames, frames) || other.frames == frames)&&(identical(other.refreshRate, refreshRate) || other.refreshRate == refreshRate)&&(identical(other.budgetMs, budgetMs) || other.budgetMs == budgetMs)&&(identical(other.buildP50, buildP50) || other.buildP50 == buildP50)&&(identical(other.buildP90, buildP90) || other.buildP90 == buildP90)&&(identical(other.buildP99, buildP99) || other.buildP99 == buildP99)&&(identical(other.rasterP50, rasterP50) || other.rasterP50 == rasterP50)&&(identical(other.rasterP90, rasterP90) || other.rasterP90 == rasterP90)&&(identical(other.rasterP99, rasterP99) || other.rasterP99 == rasterP99)&&(identical(other.framesOverBudget, framesOverBudget) || other.framesOverBudget == framesOverBudget));
}


@override
int get hashCode => Object.hash(runtimeType,frames,refreshRate,budgetMs,buildP50,buildP90,buildP99,rasterP50,rasterP90,rasterP99,framesOverBudget);

@override
String toString() {
  return 'FrameReport(frames: $frames, refreshRate: $refreshRate, budgetMs: $budgetMs, buildP50: $buildP50, buildP90: $buildP90, buildP99: $buildP99, rasterP50: $rasterP50, rasterP90: $rasterP90, rasterP99: $rasterP99, framesOverBudget: $framesOverBudget)';
}


}

/// @nodoc
abstract mixin class $FrameReportCopyWith<$Res>  {
  factory $FrameReportCopyWith(FrameReport value, $Res Function(FrameReport) _then) = _$FrameReportCopyWithImpl;
@useResult
$Res call({
 int frames, double refreshRate, double budgetMs, double buildP50, double buildP90, double buildP99, double rasterP50, double rasterP90, double rasterP99, int framesOverBudget
});




}
/// @nodoc
class _$FrameReportCopyWithImpl<$Res>
    implements $FrameReportCopyWith<$Res> {
  _$FrameReportCopyWithImpl(this._self, this._then);

  final FrameReport _self;
  final $Res Function(FrameReport) _then;

/// Create a copy of FrameReport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? frames = null,Object? refreshRate = null,Object? budgetMs = null,Object? buildP50 = null,Object? buildP90 = null,Object? buildP99 = null,Object? rasterP50 = null,Object? rasterP90 = null,Object? rasterP99 = null,Object? framesOverBudget = null,}) {
  return _then(_self.copyWith(
frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as int,refreshRate: null == refreshRate ? _self.refreshRate : refreshRate // ignore: cast_nullable_to_non_nullable
as double,budgetMs: null == budgetMs ? _self.budgetMs : budgetMs // ignore: cast_nullable_to_non_nullable
as double,buildP50: null == buildP50 ? _self.buildP50 : buildP50 // ignore: cast_nullable_to_non_nullable
as double,buildP90: null == buildP90 ? _self.buildP90 : buildP90 // ignore: cast_nullable_to_non_nullable
as double,buildP99: null == buildP99 ? _self.buildP99 : buildP99 // ignore: cast_nullable_to_non_nullable
as double,rasterP50: null == rasterP50 ? _self.rasterP50 : rasterP50 // ignore: cast_nullable_to_non_nullable
as double,rasterP90: null == rasterP90 ? _self.rasterP90 : rasterP90 // ignore: cast_nullable_to_non_nullable
as double,rasterP99: null == rasterP99 ? _self.rasterP99 : rasterP99 // ignore: cast_nullable_to_non_nullable
as double,framesOverBudget: null == framesOverBudget ? _self.framesOverBudget : framesOverBudget // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [FrameReport].
extension FrameReportPatterns on FrameReport {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FrameReport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FrameReport() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FrameReport value)  $default,){
final _that = this;
switch (_that) {
case _FrameReport():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FrameReport value)?  $default,){
final _that = this;
switch (_that) {
case _FrameReport() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int frames,  double refreshRate,  double budgetMs,  double buildP50,  double buildP90,  double buildP99,  double rasterP50,  double rasterP90,  double rasterP99,  int framesOverBudget)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FrameReport() when $default != null:
return $default(_that.frames,_that.refreshRate,_that.budgetMs,_that.buildP50,_that.buildP90,_that.buildP99,_that.rasterP50,_that.rasterP90,_that.rasterP99,_that.framesOverBudget);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int frames,  double refreshRate,  double budgetMs,  double buildP50,  double buildP90,  double buildP99,  double rasterP50,  double rasterP90,  double rasterP99,  int framesOverBudget)  $default,) {final _that = this;
switch (_that) {
case _FrameReport():
return $default(_that.frames,_that.refreshRate,_that.budgetMs,_that.buildP50,_that.buildP90,_that.buildP99,_that.rasterP50,_that.rasterP90,_that.rasterP99,_that.framesOverBudget);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int frames,  double refreshRate,  double budgetMs,  double buildP50,  double buildP90,  double buildP99,  double rasterP50,  double rasterP90,  double rasterP99,  int framesOverBudget)?  $default,) {final _that = this;
switch (_that) {
case _FrameReport() when $default != null:
return $default(_that.frames,_that.refreshRate,_that.budgetMs,_that.buildP50,_that.buildP90,_that.buildP99,_that.rasterP50,_that.rasterP90,_that.rasterP99,_that.framesOverBudget);case _:
  return null;

}
}

}

/// @nodoc


class _FrameReport extends FrameReport {
  const _FrameReport({required this.frames, required this.refreshRate, required this.budgetMs, required this.buildP50, required this.buildP90, required this.buildP99, required this.rasterP50, required this.rasterP90, required this.rasterP99, required this.framesOverBudget}): super._();
  

/// How many frames were drawn.
@override final  int frames;
/// The display's refresh rate in hertz, as the app reported it.
@override final  double refreshRate;
/// How long a frame may take on each thread to keep up with the display:
/// `1000 / refreshRate`, in milliseconds.
@override final  double budgetMs;
/// The median, 90th and 99th percentile of the build times, in
/// milliseconds.
@override final  double buildP50;
@override final  double buildP90;
@override final  double buildP99;
/// The same for the raster times.
@override final  double rasterP50;
@override final  double rasterP90;
@override final  double rasterP99;
/// How many frames took longer than [budgetMs] on either thread.
@override final  int framesOverBudget;

/// Create a copy of FrameReport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FrameReportCopyWith<_FrameReport> get copyWith => __$FrameReportCopyWithImpl<_FrameReport>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FrameReport&&(identical(other.frames, frames) || other.frames == frames)&&(identical(other.refreshRate, refreshRate) || other.refreshRate == refreshRate)&&(identical(other.budgetMs, budgetMs) || other.budgetMs == budgetMs)&&(identical(other.buildP50, buildP50) || other.buildP50 == buildP50)&&(identical(other.buildP90, buildP90) || other.buildP90 == buildP90)&&(identical(other.buildP99, buildP99) || other.buildP99 == buildP99)&&(identical(other.rasterP50, rasterP50) || other.rasterP50 == rasterP50)&&(identical(other.rasterP90, rasterP90) || other.rasterP90 == rasterP90)&&(identical(other.rasterP99, rasterP99) || other.rasterP99 == rasterP99)&&(identical(other.framesOverBudget, framesOverBudget) || other.framesOverBudget == framesOverBudget));
}


@override
int get hashCode => Object.hash(runtimeType,frames,refreshRate,budgetMs,buildP50,buildP90,buildP99,rasterP50,rasterP90,rasterP99,framesOverBudget);

@override
String toString() {
  return 'FrameReport(frames: $frames, refreshRate: $refreshRate, budgetMs: $budgetMs, buildP50: $buildP50, buildP90: $buildP90, buildP99: $buildP99, rasterP50: $rasterP50, rasterP90: $rasterP90, rasterP99: $rasterP99, framesOverBudget: $framesOverBudget)';
}


}

/// @nodoc
abstract mixin class _$FrameReportCopyWith<$Res> implements $FrameReportCopyWith<$Res> {
  factory _$FrameReportCopyWith(_FrameReport value, $Res Function(_FrameReport) _then) = __$FrameReportCopyWithImpl;
@override @useResult
$Res call({
 int frames, double refreshRate, double budgetMs, double buildP50, double buildP90, double buildP99, double rasterP50, double rasterP90, double rasterP99, int framesOverBudget
});




}
/// @nodoc
class __$FrameReportCopyWithImpl<$Res>
    implements _$FrameReportCopyWith<$Res> {
  __$FrameReportCopyWithImpl(this._self, this._then);

  final _FrameReport _self;
  final $Res Function(_FrameReport) _then;

/// Create a copy of FrameReport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? frames = null,Object? refreshRate = null,Object? budgetMs = null,Object? buildP50 = null,Object? buildP90 = null,Object? buildP99 = null,Object? rasterP50 = null,Object? rasterP90 = null,Object? rasterP99 = null,Object? framesOverBudget = null,}) {
  return _then(_FrameReport(
frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as int,refreshRate: null == refreshRate ? _self.refreshRate : refreshRate // ignore: cast_nullable_to_non_nullable
as double,budgetMs: null == budgetMs ? _self.budgetMs : budgetMs // ignore: cast_nullable_to_non_nullable
as double,buildP50: null == buildP50 ? _self.buildP50 : buildP50 // ignore: cast_nullable_to_non_nullable
as double,buildP90: null == buildP90 ? _self.buildP90 : buildP90 // ignore: cast_nullable_to_non_nullable
as double,buildP99: null == buildP99 ? _self.buildP99 : buildP99 // ignore: cast_nullable_to_non_nullable
as double,rasterP50: null == rasterP50 ? _self.rasterP50 : rasterP50 // ignore: cast_nullable_to_non_nullable
as double,rasterP90: null == rasterP90 ? _self.rasterP90 : rasterP90 // ignore: cast_nullable_to_non_nullable
as double,rasterP99: null == rasterP99 ? _self.rasterP99 : rasterP99 // ignore: cast_nullable_to_non_nullable
as double,framesOverBudget: null == framesOverBudget ? _self.framesOverBudget : framesOverBudget // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
