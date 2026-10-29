// Kanso — kanso_syntax_colors.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

/// The colours of Markdown and code syntax: the `syntax` tokens of a Kanso
/// theme file, plus `preview.code_background`.
///
/// Font weights and styles in the theme file (bold headings, italic
/// comments) are typography, so they live in `KansoTypography`.
///
/// Like `KansoColors`, a plain value the app hands down itself.
@immutable
class KansoSyntaxColors {
  const KansoSyntaxColors({
    required this.text,
    required this.heading,
    required this.blockMarker,
    required this.marker,
    required this.listMarker,
    required this.task,
    required this.hr,
    required this.link,
    required this.url,
    required this.code,
    required this.codeBlock,
    required this.escape,
    required this.comment,
    required this.commentDoc,
    required this.keyword,
    required this.string,
    required this.stringSpecial,
    required this.function,
    required this.typeName,
    required this.typeConstructor,
    required this.number,
    required this.boolean,
    required this.constant,
    required this.variable,
    required this.variableSpecial,
    required this.property,
    required this.attribute,
    required this.tag,
    required this.label,
    required this.operator,
    required this.punctuation,
    required this.bracket,
    required this.embedded,
    required this.codeBackground,
  });

  /// Kanso Dark, from `themes/kanso-dark.json`.
  static const KansoSyntaxColors dark = KansoSyntaxColors(
    text: Color(0xFFF1EFE7),
    heading: Color(0xFFF1EFE7),
    blockMarker: Color(0xFF7E7A70),
    marker: Color(0xFF7E7A70),
    listMarker: Color(0xFF7E7A70),
    task: Color(0xFF7E7A70),
    hr: Color(0xFF7E7A70),
    link: Color(0xFF869DB9),
    url: Color(0xFF7E7A70),
    code: Color(0xFFCDB892),
    codeBlock: Color(0xFFB7B3A8),
    escape: Color(0xFF7E7A70),
    comment: Color(0xFF7E7A70),
    commentDoc: Color(0xFF7E7A70),
    keyword: Color(0xFF8FB0D9),
    string: Color(0xFFA8C98A),
    stringSpecial: Color(0xFFA8C98A),
    function: Color(0xFFE6C68E),
    typeName: Color(0xFF8EC5B8),
    typeConstructor: Color(0xFF8FB0D9),
    number: Color(0xFFE0A882),
    boolean: Color(0xFFE0A882),
    constant: Color(0xFFE0A882),
    variable: Color(0xFFF1EFE7),
    variableSpecial: Color(0xFFE0A882),
    property: Color(0xFFDAD6CB),
    attribute: Color(0xFFC3A6D6),
    tag: Color(0xFF8FB0D9),
    label: Color(0xFFC3A6D6),
    operator: Color(0xFFA29E94),
    punctuation: Color(0xFFA29E94),
    bracket: Color(0xFFA29E94),
    embedded: Color(0xFFF1EFE7),
    codeBackground: Color(0xFF30302B),
  );

  /// Kanso Light, from `themes/kanso-light.json`.
  static const KansoSyntaxColors light = KansoSyntaxColors(
    text: Color(0xFF26241F),
    heading: Color(0xFF26241F),
    blockMarker: Color(0xFF857C6E),
    marker: Color(0xFF857C6E),
    listMarker: Color(0xFF857C6E),
    task: Color(0xFF857C6E),
    hr: Color(0xFF857C6E),
    link: Color(0xFF4D6684),
    url: Color(0xFF857C6E),
    code: Color(0xFF7D5B32),
    codeBlock: Color(0xFF655F55),
    escape: Color(0xFF857C6E),
    comment: Color(0xFF857C6E),
    commentDoc: Color(0xFF857C6E),
    keyword: Color(0xFF3F6496),
    string: Color(0xFF4A7A35),
    stringSpecial: Color(0xFF4A7A35),
    function: Color(0xFF8A6416),
    typeName: Color(0xFF2F7A6E),
    typeConstructor: Color(0xFF3F6496),
    number: Color(0xFFA0522D),
    boolean: Color(0xFFA0522D),
    constant: Color(0xFFA0522D),
    variable: Color(0xFF26241F),
    variableSpecial: Color(0xFFA0522D),
    property: Color(0xFF3A362F),
    attribute: Color(0xFF7A4F8F),
    tag: Color(0xFF3F6496),
    label: Color(0xFF7A4F8F),
    operator: Color(0xFF6F695E),
    punctuation: Color(0xFF6F695E),
    bracket: Color(0xFF6F695E),
    embedded: Color(0xFF26241F),
    codeBackground: Color(0xFFF4F1EA),
  );

