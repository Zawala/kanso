// Kanso — focus_live_preview_sync_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/models/repos/impl/memory_note_place_repository.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/views/widgets/focus_live_preview_sync.dart';
import 'package:kanso/shell/state/focus_mode/focus_mode_bloc.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

/// A note in Preview and focus mode off, with Live Preview in focus mode
/// as [livePreview] has it, under a [FocusLivePreviewSync].
Future<(NoteBloc, FocusModeBloc)> _pump(
  WidgetTester tester, {
  required bool livePreview,
}) async {
  final note = NoteBloc(
    MockLibraryRepository(),
    MockRendererRepository(),
    MemoryNotePlaceRepository(),
  );
  closeBlocLater(note);
  final focus = FocusModeBloc(
    SeededPreferencesRepository(
      Preferences(livePreviewInFocus: livePreview),
    ),
  )..add(const FocusModeEvent.started());
  closeBlocLater(focus);
  await tester.pumpWidget(
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<NoteBloc>.value(value: note),
        BlocProvider<FocusModeBloc>.value(value: focus),
      ],
      child: const FocusLivePreviewSync(child: SizedBox()),
    ),
  );
  await _settle(tester);
  expect(focus.state.ready, isTrue);
  expect(focus.state.livePreview, livePreview);
  return (note, focus);
}

/// Lets the BLoCs and the listeners between them run.
Future<void> _settle(WidgetTester tester) async {
  for (var turn = 0; turn < 5; turn++) {
    await tester.pump();
  }
}

void main() {
  group('FocusLivePreviewSync', () {
    testWidgets('shows Live in focus mode and the mode before after it', (
      tester,
    ) async {
      final (note, focus) = await _pump(tester, livePreview: true);
      expect(note.state.mode, NoteMode.preview);

      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(focus.state.active, isTrue);
      expect(note.state.mode, NoteMode.live);

      focus.add(const FocusModeEvent.exited());
      await _settle(tester);
      expect(focus.state.active, isFalse);
      expect(note.state.mode, NoteMode.preview);
    });

    testWidgets('a mode picked in focus mode stands after it', (tester) async {
      final (note, focus) = await _pump(tester, livePreview: true);
      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(note.state.mode, NoteMode.live);

      note.add(const NoteEvent.modeChanged(NoteMode.edit));
      await _settle(tester);
      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(focus.state.active, isFalse);
      expect(note.state.mode, NoteMode.edit);
    });

    testWidgets('with the setting off, focus mode keeps the mode', (
      tester,
    ) async {
      final (note, focus) = await _pump(tester, livePreview: false);
      note.add(const NoteEvent.modeChanged(NoteMode.edit));
      await _settle(tester);

      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(focus.state.active, isTrue);
      expect(note.state.mode, NoteMode.edit);

      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(note.state.mode, NoteMode.edit);
    });

    testWidgets('a note already in Live stays in Live after focus mode', (
      tester,
    ) async {
      final (note, focus) = await _pump(tester, livePreview: true);
      note.add(const NoteEvent.modeChanged(NoteMode.live));
      await _settle(tester);

      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(note.state.mode, NoteMode.live);
      focus.add(const FocusModeEvent.toggled());
      await _settle(tester);
      expect(note.state.mode, NoteMode.live);
    });
  });
}
