// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'kanso_fonts.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$KansoFonts {

/// Writing in the editor.
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily get editorFont;/// Reading a note.
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily get previewFont;/// Code spans, code blocks and the theme previews.
@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily get codeFont;
/// Create a copy of KansoFonts
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<KansoFonts> get copyWith => _$KansoFontsCopyWithImpl<KansoFonts>(this as KansoFonts, _$identity);

  /// Serializes this KansoFonts to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoFonts&&(identical(other.editorFont, editorFont) || other.editorFont == editorFont)&&(identical(other.previewFont, previewFont) || other.previewFont == previewFont)&&(identical(other.codeFont, codeFont) || other.codeFont == codeFont));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,editorFont,previewFont,codeFont);

@override
String toString() {
  return 'KansoFonts(editorFont: $editorFont, previewFont: $previewFont, codeFont: $codeFont)';
}


}

/// @nodoc
abstract mixin class $KansoFontsCopyWith<$Res>  {
  factory $KansoFontsCopyWith(KansoFonts value, $Res Function(KansoFonts) _then) = _$KansoFontsCopyWithImpl;
@useResult
$Res call({
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily editorFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily previewFont,@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily codeFont
});




}
/// @nodoc
class _$KansoFontsCopyWithImpl<$Res>
    implements $KansoFontsCopyWith<$Res> {
  _$KansoFontsCopyWithImpl(this._self, this._then);

  final KansoFonts _self;
  final $Res Function(KansoFonts) _then;

/// Create a copy of KansoFonts
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? editorFont = null,Object? previewFont = null,Object? codeFont = null,}) {
  return _then(_self.copyWith(
editorFont: null == editorFont ? _self.editorFont : editorFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,previewFont: null == previewFont ? _self.previewFont : previewFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,codeFont: null == codeFont ? _self.codeFont : codeFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}

}


/// Adds pattern-matching-related methods to [KansoFonts].
extension KansoFontsPatterns on KansoFonts {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KansoFonts value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KansoFonts value)  $default,){
final _that = this;
switch (_that) {
case _KansoFonts():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KansoFonts value)?  $default,){
final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
return $default(_that.editorFont,_that.previewFont,_that.codeFont);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont)  $default,) {final _that = this;
switch (_that) {
case _KansoFonts():
return $default(_that.editorFont,_that.previewFont,_that.codeFont);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont)?  $default,) {final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
return $default(_that.editorFont,_that.previewFont,_that.codeFont);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _KansoFonts extends KansoFonts {
  const _KansoFonts({@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) this.editorFont = KansoFontFamily.instrumentSans, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) this.previewFont = KansoFontFamily.instrumentSans, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) this.codeFont = KansoFontFamily.jetBrainsMono}): super._();
  factory _KansoFonts.fromJson(Map<String, dynamic> json) => _$KansoFontsFromJson(json);

/// Writing in the editor.
@override@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) final  KansoFontFamily editorFont;
/// Reading a note.
@override@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) final  KansoFontFamily previewFont;
/// Code spans, code blocks and the theme previews.
@override@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) final  KansoFontFamily codeFont;

/// Create a copy of KansoFonts
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KansoFontsCopyWith<_KansoFonts> get copyWith => __$KansoFontsCopyWithImpl<_KansoFonts>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$KansoFontsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoFonts&&(identical(other.editorFont, editorFont) || other.editorFont == editorFont)&&(identical(other.previewFont, previewFont) || other.previewFont == previewFont)&&(identical(other.codeFont, codeFont) || other.codeFont == codeFont));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,editorFont,previewFont,codeFont);

@override
String toString() {
  return 'KansoFonts(editorFont: $editorFont, previewFont: $previewFont, codeFont: $codeFont)';
}


}

/// @nodoc
abstract mixin class _$KansoFontsCopyWith<$Res> implements $KansoFontsCopyWith<$Res> {
  factory _$KansoFontsCopyWith(_KansoFonts value, $Res Function(_KansoFonts) _then) = __$KansoFontsCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily editorFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily previewFont,@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily codeFont
});




}
/// @nodoc
class __$KansoFontsCopyWithImpl<$Res>
    implements _$KansoFontsCopyWith<$Res> {
  __$KansoFontsCopyWithImpl(this._self, this._then);

  final _KansoFonts _self;
  final $Res Function(_KansoFonts) _then;

/// Create a copy of KansoFonts
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? editorFont = null,Object? previewFont = null,Object? codeFont = null,}) {
  return _then(_KansoFonts(
editorFont: null == editorFont ? _self.editorFont : editorFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,previewFont: null == previewFont ? _self.previewFont : previewFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,codeFont: null == codeFont ? _self.codeFont : codeFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

// dart format on
