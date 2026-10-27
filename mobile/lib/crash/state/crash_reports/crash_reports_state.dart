// Kanso — crash_reports_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'crash_reports_bloc.dart';

@freezed
abstract class CrashReportsState with _$CrashReportsState {
  const factory CrashReportsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The reports waiting on the device, oldest first.
    @Default(<PendingCrashReport>[]) List<PendingCrashReport> reports,

    /// Whether the "closed unexpectedly" notice is over the app.
    @Default(false) bool noticeVisible,

    /// Whether the notice shows the text of the reports.
    @Default(false) bool noticeShowsReport,

    /// The report whose text is open on the page.
    String? expanded,

    /// The reports being sent now.
    @Default(<String>{}) Set<String> sending,

    /// How the last send ended; null before one, or once it was read.
    CrashSendOutcome? outcome,
  }) = _CrashReportsState;
}
