// Kanso — crash_reports_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/load_status.dart';
import '../../models/data/crash_send_outcome.dart';
import '../../models/data/pending_crash_report.dart';
import '../../models/repos/abs/crash_report_repository.dart';

part 'crash_reports_bloc.freezed.dart';
part 'crash_reports_event.dart';
part 'crash_reports_state.dart';

/// The crash reports waiting on the device and what the person decides about
/// them: the notice after a launch that found some, and the Crash reports
/// page. Nothing is sent unless an event here asks for it.
@injectable
class CrashReportsBloc extends Bloc<CrashReportsEvent, CrashReportsState> {
  CrashReportsBloc(this._reports) : super(const CrashReportsState()) {
    on<CrashReportsStartedEvent>(_onStarted, transformer: sequential());
    on<CrashReportsNoticeReportToggledEvent>(
      (event, emit) =>
          emit(state.copyWith(noticeShowsReport: !state.noticeShowsReport)),
    );
    on<CrashReportsReportToggledEvent>(
      (event, emit) => emit(
        state.copyWith(expanded: state.expanded == event.id ? null : event.id),
      ),
    );
    on<CrashReportsSendRequestedEvent>(_onSend, transformer: sequential());
    on<CrashReportsDiscardRequestedEvent>(
      _onDiscard,
      transformer: sequential(),
    );
    on<CrashReportsSendAllRequestedEvent>(
      _onSendAll,
      transformer: sequential(),
    );
    on<CrashReportsDiscardAllRequestedEvent>(
      _onDiscardAll,
      transformer: sequential(),
    );
    on<CrashReportsOutcomeClearedEvent>(
      (event, emit) => emit(state.copyWith(outcome: null)),
    );
  }

  final CrashReportRepository _reports;

  Future<void> _onStarted(
    CrashReportsStartedEvent event,
    Emitter<CrashReportsState> emit,
  ) async {
    final loaded = await _load();
    emit(
      loaded == null
          ? state.copyWith(status: LoadStatus.failed)
          : state.copyWith(
              status: LoadStatus.ready,
              reports: loaded,
              noticeVisible: loaded.isNotEmpty,
            ),
    );
  }

  Future<void> _onSend(
    CrashReportsSendRequestedEvent event,
    Emitter<CrashReportsState> emit,
  ) async {
    emit(state.copyWith(sending: <String>{...state.sending, event.id}));
    final outcome = await _reports.send(event.id);
    emit(
      state.copyWith(
        sending: <String>{...state.sending}..remove(event.id),
        outcome: outcome,
      ),
    );
    await _reload(emit);
  }

  Future<void> _onDiscard(
    CrashReportsDiscardRequestedEvent event,
    Emitter<CrashReportsState> emit,
  ) async {
    await _reports.discard(event.id);
    await _reload(emit);
  }

  Future<void> _onSendAll(
    CrashReportsSendAllRequestedEvent event,
    Emitter<CrashReportsState> emit,
  ) async {
    final ids = <String>[for (final report in state.reports) report.id];
    emit(state.copyWith(sending: ids.toSet(), outcome: null));
    // The first thing that went wrong is the one to say.
    var outcome = CrashSendOutcome.sent;
    for (final id in ids) {
      final result = await _reports.send(id);
      if (outcome == CrashSendOutcome.sent) {
        outcome = result;
      }
    }
    emit(state.copyWith(sending: const <String>{}, outcome: outcome));
    await _reload(emit);
  }

  Future<void> _onDiscardAll(
    CrashReportsDiscardAllRequestedEvent event,
    Emitter<CrashReportsState> emit,
  ) async {
    for (final report in state.reports) {
      await _reports.discard(report.id);
    }
    emit(state.copyWith(noticeVisible: false, noticeShowsReport: false));
    await _reload(emit);
  }

  /// Reads the reports again; the notice closes once nothing is left.
  Future<void> _reload(Emitter<CrashReportsState> emit) async {
    final loaded = await _load();
    if (loaded == null) {
      return;
    }
    emit(
      state.copyWith(
        status: LoadStatus.ready,
        reports: loaded,
        noticeVisible: state.noticeVisible && loaded.isNotEmpty,
        noticeShowsReport: state.noticeShowsReport && loaded.isNotEmpty,
        expanded: loaded.any((r) => r.id == state.expanded)
            ? state.expanded
            : null,
      ),
    );
  }

  Future<List<PendingCrashReport>?> _load() async {
    final result = await _reports.pending();
    return result.fold((error) => null, (reports) => reports);
  }
}
