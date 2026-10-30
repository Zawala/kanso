// Kanso — native_preview.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/utils/context_extensions.dart';
import 'preview_blocks.dart';
import 'preview_style.dart';

/// A note's preview drawn from the core's blocks, in the fitted preview
/// palette and the note's fonts. It is a sliver: only the blocks near the
/// screen are built and laid out, and a block equal to the one drawn
/// before isn't built again. Under a [SelectionArea] round its scroll, its
/// text can be selected across blocks; a tap on an address hands it to
/// [onLink]; a double tap asks to edit.
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

  /// How many times a top-level block has been built, in debug builds.
  @visibleForTesting
  static int debugBlockBuilds = 0;

  @override
  State<NativePreview> createState() => NativePreviewState();
}

class NativePreviewState extends State<NativePreview> {
  final Map<String, GlobalKey> _headings = <String, GlobalKey>{};

  /// The top-level block each keyed heading is in.
  final Map<String, int> _anchors = <String, int>{};

  /// The keys of the headings in each top-level block that has any, in the
  /// order they come: the first heading with each id that has one is
  /// drawn with its id's key; a later one with the same id has none, and
  /// scrolling goes to the first, as on the page.
  Map<int, List<GlobalKey?>> _keys = const <int, List<GlobalKey?>>{};

  /// The blocks [_keys] were worked out for.
  List<PreviewBlock>? _keyed;

  /// The block last built at each place, given again while it is the same.
  final Map<int, _TopBlock> _built = <int, _TopBlock>{};

  late final _DoubleTap _taps = _DoubleTap(() => widget.onEdit?.call());

  PreviewStyle? _style;

  /// Calls [visit] with each heading in [block], quotes and lists
  /// included, in the order they come.
  static void _headingsIn(
    PreviewBlock block,
    void Function(PreviewBlock_Heading heading) visit,
  ) {
    switch (block) {
      case final PreviewBlock_Heading heading:
        visit(heading);
      case PreviewBlock_Quote(:final blocks):
        for (final inner in blocks) {
          _headingsIn(inner, visit);
        }
      case PreviewBlock_List(:final items):
        for (final item in items) {
          for (final inner in item.blocks) {
            _headingsIn(inner, visit);
          }
        }
      default:
        break;
    }
  }

  void _keyHeadings() {
    final blocks = widget.blocks;
    if (identical(blocks, _keyed)) {
      return;
    }
    _keyed = blocks;
    _anchors.clear();
    final seen = <String>{};
    final keys = <int, List<GlobalKey?>>{};
    for (var at = 0; at < blocks.length; at++) {
      List<GlobalKey?>? found;
      _headingsIn(blocks[at], (heading) {
        final id = heading.id;
        GlobalKey? key;
        if (id.isNotEmpty && seen.add(id)) {
          _anchors[id] = at;
          key = _headings.putIfAbsent(
            id,
            () => GlobalKey(debugLabel: 'heading $id'),
          );
        }
        (found ??= <GlobalKey?>[]).add(key);
      });
      if (found != null) {
        keys[at] = found!;
      }
    }
    _keys = keys;
    _headings.removeWhere((id, _) => !seen.contains(id));
    _built.removeWhere((at, _) => at >= blocks.length);
  }

  /// The heading [anchor]'s key, while it is drawn.
  @visibleForTesting
  GlobalKey? headingKey(String anchor) => _headings[anchor];

  /// Scrolls the note so that the heading [anchor] is at the top. When its
  /// block isn't built yet, the note first jumps to where it is likely to
  /// be, until it is.
  Future<void> scrollTo(String anchor) async {
    final key = _headings[anchor];
    final index = _anchors[anchor];
    if (key == null || index == null) {
      return;
    }
    for (var tries = 0; key.currentContext == null && tries < 8; tries++) {
      if (!mounted) {
        return;
      }
      final sliver = context.findRenderObject();
      final position = Scrollable.maybeOf(context)?.position;
      final geometry = sliver is RenderSliver ? sliver.geometry : null;
      if (sliver is! RenderSliver || geometry == null || position == null) {
        return;
      }
      final count = math.max(widget.blocks.length, 1);
      final guess =
          sliver.constraints.precedingScrollExtent +
          geometry.scrollExtent * index / count;
      position.jumpTo(
        guess.clamp(position.minScrollExtent, position.maxScrollExtent),
      );
      await WidgetsBinding.instance.endOfFrame;
    }
    final target = key.currentContext;
    if (target == null || !target.mounted) {
      return;
    }
    await Scrollable.ensureVisible(
      target,
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeOutCubic,
    );
  }

