// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'crash_reports_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CrashReportsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent()';
}


}

/// @nodoc
class $CrashReportsEventCopyWith<$Res>  {
$CrashReportsEventCopyWith(CrashReportsEvent _, $Res Function(CrashReportsEvent) __);
}


/// Adds pattern-matching-related methods to [CrashReportsEvent].
extension CrashReportsEventPatterns on CrashReportsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CrashReportsStartedEvent value)?  started,TResult Function( CrashReportsNoticeReportToggledEvent value)?  noticeReportToggled,TResult Function( CrashReportsReportToggledEvent value)?  reportToggled,TResult Function( CrashReportsSendRequestedEvent value)?  sendRequested,TResult Function( CrashReportsDiscardRequestedEvent value)?  discardRequested,TResult Function( CrashReportsSendAllRequestedEvent value)?  sendAllRequested,TResult Function( CrashReportsDiscardAllRequestedEvent value)?  discardAllRequested,TResult Function( CrashReportsOutcomeClearedEvent value)?  outcomeCleared,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CrashReportsStartedEvent() when started != null:
return started(_that);case CrashReportsNoticeReportToggledEvent() when noticeReportToggled != null:
return noticeReportToggled(_that);case CrashReportsReportToggledEvent() when reportToggled != null:
return reportToggled(_that);case CrashReportsSendRequestedEvent() when sendRequested != null:
return sendRequested(_that);case CrashReportsDiscardRequestedEvent() when discardRequested != null:
return discardRequested(_that);case CrashReportsSendAllRequestedEvent() when sendAllRequested != null:
return sendAllRequested(_that);case CrashReportsDiscardAllRequestedEvent() when discardAllRequested != null:
return discardAllRequested(_that);case CrashReportsOutcomeClearedEvent() when outcomeCleared != null:
return outcomeCleared(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CrashReportsStartedEvent value)  started,required TResult Function( CrashReportsNoticeReportToggledEvent value)  noticeReportToggled,required TResult Function( CrashReportsReportToggledEvent value)  reportToggled,required TResult Function( CrashReportsSendRequestedEvent value)  sendRequested,required TResult Function( CrashReportsDiscardRequestedEvent value)  discardRequested,required TResult Function( CrashReportsSendAllRequestedEvent value)  sendAllRequested,required TResult Function( CrashReportsDiscardAllRequestedEvent value)  discardAllRequested,required TResult Function( CrashReportsOutcomeClearedEvent value)  outcomeCleared,}){
final _that = this;
switch (_that) {
case CrashReportsStartedEvent():
return started(_that);case CrashReportsNoticeReportToggledEvent():
return noticeReportToggled(_that);case CrashReportsReportToggledEvent():
return reportToggled(_that);case CrashReportsSendRequestedEvent():
return sendRequested(_that);case CrashReportsDiscardRequestedEvent():
return discardRequested(_that);case CrashReportsSendAllRequestedEvent():
return sendAllRequested(_that);case CrashReportsDiscardAllRequestedEvent():
return discardAllRequested(_that);case CrashReportsOutcomeClearedEvent():
return outcomeCleared(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CrashReportsStartedEvent value)?  started,TResult? Function( CrashReportsNoticeReportToggledEvent value)?  noticeReportToggled,TResult? Function( CrashReportsReportToggledEvent value)?  reportToggled,TResult? Function( CrashReportsSendRequestedEvent value)?  sendRequested,TResult? Function( CrashReportsDiscardRequestedEvent value)?  discardRequested,TResult? Function( CrashReportsSendAllRequestedEvent value)?  sendAllRequested,TResult? Function( CrashReportsDiscardAllRequestedEvent value)?  discardAllRequested,TResult? Function( CrashReportsOutcomeClearedEvent value)?  outcomeCleared,}){
final _that = this;
switch (_that) {
case CrashReportsStartedEvent() when started != null:
return started(_that);case CrashReportsNoticeReportToggledEvent() when noticeReportToggled != null:
return noticeReportToggled(_that);case CrashReportsReportToggledEvent() when reportToggled != null:
return reportToggled(_that);case CrashReportsSendRequestedEvent() when sendRequested != null:
return sendRequested(_that);case CrashReportsDiscardRequestedEvent() when discardRequested != null:
return discardRequested(_that);case CrashReportsSendAllRequestedEvent() when sendAllRequested != null:
return sendAllRequested(_that);case CrashReportsDiscardAllRequestedEvent() when discardAllRequested != null:
return discardAllRequested(_that);case CrashReportsOutcomeClearedEvent() when outcomeCleared != null:
return outcomeCleared(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  noticeReportToggled,TResult Function( String id)?  reportToggled,TResult Function( String id)?  sendRequested,TResult Function( String id)?  discardRequested,TResult Function()?  sendAllRequested,TResult Function()?  discardAllRequested,TResult Function()?  outcomeCleared,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CrashReportsStartedEvent() when started != null:
return started();case CrashReportsNoticeReportToggledEvent() when noticeReportToggled != null:
return noticeReportToggled();case CrashReportsReportToggledEvent() when reportToggled != null:
return reportToggled(_that.id);case CrashReportsSendRequestedEvent() when sendRequested != null:
return sendRequested(_that.id);case CrashReportsDiscardRequestedEvent() when discardRequested != null:
return discardRequested(_that.id);case CrashReportsSendAllRequestedEvent() when sendAllRequested != null:
return sendAllRequested();case CrashReportsDiscardAllRequestedEvent() when discardAllRequested != null:
return discardAllRequested();case CrashReportsOutcomeClearedEvent() when outcomeCleared != null:
return outcomeCleared();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  noticeReportToggled,required TResult Function( String id)  reportToggled,required TResult Function( String id)  sendRequested,required TResult Function( String id)  discardRequested,required TResult Function()  sendAllRequested,required TResult Function()  discardAllRequested,required TResult Function()  outcomeCleared,}) {final _that = this;
switch (_that) {
case CrashReportsStartedEvent():
return started();case CrashReportsNoticeReportToggledEvent():
return noticeReportToggled();case CrashReportsReportToggledEvent():
return reportToggled(_that.id);case CrashReportsSendRequestedEvent():
return sendRequested(_that.id);case CrashReportsDiscardRequestedEvent():
return discardRequested(_that.id);case CrashReportsSendAllRequestedEvent():
return sendAllRequested();case CrashReportsDiscardAllRequestedEvent():
return discardAllRequested();case CrashReportsOutcomeClearedEvent():
return outcomeCleared();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  noticeReportToggled,TResult? Function( String id)?  reportToggled,TResult? Function( String id)?  sendRequested,TResult? Function( String id)?  discardRequested,TResult? Function()?  sendAllRequested,TResult? Function()?  discardAllRequested,TResult? Function()?  outcomeCleared,}) {final _that = this;
switch (_that) {
case CrashReportsStartedEvent() when started != null:
return started();case CrashReportsNoticeReportToggledEvent() when noticeReportToggled != null:
return noticeReportToggled();case CrashReportsReportToggledEvent() when reportToggled != null:
return reportToggled(_that.id);case CrashReportsSendRequestedEvent() when sendRequested != null:
return sendRequested(_that.id);case CrashReportsDiscardRequestedEvent() when discardRequested != null:
return discardRequested(_that.id);case CrashReportsSendAllRequestedEvent() when sendAllRequested != null:
return sendAllRequested();case CrashReportsDiscardAllRequestedEvent() when discardAllRequested != null:
return discardAllRequested();case CrashReportsOutcomeClearedEvent() when outcomeCleared != null:
return outcomeCleared();case _:
  return null;

}
}

}

