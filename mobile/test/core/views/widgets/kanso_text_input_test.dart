// Kanso — kanso_text_input_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';

import '../../../helpers/pump.dart';

Future<TextEditingController> _pump(
  WidgetTester tester, {
  String text = '',
  bool enabled = true,
  List<String>? changes,
  List<String>? submitted,
}) async {
  final controller = TextEditingController(text: text);
  addTearDown(controller.dispose);
  await pumpKanso(
    tester,
    SizedBox(
      width: 320,
      child: KansoTextInput(
        controller: controller,
        hint: 'Username',
        enabled: enabled,
        onChanged: changes?.add,
        onSubmitted: submitted?.add,
      ),
    ),
  );
  return controller;
}

void main() {
  group('KansoTextInput', () {
    testWidgets('shows the hint until there is text', (tester) async {
      final controller = await _pump(tester);
      expect(find.text('Username'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'ana');
      await tester.pump();
      expect(find.text('Username'), findsNothing);
      expect(controller.text, 'ana');
    });

    testWidgets('shows the text it already holds, and no hint', (tester) async {
      await _pump(tester, text: 'ana');
      expect(find.text('ana'), findsOneWidget);
      expect(find.text('Username'), findsNothing);
    });

    testWidgets('tells of every change and of the action key', (tester) async {
      final changes = <String>[];
      final submitted = <String>[];
      await _pump(tester, changes: changes, submitted: submitted);
      await tester.enterText(find.byType(EditableText), 'an');
      await tester.enterText(find.byType(EditableText), 'ana');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      expect(changes, <String>['an', 'ana']);
      expect(submitted, <String>['ana']);
    });

    testWidgets('draws the accent outline only while it has focus', (
      tester,
    ) async {
      await _pump(tester);
      Border border() =>
          ((tester
                          .widget<Container>(
                            find
                                .descendant(
                                  of: find.byType(KansoTextInput),
                                  matching: find.byType(Container),
                                )
                                .first,
                          )
                          .decoration!
                      as BoxDecoration)
                  .border!)
              as Border;
      final quiet = border().top.color;
      await tester.tap(find.byType(KansoTextInput));
      await tester.pump();
      expect(border().top.color, isNot(quiet));
      expect(border().top.width, 1.5);
    });

    testWidgets('does not take input while it is not enabled', (tester) async {
      final controller = await _pump(tester, enabled: false);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).readOnly,
        isTrue,
      );
      expect(controller.text, isEmpty);
    });

    testWidgets('is named to a screen reader by its hint', (tester) async {
      final handle = tester.ensureSemantics();
      await _pump(tester);
      expect(find.bySemanticsLabel('Username'), findsWidgets);
      handle.dispose();
    });
  });
}
