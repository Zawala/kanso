// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pending_crash_report.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PendingCrashReport {

/// The file's name without its extension.
 String get id; CrashReport get report;/// The exact text that is sent, and shown to the person.
 String get body;
/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PendingCrashReportCopyWith<PendingCrashReport> get copyWith => _$PendingCrashReportCopyWithImpl<PendingCrashReport>(this as PendingCrashReport, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PendingCrashReport&&(identical(other.id, id) || other.id == id)&&(identical(other.report, report) || other.report == report)&&(identical(other.body, body) || other.body == body));
}


@override
int get hashCode => Object.hash(runtimeType,id,report,body);

@override
String toString() {
  return 'PendingCrashReport(id: $id, report: $report, body: $body)';
}


}

/// @nodoc
abstract mixin class $PendingCrashReportCopyWith<$Res>  {
  factory $PendingCrashReportCopyWith(PendingCrashReport value, $Res Function(PendingCrashReport) _then) = _$PendingCrashReportCopyWithImpl;
@useResult
$Res call({
 String id, CrashReport report, String body
});


$CrashReportCopyWith<$Res> get report;

}
/// @nodoc
class _$PendingCrashReportCopyWithImpl<$Res>
    implements $PendingCrashReportCopyWith<$Res> {
  _$PendingCrashReportCopyWithImpl(this._self, this._then);

  final PendingCrashReport _self;
  final $Res Function(PendingCrashReport) _then;

/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? report = null,Object? body = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,report: null == report ? _self.report : report // ignore: cast_nullable_to_non_nullable
as CrashReport,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,
  ));
}
/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashReportCopyWith<$Res> get report {
  
  return $CrashReportCopyWith<$Res>(_self.report, (value) {
    return _then(_self.copyWith(report: value));
  });
}
}


/// Adds pattern-matching-related methods to [PendingCrashReport].
extension PendingCrashReportPatterns on PendingCrashReport {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PendingCrashReport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PendingCrashReport() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PendingCrashReport value)  $default,){
final _that = this;
switch (_that) {
case _PendingCrashReport():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PendingCrashReport value)?  $default,){
final _that = this;
switch (_that) {
case _PendingCrashReport() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  CrashReport report,  String body)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PendingCrashReport() when $default != null:
return $default(_that.id,_that.report,_that.body);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  CrashReport report,  String body)  $default,) {final _that = this;
switch (_that) {
case _PendingCrashReport():
return $default(_that.id,_that.report,_that.body);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  CrashReport report,  String body)?  $default,) {final _that = this;
switch (_that) {
case _PendingCrashReport() when $default != null:
return $default(_that.id,_that.report,_that.body);case _:
  return null;

}
}

}

/// @nodoc


class _PendingCrashReport implements PendingCrashReport {
  const _PendingCrashReport({required this.id, required this.report, required this.body});
  

/// The file's name without its extension.
@override final  String id;
@override final  CrashReport report;
/// The exact text that is sent, and shown to the person.
@override final  String body;

/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PendingCrashReportCopyWith<_PendingCrashReport> get copyWith => __$PendingCrashReportCopyWithImpl<_PendingCrashReport>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PendingCrashReport&&(identical(other.id, id) || other.id == id)&&(identical(other.report, report) || other.report == report)&&(identical(other.body, body) || other.body == body));
}


@override
int get hashCode => Object.hash(runtimeType,id,report,body);

@override
String toString() {
  return 'PendingCrashReport(id: $id, report: $report, body: $body)';
}


}

/// @nodoc
abstract mixin class _$PendingCrashReportCopyWith<$Res> implements $PendingCrashReportCopyWith<$Res> {
  factory _$PendingCrashReportCopyWith(_PendingCrashReport value, $Res Function(_PendingCrashReport) _then) = __$PendingCrashReportCopyWithImpl;
@override @useResult
$Res call({
 String id, CrashReport report, String body
});


@override $CrashReportCopyWith<$Res> get report;

}
/// @nodoc
class __$PendingCrashReportCopyWithImpl<$Res>
    implements _$PendingCrashReportCopyWith<$Res> {
  __$PendingCrashReportCopyWithImpl(this._self, this._then);

  final _PendingCrashReport _self;
  final $Res Function(_PendingCrashReport) _then;

/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? report = null,Object? body = null,}) {
  return _then(_PendingCrashReport(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,report: null == report ? _self.report : report // ignore: cast_nullable_to_non_nullable
as CrashReport,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of PendingCrashReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CrashReportCopyWith<$Res> get report {
  
  return $CrashReportCopyWith<$Res>(_self.report, (value) {
    return _then(_self.copyWith(report: value));
  });
}
}

// dart format on
