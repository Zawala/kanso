// Kanso — layout_class_notifier_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/router/layout_class_notifier.dart';

void main() {
  group('LayoutClassNotifier', () {
    testWidgets('starts from the size of the window the app opened in', (
      tester,
    ) async {
      tester.view
        ..physicalSize = const Size(1180, 820)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      expect(LayoutClassNotifier().value, LayoutClass.triple);
      tester.view.physicalSize = const Size(393, 852);
      expect(LayoutClassNotifier().value, LayoutClass.single);
    });

    test('knows a tablet from a phone by the display, not the window', () {
      expect(
        LayoutClassNotifier.starting(LayoutClass.single).tabletDevice,
        isFalse,
      );
      expect(
        LayoutClassNotifier.starting(
          LayoutClass.single,
          tabletDevice: true,
        ).tabletDevice,
        isTrue,
      );
    });

    test('takes the class of each size it is told', () {
      final notifier = LayoutClassNotifier.starting(LayoutClass.single)
        ..update(const Size(1180, 820));
      expect(notifier.value, LayoutClass.triple);
      notifier.update(const Size(820, 1180));
      expect(notifier.value, LayoutClass.double);
      notifier.update(const Size(393, 852));
      expect(notifier.value, LayoutClass.single);
    });

    test('is heard only when the class changes', () {
      final notifier = LayoutClassNotifier.starting(LayoutClass.single);
      var heard = 0;
      notifier
        ..addListener(() => heard++)
        ..update(const Size(400, 800))
        ..update(const Size(500, 900));
      expect(heard, 0);
      notifier
        ..update(const Size(1180, 820))
        ..update(const Size(1200, 800));
      expect(heard, 1);
    });
  });
}
