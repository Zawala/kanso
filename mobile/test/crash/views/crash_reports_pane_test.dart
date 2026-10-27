// Kanso — crash_reports_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/crash_send_outcome.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/crash/views/widgets/crash_reports_pane.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/pump_routed.dart';

Future<FixtureCrashReportRepository> _pump(
  WidgetTester tester,
  FixtureCrashReportRepository repository,
) async {
  final bloc = CrashReportsBloc(repository)
    ..add(const CrashReportsEvent.started());
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<CrashReportsBloc>.value(
      value: bloc,
      child: const CrashReportsPane(),
    ),
  );
  await tester.pump();
  await tester.pump();
  return repository;
}

void main() {
  group('CrashReportsPane', () {
    testWidgets('says what a report holds, and that nothing is sent alone', (
      tester,
    ) async {
      await _pump(tester, FixtureCrashReportRepository([]));
      expect(find.text('Crash reports'), findsOneWidget);
      expect(
        find.textContaining('never holds note text, titles or file names'),
        findsOneWidget,
      );
      expect(find.textContaining('Nothing is sent unless'), findsOneWidget);
    });

    testWidgets('an empty page says what will appear', (tester) async {
      await _pump(tester, FixtureCrashReportRepository([]));
      expect(find.text('No crash reports'), findsOneWidget);
      expect(find.textContaining('waits here'), findsOneWidget);
    });

    testWidgets('lists the day, the kind and the type of each report', (
      tester,
    ) async {
      await _pump(
        tester,
        FixtureCrashReportRepository([
          samplePending('1-1'),
          samplePending(
            '2-1',
            sampleCrashReport(
              kind: CrashKind.androidAnr,
              type: 'ANR',
              day: '2026-10-25',
            ),
          ),
        ]),
      );
      expect(find.text('StateError'), findsOneWidget);
      expect(find.text('2026-10-26 · App'), findsOneWidget);
      expect(find.text('ANR'), findsOneWidget);
      expect(find.text('2026-10-25 · Android not responding'), findsOneWidget);
      expect(find.text('Send all'), findsOneWidget);
    });

    testWidgets('opening a report shows its exact text; Send sends it', (
      tester,
    ) async {
      final repository = await _pump(
        tester,
        FixtureCrashReportRepository([samplePending('1-1')]),
      );
      await tester.tap(find.text('StateError'));
      await tester.pump();
      expect(
        tester.widget<SelectableText>(find.byType(SelectableText)).data,
        repository.reports.single.body,
      );
      await tester.tap(find.text('Send'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(repository.sent, ['1-1']);
      expect(find.text('No crash reports'), findsOneWidget);
      expect(find.text('Report sent. Thank you.'), findsOneWidget);
    });

    testWidgets('Delete removes a report without sending it', (tester) async {
      final repository = await _pump(
        tester,
        FixtureCrashReportRepository([samplePending('1-1')]),
      );
      await tester.tap(find.text('StateError'));
      await tester.pump();
      await tester.tap(find.text('Delete'));
      await tester.pump();
      await tester.pump();
      expect(repository.sent, isEmpty);
      expect(repository.discarded, ['1-1']);
      expect(find.text('No crash reports'), findsOneWidget);
    });

    testWidgets('a report that could not be sent stays, with the reason', (
      tester,
    ) async {
      final repository = await _pump(
        tester,
        FixtureCrashReportRepository(
          [samplePending('1-1')],
          outcome: CrashSendOutcome.rejected,
        ),
      );
      await tester.tap(find.text('StateError'));
      await tester.pump();
      await tester.tap(find.text('Send'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(repository.reports, hasLength(1));
      expect(find.text('The report couldn’t be sent.'), findsOneWidget);
    });
  });
}
