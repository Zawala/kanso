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
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/shell/views/widgets/library_navigation.dart';

/// The orientation the flows run in, `portrait` or `landscape`, set with
/// `--dart-define=KANSO_ORIENTATION=landscape`; the device's own if not set.
const String flowOrientation = String.fromEnvironment('KANSO_ORIENTATION');

/// Starts the app as `main` does, on a library in a new temporary folder,
/// and presses OK on the notice of the first launch.
Future<void> startApp(WidgetTester tester) async {
  await turnDevice(tester);
  await bootstrap(AppEnvironment.integration);
  await tester.pumpAndSettle();
  await pumpUntil(tester, find.text('Notes stay on this device'));
  await tester.tap(find.text('OK'));
  await tester.pumpAndSettle();
  expect(find.text('Notes stay on this device'), findsNothing);
}

/// Turns the device to [flowOrientation] and waits until the window is that
/// way round; it is turned back when the test ends. A device that cannot
/// turn (a phone's landscape is fine, an emulator with rotation locked is
/// not) fails here, not later in a flow that finds the wrong layout.
Future<void> turnDevice(WidgetTester tester) async {
  if (flowOrientation.isEmpty) {
    return;
  }
  const landscape = flowOrientation == 'landscape';
  addTearDown(
    () => SystemChrome.setPreferredOrientations(<DeviceOrientation>[]),
  );
  await SystemChrome.setPreferredOrientations(
    landscape
        ? <DeviceOrientation>[
            DeviceOrientation.landscapeLeft,
            DeviceOrientation.landscapeRight,
          ]
        : <DeviceOrientation>[
            DeviceOrientation.portraitUp,
            DeviceOrientation.portraitDown,
          ],
  );
  await pumpUntilTrue(
    tester,
    () {
      final size = tester.view.physicalSize;
      return (size.width > size.height) == landscape;
    },
    reason: 'the window to be in $flowOrientation',
  );
}

/// [label] in the list of notes, not in the library beside it or in the
/// note open beside the list.
Finder inList(String label) => find.descendant(
  of: find.byType(NoteListPane),
  matching: find.text(label),
);

/// Whether the window shows one pane at a time, so that a note is a page of
/// its own on top of the list.
bool onePane(WidgetTester tester) {
  final note = find.byType(NotePage);
  return note.evaluate().isNotEmpty && !tester.widget<NotePage>(note).inPane;
}

/// Leaves a note for its list: a page to go back from where there is one
/// pane, nothing to do where the note is beside its list.
Future<void> leaveNote(WidgetTester tester) async {
  if (onePane(tester)) {
    // The bar's chevron, not the history arrows of the bottom bar.
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
  }
}

/// Closes the library's drawer with a tap on what is left of the window
/// beside it; there is nothing to close where the library is a pane.
Future<void> closeLibrary(WidgetTester tester) async {
  if (find.bySemanticsLabel('Library').evaluate().isEmpty) {
    return;
  }
  final width = tester.view.physicalSize.width / tester.view.devicePixelRatio;
  await tester.tapAt(Offset(width - 5, 400));
  await tester.pumpAndSettle();
}

/// Opens search: the button of the list's bar, or, where the library is
/// beside the list, the field at the top of the library.
Future<void> openSearch(WidgetTester tester) async {
  final button = find.bySemanticsLabel('Search');
  if (button.evaluate().isNotEmpty) {
    await tester.tap(button);
  } else {
    await tester.tap(find.text('Search notes'));
  }
  await tester.pumpAndSettle();
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