/// @nodoc


class CrashReportsStartedEvent implements CrashReportsEvent {
  const CrashReportsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent.started()';
}


}




/// @nodoc


class CrashReportsNoticeReportToggledEvent implements CrashReportsEvent {
  const CrashReportsNoticeReportToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsNoticeReportToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent.noticeReportToggled()';
}


}




/// @nodoc


class CrashReportsReportToggledEvent implements CrashReportsEvent {
  const CrashReportsReportToggledEvent(this.id);
  

 final  String id;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashReportsReportToggledEventCopyWith<CrashReportsReportToggledEvent> get copyWith => _$CrashReportsReportToggledEventCopyWithImpl<CrashReportsReportToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsReportToggledEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CrashReportsEvent.reportToggled(id: $id)';
}


}

/// @nodoc
abstract mixin class $CrashReportsReportToggledEventCopyWith<$Res> implements $CrashReportsEventCopyWith<$Res> {
  factory $CrashReportsReportToggledEventCopyWith(CrashReportsReportToggledEvent value, $Res Function(CrashReportsReportToggledEvent) _then) = _$CrashReportsReportToggledEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CrashReportsReportToggledEventCopyWithImpl<$Res>
    implements $CrashReportsReportToggledEventCopyWith<$Res> {
  _$CrashReportsReportToggledEventCopyWithImpl(this._self, this._then);

  final CrashReportsReportToggledEvent _self;
  final $Res Function(CrashReportsReportToggledEvent) _then;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(CrashReportsReportToggledEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CrashReportsSendRequestedEvent implements CrashReportsEvent {
  const CrashReportsSendRequestedEvent(this.id);
  

 final  String id;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashReportsSendRequestedEventCopyWith<CrashReportsSendRequestedEvent> get copyWith => _$CrashReportsSendRequestedEventCopyWithImpl<CrashReportsSendRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsSendRequestedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CrashReportsEvent.sendRequested(id: $id)';
}


}

/// @nodoc
abstract mixin class $CrashReportsSendRequestedEventCopyWith<$Res> implements $CrashReportsEventCopyWith<$Res> {
  factory $CrashReportsSendRequestedEventCopyWith(CrashReportsSendRequestedEvent value, $Res Function(CrashReportsSendRequestedEvent) _then) = _$CrashReportsSendRequestedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CrashReportsSendRequestedEventCopyWithImpl<$Res>
    implements $CrashReportsSendRequestedEventCopyWith<$Res> {
  _$CrashReportsSendRequestedEventCopyWithImpl(this._self, this._then);

  final CrashReportsSendRequestedEvent _self;
  final $Res Function(CrashReportsSendRequestedEvent) _then;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(CrashReportsSendRequestedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CrashReportsDiscardRequestedEvent implements CrashReportsEvent {
  const CrashReportsDiscardRequestedEvent(this.id);
  

 final  String id;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashReportsDiscardRequestedEventCopyWith<CrashReportsDiscardRequestedEvent> get copyWith => _$CrashReportsDiscardRequestedEventCopyWithImpl<CrashReportsDiscardRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsDiscardRequestedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'CrashReportsEvent.discardRequested(id: $id)';
}


}

/// @nodoc
abstract mixin class $CrashReportsDiscardRequestedEventCopyWith<$Res> implements $CrashReportsEventCopyWith<$Res> {
  factory $CrashReportsDiscardRequestedEventCopyWith(CrashReportsDiscardRequestedEvent value, $Res Function(CrashReportsDiscardRequestedEvent) _then) = _$CrashReportsDiscardRequestedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$CrashReportsDiscardRequestedEventCopyWithImpl<$Res>
    implements $CrashReportsDiscardRequestedEventCopyWith<$Res> {
  _$CrashReportsDiscardRequestedEventCopyWithImpl(this._self, this._then);

  final CrashReportsDiscardRequestedEvent _self;
  final $Res Function(CrashReportsDiscardRequestedEvent) _then;

/// Create a copy of CrashReportsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(CrashReportsDiscardRequestedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CrashReportsSendAllRequestedEvent implements CrashReportsEvent {
  const CrashReportsSendAllRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsSendAllRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent.sendAllRequested()';
}


}




/// @nodoc


class CrashReportsDiscardAllRequestedEvent implements CrashReportsEvent {
  const CrashReportsDiscardAllRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsDiscardAllRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent.discardAllRequested()';
}


}




/// @nodoc


class CrashReportsOutcomeClearedEvent implements CrashReportsEvent {
  const CrashReportsOutcomeClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsOutcomeClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CrashReportsEvent.outcomeCleared()';
}


}




/// @nodoc
mixin _$CrashReportsState {

 LoadStatus get status;/// The reports waiting on the device, oldest first.
 List<PendingCrashReport> get reports;/// Whether the "closed unexpectedly" notice is over the app.
 bool get noticeVisible;/// Whether the notice shows the text of the reports.
 bool get noticeShowsReport;/// The report whose text is open on the page.
 String? get expanded;/// The reports being sent now.
 Set<String> get sending;/// How the last send ended; null before one, or once it was read.
 CrashSendOutcome? get outcome;
/// Create a copy of CrashReportsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CrashReportsStateCopyWith<CrashReportsState> get copyWith => _$CrashReportsStateCopyWithImpl<CrashReportsState>(this as CrashReportsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CrashReportsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.reports, reports)&&(identical(other.noticeVisible, noticeVisible) || other.noticeVisible == noticeVisible)&&(identical(other.noticeShowsReport, noticeShowsReport) || other.noticeShowsReport == noticeShowsReport)&&(identical(other.expanded, expanded) || other.expanded == expanded)&&const DeepCollectionEquality().equals(other.sending, sending)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(reports),noticeVisible,noticeShowsReport,expanded,const DeepCollectionEquality().hash(sending),outcome);

@override
String toString() {
  return 'CrashReportsState(status: $status, reports: $reports, noticeVisible: $noticeVisible, noticeShowsReport: $noticeShowsReport, expanded: $expanded, sending: $sending, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class $CrashReportsStateCopyWith<$Res>  {
  factory $CrashReportsStateCopyWith(CrashReportsState value, $Res Function(CrashReportsState) _then) = _$CrashReportsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<PendingCrashReport> reports, bool noticeVisible, bool noticeShowsReport, String? expanded, Set<String> sending, CrashSendOutcome? outcome
});




}
/// @nodoc
class _$CrashReportsStateCopyWithImpl<$Res>
    implements $CrashReportsStateCopyWith<$Res> {
  _$CrashReportsStateCopyWithImpl(this._self, this._then);

  final CrashReportsState _self;
  final $Res Function(CrashReportsState) _then;

/// Create a copy of CrashReportsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? reports = null,Object? noticeVisible = null,Object? noticeShowsReport = null,Object? expanded = freezed,Object? sending = null,Object? outcome = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,reports: null == reports ? _self.reports : reports // ignore: cast_nullable_to_non_nullable
as List<PendingCrashReport>,noticeVisible: null == noticeVisible ? _self.noticeVisible : noticeVisible // ignore: cast_nullable_to_non_nullable
as bool,noticeShowsReport: null == noticeShowsReport ? _self.noticeShowsReport : noticeShowsReport // ignore: cast_nullable_to_non_nullable
as bool,expanded: freezed == expanded ? _self.expanded : expanded // ignore: cast_nullable_to_non_nullable
as String?,sending: null == sending ? _self.sending : sending // ignore: cast_nullable_to_non_nullable
as Set<String>,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as CrashSendOutcome?,
  ));
}

}


