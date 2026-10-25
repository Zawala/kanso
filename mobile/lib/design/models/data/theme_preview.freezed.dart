// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_preview.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemePreview {

 String get file; String get name; String get author; String get license; String get source; ThemeAppearance get appearance; Color get editor; Color get text; Color get accent; Color get heading; Color get marker; Color get quote; Color get link; Color get url; Color get code; Color get strong; Color get emphasis; bool get builtIn;
/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemePreviewCopyWith<ThemePreview> get copyWith => _$ThemePreviewCopyWithImpl<ThemePreview>(this as ThemePreview, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePreview&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.source, source) || other.source == source)&&(identical(other.appearance, appearance) || other.appearance == appearance)&&(identical(other.editor, editor) || other.editor == editor)&&(identical(other.text, text) || other.text == text)&&(identical(other.accent, accent) || other.accent == accent)&&(identical(other.heading, heading) || other.heading == heading)&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.quote, quote) || other.quote == quote)&&(identical(other.link, link) || other.link == link)&&(identical(other.url, url) || other.url == url)&&(identical(other.code, code) || other.code == code)&&(identical(other.strong, strong) || other.strong == strong)&&(identical(other.emphasis, emphasis) || other.emphasis == emphasis)&&(identical(other.builtIn, builtIn) || other.builtIn == builtIn));
}


@override
int get hashCode => Object.hash(runtimeType,file,name,author,license,source,appearance,editor,text,accent,heading,marker,quote,link,url,code,strong,emphasis,builtIn);

@override
String toString() {
  return 'ThemePreview(file: $file, name: $name, author: $author, license: $license, source: $source, appearance: $appearance, editor: $editor, text: $text, accent: $accent, heading: $heading, marker: $marker, quote: $quote, link: $link, url: $url, code: $code, strong: $strong, emphasis: $emphasis, builtIn: $builtIn)';
}


}

/// @nodoc
abstract mixin class $ThemePreviewCopyWith<$Res>  {
  factory $ThemePreviewCopyWith(ThemePreview value, $Res Function(ThemePreview) _then) = _$ThemePreviewCopyWithImpl;
@useResult
$Res call({
 String file, String name, String author, String license, String source, ThemeAppearance appearance, Color editor, Color text, Color accent, Color heading, Color marker, Color quote, Color link, Color url, Color code, Color strong, Color emphasis, bool builtIn
});




}
/// @nodoc
class _$ThemePreviewCopyWithImpl<$Res>
    implements $ThemePreviewCopyWith<$Res> {
  _$ThemePreviewCopyWithImpl(this._self, this._then);

  final ThemePreview _self;
  final $Res Function(ThemePreview) _then;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? file = null,Object? name = null,Object? author = null,Object? license = null,Object? source = null,Object? appearance = null,Object? editor = null,Object? text = null,Object? accent = null,Object? heading = null,Object? marker = null,Object? quote = null,Object? link = null,Object? url = null,Object? code = null,Object? strong = null,Object? emphasis = null,Object? builtIn = null,}) {
  return _then(_self.copyWith(
file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,source: null == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as String,appearance: null == appearance ? _self.appearance : appearance // ignore: cast_nullable_to_non_nullable
as ThemeAppearance,editor: null == editor ? _self.editor : editor // ignore: cast_nullable_to_non_nullable
as Color,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as Color,accent: null == accent ? _self.accent : accent // ignore: cast_nullable_to_non_nullable
as Color,heading: null == heading ? _self.heading : heading // ignore: cast_nullable_to_non_nullable
as Color,marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as Color,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as Color,link: null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as Color,url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as Color,code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as Color,strong: null == strong ? _self.strong : strong // ignore: cast_nullable_to_non_nullable
as Color,emphasis: null == emphasis ? _self.emphasis : emphasis // ignore: cast_nullable_to_non_nullable
as Color,builtIn: null == builtIn ? _self.builtIn : builtIn // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [ThemePreview].
extension ThemePreviewPatterns on ThemePreview {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemePreview value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemePreview value)  $default,){
final _that = this;
switch (_that) {
case _ThemePreview():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemePreview value)?  $default,){
final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String file,  String name,  String author,  String license,  String source,  ThemeAppearance appearance,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  bool builtIn)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
return $default(_that.file,_that.name,_that.author,_that.license,_that.source,_that.appearance,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.builtIn);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String file,  String name,  String author,  String license,  String source,  ThemeAppearance appearance,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  bool builtIn)  $default,) {final _that = this;
switch (_that) {
case _ThemePreview():
return $default(_that.file,_that.name,_that.author,_that.license,_that.source,_that.appearance,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.builtIn);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String file,  String name,  String author,  String license,  String source,  ThemeAppearance appearance,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  bool builtIn)?  $default,) {final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
return $default(_that.file,_that.name,_that.author,_that.license,_that.source,_that.appearance,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.builtIn);case _:
  return null;

}
}

}

