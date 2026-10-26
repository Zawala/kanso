// Kanso — register_page_blocs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/injection.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';

import 'browse_blocs.dart';
import 'fixture_library_repository.dart';
import 'seeded_appearance_bloc.dart';

/// Registers, for a test that builds the app's pages, the BLoCs the pages
/// make for themselves, over [library]; they are taken out again when the
/// test ends.
void registerPageBlocs(FixtureLibraryRepository library) {
  final haptics = RecordingHaptics();
  if (getIt.isRegistered<NoteListBloc>()) {
    getIt.unregister<NoteListBloc>();
  }
  getIt.registerFactory<NoteListBloc>(
    () => NoteListBloc(library, ForgetfulPreferencesRepository(), haptics),
  );
  addTearDown(() => unawaited(getIt.reset()));
}
