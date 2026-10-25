// Kanso — kanso_colors_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_syntax_colors.dart';

/// `#RRGGBB` as an opaque colour.
Color _hex(String value) {
  return Color(0xFF000000 | int.parse(value.substring(1), radix: 16));
}

Map<String, dynamic> _theme(String name) {
  final file = File('../themes/$name.json');
  return jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
}

Map<String, String> _stringMap(Object? value) {
  return (value! as Map<String, dynamic>).map(
    (key, value) => MapEntry(key, value as String),
  );
}

/// The syntax tokens as colours: a plain string, or the `color` of a styled
/// token, plus `preview.code_background`.
Map<String, String> _syntaxColors(Map<String, dynamic> theme) {
  final colors = <String, String>{};
  (theme['syntax'] as Map<String, dynamic>).forEach((key, value) {
    if (value is String) {
      colors[key] = value;
    } else if (value is Map<String, dynamic> && value['color'] is String) {
      colors[key] = value['color'] as String;
    }
  });
  colors['code_background'] =
      (theme['preview'] as Map<String, dynamic>)['code_background'] as String;
  return colors;
}

Map<String, Color> _uiTokens(KansoColors c) => <String, Color>{
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

Map<String, Color> _syntaxTokens(KansoSyntaxColors c) => <String, Color>{
  'text': c.text,
  'heading': c.heading,
  'block_marker': c.blockMarker,
  'marker': c.marker,
  'list_marker': c.listMarker,
  'task': c.task,
  'hr': c.hr,
  'link': c.link,
  'url': c.url,
  'code': c.code,
  'code_block': c.codeBlock,
  'escape': c.escape,
  'comment': c.comment,
  'comment_doc': c.commentDoc,
  'keyword': c.keyword,
  'string': c.string,
  'string_special': c.stringSpecial,
  'function': c.function,
  'type': c.typeName,
  'type_constructor': c.typeConstructor,
  'number': c.number,
  'boolean': c.boolean,
  'constant': c.constant,
  'variable': c.variable,
  'variable_special': c.variableSpecial,
  'property': c.property,
  'attribute': c.attribute,
  'tag': c.tag,
  'label': c.label,
  'operator': c.operator,
  'punctuation': c.punctuation,
  'bracket': c.bracket,
  'embedded': c.embedded,
  'code_background': c.codeBackground,
};

void _expectMatches(Map<String, Color> dart, Map<String, String> file) {
  expect(dart.keys.toSet(), file.keys.toSet(), reason: 'token names differ');
  for (final entry in file.entries) {
    expect(
      dart[entry.key],
      _hex(entry.value),
      reason: '${entry.key} differs from the theme file',
    );
  }
}

void main() {
  group('KansoColors', () {
    test('Dark matches themes/kanso-dark.json', () {
      _expectMatches(
        _uiTokens(KansoColors.dark),
        _stringMap(_theme('kanso-dark')['ui']),
      );
    });

    test('Light matches themes/kanso-light.json', () {
      _expectMatches(
        _uiTokens(KansoColors.light),
        _stringMap(_theme('kanso-light')['ui']),
      );
    });

    test('lerp at the ends gives the ends', () {
      final start = KansoColors.dark.lerp(KansoColors.light, 0);
      expect(start.editor, KansoColors.dark.editor);
      final end = KansoColors.dark.lerp(KansoColors.light, 1);
      expect(end.editor, KansoColors.light.editor);
    });

    test('copyWith replaces only the named token', () {
      final copy = KansoColors.dark.copyWith(accent: const Color(0xFF123456));
      expect(copy.accent, const Color(0xFF123456));
      expect(copy.text, KansoColors.dark.text);
    });
  });

  group('KansoSyntaxColors', () {
    test('Dark matches themes/kanso-dark.json', () {
      _expectMatches(
        _syntaxTokens(KansoSyntaxColors.dark),
        _syntaxColors(_theme('kanso-dark')),
      );
    });

    test('Light matches themes/kanso-light.json', () {
      _expectMatches(
        _syntaxTokens(KansoSyntaxColors.light),
        _syntaxColors(_theme('kanso-light')),
      );
    });
  });
}
