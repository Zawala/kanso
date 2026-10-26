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
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_library_repository.dart';
import 'register_page_blocs.dart';
import 'seeded_appearance_bloc.dart';

/// The app on the mock library, and what a test can look at of it.
class BrowseApp {
  BrowseApp(this.router, this.blocs, this.appearance);

  final GoRouter router;
  final BrowseBlocs blocs;
  final AppearanceBloc appearance;

  /// Where the router is, as an address.
  String get where => router.routerDelegate.currentConfiguration.uri.toString();
}

/// Pumps the whole app, as `main` builds it but over the mock library, open
/// at [at] in a window of [size], light. [library] is the library to use
/// when a test has made one.
Future<BrowseApp> pumpBrowseApp(
  WidgetTester tester,
  Size size, {
  String at = '/notes',
  AppearanceState appearance = const AppearanceState(),
  FixtureLibraryRepository? library,
}) async {
  final bloc = SeededAppearanceBloc(appearance);
  addTearDown(bloc.close);
  final blocs = await BrowseBlocs.start(library);
  registerPageBlocs(blocs.library);
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final kansoRouter = KansoRouter(
    const DesignGalleryAccess(available: false),
    LayoutClassNotifier.starting(
      KansoLayout.classFor(size),
      tabletDevice: size.shortestSide >= KansoLayout.tabletShortestSide,
    ),
  );
  final router = kansoRouter.config;
  addTearDown(router.dispose);
  router.go(at);
  await tester.pumpWidget(
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: bloc),
        BlocProvider<NavigationHistoryBloc>.value(value: blocs.history),
        BlocProvider<LibraryIndexBloc>.value(value: blocs.index),
        BlocProvider<SyncStatusBloc>.value(value: blocs.sync),
        BlocProvider<ListNavigationBloc>.value(value: blocs.listNavigation),
        BlocProvider<LibraryPaneBloc>.value(value: blocs.libraryPane),
        BlocProvider<BackupNoticeBloc>.value(value: hiddenBackupNoticeBloc()),
      ],
      child: App(router: kansoRouter),
    ),
  );
  await tester.pumpAndSettle();
  return BrowseApp(router, blocs, bloc);
}
