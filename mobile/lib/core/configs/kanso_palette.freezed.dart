// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'kanso_palette.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$KansoPalette {

/// The theme file's name without `.json`: `nord`, `kanso-light`.
 String get id;/// The name people see: `Kanso Light`.
 String get name;/// Whether the theme is a dark one.
 bool get dark;/// The interface colours.
 KansoColors get colors;/// The Markdown and code colours.
 KansoSyntaxColors get syntax;/// The syntax tokens that are italic or bold, by the theme file's token
/// name.
 Map<String, KansoSyntaxStyle> get syntaxStyles;/// The colours the preview and drawings are made in.
 PaletteDto get preview;
/// Create a copy of KansoPalette
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<KansoPalette> get copyWith => _$KansoPaletteCopyWithImpl<KansoPalette>(this as KansoPalette, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoPalette&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.colors, colors) || other.colors == colors)&&(identical(other.syntax, syntax) || other.syntax == syntax)&&const DeepCollectionEquality().equals(other.syntaxStyles, syntaxStyles)&&(identical(other.preview, preview) || other.preview == preview));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,dark,colors,syntax,const DeepCollectionEquality().hash(syntaxStyles),preview);

@override
String toString() {
  return 'KansoPalette(id: $id, name: $name, dark: $dark, colors: $colors, syntax: $syntax, syntaxStyles: $syntaxStyles, preview: $preview)';
}


}

/// @nodoc
abstract mixin class $KansoPaletteCopyWith<$Res>  {
  factory $KansoPaletteCopyWith(KansoPalette value, $Res Function(KansoPalette) _then) = _$KansoPaletteCopyWithImpl;
@useResult
$Res call({
 String id, String name, bool dark, KansoColors colors, KansoSyntaxColors syntax, Map<String, KansoSyntaxStyle> syntaxStyles, PaletteDto preview
});




}
/// @nodoc
class _$KansoPaletteCopyWithImpl<$Res>
    implements $KansoPaletteCopyWith<$Res> {
  _$KansoPaletteCopyWithImpl(this._self, this._then);

  final KansoPalette _self;
  final $Res Function(KansoPalette) _then;

/// Create a copy of KansoPalette
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? dark = null,Object? colors = null,Object? syntax = null,Object? syntaxStyles = null,Object? preview = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,colors: null == colors ? _self.colors : colors // ignore: cast_nullable_to_non_nullable
as KansoColors,syntax: null == syntax ? _self.syntax : syntax // ignore: cast_nullable_to_non_nullable
as KansoSyntaxColors,syntaxStyles: null == syntaxStyles ? _self.syntaxStyles : syntaxStyles // ignore: cast_nullable_to_non_nullable
as Map<String, KansoSyntaxStyle>,preview: null == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as PaletteDto,
  ));
}

}


/// Adds pattern-matching-related methods to [KansoPalette].
extension KansoPalettePatterns on KansoPalette {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _KansoPalette value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _KansoPalette() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _KansoPalette value)  $default,){
final _that = this;
switch (_that) {
case _KansoPalette():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _KansoPalette value)?  $default,){
final _that = this;
switch (_that) {
case _KansoPalette() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  bool dark,  KansoColors colors,  KansoSyntaxColors syntax,  Map<String, KansoSyntaxStyle> syntaxStyles,  PaletteDto preview)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _KansoPalette() when $default != null:
return $default(_that.id,_that.name,_that.dark,_that.colors,_that.syntax,_that.syntaxStyles,_that.preview);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  bool dark,  KansoColors colors,  KansoSyntaxColors syntax,  Map<String, KansoSyntaxStyle> syntaxStyles,  PaletteDto preview)  $default,) {final _that = this;
switch (_that) {
case _KansoPalette():
return $default(_that.id,_that.name,_that.dark,_that.colors,_that.syntax,_that.syntaxStyles,_that.preview);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  bool dark,  KansoColors colors,  KansoSyntaxColors syntax,  Map<String, KansoSyntaxStyle> syntaxStyles,  PaletteDto preview)?  $default,) {final _that = this;
switch (_that) {
case _KansoPalette() when $default != null:
return $default(_that.id,_that.name,_that.dark,_that.colors,_that.syntax,_that.syntaxStyles,_that.preview);case _:
  return null;

}
}

}

/// @nodoc


class _KansoPalette extends KansoPalette {
  const _KansoPalette({required this.id, required this.name, required this.dark, required this.colors, required this.syntax, required final  Map<String, KansoSyntaxStyle> syntaxStyles, required this.preview}): _syntaxStyles = syntaxStyles,super._();
  

/// The theme file's name without `.json`: `nord`, `kanso-light`.
@override final  String id;
/// The name people see: `Kanso Light`.
@override final  String name;
/// Whether the theme is a dark one.
@override final  bool dark;
/// The interface colours.
@override final  KansoColors colors;
/// The Markdown and code colours.
@override final  KansoSyntaxColors syntax;
/// The syntax tokens that are italic or bold, by the theme file's token
/// name.
 final  Map<String, KansoSyntaxStyle> _syntaxStyles;
/// The syntax tokens that are italic or bold, by the theme file's token
/// name.
@override Map<String, KansoSyntaxStyle> get syntaxStyles {
  if (_syntaxStyles is EqualUnmodifiableMapView) return _syntaxStyles;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_syntaxStyles);
}

/// The colours the preview and drawings are made in.
@override final  PaletteDto preview;

/// Create a copy of KansoPalette
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$KansoPaletteCopyWith<_KansoPalette> get copyWith => __$KansoPaletteCopyWithImpl<_KansoPalette>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _KansoPalette&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.colors, colors) || other.colors == colors)&&(identical(other.syntax, syntax) || other.syntax == syntax)&&const DeepCollectionEquality().equals(other._syntaxStyles, _syntaxStyles)&&(identical(other.preview, preview) || other.preview == preview));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,dark,colors,syntax,const DeepCollectionEquality().hash(_syntaxStyles),preview);

@override
String toString() {
  return 'KansoPalette(id: $id, name: $name, dark: $dark, colors: $colors, syntax: $syntax, syntaxStyles: $syntaxStyles, preview: $preview)';
}


}

/// @nodoc
abstract mixin class _$KansoPaletteCopyWith<$Res> implements $KansoPaletteCopyWith<$Res> {
  factory _$KansoPaletteCopyWith(_KansoPalette value, $Res Function(_KansoPalette) _then) = __$KansoPaletteCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, bool dark, KansoColors colors, KansoSyntaxColors syntax, Map<String, KansoSyntaxStyle> syntaxStyles, PaletteDto preview
});




}
/// @nodoc
class __$KansoPaletteCopyWithImpl<$Res>
    implements _$KansoPaletteCopyWith<$Res> {
  __$KansoPaletteCopyWithImpl(this._self, this._then);

  final _KansoPalette _self;
  final $Res Function(_KansoPalette) _then;

/// Create a copy of KansoPalette
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? dark = null,Object? colors = null,Object? syntax = null,Object? syntaxStyles = null,Object? preview = null,}) {
  return _then(_KansoPalette(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,colors: null == colors ? _self.colors : colors // ignore: cast_nullable_to_non_nullable
as KansoColors,syntax: null == syntax ? _self.syntax : syntax // ignore: cast_nullable_to_non_nullable
as KansoSyntaxColors,syntaxStyles: null == syntaxStyles ? _self._syntaxStyles : syntaxStyles // ignore: cast_nullable_to_non_nullable
as Map<String, KansoSyntaxStyle>,preview: null == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as PaletteDto,
  ));
}


}

// dart format on
