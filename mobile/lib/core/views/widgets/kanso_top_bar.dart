// Kanso — kanso_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

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
                  // More actions than the bar has room for beside [leading]
                  // are drawn a little smaller, all of them in reach, rather
                  // than run under it and off the edge.
                  trailing: trailing.isEmpty
                      ? null
                      : FittedBox(
                          fit: BoxFit.scaleDown,
                          alignment: AlignmentDirectional.centerEnd,
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: trailing,
                          ),
                        ),
                ),
              ),
            ),
            if (bordered) _shown(context, const KansoDivider()),
          ],
        ),
      ),
    );
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
