// Kanso — kanso_segmented_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_segment.dart';
import 'package:kanso/core/views/widgets/kanso_segment_cell.dart';
import 'package:kanso/core/views/widgets/kanso_segmented.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoSegmented', () {
    testWidgets('shows every label and reports a tap on another one', (
      tester,
    ) async {
      String? chosen;
      await pumpKanso(
        tester,
        KansoSegmented<String>(
          selected: 'a',
          onChanged: (value) => chosen = value,
          segments: const <KansoSegment<String>>[
            KansoSegment<String>(value: 'a', label: 'One'),
            KansoSegment<String>(value: 'b', label: 'Two'),
          ],
        ),
      );
      expect(find.text('One'), findsOneWidget);
      expect(find.text('Two'), findsOneWidget);
      await tester.tap(find.text('Two'));
      expect(chosen, 'b');
    });

    testWidgets('expanded gives each segment the same width', (tester) async {
      await pumpKanso(
        tester,
        KansoSegmented<int>(
          expanded: true,
          selected: 1,
          onChanged: (_) {},
          segments: const <KansoSegment<int>>[
            KansoSegment<int>(value: 1, label: 'Short'),
            KansoSegment<int>(value: 2, label: 'A much longer one'),
          ],
        ),
      );
      final cells = find.byType(KansoSegmentCell<int>);
      expect(
        tester.getSize(cells.at(0)).width,
        tester.getSize(cells.at(1)).width,
      );
    });
  });
}
