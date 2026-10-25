// Kanso — list_panes_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/core/views/widgets/kanso_empty_state.dart';
import 'package:kanso/core/views/widgets/kanso_skeleton.dart';
import 'package:kanso/design/models/data/design_layout.dart';
import 'package:kanso/design/views/widgets/empty_notebook_pane.dart';
import 'package:kanso/design/views/widgets/loading_list_pane.dart';
import 'package:kanso/design/views/widgets/new_note_button.dart';
import 'package:kanso/design/views/widgets/no_note_selected_pane.dart';
import 'package:kanso/design/views/widgets/note_list_pane.dart';
import 'package:kanso/design/views/widgets/skeleton_note_row.dart';

import '../../../helpers/pump.dart';

Future<void> _tall(WidgetTester tester) async {
  tester.view
    ..physicalSize = const Size(393, 852)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
}

void main() {
  group('NoteListPane', () {
    testWidgets('scrolls its last row clear of the floating button', (
      tester,
    ) async {
      await _tall(tester);
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          height: 852,
          child: NoteListPane(layout: DesignLayout.phone),
        ),
      );
      final list = tester.widget<ListView>(find.byType(ListView));
      final padding = list.padding! as EdgeInsets;
      expect(
        padding.bottom,
        greaterThanOrEqualTo(
          KansoSizes.floatingButton + 2 * 16,
        ),
      );
      await tester.drag(find.byType(ListView), const Offset(0, -5000));
      await tester.pump();
      final button = tester.getRect(find.byType(NewNoteButton));
      final lastRow = tester.getRect(find.text('Garden beds plan').last);
      expect(lastRow.bottom, lessThan(button.top));
    });

    testWidgets('a tablet has no floating button', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 340,
          height: 800,
          child: NoteListPane(layout: DesignLayout.tablet),
        ),
      );
      expect(find.byType(NewNoteButton), findsNothing);
    });
  });

  group('LoadingListPane', () {
    testWidgets('is made of still bars shaped like rows', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          height: 852,
          child: LoadingListPane(layout: DesignLayout.phone),
        ),
      );
      expect(find.byType(SkeletonNoteRow), findsWidgets);
      expect(find.byType(KansoSkeleton), findsWidgets);
      expect(tester.hasRunningAnimations, isFalse);
    });

    testWidgets('says to a screen reader that it is loading', (tester) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          height: 852,
          child: LoadingListPane(layout: DesignLayout.phone),
        ),
      );
      expect(find.bySemanticsLabel('Loading notes'), findsOneWidget);
      handle.dispose();
    });

    testWidgets('a skeleton row is about as tall as a real row', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          child: IntrinsicHeight(child: SkeletonNoteRow()),
        ),
      );
      final height = tester.getSize(find.byType(SkeletonNoteRow)).height;
      expect(height, inInclusiveRange(100, 120));
    });
  });

  group('EmptyNotebookPane', () {
    testWidgets('names the notebook and offers the next step', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 393,
          height: 852,
          child: EmptyNotebookPane(layout: DesignLayout.phone),
        ),
      );
      expect(find.text('No notes in Someday yet'), findsOneWidget);
      expect(find.text('0 notes'), findsOneWidget);
      expect(
        find.descendant(
          of: find.byType(KansoEmptyState),
          matching: find.byType(KansoButton),
        ),
        findsOneWidget,
      );
      expect(find.text('New note'), findsOneWidget);
    });
  });

  group('NoNoteSelectedPane', () {
    testWidgets('says what to do, with nothing to press', (tester) async {
      await pumpKanso(
        tester,
        const SizedBox(width: 600, height: 600, child: NoNoteSelectedPane()),
      );
      expect(find.text('No note selected'), findsOneWidget);
      expect(find.byType(KansoButton), findsNothing);
    });
  });
}