  /// The `text` token.
  final Color text;

  /// The `heading` token.
  final Color heading;

  /// The `block_marker` token.
  final Color blockMarker;

  /// The `marker` token.
  final Color marker;

  /// The `list_marker` token.
  final Color listMarker;

  /// The `task` token.
  final Color task;

  /// The `hr` token.
  final Color hr;

  /// The `link` token.
  final Color link;

  /// The `url` token.
  final Color url;

  /// The `code` token.
  final Color code;

  /// The `code_block` token.
  final Color codeBlock;

  /// The `escape` token.
  final Color escape;

  /// The `comment` token.
  final Color comment;

  /// The `comment_doc` token.
  final Color commentDoc;

  /// The `keyword` token.
  final Color keyword;

  /// The `string` token.
  final Color string;

  /// The `string_special` token.
  final Color stringSpecial;

  /// The `function` token.
  final Color function;

  /// The `type` token.
  final Color typeName;

  /// The `type_constructor` token.
  final Color typeConstructor;

  /// The `number` token.
  final Color number;

  /// The `boolean` token.
  final Color boolean;

  /// The `constant` token.
  final Color constant;

  /// The `variable` token.
  final Color variable;

  /// The `variable_special` token.
  final Color variableSpecial;

  /// The `property` token.
  final Color property;

  /// The `attribute` token.
  final Color attribute;

  /// The `tag` token.
  final Color tag;

  /// The `label` token.
  final Color label;

  /// The `operator` token.
  final Color operator;

  /// The `punctuation` token.
  final Color punctuation;

  /// The `bracket` token.
  final Color bracket;

  /// The `embedded` token.
  final Color embedded;

  /// The `code_background` token.
  final Color codeBackground;

