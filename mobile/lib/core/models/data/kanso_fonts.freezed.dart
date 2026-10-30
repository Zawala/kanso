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

/// Menus, lists, labels and buttons.
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily get interfaceFont;/// Writing in the editor.
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily get editorFont;/// Reading a note.
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily get previewFont;/// Code spans, code blocks and the theme previews.
@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily get codeFont;/// A downloaded Google Font for the editor, when selected.
 String? get editorGoogleFont;/// A downloaded Google Font for the preview, when selected.
 String? get previewGoogleFont;/// A downloaded Google Font for the interface, when selected.
 String? get interfaceGoogleFont;/// A downloaded Google Font for code, when selected.
 String? get codeGoogleFont;/// The Google Fonts families downloaded to this device, in the order
/// they were downloaded.
 List<String> get downloadedGoogleFonts;
/// Create a copy of KansoFonts
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<KansoFonts> get copyWith => _$KansoFontsCopyWithImpl<KansoFonts>(this as KansoFonts, _$identity);

  /// Serializes this KansoFonts to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoFonts&&(identical(other.interfaceFont, interfaceFont) || other.interfaceFont == interfaceFont)&&(identical(other.editorFont, editorFont) || other.editorFont == editorFont)&&(identical(other.previewFont, previewFont) || other.previewFont == previewFont)&&(identical(other.codeFont, codeFont) || other.codeFont == codeFont)&&(identical(other.editorGoogleFont, editorGoogleFont) || other.editorGoogleFont == editorGoogleFont)&&(identical(other.previewGoogleFont, previewGoogleFont) || other.previewGoogleFont == previewGoogleFont)&&(identical(other.interfaceGoogleFont, interfaceGoogleFont) || other.interfaceGoogleFont == interfaceGoogleFont)&&(identical(other.codeGoogleFont, codeGoogleFont) || other.codeGoogleFont == codeGoogleFont)&&const DeepCollectionEquality().equals(other.downloadedGoogleFonts, downloadedGoogleFonts));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,interfaceFont,editorFont,previewFont,codeFont,editorGoogleFont,previewGoogleFont,interfaceGoogleFont,codeGoogleFont,const DeepCollectionEquality().hash(downloadedGoogleFonts));

@override
String toString() {
  return 'KansoFonts(interfaceFont: $interfaceFont, editorFont: $editorFont, previewFont: $previewFont, codeFont: $codeFont, editorGoogleFont: $editorGoogleFont, previewGoogleFont: $previewGoogleFont, interfaceGoogleFont: $interfaceGoogleFont, codeGoogleFont: $codeGoogleFont, downloadedGoogleFonts: $downloadedGoogleFonts)';
}


}

