// Kanso — flow_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/bootstrap.dart';
import 'package:kanso/core/configs/app_environment.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/core/views/widgets/kanso_sidebar_row.dart';
import 'package:kanso/shell/views/widgets/library_navigation.dart';

/// Starts the app as `main` does, on a library in a new temporary folder,
/// and presses OK on the notice of the first launch.
Future<void> startApp(WidgetTester tester) async {
  await bootstrap(AppEnvironment.integration);
  await tester.pumpAndSettle();
  await pumpUntil(tester, find.text('Notes stay on this device'));
  await tester.tap(find.text('OK'));
  await tester.pumpAndSettle();
  expect(find.text('Notes stay on this device'), findsNothing);
}

/// Pumps until [finder] finds something, for up to [timeout].
Future<void> pumpUntil(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 20),
}) async {
  await pumpUntilTrue(
    tester,
    () => finder.evaluate().isNotEmpty,
    reason: '$finder',
    timeout: timeout,
  );
}

/// Pumps until [finder] finds nothing, for up to [timeout].
Future<void> pumpUntilGone(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 20),
}) async {
  await pumpUntilTrue(
    tester,
    () => finder.evaluate().isEmpty,
    reason: 'no $finder',
    timeout: timeout,
  );
}

/// Pumps until [done] is true, for up to [timeout]; fails with [reason].
Future<void> pumpUntilTrue(
  WidgetTester tester,
  bool Function() done, {
  required String reason,
  Duration timeout = const Duration(seconds: 20),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 100));
    if (done()) {
      await tester.pumpAndSettle();
      return;
    }
  }
  fail('Gave up waiting for $reason');
}

/// Opens the library: the drawer of a phone. A window with the library beside
/// the list has it open already.
Future<void> openLibrary(WidgetTester tester) async {
  final menu = find.bySemanticsLabel('Library');
  if (menu.evaluate().isNotEmpty) {
    await tester.tap(menu);
    await tester.pumpAndSettle();
  }
  expect(find.byType(LibraryNavigation), findsOneWidget);
}

/// [label] in the library.
Finder inLibrary(String label) => find.descendant(
  of: find.byType(LibraryNavigation),
  matching: find.text(label),
);

/// The count the library shows for its row [label].
int? libraryCount(WidgetTester tester, String label) {
  final row = find.byWidgetPredicate(
    (widget) => widget is KansoSidebarRow && widget.label == label,
  );
  return tester.widget<KansoSidebarRow>(row).count;
}

/// The lock icon of a note's row or title.
final Finder lockIcon = find.byWidgetPredicate(
  (widget) => widget is KansoIcon && widget.icon == KansoIcons.lock,
);

/// A link or address from outside the app, as the platform hands it over.
Future<void> openLink(WidgetTester tester, String link) async {
  final message = const JSONMethodCodec().encodeMethodCall(
    MethodCall('pushRouteInformation', <String, dynamic>{'location': link}),
  );
  await tester.binding.defaultBinaryMessenger.handlePlatformMessage(
    'flutter/navigation',
    message,
    (_) {},
  );
  await tester.pumpAndSettle();
}
