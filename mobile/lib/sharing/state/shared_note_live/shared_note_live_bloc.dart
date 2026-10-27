// Kanso — shared_note_live_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/repos/abs/shared_notes_repository.dart';

part 'shared_note_live_bloc.freezed.dart';
part 'shared_note_live_event.dart';
part 'shared_note_live_state.dart';

/// The live room of the shared note that is open.
///
/// While the note is open and shared, its room is open: what other people
/// type is merged into the library within a moment (and so reaches the
/// editor through the library's changes), and what the person types goes out
/// the same way. Closing the bloc leaves the room. A note that is not shared
/// has no room, and this does nothing for it.
///
/// The room is closed when the app goes to the background; coming back to
/// the front is [SharedNoteLiveEvent.resumed].
@injectable
class SharedNoteLiveBloc
    extends Bloc<SharedNoteLiveEvent, SharedNoteLiveState> {
  SharedNoteLiveBloc(this._shared) : super(const SharedNoteLiveState()) {
    on<SharedNoteLiveOpenedEvent>(_onOpened, transformer: sequential());
    on<SharedNoteLiveResumedEvent>(
      (event, emit) => _enter(state.noteId, emit),
      transformer: sequential(),
    );
  }

  final SharedNotesRepository _shared;

  Future<void> _onOpened(
    SharedNoteLiveOpenedEvent event,
    Emitter<SharedNoteLiveState> emit,
  ) async {
    // The room of the note that was open before this one is left.
    if (state.live && state.noteId != event.noteId) {
      await _shared.closeLive(state.noteId);
    }
    emit(SharedNoteLiveState(noteId: event.noteId));
    await _enter(event.noteId, emit);
  }

  Future<void> _enter(String noteId, Emitter<SharedNoteLiveState> emit) async {
    if (noteId.isEmpty) {
      return;
    }
    final read = await _shared.info(noteId);
    final shared = read.fold((_) => false, (info) => info != null);
    if (!shared) {
      return;
    }
    final opened = await _shared.openLive(noteId);
    emit(state.copyWith(live: opened.isRight()));
  }

  @override
  Future<void> close() async {
    if (state.live) {
      await _shared.closeLive(state.noteId);
    }
    return super.close();
  }
}