/// Adds pattern-matching-related methods to [CrashReportsState].
extension CrashReportsStatePatterns on CrashReportsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CrashReportsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CrashReportsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CrashReportsState value)  $default,){
final _that = this;
switch (_that) {
case _CrashReportsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CrashReportsState value)?  $default,){
final _that = this;
switch (_that) {
case _CrashReportsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<PendingCrashReport> reports,  bool noticeVisible,  bool noticeShowsReport,  String? expanded,  Set<String> sending,  CrashSendOutcome? outcome)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CrashReportsState() when $default != null:
return $default(_that.status,_that.reports,_that.noticeVisible,_that.noticeShowsReport,_that.expanded,_that.sending,_that.outcome);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<PendingCrashReport> reports,  bool noticeVisible,  bool noticeShowsReport,  String? expanded,  Set<String> sending,  CrashSendOutcome? outcome)  $default,) {final _that = this;
switch (_that) {
case _CrashReportsState():
return $default(_that.status,_that.reports,_that.noticeVisible,_that.noticeShowsReport,_that.expanded,_that.sending,_that.outcome);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<PendingCrashReport> reports,  bool noticeVisible,  bool noticeShowsReport,  String? expanded,  Set<String> sending,  CrashSendOutcome? outcome)?  $default,) {final _that = this;
switch (_that) {
case _CrashReportsState() when $default != null:
return $default(_that.status,_that.reports,_that.noticeVisible,_that.noticeShowsReport,_that.expanded,_that.sending,_that.outcome);case _:
  return null;

}
}

}

