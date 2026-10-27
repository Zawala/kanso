// Kanso — pump_browse_app.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/app.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/design_gallery_access.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/core/router/layout_class_notifier.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/list_navigation/list_navigation_bloc.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_gallery_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_shared_notes_repository.dart';
import 'register_page_blocs.dart';
import 'seeded_appearance_bloc.dart';

/// The boundary around the app that a picture is taken of.
const Key appBoundaryKey = ValueKey<String>('app-boundary');

/// The app on the mock library, and what a test can look at of it.
class BrowseApp {
  BrowseApp(this.router, this.blocs, this.appearance, this.layoutClass);

  final GoRouter router;
  final BrowseBlocs blocs;
  final AppearanceBloc appearance;

  /// The layout class the router lays its pages out for.
  final LayoutClassNotifier layoutClass;

  /// Where the router is, as an address.
  String get where => router.routerDelegate.currentConfiguration.uri.toString();
}

/// Pumps the whole app, as `main` builds it but over the mock library, open
/// at [at] in a window of [size], light. [library] is the library to use
/// when a test has made one.
///
/// [devicePixelRatio] and [safeArea] (in logical pixels) are the window's
/// density and the status bar and home indicator the platform reports.
/// [tabletDevice] says whether the display is a tablet's, which a narrow
/// window of one (a Split View half) is not enough to tell; it defaults to
/// what the window's own size says. [backupNotice] is the notice's BLoC; it
/// defaults to one that is not shown. [shared] and [gallery] are the shared
/// notes and the theme gallery the app reads, if a test has made them. The
/// app is inside a repaint boundary,
/// [appBoundaryKey], to take a picture of.
Future<BrowseApp> pumpBrowseApp(
  WidgetTester tester,
  Size size, {
  String at = '/notes',
  AppearanceState appearance = const AppearanceState(),
  FixtureLibraryRepository? library,
  double devicePixelRatio = 1,
  EdgeInsets safeArea = EdgeInsets.zero,
  bool? tabletDevice,
  BackupNoticeBloc? backupNotice,
  FixtureSharedNotesRepository? shared,
  FixtureGalleryRepository? gallery,
}) async {
  final bloc = SeededAppearanceBloc(appearance);
  addTearDown(bloc.close);
  final blocs = await BrowseBlocs.start(library, null, shared);
  registerPageBlocs(blocs.library, null, gallery, blocs.shared);
  final padding = FakeViewPadding(
    left: safeArea.left * devicePixelRatio,
    top: safeArea.top * devicePixelRatio,
    right: safeArea.right * devicePixelRatio,
    bottom: safeArea.bottom * devicePixelRatio,
  );
  tester.view
    ..devicePixelRatio = devicePixelRatio
    ..physicalSize = size * devicePixelRatio
    ..padding = padding
    ..viewPadding = padding;
  addTearDown(tester.view.reset);
  final kansoRouter = KansoRouter(
    const DesignGalleryAccess(available: false),
    LayoutClassNotifier.starting(
      KansoLayout.classFor(size),
      tabletDevice:
          tabletDevice ?? size.shortestSide >= KansoLayout.tabletShortestSide,
    ),
  );
  final router = kansoRouter.config;
  addTearDown(router.dispose);
  router.go(at);
  await tester.pumpWidget(
    RepaintBoundary(
      key: appBoundaryKey,
      child: MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<AppearanceBloc>.value(value: bloc),
          BlocProvider<NavigationHistoryBloc>.value(value: blocs.history),
          BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
          BlocProvider<InvitationsBloc>.value(value: blocs.invitations),
          BlocProvider<SyncStatusBloc>.value(value: blocs.sync),
          BlocProvider<ListNavigationBloc>.value(value: blocs.listNavigation),
          BlocProvider<LibraryPaneBloc>.value(value: blocs.libraryPane),
          BlocProvider<BackupNoticeBloc>.value(
            value: backupNotice ?? hiddenBackupNoticeBloc(),
          ),
        ],
        child: App(router: kansoRouter),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return BrowseApp(router, blocs, bloc, kansoRouter.layoutClass);
}
