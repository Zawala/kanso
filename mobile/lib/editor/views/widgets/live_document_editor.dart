// Kanso — live_document_editor.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../note/views/widgets/preview/preview_style.dart';
import '../../utils/live_document/live_display.dart';
import '../../utils/live_document/live_layout_data.dart';
import '../../utils/live_document/live_rows.dart';
import '../controllers/kanso_markdown_controller.dart';
import '../controllers/live_layout_controller.dart';
import 'kanso_editor_selection_controls.dart';
import 'kanso_magnifier.dart';
import 'kanso_undo_manager_client.dart';
import 'live_block_row.dart';
import 'live_text_menu.dart';

/// The note's text in Live mode: one document, drawn as a column of the
/// preview's blocks.
///
/// The text is the [KansoMarkdownController]'s (the whole note, never a
/// window: [KansoMarkdownController.unwindowed]); every change goes through
/// it, so the Rust core's undo, saving, list continuation and formats are
/// the same as in Edit mode. The core's Live layout ([LiveLayoutController])
/// says what each block is and which marks to hide; each block is a row
/// drawn with the preview's metrics, palette and fonts, a block the
/// preview's renderer draws (code, a table, a formula, a diagram, an image)
/// as the preview's own widget until the caret goes into it, when its
/// source shows.
///
/// The caret and the selection are the note's, in units of its text, drawn
/// in each row they reach. The keyboard edits the block the caret is in:
/// the platform is given that block's lines (and the line before, for
/// context and for Backspace at its start), so composing, autocorrect and
/// dictation work as in any field, and what it changes is applied to the
/// note. It is a sliver: the screen's scroll view lays out only the rows
/// on screen.
class LiveDocumentEditor extends StatefulWidget {
  const LiveDocumentEditor({
    required this.controller,
    required this.layout,
    required this.focusNode,
    required this.palette,
    this.readOnly = false,
    this.autofocus = false,
    this.scrollPadding = const EdgeInsets.all(24),
    this.menuExtras,
    this.linkAt,
    this.onOpenLink,
    this.onComment,
    super.key,
  });

  final KansoMarkdownController controller;
  final LiveLayoutController layout;
  final FocusNode focusNode;

  /// The preview theme's colours, fitted with the interface and syntax
  /// themes in force: what the blocks are drawn in.
  final PaletteDto palette;

  /// Whether the text may be selected and copied, and not changed.
  final bool readOnly;
  final bool autofocus;

  /// How much room is kept around the caret when it is scrolled into view.
  final EdgeInsets scrollPadding;

  /// The screen's buttons in the text menu, first.
  final List<(String, VoidCallback)> Function()? menuExtras;

  /// Where the link at an offset of the note leads.
  final Future<LinkTarget?> Function(int offset)? linkAt;

  /// The person chose Open link in the menu.
  final ValueChanged<LinkTarget>? onOpenLink;

  /// The person chose Comment, with text selected; null where they may not.
  final VoidCallback? onComment;

  @override
  State<LiveDocumentEditor> createState() => LiveDocumentEditorState();
}