/// @nodoc
abstract mixin class $KansoFontsCopyWith<$Res>  {
  factory $KansoFontsCopyWith(KansoFonts value, $Res Function(KansoFonts) _then) = _$KansoFontsCopyWithImpl;
@useResult
$Res call({
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily interfaceFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily editorFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily previewFont,@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily codeFont, String? editorGoogleFont, String? previewGoogleFont, String? interfaceGoogleFont, String? codeGoogleFont, List<String> downloadedGoogleFonts
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
@pragma('vm:prefer-inline') @override $Res call({Object? interfaceFont = null,Object? editorFont = null,Object? previewFont = null,Object? codeFont = null,Object? editorGoogleFont = freezed,Object? previewGoogleFont = freezed,Object? interfaceGoogleFont = freezed,Object? codeGoogleFont = freezed,Object? downloadedGoogleFonts = null,}) {
  return _then(_self.copyWith(
interfaceFont: null == interfaceFont ? _self.interfaceFont : interfaceFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,editorFont: null == editorFont ? _self.editorFont : editorFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,previewFont: null == previewFont ? _self.previewFont : previewFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,codeFont: null == codeFont ? _self.codeFont : codeFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,editorGoogleFont: freezed == editorGoogleFont ? _self.editorGoogleFont : editorGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,previewGoogleFont: freezed == previewGoogleFont ? _self.previewGoogleFont : previewGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,interfaceGoogleFont: freezed == interfaceGoogleFont ? _self.interfaceGoogleFont : interfaceGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,codeGoogleFont: freezed == codeGoogleFont ? _self.codeGoogleFont : codeGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,downloadedGoogleFonts: null == downloadedGoogleFonts ? _self.downloadedGoogleFonts : downloadedGoogleFonts // ignore: cast_nullable_to_non_nullable
as List<String>,
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily interfaceFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont,  String? editorGoogleFont,  String? previewGoogleFont,  String? interfaceGoogleFont,  String? codeGoogleFont,  List<String> downloadedGoogleFonts)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
return $default(_that.interfaceFont,_that.editorFont,_that.previewFont,_that.codeFont,_that.editorGoogleFont,_that.previewGoogleFont,_that.interfaceGoogleFont,_that.codeGoogleFont,_that.downloadedGoogleFonts);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily interfaceFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont,  String? editorGoogleFont,  String? previewGoogleFont,  String? interfaceGoogleFont,  String? codeGoogleFont,  List<String> downloadedGoogleFonts)  $default,) {final _that = this;
switch (_that) {
case _KansoFonts():
return $default(_that.interfaceFont,_that.editorFont,_that.previewFont,_that.codeFont,_that.editorGoogleFont,_that.previewGoogleFont,_that.interfaceGoogleFont,_that.codeGoogleFont,_that.downloadedGoogleFonts);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily interfaceFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily editorFont, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans)  KansoFontFamily previewFont, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono)  KansoFontFamily codeFont,  String? editorGoogleFont,  String? previewGoogleFont,  String? interfaceGoogleFont,  String? codeGoogleFont,  List<String> downloadedGoogleFonts)?  $default,) {final _that = this;
switch (_that) {
case _KansoFonts() when $default != null:
return $default(_that.interfaceFont,_that.editorFont,_that.previewFont,_that.codeFont,_that.editorGoogleFont,_that.previewGoogleFont,_that.interfaceGoogleFont,_that.codeGoogleFont,_that.downloadedGoogleFonts);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _KansoFonts extends KansoFonts {
  const _KansoFonts({@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) this.interfaceFont = KansoFontFamily.instrumentSans, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) this.editorFont = KansoFontFamily.instrumentSans, @JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) this.previewFont = KansoFontFamily.instrumentSans, @JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) this.codeFont = KansoFontFamily.jetBrainsMono, this.editorGoogleFont, this.previewGoogleFont, this.interfaceGoogleFont, this.codeGoogleFont, final  List<String> downloadedGoogleFonts = const <String>[]}): _downloadedGoogleFonts = downloadedGoogleFonts,super._();
  factory _KansoFonts.fromJson(Map<String, dynamic> json) => _$KansoFontsFromJson(json);

/// Menus, lists, labels and buttons.
@override@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) final  KansoFontFamily interfaceFont;
/// Writing in the editor.
@override@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) final  KansoFontFamily editorFont;
/// Reading a note.
@override@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) final  KansoFontFamily previewFont;
/// Code spans, code blocks and the theme previews.
@override@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) final  KansoFontFamily codeFont;
/// A downloaded Google Font for the editor, when selected.
@override final  String? editorGoogleFont;
/// A downloaded Google Font for the preview, when selected.
@override final  String? previewGoogleFont;
/// A downloaded Google Font for the interface, when selected.
@override final  String? interfaceGoogleFont;
/// A downloaded Google Font for code, when selected.
@override final  String? codeGoogleFont;
/// The Google Fonts families downloaded to this device, in the order
/// they were downloaded.
 final  List<String> _downloadedGoogleFonts;
