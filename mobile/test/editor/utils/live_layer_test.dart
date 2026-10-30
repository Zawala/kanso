// Kanso — live_layer_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/editor/utils/live_layer.dart';

const LiveLayer _layer = LiveLayer(
  conceal: <TextRange>[
    TextRange(start: 0, end: 2),
    TextRange(start: 9, end: 11),
    TextRange(start: 14, end: 16),
  ],
  runs: <(TextRange, LiveKind)>[
    (TextRange(start: 2, end: 6), LiveKind.heading),
    (TextRange(start: 11, end: 14), LiveKind.strong),
  ],
  lines: <(TextRange, double)>[(TextRange(start: 0, end: 6), 1.6)],
);

void main() {
  group('LiveLayer', () {
    test('is read from a batch', () {
      final layer = LiveLayer.fromBatch(
        const LiveBatch(
          version: 3,
          conceal: <Range16>[Range16(start: 0, end: 2)],
          runs: <LiveRun>[
            LiveRun(range: Range16(start: 2, end: 6), kind: LiveKind.heading),
          ],
          lines: <LiveLine>[
            LiveLine(range: Range16(start: 0, end: 6), level: 1, scale: 1.6),
          ],
        ),
      );
      expect(layer.conceal, const <TextRange>[TextRange(start: 0, end: 2)]);
      expect(layer.runs.single.$2, LiveKind.heading);
      expect(layer.scaleAt(3), 1.6);
      expect(layer.scaleAt(7), 1);
    });

    test('shows the marks again on the revealed lines only', () {
      // "# Head\nsome **bold**": the caret is on the second line.
      const text = '# Head\nsome **bold**';
      final revealed = revealedLines(
        text,
        const TextSelection.collapsed(offset: 12),
      );
      expect(revealed, const TextRange(start: 7, end: 20));
      const all = LiveLayer(
        conceal: <TextRange>[
          TextRange(start: 0, end: 2),
          TextRange(start: 12, end: 14),
          TextRange(start: 18, end: 20),
        ],
      );
      final layer = all.revealing(revealed);
      expect(layer.conceal, const <TextRange>[TextRange(start: 0, end: 2)]);
    });

    test('a caret at the end of a line reveals that line', () {
      const text = 'one\ntwo';
      expect(
        revealedLines(text, const TextSelection.collapsed(offset: 3)),
        const TextRange(start: 0, end: 3),
      );
      expect(
        revealedLines(text, const TextSelection.collapsed(offset: 4)),
        const TextRange(start: 4, end: 7),
      );
      const across = TextSelection(baseOffset: 1, extentOffset: 5);
      expect(revealedLines(text, across), const TextRange(start: 0, end: 7));
    });

    test('moves along an edit; a heading grows with typing at its end', () {
      final layer = _layer.shifted(start: 6, end: 6, inserted: 2);
      expect(layer.lines.single.$1, const TextRange(start: 0, end: 8));
      expect(layer.conceal, const <TextRange>[
        TextRange(start: 0, end: 2),
        TextRange(start: 11, end: 13),
        TextRange(start: 16, end: 18),
      ]);
      expect(layer.runs[1].$1, const TextRange(start: 13, end: 16));
    });

    test('a mark an edit touches is shown', () {
      final layer = _layer.shifted(start: 10, end: 11, inserted: 0);
      expect(layer.conceal, const <TextRange>[
        TextRange(start: 0, end: 2),
        TextRange(start: 13, end: 15),
      ]);
    });

    test('moves with the window, cut to it', () {
      final layer = _layer.moved(-4, 10);
      expect(layer.conceal, const <TextRange>[TextRange(start: 5, end: 7)]);
      expect(layer.lines.single.$1, const TextRange(start: 0, end: 2));
    });
  });
}
