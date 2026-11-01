// Kanso — live_layer.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart' show TextRange;
import 'package:flutter/services.dart' show TextSelection;

import '../../core/bridge/api/types.dart';
import 'shift_runs.dart';

/// What Live mode draws over the editor's text, in units of the field: the
/// marks hidden, the spans styled, and the heading lines drawn larger.
///
/// The Rust core says it for every line ([LiveLayer.fromBatch]); the field
/// shows the marks again on the lines its selection touches ([revealing]),
/// so a caret that moves needs nothing new from Rust.
class LiveLayer {
  const LiveLayer({
    this.conceal = const <TextRange>[],
    this.runs = const <(TextRange, LiveKind)>[],
    this.lines = const <(TextRange, double)>[],
  });

  /// [batch] as a layer.
  factory LiveLayer.fromBatch(LiveBatch batch) => LiveLayer(
    conceal: <TextRange>[
      for (final range in batch.conceal)
        TextRange(start: range.start, end: range.end),
    ],
    runs: <(TextRange, LiveKind)>[
      for (final run in batch.runs)
        (TextRange(start: run.range.start, end: run.range.end), run.kind),
    ],
    lines: <(TextRange, double)>[
      for (final line in batch.lines)
        (TextRange(start: line.range.start, end: line.range.end), line.scale),
    ],
  );

  /// Nothing drawn differently.
  static const LiveLayer empty = LiveLayer();

  /// The units not drawn: in order, apart, each within one line.
  final List<TextRange> conceal;

  /// The spans drawn bold, slanted, struck, as code or as a link; they can
  /// nest.
  final List<(TextRange, LiveKind)> runs;

  /// The heading lines (without their line breaks) and how much larger
  /// they are drawn.
  final List<(TextRange, double)> lines;

  bool get isEmpty => conceal.isEmpty && runs.isEmpty && lines.isEmpty;

  /// How much larger the line [offset] is on is drawn: 1 off a heading.
  double scaleAt(int offset) {
    for (final (range, scale) in lines) {
      if (range.start <= offset && offset <= range.end) {
        return scale;
      }
    }
    return 1;
  }

  /// The layer with the marks on [revealed] shown again.
  LiveLayer revealing(TextRange revealed) {
    if (!revealed.isValid || conceal.isEmpty) {
      return this;
    }
    final kept = <TextRange>[
      for (final range in conceal)
        if (range.end <= revealed.start || range.start >= revealed.end) range,
    ];
    return kept.length == conceal.length
        ? this
        : LiveLayer(conceal: kept, runs: runs, lines: lines);
  }

  /// The layer moved to fit a text in which the units [start] to [end] were
  /// replaced by [inserted] units, until the Rust core says it afresh: a
  /// hidden mark the change touches is shown, a styled span the change is
  /// in grows or shrinks with it, and a heading line grows with typing at
  /// its end.
  LiveLayer shifted({
    required int start,
    required int end,
    required int inserted,
  }) {
    if (isEmpty) {
      return this;
    }
    final movedRuns = <(TextRange, LiveKind)>[];
    for (final (range, kind) in runs) {
      final moved = _shift(range, start, end, inserted, grows: false);
      if (moved != null) {
        movedRuns.add((moved, kind));
      }
    }
    final movedLines = <(TextRange, double)>[];
    for (final (range, scale) in lines) {
      final moved = _shift(range, start, end, inserted, grows: true);
      if (moved != null) {
        movedLines.add((moved, scale));
      }
    }
    return LiveLayer(
      conceal: shiftRanges(
        conceal,
        start: start,
        end: end,
        inserted: inserted,
      ),
      runs: movedRuns,
      lines: movedLines,
    );
  }

  /// The layer of a window that starts [delta] units before this one's
  /// (after, when negative) and is [length] units long: what falls in it,
  /// cut to it, until the Rust core says it afresh.
  LiveLayer moved(int delta, int length) {
    if (isEmpty) {
      return this;
    }
    TextRange? inside(TextRange range) {
      final start = range.start + delta;
      final end = range.end + delta;
      if (end <= 0 || start >= length) {
        return null;
      }
      return TextRange(
        start: start.clamp(0, length),
        end: end.clamp(0, length),
      );
    }

    final conceal = <TextRange>[];
    for (final range in this.conceal) {
      final kept = inside(range);
      if (kept != null) {
        conceal.add(kept);
      }
    }
    final runs = <(TextRange, LiveKind)>[];
    for (final (range, kind) in this.runs) {
      final kept = inside(range);
      if (kept != null) {
        runs.add((kept, kind));
      }
    }
    final lines = <(TextRange, double)>[];
    for (final (range, scale) in this.lines) {
      final kept = inside(range);
      if (kept != null) {
        lines.add((kept, scale));
      }
    }
    return LiveLayer(conceal: conceal, runs: runs, lines: lines);
  }

  static TextRange? _shift(
    TextRange range,
    int start,
    int end,
    int inserted, {
    required bool grows,
  }) {
    final delta = inserted - (end - start);
    if (range.end < start || (range.end == start && !grows)) {
      return range;
    }
    if (range.start >= end) {
      return TextRange(start: range.start + delta, end: range.end + delta);
    }
    final from = range.start <= start ? range.start : start + inserted;
    final to = range.end >= end
        ? range.end + delta
        : (grows ? start + inserted : start);
    return to > from || (grows && to == from)
        ? TextRange(start: from, end: to)
        : null;
  }
}

/// The lines of [text] that [selection] touches, from the start of the
/// first to the end of the last (without its line break); empty for no
/// selection.
TextRange revealedLines(String text, TextSelection selection) {
  if (!selection.isValid) {
    return TextRange.empty;
  }
  final first = selection.start.clamp(0, text.length);
  final last = selection.end.clamp(first, text.length);
  final start = first == 0 ? 0 : text.lastIndexOf('\n', first - 1) + 1;
  final newline = text.indexOf('\n', last);
  return TextRange(start: start, end: newline < 0 ? text.length : newline);
}