  /// Every colour, in the order of the fields: what equality and the hash
  /// are made of.
  List<Color> get values => <Color>[
    text,
    heading,
    blockMarker,
    marker,
    listMarker,
    task,
    hr,
    link,
    url,
    code,
    codeBlock,
    escape,
    comment,
    commentDoc,
    keyword,
    string,
    stringSpecial,
    function,
    typeName,
    typeConstructor,
    number,
    boolean,
    constant,
    variable,
    variableSpecial,
    property,
    attribute,
    tag,
    label,
    operator,
    punctuation,
    bracket,
    embedded,
    codeBackground,
  ];

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) {
      return true;
    }
    if (other is! KansoSyntaxColors) {
      return false;
    }
    final mine = values;
    final theirs = other.values;
    for (var i = 0; i < mine.length; i++) {
      if (mine[i] != theirs[i]) {
        return false;
      }
    }
    return true;
  }

  @override
  int get hashCode => Object.hashAll(values);

  KansoSyntaxColors copyWith({
    Color? text,
    Color? heading,
    Color? blockMarker,
    Color? marker,
    Color? listMarker,
    Color? task,
    Color? hr,
    Color? link,
    Color? url,
    Color? code,
    Color? codeBlock,
    Color? escape,
    Color? comment,
    Color? commentDoc,
    Color? keyword,
    Color? string,
    Color? stringSpecial,
    Color? function,
    Color? typeName,
    Color? typeConstructor,
    Color? number,
    Color? boolean,
    Color? constant,
    Color? variable,
    Color? variableSpecial,
    Color? property,
    Color? attribute,
    Color? tag,
    Color? label,
    Color? operator,
    Color? punctuation,
    Color? bracket,
    Color? embedded,
    Color? codeBackground,
  }) {
    return KansoSyntaxColors(
      text: text ?? this.text,
      heading: heading ?? this.heading,
      blockMarker: blockMarker ?? this.blockMarker,
      marker: marker ?? this.marker,
      listMarker: listMarker ?? this.listMarker,
      task: task ?? this.task,
      hr: hr ?? this.hr,
      link: link ?? this.link,
      url: url ?? this.url,
      code: code ?? this.code,
      codeBlock: codeBlock ?? this.codeBlock,
      escape: escape ?? this.escape,
      comment: comment ?? this.comment,
      commentDoc: commentDoc ?? this.commentDoc,
      keyword: keyword ?? this.keyword,
      string: string ?? this.string,
      stringSpecial: stringSpecial ?? this.stringSpecial,
      function: function ?? this.function,
      typeName: typeName ?? this.typeName,
      typeConstructor: typeConstructor ?? this.typeConstructor,
      number: number ?? this.number,
      boolean: boolean ?? this.boolean,
      constant: constant ?? this.constant,
      variable: variable ?? this.variable,
      variableSpecial: variableSpecial ?? this.variableSpecial,
      property: property ?? this.property,
      attribute: attribute ?? this.attribute,
      tag: tag ?? this.tag,
      label: label ?? this.label,
      operator: operator ?? this.operator,
      punctuation: punctuation ?? this.punctuation,
      bracket: bracket ?? this.bracket,
      embedded: embedded ?? this.embedded,
      codeBackground: codeBackground ?? this.codeBackground,
    );
  }

  KansoSyntaxColors lerp(KansoSyntaxColors other, double t) {
    return KansoSyntaxColors(
      text: Color.lerp(text, other.text, t)!,
      heading: Color.lerp(heading, other.heading, t)!,
      blockMarker: Color.lerp(blockMarker, other.blockMarker, t)!,
      marker: Color.lerp(marker, other.marker, t)!,
      listMarker: Color.lerp(listMarker, other.listMarker, t)!,
      task: Color.lerp(task, other.task, t)!,
      hr: Color.lerp(hr, other.hr, t)!,
      link: Color.lerp(link, other.link, t)!,
      url: Color.lerp(url, other.url, t)!,
      code: Color.lerp(code, other.code, t)!,
      codeBlock: Color.lerp(codeBlock, other.codeBlock, t)!,
      escape: Color.lerp(escape, other.escape, t)!,
      comment: Color.lerp(comment, other.comment, t)!,
      commentDoc: Color.lerp(commentDoc, other.commentDoc, t)!,
      keyword: Color.lerp(keyword, other.keyword, t)!,
      string: Color.lerp(string, other.string, t)!,
      stringSpecial: Color.lerp(stringSpecial, other.stringSpecial, t)!,
      function: Color.lerp(function, other.function, t)!,
      typeName: Color.lerp(typeName, other.typeName, t)!,
      typeConstructor: Color.lerp(typeConstructor, other.typeConstructor, t)!,
      number: Color.lerp(number, other.number, t)!,
      boolean: Color.lerp(boolean, other.boolean, t)!,
      constant: Color.lerp(constant, other.constant, t)!,
      variable: Color.lerp(variable, other.variable, t)!,
      variableSpecial: Color.lerp(variableSpecial, other.variableSpecial, t)!,
      property: Color.lerp(property, other.property, t)!,
      attribute: Color.lerp(attribute, other.attribute, t)!,
      tag: Color.lerp(tag, other.tag, t)!,
      label: Color.lerp(label, other.label, t)!,
      operator: Color.lerp(operator, other.operator, t)!,
      punctuation: Color.lerp(punctuation, other.punctuation, t)!,
      bracket: Color.lerp(bracket, other.bracket, t)!,
      embedded: Color.lerp(embedded, other.embedded, t)!,
      codeBackground: Color.lerp(codeBackground, other.codeBackground, t)!,
    );
  }
}
