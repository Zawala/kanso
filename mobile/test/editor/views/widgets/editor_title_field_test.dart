// Kanso — editor_title_field_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor/views/widgets/editor_title_field.dart';

import '../../../helpers/pump.dart';

void main() {
  testWidgets('lets go of the keyboard when the tap is outside it', (
    tester,
  ) async {
    final controller = TextEditingController();
    final focus = FocusNode();
    addTearDown(controller.dispose);
    addTearDown(focus.dispose);
    await pumpKanso(
      tester,
      SizedBox(
        width: 320,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            EditorTitleField(
              controller: controller,
              focusNode: focus,
              onChanged: (_) {},
              onSubmitted: () {},
            ),
            const SizedBox(key: ValueKey<String>('outside'), height: 120),
          ],
        ),
      ),
    );

    focus.requestFocus();
    await tester.pump();
    await tester.pump();
    expect(focus.hasFocus, isTrue);

    await tester.tapAt(const Offset(4, 4));
    await tester.pump();
    expect(focus.hasFocus, isFalse);
  });

  testWidgets('keeps the keyboard when the outside gesture is a drag', (
    tester,
  ) async {
    final controller = TextEditingController();
    final focus = FocusNode();
    addTearDown(controller.dispose);
    addTearDown(focus.dispose);
    await pumpKanso(
      tester,
      SizedBox(
        width: 320,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            EditorTitleField(
              controller: controller,
              focusNode: focus,
              onChanged: (_) {},
              onSubmitted: () {},
            ),
            const SizedBox(key: ValueKey<String>('outside'), height: 120),
          ],
        ),
      ),
    );

    focus.requestFocus();
    await tester.pump();
    await tester.pump();
    expect(focus.hasFocus, isTrue);

    await tester.dragFrom(const Offset(40, 120), const Offset(0, 80));
    await tester.pump();
    expect(focus.hasFocus, isTrue);
  });
}
