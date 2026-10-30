// Kanso — live_layout_controller.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart' show Either;
import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../utils/live_document/live_layout_bridge.dart';
import '../../utils/live_document/live_layout_data.dart';
import '../../utils/live_document/live_rows.dart';
import 'kanso_markdown_controller.dart';

/// Draws the preview's blocks of a text of the note: what Live draws its
/// widgets and finds its lists' spacing in.
typedef LivePreviewSource =
    Future<Either<ApplicationError, List<LivePreviewBlock>>> Function(
      String text,
    );

/// What the Live editor draws, kept up with the note's text and caret.
///
/// The core works out Live's layout of the whole text with the selection
/// revealed (`EditorDocument.liveDocument`); it is asked for again whenever
/// the text or the selection changes, one request at a time, and a layout
/// that comes back for a text that has changed since is moved along the
/// change ([LiveLayoutData.shiftedTo]) until the next one comes. The
/// preview's blocks (what the widgets are drawn from) are drawn for the
/// whole text a moment after the typing stops.
class LiveLayoutController extends ChangeNotifier {
  LiveLayoutController({
    required this.controller,
    this.previewSource,
    this.previewDelay = const Duration(milliseconds: 350),
  }) : _layout = LiveLayoutData.plain(controller.text) {
    controller.addListener(_changed);
    _changed();
    _askPreview(now: true);
  }

  final KansoMarkdownController controller;

  /// Draws the preview's blocks; none leaves the widgets as their source.
  LivePreviewSource? previewSource;

  /// How long the typing must stop for before the preview is drawn again.
  final Duration previewDelay;

  LiveLayoutData _layout;
  LiveLayoutData? _shifted;
  List<LivePreviewBlock> _previews = const <LivePreviewBlock>[];
  String? _previewText;
  String? _askedText;
  TextSelection? _askedSelection;
  bool _inFlight = false;
  bool _due = false;
  bool _scheduled = false;
  bool _disposed = false;
  Timer? _previewTimer;
  bool _previewInFlight = false;

  /// The layout of the field's text now: the latest the core gave, moved
  /// along what was typed since.
  LiveLayoutData get layout {
    final text = controller.text;
    final shifted = _shifted;
    if (shifted != null && identical(shifted.text, text)) {
      return shifted;
    }
    return _shifted = _layout.shiftedTo(text);
  }

  /// The preview's top-level blocks, of [previewText].
  List<LivePreviewBlock> get previews => _previews;

  /// The text the preview's blocks were drawn for.
  String? get previewText => _previewText;

  /// Whether the layout on screen is the core's for the text and selection
  /// now.
  bool get settled =>
      !_inFlight &&
      !_due &&
      identical(_layout.text, controller.text) &&
      _askedSelection == controller.selection;

  void _changed() {
    if (_disposed) {
      return;
    }
    final text = controller.text;
    final selection = controller.selection;
    if (identical(text, _askedText) && selection == _askedSelection) {
      return;
    }
    if (!identical(text, _previewText) && text != _previewText) {
      _askPreview();
    }
    _due = true;
    if (_scheduled || _inFlight) {
      return;
    }
    _scheduled = true;
    scheduleMicrotask(() {
      _scheduled = false;
      unawaited(_fetch());
    });
  }

  /// Asks for the layout again, as for a change.
  void refresh() {
    _askedText = null;
    _changed();
  }

  Future<void> _fetch() async {
    if (_disposed || !_due) {
      return;
    }
    _due = false;
    _inFlight = true;
    final text = controller.text;
    final selection = controller.selection;
    final version = controller.version;
    _askedText = text;
    _askedSelection = selection;
    final reveal = <Range16>[
      if (selection.isValid)
        Range16(start: selection.baseOffset, end: selection.extentOffset),
    ];
    final result = await controller.document.liveDocument(reveal);
    _inFlight = false;
    if (_disposed) {
      return;
    }
    result.fold((_) {}, (document) {
      if (document.version != version) {
        // Worked out for another text: the latest is asked for next.
        _due = true;
        return;
      }
      _layout = liveLayoutOf(document, text);
      _shifted = null;
      notifyListeners();
    });
    if (!identical(controller.text, text) ||
        controller.selection != selection) {
      _due = true;
    }
    if (_due) {
      _scheduled = true;
      SchedulerBinding.instance.scheduleFrameCallback((_) {
        _scheduled = false;
        unawaited(_fetch());
      });
      SchedulerBinding.instance.ensureVisualUpdate();
    }
  }

  void _askPreview({bool now = false}) {
    if (previewSource == null || _disposed) {
      return;
    }
    _previewTimer?.cancel();
    if (now || previewDelay == Duration.zero) {
      scheduleMicrotask(() => unawaited(_drawPreview()));
      return;
    }
    _previewTimer = Timer(previewDelay, () => unawaited(_drawPreview()));
  }

  /// Draws the preview's blocks again now: the themes or the text size
  /// changed.
  void redrawPreview() {
    _previewText = null;
    _askPreview(now: true);
  }

  bool _previewDue = false;

  Future<void> _drawPreview() async {
    final source = previewSource;
    if (source == null || _disposed) {
      return;
    }
    if (_previewInFlight) {
      // Drawn again when this one is done.
      _previewDue = true;
      return;
    }
    _previewInFlight = true;
    _previewDue = false;
    final text = controller.text;
    final drawn = await source(text);
    _previewInFlight = false;
    if (_disposed) {
      return;
    }
    drawn.fold((_) {}, (blocks) {
      _previews = blocks;
      _previewText = text;
      notifyListeners();
    });
    if (_previewDue && !identical(controller.text, text)) {
      _askPreview();
    }
  }

  @override
  void dispose() {
    _disposed = true;
    _previewTimer?.cancel();
    controller.removeListener(_changed);
    super.dispose();
  }
}
