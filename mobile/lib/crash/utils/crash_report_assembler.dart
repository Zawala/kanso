// Kanso — crash_report_assembler.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/crash_app.dart';
import '../models/data/crash_context.dart';
import '../models/data/crash_kind.dart';
import '../models/data/crash_os.dart';
import '../models/data/crash_report.dart';
import '../models/data/crash_trace.dart';
import 'crash_grammar.dart';
import 'crash_report_text.dart';
import 'crash_scrubber.dart';

/// Puts a scrubbed [CrashTrace] and the [CrashContext] of the running app
/// into a [CrashReport], checking every field against the grammar once more.
abstract final class CrashReportAssembler {
  /// The report, or null if there is no frame to report.
  ///
  /// A raw crash that names the version it ran (MetricKit does) is reported
  /// with that version and no patch number unless it is the running build:
  /// the patch of a build that is gone is not known. [fallbackDay] is used
  /// when the trace has no day.
  static CrashReport? assemble({
    required CrashKind kind,
    required CrashTrace trace,
    required CrashContext context,
    required String fallbackDay,
  }) {
    var frames = <String>[
      for (final frame in trace.frames)
        if (CrashGrammar.isFrame(frame)) frame,
    ].take(CrashGrammar.maxFrames).toList();
    if (frames.isEmpty) {
      return null;
    }
    final running = _app(context.app);
    final version = CrashScrubber.appVersion(trace.appVersion);
    final build = CrashScrubber.appBuild(trace.appBuild);
    final sameBuild =
        (version == null || version == running.version) &&
        (build == null || build == running.build);
    final report = CrashReport(
      v: 1,
      kind: kind,
      type: CrashScrubber.type(trace.type),
      app: CrashApp(
        version: version ?? running.version,
        build: build ?? running.build,
        patch: sameBuild ? running.patch : null,
      ),
      os: CrashOs(
        name: context.os.name,
        version:
            CrashScrubber.osVersion(trace.osVersion) ??
            CrashScrubber.osVersion(context.os.version) ??
            CrashScrubber.unknown,
      ),
      device:
          CrashScrubber.device(trace.device) ??
          CrashScrubber.device(context.device) ??
          CrashScrubber.unknown,
      day:
          CrashScrubber.day(trace.day) ??
          CrashScrubber.day(fallbackDay) ??
          '2000-01-01',
      frames: frames,
    );
    // The whole body is at most 32 KiB: the outermost frames go first.
    var current = report;
    while (crashReportText(current).length > CrashGrammar.maxBodyBytes) {
      frames = frames.sublist(0, frames.length - 1);
      if (frames.isEmpty) {
        return null;
      }
      current = report.copyWith(frames: frames);
    }
    return current;
  }

  static CrashApp _app(CrashApp app) => CrashApp(
    version: CrashScrubber.appVersion(app.version) ?? '0.0.0',
    build: CrashScrubber.appBuild(app.build) ?? '0',
    patch: CrashScrubber.patch(app.patch),
  );

  /// Whether [report] is wholly inside the grammar: what is checked before a
  /// report is sent, and again for a file read back from the inbox.
  static bool isValid(CrashReport report) {
    final typeOk =
        report.type == CrashScrubber.unknown ||
        CrashGrammar.type.hasMatch(report.type);
    final osOk =
        report.os.version == CrashScrubber.unknown ||
        CrashGrammar.osVersion.hasMatch(report.os.version);
    final deviceOk =
        report.device == CrashScrubber.unknown ||
        CrashGrammar.device.hasMatch(report.device);
    return report.v == 1 &&
        typeOk &&
        CrashGrammar.appVersion.hasMatch(report.app.version) &&
        CrashGrammar.appBuild.hasMatch(report.app.build) &&
        CrashScrubber.patch(report.app.patch) == report.app.patch &&
        osOk &&
        deviceOk &&
        CrashScrubber.day(report.day) == report.day &&
        report.frames.isNotEmpty &&
        report.frames.length <= CrashGrammar.maxFrames &&
        report.frames.every(CrashGrammar.isFrame) &&
        crashReportText(report).length <= CrashGrammar.maxBodyBytes;
  }
}
