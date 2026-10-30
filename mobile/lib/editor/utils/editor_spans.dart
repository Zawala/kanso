// Kanso — editor_spans.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

import '../../core/bridge/api/types.dart';
import '../../core/utils/run_style.dart';
import 'diff_values.dart';
import 'live_layer.dart';

/// The size hidden text is drawn at in Live mode: kept in the text, so
/// every offset stays where it is, but too small and clear to be seen.
const double liveHiddenSize = 0.01;

const TextStyle _hiddenStyle = TextStyle(
  fontSize: liveHiddenSize,
  color: Color(0x00000000),
  backgroundColor: Color(0x00000000),
  decoration: TextDecoration.none,
  letterSpacing: 0,
  wordSpacing: 0,
);
const TextStyle _bold = TextStyle(fontWeight: FontWeight.bold);
const TextStyle _slanted = TextStyle(fontStyle: FontStyle.italic);

/// The text of the editor as spans: each styled run in the colour, slant and
/// weight the theme gives it, the [composing] range underlined, and the
/// [misspellings] marked with [misspelledStyle], except where a run is
/// [EditorRun.quiet] (code, addresses, front matter).
///
/// The spans are the text cut at every place the style changes; joined, they
/// are [text] exactly. No span starts or ends between the two halves of a
/// surrogate pair, whatever the ranges say. [runs] must be in order and not
/// overlap; a run or range outside the text is cut to it.
///
/// A composing range is underlined with a plain line, and wins over a
/// misspelling where they meet. The words [comments] are on are drawn
/// with [commentStyle] as well, code and addresses included, and what other
/// people have selected ([selections]) on a wash of their colour, which
/// wins over a comment's.
///
/// In Live mode, [live] says what is drawn differently: its hidden marks
/// tiny and clear, its heading lines larger, and its spans bold, slanted,
/// struck through, in [liveCodeStyle] or in [liveLinkStyle]. Its spans
/// must be in order of their starts.
TextSpan editorSpans({
  required String text,
  required List<EditorRun> runs,
  required TextStyle base,
  TextRange composing = TextRange.empty,
  List<TextRange> misspellings = const <TextRange>[],
  TextStyle composingStyle = const TextStyle(
    decoration: TextDecoration.underline,
  ),
  TextStyle misspelledStyle = const TextStyle(
    decoration: TextDecoration.underline,
    decorationStyle: TextDecorationStyle.wavy,
  ),
  List<TextRange> comments = const <TextRange>[],
  TextStyle commentStyle = const TextStyle(backgroundColor: Color(0x33F5A524)),
  List<(TextRange, Color)> selections = const <(TextRange, Color)>[],
  LiveLayer live = LiveLayer.empty,
  TextStyle liveCodeStyle = const TextStyle(),
  TextStyle liveLinkStyle = const TextStyle(),
}) {
  final length = text.length;
  int snap(int at) {
    final clamped = at.clamp(0, length);
    return splitsPair(text, clamped) ? clamped - 1 : clamped;
  }

  // Where the style may change.
  final stops = <int>{0, length};
  for (final run in runs) {
    stops
      ..add(snap(run.range.start))
      ..add(snap(run.range.end));
  }
  final composed = composing.isValid && !composing.isCollapsed
      ? TextRange(start: snap(composing.start), end: snap(composing.end))
      : TextRange.empty;
  if (!composed.isCollapsed) {
    stops
      ..add(composed.start)
      ..add(composed.end);
  }
  final marks = <TextRange>[
    for (final range in misspellings)
      if (range.isValid && !range.isCollapsed)
        TextRange(start: snap(range.start), end: snap(range.end)),
  ];
  for (final mark in marks) {
    stops
      ..add(mark.start)
      ..add(mark.end);
  }
  final commented = <TextRange>[
    for (final range in comments)
      if (range.isValid && !range.isCollapsed)
        TextRange(start: snap(range.start), end: snap(range.end)),
  ];
  for (final range in commented) {
    stops
      ..add(range.start)
      ..add(range.end);
  }
  final selected = <(TextRange, Color)>[
    for (final (range, color) in selections)
      if (range.isValid && !range.isCollapsed)
        (TextRange(start: snap(range.start), end: snap(range.end)), color),
  ];
  for (final (range, _) in selected) {
    stops
      ..add(range.start)
      ..add(range.end);
  }
  final hidden = <TextRange>[
    for (final range in live.conceal)
      if (range.isValid && !range.isCollapsed)
        TextRange(start: snap(range.start), end: snap(range.end)),
  ];
  final liveRuns = <(TextRange, LiveKind)>[
    for (final (range, kind) in live.runs)
      if (range.isValid && !range.isCollapsed)
        (TextRange(start: snap(range.start), end: snap(range.end)), kind),
  ];
  final headings = <(TextRange, double)>[
    for (final (range, scale) in live.lines)
      if (range.isValid && !range.isCollapsed)
        (TextRange(start: snap(range.start), end: snap(range.end)), scale),
  ];
  for (final range in hidden) {
    stops
      ..add(range.start)
      ..add(range.end);
  }
  for (final (range, _) in liveRuns) {
    stops
      ..add(range.start)
      ..add(range.end);
  }
  for (final (range, _) in headings) {
    stops
      ..add(range.start)
      ..add(range.end);
  }
  final ordered = stops.toList()..sort();

  if (ordered.length <= 2 &&
      runs.isEmpty &&
      composed.isCollapsed &&
      commented.isEmpty &&
      selected.isEmpty &&
      liveRuns.isEmpty &&
      headings.isEmpty) {
    return TextSpan(text: text, style: base);
  }
  final fontSize = base.fontSize ?? 14;

  // One sweep over the runs and the marks, which are in order.
  final spans = <InlineSpan>[];
  var run = 0;
  var mark = 0;
  var hide = 0;
  var heading = 0;
  var nextLive = 0;
  final activeLive = <(TextRange, LiveKind)>[];
  final sortedMarks = marks..sort((a, b) => a.start.compareTo(b.start));
  for (var i = 0; i + 1 < ordered.length; i++) {
    final from = ordered[i];
    final to = ordered[i + 1];
    if (to <= from) {
      continue;
    }
    while (run < runs.length && snap(runs[run].range.end) <= from) {
      run++;
    }
    final here = run < runs.length && snap(runs[run].range.start) <= from
        ? runs[run]
        : null;
    // Marks can overlap each other: look at all that reach this span.
    while (mark < sortedMarks.length && sortedMarks[mark].end <= from) {
      mark++;
    }
    var misspelled = false;
    for (var m = mark; m < sortedMarks.length; m++) {
      if (sortedMarks[m].start > from) {
        break;
      }
      if (sortedMarks[m].end > from) {
        misspelled = true;
        break;
      }
    }
    final inComposing =
        !composed.isCollapsed && composed.start <= from && composed.end >= to;

    TextStyle? style = here == null
        ? null
        : runStyle(color: here.color, italic: here.italic, weight: here.weight);
    while (heading < headings.length && headings[heading].$1.end <= from) {
      heading++;
    }
    if (heading < headings.length && headings[heading].$1.start <= from) {
      final larger = TextStyle(fontSize: fontSize * headings[heading].$2);
      style = style?.merge(larger) ?? larger;
    }
    while (nextLive < liveRuns.length && liveRuns[nextLive].$1.start <= from) {
      activeLive.add(liveRuns[nextLive]);
      nextLive++;
    }
    activeLive.removeWhere((entry) => entry.$1.end <= from);
    var struck = false;
    for (final (_, kind) in activeLive) {
      struck = struck || kind == LiveKind.strikethrough;
      final drawn = switch (kind) {
        LiveKind.strong || LiveKind.heading => _bold,
        LiveKind.emphasis => _slanted,
        LiveKind.code => liveCodeStyle,
        LiveKind.link => liveLinkStyle,
        LiveKind.strikethrough => null,
      };
      if (drawn != null) {
        style = style?.merge(drawn) ?? drawn;
      }
    }
    if (inComposing) {
      style = style?.merge(composingStyle) ?? composingStyle;
    } else if (misspelled && !(here?.quiet ?? false)) {
      style = style?.merge(misspelledStyle) ?? misspelledStyle;
    }
    if (commented.any((range) => range.start <= from && range.end >= to)) {
      style = style?.merge(commentStyle) ?? commentStyle;
    }
    for (final (range, color) in selected) {
      if (range.start <= from && range.end >= to) {
        final wash = TextStyle(backgroundColor: color);
        style = style?.merge(wash) ?? wash;
        break;
      }
    }
    if (struck) {
      final under = style?.decoration ?? TextDecoration.none;
      final lines = <TextDecoration>[under, TextDecoration.lineThrough];
      final strike = TextStyle(decoration: TextDecoration.combine(lines));
      style = style?.merge(strike) ?? strike;
    }
    while (hide < hidden.length && hidden[hide].end <= from) {
      hide++;
    }
    if (hide < hidden.length && hidden[hide].start <= from) {
      style = style?.merge(_hiddenStyle) ?? _hiddenStyle;
    }
    spans.add(TextSpan(text: text.substring(from, to), style: style));
  }
  return TextSpan(style: base, children: spans);
}
