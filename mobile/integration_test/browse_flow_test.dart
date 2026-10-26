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
    await pumpUntil(tester, inList('Weekly review'));
    expect(inList('Pinned'), findsOneWidget);
    expect(inList('Today'), findsOneWidget);
    expect(
      top(tester, inList('Pinned')),
      lessThan(
        top(tester, inList('Weekly review')),
      ),
    );
    expect(
      top(tester, inList('Weekly review')),
      lessThan(top(tester, inList('Today'))),
    );

    // The locked note: its title and its lock, and none of its text.
    expect(inList('Private journal'), findsOneWidget);
    expect(
      find.descendant(
        of: find.ancestor(
          of: inList('Private journal'),
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
    await pumpUntil(tester, inList('Sprint plan'));
    expect(inList('Standup notes'), findsOneWidget);
    expect(inList('Garden beds'), findsNothing);
    expect(inList('Weekly review'), findsNothing);

    // A note opens with its title, notebook and styled text, and closes.
    await tester.tap(inList('Sprint plan'));
    await pumpUntil(tester, find.textContaining('Ship the beta build'));
    expect(find.text('Work'), findsWidgets);
    final heading = find.byWidgetPredicate(
      (widget) =>
          widget is RichText &&
          widget.text.toPlainText().startsWith('# Sprint plan') &&
          _hasStyledRun(widget.text),
    );
    expect(heading, findsOneWidget);
    await leaveNote(tester);
    expect(inList('Standup notes'), findsOneWidget);

    // A full swipe pins a note: it goes under Pinned, above Today.
    await tester.drag(inList('Sprint plan'), const Offset(300, 0));
    await pumpUntil(tester, inList('Pinned'));
    await tester.pumpAndSettle();
    expect(
      top(tester, inList('Pinned')),
      lessThan(top(tester, inList('Sprint plan'))),
    );
    expect(
      top(tester, inList('Sprint plan')),
      lessThan(top(tester, inList('Today'))),
    );
    expect(
      top(tester, inList('Today')),
      lessThan(top(tester, inList('Standup notes'))),
    );

    // Long press two notes and move them to Garden.
    await tester.longPress(inList('Sprint plan'));
    await tester.pump();
    await tester.tap(inList('Standup notes'));
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
    await pumpUntil(tester, inList('Standup notes'));
    for (final title in <String>[
      'Sprint plan',
      'Standup notes',
      'Garden beds',
      'Seed catalogue',
    ]) {
      expect(inList(title), findsOneWidget, reason: title);
    }

    // Trash one: the Trash count goes up by one.
    await openLibrary(tester);
    final trashed = libraryCount(tester, 'Trash') ?? 0;
    await closeLibrary(tester);
    await tester.drag(inList('Seed catalogue'), const Offset(-300, 0));
    await pumpUntilGone(tester, inList('Seed catalogue'));
    await openLibrary(tester);
    await pumpUntilTrue(
      tester,
      () => libraryCount(tester, 'Trash') == trashed + 1,
      reason: 'the Trash count to be ${trashed + 1}',
    );
    await tester.tap(inLibrary('All Notes'));
    await pumpUntil(tester, inList('Private journal'));

    // The locked note opens to its title, its lock and a card that says it
    // is locked, and the text is nowhere.
    await tester.tap(inList('Private journal'));
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
