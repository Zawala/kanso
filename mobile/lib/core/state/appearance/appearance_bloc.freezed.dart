// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'appearance_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AppearanceEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AppearanceEvent()';
}


}

/// @nodoc
class $AppearanceEventCopyWith<$Res>  {
$AppearanceEventCopyWith(AppearanceEvent _, $Res Function(AppearanceEvent) __);
}


/// Adds pattern-matching-related methods to [AppearanceEvent].
extension AppearanceEventPatterns on AppearanceEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AppearanceStartedEvent value)?  started,TResult Function( AppearanceChangedEvent value)?  changed,TResult Function( AppearanceEditorFontChangedEvent value)?  editorFontChanged,TResult Function( AppearancePreviewFontChangedEvent value)?  previewFontChanged,TResult Function( AppearanceCodeFontChangedEvent value)?  codeFontChanged,TResult Function( AppearanceTextSizeChangedEvent value)?  textSizeChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started(_that);case AppearanceChangedEvent() when changed != null:
return changed(_that);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AppearanceStartedEvent value)  started,required TResult Function( AppearanceChangedEvent value)  changed,required TResult Function( AppearanceEditorFontChangedEvent value)  editorFontChanged,required TResult Function( AppearancePreviewFontChangedEvent value)  previewFontChanged,required TResult Function( AppearanceCodeFontChangedEvent value)  codeFontChanged,required TResult Function( AppearanceTextSizeChangedEvent value)  textSizeChanged,}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent():
return started(_that);case AppearanceChangedEvent():
return changed(_that);case AppearanceEditorFontChangedEvent():
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent():
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent():
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent():
return textSizeChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AppearanceStartedEvent value)?  started,TResult? Function( AppearanceChangedEvent value)?  changed,TResult? Function( AppearanceEditorFontChangedEvent value)?  editorFontChanged,TResult? Function( AppearancePreviewFontChangedEvent value)?  previewFontChanged,TResult? Function( AppearanceCodeFontChangedEvent value)?  codeFontChanged,TResult? Function( AppearanceTextSizeChangedEvent value)?  textSizeChanged,}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started(_that);case AppearanceChangedEvent() when changed != null:
return changed(_that);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( AppearanceMode mode)?  changed,TResult Function( KansoFontFamily font)?  editorFontChanged,TResult Function( KansoFontFamily font)?  previewFontChanged,TResult Function( KansoFontFamily font)?  codeFontChanged,TResult Function( KansoTextSize size)?  textSizeChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started();case AppearanceChangedEvent() when changed != null:
return changed(_that.mode);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that.size);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( AppearanceMode mode)  changed,required TResult Function( KansoFontFamily font)  editorFontChanged,required TResult Function( KansoFontFamily font)  previewFontChanged,required TResult Function( KansoFontFamily font)  codeFontChanged,required TResult Function( KansoTextSize size)  textSizeChanged,}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent():
return started();case AppearanceChangedEvent():
return changed(_that.mode);case AppearanceEditorFontChangedEvent():
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent():
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent():
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent():
return textSizeChanged(_that.size);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( AppearanceMode mode)?  changed,TResult? Function( KansoFontFamily font)?  editorFontChanged,TResult? Function( KansoFontFamily font)?  previewFontChanged,TResult? Function( KansoFontFamily font)?  codeFontChanged,TResult? Function( KansoTextSize size)?  textSizeChanged,}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started();case AppearanceChangedEvent() when changed != null:
return changed(_that.mode);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that.size);case _:
  return null;

}
}

}

/// @nodoc


class AppearanceStartedEvent implements AppearanceEvent {
  const AppearanceStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AppearanceEvent.started()';
}


}




/// @nodoc


class AppearanceChangedEvent implements AppearanceEvent {
  const AppearanceChangedEvent(this.mode);
  

 final  AppearanceMode mode;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceChangedEventCopyWith<AppearanceChangedEvent> get copyWith => _$AppearanceChangedEventCopyWithImpl<AppearanceChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceChangedEvent&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,mode);

@override
String toString() {
  return 'AppearanceEvent.changed(mode: $mode)';
}


}

