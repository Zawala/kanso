// Kanso — store_screenshots_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Takes the pictures for the store listings (mobile/store/README.md): the
/// notes, the library, a notebook, a note, search and a locked note, on a
/// library the app seeds for itself.
///
/// It is not a check and is not part of CI. `scripts/mobile-store-screenshots.sh`
/// runs it on a simulator or emulator with `test_driver/store_screenshot_driver.dart`,
/// which writes the pictures to a folder.
void main() {
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('screens for the store listings', (tester) async {
    final semantics = tester.ensureSemantics();

    Future<void> shot(String name) async {
      await tester.pumpAndSettle();
      await binding.takeScreenshot(name);
    }

    // Android draws Flutter on a surface that a screenshot only sees once it
    // is converted to an image.
    if (Platform.isAndroid) {
      await binding.convertFlutterSurfaceToImage();
    }
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));
    await shot('01-notes');

    // The library: a drawer on a phone, beside the list on a tablet.
    await openLibrary(tester);
    await shot('02-library');

    // A notebook, and a note in it.
    await tester.tap(inLibrary('Work'));
    await pumpUntil(tester, find.text('Sprint plan'));
    await shot('03-notebook');
    await tester.tap(find.text('Sprint plan'));
    await pumpUntil(tester, find.textContaining('Ship the beta build'));
    await shot('04-note');

    // Search: the word is highlighted where it matched.
    final back = find.bySemanticsLabel('Back');
    if (back.evaluate().isNotEmpty) {
      await tester.tap(back.first);
      await tester.pumpAndSettle();
    }
    await tester.tap(find.bySemanticsLabel('Search'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(KansoSearchInput), 'garden');
    await pumpUntil(tester, find.text('Garden beds'));
    await shot('05-search');

    semantics.dispose();
  });
}
