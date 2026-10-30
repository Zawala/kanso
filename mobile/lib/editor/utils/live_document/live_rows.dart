// Kanso — live_rows.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../note/views/widgets/preview/preview_blocks.dart';
import '../../../note/views/widgets/preview/preview_list.dart';
import '../../../note/views/widgets/preview/preview_style.dart';
import 'live_layout_data.dart';

/// What a row of Live draws.
enum LiveRowKind {
  /// Text drawn in place, its marks hidden off the caret.
  text,

  /// A rule off the caret: a hairline.
  rule,

  /// A block drawn by the preview's renderer off the caret.
  widget,

  /// Lines drawn as nothing: blank lines and link definitions off the caret.
  hidden,
}

/// The colour a row's running text is in.
enum LiveTone { body, quote, done }

/// A list item's marker, drawn in the column left of its text.
@immutable
class LiveRowMarker {
  const LiveRowMarker({
    required this.kind,
    required this.range,
    required this.width,
    this.text = '',
    this.checked = false,
  });

  final LiveMarkKind kind;

  /// The source it stands for (`- `, `1. `, `- [ ] `).
  final TextRange range;

  /// How wide its column is.
  final double width;
  final String text;
  final bool checked;
}

/// A block of the note as Live draws it: one row of a lazy column, spaced,
/// set in and marked as the preview draws the same block.
@immutable
class LiveRow {
  const LiveRow({
    required this.kind,
    required this.range,
    this.above = 0,
    this.quoteDepth = 0,
    this.barsAbove = 0,
    this.barsLeft = 0,
    this.indent = 0,
    this.marker,
    this.headingLevel = 0,
    this.revealed = false,
    this.tone = LiveTone.body,
    this.preview,
  });

  final LiveRowKind kind;

  /// Its whole lines, without the line break after the last.
  final TextRange range;

  /// The room above it, in points.
  final double above;

  /// How many quote bars are drawn beside it, and how many of them run up
  /// through the room above it (the quote goes on from the row before).
  final int quoteDepth;
  final int barsAbove;

  /// Where the first bar is, from the row's left.
  final double barsLeft;

  /// Where its text starts, from the row's left.
  final double indent;

  final LiveRowMarker? marker;

  /// A heading's level; 0 for anything else.
  final int headingLevel;

  /// Whether the caret is on it: its source is drawn as written.
  final bool revealed;
  final LiveTone tone;

  /// The preview's block for it: what a widget row draws, and where a text
  /// row finds its due chips and formulas.
  final PreviewBlock? preview;

  /// Whether [offset] of the note is in this row, its ends included.
  bool holds(int offset) => offset >= range.start && offset <= range.end;
}

/// A top-level block of the preview, with the Markdown it was drawn from.
typedef LivePreviewBlock = ({String source, PreviewBlock block});

