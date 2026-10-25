// Kanso — kanso_segmented_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
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

    testWidgets('is a 44 point touch target tall', (tester) async {
      await pumpKanso(
        tester,
        KansoSegmented<int>(
          selected: 1,
          onChanged: (_) {},
          segments: const <KansoSegment<int>>[
            KansoSegment<int>(value: 1, icon: KansoIcons.eye),
            KansoSegment<int>(value: 2, icon: KansoIcons.pencil),
          ],
        ),
      );
      expect(tester.getSize(find.byType(KansoSegmented<int>)).height, 44);
      for (final cell in tester.widgetList<KansoSegmentCell<int>>(
        find.byType(KansoSegmentCell<int>),
      )) {
        expect(cell.segment.icon, isNotNull);
      }
      expect(
        tester.getSize(find.byType(KansoSegmentCell<int>).first).width,
        greaterThanOrEqualTo(44),
      );
    });

    testWidgets('names itself and its icon-only segments for readers', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        KansoSegmented<int>(
          selected: 1,
          semanticLabel: 'View mode',
          onChanged: (_) {},
          segments: const <KansoSegment<int>>[
            KansoSegment<int>(
              value: 1,
              icon: KansoIcons.eye,
              semanticLabel: 'Preview',
            ),
            KansoSegment<int>(
              value: 2,
              icon: KansoIcons.pencil,
              semanticLabel: 'Edit',
            ),
          ],
        ),
      );
      expect(find.bySemanticsLabel('View mode'), findsOneWidget);
      expect(find.bySemanticsLabel('Preview'), findsOneWidget);
      expect(find.bySemanticsLabel('Edit'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('the track is quieter than a solid fill', (tester) async {
      await pumpKanso(
        tester,
        KansoSegmented<int>(
          selected: 1,
          onChanged: (_) {},
          segments: const <KansoSegment<int>>[
            KansoSegment<int>(value: 1, label: 'A'),
            KansoSegment<int>(value: 2, label: 'B'),
          ],
        ),
      );
      final track = tester.widget<Container>(
        find
            .descendant(
              of: find.byType(KansoSegmented<int>),
              matching: find.byType(Container),
            )
            .first,
      );
      final color = (track.decoration! as BoxDecoration).color!;
      expect(color.a, lessThan(0.8));
    });
  });
}
