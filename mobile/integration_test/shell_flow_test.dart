// Kanso — shell_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/bootstrap.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/app_environment.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/injection.dart';

/// The app as a person meets it: it starts, the home page lists the Rust
/// core row, the core check page asks the library, "New note" writes a note
/// and the library's change stream reports it back to the page.
///
/// Run it with `flutter test integration_test/shell_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('a note made on the core check page is reported back', (
    tester,
  ) async {
    await bootstrap(AppEnvironment.development);
    await tester.pumpAndSettle();

    await tester.tap(find.text('Rust core'));
    await tester.pumpAndSettle();
    await _pumpUntil(tester, find.text('Library'));
    expect(find.text('Version'), findsOneWidget);
    expect(find.text('Certificates'), findsOneWidget);
    expect(find.text('Ready'), findsOneWidget);

    // The count is the library's own, on the phone: read it, make a note,
    // see it one higher.
    final before = _countShown(tester);
    await tester.tap(find.text('New note'));
    await _pumpUntil(tester, find.text('${before + 1}'));
    await _pumpUntil(tester, find.text('1 note changed'));

    // Leave the library as it was found.
    final library = getIt<LibraryRepository>();
    final notes = await library.list(
      const ListQuery(
        place: ListPlace.all(),
        pinnedOnly: false,
        sort: SortKey.updated,
        descending: true,
      ),
    );
    final made = notes
        .map((list) => list.rows)
        .getOrElse(() => <NoteRow>[])
        .first;
    expect(made.title, startsWith('Core check '));
    final trashed = await library.trash(made.id);
    expect(trashed, const Right<ApplicationError, Unit>(unit));
  });
}

/// The number the Notes row shows.
int _countShown(WidgetTester tester) {
  final row = find.ancestor(
    of: find.text('Notes'),
    matching: find.byType(Row),
  );
  final texts = tester
      .widgetList<Text>(
        find.descendant(of: row.first, matching: find.byType(Text)),
      )
      .map((text) => text.data ?? '')
      .toList();
  return int.parse(texts.last);
}

/// Pumps until [finder] finds something, for up to 20 seconds.
Future<void> _pumpUntil(WidgetTester tester, Finder finder) async {
  final deadline = DateTime.now().add(const Duration(seconds: 20));
  while (DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 100));
    if (finder.evaluate().isNotEmpty) {
      return;
    }
  }
  expect(finder, findsWidgets);
}
