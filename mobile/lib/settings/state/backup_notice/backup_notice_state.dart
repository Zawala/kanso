// Kanso — backup_notice_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'backup_notice_bloc.dart';

@freezed
abstract class BackupNoticeState with _$BackupNoticeState {
  const factory BackupNoticeState({
    /// Whether the notice is on screen.
    @Default(false) bool visible,
  }) = _BackupNoticeState;
}