/// @nodoc
abstract mixin class $AppearanceChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceChangedEventCopyWith(AppearanceChangedEvent value, $Res Function(AppearanceChangedEvent) _then) = _$AppearanceChangedEventCopyWithImpl;
@useResult
$Res call({
 AppearanceMode mode
});




}
/// @nodoc
class _$AppearanceChangedEventCopyWithImpl<$Res>
    implements $AppearanceChangedEventCopyWith<$Res> {
  _$AppearanceChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceChangedEvent _self;
  final $Res Function(AppearanceChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? mode = null,}) {
  return _then(AppearanceChangedEvent(
null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as AppearanceMode,
  ));
}


}

/// @nodoc


class AppearanceEditorFontChangedEvent implements AppearanceEvent {
  const AppearanceEditorFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceEditorFontChangedEventCopyWith<AppearanceEditorFontChangedEvent> get copyWith => _$AppearanceEditorFontChangedEventCopyWithImpl<AppearanceEditorFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceEditorFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.editorFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearanceEditorFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceEditorFontChangedEventCopyWith(AppearanceEditorFontChangedEvent value, $Res Function(AppearanceEditorFontChangedEvent) _then) = _$AppearanceEditorFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearanceEditorFontChangedEventCopyWithImpl<$Res>
    implements $AppearanceEditorFontChangedEventCopyWith<$Res> {
  _$AppearanceEditorFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceEditorFontChangedEvent _self;
  final $Res Function(AppearanceEditorFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearanceEditorFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearancePreviewFontChangedEvent implements AppearanceEvent {
  const AppearancePreviewFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearancePreviewFontChangedEventCopyWith<AppearancePreviewFontChangedEvent> get copyWith => _$AppearancePreviewFontChangedEventCopyWithImpl<AppearancePreviewFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearancePreviewFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.previewFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearancePreviewFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearancePreviewFontChangedEventCopyWith(AppearancePreviewFontChangedEvent value, $Res Function(AppearancePreviewFontChangedEvent) _then) = _$AppearancePreviewFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearancePreviewFontChangedEventCopyWithImpl<$Res>
    implements $AppearancePreviewFontChangedEventCopyWith<$Res> {
  _$AppearancePreviewFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearancePreviewFontChangedEvent _self;
  final $Res Function(AppearancePreviewFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearancePreviewFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearanceCodeFontChangedEvent implements AppearanceEvent {
  const AppearanceCodeFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceCodeFontChangedEventCopyWith<AppearanceCodeFontChangedEvent> get copyWith => _$AppearanceCodeFontChangedEventCopyWithImpl<AppearanceCodeFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceCodeFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.codeFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearanceCodeFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceCodeFontChangedEventCopyWith(AppearanceCodeFontChangedEvent value, $Res Function(AppearanceCodeFontChangedEvent) _then) = _$AppearanceCodeFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearanceCodeFontChangedEventCopyWithImpl<$Res>
    implements $AppearanceCodeFontChangedEventCopyWith<$Res> {
  _$AppearanceCodeFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceCodeFontChangedEvent _self;
  final $Res Function(AppearanceCodeFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearanceCodeFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearanceTextSizeChangedEvent implements AppearanceEvent {
  const AppearanceTextSizeChangedEvent(this.size);
  

 final  KansoTextSize size;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceTextSizeChangedEventCopyWith<AppearanceTextSizeChangedEvent> get copyWith => _$AppearanceTextSizeChangedEventCopyWithImpl<AppearanceTextSizeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceTextSizeChangedEvent&&(identical(other.size, size) || other.size == size));
}


@override
int get hashCode => Object.hash(runtimeType,size);

@override
String toString() {
  return 'AppearanceEvent.textSizeChanged(size: $size)';
}


}

/// @nodoc
abstract mixin class $AppearanceTextSizeChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceTextSizeChangedEventCopyWith(AppearanceTextSizeChangedEvent value, $Res Function(AppearanceTextSizeChangedEvent) _then) = _$AppearanceTextSizeChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoTextSize size
});




}
/// @nodoc
class _$AppearanceTextSizeChangedEventCopyWithImpl<$Res>
    implements $AppearanceTextSizeChangedEventCopyWith<$Res> {
  _$AppearanceTextSizeChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceTextSizeChangedEvent _self;
  final $Res Function(AppearanceTextSizeChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? size = null,}) {
  return _then(AppearanceTextSizeChangedEvent(
null == size ? _self.size : size // ignore: cast_nullable_to_non_nullable
as KansoTextSize,
  ));
}


}

/// @nodoc
mixin _$AppearanceState {

 AppearanceMode get mode; KansoFonts get fonts; KansoTextSize get textSize;
/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceStateCopyWith<AppearanceState> get copyWith => _$AppearanceStateCopyWithImpl<AppearanceState>(this as AppearanceState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceState&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize));
}


@override
int get hashCode => Object.hash(runtimeType,mode,fonts,textSize);

@override
String toString() {
  return 'AppearanceState(mode: $mode, fonts: $fonts, textSize: $textSize)';
}


}

/// @nodoc
abstract mixin class $AppearanceStateCopyWith<$Res>  {
  factory $AppearanceStateCopyWith(AppearanceState value, $Res Function(AppearanceState) _then) = _$AppearanceStateCopyWithImpl;
@useResult
$Res call({
 AppearanceMode mode, KansoFonts fonts, KansoTextSize textSize
});


$KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class _$AppearanceStateCopyWithImpl<$Res>
    implements $AppearanceStateCopyWith<$Res> {
  _$AppearanceStateCopyWithImpl(this._self, this._then);

  final AppearanceState _self;
  final $Res Function(AppearanceState) _then;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? mode = null,Object? fonts = null,Object? textSize = null,}) {
  return _then(_self.copyWith(
mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as AppearanceMode,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as KansoTextSize,
  ));
}
/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}
}


