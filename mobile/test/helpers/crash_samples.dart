// Kanso — crash_samples.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/patch_repository.dart';
import 'package:kanso/crash/models/data/crash_app.dart';
import 'package:kanso/crash/models/data/crash_context.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/crash_os.dart';
import 'package:kanso/crash/models/data/crash_os_name.dart';
import 'package:kanso/crash/models/data/crash_report.dart';
import 'package:kanso/crash/models/data/crash_send_outcome.dart';
import 'package:kanso/crash/models/data/pending_crash_report.dart';
import 'package:kanso/crash/models/repos/abs/crash_context_repository.dart';
import 'package:kanso/crash/models/repos/abs/crash_report_repository.dart';
import 'package:kanso/crash/state/crash_reports/crash_reports_bloc.dart';
import 'package:kanso/crash/utils/crash_report_text.dart';

/// The context of a phone in tests: Kanso 0.9.0 (57), patch 2, iOS 18.1.
const CrashContext sampleCrashContext = CrashContext(
  app: CrashApp(version: '0.9.0', build: '57', patch: 2),
  os: CrashOs(name: CrashOsName.ios, version: '18.1'),
  device: 'iPhone17,1',
);

class FixtureCrashContextRepository implements CrashContextRepository {
  FixtureCrashContextRepository([this.context = sampleCrashContext]);

  final CrashContext context;

  @override
  Future<CrashContext> read() async => context;
}

class FixturePatchRepository implements PatchRepository {
  FixturePatchRepository(this.patch);

  final int? patch;

  @override
  Future<int?> currentPatch() async => patch;
}

/// The report of the example in docs/CRASH-REPORTS.md.
CrashReport sampleCrashReport({
  CrashKind kind = CrashKind.dart,
  String type = 'StateError',
  String day = '2026-10-26',
  List<String> frames = const <String>[
    'dart _NoteEditorState._save package:kanso/note/views/widgets/note_editor.dart:120:7',
    'dart <closure> dart:async/zone.dart:1525:47',
  ],
}) => CrashReport(
  v: 1,
  kind: kind,
  type: type,
  app: sampleCrashContext.app,
  os: sampleCrashContext.os,
  device: sampleCrashContext.device,
  day: day,
  frames: frames,
);

PendingCrashReport samplePending(String id, [CrashReport? report]) {
  final value = report ?? sampleCrashReport();
  return PendingCrashReport(
    id: id,
    report: value,
    body: crashReportText(value),
  );
}

/// A crash repository in memory: [send] answers [outcome] and deletes the
/// report when it is sent.
class FixtureCrashReportRepository implements CrashReportRepository {
  FixtureCrashReportRepository(
    this.reports, {
    this.outcome = CrashSendOutcome.sent,
  });

  List<PendingCrashReport> reports;
  CrashSendOutcome outcome;
  final List<String> sent = <String>[];
  final List<String> discarded = <String>[];
  final List<(Object, StackTrace)> recorded = <(Object, StackTrace)>[];

  @override
  Future<void> recordDartError(Object error, StackTrace stack) async {
    recorded.add((error, stack));
  }

  @override
  Future<Either<ApplicationError, List<PendingCrashReport>>> pending() async =>
      Right<ApplicationError, List<PendingCrashReport>>(
        List<PendingCrashReport>.of(reports),
      );

  @override
  Future<Either<ApplicationError, Unit>> discard(String id) async {
    discarded.add(id);
    reports = <PendingCrashReport>[
      for (final report in reports)
        if (report.id != id) report,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<CrashSendOutcome> send(String id) async {
    sent.add(id);
    if (outcome == CrashSendOutcome.sent) {
      reports = <PendingCrashReport>[
        for (final report in reports)
          if (report.id != id) report,
      ];
    }
    return outcome;
  }
}

/// A crash reports bloc that shows nothing: it was not started, so it has
/// read no reports and no notice covers the app in a test.
CrashReportsBloc quietCrashReportsBloc() =>
    CrashReportsBloc(FixtureCrashReportRepository(<PendingCrashReport>[]));
