// Kanso — settings_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/reminders/state/reminders_settings/reminders_settings_bloc.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/templates/state/daily_template/daily_template_bloc.dart';

import 'close_bloc_later.dart';
import 'fixture_library_repository.dart';
import 'fixture_reminders_repository.dart';
import 'page_fixtures.dart';
import 'seeded_appearance_bloc.dart';
import 'fixture_about_repository.dart';
import 'fixture_sync_repository.dart';

/// The BLoCs the settings screens read, read: the version of the app
/// (0.1.0, build 3) and a sync status, by default no account. The
/// appearance is the golden helper's.
class SettingsBlocs {
  SettingsBlocs._(
    this.about,
    this.sync,
    this.dailyTemplate,
    this.reminders,
    this.fixtures,
  );

  final AboutBloc about;
  final SyncStatusBloc sync;
  final DailyTemplateBloc dailyTemplate;
  final RemindersSettingsBloc reminders;

  /// What the daily template and the reminders read.
  final PageFixtures fixtures;

  /// Makes them and waits until they have read.
  static Future<SettingsBlocs> start({
    SyncStatus status = signedOutStatus,
    PageFixtures? fixtures,
  }) async {
    final page = fixtures ?? PageFixtures();
    final about = AboutBloc(FixtureAboutRepository())
      ..add(const AboutEvent.started());
    closeBlocLater(about);
    final sync = SyncStatusBloc(FixtureSyncRepository(current: status))
      ..add(const SyncStatusEvent.started());
    closeBlocLater(sync);
    await about.stream.firstWhere((state) => state.status == LoadStatus.ready);
    if (sync.state.status == null) {
      await sync.stream.firstWhere((state) => state.status != null);
    }
    final dailyTemplate = DailyTemplateBloc(page.journal)
      ..add(const DailyTemplateEvent.started());
    closeBlocLater(dailyTemplate);
    final library = FixtureLibraryRepository();
    addTearDown(library.close);
    final reminders = RemindersSettingsBloc(
      ForgetfulPreferencesRepository(),
      page.notifier,
      RemindersCoordinator(
        ForgetfulPreferencesRepository(),
        FixtureRemindersRepository(),
        page.notifier,
        library,
      ),
    )..add(const RemindersSettingsEvent.started());
    closeBlocLater(reminders);
    await dailyTemplate.stream.firstWhere(
      (state) => state.status == LoadStatus.ready,
    );
    await reminders.stream.firstWhere(
      (state) => state.status == LoadStatus.ready,
    );
    return SettingsBlocs._(about, sync, dailyTemplate, reminders, page);
  }

  /// Providers of them, to put above a screen, including one that is a
  /// route pushed later.
  List<BlocProvider<dynamic>> get providers => <BlocProvider<dynamic>>[
    BlocProvider<AboutBloc>.value(value: about),
    BlocProvider<SyncStatusBloc>.value(value: sync),
    BlocProvider<DailyTemplateBloc>.value(value: dailyTemplate),
    BlocProvider<RemindersSettingsBloc>.value(value: reminders),
  ];

  /// [child] with them above it.
  Widget provide(Widget child) =>
      MultiBlocProvider(providers: providers, child: child);
}
