// Kanso — highlighted_text_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/configs/kanso_effects.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';

import '../../../helpers/pump.dart';

Iterable<BoxDecoration> _washes(WidgetTester tester) {
  return tester
      .widgetList<DecoratedBox>(
        find.descendant(
          of: find.byType(HighlightedText),
          matching: find.byType(DecoratedBox),
        ),
      )
      .map((box) => box.decoration)
      .whereType<BoxDecoration>();
}

void main() {
  group('HighlightedText', () {
    testWidgets('draws each match on a rounded accent wash', (tester) async {
      await pumpKanso(
        tester,
        const HighlightedText(
          'Sync and sync again',
          ranges: <Range16>[
            Range16(start: 0, end: 4),
            Range16(start: 9, end: 13),
          ],
          style: TextStyle(fontSize: 14),
        ),
      );
      final washes = _washes(tester).toList();
      expect(washes, hasLength(2));
      for (final wash in washes) {
        expect(
          wash.color,
          KansoColors.light.accent.withValues(alpha: KansoEffects.highlight),
        );
        final radius = wash.borderRadius! as BorderRadius;
        expect(radius.topLeft.x, greaterThan(0));
      }
    });

    testWidgets('the wash is the accent at low opacity in dark too', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const HighlightedText(
          'sync',
          ranges: <Range16>[Range16(start: 0, end: 4)],
          style: TextStyle(fontSize: 14),
        ),
        brightness: Brightness.dark,
      );
      final wash = _washes(tester).single;
      expect(wash.color!.a, lessThan(0.5));
      expect(wash.color!.a, greaterThan(0.1));
    });

    testWidgets('no match, no wash, and the text is all there', (tester) async {
      await pumpKanso(
        tester,
        const HighlightedText(
          'Nothing to see',
          ranges: <Range16>[],
          style: TextStyle(fontSize: 14),
        ),
      );
      expect(_washes(tester), isEmpty);
      expect(find.textContaining('Nothing to see'), findsOneWidget);
    });

    testWidgets('a match inside a word does not open a gap in it', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const HighlightedText(
          'Syncing',
          ranges: <Range16>[Range16(start: 0, end: 4)],
          style: TextStyle(fontSize: 14),
        ),
      );
      final matched = tester.getRect(
        find
            .descendant(
              of: find.byType(HighlightedText),
              matching: find.byType(DecoratedBox),
            )
            .first,
      );
      final rest = tester.getTopLeft(find.byType(HighlightedText)).dx;
      expect(matched.left, closeTo(rest, 0.5));
    });
  });
}
