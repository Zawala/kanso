// Kanso — preview_code.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/scheduler.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_icon.dart';
import 'preview_style.dart';

/// A code block: its runs in the theme's code colours, on the surface, with
/// its language and a copy button (`kanso-copy:<copy>`). Long lines scroll
/// sideways.
class PreviewCode extends StatelessWidget {
  const PreviewCode({
    required this.runs,
    required this.copy,
    this.language,
    super.key,
  });

  final String? language;
  final List<PreviewCodeRun> runs;
  final int copy;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return PreviewCodeBox(
      language: language,
      copy: copy,
      child: _codeText(style, runs),
    );
  }
}

/// [runs] as one text in the code style, its lines unwrapped.
Widget _codeText(PreviewStyle style, List<PreviewCodeRun> runs) => Text.rich(
  TextSpan(
    style: style.code,
    children: <InlineSpan>[
      for (final run in runs)
        TextSpan(
          text: run.text,
          style: TextStyle(
            color: run.color == null ? null : previewColor(run.color!),
            fontWeight: run.bold ? FontWeight.w700 : null,
            fontStyle: run.italic ? FontStyle.italic : null,
          ),
        ),
    ],
  ),
  softWrap: false,
);

/// A code block's runs cut into lines, so that a long block can be drawn a
/// piece at a time: pieces of [chunkLines] lines, the last shorter.
class PreviewCodeLines {
  PreviewCodeLines._(this._lines);

  /// The lines of [runs], cut once for each list of runs.
  factory PreviewCodeLines.of(List<PreviewCodeRun> runs) =>
      _cut[runs] ??= PreviewCodeLines._(_split(runs));

  static final Expando<PreviewCodeLines> _cut = Expando<PreviewCodeLines>();
  static final Expando<int> _counted = Expando<int>();

  /// A block of more lines than this is drawn in pieces.
  static const int longAt = 200;

  /// The lines in each piece of a long block.
  static const int chunkLines = 50;

  /// How many pieces a block of [runs] is drawn in: one for a block of up
  /// to [longAt] lines.
  static int chunksOf(List<PreviewCodeRun> runs) {
    final lines = _counted[runs] ??= () {
      var lines = 1;
      for (final run in runs) {
        final text = run.text;
        for (
          var at = text.indexOf('\n');
          at >= 0;
          at = text.indexOf('\n', at + 1)
        ) {
          lines++;
        }
      }
      return lines;
    }();
    return lines <= longAt ? 1 : (lines + chunkLines - 1) ~/ chunkLines;
  }

  static List<List<PreviewCodeRun>> _split(List<PreviewCodeRun> runs) {
    final lines = <List<PreviewCodeRun>>[<PreviewCodeRun>[]];
    for (final run in runs) {
      final parts = run.text.split('\n');
      for (var at = 0; at < parts.length; at++) {
        if (at > 0) {
          lines.add(<PreviewCodeRun>[]);
        }
        if (parts[at].isNotEmpty) {
          lines.last.add(
            PreviewCodeRun(
              text: parts[at],
              color: run.color,
              bold: run.bold,
              italic: run.italic,
            ),
          );
        }
      }
    }
    return lines;
  }

  final List<List<PreviewCodeRun>> _lines;

  /// The runs of the [chunk]th piece, its lines apart, with no break after
  /// the last.
  List<PreviewCodeRun> chunk(int chunk) {
    final from = chunk * chunkLines;
    final to = math.min(from + chunkLines, _lines.length);
    return <PreviewCodeRun>[
      for (var line = from; line < to; line++) ...<PreviewCodeRun>[
        if (line > from)
          const PreviewCodeRun(text: '\n', bold: false, italic: false),
        ..._lines[line],
      ],
    ];
  }
}

/// The sideways scroll the pieces of a long code block share: a piece
/// scrolled takes the others with it, and each is as wide as the widest
/// line seen in any, so they all scroll as far.
class PreviewCodeScroll extends ChangeNotifier {
  final Set<ScrollController> _controllers = <ScrollController>{};
  double _offset = 0;
  bool _following = false;

  /// The widest piece laid out so far, in points.
  double get widest => _widest;
  double _widest = 0;
  double? _wider;

  /// A controller for a piece, at the place the others are.
  ScrollController attach() {
    late final ScrollController controller;
    controller = ScrollController(initialScrollOffset: _offset)
      ..addListener(() => _moved(controller));
    _controllers.add(controller);
    return controller;
  }

  void detach(ScrollController controller) {
    _controllers.remove(controller);
    controller.dispose();
  }

