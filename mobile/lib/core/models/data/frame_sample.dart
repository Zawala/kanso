// Kanso — frame_sample.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'frame_sample.freezed.dart';

/// How long one frame took on each thread, in milliseconds: building the
/// widgets and laying them out (the UI thread), and drawing them (the raster
/// thread).
@freezed
abstract class FrameSample with _$FrameSample {
  const factory FrameSample({
    required double buildMs,
    required double rasterMs,
  }) = _FrameSample;
}
