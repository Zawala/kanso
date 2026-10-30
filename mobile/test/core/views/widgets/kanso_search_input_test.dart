// Kanso — kanso_search_input_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';
import 'package:kanso/core/views/widgets/kanso_selection_controls.dart';
import 'package:kanso/core/views/widgets/kanso_text_context_menu.dart';

import '../../../helpers/pump.dart';

Future<List<String>> _pump(
  WidgetTester tester, {
  String initialText = '',
  bool autofocus = false,
  VoidCallback? onCleared,
}) async {
  final changes = <String>[];
  await pumpKanso(
    tester,
    SizedBox(
      width: 320,
      child: KansoSearchInput(
        hint: 'Search notes',
        initialText: initialText,
        autofocus: autofocus,
        onChanged: changes.add,
        onCleared: onCleared,
        clearLabel: 'Clear search',
      ),
    ),
  );
  return changes;
}

BoxDecoration _decoration(WidgetTester tester) {
  final box = tester.widget<Container>(
    find
        .descendant(
          of: find.byType(KansoSearchInput),
          matching: find.byType(Container),
        )
        .first,
  );
  return box.decoration! as BoxDecoration;
}

void main() {
  group('KansoSearchInput', () {
    testWidgets('shows the hint until there is text', (tester) async {
      await _pump(tester);
      expect(find.text('Search notes'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'tea');
      await tester.pump();
      expect(find.text('Search notes'), findsNothing);
    });

    testWidgets('starts with the text it is given, and no hint', (
      tester,
    ) async {
      await _pump(tester, initialText: 'sync');
      expect(find.text('sync'), findsOneWidget);
      expect(find.text('Search notes'), findsNothing);
    });

    testWidgets('tells of every change to the text', (tester) async {
      final changes = await _pump(tester);
      await tester.enterText(find.byType(EditableText), 'te');
      await tester.enterText(find.byType(EditableText), 'tea');
      expect(changes, <String>['te', 'tea']);
    });

    testWidgets('has a clear button only while there is text', (tester) async {
      final handle = tester.ensureSemantics();
      await _pump(tester);
      expect(find.bySemanticsLabel('Clear search'), findsNothing);
      await tester.enterText(find.byType(EditableText), 'tea');
      await tester.pump();
      expect(find.bySemanticsLabel('Clear search'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('clearing empties the field, keeps focus and tells of it', (
      tester,
    ) async {
      var cleared = 0;
      final changes = await _pump(tester, onCleared: () => cleared++);
      await tester.enterText(find.byType(EditableText), 'tea');
      await tester.pump();
      await tester.tapAt(
        tester.getCenter(find.byType(KansoSearchInput)) + const Offset(150, 0),
      );
      await tester.pump();
      expect(cleared, 1);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        isEmpty,
      );
      expect(find.text('Search notes'), findsOneWidget);
      expect(
        tester
            .widget<EditableText>(find.byType(EditableText))
            .focusNode
            .hasFocus,
        isTrue,
      );
      // With no `onCleared` the change to nothing is what it says.
      expect(changes, <String>['tea']);
    });

    testWidgets('with no onCleared, clearing is a change to nothing', (
      tester,
    ) async {
      final changes = await _pump(tester, initialText: 'tea');
      await tester.tapAt(
        tester.getCenter(find.byType(KansoSearchInput)) + const Offset(150, 0),
      );
      await tester.pump();
      expect(changes, <String>['']);
    });

    testWidgets('is outlined in the accent colour while it has focus', (
      tester,
    ) async {
      await _pump(tester);
      final resting = _decoration(tester).border! as Border;
      await tester.tap(find.byType(EditableText));
      await tester.pump();
      final focused = _decoration(tester).border! as Border;
      expect(focused.top.width, greaterThan(resting.top.width));
      expect(focused.top.color, isNot(resting.top.color));
    });

    testWidgets('autofocus puts the cursor in it', (tester) async {
      await _pump(tester, autofocus: true);
      expect(
        tester
            .widget<EditableText>(find.byType(EditableText))
            .focusNode
            .hasFocus,
        isTrue,
      );
    });

    testWidgets('lets go of the keyboard when the tap is outside it', (
      tester,
    ) async {
      await _pump(tester);
      await tester.enterText(find.byType(EditableText), 'tea');
      await tester.pump();
      final field = tester.widget<EditableText>(find.byType(EditableText));
      expect(field.focusNode.hasFocus, isTrue);

      await tester.tapAt(const Offset(4, 4));
      await tester.pump();
      expect(field.focusNode.hasFocus, isFalse);
    });

    testWidgets('draws the same caret on every platform, with no handles or '
        'magnifier', (tester) async {
      for (final platform in <TargetPlatform>[
        TargetPlatform.iOS,
        TargetPlatform.android,
      ]) {
        debugDefaultTargetPlatformOverride = platform;
        await _pump(tester, initialText: 'tea');
        final field = tester.widget<EditableText>(find.byType(EditableText));
        expect(field.cursorWidth, 2);
        expect(field.cursorRadius, const Radius.circular(1));
        expect(field.cursorOpacityAnimates, isFalse);
        expect(field.selectionControls, isA<KansoSelectionControls>());
        expect(
          field.magnifierConfiguration,
          TextMagnifierConfiguration.disabled,
        );
        await tester.pumpWidget(const SizedBox.shrink());
        debugDefaultTargetPlatformOverride = null;
      }
    });

    testWidgets('a long press selects a word, and Kanso draws the menu', (
      tester,
    ) async {
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async => switch (call.method) {
          'Clipboard.getData' => <String, dynamic>{'text': 'pasted'},
          'Clipboard.hasStrings' => <String, dynamic>{'value': true},
          _ => null,
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      await _pump(tester, initialText: 'hello world');
      await tester.longPressAt(
        tester.getRect(find.byType(EditableText)).centerLeft +
            const Offset(20, 0),
      );
      await tester.pumpAndSettle();
      expect(find.byType(KansoTextContextMenu), findsOneWidget);
      expect(find.text('Cut'), findsOneWidget);
      expect(find.text('Copy'), findsOneWidget);
      expect(find.text('Paste'), findsOneWidget);
      expect(find.text('Select All'), findsOneWidget);
      // Nothing of the platform's own menus.
      expect(find.byType(AdaptiveTextSelectionToolbar), findsNothing);
      expect(find.byType(TextSelectionToolbar), findsNothing);

      await tester.tap(find.text('Select All'));
      await tester.pump();
      final field = tester.widget<EditableText>(find.byType(EditableText));
      expect(field.controller.selection.start, 0);
      expect(field.controller.selection.end, 'hello world'.length);
    });
  });
}