  void _moved(ScrollController from) {
    if (_following || !from.hasClients) {
      return;
    }
    _offset = from.offset;
    _following = true;
    try {
      for (final controller in _controllers) {
        if (controller != from &&
            controller.hasClients &&
            controller.offset != _offset) {
          controller.jumpTo(_offset);
        }
      }
    } finally {
      _following = false;
    }
  }

  /// Makes every piece at least [width] wide, after this frame: a piece
  /// says so while it is laid out, when the others can't be.
  void _widen(double width) {
    final wider = _wider;
    if (wider != null) {
      _wider = math.max(wider, width);
      return;
    }
    _wider = width;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      final width = _wider!;
      _wider = null;
      if (width > _widest) {
        _widest = width;
        notifyListeners();
      }
    });
  }
}

/// A piece of a long code block, drawn so the pieces look like one block:
/// the [first] has its top edge, its language and the copy button for the
/// whole block; the [last] its bottom edge; between, only the sides. Its
/// lines scroll sideways with the other pieces', through [scroll].
class PreviewCodeChunk extends StatefulWidget {
  const PreviewCodeChunk({
    required this.runs,
    required this.copy,
    required this.first,
    required this.last,
    required this.scroll,
    this.language,
    super.key,
  });

  final String? language;
  final List<PreviewCodeRun> runs;
  final int copy;
  final bool first;
  final bool last;
  final PreviewCodeScroll scroll;

  @override
  State<PreviewCodeChunk> createState() => _PreviewCodeChunkState();
}

class _PreviewCodeChunkState extends State<PreviewCodeChunk> {
  late ScrollController _controller = widget.scroll.attach();

  /// The text of a piece before the last is followed by a line break when
  /// it is selected to its end, as it is in the block.
  final _LineBreakAfter _selection = _LineBreakAfter();

  @override
  void didUpdateWidget(PreviewCodeChunk oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.scroll != oldWidget.scroll) {
      oldWidget.scroll.detach(_controller);
      _controller = widget.scroll.attach();
    }
  }

  @override
  void dispose() {
    widget.scroll.detach(_controller);
    _selection.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final text = _codeText(style, widget.runs);
    return PreviewCodeBox(
      language: widget.first ? widget.language : null,
      copy: widget.first ? widget.copy : null,
      top: widget.first,
      bottom: widget.last,
      controller: _controller,
      scroll: widget.scroll,
      child: widget.last
          ? text
          : SelectionContainer(delegate: _selection, child: text),
    );
  }
}

/// The selection in a piece of a long code block, followed by the line
/// break between it and the next piece when it runs to the piece's end.
class _LineBreakAfter extends StaticSelectionContainerDelegate {
  @override
  SelectedContent? getSelectedContent() {
    final content = super.getSelectedContent();
    final range = getSelection();
    if (content == null || range == null) {
      return content;
    }
    final end = math.max(range.startOffset, range.endOffset);
    return end < contentLength
        ? content
        : SelectedContent(plainText: '${content.plainText}\n');
  }
}

/// Lays its child out at least as wide as the widest piece of [scroll],
/// and tells [scroll] how wide it is.
class _AtLeastWidest extends SingleChildRenderObjectWidget {
  const _AtLeastWidest({required this.scroll, required super.child});

  final PreviewCodeScroll scroll;

  @override
  RenderObject createRenderObject(BuildContext context) =>
      _RenderAtLeastWidest(scroll);

  @override
  void updateRenderObject(
    BuildContext context,
    _RenderAtLeastWidest renderObject,
  ) => renderObject.scroll = scroll;
}

class _RenderAtLeastWidest extends RenderProxyBox {
  _RenderAtLeastWidest(this._scroll);

  PreviewCodeScroll _scroll;
  PreviewCodeScroll get scroll => _scroll;
  set scroll(PreviewCodeScroll value) {
    if (value == _scroll) {
      return;
    }
    if (attached) {
      _scroll.removeListener(markNeedsLayout);
      value.addListener(markNeedsLayout);
    }
    _scroll = value;
    markNeedsLayout();
  }

  @override
  void attach(PipelineOwner owner) {
    super.attach(owner);
    _scroll.addListener(markNeedsLayout);
  }

  @override
  void detach() {
    _scroll.removeListener(markNeedsLayout);
    super.detach();
  }

  @override
  void performLayout() {
    final child = this.child!..layout(constraints, parentUsesSize: true);
    final width = child.size.width;
    if (width > _scroll.widest) {
      _scroll._widen(width);
    }
    size = constraints.constrain(
      Size(math.max(width, _scroll.widest), child.size.height),
    );
  }
}

