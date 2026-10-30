// Kanso — native_preview.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/utils/context_extensions.dart';
import 'preview_blocks.dart';
import 'preview_style.dart';

/// A note's preview drawn from the core's blocks, in the fitted preview
/// palette and the note's fonts. Its text can be selected across blocks; a
/// tap on an address hands it to [onLink]; a double tap asks to edit.
class NativePreview extends StatefulWidget {
  const NativePreview({
    required this.blocks,
    required this.palette,
    required this.onLink,
    this.onEdit,
    super.key,
  });

  final List<PreviewBlock> blocks;

  /// The preview theme's colours, fitted with the interface and syntax
  /// themes in force.
  final PaletteDto palette;

  final ValueChanged<String> onLink;

  final VoidCallback? onEdit;

  @override
  State<NativePreview> createState() => NativePreviewState();
}

class NativePreviewState extends State<NativePreview> {
  final Map<String, GlobalKey> _headings = <String, GlobalKey>{};

  /// The headings drawn with a key: the first with each id that has one. A
  /// later heading with the same id has none, and scrolling goes to the
  /// first, as on the page.
  final Map<PreviewBlock_Heading, GlobalKey> _keyed =
      Map<PreviewBlock_Heading, GlobalKey>.identity();

  GlobalKey? _headingKey(PreviewBlock_Heading heading) => _keyed[heading];

  /// Finds the first heading with each id in [blocks], and the lists and
  /// quotes in them, and gives it its id's key.
  void _keyHeadings(List<PreviewBlock> blocks, Set<String> seen) {
    for (final block in blocks) {
      switch (block) {
        case PreviewBlock_Heading(:final id):
          if (id.isNotEmpty && seen.add(id)) {
            _keyed[block] = _headings.putIfAbsent(
              id,
              () => GlobalKey(debugLabel: 'heading $id'),
            );
          }
        case PreviewBlock_Quote(:final blocks):
          _keyHeadings(blocks, seen);
        case PreviewBlock_List(:final items):
          for (final item in items) {
            _keyHeadings(item.blocks, seen);
          }
        default:
          break;
      }
    }
  }

  /// The heading [anchor]'s key, while it is drawn.
  @visibleForTesting
  GlobalKey? headingKey(String anchor) => _headings[anchor];

  /// Scrolls the note so that the heading [anchor] is at the top.
  Future<void> scrollTo(String anchor) async {
    final target = _headings[anchor]?.currentContext;
    if (target == null) {
      return;
    }
    await Scrollable.ensureVisible(
      target,
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeOutCubic,
    );
  }

  @override
  Widget build(BuildContext context) {
    _keyed.clear();
    final seen = <String>{};
    _keyHeadings(widget.blocks, seen);
    _headings.removeWhere((id, _) => !seen.contains(id));
    final style = PreviewStyle(
      palette: widget.palette,
      typography: context.typography,
    );
    return Semantics(
      label: context.l10n.previewLabel,
      container: true,
      child: _DoubleTap(
        onDoubleTap: widget.onEdit,
        child: DefaultSelectionStyle(
          selectionColor: style.accent.withValues(alpha: 0.3),
          cursorColor: style.accent,
          child: SelectionArea(
            child: PreviewScope(
              style: style,
              onLink: widget.onLink,
              headingKey: _headingKey,
              child: PreviewBlocks(widget.blocks),
            ),
          ),
        ),
      ),
    );
  }
}

/// Sees a double tap without taking part in the gestures under it, so the
/// text's selection and links keep theirs.
class _DoubleTap extends StatefulWidget {
  const _DoubleTap({required this.child, this.onDoubleTap});

  final Widget child;
  final VoidCallback? onDoubleTap;

  @override
  State<_DoubleTap> createState() => _DoubleTapState();
}

class _DoubleTapState extends State<_DoubleTap> {
  Offset? _down;
  Duration? _downAt;
  Offset? _lastTap;
  Duration? _lastTapAt;

  void _onDown(PointerDownEvent event) {
    _down = event.position;
    _downAt = event.timeStamp;
  }

  void _onUp(PointerUpEvent event) {
    final down = _down;
    final downAt = _downAt;
    _down = null;
    if (down == null ||
        downAt == null ||
        (event.position - down).distance > kTouchSlop ||
        event.timeStamp - downAt > kLongPressTimeout) {
      _lastTap = null;
      return;
    }
    final last = _lastTap;
    final lastAt = _lastTapAt;
    if (last != null &&
        lastAt != null &&
        event.timeStamp - lastAt <= kDoubleTapTimeout &&
        (event.position - last).distance <= kDoubleTapSlop) {
      _lastTap = null;
      widget.onDoubleTap?.call();
      return;
    }
    _lastTap = event.position;
    _lastTapAt = event.timeStamp;
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: _onDown,
      onPointerUp: _onUp,
      onPointerCancel: (_) => _down = null,
      child: widget.child,
    );
  }
}