/// Lays out [layout] (the layout of the note's current text) as rows, with
/// the preview's metrics in [style]. [previews], the preview's top-level
/// blocks of a text of the note (the current one or one a little older),
/// give the widgets their drawings and the lists whether they are tight.
List<LiveRow> planLiveRows({
  required LiveLayoutData layout,
  required PreviewStyle style,
  List<LivePreviewBlock> previews = const <LivePreviewBlock>[],
}) {
  final text = layout.text;
  final blocks = layout.blocks;
  if (blocks.isEmpty) {
    return <LiveRow>[
      LiveRow(
        kind: LiveRowKind.text,
        range: TextRange(start: 0, end: text.length),
        revealed: true,
      ),
    ];
  }
  final marks = layout.marks;
  final widgets = <int, LiveWidgetData>{
    for (final widget in layout.widgets) widget.lines.start: widget,
  };

  // The marks on each block's first line.
  List<LiveMarkData> marksOn(LiveBlockData block) {
    final firstEnd = text.indexOf('\n', block.lines.start);
    final end = firstEnd < 0 || firstEnd > block.lines.end
        ? block.lines.end
        : firstEnd;
    final found = <LiveMarkData>[];
    var low = 0;
    var high = marks.length;
    while (low < high) {
      final middle = (low + high) >> 1;
      if (marks[middle].range.end <= block.lines.start) {
        low = middle + 1;
      } else {
        high = middle;
      }
    }
    for (var at = low; at < marks.length; at++) {
      final mark = marks[at];
      if (mark.range.start > end) {
        break;
      }
      if (mark.range.start >= block.lines.start) {
        found.add(mark);
      }
    }
    return found;
  }

  final markerWidth = PreviewList.markerWidth(style);
  double widthOf(LiveMarkKind kind) => switch (kind) {
    LiveMarkKind.task => PreviewTaskCheckbox.width,
    LiveMarkKind.hidden => 0,
    _ => markerWidth,
  };

  // Pass 1: the lists each block is in, and how far in its text is.
  final infos = List<_Info>.generate(blocks.length, (_) => _Info());
  final widths = <double>[];
  final lists = <int>[];
  var nextList = 0;
  int? top;
  final listsOfTop = <int?, List<int>>{};
  for (var at = 0; at < blocks.length; at++) {
    final block = blocks[at];
    final info = infos[at];
    if (!_shown(block)) {
      continue;
    }
    if (block.top != top || block.top == null) {
      widths.clear();
      lists.clear();
      top = block.top;
    }
    final depth = block.listDepth;
    if (lists.length > depth) {
      lists.removeRange(depth, lists.length);
    }
    if (widths.length > depth) {
      widths.removeRange(depth, widths.length);
    }
    final own = marksOn(block);
    if (block.itemStart && depth > 0) {
      LiveMarkData? listMark;
      for (final mark in own) {
        if (_isListMark(mark.kind)) {
          listMark = mark;
        }
      }
      final width = listMark != null
          ? widthOf(listMark.kind)
          : (_taskLine(text, block.lines.start)
                ? PreviewTaskCheckbox.width
                : markerWidth);
      if (lists.length < depth) {
        while (lists.length < depth - 1) {
          lists.add(nextList++);
        }
        final id = nextList++;
        lists.add(id);
        listsOfTop.putIfAbsent(block.top, () => <int>[]).add(id);
      }
      if (widths.length > depth - 1) {
        widths.removeRange(depth - 1, widths.length);
      }
      while (widths.length < depth - 1) {
        widths.add(markerWidth);
      }
      widths.add(width);
      if (listMark != null && !block.revealed) {
        info.marker = LiveRowMarker(
          kind: listMark.kind,
          range: listMark.range,
          width: width,
          text: listMark.text,
          checked: listMark.checked,
        );
      }
    } else {
      while (lists.length < depth) {
        final id = nextList++;
        lists.add(id);
        listsOfTop.putIfAbsent(block.top, () => <int>[]).add(id);
      }
      while (widths.length < depth) {
        widths.add(markerWidth);
      }
    }
    info
      ..list = depth > 0 ? lists[depth - 1] : null
      ..listIndent = widths.fold<double>(0, (sum, width) => sum + width);
    // A quote in a list item has its bars after the item's indent.
    var listBefore = -1;
    var quoteAt = -1;
    for (final mark in own) {
      if (_isListMark(mark.kind) && listBefore < 0) {
        listBefore = mark.range.start;
      }
      if (mark.kind == LiveMarkKind.quote && quoteAt < 0) {
        quoteAt = mark.range.start;
      }
    }
    info.quoteInList =
        depth > 0 && quoteAt >= 0 && listBefore >= 0 && listBefore < quoteAt;
  }

  // Which lists are loose: a blank line between two of their items, or
  // between two blocks of one of their items.
  final loose = <int>{};
  int? lastShown;
  var blankSince = false;
  for (var at = 0; at < blocks.length; at++) {
    final block = blocks[at];
    if (!_shown(block)) {
      if (block.kind == LiveBlockKind.blank) {
        blankSince = true;
      }
      continue;
    }
    final before = lastShown;
    if (blankSince &&
        before != null &&
        blocks[before].top == block.top &&
        block.top != null) {
      final previous = blocks[before];
      final next = block;
      final nextList = infos[at].list;
      final previousList = infos[before].list;
      if (next.itemStart && next.listDepth > 0) {
        if (previous.listDepth >= next.listDepth && nextList != null) {
          loose.add(nextList);
        } else if (previous.listDepth > 0 && previousList != null) {
          loose.add(previousList);
        }
      } else if (next.listDepth > 0 && nextList != null) {
        loose.add(nextList);
      }
    }
    blankSince = false;
    lastShown = at;
  }

  // The preview's top-level blocks matched to the layout's by their source.
  final unused = <String, List<PreviewBlock>>{};
  for (final preview in previews) {
    unused
        .putIfAbsent(preview.source, () => <PreviewBlock>[])
        .add(
          preview.block,
        );
  }
  final tops = <int, TextRange>{};
  for (final block in blocks) {
    final index = block.top;
    if (index == null || !_shown(block)) {
      continue;
    }
    final known = tops[index];
    tops[index] = known == null
        ? block.lines
        : TextRange(
            start: math.min(known.start, block.lines.start),
            end: math.max(known.end, block.lines.end),
          );
  }
  final leafOf = <int, PreviewBlock>{};
  final tight = <int, bool>{};
  for (final MapEntry(key: index, value: range) in tops.entries) {
    final source = text.substring(range.start, range.end);
    final candidates = unused[source];
    if (candidates == null || candidates.isEmpty) {
      continue;
    }
    final preview = candidates.removeAt(0);
    final previewLeaves = <PreviewBlock>[];
    final previewLists = <bool>[];
    _flatten(preview, previewLeaves, previewLists);
    final ours = <int>[
      for (var at = 0; at < blocks.length; at++)
        if (blocks[at].top == index && _shown(blocks[at])) at,
    ];
    if (ours.length == previewLeaves.length) {
      for (var i = 0; i < ours.length; i++) {
        leafOf[ours[i]] = previewLeaves[i];
      }
    }
    final ourLists = listsOfTop[index] ?? const <int>[];
    if (ourLists.length == previewLists.length) {
      for (var i = 0; i < ourLists.length; i++) {
        tight[ourLists[i]] = previewLists[i];
      }
    }
  }
  bool tightList(int? list) =>
      list != null && (tight[list] ?? !loose.contains(list));

  // Pass 2: the rows, spaced.
  final rows = <LiveRow>[];
  int? previous;
  var at = 0;
  while (at < blocks.length) {
    final block = blocks[at];
    if (!_shown(block)) {
      // A run of lines drawn as nothing, unless the caret is on them.
      final start = at;
      while (at < blocks.length && !_shown(blocks[at])) {
        at++;
      }
      final run = blocks.sublist(start, at);
      final revealed = run.any((b) => b.revealed);
      if (!revealed) {
        for (final hidden in run) {
          rows.add(LiveRow(kind: LiveRowKind.hidden, range: hidden.lines));
        }
        continue;
      }
      // Drawn as written; the room between the blocks around them above
      // them, or a paragraph's after the last block.
      final next = at < blocks.length ? at : null;
      var room = 0.0;
      final before = previous;
      if (before != null) {
        room = next != null
            ? _between(blocks, infos, before, next, style, tightList)
            : _between(
                blocks,
                infos,
                before,
                null,
                style,
                tightList,
              );
      }
      for (var i = 0; i < run.length; i++) {
        rows.add(
          LiveRow(
            kind: LiveRowKind.text,
            range: run[i].lines,
            above: i == 0 ? room : 0,
            revealed: true,
          ),
        );
      }
      // The block after them has had its room.
      previous = null;
      continue;
    }
    final info = infos[at];
    final before = previous;
    final room = before == null
        ? 0.0
        : _between(blocks, infos, before, at, style, tightList);
    final barsAbove = before == null || blocks[before].top != block.top
        ? 0
        : math.min(blocks[before].quoteDepth, block.quoteDepth);
    final quoteX = block.quoteDepth * PreviewStyle.quoteStep;
    final widget = widgets[block.lines.start];
    final done = _styled(layout.styles, block.lines, LiveStyleKind.taskDone);
    final tone = done
        ? LiveTone.done
        : (block.quoteDepth > 0 ? LiveTone.quote : LiveTone.body);
    final kind = widget != null
        ? LiveRowKind.widget
        : (block.kind == LiveBlockKind.rule && !block.revealed
              ? LiveRowKind.rule
              : LiveRowKind.text);
    final revealed = block.revealed && widget == null;
    rows.add(
      LiveRow(
        kind: kind,
        range: widget?.lines ?? block.lines,
        above: room,
        quoteDepth: revealed ? 0 : block.quoteDepth,
        barsAbove: revealed ? 0 : barsAbove,
        barsLeft: info.quoteInList ? info.listIndent : 0,
        indent: revealed ? 0 : quoteX + info.listIndent,
        marker: info.marker,
        headingLevel: block.kind == LiveBlockKind.heading ? block.level : 0,
        revealed: revealed,
        tone: tone,
        preview: leafOf[at],
      ),
    );
    previous = at;
    at++;
  }
  return rows;
}

