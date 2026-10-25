// Kanso — kanso_state_widgets_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/core/views/widgets/kanso_check_badge.dart';
import 'package:kanso/core/views/widgets/kanso_empty_state.dart';
import 'package:kanso/core/views/widgets/kanso_skeleton.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoEmptyState', () {
    testWidgets('shows its title, message and action', (tester) async {
      var pressed = 0;
      await pumpKanso(
        tester,
        KansoEmptyState(
          title: 'Nothing here',
          message: 'Add one.',
          action: KansoButton(
            label: 'Add',
            expanded: false,
            onPressed: () => pressed++,
          ),
        ),
      );
      expect(find.text('Nothing here'), findsOneWidget);
      expect(find.text('Add one.'), findsOneWidget);
      await tester.tap(find.text('Add'));
      expect(pressed, 1);
    });

    testWidgets('without an action it is only words', (tester) async {
      await pumpKanso(
        tester,
        const KansoEmptyState(title: 'Nothing', message: 'Yet.'),
      );
      expect(find.byType(KansoButton), findsNothing);
    });

    testWidgets('keeps its message to short lines', (tester) async {
      tester.view
        ..physicalSize = const Size(1000, 600)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await pumpKanso(
        tester,
        const KansoEmptyState(title: 'T', message: 'A message.'),
      );
      expect(
        tester.getSize(find.text('A message.')).width,
        lessThanOrEqualTo(KansoEmptyState.maxWidth),
      );
    });
  });

  group('KansoSkeleton', () {
    testWidgets('is a still bar in the fill colour, hidden from readers', (
      tester,
    ) async {
      await pumpKanso(tester, const KansoSkeleton(width: 80, height: 10));
      final box = tester.widget<Container>(
        find.descendant(
          of: find.byType(KansoSkeleton),
          matching: find.byType(Container),
        ),
      );
      expect(
        (box.decoration! as BoxDecoration).color,
        KansoColors.light.fill,
      );
      expect(tester.getSize(find.byType(KansoSkeleton)), const Size(80, 10));
      expect(find.byType(ExcludeSemantics), findsWidgets);
    });

    testWidgets('does not animate', (tester) async {
      await pumpKanso(tester, const KansoSkeleton(width: 80));
      expect(tester.hasRunningAnimations, isFalse);
      expect(tester.binding.hasScheduledFrame, isFalse);
    });
  });

  group('KansoCheckBadge', () {
    testWidgets('is a filled accent circle that says what it is', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        const KansoCheckBadge(semanticLabel: 'In use'),
      );
      expect(find.bySemanticsLabel('In use'), findsOneWidget);
      final box = tester.widget<Container>(
        find.descendant(
          of: find.byType(KansoCheckBadge),
          matching: find.byType(Container),
        ),
      );
      expect(
        (box.decoration! as BoxDecoration).color,
        KansoColors.light.accent,
      );
      handle.dispose();
    });
  });
}
