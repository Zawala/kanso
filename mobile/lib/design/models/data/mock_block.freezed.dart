// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mock_block.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MockBlock {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockBlock);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MockBlock()';
}


}

/// @nodoc
class $MockBlockCopyWith<$Res>  {
$MockBlockCopyWith(MockBlock _, $Res Function(MockBlock) __);
}


/// Adds pattern-matching-related methods to [MockBlock].
extension MockBlockPatterns on MockBlock {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( MockHeadingBlock value)?  heading,TResult Function( MockParagraphBlock value)?  paragraph,TResult Function( MockBulletsBlock value)?  bullets,TResult Function( MockTasksBlock value)?  tasks,TResult Function( MockQuoteBlock value)?  quote,TResult Function( MockCodeBlock value)?  code,required TResult orElse(),}){
final _that = this;
switch (_that) {
case MockHeadingBlock() when heading != null:
return heading(_that);case MockParagraphBlock() when paragraph != null:
return paragraph(_that);case MockBulletsBlock() when bullets != null:
return bullets(_that);case MockTasksBlock() when tasks != null:
return tasks(_that);case MockQuoteBlock() when quote != null:
return quote(_that);case MockCodeBlock() when code != null:
return code(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( MockHeadingBlock value)  heading,required TResult Function( MockParagraphBlock value)  paragraph,required TResult Function( MockBulletsBlock value)  bullets,required TResult Function( MockTasksBlock value)  tasks,required TResult Function( MockQuoteBlock value)  quote,required TResult Function( MockCodeBlock value)  code,}){
final _that = this;
switch (_that) {
case MockHeadingBlock():
return heading(_that);case MockParagraphBlock():
return paragraph(_that);case MockBulletsBlock():
return bullets(_that);case MockTasksBlock():
return tasks(_that);case MockQuoteBlock():
return quote(_that);case MockCodeBlock():
return code(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( MockHeadingBlock value)?  heading,TResult? Function( MockParagraphBlock value)?  paragraph,TResult? Function( MockBulletsBlock value)?  bullets,TResult? Function( MockTasksBlock value)?  tasks,TResult? Function( MockQuoteBlock value)?  quote,TResult? Function( MockCodeBlock value)?  code,}){
final _that = this;
switch (_that) {
case MockHeadingBlock() when heading != null:
return heading(_that);case MockParagraphBlock() when paragraph != null:
return paragraph(_that);case MockBulletsBlock() when bullets != null:
return bullets(_that);case MockTasksBlock() when tasks != null:
return tasks(_that);case MockQuoteBlock() when quote != null:
return quote(_that);case MockCodeBlock() when code != null:
return code(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int level,  String text)?  heading,TResult Function( String text)?  paragraph,TResult Function( List<String> items)?  bullets,TResult Function( List<MockTaskItem> items)?  tasks,TResult Function( String text)?  quote,TResult Function( String language,  List<List<MockCodeToken>> lines)?  code,required TResult orElse(),}) {final _that = this;
switch (_that) {
case MockHeadingBlock() when heading != null:
return heading(_that.level,_that.text);case MockParagraphBlock() when paragraph != null:
return paragraph(_that.text);case MockBulletsBlock() when bullets != null:
return bullets(_that.items);case MockTasksBlock() when tasks != null:
return tasks(_that.items);case MockQuoteBlock() when quote != null:
return quote(_that.text);case MockCodeBlock() when code != null:
return code(_that.language,_that.lines);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int level,  String text)  heading,required TResult Function( String text)  paragraph,required TResult Function( List<String> items)  bullets,required TResult Function( List<MockTaskItem> items)  tasks,required TResult Function( String text)  quote,required TResult Function( String language,  List<List<MockCodeToken>> lines)  code,}) {final _that = this;
switch (_that) {
case MockHeadingBlock():
return heading(_that.level,_that.text);case MockParagraphBlock():
return paragraph(_that.text);case MockBulletsBlock():
return bullets(_that.items);case MockTasksBlock():
return tasks(_that.items);case MockQuoteBlock():
return quote(_that.text);case MockCodeBlock():
return code(_that.language,_that.lines);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int level,  String text)?  heading,TResult? Function( String text)?  paragraph,TResult? Function( List<String> items)?  bullets,TResult? Function( List<MockTaskItem> items)?  tasks,TResult? Function( String text)?  quote,TResult? Function( String language,  List<List<MockCodeToken>> lines)?  code,}) {final _that = this;
switch (_that) {
case MockHeadingBlock() when heading != null:
return heading(_that.level,_that.text);case MockParagraphBlock() when paragraph != null:
return paragraph(_that.text);case MockBulletsBlock() when bullets != null:
return bullets(_that.items);case MockTasksBlock() when tasks != null:
return tasks(_that.items);case MockQuoteBlock() when quote != null:
return quote(_that.text);case MockCodeBlock() when code != null:
return code(_that.language,_that.lines);case _:
  return null;

}
}

}

