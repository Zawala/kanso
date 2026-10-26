// Kanso — kanso_swipe_actions.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart' show CustomSemanticsAction;

import '../../configs/kanso_motion.dart';
import '../../models/data/kanso_swipe_action.dart';
import '../../utils/swipe_outcome.dart';
import 'kanso_swipe_backdrop.dart';
import 'kanso_swipe_group.dart';
import 'kanso_swipe_scope.dart';

/// A row that can be swiped sideways to show actions behind it: the
/// [startActions] when it is moved towards the end of the row, the
/// [endActions] when it is moved the other way.
///
/// What happens when the finger lets go is decided by `swipeOutcome`: the
/// row goes back, stays with its actions shown for a tap, or (with
/// [onCommitStart] and [onCommitEnd]) does the outer action of that side at
/// once and goes back. [onArmed] is called when a swipe reaches the point
/// where letting go does that. Only one row of a `KansoSwipeHost` has its
/// actions shown at a time; [id] tells them apart.
///
/// The actions are also offered to a screen reader as custom actions, so a
/// swipe is never the only way.
class KansoSwipeActions extends StatefulWidget {
  const KansoSwipeActions({
    required this.id,
    required this.child,
    this.startActions = const <KansoSwipeAction>[],
    this.endActions = const <KansoSwipeAction>[],
    this.onCommitStart,
    this.onCommitEnd,
    this.onArmed,
    this.enabled = true,
    super.key,
  });

  final Object id;
  final Widget child;
  final List<KansoSwipeAction> startActions;
  final List<KansoSwipeAction> endActions;
  final VoidCallback? onCommitStart;
  final VoidCallback? onCommitEnd;
  final VoidCallback? onArmed;

  /// Whether a swipe does anything.
  final bool enabled;

  @override
  State<KansoSwipeActions> createState() => _KansoSwipeActionsState();
}

class _KansoSwipeActionsState extends State<KansoSwipeActions>
    with SingleTickerProviderStateMixin {
  /// How far the row is moved, as a share of its width: positive towards the
  /// end of the row.
  late final AnimationController controller;
  KansoSwipeGroup? group;
  double width = 0;
  bool armed = false;

  @override
  void initState() {
    super.initState();
    controller = AnimationController(vsync: this, value: 0, lowerBound: -1);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final next = KansoSwipeScope.maybeOf(context);
    if (next != group) {
      group?.removeListener(otherRowOpened);
      group = next?..addListener(otherRowOpened);
    }
  }

  @override
  void dispose() {
    group?.removeListener(otherRowOpened);
    controller.dispose();
    super.dispose();
  }

  void otherRowOpened() {
    if (group?.value != widget.id && controller.value != 0) {
      moveTo(0);
    }
  }

  bool get rightToLeft => Directionality.of(context) == TextDirection.rtl;

  void moveTo(double share) {
    controller.animateTo(
      share,
      duration: KansoMotion.resolve(context, KansoMotion.short),
      curve: KansoMotion.enter,
    );
  }

  void begin(DragStartDetails details) {
    controller.stop();
    group?.value = widget.id;
    armed = false;
  }

  void drag(DragUpdateDetails details) {
    final delta = (details.primaryDelta ?? 0) * (rightToLeft ? -1 : 1);
    var next = controller.value + delta / width;
    if ((next > 0 && widget.startActions.isEmpty) ||
        (next < 0 && widget.endActions.isEmpty)) {
      next = 0;
    }
    controller.value = next.clamp(-1.0, 1.0);
    final commit = next > 0
        ? widget.onCommitStart != null
        : widget.onCommitEnd != null;
    final nowArmed = commit && next.abs() >= SwipeThresholds.commitFraction;
    if (nowArmed && !armed) {
      widget.onArmed?.call();
    }
    armed = nowArmed;
  }

  void release(DragEndDetails details) {
    final offset = controller.value * width;
    final along = (details.primaryVelocity ?? 0) * (rightToLeft ? -1 : 1);
    final outcome = swipeOutcome(
      offset: offset,
      velocity: offset < 0 ? -along : along,
      width: width,
      canStart: widget.startActions.isNotEmpty,
      canEnd: widget.endActions.isNotEmpty,
      startCommits: widget.onCommitStart != null,
      endCommits: widget.onCommitEnd != null,
      endActions: widget.endActions.length,
    );
    armed = false;
    switch (outcome) {
      case SwipeOutcome.none:
        moveTo(0);
      case SwipeOutcome.revealStart:
        moveTo(
          SwipeThresholds.actionWidth * widget.startActions.length / width,
        );
      case SwipeOutcome.revealEnd:
        moveTo(-SwipeThresholds.actionWidth * widget.endActions.length / width);
      case SwipeOutcome.commitStart:
        widget.onCommitStart?.call();
        moveTo(0);
      case SwipeOutcome.commitEnd:
        widget.onCommitEnd?.call();
        moveTo(0);
    }
  }

  @override
  Widget build(BuildContext context) {
    final actions = <KansoSwipeAction>[
      ...widget.startActions,
      ...widget.endActions,
    ];
    final row = Semantics(
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        for (final action in actions)
          CustomSemanticsAction(label: action.label): action.onPressed,
      },
      child: widget.child,
    );
    if (!widget.enabled || actions.isEmpty) {
      return row;
    }
    return LayoutBuilder(
      builder: (context, constraints) {
        width = constraints.maxWidth;
        return GestureDetector(
          onHorizontalDragStart: begin,
          onHorizontalDragUpdate: drag,
          onHorizontalDragEnd: release,
          child: ClipRect(
            child: AnimatedBuilder(
              animation: controller,
              builder: (context, child) {
                final share = controller.value;
                final open = share != 0;
                final logical = share * width;
                return Stack(
                  children: <Widget>[
                    if (open)
                      Positioned.fill(
                        child: KansoSwipeBackdrop(
                          // A tap on an action closes the row, then acts.
                          actions: <KansoSwipeAction>[
                            for (final action
                                in share > 0
                                    ? widget.startActions
                                    : widget.endActions)
                              action.copyWith(
                                onPressed: () {
                                  moveTo(0);
                                  action.onPressed();
                                },
                              ),
                          ],
                          shown: logical.abs(),
                          atStart: share > 0 != rightToLeft,
                        ),
                      ),
                    Transform.translate(
                      offset: Offset(rightToLeft ? -logical : logical, 0),
                      // A tap on a row that has its actions shown closes it
                      // and goes no further.
                      child: open
                          ? GestureDetector(
                              behavior: HitTestBehavior.opaque,
                              onTap: () => moveTo(0),
                              child: IgnorePointer(child: child),
                            )
                          : child,
                    ),
                  ],
                );
              },
              child: row,
            ),
          ),
        );
      },
    );
  }
}
