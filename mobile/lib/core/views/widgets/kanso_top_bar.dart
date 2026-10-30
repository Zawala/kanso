// Kanso — kanso_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// The bar at the top of a page: [leading], a centred title and [trailing].
///
/// It takes the status bar's height as well, so its colour reaches the top
/// of the screen; the content sits below it.
class KansoTopBar extends StatelessWidget {
  const KansoTopBar({
    this.leading,
    this.title,
    this.subtitle,
    this.trailing = const <Widget>[],
    this.backgroundColor,
    this.bordered = true,
    this.titleShown,
    super.key,
  });

  final Widget? leading;
  final String? title;

  /// A quiet line under the title.
  final String? subtitle;

  /// The actions at the end, in a row. They have the room [leading]
  /// leaves; one wrapped in a [KansoTopBarItem] gives way, when there is not
  /// enough, to the ones that are not.
  final List<Widget> trailing;

  /// The bar's colour; the colour of the page below when null.
  final Color? backgroundColor;

  /// A line along the bottom edge.
  final bool bordered;

  /// Whether the [title] and the line are shown, for a page whose large
  /// title scrolls with it: they fade in once it has gone under the bar.
  /// Always shown when null.
  final ValueListenable<bool>? titleShown;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final name = title;
    final note = subtitle;
    // The bar is 52 points, and taller when the device's text size makes
    // its title (and subtitle) taller than that.
    final scaler = MediaQuery.textScalerOf(context);
    double line(TextStyle style) =>
        scaler.scale(style.fontSize ?? 0) * (style.height ?? 1);
    final height = math.max(
      KansoSizes.barHeight,
      (name == null ? 0 : line(typography.barTitle)) +
          (note == null ? 0 : line(typography.meta)) +
          2 * KansoSpacing.xxs,
    );
    return ColoredBox(
      color: backgroundColor ?? colors.editor,
      child: SafeArea(
        bottom: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            SizedBox(
              height: height,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.xxs,
                ),
                child: _TopBarRow(
                  leading: leading,
                  middle: name == null
                      ? null
                      : _shown(
                          context,
                          Column(
                            mainAxisSize: MainAxisSize.min,
                            children: <Widget>[
                              Text(
                                name,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: typography.barTitle,
                              ),
                              if (note != null)
                                Text(
                                  note,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: typography.meta,
                                ),
                            ],
                          ),
                        ),
                  trailing: trailing.isEmpty
                      ? null
                      : _TrailingRow(children: trailing),
                ),
              ),
            ),
            if (bordered) _shown(context, const KansoDivider()),
          ],
        ),
      ),
    );
  }

  /// The ids of the [KansoTopBarItem]s of the bar above [context] that
  /// gave way for want of room, for a menu to offer instead. Nothing when
  /// [context] is not in a bar's [trailing].
  static Set<Object> hiddenOf(BuildContext context) {
    final row = context.findAncestorRenderObjectOfType<_RenderTrailingRow>();
    return <Object>{
      for (final child in row?.getChildrenAsList() ?? const <RenderBox>[])
        if (child.parentData case KansoTopBarParentData(
          hidden: true,
          :final id?,
        ))
          id,
    };
  }

  /// [child], fading in and out with [titleShown].
  Widget _shown(BuildContext context, Widget child) {
    final shown = titleShown;
    if (shown == null) {
      return child;
    }
    return ValueListenableBuilder<bool>(
      valueListenable: shown,
      child: child,
      // Gone, not just clear, while hidden: the large title is the one a
      // screen reader finds.
      builder: (context, visible, child) => AnimatedSwitcher(
        duration: KansoMotion.resolve(context, KansoMotion.fast),
        child: visible ? child : const SizedBox.shrink(),
      ),
    );
  }
}

enum _Slot { leading, middle, trailing }

/// [leading] at the start, [trailing] at the end, and [middle] centred on
/// the bar where it fits between them, as [NavigationToolbar] lays them
/// out; but [trailing] has only the room [leading] leaves, so it never runs
/// under it.
class _TopBarRow extends StatelessWidget {
  const _TopBarRow({this.leading, this.middle, this.trailing});

  final Widget? leading;
  final Widget? middle;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return CustomMultiChildLayout(
      delegate: _TopBarLayout(Directionality.of(context)),
      children: <Widget>[
        if (leading case final leading?)
          LayoutId(id: _Slot.leading, child: leading),
        if (middle case final middle?)
          LayoutId(id: _Slot.middle, child: middle),
        if (trailing case final trailing?)
          LayoutId(id: _Slot.trailing, child: trailing),
      ],
    );
  }
}

