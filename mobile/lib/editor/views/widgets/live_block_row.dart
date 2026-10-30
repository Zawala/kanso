// Kanso — live_block_row.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

import '../../../note/views/widgets/preview/preview_blocks.dart';
import '../../../note/views/widgets/preview/preview_list.dart';
import '../../../note/views/widgets/preview/preview_list_marker.dart';
import '../../../note/views/widgets/preview/preview_style.dart';
import '../../utils/live_document/live_display.dart';
import '../../utils/live_document/live_layout_data.dart';
import '../../utils/live_document/live_rows.dart';

/// What a row of the Live editor asks of the editor it is in: the
/// selection to draw, where the handles hang, and what a touch on it does.
abstract interface class LiveRowHost {
  /// The selection, in units of the note.
  TextSelection get selection;

  /// Whether the caret is drawn now (focused, collapsed, blinking on), and
  /// in which row.
  bool get caretShown;
  int? get caretRow;

  /// What repaints the rows' selection and caret.
  Listenable get repaint;

  /// Where the selection's handles hang.
  LayerLink get startLink;
  LayerLink get endLink;

  /// Whether the handles are being shown, so the rows hang them.
  bool get handlesShown;

  /// The rows the selection's start and end are drawn in.
  int? get startRow;
  int? get endRow;

  Color get selectionColor;
  Color get caretColor;

  /// Other people's carets in [row], with their colours, in units of the
  /// note.
  List<(int, Color)> remoteCaretsIn(LiveRow row);

  /// What is laid over [row]'s text.
  LiveOverlays overlaysFor(LiveRow row);

  void register(int index, LiveBlockRowState row);
  void unregister(int index, LiveBlockRowState row);

  /// A tap (or several), a press held and moved, a mouse drag, on the row
  /// at [index], at a point on the screen.
  void tapped(int index, Offset global, int count, PointerDeviceKind kind);
  void dragged(Offset global, {required bool start});
  void held(Offset global, {required bool start});
  void released(Offset global);

  /// The task box of [marker] was tapped.
  void toggleTask(LiveRowMarker marker);

  /// An address in a block drawn by the preview's renderer was tapped.
  void widgetLink(LiveRow row, String url);
}

/// A row of the Live editor: a block of the note drawn as the preview
/// draws it, spaced and set in as there, with the selection and the caret
/// over its text.
class LiveBlockRow extends StatefulWidget {
  const LiveBlockRow({
    required this.index,
    required this.row,
    required this.layout,
    required this.style,
    required this.host,
    super.key,
  });

  final int index;
  final LiveRow row;

  /// The layout the row is part of.
  final LiveLayoutData layout;
  final PreviewStyle style;
  final LiveRowHost host;

  @override
  State<LiveBlockRow> createState() => LiveBlockRowState();
}

class LiveBlockRowState extends State<LiveBlockRow> {
  final GlobalKey _text = GlobalKey(debugLabel: 'live row text');
  LiveDisplay? _display;
  LiveDisplay? get display => _display;
  LiveRow get row => widget.row;

  /// The row's own box.
  RenderBox? get box {
    final found = context.findRenderObject();
    return found is RenderBox && found.attached && found.hasSize ? found : null;
  }

  /// Its text, laid out: none for a rule, a widget or hidden lines.
  RenderParagraph? get paragraph {
    final found = _text.currentContext?.findRenderObject();
    return found is RenderParagraph && found.attached && found.hasSize
        ? found
        : null;
  }

  @override
  void initState() {
    super.initState();
    widget.host.register(widget.index, this);
  }