/// @nodoc


class MockHeadingBlock implements MockBlock {
  const MockHeadingBlock({required this.level, required this.text});
  

 final  int level;
 final  String text;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockHeadingBlockCopyWith<MockHeadingBlock> get copyWith => _$MockHeadingBlockCopyWithImpl<MockHeadingBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockHeadingBlock&&(identical(other.level, level) || other.level == level)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,level,text);

@override
String toString() {
  return 'MockBlock.heading(level: $level, text: $text)';
}


}

/// @nodoc
abstract mixin class $MockHeadingBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockHeadingBlockCopyWith(MockHeadingBlock value, $Res Function(MockHeadingBlock) _then) = _$MockHeadingBlockCopyWithImpl;
@useResult
$Res call({
 int level, String text
});




}
/// @nodoc
class _$MockHeadingBlockCopyWithImpl<$Res>
    implements $MockHeadingBlockCopyWith<$Res> {
  _$MockHeadingBlockCopyWithImpl(this._self, this._then);

  final MockHeadingBlock _self;
  final $Res Function(MockHeadingBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? level = null,Object? text = null,}) {
  return _then(MockHeadingBlock(
level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class MockParagraphBlock implements MockBlock {
  const MockParagraphBlock(this.text);
  

 final  String text;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockParagraphBlockCopyWith<MockParagraphBlock> get copyWith => _$MockParagraphBlockCopyWithImpl<MockParagraphBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockParagraphBlock&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'MockBlock.paragraph(text: $text)';
}


}

/// @nodoc
abstract mixin class $MockParagraphBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockParagraphBlockCopyWith(MockParagraphBlock value, $Res Function(MockParagraphBlock) _then) = _$MockParagraphBlockCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$MockParagraphBlockCopyWithImpl<$Res>
    implements $MockParagraphBlockCopyWith<$Res> {
  _$MockParagraphBlockCopyWithImpl(this._self, this._then);

  final MockParagraphBlock _self;
  final $Res Function(MockParagraphBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(MockParagraphBlock(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class MockBulletsBlock implements MockBlock {
  const MockBulletsBlock(final  List<String> items): _items = items;
  

 final  List<String> _items;
 List<String> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}


/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockBulletsBlockCopyWith<MockBulletsBlock> get copyWith => _$MockBulletsBlockCopyWithImpl<MockBulletsBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockBulletsBlock&&const DeepCollectionEquality().equals(other._items, _items));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_items));

@override
String toString() {
  return 'MockBlock.bullets(items: $items)';
}


}

/// @nodoc
abstract mixin class $MockBulletsBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockBulletsBlockCopyWith(MockBulletsBlock value, $Res Function(MockBulletsBlock) _then) = _$MockBulletsBlockCopyWithImpl;
@useResult
$Res call({
 List<String> items
});




}
/// @nodoc
class _$MockBulletsBlockCopyWithImpl<$Res>
    implements $MockBulletsBlockCopyWith<$Res> {
  _$MockBulletsBlockCopyWithImpl(this._self, this._then);

  final MockBulletsBlock _self;
  final $Res Function(MockBulletsBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? items = null,}) {
  return _then(MockBulletsBlock(
null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class MockTasksBlock implements MockBlock {
  const MockTasksBlock(final  List<MockTaskItem> items): _items = items;
  

 final  List<MockTaskItem> _items;
 List<MockTaskItem> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}


/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockTasksBlockCopyWith<MockTasksBlock> get copyWith => _$MockTasksBlockCopyWithImpl<MockTasksBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockTasksBlock&&const DeepCollectionEquality().equals(other._items, _items));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_items));

@override
String toString() {
  return 'MockBlock.tasks(items: $items)';
}


}

