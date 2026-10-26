// Kanso — open_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/search/views/widgets/search_result_tile.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Links from outside the app, handed to it as the platform hands them:
/// `kanso://open/note/<id>`, `kanso://open/notebook/<id>` and
/// `kanso://open/search?q=…`, and a note that is not there.
///
/// Run it with `flutter test integration_test/open_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('kanso:// links open a note, a notebook and a search', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));

    // A note.
    await openLink(
      tester,
      'kanso://open/note/${Uri.encodeComponent(seeded.sprintPlan)}',
    );
    await pumpUntil(tester, find.textContaining('Ship the beta build'));

    // A notebook: its notes, and no others.
    await openLink(
      tester,
      'kanso://open/notebook/${Uri.encodeComponent(seeded.garden)}',
    );
    await pumpUntil(tester, find.text('Garden beds'));
    expect(find.text('Seed catalogue'), findsOneWidget);
    expect(find.text('Sprint plan'), findsNothing);

    // A search, with its words in the field and its results.
    await openLink(tester, 'kanso://open/search?q=beta');
    Finder tile(String title) => find.byWidgetPredicate(
      (widget) => widget is SearchResultTile && widget.hit.row.title == title,
    );
    await pumpUntil(tester, tile('Sprint plan'));
    expect(tile('Standup notes'), findsOneWidget);

    // A note that is not there says so, and nothing is made up.
    await openLink(tester, 'kanso://open/note/note%3Anot-there');
    await pumpUntil(tester, find.text("This note isn't here"));

    // A locked note opens to its lock, not its text.
    await openLink(
      tester,
      'kanso://open/note/${Uri.encodeComponent(seeded.privateJournal)}',
    );
    await pumpUntil(tester, find.text('This note is locked'));
    expect(find.textContaining(SeededLibrary.secretBody), findsNothing);
    semantics.dispose();
  });
}
