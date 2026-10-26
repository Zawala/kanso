// Kanso — kanso_controls_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/models/data/note_status.dart';
import 'package:kanso/core/models/data/tag_color.dart';
import 'package:kanso/core/views/widgets/kanso_checkbox.dart';
import 'package:kanso/core/views/widgets/kanso_chip.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/core/views/widgets/kanso_message_card.dart';
import 'package:kanso/core/views/widgets/kanso_note_row.dart';
import 'package:kanso/core/views/widgets/kanso_search_field.dart';
import 'package:kanso/core/views/widgets/kanso_sidebar_row.dart';
import 'package:kanso/core/views/widgets/kanso_tag_chip.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoCheckbox', () {
    testWidgets('shows a check when on and none when off', (tester) async {
      await pumpKanso(tester, const KansoCheckbox(value: true));
      expect(find.byType(KansoIcon), findsOneWidget);
      await pumpKanso(tester, const KansoCheckbox(value: false));
      expect(find.byType(KansoIcon), findsNothing);
    });

    testWidgets('a tap asks for the opposite value', (tester) async {
      bool? asked;
      await pumpKanso(
        tester,
        KansoCheckbox(value: false, onChanged: (value) => asked = value),
      );
      await tester.tap(find.byType(KansoCheckbox));
      expect(asked, isTrue);
    });
  });

  group('KansoChip', () {
    testWidgets('reports a tap', (tester) async {
      var taps = 0;
      await pumpKanso(tester, KansoChip(label: 'All', onTap: () => taps++));
      await tester.tap(find.text('All'));
      expect(taps, 1);
    });
  });

  group('KansoIconButton', () {
    testWidgets('has a 44 point target and a semantic label', (tester) async {
      await pumpKanso(
        tester,
        KansoIconButton(
          icon: KansoIcons.search,
          semanticLabel: 'Search',
          onPressed: () {},
        ),
      );
      final size = tester.getSize(find.byType(KansoIconButton));
      expect(size.width, greaterThanOrEqualTo(44));
      expect(size.height, greaterThanOrEqualTo(44));
      expect(find.bySemanticsLabel('Search'), findsOneWidget);
    });
  });

  group('KansoSearchField', () {
    testWidgets('shows the hint when empty and the query when typed', (
      tester,
    ) async {
      await pumpKanso(tester, const KansoSearchField(hint: 'Search notes'));
      expect(find.text('Search notes'), findsOneWidget);
      await pumpKanso(
        tester,
        const KansoSearchField(hint: 'Search notes', query: 'sync'),
      );
      expect(find.text('sync'), findsOneWidget);
      expect(find.text('Search notes'), findsNothing);
    });

    testWidgets('the clear button needs a query and a handler', (tester) async {
      var cleared = 0;
      await pumpKanso(
        tester,
        KansoSearchField(
          hint: 'Search notes',
          query: 'sync',
          onClear: () => cleared++,
        ),
      );
      expect(find.byType(KansoIcon), findsNWidgets(2));
      await tester.tap(find.byType(KansoIcon).last);
      expect(cleared, 1);
    });
  });

  group('KansoSidebarRow', () {
    testWidgets('shows its label, count and chevron', (tester) async {
      await pumpKanso(
        tester,
        const KansoSidebarRow(
          icon: KansoIcons.notebook,
          label: 'Journal',
          count: 48,
          chevron: true,
        ),
      );
      expect(find.text('Journal'), findsOneWidget);
      expect(find.text('48'), findsOneWidget);
      expect(find.byType(KansoIcon), findsNWidgets(2));
    });

    testWidgets('is 44 points tall (a touch target)', (tester) async {
      await pumpKanso(
        tester,
        const KansoSidebarRow(label: 'Journal', icon: KansoIcons.notebook),
      );
      expect(
        tester
            .getSize(
              find.descendant(
                of: find.byType(KansoSidebarRow),
                matching: find.byType(Container),
              ),
            )
            .height,
        KansoSizes.touchTarget,
      );
    });
  });

  group('KansoChip', () {
    testWidgets('looks 32 points tall and reacts over 44', (tester) async {
      var taps = 0;
      await pumpKanso(
        tester,
        KansoChip(label: 'Titles', onTap: () => taps++),
      );
      expect(tester.getSize(find.byType(KansoChip)).height, 44);
      expect(
        tester
            .getSize(
              find.descendant(
                of: find.byType(KansoChip),
                matching: find.byType(Container),
              ),
            )
            .height,
        KansoSizes.chip,
      );
      // A touch above the pill, inside the target, still lands.
      final center = tester.getCenter(find.byType(KansoChip));
      await tester.tapAt(center - const Offset(0, 19));
      expect(taps, 1);
    });
  });

  group('KansoNoteRow', () {
    testWidgets('shows title, excerpt, notebook, tags and time', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        SizedBox(
          width: 361,
          child: KansoNoteRow(
            title: 'Sync design notes',
            excerpt: 'Every device keeps a full copy.',
            notebook: 'Projects / Kanso',
            time: '09:41',
            status: NoteStatus.active,
            tags: <KansoTagChip>[
              KansoTagChip(name: 'idea', color: TagColor.teal.color),
            ],
            pinned: true,
          ),
        ),
      );
      expect(find.text('Sync design notes'), findsOneWidget);
      expect(find.text('Every device keeps a full copy.'), findsOneWidget);
      expect(find.text('Projects / Kanso'), findsOneWidget);
      expect(find.text('#idea'), findsOneWidget);
      expect(find.text('09:41'), findsOneWidget);
    });

    testWidgets('keeps its time at the end of the line', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 361,
          child: KansoNoteRow(
            title: 'T',
            excerpt: 'E',
            notebook: 'Journal',
            time: 'Mon',
          ),
        ),
      );
      final row = tester.getTopRight(find.byType(KansoNoteRow)).dx;
      expect(tester.getTopRight(find.text('Mon')).dx, closeTo(row - 16, 0.5));
    });

    testWidgets('a narrow row does not overflow', (tester) async {
      await pumpKanso(
        tester,
        SizedBox(
          width: 250,
          child: KansoNoteRow(
            title: 'A very long title that cannot possibly fit on one line',
            excerpt: 'E',
            notebook: 'Projects / Kanso / Something',
            time: '09:41',
            status: NoteStatus.onHold,
            locked: true,
            pinned: true,
            tags: <KansoTagChip>[
              KansoTagChip(name: 'idea', color: TagColor.teal.color),
            ],
          ),
        ),
      );
      expect(tester.takeException(), isNull);
    });
  });

  group('KansoMessageCard', () {
    testWidgets('shows its title, message and actions', (tester) async {
      await pumpKanso(
        tester,
        const KansoMessageCard(
          title: 'This note is locked',
          message: 'Unlock it to read.',
          actions: <Widget>[Text('Unlock')],
        ),
      );
      expect(find.text('This note is locked'), findsOneWidget);
      expect(find.text('Unlock it to read.'), findsOneWidget);
      expect(find.text('Unlock'), findsOneWidget);
    });
  });
}
