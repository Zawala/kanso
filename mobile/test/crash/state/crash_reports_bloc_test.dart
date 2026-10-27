// Kanso — crash_reports_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/crash/models/data/crash_send_outcome.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';

import '../../helpers/crash_samples.dart';

void main() {
  late FixtureCrashReportRepository repository;

  setUp(() {
    repository = FixtureCrashReportRepository([
      samplePending('1-1'),
      samplePending('2-1', sampleCrashReport(type: 'RangeError')),
    ]);
  });

  CrashReportsBloc build() => CrashReportsBloc(repository);

  group('CrashReportsBloc', () {
    blocTest<CrashReportsBloc, CrashReportsState>(
      'started reads the reports and shows the notice',
      build: build,
      act: (bloc) => bloc.add(const CrashReportsEvent.started()),
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.reports, hasLength(2));
        expect(bloc.state.noticeVisible, isTrue);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'started with nothing waiting shows no notice',
      build: () {
        repository.reports = [];
        return build();
      },
      act: (bloc) => bloc.add(const CrashReportsEvent.started()),
      verify: (bloc) {
        expect(bloc.state.noticeVisible, isFalse);
        expect(bloc.state.reports, isEmpty);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'nothing is sent by reading',
      build: build,
      act: (bloc) => bloc.add(const CrashReportsEvent.started()),
      verify: (_) => expect(repository.sent, isEmpty),
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'send all sends every report and closes the notice',
      build: build,
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.noticeVisible);
        bloc.add(const CrashReportsEvent.sendAllRequested());
      },
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(repository.sent, ['1-1', '2-1']);
        expect(bloc.state.reports, isEmpty);
        expect(bloc.state.noticeVisible, isFalse);
        expect(bloc.state.outcome, CrashSendOutcome.sent);
        expect(bloc.state.sending, isEmpty);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'a send that fails keeps the reports and the notice, and says why',
      build: () {
        repository.outcome = CrashSendOutcome.tryLater;
        return build();
      },
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.noticeVisible);
        bloc.add(const CrashReportsEvent.sendAllRequested());
      },
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(bloc.state.reports, hasLength(2));
        expect(bloc.state.noticeVisible, isTrue);
        expect(bloc.state.outcome, CrashSendOutcome.tryLater);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      "don't send deletes every report and closes the notice",
      build: build,
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.noticeVisible);
        bloc.add(const CrashReportsEvent.discardAllRequested());
      },
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(repository.discarded, ['1-1', '2-1']);
        expect(repository.sent, isEmpty);
        expect(bloc.state.reports, isEmpty);
        expect(bloc.state.noticeVisible, isFalse);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'one report can be sent or deleted on its own',
      build: build,
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.reports.isNotEmpty);
        bloc
          ..add(const CrashReportsEvent.sendRequested('1-1'))
          ..add(const CrashReportsEvent.discardRequested('2-1'));
      },
      wait: const Duration(milliseconds: 50),
      verify: (bloc) {
        expect(repository.sent, ['1-1']);
        expect(repository.discarded, ['2-1']);
        expect(bloc.state.reports, isEmpty);
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'the text of a report opens and closes, in the notice and on the page',
      build: build,
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.reports.isNotEmpty);
        bloc
          ..add(const CrashReportsEvent.noticeReportToggled())
          ..add(const CrashReportsEvent.reportToggled('1-1'));
      },
      verify: (bloc) {
        expect(bloc.state.noticeShowsReport, isTrue);
        expect(bloc.state.expanded, '1-1');
      },
    );

    blocTest<CrashReportsBloc, CrashReportsState>(
      'the message about a send can be cleared',
      build: () {
        repository.outcome = CrashSendOutcome.rejected;
        return build();
      },
      act: (bloc) async {
        bloc.add(const CrashReportsEvent.started());
        await bloc.stream.firstWhere((s) => s.reports.isNotEmpty);
        bloc.add(const CrashReportsEvent.sendRequested('1-1'));
        await bloc.stream.firstWhere((s) => s.outcome != null);
        bloc.add(const CrashReportsEvent.outcomeCleared());
      },
      wait: const Duration(milliseconds: 50),
      verify: (bloc) => expect(bloc.state.outcome, isNull),
    );
  });
}