/// @nodoc


class _ThemePreview extends ThemePreview {
  const _ThemePreview({required this.file, required this.name, required this.author, required this.license, required this.source, required this.appearance, required this.editor, required this.text, required this.accent, required this.heading, required this.marker, required this.quote, required this.link, required this.url, required this.code, required this.strong, required this.emphasis, this.builtIn = false}): super._();
  

@override final  String file;
@override final  String name;
@override final  String author;
@override final  String license;
@override final  String source;
@override final  ThemeAppearance appearance;
@override final  Color editor;
@override final  Color text;
@override final  Color accent;
@override final  Color heading;
@override final  Color marker;
@override final  Color quote;
@override final  Color link;
@override final  Color url;
@override final  Color code;
@override final  Color strong;
@override final  Color emphasis;
@override@JsonKey() final  bool builtIn;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemePreviewCopyWith<_ThemePreview> get copyWith => __$ThemePreviewCopyWithImpl<_ThemePreview>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemePreview&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.author, author) || other.author == author)&&(identical(other.license, license) || other.license == license)&&(identical(other.source, source) || other.source == source)&&(identical(other.appearance, appearance) || other.appearance == appearance)&&(identical(other.editor, editor) || other.editor == editor)&&(identical(other.text, text) || other.text == text)&&(identical(other.accent, accent) || other.accent == accent)&&(identical(other.heading, heading) || other.heading == heading)&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.quote, quote) || other.quote == quote)&&(identical(other.link, link) || other.link == link)&&(identical(other.url, url) || other.url == url)&&(identical(other.code, code) || other.code == code)&&(identical(other.strong, strong) || other.strong == strong)&&(identical(other.emphasis, emphasis) || other.emphasis == emphasis)&&(identical(other.builtIn, builtIn) || other.builtIn == builtIn));
}


@override
int get hashCode => Object.hash(runtimeType,file,name,author,license,source,appearance,editor,text,accent,heading,marker,quote,link,url,code,strong,emphasis,builtIn);

@override
String toString() {
  return 'ThemePreview(file: $file, name: $name, author: $author, license: $license, source: $source, appearance: $appearance, editor: $editor, text: $text, accent: $accent, heading: $heading, marker: $marker, quote: $quote, link: $link, url: $url, code: $code, strong: $strong, emphasis: $emphasis, builtIn: $builtIn)';
}


}

/// @nodoc
abstract mixin class _$ThemePreviewCopyWith<$Res> implements $ThemePreviewCopyWith<$Res> {
  factory _$ThemePreviewCopyWith(_ThemePreview value, $Res Function(_ThemePreview) _then) = __$ThemePreviewCopyWithImpl;
@override @useResult
$Res call({
 String file, String name, String author, String license, String source, ThemeAppearance appearance, Color editor, Color text, Color accent, Color heading, Color marker, Color quote, Color link, Color url, Color code, Color strong, Color emphasis, bool builtIn
});




}
/// @nodoc
class __$ThemePreviewCopyWithImpl<$Res>
    implements _$ThemePreviewCopyWith<$Res> {
  __$ThemePreviewCopyWithImpl(this._self, this._then);

  final _ThemePreview _self;
  final $Res Function(_ThemePreview) _then;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? file = null,Object? name = null,Object? author = null,Object? license = null,Object? source = null,Object? appearance = null,Object? editor = null,Object? text = null,Object? accent = null,Object? heading = null,Object? marker = null,Object? quote = null,Object? link = null,Object? url = null,Object? code = null,Object? strong = null,Object? emphasis = null,Object? builtIn = null,}) {
  return _then(_ThemePreview(
file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,license: null == license ? _self.license : license // ignore: cast_nullable_to_non_nullable
as String,source: null == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as String,appearance: null == appearance ? _self.appearance : appearance // ignore: cast_nullable_to_non_nullable
as ThemeAppearance,editor: null == editor ? _self.editor : editor // ignore: cast_nullable_to_non_nullable
as Color,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as Color,accent: null == accent ? _self.accent : accent // ignore: cast_nullable_to_non_nullable
as Color,heading: null == heading ? _self.heading : heading // ignore: cast_nullable_to_non_nullable
as Color,marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as Color,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as Color,link: null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as Color,url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as Color,code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as Color,strong: null == strong ? _self.strong : strong // ignore: cast_nullable_to_non_nullable
as Color,emphasis: null == emphasis ? _self.emphasis : emphasis // ignore: cast_nullable_to_non_nullable
as Color,builtIn: null == builtIn ? _self.builtIn : builtIn // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