/// Adds pattern-matching-related methods to [AppearanceState].
extension AppearanceStatePatterns on AppearanceState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AppearanceState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AppearanceState value)  $default,){
final _that = this;
switch (_that) {
case _AppearanceState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AppearanceState value)?  $default,){
final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( AppearanceMode mode,  KansoFonts fonts,  KansoTextSize textSize)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
return $default(_that.mode,_that.fonts,_that.textSize);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( AppearanceMode mode,  KansoFonts fonts,  KansoTextSize textSize)  $default,) {final _that = this;
switch (_that) {
case _AppearanceState():
return $default(_that.mode,_that.fonts,_that.textSize);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( AppearanceMode mode,  KansoFonts fonts,  KansoTextSize textSize)?  $default,) {final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
return $default(_that.mode,_that.fonts,_that.textSize);case _:
  return null;

}
}

}

/// @nodoc


class _AppearanceState implements AppearanceState {
  const _AppearanceState({this.mode = AppearanceMode.system, this.fonts = const KansoFonts(), this.textSize = KansoTextSize.standard});
  

@override@JsonKey() final  AppearanceMode mode;
@override@JsonKey() final  KansoFonts fonts;
@override@JsonKey() final  KansoTextSize textSize;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AppearanceStateCopyWith<_AppearanceState> get copyWith => __$AppearanceStateCopyWithImpl<_AppearanceState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AppearanceState&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.textSize, textSize) || other.textSize == textSize));
}


@override
int get hashCode => Object.hash(runtimeType,mode,fonts,textSize);

@override
String toString() {
  return 'AppearanceState(mode: $mode, fonts: $fonts, textSize: $textSize)';
}


}

/// @nodoc
abstract mixin class _$AppearanceStateCopyWith<$Res> implements $AppearanceStateCopyWith<$Res> {
  factory _$AppearanceStateCopyWith(_AppearanceState value, $Res Function(_AppearanceState) _then) = __$AppearanceStateCopyWithImpl;
@override @useResult
$Res call({
 AppearanceMode mode, KansoFonts fonts, KansoTextSize textSize
});


@override $KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class __$AppearanceStateCopyWithImpl<$Res>
    implements _$AppearanceStateCopyWith<$Res> {
  __$AppearanceStateCopyWithImpl(this._self, this._then);

  final _AppearanceState _self;
  final $Res Function(_AppearanceState) _then;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? mode = null,Object? fonts = null,Object? textSize = null,}) {
  return _then(_AppearanceState(
mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as AppearanceMode,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as KansoTextSize,
  ));
}

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}
}

// dart format on
