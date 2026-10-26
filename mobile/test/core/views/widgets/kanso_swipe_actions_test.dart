// Kanso — kanso_swipe_actions_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/models/data/kanso_swipe_action.dart';
import 'package:kanso/core/views/widgets/kanso_swipe_actions.dart';
import 'package:kanso/core/views/widgets/kanso_swipe_host.dart';

import '../../../helpers/pump.dart';

KansoSwipeAction _action(String label, List<String> log, Color color) {
  return KansoSwipeAction(
    icon: KansoIcons.pin,
    label: label,
    background: color,
    onPressed: () => log.add(label),
  );
}

Widget _swipeable(
  String id,
  List<String> log, {
  bool start = true,
  bool end = true,
  bool commitStart = true,
  bool commitEnd = true,
  bool enabled = true,
  VoidCallback? onArmed,
}) {
  return SizedBox(
    width: 400,
    child: KansoSwipeActions(
      id: id,
      enabled: enabled,
      startActions: start
          ? <KansoSwipeAction>[_action('Pin', log, Colors.blue)]
          : const <KansoSwipeAction>[],
      endActions: end
          ? <KansoSwipeAction>[
              _action('Move', log, Colors.orange),
              _action('Trash', log, Colors.red),
            ]
          : const <KansoSwipeAction>[],
      onCommitStart: commitStart ? () => log.add('commit start') : null,
      onCommitEnd: commitEnd ? () => log.add('commit end') : null,
      onArmed: onArmed,
      child: Container(key: Key('row-$id'), height: 80, color: Colors.white),
    ),
  );
}

double _rowLeft(WidgetTester tester, [String id = 'a']) =>
    tester.getTopLeft(find.byKey(Key('row-$id'))).dx;

void main() {
  late List<String> log;

  setUp(() => log = <String>[]);

  Future<void> pumpRow(WidgetTester tester, Widget row) async {
    await pumpKanso(tester, KansoSwipeHost(child: row));
  }

  group('KansoSwipeActions', () {
    testWidgets('a short drag goes back', (tester) async {
      await pumpRow(tester, _swipeable('a', log));
      await tester.drag(find.byKey(const Key('row-a')), const Offset(20, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(200));
      expect(find.text('Pin'), findsNothing);
      expect(log, isEmpty);
    });

    testWidgets('a drag towards the end shows the start action', (
      tester,
    ) async {
      await pumpRow(tester, _swipeable('a', log));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(100, 0));
      await tester.pumpAndSettle();
      // It stays, 72 points over, with the action to tap.
      expect(_rowLeft(tester), moreOrLessEquals(home + 72));
      expect(find.text('Pin'), findsOneWidget);
      await tester.tap(find.text('Pin'));
      await tester.pumpAndSettle();
      expect(log, <String>['Pin']);
      // A tap on an action closes the row.
      expect(_rowLeft(tester), moreOrLessEquals(home));
    });

    testWidgets('a drag the other way shows both end actions', (tester) async {
      await pumpRow(tester, _swipeable('a', log));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-100, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home - 144));
      expect(find.text('Move'), findsOneWidget);
      expect(find.text('Trash'), findsOneWidget);
      await tester.tap(find.text('Move'));
      await tester.pumpAndSettle();
      expect(log, <String>['Move']);
    });

    testWidgets('a tap on a row that is open closes it, and goes no further', (
      tester,
    ) async {
      await pumpRow(tester, _swipeable('a', log));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-100, 0));
      await tester.pumpAndSettle();
      await tester.tapAt(tester.getCenter(find.byKey(const Key('row-a'))));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home));
      expect(log, isEmpty);
    });

    testWidgets('a full swipe does the outer action and goes back', (
      tester,
    ) async {
      await pumpRow(tester, _swipeable('a', log));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-260, 0));
      await tester.pumpAndSettle();
      expect(log, <String>['commit end']);
      expect(_rowLeft(tester), moreOrLessEquals(home));

      await tester.drag(find.byKey(const Key('row-a')), const Offset(260, 0));
      await tester.pumpAndSettle();
      expect(log, <String>['commit end', 'commit start']);
    });

    testWidgets('a fling does the outer action once it has shown them', (
      tester,
    ) async {
      await pumpRow(tester, _swipeable('a', log));
      await tester.fling(
        find.byKey(const Key('row-a')),
        const Offset(-170, 0),
        2000,
      );
      await tester.pumpAndSettle();
      expect(log, <String>['commit end']);
    });

    testWidgets('a side that does not commit only shows its actions', (
      tester,
    ) async {
      await pumpRow(
        tester,
        _swipeable('a', log, commitStart: false, commitEnd: false),
      );
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-300, 0));
      await tester.pumpAndSettle();
      expect(log, isEmpty);
      expect(_rowLeft(tester), moreOrLessEquals(home - 144));
    });

    testWidgets('a side with no actions does not move', (tester) async {
      await pumpRow(tester, _swipeable('a', log, start: false));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(150, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home));
      expect(find.text('Pin'), findsNothing);
    });

    testWidgets('does nothing when it is not enabled', (tester) async {
      await pumpRow(tester, _swipeable('a', log, enabled: false));
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-150, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home));
    });

    testWidgets('has the actions on the other side in a language that reads '
        'from the right', (tester) async {
      await pumpRow(
        tester,
        Directionality(
          textDirection: TextDirection.rtl,
          child: _swipeable('a', log),
        ),
      );
      final home = _rowLeft(tester);
      // The start of the row is its right edge: towards its end is left.
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-100, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home - 72));
      expect(find.text('Pin'), findsOneWidget);
    });

    testWidgets('gives a bump when a swipe reaches the full swipe, once', (
      tester,
    ) async {
      var armed = 0;
      await pumpRow(tester, _swipeable('a', log, onArmed: () => armed++));
      final gesture = await tester.startGesture(
        tester.getCenter(find.byKey(const Key('row-a'))),
      );
      await gesture.moveBy(const Offset(-100, 0));
      expect(armed, 0);
      await gesture.moveBy(const Offset(-150, 0));
      await gesture.moveBy(const Offset(-30, 0));
      expect(armed, 1);
      // Back under the line and over it again is another bump.
      await gesture.moveBy(const Offset(200, 0));
      await gesture.moveBy(const Offset(-200, 0));
      expect(armed, 2);
      await gesture.up();
      await tester.pumpAndSettle();
    });

    testWidgets('only one row of a list has its actions shown', (
      tester,
    ) async {
      await pumpRow(
        tester,
        Column(
          children: <Widget>[_swipeable('a', log), _swipeable('b', log)],
        ),
      );
      final home = _rowLeft(tester);
      await tester.drag(find.byKey(const Key('row-a')), const Offset(-100, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester), moreOrLessEquals(home - 144));
      await tester.drag(find.byKey(const Key('row-b')), const Offset(-100, 0));
      await tester.pumpAndSettle();
      expect(_rowLeft(tester, 'b'), moreOrLessEquals(home - 144));
      expect(_rowLeft(tester), moreOrLessEquals(home));
    });

    testWidgets('offers its actions to a screen reader as actions', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpRow(tester, _swipeable('a', log));
      final node = tester.getSemantics(
        find
            .descendant(
              of: find.byType(KansoSwipeActions),
              matching: find.byType(Semantics),
            )
            .first,
      );
      expect(node.getSemanticsData().customSemanticsActionIds, hasLength(3));
      handle.dispose();
    });
  });
}