class _TopBarLayout extends MultiChildLayoutDelegate {
  _TopBarLayout(this.textDirection);

  final TextDirection textDirection;

  @override
  void performLayout(Size size) {
    final rtl = textDirection == TextDirection.rtl;
    var leadingWidth = 0.0;
    var trailingWidth = 0.0;
    if (hasChild(_Slot.leading)) {
      leadingWidth = layoutChild(
        _Slot.leading,
        BoxConstraints(
          maxWidth: size.width,
          minHeight: size.height,
          maxHeight: size.height,
        ),
      ).width;
      positionChild(
        _Slot.leading,
        Offset(rtl ? size.width - leadingWidth : 0, 0),
      );
    }
    if (hasChild(_Slot.trailing)) {
      final trailing = layoutChild(
        _Slot.trailing,
        BoxConstraints.loose(
          size,
        ).copyWith(maxWidth: math.max(size.width - leadingWidth, 0)),
      );
      trailingWidth = trailing.width;
      positionChild(
        _Slot.trailing,
        Offset(
          rtl ? 0 : size.width - trailingWidth,
          (size.height - trailing.height) / 2,
        ),
      );
    }
    if (hasChild(_Slot.middle)) {
      final middle = layoutChild(
        _Slot.middle,
        BoxConstraints.loose(size).copyWith(
          maxWidth: math.max(size.width - leadingWidth - trailingWidth, 0),
        ),
      );
      // Centred on the bar, unless that would put it under either side.
      final start = math.min(
        math.max((size.width - middle.width) / 2, leadingWidth),
        math.max(leadingWidth, size.width - trailingWidth - middle.width),
      );
      positionChild(
        _Slot.middle,
        Offset(
          rtl ? size.width - middle.width - start : start,
          (size.height - middle.height) / 2,
        ),
      );
    }
  }

  @override
  bool shouldRelayout(_TopBarLayout oldDelegate) =>
      oldDelegate.textDirection != textDirection;
}

/// One of a [KansoTopBar]'s trailing actions that gives way when the bar
/// has no room for all of them beside its leading one: it is then not shown,
/// and [KansoTopBar.hiddenOf] has its [id], so that a menu can offer it
/// instead. The one of lowest [priority] goes first; an action not wrapped in
/// one never does.
class KansoTopBarItem extends ParentDataWidget<KansoTopBarParentData> {
  const KansoTopBarItem({
    required this.id,
    required this.priority,
    required super.child,
    super.key,
  });

  /// What [KansoTopBar.hiddenOf] calls it.
  final Object id;
  final int priority;

  @override
  void applyParentData(RenderObject renderObject) {
    final data = renderObject.parentData! as KansoTopBarParentData;
    if (data.id != id || data.priority != priority) {
      data
        ..id = id
        ..priority = priority;
      renderObject.parent?.markNeedsLayout();
    }
  }

  @override
  Type get debugTypicalAncestorWidgetClass => KansoTopBar;
}

/// Where an action sits in a [KansoTopBar]'s trailing row, and whether it
/// gave way.
class KansoTopBarParentData extends ContainerBoxParentData<RenderBox> {
  /// From the [KansoTopBarItem] around it, if any.
  Object? id;
  int? priority;

  /// Whether it gave way for want of room: not drawn, touched or read out.
  bool hidden = false;
}

/// [KansoTopBar]'s actions in a row as narrow as they are, with the
/// [KansoTopBarItem]s that don't fit in the room it has left out.
class _TrailingRow extends MultiChildRenderObjectWidget {
  const _TrailingRow({required super.children});

  @override
  _RenderTrailingRow createRenderObject(BuildContext context) =>
      _RenderTrailingRow(Directionality.of(context));

  @override
  void updateRenderObject(
    BuildContext context,
    _RenderTrailingRow renderObject,
  ) {
    renderObject.textDirection = Directionality.of(context);
  }
}

