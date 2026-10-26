// Kanso — theme_parity_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';
import 'dart:ui';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_syntax_colors.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_renderer_repository.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';

import '../../helpers/core_library.dart';

/// `#RRGGBB` or `#RRGGBBAA` as an opaque colour of the first three bytes.
Color _hex(String value) {
  return Color(0xFF000000 | int.parse(value.substring(1, 7), radix: 16));
}

Map<String, dynamic> _object(Object? value) => value! as Map<String, dynamic>;

Map<String, Color> _uiColors(KansoColors c) => <String, Color>{
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

Map<String, Color> _syntaxColors(KansoSyntaxColors c) => <String, Color>{
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
};

/// Checks every colour, italic and weight of [palette] against what the
/// desktop records for the theme: [golden] is its entry of
/// `themes.golden.json`.
void _expectDrawsAsDesktop(KansoPalette palette, Map<String, dynamic> golden) {
  final id = palette.id;
  expect(palette.name, golden['name'], reason: '$id: name');
  expect(palette.dark, golden['dark'], reason: '$id: dark');

  final colors = _object(golden['colors']);
  _uiColors(palette.colors).forEach((key, color) {
    expect(color, _hex(colors[key] as String), reason: '$id: ui $key');
  });

  final own = _object(golden['own_panes']);
  final syntax = _object(own['syntax']);
  final plain = _hex(_object(syntax['text'])['color'] as String);
  _syntaxColors(palette.syntax).forEach((key, color) {
    final entry = _object(syntax[key]);
    final expected = entry['color'] == null
        ? plain
        : _hex(entry['color'] as String);
    expect(color, expected, reason: '$id: syntax $key');
  });
  expect(
    palette.syntax.codeBackground,
    _hex(_object(own['preview'])['code_background'] as String),
    reason: '$id: code background',
  );

  // Italic and weight, token by token, against the file's font_style and
  // font_weight.
  syntax.forEach((key, value) {
    final entry = _object(value);
    final style = palette.syntaxStyles[key];
    final italic = switch (entry['font_style']) {
      'italic' => true,
      'normal' => false,
      _ => null,
    };
    expect(style?.italic, italic, reason: '$id: italic of $key');
    expect(
      style?.weight,
      entry['font_weight'] as int?,
      reason: '$id: weight of $key',
    );
  });
  expect(
    palette.syntaxStyles.keys.toSet().difference(syntax.keys.toSet()),
    isEmpty,
    reason: '$id: a style for a token the theme file does not have',
  );
}

void main() {
  late Map<String, Map<String, dynamic>> golden;
  late Set<String> indexIds;
  late RendererRepository renderer;
  late List<BuiltinTheme> themes;

  setUpAll(() async {
    await initCoreForTests();
    golden = <String, Map<String, dynamic>>{
      for (final entry in _object(
        _object(
          jsonDecode(
            File(
              '../crates/kanso-app/src/themes.golden.json',
            ).readAsStringSync(),
          ),
        )['themes'],
      ).entries)
        entry.key: _object(entry.value),
    };
    final index = _object(
      jsonDecode(File('../themes/index.json').readAsStringSync()),
    );
    indexIds = <String>{
      for (final file in index['themes'] as List<dynamic>)
        (_object(file)['file'] as String).replaceAll('.json', ''),
    };
    final library = await openTestLibrary();
    renderer = RustRendererRepository(library.renderer());
    themes = (await renderer.builtinThemes()).getOrElse(
      () => throw StateError('The built-in themes did not load.'),
    );
  });

  group('built-in themes', () {
    test('the desktop, index.json and the core list the same themes', () {
      final ids = themes.map((theme) => theme.id).toSet();
      expect(ids, golden.keys.toSet());
      expect(ids, indexIds);
      expect(themes, hasLength(greaterThan(70)));
    });

    test('every theme is drawn as the desktop draws it', () {
      for (final theme in themes) {
        _expectDrawsAsDesktop(
          paletteFromTokens(theme.id, theme.tokens),
          golden[theme.id]!,
        );
      }
    });

    test(
      'a theme asked for alone gives the same palette as in the list',
      () async {
        for (final theme in themes.take(6)) {
          final alone = await renderer.builtinThemeTokens(theme.id);
          expect(
            alone.map((tokens) => paletteFromTokens(theme.id, tokens)),
            Right<ApplicationError, KansoPalette>(
              paletteFromTokens(theme.id, theme.tokens),
            ),
          );
        }
      },
    );
  });

  group('KansoPalette', () {
    test('Kanso Light is the mapping of its tokens', () {
      final tokens = themes.firstWhere((t) => t.id == 'kanso-light').tokens;
      expect(KansoPalette.kansoLight, paletteFromTokens('kanso-light', tokens));
      _expectDrawsAsDesktop(KansoPalette.kansoLight, golden['kanso-light']!);
    });

    test('Kanso Dark is the mapping of its tokens', () {
      final tokens = themes.firstWhere((t) => t.id == 'kanso-dark').tokens;
      expect(KansoPalette.kansoDark, paletteFromTokens('kanso-dark', tokens));
      _expectDrawsAsDesktop(KansoPalette.kansoDark, golden['kanso-dark']!);
    });

    test('palettes compare by value', () {
      final tokens = themes.firstWhere((t) => t.id == 'nord').tokens;
      expect(
        paletteFromTokens('nord', tokens),
        paletteFromTokens('nord', tokens),
      );
      expect(
        paletteFromTokens('nord', tokens).hashCode,
        paletteFromTokens('nord', tokens).hashCode,
      );
      expect(
        paletteFromTokens('nord', tokens),
        isNot(KansoPalette.kansoDark),
      );
    });
  });
}
