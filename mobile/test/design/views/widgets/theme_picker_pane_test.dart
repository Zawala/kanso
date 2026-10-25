// Kanso — theme_picker_pane_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_badge.dart';
import 'package:kanso/core/views/widgets/kanso_check_badge.dart';
import 'package:kanso/design/models/data/design_layout.dart';
import 'package:kanso/design/views/widgets/installed_theme_card.dart';
import 'package:kanso/design/views/widgets/theme_picker_pane.dart';

import '../../../helpers/pump.dart';

void main() {
  group('ThemePickerPane', () {
    testWidgets('puts the theme in use first and marks it with a check', (
      tester,
    ) async {
      tester.view
        ..physicalSize = const Size(393, 852)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          height: 852,
          child: ThemePickerPane(layout: DesignLayout.phone),
        ),
      );
      final cards = tester
          .widgetList<InstalledThemeCard>(find.byType(InstalledThemeCard))
          .toList();
      expect(cards.first.current, isTrue);
      expect(cards.where((card) => card.current), hasLength(1));
      expect(find.byType(KansoCheckBadge), findsOneWidget);
      // The check is on the first card, where it can be seen.
      expect(
        tester.getTopLeft(find.byType(KansoCheckBadge)).dx,
        lessThan(393),
      );
    });

    testWidgets('every installed card says where its theme came from', (
      tester,
    ) async {
      tester.view
        ..physicalSize = const Size(1200, 852)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await pumpKanso(
        tester,
        const SizedBox(
          width: 1200,
          height: 852,
          child: ThemePickerPane(layout: DesignLayout.tablet),
        ),
      );
      final badges = find.descendant(
        of: find.byType(InstalledThemeCard),
        matching: find.byType(KansoBadge),
      );
      final cards = find.byType(InstalledThemeCard);
      expect(badges, findsNWidgets(tester.widgetList(cards).length));
      // Aligned: every badge ends the same distance from its card's edge.
      final gaps = <double>[
        for (var i = 0; i < tester.widgetList(cards).length; i++)
          tester.getTopRight(cards.at(i)).dx -
              tester.getTopRight(badges.at(i)).dx,
      ];
      for (final gap in gaps) {
        expect(gap, closeTo(gaps.first, 0.01));
      }
    });
  });
}
