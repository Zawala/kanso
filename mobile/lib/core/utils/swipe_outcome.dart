// Kanso — swipe_outcome.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a swipe on a row does when the finger lets go.
enum SwipeOutcome {
  /// Nothing: the row goes back.
  none,

  /// The start actions stay shown, for a tap.
  revealStart,

  /// The end actions stay shown, for a tap.
  revealEnd,

  /// The first action of the start side is done at once.
  commitStart,

  /// The outermost action of the end side is done at once.
  commitEnd,
}

/// The numbers that decide a swipe.
abstract final class SwipeThresholds {
  /// 72: how wide one revealed action is.
  static const double actionWidth = 72;

  /// 0.55: how far across the row a swipe is when it is a full swipe, as a
  /// share of the row's width.
  static const double commitFraction = 0.55;

  /// 900: how fast, in points a second, a swipe in its own direction is a
  /// fling; a fling that has revealed every action is a full swipe.
  static const double flingVelocity = 900;

  /// How much of the actions' width a swipe must have shown to keep them
  /// shown, as a share.
  static const double revealFraction = 0.5;
}

/// What a swipe that let go with the row moved [offset] points (positive:
/// towards the end of the row, which shows the start actions; negative: the
/// other way) is worth. [velocity] is the finger's speed along the
/// direction of [offset] in points a second (negative when it is coming
/// back), and [width] the row's width.
///
/// A swipe less than half way to showing the actions does nothing; one past
/// that keeps them shown; one that reaches [SwipeThresholds.commitFraction]
/// of the width, or is a fling that has shown all the actions, does the
/// outer action. A side with no actions ([canStart], [canEnd]) does nothing;
/// a side that does not commit ([startCommits], [endCommits]) stops at
/// showing them. A finger
/// coming back fast closes the row.
SwipeOutcome swipeOutcome({
  required double offset,
  required double velocity,
  required double width,
  bool canStart = true,
  bool canEnd = true,
  bool startCommits = true,
  bool endCommits = true,
  int endActions = 2,
}) {
  if (offset == 0 || velocity <= -SwipeThresholds.flingVelocity) {
    return SwipeOutcome.none;
  }
  final towardsEnd = offset > 0;
  if (towardsEnd ? !canStart : !canEnd) {
    return SwipeOutcome.none;
  }
  final shown = offset.abs();
  final actions = towardsEnd ? 1 : endActions;
  final reach = SwipeThresholds.actionWidth * actions;
  final fling = velocity >= SwipeThresholds.flingVelocity;
  final commits = towardsEnd ? startCommits : endCommits;
  if (commits &&
      (shown >= SwipeThresholds.commitFraction * width ||
          (fling && shown >= reach))) {
    return towardsEnd ? SwipeOutcome.commitStart : SwipeOutcome.commitEnd;
  }
  if (shown >= reach * SwipeThresholds.revealFraction) {
    return towardsEnd ? SwipeOutcome.revealStart : SwipeOutcome.revealEnd;
  }
  return SwipeOutcome.none;
}
