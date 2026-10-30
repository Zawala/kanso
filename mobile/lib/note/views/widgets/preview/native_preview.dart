// Kanso — native_preview.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:super_sliver_list/super_sliver_list.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/utils/context_extensions.dart';
import 'preview_blocks.dart';
import 'preview_code.dart';
import 'preview_style.dart';

/// A note's preview drawn from the core's blocks, in the fitted preview
/// palette and the note's fonts. It is a sliver: only the blocks near the
/// screen are built and laid out, and a block equal to the one drawn
/// before isn't built again. The blocks far from the screen are never laid
/// out: their heights are guessed, so a jump far down is as quick as a
/// short one. A long code block is drawn in pieces, each built as it comes
/// near. Under a [SelectionArea] round its scroll, its text can be selected
/// across blocks; a tap on an address hands it to [onLink]; a double tap
/// asks to edit.
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

  /// How many times a top-level block, or a piece of a long code block, has
  /// been built, in debug builds.
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

  /// The block last built at each place in the list, given again while it
  /// is the same.
  final Map<int, _TopBlock> _built = <int, _TopBlock>{};

  /// Where in the list each top-level block starts, and the list's end,
  /// when a long code block takes more than one place; null while each
  /// block takes one.
  Int32List? _starts;

  /// The sideways scroll of each long code block's pieces.
  final Map<int, PreviewCodeScroll> _codeScrolls = <int, PreviewCodeScroll>{};

  final ListController _list = ListController();

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
    Int32List? starts;
    var items = 0;
    for (var at = 0; at < blocks.length; at++) {
      final block = blocks[at];
      final chunks = block is PreviewBlock_Code
          ? PreviewCodeLines.chunksOf(block.runs)
          : 1;
      if (chunks > 1 && starts == null) {
        starts = Int32List(blocks.length + 1);
        for (var before = 0; before < at; before++) {
          starts[before] = before;
        }
      }
      starts?[at] = items;
      items += chunks;
      List<GlobalKey?>? found;
      _headingsIn(block, (heading) {
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
    starts?[blocks.length] = items;
    _keys = keys;
    _starts = starts;
    _headings.removeWhere((id, _) => !seen.contains(id));
    _built.removeWhere((at, _) => at >= items);
    _codeScrolls.removeWhere(
      (at, _) =>
          starts == null ||
          at >= blocks.length ||
          starts[at + 1] - starts[at] < 2,
    );
  }

  /// How many places the blocks take in the list.
  int get _items => _starts?.last ?? widget.blocks.length;

  /// The place in the list where the top-level block [at] starts.
  int _itemOf(int at) => _starts?[at] ?? at;

  /// The top-level block at the place [item] in the list, and which piece
  /// of it, for a long code block.
  (int, int?) _blockAt(int item) {
    final starts = _starts;
    if (starts == null) {
      return (item, null);
    }
    // The last block that starts at or before [item].
    var low = 0;
    var high = starts.length - 1;
    while (high - low > 1) {
      final middle = (low + high) ~/ 2;
      if (starts[middle] <= item) {
        low = middle;
      } else {
        high = middle;
      }
    }
    final at = low;
    final chunks = starts[at + 1] - starts[at];
    return (at, chunks > 1 ? item - starts[at] : null);
  }

  @override
  void dispose() {
    _list.dispose();
    for (final scroll in _codeScrolls.values) {
      scroll.dispose();
    }
    super.dispose();
  }

  /// The heading [anchor]'s key, while it is drawn.
  @visibleForTesting
  GlobalKey? headingKey(String anchor) => _headings[anchor];

  /// Scrolls the note so that the heading [anchor] is at the top. When its
  /// block isn't built yet, the note first jumps to it.
  Future<void> scrollTo(String anchor) async {
    final key = _headings[anchor];
    final at = _anchors[anchor];
    if (key == null || at == null) {
      return;
    }
    if (key.currentContext != null) {
      // Near: glide to it.
      await _reveal(key, const Duration(milliseconds: 300));
      return;
    }
    final position = Scrollable.maybeOf(context)?.position;
    if (position == null || !_list.isAttached) {
      return;
    }
    // Far: jump to its block. The list jumps by a scroll controller; the
    // note's scroll has none of its own, so one is lent its position.
    final controller = ScrollController()..attach(position);
    try {
      _list.jumpToItem(
        index: _itemOf(at),
        scrollController: controller,
        alignment: 0,
      );
    } finally {
      controller
        ..detach(position)
        ..dispose();
    }
    await WidgetsBinding.instance.endOfFrame;
    // The heading may be below the top of its block, in a list or a quote.
    // Jumped there too: the heights of the blocks around it are measured as
    // they are built, and a glide's end would be out by then.
    await _reveal(key, Duration.zero);
  }

  Future<void> _reveal(GlobalKey key, Duration duration) async {
    final target = key.currentContext;
    if (!mounted || target == null || !target.mounted) {
      return;
    }
    await Scrollable.ensureVisible(
      target,
      duration: duration,
      curve: Curves.easeOutCubic,
    );
  }

  void _onLink(String url) => widget.onLink(url);

  static GlobalKey? _noKey(PreviewBlock_Heading heading) => null;

  Widget _block(BuildContext context, int item) {
    final blocks = widget.blocks;
    final (at, chunk) = _blockAt(item);
    final block = blocks[at];
    final before = at == 0 ? null : blocks[at - 1];
    final keys = _keys[at] ?? const <GlobalKey?>[];
    final piece = chunk == null
        ? null
        : (
            chunk: chunk,
            last: item + 1 == _starts![at + 1],
            scroll: _codeScrolls.putIfAbsent(at, PreviewCodeScroll.new),
          );
    var built = _built[item];
    if (built == null || !built.draws(block, before, keys, piece)) {
      built = _built[item] = _TopBlock(
        key: ValueKey<(int, Type, int?)>((item, block.runtimeType, chunk)),
        block: block,
        before: before,
        keys: keys,
        piece: piece,
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
          child: SuperSliverList.builder(
            listController: _list,
            itemCount: _items,
            itemBuilder: _block,
          ),
        ),
      ),
    );
  }
}

