// Kanso — kanso_bottom_sheet_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet.dart';
import 'package:kanso/core/views/widgets/kanso_bottom_sheet_route.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';

import '../../../helpers/pump.dart';

Future<void> _openSheet(
  WidgetTester tester,
  Widget content, {
  Brightness brightness = Brightness.light,
}) async {
  await pumpKanso(
    tester,
    Builder(
      builder: (context) => TextButton(
        onPressed: () => showKansoBottomSheet<void>(
          context,
          builder: (_) => content,
        ),
        child: const Text('Open sheet'),
      ),
    ),
    brightness: brightness,
  );
  await tester.tap(find.text('Open sheet'));
  await tester.pumpAndSettle();
}

void main() {
  group('KansoBottomSheet', () {
    testWidgets('drags down from the handle to dismiss', (tester) async {
      await _openSheet(tester, const Text('Sheet body'));

      expect(find.text('Sheet body'), findsOneWidget);
      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 150),
      );
      await tester.pumpAndSettle();

      expect(find.text('Sheet body'), findsNothing);
    });

    testWidgets('moves above the keyboard and keeps its content visible', (
      tester,
    ) async {
      tester.view.viewInsets = const FakeViewPadding(bottom: 840);
      addTearDown(tester.view.resetViewInsets);
      final controller = TextEditingController();
      addTearDown(controller.dispose);

      await _openSheet(
        tester,
        Padding(
          padding: const EdgeInsets.all(24),
          child: KansoTextInput(controller: controller, hint: 'A field'),
        ),
      );

      final padding = tester.widget<AnimatedPadding>(
        find.descendant(
          of: find.byType(KansoBottomSheet),
          matching: find.byType(AnimatedPadding),
        ),
      );
      expect(
        (padding.padding as EdgeInsets).bottom,
        closeTo(840 / tester.view.devicePixelRatio, 0.01),
      );
      expect(find.text('A field'), findsOneWidget);
    });

    testWidgets('releases focus when the sheet goes away', (tester) async {
      final focus = FocusNode();
      final controller = TextEditingController();
      addTearDown(focus.dispose);
      addTearDown(controller.dispose);
      await _openSheet(
        tester,
        KansoTextInput(
          controller: controller,
          focusNode: focus,
          autofocus: true,
        ),
      );
      await tester.pumpAndSettle();
      expect(focus.hasFocus, isTrue);

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 150),
      );
      await tester.pumpAndSettle();
      expect(find.byType(KansoTextInput), findsNothing);
      expect(focus.hasFocus, isFalse);
    });

    testWidgets('takes a drag from a child and springs back when short', (
      tester,
    ) async {
      await _openSheet(
        tester,
        const Padding(
          padding: EdgeInsets.all(48),
          child: Text('A tall sheet'),
        ),
      );
      final before = tester.getTopLeft(find.text('A tall sheet')).dy;

      await tester.drag(
        find.byKey(KansoBottomSheet.handleKey),
        const Offset(0, 60),
      );
      await tester.pumpAndSettle();

      expect(find.text('A tall sheet'), findsOneWidget);
      expect(
        tester.getTopLeft(find.text('A tall sheet')).dy,
        closeTo(before, 1),
      );
    });
  });
}