  void _onLink(String url) => widget.onLink(url);

  static GlobalKey? _noKey(PreviewBlock_Heading heading) => null;

  Widget _block(BuildContext context, int at) {
    final blocks = widget.blocks;
    final block = blocks[at];
    final before = at == 0 ? null : blocks[at - 1];
    final keys = _keys[at] ?? const <GlobalKey?>[];
    var built = _built[at];
    if (built == null || !built.draws(block, before, keys)) {
      built = _built[at] = _TopBlock(
        key: ValueKey<(int, Type)>((at, block.runtimeType)),
        block: block,
        before: before,
        keys: keys,
      );
    }
    return Listener(
      behavior: HitTestBehavior.translucent,
      onPointerDown: _taps.down,
      onPointerUp: _taps.up,
      onPointerCancel: _taps.cancel,
      child: built,
    );
  }

  @override
  Widget build(BuildContext context) {
    _keyHeadings();
    final style = PreviewStyle(
      palette: widget.palette,
      typography: context.typography,
    );
    // The same style is kept, so the blocks drawn don't hear of a change.
    if (style != _style) {
      _style = style;
    }
    return SliverSemantics(
      label: context.l10n.previewLabel,
      container: true,
      sliver: DefaultSelectionStyle(
        selectionColor: style.accent.withValues(alpha: 0.3),
        cursorColor: style.accent,
        child: PreviewScope(
          style: _style!,
          onLink: _onLink,
          headingKey: _noKey,
          child: SliverList.builder(
            itemCount: widget.blocks.length,
            itemBuilder: _block,
          ),
        ),
      ),
    );
  }
}

/// A block at the top of a note, below the room between it and the block
/// [before] it. Its headings are drawn with [keys], in the order they come.
class _TopBlock extends StatelessWidget {
  const _TopBlock({
    required this.block,
    required this.before,
    required this.keys,
    super.key,
  });

  final PreviewBlock block;
  final PreviewBlock? before;
  final List<GlobalKey?> keys;

  /// What the room above a block depends on in the block before it.
  static Object? _spacing(PreviewBlock? block) => switch (block) {
    null => null,
    PreviewBlock_Heading(:final level) => ('heading', level),
    PreviewBlock_Figure(:final kind) => ('figure', kind),
    _ => block.runtimeType,
  };

  /// Whether this draws [block] below [before] with [keys], as it is.
  bool draws(
    PreviewBlock block,
    PreviewBlock? before,
    List<GlobalKey?> keys,
  ) =>
      _spacing(before) == _spacing(this.before) &&
      listEquals(keys, this.keys) &&
      (identical(block, this.block) || block == this.block);

  @override
  Widget build(BuildContext context) {
    assert(() {
      NativePreview.debugBlockBuilds++;
      return true;
    }(), 'counts the builds');
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final room = before == null
        ? 0.0
        : PreviewBlocks.room(before!, block, style);
    Widget child = PreviewBlockView(block);
    if (keys.isNotEmpty) {
      final keyed = Map<PreviewBlock_Heading, GlobalKey>.identity();
      var at = 0;
      NativePreviewState._headingsIn(block, (heading) {
        final key = keys[at++];
        if (key != null) {
          keyed[heading] = key;
        }
      });
      child = PreviewScope(
        style: style,
        onLink: scope.onLink,
        headingKey: (heading) => keyed[heading],
        depth: scope.depth,
        child: child,
      );
    }
    return Padding(
      padding: EdgeInsets.only(top: room),
      child: child,
    );
  }
}

/// Sees a double tap from the pointers under the blocks, without taking
/// part in the gestures there, so the text's selection and links keep
/// theirs.
class _DoubleTap {
  _DoubleTap(this.onDoubleTap);

  final VoidCallback onDoubleTap;

  Offset? _down;
  Duration? _downAt;
  Offset? _lastTap;
  Duration? _lastTapAt;

  void down(PointerDownEvent event) {
    _down = event.position;
    _downAt = event.timeStamp;
  }

  void cancel(PointerCancelEvent event) => _down = null;

  void up(PointerUpEvent event) {
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
      onDoubleTap();
      return;
    }
    _lastTap = event.position;
    _lastTapAt = event.timeStamp;
  }
}
