// Kanso — crash_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/pending_crash_report.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/crash/views/widgets/crash_notice_host.dart';
import 'package:kanso/crash/views/widgets/crash_reports_pane.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/golden.dart';

Future<Widget> _page(
  Widget Function(Widget child) wrap, {
  required List<PendingCrashReport> reports,
  bool open = false,
  bool notice = false,
}) async {
  final bloc = CrashReportsBloc(
    FixtureCrashReportRepository(List<PendingCrashReport>.of(reports)),
  )..add(const CrashReportsEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((s) => s.status.name != 'loading');
  if (notice && open) {
    bloc.add(const CrashReportsEvent.noticeReportToggled());
  }
  if (!notice && open) {
    bloc.add(const CrashReportsEvent.reportToggled('1-1'));
  }
  return BlocProvider<CrashReportsBloc>.value(
    value: bloc,
    child: wrap(const SizedBox()),
  );
}

void main() {
  final reports = [
    samplePending('1-1'),
    samplePending(
      '2-1',
      sampleCrashReport(
        kind: CrashKind.androidNative,
        type: 'SIGSEGV',
        day: '2026-10-25',
        frames: const ['native libflutter.so +0x1f2e30 abort+0x4'],
      ),
    ),
  ];

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the crash pictures, $name', () {
      testWidgets('the notice', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            (_) => const CrashNoticeHost(
              child: Scaffold(body: Center(child: Text('Kanso'))),
            ),
            reports: reports,
            notice: true,
          ),
          name: 'phone_crash_notice_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the notice with the report shown', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            (_) => const CrashNoticeHost(
              child: Scaffold(body: Center(child: Text('Kanso'))),
            ),
            reports: reports.sublist(0, 1),
            notice: true,
            open: true,
          ),
          name: 'phone_crash_notice_report_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the page, empty', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            (_) => const Scaffold(body: CrashReportsPane()),
            reports: const <PendingCrashReport>[],
          ),
          name: 'phone_crash_reports_empty_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the page, with a report open', (tester) async {
        await expectGoldenOnBothPlatforms(
          tester,
          await _page(
            (_) => const Scaffold(body: CrashReportsPane()),
            reports: reports,
            open: true,
          ),
          name: 'phone_crash_reports_open_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });
    });
  }
}