/// A source that couldn't be drawn: its code, with a badge saying why (the
/// details on a long press) and, for a fence, a copy button.
class PreviewFailed extends StatelessWidget {
  const PreviewFailed({
    required this.badge,
    required this.detail,
    required this.code,
    this.language,
    this.copy,
    super.key,
  });

  final String badge;
  final String detail;
  final String? language;
  final String code;
  final int? copy;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Stack(
      fit: StackFit.passthrough,
      children: <Widget>[
        PreviewCodeBox(
          language: language,
          copy: copy,
          child: Text(code, style: style.code, softWrap: false),
        ),
        Positioned(
          top: 7,
          left: 10,
          child: PreviewBadge(badge, detail: detail),
        ),
      ],
    );
  }
}

/// A query that can't be read: its text, with a badge, and why.
class PreviewQueryInvalid extends StatelessWidget {
  const PreviewQueryInvalid({
    required this.code,
    required this.message,
    super.key,
  });

  final String code;
  final String message;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: <Widget>[
        Stack(
          fit: StackFit.passthrough,
          children: <Widget>[
            PreviewCodeBox(
              child: Text(code, style: style.code, softWrap: false),
            ),
            Positioned(
              top: 7,
              left: 10,
              child: PreviewBadge(context.l10n.previewQueryBadge),
            ),
          ],
        ),
        SizedBox(height: 0.4 * style.size),
        Text(
          message,
          style: style.sized(0.8).copyWith(color: style.muted),
        ),
      ],
    );
  }
}

/// The page's block of code: the surface with a quiet edge and rounded
/// corners, room above for the [language] label and the copy button.
///
/// A piece of a long block has only the edges it shares with the block:
/// the [top] one, the [bottom] one, or neither; and it scrolls sideways with
/// [controller], at least as wide as [scroll] says.
class PreviewCodeBox extends StatelessWidget {
  const PreviewCodeBox({
    required this.child,
    this.language,
    this.copy,
    this.top = true,
    this.bottom = true,
    this.controller,
    this.scroll,
    super.key,
  });

  final Widget child;
  final String? language;

  /// The code's `kanso-copy:` number; no copy button when null.
  final int? copy;

  final bool top;
  final bool bottom;
  final ScrollController? controller;
  final PreviewCodeScroll? scroll;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final copy = this.copy;
    final language = this.language;
    final edge = BorderSide(color: style.quiet);
    const corner = Radius.circular(8);
    final scroll = this.scroll;
    return Container(
      decoration: BoxDecoration(
        color: style.surface,
        border: Border(
          top: top ? edge : BorderSide.none,
          bottom: bottom ? edge : BorderSide.none,
          left: edge,
          right: edge,
        ),
        borderRadius: !top && !bottom
            ? null
            : BorderRadius.vertical(
                top: top ? corner : Radius.zero,
                bottom: bottom ? corner : Radius.zero,
              ),
      ),
      child: Stack(
        children: <Widget>[
          Padding(
            padding: EdgeInsets.fromLTRB(0, top ? 34 : 0, 0, bottom ? 14 : 0),
            child: SingleChildScrollView(
              controller: controller,
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: scroll == null
                  ? child
                  : _AtLeastWidest(scroll: scroll, child: child),
            ),
          ),
          if (language != null && language.isNotEmpty)
            Positioned(
              top: 7,
              right: copy == null ? 12 : 44,
              child: Text(language, style: style.label),
            ),
          if (copy != null)
            Positioned(
              top: 0,
              right: 0,
              child: _CopyButton(onTap: () => scope.onLink('kanso-copy:$copy')),
            ),
        ],
      ),
    );
  }
}

/// A small label over a block: why a source wasn't drawn, or "Query".
class PreviewBadge extends StatelessWidget {
  const PreviewBadge(this.text, {this.detail, super.key});

  final String text;

  /// More about it, shown on a long press.
  final String? detail;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final badge = Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 1),
      decoration: BoxDecoration(
        color: style.tableHead,
        border: Border.all(color: style.border),
        borderRadius: BorderRadius.circular(5),
      ),
      child: Text(text, style: style.sized(0.72).copyWith(color: style.muted)),
    );
    final detail = this.detail;
    return detail == null || detail.isEmpty
        ? badge
        : Tooltip(message: detail, child: badge);
  }
}

class _CopyButton extends StatelessWidget {
  const _CopyButton({required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Semantics(
      container: true,
      button: true,
      label: context.l10n.previewCopyCode,
      onTap: onTap,
      excludeSemantics: true,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: SizedBox.square(
          dimension: 44,
          child: Center(
            child: KansoIcon(KansoIcons.copy, size: 16, color: style.muted),
          ),
        ),
      ),
    );
  }
}
