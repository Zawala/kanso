// Kanso — crash_reports_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'crash_reports_bloc.dart';

@freezed
sealed class CrashReportsEvent with _$CrashReportsEvent {
  /// The app launched, or the Crash reports page opened: read the reports.
  /// At launch the notice is shown if there are any.
  const factory CrashReportsEvent.started() = CrashReportsStartedEvent;

  /// The notice's Show report was pressed (or pressed again).
  const factory CrashReportsEvent.noticeReportToggled() =
      CrashReportsNoticeReportToggledEvent;

  /// A report's row was opened or closed on the page.
  const factory CrashReportsEvent.reportToggled(String id) =
      CrashReportsReportToggledEvent;

  /// Send report [id].
  const factory CrashReportsEvent.sendRequested(String id) =
      CrashReportsSendRequestedEvent;

  /// Delete report [id] without sending it.
  const factory CrashReportsEvent.discardRequested(String id) =
      CrashReportsDiscardRequestedEvent;

  /// Send every report (the notice's Send).
  const factory CrashReportsEvent.sendAllRequested() =
      CrashReportsSendAllRequestedEvent;

  /// Delete every report and close the notice (the notice's Don't send).
  const factory CrashReportsEvent.discardAllRequested() =
      CrashReportsDiscardAllRequestedEvent;

  /// The message about the last send was read.
  const factory CrashReportsEvent.outcomeCleared() =
      CrashReportsOutcomeClearedEvent;
}