class _RenderTrailingRow extends RenderBox
    with
        ContainerRenderObjectMixin<RenderBox, KansoTopBarParentData>,
        RenderBoxContainerDefaultsMixin<RenderBox, KansoTopBarParentData> {
  _RenderTrailingRow(this._textDirection);

  TextDirection _textDirection;
  set textDirection(TextDirection value) {
    if (value != _textDirection) {
      _textDirection = value;
      markNeedsLayout();
    }
  }

  Iterable<RenderBox> get _shown => getChildrenAsList().where(
    (child) => !(child.parentData! as KansoTopBarParentData).hidden,
  );

  @override
  void setupParentData(RenderBox child) {
    if (child.parentData is! KansoTopBarParentData) {
      child.parentData = KansoTopBarParentData();
    }
  }

  /// The children that give way within [constraints], each child sized by
  /// [sizeOf], and the row's size without them.
  (Set<RenderBox>, Size) _plan(
    BoxConstraints constraints,
    Size Function(RenderBox child, BoxConstraints constraints) sizeOf,
  ) {
    final loose = BoxConstraints(maxHeight: constraints.maxHeight);
    final children = getChildrenAsList();
    final sizes = <RenderBox, Size>{
      for (final child in children) child: sizeOf(child, loose),
    };
    var width = 0.0;
    for (final size in sizes.values) {
      width += size.width;
    }
    // Lowest priority first, and of the same, the first in the row.
    final yielding =
        <RenderBox>[
          for (final child in children)
            if ((child.parentData! as KansoTopBarParentData).priority != null &&
                sizes[child]!.width > 0)
              child,
        ]..sort((a, b) {
          int priority(RenderBox child) =>
              (child.parentData! as KansoTopBarParentData).priority!;
          final byPriority = priority(a).compareTo(priority(b));
          return byPriority != 0
              ? byPriority
              : children.indexOf(a).compareTo(children.indexOf(b));
        });
    final hidden = <RenderBox>{};
    for (final child in yielding) {
      if (width <= constraints.maxWidth + precisionErrorTolerance) {
        break;
      }
      hidden.add(child);
      width -= sizes[child]!.width;
    }
    var height = 0.0;
    for (final child in children) {
      if (!hidden.contains(child)) {
        height = math.max(height, sizes[child]!.height);
      }
    }
    return (hidden, constraints.constrain(Size(width, height)));
  }

  @override
  Size computeDryLayout(BoxConstraints constraints) =>
      _plan(constraints, (child, loose) => child.getDryLayout(loose)).$2;

  @override
  void performLayout() {
    final (hidden, size) = _plan(constraints, (child, loose) {
      child.layout(loose, parentUsesSize: true);
      return child.size;
    });
    this.size = size;
    final children = getChildrenAsList();
    var x = 0.0;
    var changed = false;
    for (final child
        in _textDirection == TextDirection.rtl ? children.reversed : children) {
      final data = child.parentData! as KansoTopBarParentData;
      if (data.hidden != hidden.contains(child)) {
        data.hidden = !data.hidden;
        changed = true;
      }
      if (data.hidden) {
        data.offset = Offset.zero;
        continue;
      }
      data.offset = Offset(x, (size.height - child.size.height) / 2);
      x += child.size.width;
    }
    if (changed) {
      markNeedsSemanticsUpdate();
    }
  }

  @override
  double computeMinIntrinsicWidth(double height) {
    var width = 0.0;
    for (final child in getChildrenAsList()) {
      if ((child.parentData! as KansoTopBarParentData).priority == null) {
        width += child.getMinIntrinsicWidth(height);
      }
    }
    return width;
  }

  @override
  double computeMaxIntrinsicWidth(double height) {
    var width = 0.0;
    for (final child in getChildrenAsList()) {
      width += child.getMaxIntrinsicWidth(height);
    }
    return width;
  }

  @override
  double computeMinIntrinsicHeight(double width) {
    var height = 0.0;
    for (final child in getChildrenAsList()) {
      height = math.max(height, child.getMinIntrinsicHeight(double.infinity));
    }
    return height;
  }

  @override
  double computeMaxIntrinsicHeight(double width) {
    var height = 0.0;
    for (final child in getChildrenAsList()) {
      height = math.max(height, child.getMaxIntrinsicHeight(double.infinity));
    }
    return height;
  }

  @override
  void paint(PaintingContext context, Offset offset) {
    for (final child in _shown) {
      final data = child.parentData! as KansoTopBarParentData;
      context.paintChild(child, data.offset + offset);
    }
  }

  @override
  bool hitTestChildren(BoxHitTestResult result, {required Offset position}) {
    for (final child in _shown.toList().reversed) {
      final data = child.parentData! as KansoTopBarParentData;
      final hit = result.addWithPaintOffset(
        offset: data.offset,
        position: position,
        hitTest: (result, transformed) =>
            child.hitTest(result, position: transformed),
      );
      if (hit) {
        return true;
      }
    }
    return false;
  }

  @override
  void visitChildrenForSemantics(RenderObjectVisitor visitor) {
    _shown.forEach(visitor);
  }
}