class _Info {
  int? list;
  double listIndent = 0;
  LiveRowMarker? marker;
  bool quoteInList = false;
}

/// Whether a block is drawn at all: blank lines and link definitions are
/// not, off the caret (the preview leaves definitions out).
bool _shown(LiveBlockData block) =>
    block.kind != LiveBlockKind.blank && block.kind != LiveBlockKind.definition;

bool _isListMark(LiveMarkKind kind) => switch (kind) {
  LiveMarkKind.bullet ||
  LiveMarkKind.number ||
  LiveMarkKind.task ||
  LiveMarkKind.hidden => true,
  _ => false,
};

final RegExp _taskPattern = RegExp(
  r'^[ \t]*(?:>[ \t]?)*[ \t]*(?:[-*+]|\d{1,9}[.)])[ \t]+\[[ xX]\]',
);

/// Whether the line at [start] of [text] is a task item's.
bool _taskLine(String text, int start) {
  final end = text.indexOf('\n', start);
  return _taskPattern.hasMatch(
    text.substring(start, end < 0 ? text.length : end),
  );
}

bool _styled(List<LiveStyleData> styles, TextRange lines, LiveStyleKind kind) {
  for (final style in styles) {
    if (style.range.start > lines.end) {
      break;
    }
    if (style.kind == kind &&
        style.range.end > lines.start &&
        style.range.start <= lines.end) {
      return true;
    }
  }
  return false;
}