class LiveDocumentEditorState extends State<LiveDocumentEditor>
    with DeltaTextInputClient, WidgetsBindingObserver
    implements LiveRowHost, TextSelectionDelegate {
  final Map<int, LiveBlockRowState> _registered = <int, LiveBlockRowState>{};
  final ValueNotifier<bool> _blink = ValueNotifier<bool>(true);
  late Listenable _repaint;
  final ContextMenuController _menu = ContextMenuController();
  final KansoEditorSelectionControls _controls = KansoEditorSelectionControls();
  late KansoUndoManagerClient _undo;

  @override
  final LayerLink startLink = LayerLink();
  @override
  final LayerLink endLink = LayerLink();

  Timer? _blinkTimer;
  TextInputConnection? _input;
  TextEditingValue? _sent;
  int _imeStart = 0;
  SelectionOverlay? _overlay;
  bool _handles = false;

  // The rows, and what they were laid out from.
  List<LiveRow> _rows = const <LiveRow>[];
  LiveLayoutData? _rowsLayout;
  List<LivePreviewBlock>? _rowsPreviews;
  PreviewStyle? _rowsStyle;
  PreviewStyle? _style;

  // Where vertical moves aim, on the screen.
  double? _goalX;

  // A drag of a handle, a held press or a mouse: the end of the selection
  // that stays, and how far the finger is from the line it moves.
  int? _anchor;
  Offset _dragShift = Offset.zero;
  bool _showCaretDue = false;
  bool _floating = false;
  Offset _floatingFrom = Offset.zero;

  KansoMarkdownController get controller => widget.controller;

  /// The rows the note is drawn as now.
  @visibleForTesting
  List<LiveRow> get rows => _rows;

  /// The rows laid out now, by index.
  @visibleForTesting
  Map<int, LiveBlockRowState> get rowStates => _registered;

  @override
  void initState() {
    super.initState();
    _repaint = Listenable.merge(<Listenable>[controller, _blink]);
    controller.addListener(_changed);
    widget.layout.addListener(_laidOut);
    widget.focusNode.addListener(_focusChanged);
    WidgetsBinding.instance.addObserver(this);
    _undo = KansoUndoManagerClient(
      controller: controller,
      focusNode: widget.focusNode,
    )..attach();
    if (widget.autofocus) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          widget.focusNode.requestFocus();
        }
      });
    }
    if (widget.focusNode.hasFocus) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _focusChanged());
    }
  }

  @override
  void didUpdateWidget(LiveDocumentEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != controller) {
      oldWidget.controller.removeListener(_changed);
      controller.addListener(_changed);
      _repaint = Listenable.merge(<Listenable>[controller, _blink]);
    }
    if (oldWidget.layout != widget.layout) {
      oldWidget.layout.removeListener(_laidOut);
      widget.layout.addListener(_laidOut);
    }
    if (oldWidget.focusNode != widget.focusNode) {
      oldWidget.focusNode.removeListener(_focusChanged);
      widget.focusNode.addListener(_focusChanged);
    }
    if (oldWidget.controller != controller ||
        oldWidget.focusNode != widget.focusNode) {
      _undo.detach();
      _undo = KansoUndoManagerClient(
        controller: controller,
        focusNode: widget.focusNode,
      )..attach();
    }
    if (widget.readOnly && !oldWidget.readOnly) {
      _closeInput();
    } else if (!widget.readOnly &&
        oldWidget.readOnly &&
        widget.focusNode.hasFocus) {
      _openInput();
    }
  }

  @override
  void dispose() {
    controller.removeListener(_changed);
    widget.layout.removeListener(_laidOut);
    widget.focusNode.removeListener(_focusChanged);
    WidgetsBinding.instance.removeObserver(this);
    _undo.detach();
    _blinkTimer?.cancel();
    _closeInput();
    _menu.remove();
    _overlay?.dispose();
    _overlay = null;
    _blink.dispose();
    super.dispose();
  }

  // MARK: Changes

  TextEditingValue? _seen;

  void _changed() {
    if (!mounted) {
      return;
    }
    final now = controller.value;
    final before = _seen;
    _seen = now;
    final textChanged = before == null || before.text != now.text;
    final moved = before == null || before.selection != now.selection;
    if (textChanged || moved) {
      if (moved) {
        _goalX = _keepGoal ? _goalX : null;
      }
      _restartBlink();
      if (widget.focusNode.hasFocus) {
        _sendValue();
        _scheduleShowCaret();
      }
      if (textChanged && _menu.isShown) {
        _menu.remove();
      }
      if (now.selection.isCollapsed && _handles && _anchor == null) {
        _hideHandles();
      }
    }
    setState(() {});
  }

  void _laidOut() {
    if (mounted) {
      setState(() {});
      _updateOverlay();
    }
  }

  bool _keepGoal = false;

  void _focusChanged() {
    if (!mounted) {
      return;
    }
    if (widget.focusNode.hasFocus) {
      if (!controller.selection.isValid) {
        controller.selection = TextSelection.collapsed(
          offset: controller.text.length,
        );
      }
      _openInput();
      _restartBlink();
      _scheduleShowCaret();
    } else {
      _closeInput();
      _blinkTimer?.cancel();
      _menu.remove();
      _hideHandles();
    }
    setState(() {});
  }

  @override
  void didChangeMetrics() {
    // The keyboard came or went: the caret stays in view.
    if (widget.focusNode.hasFocus) {
      _scheduleShowCaret();
    }
  }

  // MARK: Rows

  List<LiveRow> _plan(PreviewStyle style) {
    final layout = widget.layout.layout;
    final previews = widget.layout.previews;
    if (!identical(layout, _rowsLayout) ||
        !identical(previews, _rowsPreviews) ||
        style != _rowsStyle) {
      _rows = planLiveRows(layout: layout, style: style, previews: previews);
      _rowsLayout = layout;
      _rowsPreviews = previews;
      _rowsStyle = style;
    }
    return _rows;
  }

  /// The row [offset] of the note is drawn in.
  int? _rowOf(int offset) {
    final rows = _rows;
    var low = 0;
    var high = rows.length - 1;
    var found = -1;
    while (low <= high) {
      final middle = (low + high) >> 1;
      if (rows[middle].range.start <= offset) {
        found = middle;
        low = middle + 1;
      } else {
        high = middle - 1;
      }
    }
    // The last row starting at or before it that holds it and is drawn.
    for (var at = found; at >= 0; at--) {
      final row = rows[at];
      if (row.kind == LiveRowKind.hidden) {
        continue;
      }
      if (row.holds(offset)) {
        return at;
      }
      if (row.range.end < offset) {
        break;
      }
    }
    return null;
  }

  // MARK: LiveRowHost

  @override
  TextSelection get selection => controller.selection;

  @override
  Listenable get repaint => _repaint;

  @override
  bool get caretShown =>
      widget.focusNode.hasFocus &&
      _blink.value &&
      selection.isValid &&
      selection.isCollapsed;

  @override
  int? get caretRow =>
      selection.isValid ? _rowOf(selection.extentOffset) : null;

  @override
  int? get startRow => selection.isValid ? _rowOf(selection.start) : null;

  @override
  int? get endRow => selection.isValid ? _rowOf(selection.end) : null;

  @override
  bool get handlesShown => _handles;

  @override
  Color get selectionColor => context.colors.accent.withValues(alpha: 0.3);

  @override
  Color get caretColor => context.colors.accent;

  @override
  List<(int, Color)> remoteCaretsIn(LiveRow row) => <(int, Color)>[
    for (final cursor in controller.remoteCursors)
      if (row.holds(cursor.caret))
        (cursor.caret, controller.cursorColor(cursor)),
  ];

  @override
  LiveOverlays overlaysFor(LiveRow row) {
    final value = controller.value;
    final composing = value.composing;
    return LiveOverlays(
      comments: controller.commentMarks,
      commentStyle: controller.commentStyle,
      selections: <(TextRange, Color)>[
        for (final cursor in controller.remoteCursors)
          if (cursor.end > cursor.start)
            (
              TextRange(start: cursor.start, end: cursor.end),
              controller.cursorColor(cursor).withValues(alpha: 0.22),
            ),
      ],
      composing: composing.isValid && !composing.isCollapsed
          ? composing
          : TextRange.empty,
    );
  }

  @override
  void register(int index, LiveBlockRowState row) => _registered[index] = row;

  @override
  void unregister(int index, LiveBlockRowState row) {
    if (identical(_registered[index], row)) {
      _registered.remove(index);
    }
  }

  // MARK: Where things are

  /// The row laid out under [global], a point on the screen, or the
  /// nearest.
  LiveBlockRowState? _rowAt(Offset global) {
    LiveBlockRowState? nearest;
    var distance = double.infinity;
    final indices = _registered.keys.toList()..sort();
    for (final index in indices) {
      final row = _registered[index]!;
      if (row.row.kind == LiveRowKind.hidden) {
        continue;
      }
      final box = row.box;
      if (box == null || box.size.height <= 0) {
        continue;
      }
      final local = box.globalToLocal(global);
      if (local.dy >= 0 && local.dy < box.size.height) {
        return row;
      }
      final away = local.dy < 0 ? -local.dy : local.dy - box.size.height;
      if (away < distance) {
        distance = away;
        nearest = row;
      }
    }
    return nearest;
  }

  /// The offset of the note nearest [global].
  int? _offsetAt(Offset global) => _rowAt(global)?.offsetAt(global);

  /// Where the caret at [offset] is drawn on the screen, if its row is
  /// laid out.
  Rect? _caretOnScreen(int offset) {
    final index = _rowOf(offset);
    final row = index == null ? null : _registered[index];
    final box = row?.box;
    final rect = row?.caretRect(offset);
    if (box == null || rect == null) {
      return null;
    }
    return rect.shift(box.localToGlobal(Offset.zero));
  }

  // MARK: Gestures

  @override
  void tapped(int index, Offset global, int count, PointerDeviceKind kind) {
    final row = _registered[index];
    if (row == null) {
      return;
    }
    final wasFocused = widget.focusNode.hasFocus;
    if (!wasFocused) {
      widget.focusNode.requestFocus();
    }
    final offset = row.offsetAt(global);
    final shift =
        kind == PointerDeviceKind.mouse &&
        HardwareKeyboard.instance.isShiftPressed;
    if (count >= 3) {
      _select(
        TextSelection(
          baseOffset: row.row.range.start,
          extentOffset: row.row.range.end,
        ),
      );
      _showHandles();
      _showMenu();
      return;
    }
    if (count == 2) {
      final word = _wordAt(offset);
      _select(TextSelection(baseOffset: word.start, extentOffset: word.end));
      if (kind != PointerDeviceKind.mouse) {
        _showHandles();
        _showMenu();
      }
      return;
    }
    if (shift && selection.isValid) {
      _select(
        TextSelection(baseOffset: selection.baseOffset, extentOffset: offset),
      );
      return;
    }
    final same =
        selection.isValid &&
        selection.isCollapsed &&
        selection.extentOffset == offset;
    _select(TextSelection.collapsed(offset: offset));
    _hideHandles();
    if (same && wasFocused && kind != PointerDeviceKind.mouse) {
      // A tap on the caret shows the menu, or hides it.
      if (_menu.isShown) {
        _menu.remove();
      } else {
        _showMenu();
      }
    } else {
      _menu.remove();
    }
  }

  @override
  void dragged(Offset global, {required bool start}) {
    final offset = _offsetAt(global);
    if (offset == null) {
      return;
    }
    if (start) {
      widget.focusNode.requestFocus();
      _anchor = offset;
      _select(TextSelection.collapsed(offset: offset));
      return;
    }
    final anchor = _anchor ?? selection.baseOffset;
    _select(TextSelection(baseOffset: anchor, extentOffset: offset));
  }

  @override
  void held(Offset global, {required bool start}) {
    final offset = _offsetAt(global);
    if (offset == null) {
      return;
    }
    if (start) {
      widget.focusNode.requestFocus();
      final word = _wordAt(offset);
      _anchor = word.start;
      _select(TextSelection(baseOffset: word.start, extentOffset: word.end));
      _showHandles();
      _magnify(global, offset);
      return;
    }
    final anchor = _anchor ?? selection.baseOffset;
    _select(TextSelection(baseOffset: anchor, extentOffset: offset));
    _magnify(global, offset);
  }

  @override
  void released(Offset global) {
    _anchor = null;
    _overlay?.hideMagnifier();
    if (!selection.isCollapsed) {
      _showHandles();
    }
    _showMenu();
  }

  void _magnify(Offset global, int offset) {
    final overlay = _overlay;
    final caret = _caretOnScreen(offset);
    if (overlay == null || caret == null) {
      return;
    }
    final info = MagnifierInfo(
      globalGesturePosition: global,
      caretRect: caret,
      fieldBounds: caret.inflate(400),
      currentLineBoundaries: Rect.fromLTRB(
        caret.left - 200,
        caret.top,
        caret.left + 200,
        caret.bottom,
      ),
    );
    if (overlay.magnifierIsVisible) {
      overlay.updateMagnifier(info);
    } else {
      overlay.showMagnifier(info);
    }
  }

  @override
  void toggleTask(LiveRowMarker marker) {
    if (widget.readOnly) {
      return;
    }
    final text = controller.text;
    final end = marker.range.end.clamp(0, text.length);
    final open = text.lastIndexOf('[', end);
    if (open < marker.range.start || open + 2 >= text.length) {
      return;
    }
    final box = TextRange(start: open + 1, end: open + 2);
    final ticked = text.substring(box.start, box.end) != ' ';
    final kept = selection;
    controller.run(
      EditCommand.insert(
        range: Range16(start: box.start, end: box.end),
        text: ticked ? ' ' : 'x',
      ),
    );
    // The caret stays where it was: a tick is not a move into the task.
    if (kept.isValid &&
        kept.end <= controller.text.length &&
        !controller.isComposing) {
      controller.selection = kept;
    }
  }

  @override
  void widgetLink(LiveRow row, String url) {
    if (url.startsWith('kanso-copy:')) {
      final source = controller.text.substring(row.range.start, row.range.end);
      final lines = source.split('\n');
      final fenced =
          lines.length >= 2 &&
          RegExp(r'^\s*(```|~~~)').hasMatch(lines.first) &&
          RegExp(r'^\s*(```|~~~)\s*$').hasMatch(lines.last);
      final code = fenced
          ? lines.sublist(1, lines.length - 1).join('\n')
          : source;
      unawaited(Clipboard.setData(ClipboardData(text: code)));
      return;
    }
    // Anything else in a drawn block: its source shows, the caret in it.
    widget.focusNode.requestFocus();
    _select(TextSelection.collapsed(offset: row.range.start));
  }

  // MARK: The selection

  void _select(TextSelection next, {bool keepGoal = false}) {
    final length = controller.text.length;
    final clamped = TextSelection(
      baseOffset: next.baseOffset.clamp(0, length),
      extentOffset: next.extentOffset.clamp(0, length),
    );
    _keepGoal = keepGoal;
    try {
      controller.selection = clamped;
    } finally {
      _keepGoal = false;
    }
    _updateOverlay();
  }

  /// The word at [offset], or the empty range there.
  TextRange _wordAt(int offset) {
    final text = controller.text;
    bool word(int at) =>
        at >= 0 &&
        at < text.length &&
        RegExp(r'[\p{L}\p{N}_]', unicode: true).hasMatch(text[at]);
    var start = offset;
    var end = offset;
    if (!word(offset) && word(offset - 1)) {
      start = offset - 1;
      end = offset - 1;
    }
    if (!word(start)) {
      return TextRange(
        start: offset,
        end: offset < text.length && text[offset] != '\n' ? offset + 1 : offset,
      );
    }
    while (word(start - 1)) {
      start--;
    }
    while (word(end)) {
      end++;
    }
    return TextRange(start: start, end: end);
  }

  // MARK: Handles, magnifier and menu

  void _ensureOverlay() {
    if (_overlay != null) {
      return;
    }
    _overlay = SelectionOverlay(
      context: context,
      debugRequiredFor: widget,
      startHandleType: TextSelectionHandleType.left,
      lineHeightAtStart: _lineHeightAt(selection.start),
      endHandleType: TextSelectionHandleType.right,
      lineHeightAtEnd: _lineHeightAt(selection.end),
      selectionEndpoints: const <TextSelectionPoint>[],
      selectionControls: _controls,
      // ignore: deprecated_member_use
      selectionDelegate: this,
      clipboardStatus: null,
      startHandleLayerLink: startLink,
      endHandleLayerLink: endLink,
      toolbarLayerLink: LayerLink(),
      magnifierConfiguration: KansoMagnifier.configuration(),
      onStartHandleDragStart: (details) => _handleStart(details, start: true),
      onStartHandleDragUpdate: _handleMoved,
      onStartHandleDragEnd: _handleEnd,
      onEndHandleDragStart: (details) => _handleStart(details, start: false),
      onEndHandleDragUpdate: _handleMoved,
      onEndHandleDragEnd: _handleEnd,
    );
  }

  double _lineHeightAt(int offset) {
    final caret = selection.isValid ? _caretOnScreen(offset) : null;
    return caret?.height ?? (_style?.size ?? 18) * 1.45;
  }

  void _showHandles() {
    if (!mounted) {
      return;
    }
    _ensureOverlay();
    _handles = true;
    _overlay!.showHandles();
    _updateOverlay();
    _blink.value = !_blink.value;
    _blink.value = !_blink.value;
  }

  void _hideHandles() {
    if (!_handles) {
      return;
    }
    _handles = false;
    _overlay?.hideHandles();
  }

  void _updateOverlay() {
    final overlay = _overlay;
    if (overlay == null || !selection.isValid) {
      return;
    }
    overlay
      ..startHandleType = selection.isCollapsed
          ? TextSelectionHandleType.collapsed
          : TextSelectionHandleType.left
      ..endHandleType = selection.isCollapsed
          ? TextSelectionHandleType.collapsed
          : TextSelectionHandleType.right
      ..lineHeightAtStart = _lineHeightAt(selection.start)
      ..lineHeightAtEnd = _lineHeightAt(selection.end);
  }

  void _handleStart(DragStartDetails details, {required bool start}) {
    _anchor = start ? selection.end : selection.start;
    final moving = start ? selection.start : selection.end;
    final caret = _caretOnScreen(moving);
    _dragShift = caret == null
        ? Offset.zero
        : caret.center - details.globalPosition;
    _menu.remove();
    _magnify(details.globalPosition + _dragShift, moving);
  }

  void _handleMoved(DragUpdateDetails details) {
    final at = details.globalPosition + _dragShift;
    final offset = _offsetAt(at);
    final anchor = _anchor;
    if (offset == null || anchor == null) {
      return;
    }
    _select(TextSelection(baseOffset: anchor, extentOffset: offset));
    _magnify(at, offset);
  }

  void _handleEnd(DragEndDetails details) {
    _anchor = null;
    _overlay?.hideMagnifier();
    _showMenu();
  }

  /// Where the menu over the selection is anchored, on the screen.
  TextSelectionToolbarAnchors? _anchors() {
    if (!selection.isValid) {
      return null;
    }
    final first = _caretOnScreen(selection.start);
    final last = _caretOnScreen(selection.end) ?? first;
    if (first == null || last == null) {
      return null;
    }
    final x = selection.isCollapsed || first.top == last.top
        ? (first.left + last.left) / 2
        : (first.left + last.left) / 2;
    return TextSelectionToolbarAnchors(
      primaryAnchor: Offset(x, first.top),
      secondaryAnchor: Offset(x, last.bottom),
    );
  }

  void _showMenu() {
    if (!mounted || !selection.isValid) {
      return;
    }
    final anchors = _anchors();
    if (anchors == null) {
      return;
    }
    final l10n = context.l10n;
    final editable = !widget.readOnly;
    final chosen = selection;
    final comment = widget.onComment;
    final buttons = <(String, VoidCallback)>[
      if (editable && !chosen.isCollapsed)
        (l10n.menuCut, () => cutSelection(SelectionChangedCause.toolbar)),
      if (!chosen.isCollapsed)
        (l10n.menuCopy, () => copySelection(SelectionChangedCause.toolbar)),
      if (editable)
        (
          l10n.menuPaste,
          () => unawaited(pasteText(SelectionChangedCause.toolbar)),
        ),
      if (controller.text.isNotEmpty &&
          !(chosen.start == 0 && chosen.end == controller.text.length))
        (l10n.menuSelectAll, () => selectAll(SelectionChangedCause.toolbar)),
      if (comment != null && !chosen.isCollapsed)
        (
          l10n.menuComment,
          () {
            _menu.remove();
            comment();
          },
        ),
    ];
    final extras =
        widget.menuExtras?.call() ?? const <(String, VoidCallback)>[];
    final linkAt = widget.linkAt;
    final open = widget.onOpenLink;
    _menu.show(
      context: context,
      contextMenuBuilder: (context) => TextFieldTapRegion(
        child: LiveTextMenu(
          anchors: anchors,
          buttons: buttons,
          extra: <(String, VoidCallback)>[
            for (final (label, action) in extras)
              (
                label,
                () {
                  _menu.remove();
                  action();
                },
              ),
          ],
          link: linkAt?.call(chosen.start),
          onOpen: open == null
              ? null
              : (target) {
                  _menu.remove();
                  open(target);
                },
        ),
      ),
    );
  }

  /// Where the caret is drawn on the screen now, if its row is laid out.
  Rect? caretOnScreen() =>
      selection.isValid ? _caretOnScreen(selection.extentOffset) : null;

  /// Whether the text menu is showing.
  @visibleForTesting
  bool get menuShown => _menu.isShown;

  // MARK: Edits

  /// Replaces [start] to [end] of the note with [text], the caret after it
  /// (or at [selection]), as one change the core is told of.
  void _replace(int start, int end, String text, {TextSelection? selection}) {
    if (widget.readOnly) {
      return;
    }
    final note = controller.text;
    final from = start.clamp(0, note.length);
    final to = end.clamp(from, note.length);
    final next = note.replaceRange(from, to, text);
    controller.value = TextEditingValue(
      text: next,
      selection:
          selection ?? TextSelection.collapsed(offset: from + text.length),
    );
  }

  // MARK: TextSelectionDelegate

  @override
  TextEditingValue get textEditingValue => controller.value;

  @override
  void userUpdateTextEditingValue(
    TextEditingValue value,
    SelectionChangedCause cause,
  ) {
    controller.value = value;
  }

  @override
  void hideToolbar([bool hideHandles = true]) {
    _menu.remove();
    if (hideHandles) {
      _hideHandles();
    }
  }

  @override
  void bringIntoView(TextPosition position) => _scheduleShowCaret();

  @override
  bool get cutEnabled => !widget.readOnly && !selection.isCollapsed;

  @override
  bool get copyEnabled => !selection.isCollapsed;

  @override
  bool get pasteEnabled => !widget.readOnly;

  @override
  bool get selectAllEnabled => controller.text.isNotEmpty;

  @override
  bool get lookUpEnabled => false;

  @override
  bool get searchWebEnabled => false;

  @override
  bool get shareEnabled => false;

  @override
  bool get liveTextInputEnabled => false;

  @override
  void copySelection(SelectionChangedCause cause) {
    if (!selection.isValid || selection.isCollapsed) {
      return;
    }
    unawaited(
      Clipboard.setData(
        ClipboardData(text: selection.textInside(controller.text)),
      ),
    );
    if (cause == SelectionChangedCause.toolbar) {
      hideToolbar();
    }
  }

  @override
  void cutSelection(SelectionChangedCause cause) {
    if (widget.readOnly || !selection.isValid || selection.isCollapsed) {
      return;
    }
    final chosen = selection;
    unawaited(
      Clipboard.setData(
        ClipboardData(text: chosen.textInside(controller.text)),
      ),
    );
    _replace(chosen.start, chosen.end, '');
    hideToolbar();
  }

  @override
  Future<void> pasteText(SelectionChangedCause cause) async {
    if (widget.readOnly) {
      return;
    }
    final chosen = selection;
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text;
    if (!mounted || text == null || text.isEmpty || !chosen.isValid) {
      return;
    }
    _replace(chosen.start, chosen.end, text);
    hideToolbar();
  }

  @override
  void selectAll(SelectionChangedCause cause) {
    _select(TextSelection(baseOffset: 0, extentOffset: controller.text.length));
    if (cause == SelectionChangedCause.toolbar) {
      _menu.remove();
      _showHandles();
      _showMenu();
    }
  }

  // MARK: Keyboard

  int _before(int offset) {
    final text = controller.text;
    if (offset <= 0) {
      return 0;
    }
    if (offset >= 2 &&
        _isLowSurrogate(text.codeUnitAt(offset - 1)) &&
        _isHighSurrogate(text.codeUnitAt(offset - 2))) {
      return offset - 2;
    }
    return offset - 1;
  }

  int _after(int offset) {
    final text = controller.text;
    if (offset >= text.length) {
      return text.length;
    }
    if (offset + 1 < text.length &&
        _isHighSurrogate(text.codeUnitAt(offset)) &&
        _isLowSurrogate(text.codeUnitAt(offset + 1))) {
      return offset + 2;
    }
    return offset + 1;
  }

  static bool _isHighSurrogate(int unit) => unit >= 0xD800 && unit <= 0xDBFF;
  static bool _isLowSurrogate(int unit) => unit >= 0xDC00 && unit <= 0xDFFF;

  int _wordBoundary(int offset, {required bool forward}) {
    final text = controller.text;
    bool word(int at) => RegExp(
      r'[\p{L}\p{N}_]',
      unicode: true,
    ).hasMatch(text[at]);
    var at = offset;
    if (forward) {
      while (at < text.length && !word(at)) {
        at++;
      }
      while (at < text.length && word(at)) {
        at++;
      }
    } else {
      while (at > 0 && !word(at - 1)) {
        at--;
      }
      while (at > 0 && word(at - 1)) {
        at--;
      }
    }
    return at;
  }

  int _lineBoundary(int offset, {required bool forward}) {
    final text = controller.text;
    if (forward) {
      final end = text.indexOf('\n', offset);
      return end < 0 ? text.length : end;
    }
    return offset <= 0 ? 0 : text.lastIndexOf('\n', offset - 1) + 1;
  }

  /// Moves the caret, or the selection's extent, to [target].
  void _moveTo(int target, {required bool collapse, bool keepGoal = false}) {
    if (!selection.isValid) {
      return;
    }
    _select(
      collapse
          ? TextSelection.collapsed(offset: target)
          : TextSelection(
              baseOffset: selection.baseOffset,
              extentOffset: target,
            ),
      keepGoal: keepGoal,
    );
    _menu.remove();
    if (collapse) {
      _hideHandles();
    }
  }

  /// Where the caret goes a line up or down from [offset], across rows,
  /// aiming at the same place across the screen.
  int? _vertical(int offset, {required bool down}) {
    final index = _rowOf(offset);
    if (index == null) {
      return null;
    }
    final row = _registered[index];
    final caret = _caretOnScreen(offset);
    if (row == null || caret == null) {
      return down
          ? _after(_lineBoundary(offset, forward: true))
          : _lineBoundary(
              _before(_lineBoundary(offset, forward: false)),
              forward: false,
            );
    }
    final x = _goalX ??= caret.left;
    final inside = row.verticalFrom(offset, x, down: down);
    if (inside != null) {
      return inside;
    }
    for (
      var at = down ? index + 1 : index - 1;
      at >= 0 && at < _rows.length;
      at += down ? 1 : -1
    ) {
      if (_rows[at].kind == LiveRowKind.hidden) {
        continue;
      }
      final next = _registered[at];
      if (next == null) {
        return down ? _rows[at].range.start : _rows[at].range.end;
      }
      return next.edgeOffset(x, last: !down);
    }
    return down ? controller.text.length : 0;
  }

  Object? _deleteCharacter(DeleteCharacterIntent intent) {
    if (widget.readOnly || !selection.isValid) {
      return null;
    }
    if (!selection.isCollapsed) {
      _replace(selection.start, selection.end, '');
      return null;
    }
    final at = selection.extentOffset;
    if (intent.forward) {
      _replace(at, _after(at), '');
    } else {
      final from = _before(at);
      _replace(from, at, '');
    }
    return null;
  }

  Object? _deleteTo(int Function(int offset) boundary) {
    if (widget.readOnly || !selection.isValid) {
      return null;
    }
    if (!selection.isCollapsed) {
      _replace(selection.start, selection.end, '');
      return null;
    }
    final at = selection.extentOffset;
    final to = boundary(at);
    _replace(math.min(at, to), math.max(at, to), '');
    return null;
  }

  Map<Type, Action<Intent>> get _actions => <Type, Action<Intent>>{
    DeleteCharacterIntent: CallbackAction<DeleteCharacterIntent>(
      onInvoke: _deleteCharacter,
    ),
    DeleteToNextWordBoundaryIntent:
        CallbackAction<DeleteToNextWordBoundaryIntent>(
          onInvoke: (intent) => _deleteTo(
            (at) => _wordBoundary(at, forward: intent.forward),
          ),
        ),
    DeleteToLineBreakIntent: CallbackAction<DeleteToLineBreakIntent>(
      onInvoke: (intent) =>
          _deleteTo((at) => _lineBoundary(at, forward: intent.forward)),
    ),
    ExtendSelectionByCharacterIntent:
        CallbackAction<ExtendSelectionByCharacterIntent>(
          onInvoke: (intent) {
            if (!selection.isValid) {
              return null;
            }
            if (intent.collapseSelection && !selection.isCollapsed) {
              _moveTo(
                intent.forward ? selection.end : selection.start,
                collapse: true,
              );
              return null;
            }
            final at = selection.extentOffset;
            _moveTo(
              intent.forward ? _after(at) : _before(at),
              collapse: intent.collapseSelection,
            );
            return null;
          },
        ),
    ExtendSelectionToNextWordBoundaryIntent:
        CallbackAction<ExtendSelectionToNextWordBoundaryIntent>(
          onInvoke: (intent) {
            if (selection.isValid) {
              _moveTo(
                _wordBoundary(
                  selection.extentOffset,
                  forward: intent.forward,
                ),
                collapse: intent.collapseSelection,
              );
            }
            return null;
          },
        ),
    ExtendSelectionToLineBreakIntent:
        CallbackAction<ExtendSelectionToLineBreakIntent>(
          onInvoke: (intent) {
            if (selection.isValid) {
              _moveTo(
                _lineBoundary(
                  selection.extentOffset,
                  forward: intent.forward,
                ),
                collapse: intent.collapseSelection,
              );
            }
            return null;
          },
        ),
    ExpandSelectionToLineBreakIntent:
        CallbackAction<ExpandSelectionToLineBreakIntent>(
          onInvoke: (intent) {
            if (selection.isValid) {
              _moveTo(
                _lineBoundary(
                  selection.extentOffset,
                  forward: intent.forward,
                ),
                collapse: false,
              );
            }
            return null;
          },
        ),
    ExtendSelectionToDocumentBoundaryIntent:
        CallbackAction<ExtendSelectionToDocumentBoundaryIntent>(
          onInvoke: (intent) {
            if (selection.isValid) {
              _moveTo(
                intent.forward ? controller.text.length : 0,
                collapse: intent.collapseSelection,
              );
            }
            return null;
          },
        ),
    ExpandSelectionToDocumentBoundaryIntent:
        CallbackAction<ExpandSelectionToDocumentBoundaryIntent>(
          onInvoke: (intent) {
            if (selection.isValid) {
              _moveTo(
                intent.forward ? controller.text.length : 0,
                collapse: false,
              );
            }
            return null;
          },
        ),
    ExtendSelectionVerticallyToAdjacentLineIntent:
        CallbackAction<ExtendSelectionVerticallyToAdjacentLineIntent>(
          onInvoke: (intent) {
            if (!selection.isValid) {
              return null;
            }
            final target = _vertical(
              selection.extentOffset,
              down: intent.forward,
            );
            if (target != null) {
              _moveTo(
                target,
                collapse: intent.collapseSelection,
                keepGoal: true,
              );
            }
            return null;
          },
        ),
    SelectAllTextIntent: CallbackAction<SelectAllTextIntent>(
      onInvoke: (intent) {
        selectAll(intent.cause);
        return null;
      },
    ),
    CopySelectionTextIntent: CallbackAction<CopySelectionTextIntent>(
      onInvoke: (intent) {
        if (intent.collapseSelection) {
          cutSelection(intent.cause);
        } else {
          copySelection(intent.cause);
        }
        return null;
      },
    ),
    PasteTextIntent: CallbackAction<PasteTextIntent>(
      onInvoke: (intent) {
        unawaited(pasteText(intent.cause));
        return null;
      },
    ),
    UndoTextIntent: CallbackAction<UndoTextIntent>(
      onInvoke: (_) => controller.undo(),
    ),
    RedoTextIntent: CallbackAction<RedoTextIntent>(
      onInvoke: (_) => controller.redo(),
    ),
    DismissIntent: CallbackAction<DismissIntent>(
      onInvoke: (_) {
        hideToolbar();
        return null;
      },
    ),
  };

  // MARK: The platform's input

  /// The block the platform is given: from the start of the line before
  /// the selection's first line to the end of its last line.
  (int, int) _window() {
    final text = controller.text;
    final chosen = selection.isValid
        ? selection
        : TextSelection.collapsed(offset: text.length);
    final first = _lineBoundary(chosen.start, forward: false);
    final start = first == 0 ? 0 : _lineBoundary(first - 1, forward: false);
    final end = _lineBoundary(chosen.end, forward: true);
    return (start, end);
  }

  TextEditingValue _windowValue() {
    final value = controller.value;
    final (start, end) = _window();
    _imeStart = start;
    TextSelection shift(TextSelection s) => TextSelection(
      baseOffset: (s.baseOffset - start).clamp(0, end - start),
      extentOffset: (s.extentOffset - start).clamp(0, end - start),
      affinity: s.affinity,
    );
    final composing = value.composing;
    return TextEditingValue(
      text: value.text.substring(start, end),
      selection: value.selection.isValid
          ? shift(value.selection)
          : TextSelection.collapsed(offset: end - start),
      composing:
          composing.isValid && composing.start >= start && composing.end <= end
          ? TextRange(
              start: composing.start - start,
              end: composing.end - start,
            )
          : TextRange.empty,
    );
  }

  void _openInput() {
    if (widget.readOnly || (_input?.attached ?? false)) {
      return;
    }
    final input = TextInput.attach(
      this,
      TextInputConfiguration(
        inputType: TextInputType.multiline,
        inputAction: TextInputAction.newline,
        enableDeltaModel: true,
        keyboardAppearance: Theme.of(context).brightness,
        textCapitalization: TextCapitalization.sentences,
      ),
    );
    _input = input;
    _sent = null;
    _sendValue();
    input.show();
    _sendGeometry();
  }

  void _closeInput() {
    _input?.close();
    _input = null;
    _sent = null;
  }

  void _sendValue() {
    final input = _input;
    if (input == null || !input.attached) {
      return;
    }
    final value = _windowValue();
    if (value == _sent) {
      return;
    }
    _sent = value;
    input.setEditingState(value);
    _sendGeometry();
  }

  bool _geometryDue = false;

  /// Tells the platform where the block and the caret are on the screen,
  /// for its candidates and its own caret.
  void _sendGeometry() {
    if (_geometryDue) {
      return;
    }
    _geometryDue = true;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _geometryDue = false;
      final input = _input;
      if (!mounted || input == null || !input.attached || !selection.isValid) {
        return;
      }
      final index = _rowOf(selection.extentOffset);
      final row = index == null ? null : _registered[index];
      final box = row?.box;
      final caret = row?.caretRect(selection.extentOffset);
      if (box == null || caret == null) {
        return;
      }
      input
        ..setEditableSizeAndTransform(box.size, box.getTransformTo(null))
        ..setCaretRect(Rect.fromLTWH(caret.left, caret.top, 2, caret.height))
        ..setComposingRect(caret);
    });
  }

  @override
  TextEditingValue? get currentTextEditingValue => _sent;

  @override
  AutofillScope? get currentAutofillScope => null;

  @override
  void updateEditingValue(TextEditingValue value) => _fromPlatform(value);

  @override
  void updateEditingValueWithDeltas(List<TextEditingDelta> textEditingDeltas) {
    var value = _sent ?? _windowValue();
    for (final delta in textEditingDeltas) {
      value = delta.apply(value);
    }
    _fromPlatform(value);
  }

  /// What the platform made of the block it was given, applied to the
  /// note.
  void _fromPlatform(TextEditingValue value) {
    final before = _sent;
    if (before == null || widget.readOnly) {
      return;
    }
    if (value == before) {
      return;
    }
    final origin = _imeStart;
    final note = controller.text;
    final old = before.text;
    final now = value.text;
    var start = 0;
    final shorter = math.min(old.length, now.length);
    while (start < shorter && old.codeUnitAt(start) == now.codeUnitAt(start)) {
      start++;
    }
    var oldEnd = old.length;
    var newEnd = now.length;
    while (oldEnd > start &&
        newEnd > start &&
        old.codeUnitAt(oldEnd - 1) == now.codeUnitAt(newEnd - 1)) {
      oldEnd--;
      newEnd--;
    }
    if (origin + oldEnd > note.length) {
      // The note changed under the platform: give it the block again.
      _sent = null;
      _sendValue();
      return;
    }
    final text = old == now
        ? note
        : note.replaceRange(
            origin + start,
            origin + oldEnd,
            now.substring(start, newEnd),
          );
    final length = text.length;
    final selected = value.selection.isValid
        ? TextSelection(
            baseOffset: (value.selection.baseOffset + origin).clamp(0, length),
            extentOffset: (value.selection.extentOffset + origin).clamp(
              0,
              length,
            ),
            affinity: value.selection.affinity,
          )
        : controller.selection;
    final composing = value.composing.isValid && !value.composing.isCollapsed
        ? TextRange(
            start: (value.composing.start + origin).clamp(0, length),
            end: (value.composing.end + origin).clamp(0, length),
          )
        : TextRange.empty;
    _sent = value;
    _menu.remove();
    controller.value = TextEditingValue(
      text: text,
      selection: selected,
      composing: composing,
    );
  }

  @override
  void performAction(TextInputAction action) {
    // A newline comes as text; nothing else applies to a note.
  }

  @override
  void performPrivateCommand(String action, Map<String, dynamic> data) {}

  @override
  void updateFloatingCursor(RawFloatingCursorPoint point) {
    switch (point.state) {
      case FloatingCursorDragState.Start:
        final caret = selection.isValid
            ? _caretOnScreen(selection.extentOffset)
            : null;
        _floating = caret != null;
        _floatingFrom = caret?.center ?? Offset.zero;
      case FloatingCursorDragState.Update:
        final moved = point.offset;
        if (!_floating || moved == null) {
          return;
        }
        final offset = _offsetAt(_floatingFrom + moved);
        if (offset != null) {
          _select(TextSelection.collapsed(offset: offset));
        }
      case FloatingCursorDragState.End:
        _floating = false;
    }
  }

  @override
  void showAutocorrectionPromptRect(int start, int end) {}

  @override
  void insertContent(KeyboardInsertedContent content) {}

  @override
  bool onFocusReceived() => false;

  @override
  void didChangeInputControl(
    TextInputControl? oldControl,
    TextInputControl? newControl,
  ) {}

  @override
  void showToolbar() => _showMenu();

  @override
  void insertTextPlaceholder(Size size) {}

  @override
  void removeTextPlaceholder() {}

  @override
  void performSelector(String selectorName) {
    final intent = intentForMacOSSelector(selectorName);
    if (intent != null) {
      final target = primaryFocus?.context;
      if (target != null) {
        Actions.maybeInvoke(target, intent);
      }
    }
  }

  @override
  void connectionClosed() {
    _input = null;
    _sent = null;
  }

  // MARK: The caret

  void _restartBlink() {
    _blinkTimer?.cancel();
    _blink.value = true;
    if (!widget.focusNode.hasFocus || EditableText.debugDeterministicCursor) {
      return;
    }
    _blinkTimer = Timer.periodic(
      const Duration(milliseconds: 500),
      (_) => _blink.value = !_blink.value,
    );
  }

  /// Scrolls the caret into view once the rows are laid out.
  void _scheduleShowCaret() {
    if (_showCaretDue) {
      return;
    }
    _showCaretDue = true;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _showCaretDue = false;
      if (mounted) {
        _showCaret();
      }
    });
  }

  int _seekTries = 0;

  /// Where to scroll to bring the row at [index], not laid out, near the
  /// screen: worked out from the rows laid out, the nearest of them and
  /// how tall they are on average; from the list's estimated length when
  /// none is.
  double? _seek(int index) {
    final scrollable = Scrollable.maybeOf(context);
    final position = scrollable?.position;
    final viewport = scrollable?.context.findRenderObject();
    if (position == null || viewport is! RenderBox || _rows.isEmpty) {
      return null;
    }
    final built = <(int, RenderBox)>[
      for (final entry in _registered.entries)
        if (entry.value.box case final RenderBox box) (entry.key, box),
    ]..sort((a, b) => a.$1.compareTo(b.$1));
    if (built.isEmpty) {
      final sliver = context.findRenderObject();
      final geometry = sliver is RenderSliver ? sliver.geometry : null;
      if (sliver is! RenderSliver || geometry == null) {
        return null;
      }
      return sliver.constraints.precedingScrollExtent +
          geometry.scrollExtent * index / _rows.length;
    }
    var tall = 0.0;
    for (final (_, box) in built) {
      tall += box.size.height;
    }
    final average = tall / built.length;
    final (near, box) = index < built.first.$1 ? built.first : built.last;
    final top = box.localToGlobal(Offset.zero, ancestor: viewport).dy;
    return position.pixels +
        top +
        (index - near) * average -
        position.viewportDimension / 3;
  }

  void _showCaret() {
    if (!selection.isValid) {
      return;
    }
    final offset = selection.extentOffset;
    final index = _rowOf(offset);
    if (index == null) {
      return;
    }
    final row = _registered[index];
    final box = row?.box;
    final caret = row?.caretRect(offset);
    if (row == null || box == null) {
      // Not laid out: scroll towards it, and look again.
      final target = _seek(index);
      if (target == null || _seekTries >= 8) {
        _seekTries = 0;
        return;
      }
      _seekTries++;
      final position = Scrollable.of(context).position;
      position.jumpTo(
        target.clamp(position.minScrollExtent, position.maxScrollExtent),
      );
      _scheduleShowCaret();
      return;
    }
    _seekTries = 0;
    final rect = caret ?? Rect.fromLTWH(0, 0, 1, box.size.height);
    final padding = widget.scrollPadding;
    box.showOnScreen(
      rect: Rect.fromLTRB(
        rect.left - padding.left,
        rect.top - padding.top,
        rect.right + padding.right,
        rect.bottom + padding.bottom,
      ),
      duration: const Duration(milliseconds: 100),
      curve: Curves.fastOutSlowIn,
    );
    _sendGeometry();
  }

  @override
  Widget build(BuildContext context) {
    final style = PreviewStyle(
      palette: widget.palette,
      typography: context.typography,
    );
    _style = style;
    final rows = _plan(style);
    final layout = widget.layout.layout;
    return Actions(
      actions: _actions,
      child: Focus(
        focusNode: widget.focusNode,
        includeSemantics: false,
        child: PreviewScope(
          style: style,
          onLink: (_) {},
          headingKey: (_) => null,
          child: SliverList(
            delegate: SliverChildBuilderDelegate(
              (context, index) => LiveBlockRow(
                key: ValueKey<int>(index),
                index: index,
                row: rows[index],
                layout: layout,
                style: style,
                host: this,
              ),
              childCount: rows.length,
              addAutomaticKeepAlives: false,
            ),
          ),
        ),
      ),
    );
  }
}
