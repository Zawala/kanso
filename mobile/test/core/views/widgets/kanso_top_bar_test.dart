// Kanso — kanso_top_bar_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_top_bar.dart';

import '../../../helpers/kanso_test_app.dart';

/// A 44-point action labelled [label].
Widget _action(String label) => SizedBox.square(
  key: ValueKey<String>(label),
  dimension: 44,
  child: Text(label),
);

void main() {
  Future<void> pump(
    WidgetTester tester, {
    required double width,
    required int actions,
    String? title,
  }) async {
    tester.view
      ..physicalSize = Size(width, 800)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      KansoTestApp(
        home: Align(
          alignment: Alignment.topCenter,
          child: KansoTopBar(
            leading: _action('back'),
            title: title,
            trailing: <Widget>[
              for (var at = 0; at < actions; at++) _action('a$at'),
            ],
          ),
        ),
      ),
    );
  }

  group('KansoTopBar', () {
    testWidgets('more actions than fit are drawn smaller beside the '
        'leading one, all in reach, and nothing overflows', (tester) async {
      // Nine 44-point actions and the back button on a 320-point phone.
      await pump(tester, width: 320, actions: 9);
      expect(tester.takeException(), isNull);
      Rect rect(String label) =>
          tester.getRect(find.byKey(ValueKey<String>(label)));
      final back = rect('back');
      expect(back.width, 44);
      final first = rect('a0');
      final last = rect('a8');
      // Beside the back button, not over it.
      expect(first.left, greaterThanOrEqualTo(back.right - 0.01));
      expect(last.right, closeTo(316, 0.01));
      expect(first.width, lessThan(44));
    });

    testWidgets('a bar whose actions fit lays them out as before, the title '
        'centred', (tester) async {
      await pump(tester, width: 400, actions: 2, title: 'Notes');
      expect(tester.takeException(), isNull);
      final last = tester.getRect(find.byKey(const ValueKey<String>('a1')));
      final first = tester.getRect(find.byKey(const ValueKey<String>('a0')));
      // The bar's side padding is 4 points.
      expect(last.right, 396);
      expect(first.right, last.left);
      expect(tester.getCenter(find.text('Notes')).dx, closeTo(200, 0.5));
    });
  });
}
