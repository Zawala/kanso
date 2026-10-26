// Kanso — core_check_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/diagnostics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../models/data/core_check_status.dart';

part 'core_check_bloc.freezed.dart';
part 'core_check_event.dart';
part 'core_check_state.dart';

/// Asks the Rust library real questions and shows the answers: its version,
/// whether it can check certificates, how many notes it holds, and what it
/// says changed. It exists to prove the app and the library work together;
/// the screens that replace it come later.
@injectable
class CoreCheckBloc extends Bloc<CoreCheckEvent, CoreCheckState> {
  CoreCheckBloc(this._library, this._diagnostics)
    : super(const CoreCheckState()) {
    on<CoreCheckStartedEvent>(_onStarted);
    on<CoreCheckNoteCreatedEvent>(_onNoteCreated);
    on<CoreCheckChangeReceivedEvent>(_onChangeReceived);
  }

  /// The newest notes shown.
  static const int latestCount = 3;

  /// Every note outside the trash, the newest edit first.
  static const NoteQuery _everyNote = NoteQuery(
    trash: false,
    limit: 0,
    skip: 0,
    sort: SortKey.updated,
    descending: true,
  );

  final LibraryRepository _library;
  final DiagnosticsRepository _diagnostics;
  StreamSubscription<List<String>>? _changes;

  Future<void> _onStarted(
    CoreCheckStartedEvent event,
    Emitter<CoreCheckState> emit,
  ) async {
    emit(const CoreCheckState());
    final version = await _diagnostics.version();
    final tls = await _diagnostics.tlsReady();
    final notes = await _library.notes(_everyNote);
    final error = _firstError(<Either<ApplicationError, Object?>>[
      version,
      tls,
      notes,
    ]);
    if (error != null) {
      emit(state.copyWith(status: CoreCheckStatus.failed, error: error));
      return;
    }
    final rows = notes.getOrElse(() => const <NoteRow>[]);
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(CoreCheckEvent.changeReceived(ids)),
    );
    emit(
      state.copyWith(
        status: CoreCheckStatus.ready,
        version: version.getOrElse(() => ''),
        tlsReady: tls.getOrElse(() => false),
        noteCount: rows.length,
        latest: rows.take(latestCount).toList(),
      ),
    );
  }

  Future<void> _onNoteCreated(
    CoreCheckNoteCreatedEvent event,
    Emitter<CoreCheckState> emit,
  ) async {
    final saved = await _library.create(
      NewNote(
        title: 'Core check ${state.noteCount + 1}',
        body: 'Made by the core check.',
      ),
    );
    // The change stream reports the new note; here only a failure shows.
    final error = saved.fold<ApplicationError?>((error) => error, (_) => null);
    if (error != null) {
      emit(state.copyWith(error: error));
    }
  }

  Future<void> _onChangeReceived(
    CoreCheckChangeReceivedEvent event,
    Emitter<CoreCheckState> emit,
  ) async {
    final notes = await _library.notes(_everyNote);
    notes.fold((error) => emit(state.copyWith(error: error)), (rows) {
      emit(
        state.copyWith(
          noteCount: rows.length,
          latest: rows.take(latestCount).toList(),
          lastChange: event.ids,
          error: null,
        ),
      );
    });
  }

  ApplicationError? _firstError(
    List<Either<ApplicationError, Object?>> answers,
  ) {
    for (final answer in answers) {
      final error = answer.fold<ApplicationError?>(
        (error) => error,
        (_) => null,
      );
      if (error != null) {
        return error;
      }
    }
    return null;
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
