// Kanso — backup_notice_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'backup_notice_bloc.dart';

@freezed
sealed class BackupNoticeEvent with _$BackupNoticeEvent {
  /// The app started: find out whether the notice has been seen.
  const factory BackupNoticeEvent.started() = BackupNoticeStartedEvent;

  /// The person pressed OK.
  const factory BackupNoticeEvent.dismissed() = BackupNoticeDismissedEvent;
}
