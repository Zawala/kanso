// Kanso — source_line.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'source_line.freezed.dart';

/// One line of a note's text, with the styled runs that fall on it.
@freezed
abstract class SourceLine with _$SourceLine {
  const factory SourceLine({
    /// Where the line starts in the note's text, in UTF-16 units.
    required int start,

    /// Where it ends: before the line break.
    required int end,

    /// The runs on this line, in order and not overlapping, each cut to the
    /// line; text between them is plain.
    @Default(<StyledRun>[]) List<StyledRun> runs,
  }) = _SourceLine;
}
