// Kanso — register_page_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/board/state/board/board_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/state/calendar/calendar_bloc.dart';
import 'package:kanso/journal/state/journal/journal_bloc.dart';
import 'package:kanso/reminders/state/reminders_settings/reminders_settings_bloc.dart';
import 'package:kanso/reminders/utils/reminders_coordinator.dart';
import 'package:kanso/templates/state/daily_template/daily_template_bloc.dart';
import 'package:kanso/templates/state/template_picker/template_picker_bloc.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:kanso/search/state/search/search_bloc.dart';
import 'package:kanso/settings/state/about/about_bloc.dart';
import 'package:kanso/settings/state/licences/licences_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_about_repository.dart';
import 'fixture_library_repository.dart';
import 'fixture_licences_repository.dart';
import 'fixture_renderer_repository.dart';
import 'fixture_reminders_repository.dart';
import 'page_fixtures.dart';
import 'seeded_appearance_bloc.dart';

/// Registers, for a test that builds the app's pages, the BLoCs the pages
/// make for themselves, over [library]; they are taken out again when the
/// test ends.
void registerPageBlocs(
  FixtureLibraryRepository library, [
  FixtureRendererRepository? renderer,
  PageFixtures? fixtures,
]) {
  final page = fixtures ?? PageFixtures();
  final haptics = RecordingHaptics();
  if (getIt.isRegistered<NoteListBloc>()) {
    getIt.unregister<NoteListBloc>();
  }
  getIt.registerFactory<NoteListBloc>(
    () => NoteListBloc(library, ForgetfulPreferencesRepository(), haptics),
  );
  if (getIt.isRegistered<NoteBloc>()) {
    getIt.unregister<NoteBloc>();
  }
  getIt.registerFactory<NoteBloc>(
    () => NoteBloc(library, renderer ?? FixtureRendererRepository()),
  );
  if (getIt.isRegistered<SearchBloc>()) {
    getIt.unregister<SearchBloc>();
  }
  getIt.registerFactory<SearchBloc>(() => SearchBloc(library));
  if (getIt.isRegistered<AboutBloc>()) {
    getIt.unregister<AboutBloc>();
  }
  getIt.registerFactory<AboutBloc>(() => AboutBloc(FixtureAboutRepository()));
  if (getIt.isRegistered<LicencesBloc>()) {
    getIt.unregister<LicencesBloc>();
  }
  getIt.registerFactory<LicencesBloc>(
    () => LicencesBloc(FixtureLicencesRepository()),
  );
  if (getIt.isRegistered<ThemePickerBloc>()) {
    getIt.unregister<ThemePickerBloc>();
  }
  getIt.registerFactory<ThemePickerBloc>(
    () => ThemePickerBloc(renderer ?? FixtureRendererRepository()),
  );
  void replace<T extends Object>(T Function() make) {
    if (getIt.isRegistered<T>()) {
      getIt.unregister<T>();
    }
    getIt.registerFactory<T>(make);
  }

  replace<JournalBloc>(() => JournalBloc(page.journal, page.journal));
  replace<CalendarBloc>(() => CalendarBloc(page.journal));
  replace<TemplatePickerBloc>(() => TemplatePickerBloc(page.journal));
  replace<DailyTemplateBloc>(() => DailyTemplateBloc(page.journal));
  replace<BoardBloc>(() => BoardBloc(page.boards, library));
  final coordinator = RemindersCoordinator(
    ForgetfulPreferencesRepository(),
    FixtureRemindersRepository(),
    page.notifier,
    library,
  );
  replace<RemindersSettingsBloc>(
    () => RemindersSettingsBloc(
      ForgetfulPreferencesRepository(),
      page.notifier,
      coordinator,
    ),
  );
  addTearDown(() => unawaited(getIt.reset()));
}