/// @nodoc


class _CrashReportsState implements CrashReportsState {
  const _CrashReportsState({this.status = LoadStatus.loading, final  List<PendingCrashReport> reports = const <PendingCrashReport>[], this.noticeVisible = false, this.noticeShowsReport = false, this.expanded, final  Set<String> sending = const <String>{}, this.outcome}): _reports = reports,_sending = sending;
  

@override@JsonKey() final  LoadStatus status;
/// The reports waiting on the device, oldest first.
 final  List<PendingCrashReport> _reports;
/// The reports waiting on the device, oldest first.
@override@JsonKey() List<PendingCrashReport> get reports {
  if (_reports is EqualUnmodifiableListView) return _reports;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_reports);
}

/// Whether the "closed unexpectedly" notice is over the app.
@override@JsonKey() final  bool noticeVisible;
/// Whether the notice shows the text of the reports.
@override@JsonKey() final  bool noticeShowsReport;
/// The report whose text is open on the page.
@override final  String? expanded;
/// The reports being sent now.
 final  Set<String> _sending;
/// The reports being sent now.
@override@JsonKey() Set<String> get sending {
  if (_sending is EqualUnmodifiableSetView) return _sending;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_sending);
}

/// How the last send ended; null before one, or once it was read.
@override final  CrashSendOutcome? outcome;

