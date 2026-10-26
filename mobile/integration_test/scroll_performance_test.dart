// Kanso — scroll_performance_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/models/data/frame_sample.dart';
import 'package:kanso/core/utils/frame_report.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The lowest refresh rate the display may report, from
/// `--dart-define=KANSO_MIN_REFRESH_RATE=119`: the way to say the device is a
/// 120 Hz one, so that a display that fell back to 60 Hz fails the check
/// instead of passing it at a looser budget. Without it nothing is asked of
/// the display.
const int minRefreshRate = int.fromEnvironment('KANSO_MIN_REFRESH_RATE');

/// How smoothly the note list scrolls on a large library: 2,000 notes in 8
/// notebooks, flung up and down.
///
/// Every run records the frame timings and reports them (to `flutter drive`
/// through `test_driver/perf_driver.dart`, and to the log). Only in profile
/// mode, which is what a shipped app runs in, are the numbers held to a
/// limit: debug mode is slow on purpose and a simulator or emulator draws at
/// 60 Hz, so there it checks only that the report was made. On a device:
///
/// ```sh
/// flutter drive --profile \
///   --driver=test_driver/perf_driver.dart \
///   --target=integration_test/scroll_performance_test.dart \
///   -d <device> [--dart-define=KANSO_MIN_REFRESH_RATE=119]
/// ```
void main() {
  // Frames are drawn as they would be for a person, not as the test pumps.
  final binding = IntegrationTestWidgetsFlutterBinding.ensureInitialized()
    ..framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('scrolling a large library keeps up with the display', (
    tester,
  ) async {
    await startApp(tester);
    await seedManyNotes();
    await pumpUntil(
      tester,
      find.text('2000 notes'),
      timeout: const Duration(minutes: 2),
    );

    final samples = <FrameSample>[];
    void collect(List<FrameTiming> timings) {
      for (final timing in timings) {
        samples.add(
          FrameSample(
            buildMs: timing.buildDuration.inMicroseconds / 1000,
            rasterMs: timing.rasterDuration.inMicroseconds / 1000,
          ),
        );
      }
    }

    final list = find.descendant(
      of: find.byType(NoteListPane),
      matching: find.byType(Scrollable),
    );
    expect(list, findsWidgets);
    final scrollable = list.first;

    SchedulerBinding.instance.addTimingsCallback(collect);
    // Three flings down the list, each left to settle, and the same back up,
    // so the rows built and the rows dropped are both measured.
    for (var i = 0; i < 3; i++) {
      await tester.fling(scrollable, const Offset(0, -900), 3500);
      await tester.pumpAndSettle();
    }
    for (var i = 0; i < 3; i++) {
      await tester.fling(scrollable, const Offset(0, 900), 3500);
      await tester.pumpAndSettle();
    }
    // The engine hands over the timings in batches, a moment after the
    // frames.
    await tester.pump(const Duration(seconds: 1));
    await Future<void>.delayed(const Duration(seconds: 2));
    await tester.pump();
    SchedulerBinding.instance.removeTimingsCallback(collect);

    final refreshRate = tester.view.display.refreshRate;
    final report = frameReport(samples, refreshRate);
    binding.reportData = <String, dynamic>{
      ...?binding.reportData,
      'frame_report': report.toJson(),
    };
    // ignore: avoid_print
    print(
      'Scrolling report: ${report.frames} frames at '
      '${report.refreshRate} Hz (budget ${report.budgetMs.toStringAsFixed(2)} '
      'ms): build p50/p90/p99 ${report.buildP50}/${report.buildP90}/'
      '${report.buildP99} ms, raster p50/p90/p99 ${report.rasterP50}/'
      '${report.rasterP90}/${report.rasterP99} ms, '
      '${report.framesOverBudget} over the budget.',
    );

    // The report is made in every mode.
    expect(report.frames, greaterThan(0));
    if (kProfileMode) {
      expect(
        frameProblems(report, minRefreshRate: minRefreshRate.toDouble()),
        isEmpty,
        reason: 'The list did not keep up with the display.',
      );
    }
  });
}
