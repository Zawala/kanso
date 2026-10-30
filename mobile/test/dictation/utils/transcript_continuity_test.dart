// Kanso — transcript_continuity_test.dart
// Created 2026-10-29.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/dictation/utils/transcript_continuity.dart';

void main() {
  group('TranscriptContinuity', () {
    test('carries on when the recognizer adds to the same phrase', () {
      expect(
        TranscriptContinuity.startsOver(
          TranscriptContinuity.words('buy milk'),
          TranscriptContinuity.words('buy milk and eggs'),
          gap: const Duration(seconds: 2),
        ),
        isFalse,
      );
    });

    test('starts a new phrase after a pause with unrelated words', () {
      expect(
        TranscriptContinuity.startsOver(
          TranscriptContinuity.words('buy milk'),
          TranscriptContinuity.words('call Ana'),
          gap: const Duration(seconds: 1),
        ),
        isTrue,
      );
    });

    test('treats a long correction as the same phrase', () {
      expect(
        TranscriptContinuity.startsOver(
          TranscriptContinuity.words('buy milk and eggs and bread'),
          TranscriptContinuity.words('buy milk and bacon and bread'),
          gap: const Duration(seconds: 1),
        ),
        isFalse,
      );
    });

    test('ignores case and punctuation when comparing words', () {
      expect(
        TranscriptContinuity.startsOver(
          TranscriptContinuity.words('Hello, there.'),
          TranscriptContinuity.words('hello there friend'),
          gap: const Duration(seconds: 1),
        ),
        isFalse,
      );
    });

    test('a reset lets the next report begin a phrase', () {
      var now = DateTime(2026, 10, 29, 12);
      final continuity = TranscriptContinuity(clock: () => now);
      expect(continuity.restarts('buy milk', 'buy milk and eggs'), isFalse);
      continuity.reset();
      now = now.add(const Duration(seconds: 2));
      expect(continuity.restarts('buy milk', 'call Ana'), isFalse);
    });
  });
}
