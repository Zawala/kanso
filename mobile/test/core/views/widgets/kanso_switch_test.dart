// Kanso — kanso_switch_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';

import '../../../helpers/pump.dart';

Color _trackColor(WidgetTester tester) {
  final container = tester.widget<AnimatedContainer>(
    find.descendant(
      of: find.byType(KansoSwitch),
      matching: find.byType(AnimatedContainer),
    ),
  );
  return (container.decoration! as BoxDecoration).color!;
}

void main() {
  group('KansoSwitch', () {
    testWidgets('shows on in the button colour and off in a quiet one', (
      tester,
    ) async {
      await pumpKanso(tester, const KansoSwitch(value: true));
      expect(_trackColor(tester), KansoColors.light.button);
      await pumpKanso(tester, const KansoSwitch(value: false));
      expect(_trackColor(tester), KansoColors.light.selected);
    });

    testWidgets('puts the knob at the end when on and the start when off', (
      tester,
    ) async {
      await pumpKanso(tester, const KansoSwitch(value: true));
      var container = tester.widget<AnimatedContainer>(
        find.byType(AnimatedContainer),
      );
      expect(container.alignment, Alignment.centerRight);
      await pumpKanso(tester, const KansoSwitch(value: false));
      container = tester.widget<AnimatedContainer>(
        find.byType(AnimatedContainer),
      );
      expect(container.alignment, Alignment.centerLeft);
    });

    testWidgets('a tap asks for the opposite value', (tester) async {
      bool? asked;
      await pumpKanso(
        tester,
        KansoSwitch(value: true, onChanged: (value) => asked = value),
      );
      await tester.tap(find.byType(KansoSwitch));
      expect(asked, isFalse);
    });

    testWidgets('without a handler a tap does nothing', (tester) async {
      await pumpKanso(tester, const KansoSwitch(value: true));
      await tester.tap(find.byType(KansoSwitch));
      expect(tester.takeException(), isNull);
    });
  });
}
