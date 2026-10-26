// Kanso — backup_notice_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/repos/abs/preferences_repository.dart';

part 'backup_notice_bloc.freezed.dart';
part 'backup_notice_event.dart';
part 'backup_notice_state.dart';

/// Whether the notice that this phone's notes are not in device backups is
/// to be shown: once, the first time the app opens.
///
/// It is seen once it is dismissed; the preferences remember it. A notice
/// that cannot be remembered is still dismissed for this run.
@injectable
class BackupNoticeBloc extends Bloc<BackupNoticeEvent, BackupNoticeState> {
  BackupNoticeBloc(this._preferences) : super(const BackupNoticeState()) {
    on<BackupNoticeStartedEvent>(_onStarted, transformer: restartable());
    on<BackupNoticeDismissedEvent>(_onDismissed, transformer: droppable());
  }

  final PreferencesRepository _preferences;

  Future<void> _onStarted(
    BackupNoticeStartedEvent event,
    Emitter<BackupNoticeState> emit,
  ) async {
    final loaded = await _preferences.load();
    // Preferences that cannot be read are the defaults, so the notice shows
    // and the next save puts a file there.
    final seen = loaded.fold((_) => false, (found) => found.backupNoticeSeen);
    emit(BackupNoticeState(visible: !seen));
  }

  Future<void> _onDismissed(
    BackupNoticeDismissedEvent event,
    Emitter<BackupNoticeState> emit,
  ) async {
    emit(const BackupNoticeState());
    await _preferences.update(
      (current) => current.copyWith(backupNoticeSeen: true),
    );
  }
}
