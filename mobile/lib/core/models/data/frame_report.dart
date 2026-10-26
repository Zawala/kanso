// Kanso — frame_report.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'frame_report.freezed.dart';

/// What a run of frames cost, against what the display allows: the
/// numbers of the scrolling check.
@freezed
abstract class FrameReport with _$FrameReport {
  const factory FrameReport({
    /// How many frames were drawn.
    required int frames,

    /// The display's refresh rate in hertz, as the app reported it.
    required double refreshRate,

    /// How long a frame may take on each thread to keep up with the display:
    /// `1000 / refreshRate`, in milliseconds.
    required double budgetMs,

    /// The median, 90th and 99th percentile of the build times, in
    /// milliseconds.
    required double buildP50,
    required double buildP90,
    required double buildP99,

    /// The same for the raster times.
    required double rasterP50,
    required double rasterP90,
    required double rasterP99,

    /// How many frames took longer than [budgetMs] on either thread.
    required int framesOverBudget,
  }) = _FrameReport;

  const FrameReport._();

  /// The share of frames that took longer than the budget, from 0 to 1; 0
  /// with no frames.
  double get overBudgetShare => frames == 0 ? 0 : framesOverBudget / frames;

  /// The report as JSON, for the test driver to write down.
  Map<String, Object> toJson() => <String, Object>{
    'frames': frames,
    'refreshRate': refreshRate,
    'budgetMs': budgetMs,
    'buildP50': buildP50,
    'buildP90': buildP90,
    'buildP99': buildP99,
    'rasterP50': rasterP50,
    'rasterP90': rasterP90,
    'rasterP99': rasterP99,
    'framesOverBudget': framesOverBudget,
    'overBudgetShare': overBudgetShare,
  };
}
