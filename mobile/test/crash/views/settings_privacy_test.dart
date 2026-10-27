// Kanso — settings_privacy_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/models/data/app_version.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_sync_repository.dart';
import '../../helpers/mock_repositories.dart';
import '../../helpers/pump_routed.dart';
import '../../helpers/seeded_appearance_bloc.dart';

Future<GoRouter> _pump(WidgetTester tester, {int? patch}) async {
  final about = MockAboutRepository();
  when(about.version).thenAnswer(
    (_) async => Right<ApplicationError, AppVersion>(
      AppVersion(version: '0.9.0', build: '57', patch: patch),
    ),
  );
  final aboutBloc = AboutBloc(about)..add(const AboutEvent.started());
  final syncBloc = SyncStatusBloc(FixtureSyncRepository())
    ..add(const SyncStatusEvent.started());
  final appearance = SeededAppearanceBloc(const AppearanceState());
  final library = FixtureLibraryRepository();
  addTearDown(() => unawaited(library.close()));
  final index = LibraryIndexBloc(library)
    ..add(const LibraryIndexEvent.started());
  final defaultNotebook = DefaultNotebookBloc(
    SeededPreferencesRepository(const Preferences()),
  )..add(const DefaultNotebookEvent.started());
  closeBlocLater(index);
  closeBlocLater(defaultNotebook);
  closeBlocLater(aboutBloc);
  closeBlocLater(syncBloc);
  closeBlocLater(appearance);
  final router = await pumpRouted(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<SyncStatusBloc>.value(value: syncBloc),
        BlocProvider<AboutBloc>.value(value: aboutBloc),
        BlocProvider<LibraryIndexBloc>.value(value: index),
        BlocProvider<DefaultNotebookBloc>.value(value: defaultNotebook),
      ],
      child: const SettingsPane(),
    ),
    size: const Size(393, 1400),
  );
  await tester.pump();
  await tester.pump();
  return router;
}

void main() {
  group('Settings: Privacy and the patch', () {
    testWidgets('Privacy has the Crash reports row, which opens its page', (
      tester,
    ) async {
      final router = await _pump(tester);
      expect(find.text('Privacy'), findsOneWidget);
      expect(
        find.text('Kanso sends a crash report only when you choose to.'),
        findsOneWidget,
      );
      await tester.tap(find.text('Crash reports'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/settings/crash-reports');
    });

    testWidgets('the Version row adds the patch when there is one', (
      tester,
    ) async {
      await _pump(tester, patch: 2);
      expect(find.text('0.9.0 (57), patch 2'), findsOneWidget);
    });

    testWidgets('and is as it was without one', (tester) async {
      await _pump(tester);
      expect(find.text('0.9.0 (57)'), findsOneWidget);
    });
  });
}
