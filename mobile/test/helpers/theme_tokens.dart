// Kanso — theme_tokens.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';

int _rgb(Color color) => color.toARGB32() & 0xFFFFFF;

/// A palette for the theme [id] that is Kanso Light or Dark in every colour
/// but its name, for tests that need several different themes.
KansoPalette fakePalette(String id, {bool dark = false}) {
  final base = dark ? KansoPalette.kansoDark : KansoPalette.kansoLight;
  return base.copyWith(id: id, name: id);
}

/// The theme tokens the Rust core would send for [palette]: the inverse of
/// `paletteFromTokens`, so tests can stand in for the core without it.
ThemeTokens tokensFor(KansoPalette palette) {
  final c = palette.colors;
  final s = palette.syntax;
  final ui = <String, Color>{
    'app': c.app,
    'sidebar': c.sidebar,
    'list': c.list,
    'editor': c.editor,
    'elevated': c.elevated,
    'fill': c.fill,
    'text': c.text,
    'text_secondary': c.textSecondary,
    'text_muted': c.textMuted,
    'divider': c.divider,
    'accent': c.accent,
    'link': c.link,
    'row_active': c.rowActive,
    'row_hover': c.rowHover,
    'sidebar_active': c.sidebarActive,
    'sidebar_hover': c.sidebarHover,
    'hover': c.hover,
    'selected': c.selected,
    'active_line': c.activeLine,
    'button': c.button,
    'button_text': c.buttonText,
    'success': c.success,
    'warning': c.warning,
    'danger': c.danger,
  };
  final syntax = <String, Color>{
    'text': s.text,
    'heading': s.heading,
    'block_marker': s.blockMarker,
    'marker': s.marker,
    'list_marker': s.listMarker,
    'task': s.task,
    'hr': s.hr,
    'link': s.link,
    'url': s.url,
    'code': s.code,
    'code_block': s.codeBlock,
    'escape': s.escape,
    'comment': s.comment,
    'comment_doc': s.commentDoc,
    'keyword': s.keyword,
    'string': s.string,
    'string_special': s.stringSpecial,
    'function': s.function,
    'type': s.typeName,
    'type_constructor': s.typeConstructor,
    'number': s.number,
    'boolean': s.boolean,
    'constant': s.constant,
    'variable': s.variable,
    'variable_special': s.variableSpecial,
    'property': s.property,
    'attribute': s.attribute,
    'tag': s.tag,
    'label': s.label,
    'operator': s.operator,
    'punctuation': s.punctuation,
    'bracket': s.bracket,
    'embedded': s.embedded,
  };
  return ThemeTokens(
    name: palette.name,
    dark: palette.dark,
    ui: <ColorRow>[
      for (final entry in ui.entries)
        ColorRow(name: entry.key, rgb: _rgb(entry.value)),
    ],
    caret: _rgb(c.accent),
    selection: _rgb(c.accent),
    selectionAlpha: 77,
    syntax: <SyntaxStyleRow>[
      for (final entry in syntax.entries)
        SyntaxStyleRow(
          token: entry.key,
          color: _rgb(entry.value),
          italic: palette.syntaxStyles[entry.key]?.italic,
          weight: palette.syntaxStyles[entry.key]?.weight,
        ),
      for (final entry in palette.syntaxStyles.entries)
        if (!syntax.containsKey(entry.key))
          SyntaxStyleRow(
            token: entry.key,
            italic: entry.value.italic,
            weight: entry.value.weight,
          ),
    ],
    palette: palette.preview,
  );
}
