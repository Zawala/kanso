// Kanso — register_editor_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart' show addTearDown;
import 'package:kanso/editor/models/repos/abs/editor_repository.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/models/repos/abs/note_place_repository.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';

import 'fake_editor_repository.dart';
import 'fake_web_link_repository.dart';
import 'fixture_library_repository.dart';

/// Registers, for a test that builds the note page in edit mode, an
/// `EditorBloc` that opens [document]; it is taken out when the test ends.
FakeEditorRepository registerEditorBloc(
  EditorDocument document, {
  FixtureLibraryRepository? library,
  FakeWebLinkRepository? links,
  NotePlaceRepository? places,
}) {
  final repository = FakeEditorRepository(document: document);
  if (getIt.isRegistered<EditorBloc>()) {
    getIt.unregister<EditorBloc>();
  }
  getIt.registerFactory<EditorBloc>(
    () => EditorBloc(
      repository,
      library ?? FixtureLibraryRepository(),
      places ?? MemoryNotePlaceRepository(),
      links ?? FakeWebLinkRepository(),
    ),
  );
  addTearDown(() {
    if (getIt.isRegistered<EditorBloc>()) {
      getIt.unregister<EditorBloc>();
    }
  });
  return repository;
}
