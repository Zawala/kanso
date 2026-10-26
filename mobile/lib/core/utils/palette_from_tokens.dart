// Kanso — palette_from_tokens.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import '../bridge/api/types.dart';
import '../configs/kanso_colors.dart';
import '../configs/kanso_palette.dart';
import '../configs/kanso_syntax_colors.dart';
import '../models/data/kanso_syntax_style.dart';

/// The palette of the theme [id] drawn from its fitted [tokens]: the one
/// place the Rust core's theme tokens become the app's colours.
///
/// The tokens are what the desktop draws for the theme, so the palette is
/// too. A token the rows leave out (the core always sends them) keeps
/// Kanso's own colour for a light or dark theme; a syntax token with no
/// colour is drawn in the syntax's plain `text` colour, as the engine's
/// fold of styles does.
KansoPalette paletteFromTokens(String id, ThemeTokens tokens) {
  final base = tokens.dark ? KansoColors.dark : KansoColors.light;
  final baseSyntax = tokens.dark
      ? KansoSyntaxColors.dark
      : KansoSyntaxColors.light;
  final ui = <String, int>{for (final row in tokens.ui) row.name: row.rgb};
  Color pick(String name, Color fallback) {
    final rgb = ui[name];
    return rgb == null ? fallback : opaqueColor(rgb);
  }

  final colors = KansoColors(
    app: pick('app', base.app),
    sidebar: pick('sidebar', base.sidebar),
    list: pick('list', base.list),
    editor: pick('editor', base.editor),
    elevated: pick('elevated', base.elevated),
    fill: pick('fill', base.fill),
    text: pick('text', base.text),
    textSecondary: pick('text_secondary', base.textSecondary),
    textMuted: pick('text_muted', base.textMuted),
    divider: pick('divider', base.divider),
    accent: pick('accent', base.accent),
    link: pick('link', base.link),
    rowActive: pick('row_active', base.rowActive),
    rowHover: pick('row_hover', base.rowHover),
    sidebarActive: pick('sidebar_active', base.sidebarActive),
    sidebarHover: pick('sidebar_hover', base.sidebarHover),
    hover: pick('hover', base.hover),
    selected: pick('selected', base.selected),
    activeLine: pick('active_line', base.activeLine),
    button: pick('button', base.button),
    buttonText: pick('button_text', base.buttonText),
    success: pick('success', base.success),
    warning: pick('warning', base.warning),
    danger: pick('danger', base.danger),
  );

  final rows = <String, SyntaxStyleRow>{
    for (final row in tokens.syntax) row.token: row,
  };
  final plain = rows['text']?.color;
  final text = plain == null ? baseSyntax.text : opaqueColor(plain);
  Color token(String name) {
    final rgb = rows[name]?.color;
    return rgb == null ? text : opaqueColor(rgb);
  }

  final syntax = KansoSyntaxColors(
    text: text,
    heading: token('heading'),
    blockMarker: token('block_marker'),
    marker: token('marker'),
    listMarker: token('list_marker'),
    task: token('task'),
    hr: token('hr'),
    link: token('link'),
    url: token('url'),
    code: token('code'),
    codeBlock: token('code_block'),
    escape: token('escape'),
    comment: token('comment'),
    commentDoc: token('comment_doc'),
    keyword: token('keyword'),
    string: token('string'),
    stringSpecial: token('string_special'),
    function: token('function'),
    typeName: token('type'),
    typeConstructor: token('type_constructor'),
    number: token('number'),
    boolean: token('boolean'),
    constant: token('constant'),
    variable: token('variable'),
    variableSpecial: token('variable_special'),
    property: token('property'),
    attribute: token('attribute'),
    tag: token('tag'),
    label: token('label'),
    operator: token('operator'),
    punctuation: token('punctuation'),
    bracket: token('bracket'),
    embedded: token('embedded'),
    codeBackground: opaqueColor(tokens.palette.surface),
  );

  return KansoPalette(
    id: id,
    name: tokens.name,
    dark: tokens.dark,
    colors: colors,
    syntax: syntax,
    syntaxStyles: <String, KansoSyntaxStyle>{
      for (final row in tokens.syntax)
        if (row.italic != null || row.weight != null)
          row.token: KansoSyntaxStyle(italic: row.italic, weight: row.weight),
    },
    preview: tokens.palette,
  );
}

/// The opaque colour of the 24-bit [rgb] value a theme token holds.
Color opaqueColor(int rgb) => Color(0xFF000000 | rgb);
