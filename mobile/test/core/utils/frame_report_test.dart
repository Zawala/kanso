// Kanso — frame_report_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, CombinableAny, Generator, Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/models/data/frame_sample.dart';
import 'package:kanso/core/utils/frame_report.dart';

import '../../helpers/property.dart';

extension _FrameAny on Any {
  /// Frames of 0 to 30 ms on each thread, in tenths of a millisecond.
  Generator<FrameSample> get frameSample => combine2(
    intInRange(0, 300),
    intInRange(0, 300),
    (int build, int raster) =>
        FrameSample(buildMs: build / 10, rasterMs: raster / 10),
  );
}

void main() {
  group('frameReport', () {
    test('has the budget of the refresh rate', () {
      expect(
        frameReport(const <FrameSample>[], 120).budgetMs,
        closeTo(8.33, 0.01),
      );
      expect(
        frameReport(const <FrameSample>[], 60).budgetMs,
        closeTo(16.67, 0.01),
      );
    });

    test('takes the nearest-rank percentiles of each thread', () {
      final samples = <FrameSample>[
        for (var i = 1; i <= 100; i++)
          FrameSample(buildMs: i.toDouble(), rasterMs: 101.0 - i),
      ];
      final report = frameReport(samples, 60);
      expect(report.frames, 100);
      expect(report.buildP50, 50);
      expect(report.buildP90, 90);
      expect(report.buildP99, 99);
      expect(report.rasterP50, 50);
      expect(report.rasterP90, 90);
      expect(report.rasterP99, 99);
    });

    test('counts a frame over the budget on either thread, once', () {
      final report = frameReport(const <FrameSample>[
        FrameSample(buildMs: 1, rasterMs: 1),
        FrameSample(buildMs: 9, rasterMs: 1),
        FrameSample(buildMs: 1, rasterMs: 9),
        FrameSample(buildMs: 9, rasterMs: 9),
      ], 120);
      expect(report.framesOverBudget, 3);
      expect(report.overBudgetShare, 0.75);
    });

    test('a frame exactly on the budget is not over it', () {
      final report = frameReport(const <FrameSample>[
        FrameSample(buildMs: 10, rasterMs: 10),
      ], 100);
      expect(report.framesOverBudget, 0);
    });

    test('no frames is a report of zeros', () {
      final report = frameReport(const <FrameSample>[], 60);
      expect(report.frames, 0);
      expect(report.buildP99, 0);
      expect(report.overBudgetShare, 0);
    });

    test('writes JSON for the driver', () {
      final json = frameReport(const <FrameSample>[
        FrameSample(buildMs: 2, rasterMs: 3),
      ], 120).toJson();
      expect(json['frames'], 1);
      expect(json['refreshRate'], 120);
      expect(json['buildP99'], 2);
      expect(json['rasterP99'], 3);
    });

    Glados2<List<FrameSample>, int>(
      any.list(any.frameSample),
      any.intInRange(24, 241),
      kansoExploreConfig(),
    ).test('its counts agree with the timings', (samples, hertz) {
      final report = frameReport(samples, hertz.toDouble());
      final budget = 1000 / hertz;
      expect(report.frames, samples.length);
      expect(
        report.framesOverBudget,
        samples.where((s) => s.buildMs > budget || s.rasterMs > budget).length,
      );
      expect(report.framesOverBudget, lessThanOrEqualTo(report.frames));
      // Percentiles are ordered, and are values that were measured.
      expect(report.buildP50, lessThanOrEqualTo(report.buildP90));
      expect(report.buildP90, lessThanOrEqualTo(report.buildP99));
      expect(report.rasterP50, lessThanOrEqualTo(report.rasterP90));
      expect(report.rasterP90, lessThanOrEqualTo(report.rasterP99));
      if (samples.isNotEmpty) {
        expect(samples.map((s) => s.buildMs), contains(report.buildP99));
        expect(samples.map((s) => s.rasterMs), contains(report.rasterP99));
      }
    });

    Glados2<List<FrameSample>, int>(
      any.list(any.frameSample),
      any.intInRange(24, 241),
      kansoExploreConfig(),
    ).test('p99 under the budget and at most 1% over agree with the counts', (
      samples,
      hertz,
    ) {
      final report = frameReport(samples, hertz.toDouble());
      final problems = frameProblems(report);
      final passes =
          samples.isNotEmpty &&
          report.buildP99 < report.budgetMs &&
          report.rasterP99 < report.budgetMs &&
          report.overBudgetShare <= 0.01;
      expect(problems.isEmpty, passes, reason: '$problems');
    });
  });

  group('frameProblems', () {
    test('a calm run at 120 Hz passes', () {
      final report = frameReport(<FrameSample>[
        for (var i = 0; i < 200; i++)
          const FrameSample(buildMs: 3, rasterMs: 4),
      ], 120);
      expect(frameProblems(report, minRefreshRate: 119), isEmpty);
    });

    test('says what is wrong', () {
      final report = frameReport(<FrameSample>[
        for (var i = 0; i < 100; i++)
          const FrameSample(buildMs: 12, rasterMs: 2),
      ], 60);
      expect(frameProblems(report), isEmpty);
      final slow = frameReport(<FrameSample>[
        for (var i = 0; i < 100; i++)
          const FrameSample(buildMs: 12, rasterMs: 2),
      ], 120);
      expect(
        frameProblems(slow),
        <Matcher>[contains('p99 build'), contains('100 of 100 frames')],
      );
      expect(
        frameProblems(report, minRefreshRate: 119),
        <Matcher>[contains('60.0 Hz')],
      );
      expect(frameProblems(frameReport(const <FrameSample>[], 60)), <Matcher>[
        contains('no frames'),
      ]);
    });
  });
}
