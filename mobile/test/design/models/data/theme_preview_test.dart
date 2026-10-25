// Kanso — theme_preview_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/models/data/mock_data.dart';
import 'package:kanso/design/models/data/theme_appearance.dart';
import 'package:kanso/design/models/data/theme_preview.dart';

Color _hex(String value) {
  return Color(0xFF000000 | int.parse(value.substring(1), radix: 16));
}

/// A syntax token's colour: a plain string, or the `color` of a styled one.
String? _tokenColor(Object? token) {
  if (token is String) {
    return token;
  }
  if (token is Map<String, dynamic>) {
    return token['color'] as String?;
  }
  return null;
}

void main() {
  final all = <ThemePreview>[
    ...MockData.installedThemes,
    ...MockData.galleryThemes,
  ];

  group('ThemePreview fixtures match themes/*.json', () {
    for (final theme in all) {
      test(theme.name, () {
        final file = File('../themes/${theme.file}.json');
        final json =
            jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
        final ui = json['ui'] as Map<String, dynamic>;
        final syntax = json['syntax'] as Map<String, dynamic>;
        final text = ui['text'] as String;
        final marker = _tokenColor(syntax['marker'])!;

        expect(theme.name, json['name']);
        expect(theme.author, json['author']);
        expect(theme.license, json['license']);
        expect(theme.source, json['source']);
        expect(
          theme.appearance,
          json['appearance'] == 'light'
              ? ThemeAppearance.light
              : ThemeAppearance.dark,
        );
        expect(theme.editor, _hex(ui['editor'] as String));
        expect(theme.text, _hex(text));
        expect(theme.accent, _hex(ui['accent'] as String));
        expect(theme.heading, _hex(_tokenColor(syntax['heading'])!));
        expect(theme.marker, _hex(marker));
        expect(
          theme.quote,
          _hex(_tokenColor(syntax['quote']) ?? marker),
        );
        expect(theme.link, _hex(_tokenColor(syntax['link'])!));
        expect(theme.url, _hex(_tokenColor(syntax['url'])!));
        expect(theme.code, _hex(_tokenColor(syntax['code'])!));
        expect(
          theme.strong,
          _hex(_tokenColor(syntax['strong']) ?? text),
        );
        expect(
          theme.emphasis,
          _hex(_tokenColor(syntax['emphasis']) ?? text),
        );
      });
    }
  });

  group('ThemePreview', () {
    test('the repository is the owner and name from the source', () {
      expect(MockData.tokyoNight.repository, 'enkia/tokyo-night-vscode-theme');
      expect(MockData.kansoDark.repository, 'iamngoni/kanso');
    });

    test('Kanso Dark is installed, built in and in use', () {
      expect(MockData.installedThemes, contains(MockData.kansoDark));
      expect(MockData.kansoDark.builtIn, isTrue);
      expect(MockData.currentThemeName, MockData.kansoDark.name);
    });

    test('the gallery offers none of the installed themes', () {
      final installed = MockData.installedThemes.map((t) => t.file).toSet();
      expect(
        MockData.galleryThemes.every((t) => !installed.contains(t.file)),
        isTrue,
      );
    });
  });

  group('MockData', () {
    test('the notebooks add up to All Notes', () {
      final total = MockData.notebooks
          .where((notebook) => notebook.depth == 0)
          .fold<int>(0, (sum, notebook) => sum + notebook.count);
      expect(total, MockData.allNotesCount);
    });

    test('Projects holds its two notebooks', () {
      final projects = MockData.notebooks.firstWhere(
        (notebook) => notebook.name == 'Projects',
      );
      final inside = MockData.notebooks
          .where((notebook) => notebook.depth == 1)
          .fold<int>(0, (sum, notebook) => sum + notebook.count);
      expect(projects.count, inside);
    });
  });
}