  @override
  void didUpdateWidget(LiveBlockRow oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.index != widget.index || oldWidget.host != widget.host) {
      oldWidget.host.unregister(oldWidget.index, this);
      widget.host.register(widget.index, this);
    }
  }

  @override
  void dispose() {
    widget.host.unregister(widget.index, this);
    super.dispose();
  }

  /// Where [offset] of the note is drawn in the row's box, and how tall
  /// its line is; null when the row has no text there.
  Rect? caretRect(int offset) {
    final text = paragraph;
    final shown = _display;
    final own = box;
    if (text == null || shown == null || own == null) {
      return null;
    }
    final position = TextPosition(offset: shown.toDisplay(offset));
    final at = text.getOffsetForCaret(position, Rect.zero);
    final height = text.getFullHeightForCaret(position);
    final topLeft = text.localToGlobal(at, ancestor: own);
    return Rect.fromLTWH(topLeft.dx, topLeft.dy, 0, height);
  }

  /// The offset of the note nearest [global], a point on the screen, in
  /// this row.
  int offsetAt(Offset global) {
    final text = paragraph;
    final shown = _display;
    if (text == null || shown == null || row.kind != LiveRowKind.text) {
      return switch (row.kind) {
        LiveRowKind.rule => row.range.end,
        _ => row.range.start,
      };
    }
    final local = text.globalToLocal(global);
    final position = text.getPositionForOffset(local);
    return shown.toSource(position.offset);
  }

  /// The offset a line up or down from [offset] in this row, at [globalX]
  /// across the screen; null when that line is in another row.
  int? verticalFrom(int offset, double globalX, {required bool down}) {
    final text = paragraph;
    final shown = _display;
    if (text == null || shown == null) {
      return null;
    }
    final position = TextPosition(offset: shown.toDisplay(offset));
    final at = text.getOffsetForCaret(position, Rect.zero);
    final height = text.getFullHeightForCaret(position);
    final y = down ? at.dy + height + 1 : at.dy - 1;
    if (y < 0 || y >= text.size.height) {
      return null;
    }
    final x = text.globalToLocal(Offset(globalX, 0)).dx;
    return shown.toSource(text.getPositionForOffset(Offset(x, y)).offset);
  }

  /// The offset on the row's first line (or its [last]) at [globalX], for
  /// a caret coming into it from above (or below).
  int edgeOffset(double globalX, {required bool last}) {
    final text = paragraph;
    final shown = _display;
    if (text == null || shown == null || row.kind != LiveRowKind.text) {
      return last ? row.range.end : row.range.start;
    }
    final x = text.globalToLocal(Offset(globalX, 0)).dx;
    final y = last ? text.size.height - 1 : 1.0;
    return shown.toSource(text.getPositionForOffset(Offset(x, y)).offset);
  }

  /// The boxes of [start] to [end] of the note in the row's box.
  List<Rect> boxesFor(int start, int end) {
    final text = paragraph;
    final shown = _display;
    final own = box;
    if (text == null || shown == null || own == null) {
      return const <Rect>[];
    }
    final from = shown.toDisplay(start);
    final to = shown.toDisplay(end);
    if (from >= to) {
      return const <Rect>[];
    }
    final shift = text.localToGlobal(Offset.zero, ancestor: own);
    return <Rect>[
      for (final b in text.getBoxesForSelection(
        TextSelection(baseOffset: from, extentOffset: to),
      ))
        b.toRect().shift(shift),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final row = widget.row;
    final style = widget.style;
    final host = widget.host;
    if (row.kind == LiveRowKind.hidden) {
      _display = null;
      return const SizedBox.shrink();
    }
    final tone = switch (row.tone) {
      LiveTone.body => style,
      LiveTone.quote => style.withText(style.quote),
      LiveTone.done => style.withText(style.taskDone),
    };
    Widget content;
    switch (row.kind) {
      case LiveRowKind.rule:
        _display = null;
        content = Container(height: 1, color: style.border);
      case LiveRowKind.widget when row.preview != null:
        _display = null;
        final scope = PreviewScope.of(context);
        content = PreviewScope(
          style: tone,
          onLink: (url) => host.widgetLink(row, url),
          headingKey: scope.headingKey,
          child: PreviewBlockView(row.preview!),
        );
      case LiveRowKind.text || LiveRowKind.widget || LiveRowKind.hidden:
        final shown = buildLiveDisplay(
          layout: widget.layout,
          row: row,
          style: style,
          overlays: host.overlaysFor(row),
        );
        _display = shown;
        content = _LiveText(
          state: this,
          display: shown,
          textKey: _text,
        );
    }
    final marker = row.marker;
    final Widget line;
    if (marker == null) {
      line = Padding(
        padding: EdgeInsets.only(left: row.indent),
        child: content,
      );
    } else {
      final Widget drawn = switch (marker.kind) {
        LiveMarkKind.task => PreviewScope(
          style: style,
          onLink: (_) => host.toggleTask(marker),
          headingKey: PreviewScope.of(context).headingKey,
          child: PreviewTaskCheckbox(checked: marker.checked, index: 0),
        ),
        LiveMarkKind.hidden => const SizedBox.shrink(),
        _ => PreviewListMarker(marker.text),
      };
      line = Padding(
        padding: EdgeInsets.only(left: row.indent - marker.width),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            SizedBox(width: marker.width, child: drawn),
            Expanded(child: content),
          ],
        ),
      );
    }
    Widget drawn = Padding(
      padding: EdgeInsets.only(top: row.above),
      child: line,
    );
    if (row.quoteDepth > 0) {
      drawn = CustomPaint(
        painter: _QuoteBars(
          depth: row.quoteDepth,
          barsAbove: row.barsAbove,
          above: row.above,
          left: row.barsLeft,
          color: style.quoteBar,
        ),
        child: drawn,
      );
    }
    return RawGestureDetector(
      behavior: HitTestBehavior.opaque,
      gestures: <Type, GestureRecognizerFactory>{
        TapAndPanGestureRecognizer:
            GestureRecognizerFactoryWithHandlers<TapAndPanGestureRecognizer>(
              () => TapAndPanGestureRecognizer(debugOwner: this),
              (recognizer) {
                recognizer
                  ..dragStartBehavior = DragStartBehavior.down
                  ..onTapUp = (details) {
                    host.tapped(
                      widget.index,
                      details.globalPosition,
                      details.consecutiveTapCount,
                      details.kind,
                    );
                  }
                  ..onDragStart = (details) {
                    if (details.kind == PointerDeviceKind.mouse) {
                      host.dragged(details.globalPosition, start: true);
                    }
                  }
                  ..onDragUpdate = (details) {
                    if (details.kind == PointerDeviceKind.mouse) {
                      host.dragged(details.globalPosition, start: false);
                    }
                  };
              },
            ),
        LongPressGestureRecognizer:
            GestureRecognizerFactoryWithHandlers<LongPressGestureRecognizer>(
              () => LongPressGestureRecognizer(debugOwner: this),
              (recognizer) {
                recognizer
                  ..onLongPressStart = (details) {
                    host.held(details.globalPosition, start: true);
                  }
                  ..onLongPressMoveUpdate = (details) {
                    host.held(details.globalPosition, start: false);
                  }
                  ..onLongPressEnd = (details) {
                    host.released(details.globalPosition);
                  };
              },
            ),
      },
      child: drawn,
    );
  }
}

