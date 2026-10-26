// Kanso — source_lines.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import '../../note/models/data/source_line.dart';
import '../bridge/api/types.dart';

/// [text] cut into its lines, each with the [runs] that fall on it.
///
/// A line is the text between two line breaks (`\n`, or `\r\n`), without
/// the break; a text with `n` breaks has `n + 1` lines, and the last may be
/// empty. A run that spans lines is cut at each break. Runs are taken in
/// the order of their start; the part of a run that overlaps what an
/// earlier run covered is left out, and so is an empty run or the part of
/// one outside the text. Offsets are UTF-16 units, as Dart strings index.
///
/// The time it takes grows with the size of the text and the runs, so a
/// large note is cut as fast as it is read.
List<SourceLine> sourceLines(String text, List<StyledRun> runs) {
  final ordered = <StyledRun>[...runs]
    ..sort((a, b) => a.range.start.compareTo(b.range.start));
  // The runs as they are laid down: in order, inside the text, and with
  // nothing covered twice.
  final laid = <StyledRun>[];
  var covered = 0;
  for (final run in ordered) {
    final from = math.max(run.range.start, covered);
    final to = math.min(run.range.end, text.length);
    if (to > from) {
      laid.add(_cut(run, from, to));
      covered = to;
    }
  }
  final lines = <SourceLine>[];
  var first = 0;
  var lineStart = 0;
  while (true) {
    final newline = text.indexOf('\n', lineStart);
    final hardEnd = newline < 0 ? text.length : newline;
    final lineEnd = hardEnd > lineStart && text.codeUnitAt(hardEnd - 1) == 13
        ? hardEnd - 1
        : hardEnd;
    // Runs that ended before this line cannot touch it or a later one.
    while (first < laid.length && laid[first].range.end <= lineStart) {
      first++;
    }
    final onLine = <StyledRun>[];
    for (var at = first; at < laid.length; at++) {
      final run = laid[at];
      if (run.range.start >= lineEnd) {
        break;
      }
      final from = math.max(run.range.start, lineStart);
      final to = math.min(run.range.end, lineEnd);
      if (to > from) {
        onLine.add(_cut(run, from, to));
      }
    }
    lines.add(SourceLine(start: lineStart, end: lineEnd, runs: onLine));
    if (newline < 0) {
      return lines;
    }
    lineStart = newline + 1;
  }
}

StyledRun _cut(StyledRun run, int from, int to) {
  if (from == run.range.start && to == run.range.end) {
    return run;
  }
  return StyledRun(
    range: Range16(start: from, end: to),
    color: run.color,
    italic: run.italic,
    weight: run.weight,
  );
}
