// Kanso — crash_notice_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/models/data/crash_send_outcome.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/crash/views/widgets/crash_notice_card.dart';
import 'package:kanso/crash/views/widgets/crash_notice_host.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/pump.dart';

Future<(CrashReportsBloc, FixtureCrashReportRepository, ValueNotifier<int>)>
_pump(
  WidgetTester tester, {
  bool reports = true,
  CrashSendOutcome outcome = CrashSendOutcome.sent,
}) async {
  final repository = FixtureCrashReportRepository(
    reports ? [samplePending('1-1')] : [],
    outcome: outcome,
  );
  final bloc = CrashReportsBloc(repository)
    ..add(const CrashReportsEvent.started());
  closeBlocLater(bloc);
  final taps = ValueNotifier<int>(0);
  addTearDown(taps.dispose);
  await pumpKanso(
    tester,
    BlocProvider<CrashReportsBloc>.value(
      value: bloc,
      child: CrashNoticeHost(
        child: SizedBox.expand(
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => taps.value++,
            child: const Center(child: Text('Behind')),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
  return (bloc, repository, taps);
}

void main() {
  group('CrashNoticeHost', () {
    testWidgets('asks, in plain words, whether to send a crash report', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.byType(CrashNoticeCard), findsOneWidget);
      expect(find.text('Kanso closed unexpectedly'), findsOneWidget);
      expect(find.textContaining('Send a crash report?'), findsOneWidget);
      expect(
        find.textContaining('never note text, titles or file names'),
        findsOneWidget,
      );
      expect(find.text('Send'), findsOneWidget);
      expect(find.text('Don’t send'), findsOneWidget);
      expect(find.text('Show report'), findsOneWidget);
    });

    testWidgets('shows nothing when nothing is waiting', (tester) async {
      await _pump(tester, reports: false);
      expect(find.byType(CrashNoticeCard), findsNothing);
    });

    testWidgets('Show report shows the exact text that would be sent', (
      tester,
    ) async {
      final (_, repository, _) = await _pump(tester);
      await tester.tap(find.text('Show report'));
      await tester.pump();
      final shown = find.byType(SelectableText);
      expect(shown, findsOneWidget);
      expect(
        tester.widget<SelectableText>(shown).data,
        repository.reports.single.body,
      );
      expect(find.textContaining('"kind": "dart"'), findsOneWidget);
      expect(repository.sent, isEmpty);
      await tester.tap(find.text('Hide report'));
      await tester.pump();
      expect(find.byType(SelectableText), findsNothing);
    });

    testWidgets('covers what is behind it until the person chooses', (
      tester,
    ) async {
      final (_, _, taps) = await _pump(tester);
      await tester.tapAt(const Offset(10, 10));
      expect(taps.value, 0);
    });

    testWidgets('Send sends the report and the notice goes', (tester) async {
      final (bloc, repository, taps) = await _pump(tester);
      await tester.tap(find.text('Send'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(repository.sent, ['1-1']);
      expect(find.byType(CrashNoticeCard), findsNothing);
      expect(bloc.state.noticeVisible, isFalse);
      await tester.tapAt(const Offset(10, 10));
      expect(taps.value, 1);
    });

    testWidgets('Don’t send deletes the report and sends nothing', (
      tester,
    ) async {
      final (_, repository, _) = await _pump(tester);
      await tester.tap(find.text('Don’t send'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(repository.sent, isEmpty);
      expect(repository.discarded, ['1-1']);
      expect(find.byType(CrashNoticeCard), findsNothing);
    });

    testWidgets('a send that fails stays and says so', (tester) async {
      final (_, repository, _) = await _pump(
        tester,
        outcome: CrashSendOutcome.tryLater,
      );
      await tester.tap(find.text('Send'));
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(repository.reports, hasLength(1));
      expect(find.byType(CrashNoticeCard), findsOneWidget);
      expect(
        find.text('Too many reports just now. Try again later.'),
        findsOneWidget,
      );
    });
  });
}