/// A row's text, with the selection under it and the caret over it, and
/// where the handles hang.
class _LiveText extends StatelessWidget {
  const _LiveText({
    required this.state,
    required this.display,
    required this.textKey,
  });

  final LiveBlockRowState state;
  final LiveDisplay display;
  final GlobalKey textKey;

  @override
  Widget build(BuildContext context) {
    final host = state.widget.host;
    return _Leaders(
      host: host,
      state: state,
      child: CustomPaint(
        painter: _SelectionPainter(state: state, display: display),
        foregroundPainter: _CaretPainter(state: state, display: display),
        child: Text.rich(
          display.span,
          key: textKey,
          style: display.isEmpty ? display.base : null,
        ),
      ),
    );
  }
}

/// The selection under a row's text.
class _SelectionPainter extends CustomPainter {
  _SelectionPainter({required this.state, required this.display})
    : super(repaint: state.widget.host.repaint);

  final LiveBlockRowState state;
  final LiveDisplay display;

  @override
  void paint(Canvas canvas, Size size) {
    final host = state.widget.host;
    final selection = host.selection;
    final row = state.widget.row;
    if (!selection.isValid || selection.isCollapsed) {
      return;
    }
    final start = selection.start < row.range.start
        ? row.range.start
        : selection.start;
    final end = selection.end > row.range.end ? row.range.end : selection.end;
    final text = state.paragraph;
    if (start > end || text == null) {
      return;
    }
    final paint = Paint()..color = host.selectionColor;
    final from = display.toDisplay(start);
    final to = display.toDisplay(end);
    if (from < to) {
      for (final box in text.getBoxesForSelection(
        TextSelection(baseOffset: from, extentOffset: to),
      )) {
        canvas.drawRect(box.toRect(), paint);
      }
    }
    // A line break selected: a sliver past the end of the row's text.
    if (selection.end > row.range.end && selection.start <= row.range.end) {
      final at = text.getOffsetForCaret(
        TextPosition(offset: display.length),
        Rect.zero,
      );
      final height = text.getFullHeightForCaret(
        TextPosition(offset: display.length),
      );
      canvas.drawRect(Rect.fromLTWH(at.dx, at.dy, 6, height), paint);
    }
  }

  @override
  bool shouldRepaint(_SelectionPainter oldDelegate) =>
      oldDelegate.display != display || oldDelegate.state != state;
}

/// The caret over a row's text, and other people's.
class _CaretPainter extends CustomPainter {
  _CaretPainter({required this.state, required this.display})
    : super(repaint: state.widget.host.repaint);

  final LiveBlockRowState state;
  final LiveDisplay display;

  /// How wide a caret is.
  static const double width = 2;

