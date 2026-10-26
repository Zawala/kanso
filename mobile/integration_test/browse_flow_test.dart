// Kanso — browse_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/views/widgets/kanso_note_row.dart';
import 'package:kanso/note_list/views/widgets/move_sheet.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Browsing as a person does, on a library of the app's own: the places and
/// their notes, a note opened and closed, notes pinned, moved and trashed,
/// and a locked note that shows its title and nothing else.
///
/// Run it with `flutter test integration_test/browse_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  /// The vertical position of the first [finder] on screen.
  double top(WidgetTester tester, Finder finder) =>
      tester.getTopLeft(finder.first).dy;

  testWidgets('browse the library, change notes, and keep a locked note shut', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();

    // All Notes: the pinned note under Pinned, the rest under Today.
    await pumpUntil(tester, find.text('Weekly review'));
    expect(find.text('Pinned'), findsOneWidget);
    expect(find.text('Today'), findsOneWidget);
    expect(
      top(tester, find.text('Pinned')),
      lessThan(
        top(tester, find.text('Weekly review')),
      ),
    );
    expect(
      top(tester, find.text('Weekly review')),
      lessThan(top(tester, find.text('Today'))),
    );

    // The locked note: its title and its lock, and none of its text.
    expect(find.text('Private journal'), findsOneWidget);
    expect(
      find.descendant(
        of: find.ancestor(
          of: find.text('Private journal'),
          matching: find.byType(KansoNoteRow),
        ),
        matching: lockIcon,
      ),
      findsOneWidget,
    );
    expect(find.textContaining(SeededLibrary.secretBody), findsNothing);

    // The library: Work lists its notes and not Garden's.
    await openLibrary(tester);
    await tester.tap(inLibrary('Work'));
    await pumpUntil(tester, find.text('Sprint plan'));
    expect(find.text('Standup notes'), findsOneWidget);
    expect(find.text('Garden beds'), findsNothing);
    expect(find.text('Weekly review'), findsNothing);

    // A note opens with its title, notebook and styled text, and closes.
    await tester.tap(find.text('Sprint plan'));
    await pumpUntil(tester, find.textContaining('Ship the beta build'));
    expect(find.text('Work'), findsWidgets);
    final heading = find.byWidgetPredicate(
      (widget) =>
          widget is RichText &&
          widget.text.toPlainText().startsWith('# Sprint plan') &&
          _hasStyledRun(widget.text),
    );
    expect(heading, findsOneWidget);
    // The bar's chevron, not the history arrows of the bottom bar.
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
    expect(find.text('Standup notes'), findsOneWidget);

    // A full swipe pins a note: it goes under Pinned, above Today.
    await tester.drag(find.text('Sprint plan'), const Offset(300, 0));
    await pumpUntil(tester, find.text('Pinned'));
    await tester.pumpAndSettle();
    expect(
      top(tester, find.text('Pinned')),
      lessThan(top(tester, find.text('Sprint plan'))),
    );
    expect(
      top(tester, find.text('Sprint plan')),
      lessThan(top(tester, find.text('Today'))),
    );
    expect(
      top(tester, find.text('Today')),
      lessThan(top(tester, find.text('Standup notes'))),
    );

    // Long press two notes and move them to Garden.
    await tester.longPress(find.text('Sprint plan'));
    await tester.pump();
    await tester.tap(find.text('Standup notes'));
    await tester.pump();
    expect(find.text('2 selected'), findsOneWidget);
    await tester.tap(find.bySemanticsLabel('Move'));
    await tester.pumpAndSettle();
    expect(find.text('Move 2 notes to'), findsOneWidget);
    await tester.tap(
      find.descendant(
        of: find.byType(MoveSheet),
        matching: find.text('Garden'),
      ),
    );
    await pumpUntil(tester, find.text('No notes in Work yet'));
    await openLibrary(tester);
    await tester.tap(inLibrary('Garden'));
    await pumpUntil(tester, find.text('Standup notes'));
    for (final title in <String>[
      'Sprint plan',
      'Standup notes',
      'Garden beds',
      'Seed catalogue',
    ]) {
      expect(find.text(title), findsOneWidget, reason: title);
    }

    // Trash one: the Trash count goes up by one.
    await openLibrary(tester);
    final trashed = libraryCount(tester, 'Trash') ?? 0;
    final width = tester.view.physicalSize.width / tester.view.devicePixelRatio;
    await tester.tapAt(Offset(width - 5, 400));
    await tester.pumpAndSettle();
    await tester.drag(find.text('Seed catalogue'), const Offset(-300, 0));
    await pumpUntilGone(tester, find.text('Seed catalogue'));
    await openLibrary(tester);
    await pumpUntilTrue(
      tester,
      () => libraryCount(tester, 'Trash') == trashed + 1,
      reason: 'the Trash count to be ${trashed + 1}',
    );
    await tester.tap(inLibrary('All Notes'));
    await pumpUntil(tester, find.text('Private journal'));

    // The locked note opens to its title, its lock and a card that says it
    // is locked, and the text is nowhere.
    await tester.tap(find.text('Private journal'));
    await pumpUntil(tester, find.text('This note is locked'));
    expect(lockIcon, findsWidgets);
    expect(find.textContaining(SeededLibrary.secretBody), findsNothing);
    expect(
      find.textContaining(SeededLibrary.secretBody, skipOffstage: false),
      findsNothing,
    );
    expect(seeded.privateJournal, isNotEmpty);
    semantics.dispose();
  });
}

/// Whether [span] has a run with a style of its own, as Rust's highlighting
/// gives a heading.
bool _hasStyledRun(InlineSpan span) {
  var styled = false;
  span.visitChildren((child) {
    if (child is TextSpan &&
        child.text != null &&
        child.style != null &&
        (child.style!.color != null || child.style!.fontWeight != null)) {
      styled = true;
    }
    return !styled;
  });
  return styled;
}