/// The Google Fonts families downloaded to this device, in the order
/// they were downloaded.
@override@JsonKey() List<String> get downloadedGoogleFonts {
  if (_downloadedGoogleFonts is EqualUnmodifiableListView) return _downloadedGoogleFonts;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_downloadedGoogleFonts);
}


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
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoFonts&&(identical(other.interfaceFont, interfaceFont) || other.interfaceFont == interfaceFont)&&(identical(other.editorFont, editorFont) || other.editorFont == editorFont)&&(identical(other.previewFont, previewFont) || other.previewFont == previewFont)&&(identical(other.codeFont, codeFont) || other.codeFont == codeFont)&&(identical(other.editorGoogleFont, editorGoogleFont) || other.editorGoogleFont == editorGoogleFont)&&(identical(other.previewGoogleFont, previewGoogleFont) || other.previewGoogleFont == previewGoogleFont)&&(identical(other.interfaceGoogleFont, interfaceGoogleFont) || other.interfaceGoogleFont == interfaceGoogleFont)&&(identical(other.codeGoogleFont, codeGoogleFont) || other.codeGoogleFont == codeGoogleFont)&&const DeepCollectionEquality().equals(other._downloadedGoogleFonts, _downloadedGoogleFonts));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,interfaceFont,editorFont,previewFont,codeFont,editorGoogleFont,previewGoogleFont,interfaceGoogleFont,codeGoogleFont,const DeepCollectionEquality().hash(_downloadedGoogleFonts));

@override
String toString() {
  return 'KansoFonts(interfaceFont: $interfaceFont, editorFont: $editorFont, previewFont: $previewFont, codeFont: $codeFont, editorGoogleFont: $editorGoogleFont, previewGoogleFont: $previewGoogleFont, interfaceGoogleFont: $interfaceGoogleFont, codeGoogleFont: $codeGoogleFont, downloadedGoogleFonts: $downloadedGoogleFonts)';
}


}

/// @nodoc
abstract mixin class _$KansoFontsCopyWith<$Res> implements $KansoFontsCopyWith<$Res> {
  factory _$KansoFontsCopyWith(_KansoFonts value, $Res Function(_KansoFonts) _then) = __$KansoFontsCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily interfaceFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily editorFont,@JsonKey(unknownEnumValue: KansoFontFamily.instrumentSans) KansoFontFamily previewFont,@JsonKey(unknownEnumValue: KansoFontFamily.jetBrainsMono) KansoFontFamily codeFont, String? editorGoogleFont, String? previewGoogleFont, String? interfaceGoogleFont, String? codeGoogleFont, List<String> downloadedGoogleFonts
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
@override @pragma('vm:prefer-inline') $Res call({Object? interfaceFont = null,Object? editorFont = null,Object? previewFont = null,Object? codeFont = null,Object? editorGoogleFont = freezed,Object? previewGoogleFont = freezed,Object? interfaceGoogleFont = freezed,Object? codeGoogleFont = freezed,Object? downloadedGoogleFonts = null,}) {
  return _then(_KansoFonts(
interfaceFont: null == interfaceFont ? _self.interfaceFont : interfaceFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,editorFont: null == editorFont ? _self.editorFont : editorFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,previewFont: null == previewFont ? _self.previewFont : previewFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,codeFont: null == codeFont ? _self.codeFont : codeFont // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,editorGoogleFont: freezed == editorGoogleFont ? _self.editorGoogleFont : editorGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,previewGoogleFont: freezed == previewGoogleFont ? _self.previewGoogleFont : previewGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,interfaceGoogleFont: freezed == interfaceGoogleFont ? _self.interfaceGoogleFont : interfaceGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,codeGoogleFont: freezed == codeGoogleFont ? _self.codeGoogleFont : codeGoogleFont // ignore: cast_nullable_to_non_nullable
as String?,downloadedGoogleFonts: null == downloadedGoogleFonts ? _self._downloadedGoogleFonts : downloadedGoogleFonts // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

// dart format on
