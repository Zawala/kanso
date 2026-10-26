// Kanso — swipe_outcome_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/utils/swipe_outcome.dart';

import '../../helpers/property.dart';

SwipeOutcome _outcome(
  SwipeRelease release, {
  int? offset,
  int? velocity,
}) {
  final (shown, speed, width, sides) = release;
  return swipeOutcome(
    offset: (offset ?? shown).toDouble(),
    velocity: (velocity ?? speed).toDouble(),
    width: width.toDouble(),
    canStart: sides & 1 != 0,
    canEnd: sides & 2 != 0,
    startCommits: sides & 4 != 0,
    endCommits: sides & 8 != 0,
  );
}

/// How much of the actions a swipe has done: nothing, shown, or acted.
int _rank(SwipeOutcome outcome) => switch (outcome) {
  SwipeOutcome.none => 0,
  SwipeOutcome.revealStart || SwipeOutcome.revealEnd => 1,
  SwipeOutcome.commitStart || SwipeOutcome.commitEnd => 2,
};

void main() {
  group('swipeOutcome', () {
    test('a short swipe does nothing', () {
      expect(
        swipeOutcome(offset: 20, velocity: 0, width: 393),
        SwipeOutcome.none,
      );
      expect(
        swipeOutcome(offset: -50, velocity: 0, width: 393),
        SwipeOutcome.none,
      );
    });

    test('a swipe half way to the actions keeps them shown', () {
      expect(
        swipeOutcome(offset: 40, velocity: 0, width: 393),
        SwipeOutcome.revealStart,
      );
      // Two actions are 144 wide.
      expect(
        swipeOutcome(offset: -80, velocity: 0, width: 393),
        SwipeOutcome.revealEnd,
      );
    });

    test('a full swipe does the outer action', () {
      expect(
        swipeOutcome(offset: 230, velocity: 0, width: 393),
        SwipeOutcome.commitStart,
      );
      expect(
        swipeOutcome(offset: -230, velocity: 0, width: 393),
        SwipeOutcome.commitEnd,
      );
    });

    test('a fling that has shown the actions does the outer action', () {
      expect(
        swipeOutcome(offset: -150, velocity: 1500, width: 393),
        SwipeOutcome.commitEnd,
      );
      expect(
        swipeOutcome(offset: -100, velocity: 1500, width: 393),
        SwipeOutcome.revealEnd,
      );
    });

    test('a finger coming back fast closes the row', () {
      expect(
        swipeOutcome(offset: 200, velocity: -1500, width: 393),
        SwipeOutcome.none,
      );
    });

    test('a side with no actions, or that does not commit, is held back', () {
      expect(
        swipeOutcome(offset: 200, velocity: 0, width: 393, canStart: false),
        SwipeOutcome.none,
      );
      expect(
        swipeOutcome(offset: -300, velocity: 0, width: 393, endCommits: false),
        SwipeOutcome.revealEnd,
      );
      expect(
        swipeOutcome(
          offset: 300,
          velocity: 0,
          width: 393,
          startCommits: false,
        ),
        SwipeOutcome.revealStart,
      );
    });

    test('a single end action shows after 36 points', () {
      expect(
        swipeOutcome(offset: -40, velocity: 0, width: 393, endActions: 1),
        SwipeOutcome.revealEnd,
      );
    });

    Glados<SwipeRelease>(any.swipeRelease, kansoExploreConfig()).test(
      'the further the swipe, the more it does',
      (release) {
        final (offset, _, _, _) = release;
        final sign = offset < 0 ? -1 : 1;
        var last = 0;
        for (var shown = 0; shown <= 600; shown += 12) {
          final rank = _rank(_outcome(release, offset: sign * shown));
          expect(rank, greaterThanOrEqualTo(last), reason: 'at $shown');
          last = rank;
        }
      },
    );

    Glados<SwipeRelease>(any.swipeRelease, kansoExploreConfig()).test(
      'it acts only past the full swipe, or on a fling that has shown them',
      (release) {
        final (offset, velocity, width, _) = release;
        final outcome = _outcome(release);
        if (outcome == SwipeOutcome.commitStart ||
            outcome == SwipeOutcome.commitEnd) {
          final far = offset.abs() >= SwipeThresholds.commitFraction * width;
          final fling =
              velocity >= SwipeThresholds.flingVelocity &&
              offset.abs() >=
                  SwipeThresholds.actionWidth *
                      (outcome == SwipeOutcome.commitStart ? 1 : 2);
          expect(far || fling, isTrue);
        }
      },
    );

    Glados<SwipeRelease>(any.swipeRelease, kansoExploreConfig()).test(
      'the direction of the swipe says which side it is',
      (release) {
        final (offset, _, _, _) = release;
        final outcome = _outcome(release);
        if (offset > 0) {
          expect(outcome, isNot(SwipeOutcome.revealEnd));
          expect(outcome, isNot(SwipeOutcome.commitEnd));
        } else if (offset < 0) {
          expect(outcome, isNot(SwipeOutcome.revealStart));
          expect(outcome, isNot(SwipeOutcome.commitStart));
        } else {
          expect(outcome, SwipeOutcome.none);
        }
      },
    );

    Glados<SwipeRelease>(any.swipeRelease, kansoExploreConfig()).test(
      'a side without actions does nothing, and one that does not commit '
      'never acts',
      (release) {
        final (offset, _, _, sides) = release;
        final outcome = _outcome(release);
        if (offset > 0 && sides & 1 == 0) {
          expect(outcome, SwipeOutcome.none);
        }
        if (offset < 0 && sides & 2 == 0) {
          expect(outcome, SwipeOutcome.none);
        }
        if (sides & 4 == 0) {
          expect(outcome, isNot(SwipeOutcome.commitStart));
        }
        if (sides & 8 == 0) {
          expect(outcome, isNot(SwipeOutcome.commitEnd));
        }
      },
    );

    Glados<SwipeRelease>(any.swipeRelease, kansoExploreConfig()).test(
      'a finger coming back fast always closes the row',
      (release) {
        expect(
          _outcome(release, velocity: -1000),
          SwipeOutcome.none,
        );
      },
    );
  });
}