/// The preview's leaf blocks of [block] in order, and whether each list in
/// it is tight, in the order the lists start.
void _flatten(
  PreviewBlock block,
  List<PreviewBlock> leaves,
  List<bool> lists,
) {
  switch (block) {
    case PreviewBlock_List(:final tight, :final items):
      lists.add(tight);
      for (final item in items) {
        for (final inner in item.blocks) {
          _flatten(inner, leaves, lists);
        }
      }
    case PreviewBlock_Quote(:final blocks):
      for (final inner in blocks) {
        _flatten(inner, leaves, lists);
      }
    default:
      leaves.add(block);
  }
}

/// What a block is to the room around it.
PreviewRoom _roomKind(LiveBlockData block) => switch (block.kind) {
  LiveBlockKind.heading => PreviewRoom.heading,
  LiveBlockKind.rule => PreviewRoom.rule,
  LiveBlockKind.paragraph ||
  LiveBlockKind.html ||
  LiveBlockKind.image ||
  LiveBlockKind.blank ||
  LiveBlockKind.definition => PreviewRoom.text,
  LiveBlockKind.math => PreviewRoom.math,
  _ => PreviewRoom.other,
};

/// The room between the block [a] and the next drawn, [b] (none: a
/// paragraph after [a], for lines typed after the last block), as the
/// preview folds the room below one block and above the next.
double _between(
  List<LiveBlockData> blocks,
  List<_Info> infos,
  int a,
  int? b,
  PreviewStyle style,
  bool Function(int? list) tightList,
) {
  final first = blocks[a];
  final next = b == null
      ? const LiveBlockData(
          kind: LiveBlockKind.paragraph,
          lines: TextRange.empty,
        )
      : blocks[b];
  final firstTight = first.listDepth > 0 && tightList(infos[a].list);
  final nextTight = b != null && next.listDepth > 0 && tightList(infos[b].list);
  (double, double) roomOf(LiveBlockData block, {required bool tight}) =>
      PreviewBlocks.room(
        _roomKind(block),
        style,
        level: math.max(1, block.level),
        tight: tight,
      );
  final bothHeadings =
      first.kind == LiveBlockKind.heading && next.kind == LiveBlockKind.heading;
  if (first.top != next.top || first.top == null) {
    // Two top-level blocks: a list or a quote is spaced as a whole.
    (double, double) topRoom(LiveBlockData block) =>
        block.listDepth > 0 || block.quoteDepth > 0
        ? PreviewBlocks.room(PreviewRoom.other, style)
        : roomOf(block, tight: false);
    final (_, below) = topRoom(first);
    var (above, _) = topRoom(next);
    if (bothHeadings &&
        first.listDepth == 0 &&
        first.quoteDepth == 0 &&
        next.listDepth == 0 &&
        next.quoteDepth == 0) {
      above = PreviewBlocks.headingAfterHeading(next.level, style);
    }
    return math.max(above, below);
  }
  if (next.itemStart &&
      next.listDepth > 0 &&
      next.listDepth <= first.listDepth) {
    // The next item of a list.
    return PreviewList.itemGap(style, tight: tightList(infos[b!].list));
  }
  if (next.quoteDepth < first.quoteDepth || next.listDepth < first.listDepth) {
    // Past the end of a list or a quote, which keeps a gap below it.
    final (above, _) = roomOf(next, tight: nextTight);
    return math.max(style.gap, above);
  }
  if (next.quoteDepth > first.quoteDepth || next.listDepth > first.listDepth) {
    // A list or a quote starts, with no room above it of its own.
    final (_, below) = roomOf(first, tight: firstTight);
    return below;
  }
  final (_, below) = roomOf(first, tight: firstTight);
  var (above, _) = roomOf(next, tight: nextTight);
  if (bothHeadings) {
    above = PreviewBlocks.headingAfterHeading(next.level, style);
  }
  return math.max(above, below);
}
