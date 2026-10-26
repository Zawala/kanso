// Kanso — themed_note_list_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/real_themes.dart';

void main() {
  loadRealThemes();

  // The list drawn from the Rust core's tokens of two built-in themes, to
  // show the whole path from a theme file to the screen.
  final themes = <(String, Brightness)>[
    ('nord', Brightness.dark),
    ('github-light', Brightness.light),
  ];
  for (final (id, brightness) in themes) {
    testWidgets('the list in $id', (tester) async {
      final blocs = await BrowseBlocs.start();
      final list = await blocs.listOf(const ListPlace.all());
      await atMockNow(
        () => expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            const Scaffold(
              resizeToAvoidBottomInset: false,
              body: NoteListPane(),
            ),
            list: list,
          ),
          name: 'phone_note_list_${id.replaceAll('-', '_')}',
          size: phoneSize,
          brightness: brightness,
          palette: realPalette(id),
        ),
      );
    });
  }
}
