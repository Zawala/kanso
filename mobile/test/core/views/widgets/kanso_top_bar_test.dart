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
    TextDirection direction = TextDirection.ltr,
  }) async {
    tester.view
      ..physicalSize = Size(width, 800)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      KansoTestApp(
        home: Align(
          alignment: Alignment.topCenter,
          child: Directionality(
            textDirection: direction,
            child: KansoTopBar(
              leading: _action('back'),
              title: title,
              // The first and the last stay; the others give way, the
              // second first, then the third, and so on.
              trailing: <Widget>[
                for (var at = 0; at < actions; at++)
                  if (at == 0 || at == actions - 1)
                    _action('a$at')
                  else
                    KansoTopBarItem(
                      id: 'a$at',
                      priority: at,
                      child: _action('a$at'),
                    ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  group('KansoTopBar', () {
    testWidgets('actions that do not fit beside the leading one give way, '
        'the lowest priority first, and the rest stay whole', (tester) async {
      // Nine 44-point actions and the back button on a 320-point phone:
      // room for six beside it.
      await pump(tester, width: 320, actions: 9);
      expect(tester.takeException(), isNull);
      final shown = <String>[
        for (var at = 0; at < 9; at++)
          if (find
              .byKey(ValueKey<String>('a$at'))
              .hitTestable()
              .evaluate()
              .isNotEmpty)
            'a$at',
      ];
      expect(shown, <String>['a0', 'a4', 'a5', 'a6', 'a7', 'a8']);
      Rect rect(String label) =>
          tester.getRect(find.byKey(ValueKey<String>(label)));
      final back = rect('back');
      for (final label in shown) {
        expect(rect(label).width, 44);
      }
      expect(rect('a0').left, greaterThanOrEqualTo(back.right - 0.01));
      expect(rect('a8').right, 316);
      expect(rect('a0').right, rect('a4').left);
      // What gave way is named for a menu, and is not read out.
      final context = tester.element(find.byKey(const ValueKey<String>('a8')));
      expect(KansoTopBar.hiddenOf(context), <Object>{'a1', 'a2', 'a3'});
      expect(find.bySemanticsLabel('a1'), findsNothing);
      expect(find.bySemanticsLabel('a4'), findsOneWidget);
    });

    testWidgets('right to left, the actions run from the left edge', (
      tester,
    ) async {
      await pump(
        tester,
        width: 320,
        actions: 9,
        direction: TextDirection.rtl,
      );
      expect(tester.takeException(), isNull);
      Rect rect(String label) =>
          tester.getRect(find.byKey(ValueKey<String>(label)));
      expect(rect('back').right, 316);
      expect(rect('a8').left, 4);
      expect(rect('a0').right, lessThanOrEqualTo(rect('back').left + 0.01));
      expect(rect('a4').right, rect('a0').left);
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
      final context = tester.element(find.byKey(const ValueKey<String>('a1')));
      expect(KansoTopBar.hiddenOf(context), isEmpty);
    });
  });
}