/// Which piece of a long code block a place in the list draws: the
/// `chunk`th, whether it is the `last`, and the sideways `scroll` the
/// pieces share.
typedef _Piece = ({int chunk, bool last, PreviewCodeScroll scroll});

/// A block at the top of a note, below the room between it and the block
/// [before] it. Its headings are drawn with [keys], in the order they come.
/// For a long code block, it is the [piece] of it; the pieces after the
/// first have no room above.
class _TopBlock extends StatelessWidget {
  const _TopBlock({
    required this.block,
    required this.before,
    required this.keys,
    required this.piece,
    super.key,
  });

  final PreviewBlock block;
  final PreviewBlock? before;
  final List<GlobalKey?> keys;
  final _Piece? piece;

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
    _Piece? piece,
  ) =>
      _spacing(before) == _spacing(this.before) &&
      listEquals(keys, this.keys) &&
      piece == this.piece &&
      (identical(block, this.block) || block == this.block);

  @override
  Widget build(BuildContext context) {
    assert(() {
      NativePreview.debugBlockBuilds++;
      return true;
    }(), 'counts the builds');
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final piece = this.piece;
    final room = before == null || (piece != null && piece.chunk > 0)
        ? 0.0
        : PreviewBlocks.room(before!, block, style);
    Widget child;
    final code = block;
    if (piece != null && code is PreviewBlock_Code) {
      child = PreviewCodeChunk(
        language: code.language,
        runs: PreviewCodeLines.of(code.runs).chunk(piece.chunk),
        copy: code.copy,
        first: piece.chunk == 0,
        last: piece.last,
        scroll: piece.scroll,
      );
    } else {
      child = PreviewBlockView(block);
    }
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
