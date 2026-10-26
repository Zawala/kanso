// Kanso — kanso_drawer.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_shadows.dart';
import 'kanso_divider.dart';
import 'kanso_drawer_controller.dart';
import 'kanso_drawer_scope.dart';
import 'kanso_scrim.dart';

/// A panel that slides in over the start edge of [child], with a veil over
/// what it covers: the library on a phone.
///
/// The pages under it find its [KansoDrawerController] with
/// [KansoDrawerScope.maybeOf] to open it. A tap on the veil, a drag of the
/// panel towards the edge, and the system's back close it. With
/// [edgeDragEnabled], dragging from the start edge opens it too.
class KansoDrawer extends StatefulWidget {
  const KansoDrawer({
    required this.width,
    required this.drawer,
    required this.child,
    this.edgeDragEnabled = false,
    super.key,
  });

  /// How wide the panel is.
  final double width;

  /// What is in the panel.
  final Widget drawer;
  final Widget child;
  final bool edgeDragEnabled;

  /// How far from the start edge a drag opens the drawer from.
  static const double edgeWidth = 24;

  /// The speed of a fling, in points a second, that opens or closes it.
  static const double flingVelocity = 700;

  @override
  State<KansoDrawer> createState() => KansoDrawerState();
}

/// The state of a [KansoDrawer]: the panel's position, and the controller of
/// what is inside.
class KansoDrawerState extends State<KansoDrawer>
    with SingleTickerProviderStateMixin
    implements KansoDrawerController {
  late final AnimationController _animation = AnimationController(
    vsync: this,
    duration: KansoMotion.content,
    reverseDuration: KansoMotion.short,
  );

  @override
  bool get isOpen =>
      _animation.status == AnimationStatus.forward ||
      _animation.status == AnimationStatus.completed;

  @override
  void open() => _animation.animateTo(1, curve: KansoMotion.enter);

  @override
  void close() => _animation.animateBack(0, curve: KansoMotion.exit);

  @override
  void toggle() => isOpen ? close() : open();

  @override
  void didUpdateWidget(KansoDrawer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.width != widget.width && !_animation.isDismissed) {
      // The window changed shape under an open drawer: close it.
      _animation.value = 0;
    }
  }

  @override
  void dispose() {
    _animation.dispose();
    super.dispose();
  }

  void _drag(DragUpdateDetails details) {
    final delta = (details.primaryDelta ?? 0) / widget.width;
    final direction = Directionality.of(context) == TextDirection.rtl ? -1 : 1;
    _animation.value += delta * direction;
  }

  void _release(DragEndDetails details) {
    final velocity =
        (details.primaryVelocity ?? 0) *
        (Directionality.of(context) == TextDirection.rtl ? -1 : 1);
    if (velocity.abs() >= KansoDrawer.flingVelocity) {
      velocity > 0 ? open() : close();
    } else {
      _animation.value > 0.5 ? open() : close();
    }
  }

  @override
  Widget build(BuildContext context) {
    return KansoDrawerScope(
      controller: this,
      child: AnimatedBuilder(
        animation: _animation,
        builder: (context, child) {
          final open = !_animation.isDismissed;
          return PopScope(
            // The system's back closes an open drawer before it leaves.
            canPop: !isOpen,
            onPopInvokedWithResult: (didPop, result) {
              if (!didPop) {
                close();
              }
            },
            child: Stack(
              children: <Widget>[
                Positioned.fill(child: widget.child),
                if (widget.edgeDragEnabled && !open)
                  PositionedDirectional(
                    top: 0,
                    bottom: 0,
                    start: 0,
                    width: KansoDrawer.edgeWidth,
                    child: GestureDetector(
                      behavior: HitTestBehavior.translucent,
                      onHorizontalDragUpdate: _drag,
                      onHorizontalDragEnd: _release,
                    ),
                  ),
                if (open) ...<Widget>[
                  Positioned.fill(
                    child: FadeTransition(
                      opacity: _animation,
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: close,
                        child: const KansoScrim(),
                      ),
                    ),
                  ),
                  PositionedDirectional(
                    top: 0,
                    bottom: 0,
                    start: -widget.width * (1 - _animation.value),
                    width: widget.width,
                    child: GestureDetector(
                      onHorizontalDragUpdate: _drag,
                      onHorizontalDragEnd: _release,
                      child: DecoratedBox(
                        decoration: BoxDecoration(
                          boxShadow: KansoShadows.sheet(
                            Theme.brightnessOf(context),
                          ),
                        ),
                        child: Row(
                          children: <Widget>[
                            Expanded(child: widget.drawer),
                            const KansoDivider(vertical: true),
                          ],
                        ),
                      ),
                    ),
                  ),
                ],
              ],
            ),
          );
        },
      ),
    );
  }
}
