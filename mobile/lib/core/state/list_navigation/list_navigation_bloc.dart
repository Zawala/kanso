// Kanso — list_navigation_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart';
import '../../models/repos/abs/library_repository.dart';

part 'list_navigation_bloc.freezed.dart';
part 'list_navigation_event.dart';
part 'list_navigation_state.dart';

/// The notes of the list on screen, in order, for the keyboard to step
/// through them from outside the list; and the pin it toggles.
///
/// The list or the search that is shown reports its notes as they change.
@injectable
class ListNavigationBloc
    extends Bloc<ListNavigationEvent, ListNavigationState> {
  ListNavigationBloc(this._library) : super(const ListNavigationState()) {
    on<ListNavigationListShownEvent>(
      (event, emit) => emit(state.copyWith(noteIds: event.noteIds)),
    );
    on<ListNavigationPinRequestedEvent>(
      (event, emit) =>
          _library.change(<String>[event.noteId], const NoteChange.togglePin()),
      transformer: droppable(),
    );
  }

  final LibraryRepository _library;
}