/// Create a copy of CrashReportsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CrashReportsStateCopyWith<_CrashReportsState> get copyWith => __$CrashReportsStateCopyWithImpl<_CrashReportsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CrashReportsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._reports, _reports)&&(identical(other.noticeVisible, noticeVisible) || other.noticeVisible == noticeVisible)&&(identical(other.noticeShowsReport, noticeShowsReport) || other.noticeShowsReport == noticeShowsReport)&&(identical(other.expanded, expanded) || other.expanded == expanded)&&const DeepCollectionEquality().equals(other._sending, _sending)&&(identical(other.outcome, outcome) || other.outcome == outcome));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_reports),noticeVisible,noticeShowsReport,expanded,const DeepCollectionEquality().hash(_sending),outcome);

@override
String toString() {
  return 'CrashReportsState(status: $status, reports: $reports, noticeVisible: $noticeVisible, noticeShowsReport: $noticeShowsReport, expanded: $expanded, sending: $sending, outcome: $outcome)';
}


}

/// @nodoc
abstract mixin class _$CrashReportsStateCopyWith<$Res> implements $CrashReportsStateCopyWith<$Res> {
  factory _$CrashReportsStateCopyWith(_CrashReportsState value, $Res Function(_CrashReportsState) _then) = __$CrashReportsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<PendingCrashReport> reports, bool noticeVisible, bool noticeShowsReport, String? expanded, Set<String> sending, CrashSendOutcome? outcome
});




}
/// @nodoc
class __$CrashReportsStateCopyWithImpl<$Res>
    implements _$CrashReportsStateCopyWith<$Res> {
  __$CrashReportsStateCopyWithImpl(this._self, this._then);

  final _CrashReportsState _self;
  final $Res Function(_CrashReportsState) _then;

/// Create a copy of CrashReportsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? reports = null,Object? noticeVisible = null,Object? noticeShowsReport = null,Object? expanded = freezed,Object? sending = null,Object? outcome = freezed,}) {
  return _then(_CrashReportsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,reports: null == reports ? _self._reports : reports // ignore: cast_nullable_to_non_nullable
as List<PendingCrashReport>,noticeVisible: null == noticeVisible ? _self.noticeVisible : noticeVisible // ignore: cast_nullable_to_non_nullable
as bool,noticeShowsReport: null == noticeShowsReport ? _self.noticeShowsReport : noticeShowsReport // ignore: cast_nullable_to_non_nullable
as bool,expanded: freezed == expanded ? _self.expanded : expanded // ignore: cast_nullable_to_non_nullable
as String?,sending: null == sending ? _self._sending : sending // ignore: cast_nullable_to_non_nullable
as Set<String>,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as CrashSendOutcome?,
  ));
}


}

// dart format on