/// @nodoc
abstract mixin class $MockTasksBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockTasksBlockCopyWith(MockTasksBlock value, $Res Function(MockTasksBlock) _then) = _$MockTasksBlockCopyWithImpl;
@useResult
$Res call({
 List<MockTaskItem> items
});




}
/// @nodoc
class _$MockTasksBlockCopyWithImpl<$Res>
    implements $MockTasksBlockCopyWith<$Res> {
  _$MockTasksBlockCopyWithImpl(this._self, this._then);

  final MockTasksBlock _self;
  final $Res Function(MockTasksBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? items = null,}) {
  return _then(MockTasksBlock(
null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<MockTaskItem>,
  ));
}


}

/// @nodoc


class MockQuoteBlock implements MockBlock {
  const MockQuoteBlock(this.text);
  

 final  String text;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockQuoteBlockCopyWith<MockQuoteBlock> get copyWith => _$MockQuoteBlockCopyWithImpl<MockQuoteBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockQuoteBlock&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'MockBlock.quote(text: $text)';
}


}

/// @nodoc
abstract mixin class $MockQuoteBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockQuoteBlockCopyWith(MockQuoteBlock value, $Res Function(MockQuoteBlock) _then) = _$MockQuoteBlockCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$MockQuoteBlockCopyWithImpl<$Res>
    implements $MockQuoteBlockCopyWith<$Res> {
  _$MockQuoteBlockCopyWithImpl(this._self, this._then);

  final MockQuoteBlock _self;
  final $Res Function(MockQuoteBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(MockQuoteBlock(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class MockCodeBlock implements MockBlock {
  const MockCodeBlock({required this.language, required final  List<List<MockCodeToken>> lines}): _lines = lines;
  

 final  String language;
 final  List<List<MockCodeToken>> _lines;
 List<List<MockCodeToken>> get lines {
  if (_lines is EqualUnmodifiableListView) return _lines;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_lines);
}


/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MockCodeBlockCopyWith<MockCodeBlock> get copyWith => _$MockCodeBlockCopyWithImpl<MockCodeBlock>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MockCodeBlock&&(identical(other.language, language) || other.language == language)&&const DeepCollectionEquality().equals(other._lines, _lines));
}


@override
int get hashCode => Object.hash(runtimeType,language,const DeepCollectionEquality().hash(_lines));

@override
String toString() {
  return 'MockBlock.code(language: $language, lines: $lines)';
}


}

/// @nodoc
abstract mixin class $MockCodeBlockCopyWith<$Res> implements $MockBlockCopyWith<$Res> {
  factory $MockCodeBlockCopyWith(MockCodeBlock value, $Res Function(MockCodeBlock) _then) = _$MockCodeBlockCopyWithImpl;
@useResult
$Res call({
 String language, List<List<MockCodeToken>> lines
});




}
/// @nodoc
class _$MockCodeBlockCopyWithImpl<$Res>
    implements $MockCodeBlockCopyWith<$Res> {
  _$MockCodeBlockCopyWithImpl(this._self, this._then);

  final MockCodeBlock _self;
  final $Res Function(MockCodeBlock) _then;

/// Create a copy of MockBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? language = null,Object? lines = null,}) {
  return _then(MockCodeBlock(
language: null == language ? _self.language : language // ignore: cast_nullable_to_non_nullable
as String,lines: null == lines ? _self._lines : lines // ignore: cast_nullable_to_non_nullable
as List<List<MockCodeToken>>,
  ));
}


}

// dart format on
