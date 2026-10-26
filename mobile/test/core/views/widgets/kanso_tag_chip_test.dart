// Kanso — kanso_tag_chip_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/tag_color.dart';
import 'package:kanso/core/utils/muted_tag_color.dart';
import 'package:kanso/core/views/widgets/kanso_tag_chip.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoTagChip', () {
    testWidgets('shows the name after a hash', (tester) async {
      await pumpKanso(
        tester,
        KansoTagChip(name: 'idea', color: TagColor.teal.color),
      );
      expect(find.text('#idea'), findsOneWidget);
    });

    testWidgets('is only as wide as its text', (tester) async {
      await pumpKanso(
        tester,
        SizedBox(
          width: 300,
          child: Wrap(
            children: <Widget>[
              KansoTagChip(name: 'idea', color: TagColor.teal.color),
            ],
          ),
        ),
      );
      expect(tester.getSize(find.byType(KansoTagChip)).width, lessThan(100));
    });

    testWidgets('draws the colour calmed for the theme', (tester) async {
      await pumpKanso(
        tester,
        KansoTagChip(name: 'todo', color: TagColor.pink.color),
        brightness: Brightness.dark,
      );
      final text = tester.widget<Text>(find.text('#todo'));
      expect(
        text.style!.color,
        mutedTagColor(TagColor.pink.color, dark: true),
      );
    });
  });
}