  void _caret(Canvas canvas, int offset, Color color, RenderParagraph text) {
    final position = TextPosition(offset: display.toDisplay(offset));
    final at = text.getOffsetForCaret(position, Rect.zero);
    final line = text.getFullHeightForCaret(position);
    final size = display.base.fontSize ?? 18;
    final height = (size * 1.25).roundToDouble().clamp(0.0, line);
    final top = at.dy + (line - height) / 2;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(at.dx, top, width, height),
        const Radius.circular(1),
      ),
      Paint()..color = color,
    );
  }

  @override
  void paint(Canvas canvas, Size size) {
    final host = state.widget.host;
    final row = state.widget.row;
    final text = state.paragraph;
    if (text == null) {
      return;
    }
    for (final (offset, color) in host.remoteCaretsIn(row)) {
      _caret(canvas, offset, color, text);
    }
    if (!host.caretShown || host.caretRow != state.widget.index) {
      return;
    }
    _caret(canvas, host.selection.extentOffset, host.caretColor, text);
  }

  @override
  bool shouldRepaint(_CaretPainter oldDelegate) =>
      oldDelegate.display != display || oldDelegate.state != state;
}

/// The bars of a row's quotes, the ones the quote goes on through running
/// up through the room above the row.
class _QuoteBars extends CustomPainter {
  _QuoteBars({
    required this.depth,
    required this.barsAbove,
    required this.above,
    required this.left,
    required this.color,
  });

  final int depth;
  final int barsAbove;
  final double above;
  final double left;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = color;
    for (var at = 0; at < depth; at++) {
      // A bar that goes on from the row above starts a pixel into it, so
      // the two are one bar, not two that meet on a fraction of a pixel.
      final top = at < barsAbove ? -1.0 : above;
      canvas.drawRect(
        Rect.fromLTWH(
          left + at * PreviewStyle.quoteStep,
          top,
          PreviewStyle.quoteBarWidth,
          size.height - top,
        ),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(_QuoteBars oldDelegate) =>
      oldDelegate.depth != depth ||
      oldDelegate.barsAbove != barsAbove ||
      oldDelegate.above != above ||
      oldDelegate.left != left ||
      oldDelegate.color != color;
}

/// Hangs the selection's handles on the row that has an end of it, at the
/// foot of that end's line, as the field does.
class _Leaders extends SingleChildRenderObjectWidget {
  const _Leaders({required this.host, required this.state, super.child});

  final LiveRowHost host;
  final LiveBlockRowState state;

  @override
  RenderObject createRenderObject(BuildContext context) =>
      _RenderLeaders(host: host, state: state);

  @override
  void updateRenderObject(BuildContext context, _RenderLeaders renderObject) {
    renderObject
      ..host = host
      ..state = state;
  }
}

class _RenderLeaders extends RenderProxyBox {
  _RenderLeaders({required LiveRowHost host, required this.state})
    // The setter does more than set it once attached.
    // ignore: prefer_initializing_formals
    : _host = host;

  LiveRowHost _host;
  LiveBlockRowState state;

  set host(LiveRowHost value) {
    if (identical(value, _host)) {
      return;
    }
    if (attached) {
      _host.repaint.removeListener(markNeedsPaint);
      value.repaint.addListener(markNeedsPaint);
    }
    _host = value;
  }

  @override
  void attach(PipelineOwner owner) {
    super.attach(owner);
    _host.repaint.addListener(markNeedsPaint);
  }

  @override
  void detach() {
    _host.repaint.removeListener(markNeedsPaint);
    super.detach();
  }

  @override
  bool get alwaysNeedsCompositing => true;

  /// Where an end of the selection at [offset] hangs its handle, if it is
  /// in this row: the foot of its line.
  Offset? _point(int offset) {
    final row = state.widget.row;
    if (!row.holds(offset)) {
      return null;
    }
    final rect = state.caretRect(offset);
    final own = state.box;
    if (rect == null || own == null) {
      return null;
    }
    // [rect] is in the row's box; this box is the text's.
    final shift = localToGlobal(Offset.zero, ancestor: own);
    return Offset(rect.left - shift.dx, rect.bottom - shift.dy);
  }

  @override
  void paint(PaintingContext context, Offset offset) {
    super.paint(context, offset);
    final host = _host;
    final selection = host.selection;
    if (!host.handlesShown || !selection.isValid) {
      return;
    }
    final index = state.widget.index;
    final start = host.startRow == index ? _point(selection.start) : null;
    final end = host.endRow == index ? _point(selection.end) : null;
    // Empty layers that only say where the handles hang.
    if (start != null) {
      context.pushLayer(
        LeaderLayer(link: host.startLink, offset: offset + start),
        (_, _) {},
        Offset.zero,
      );
    }
    if (end != null) {
      context.pushLayer(
        LeaderLayer(link: host.endLink, offset: offset + end),
        (_, _) {},
        Offset.zero,
      );
    }
  }
}
