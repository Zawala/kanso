// Kanso — source_lines_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        Generator,
        GeneratorUtils,
        Glados2,
        IntAnys,
        ListAnys,
        any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/utils/source_lines.dart';

import '../../helpers/property.dart';

StyledRun _run(int start, int end, {int? color, int? weight, bool? italic}) {
  return StyledRun(
    range: Range16(start: start, end: end),
    color: color,
    weight: weight,
    italic: italic,
  );
}

String _slice(String text, int start, int end) => text.substring(start, end);

extension _Inputs on Any {
  /// Text of a few letters and line breaks, some of them `\r\n`.
  Generator<String> get lined => listWithLengthInRange<String>(
    0,
    30,
    choose(<String>['a', 'b', ' ', '\n', '\r', 'c']),
  ).map((chars) => chars.join());

  /// Runs of any size and place, overlapping and past the end included.
  Generator<List<StyledRun>> get anyRuns => listWithLengthInRange<StyledRun>(
    0,
    8,
    combine2(
      intInRange(0, 40),
      intInRange(0, 16),
      (int start, int length) => _run(start, start + length, color: start),
    ),
  );
}

void main() {
  group('sourceLines', () {
    test('has a line for a text with no break, and one for none', () {
      expect(sourceLines('', const <StyledRun>[]), hasLength(1));
      final lines = sourceLines('one', const <StyledRun>[]);
      expect(lines, hasLength(1));
      expect(lines.single.start, 0);
      expect(lines.single.end, 3);
    });

    test('cuts at each break, with an empty last line after a final one', () {
      final lines = sourceLines('ab\n\ncd\n', const <StyledRun>[]);
      expect(lines.map((l) => (l.start, l.end)), <(int, int)>[
        (0, 2),
        (3, 3),
        (4, 6),
        (7, 7),
      ]);
    });

    test('leaves the carriage return of a Windows break out of the line', () {
      final lines = sourceLines('ab\r\ncd', const <StyledRun>[]);
      expect(lines.map((l) => (l.start, l.end)), <(int, int)>[
        (0, 2),
        (4, 6),
      ]);
    });

    test('puts each run on its line, and cuts one that spans lines', () {
      const text = 'ab\ncd\nef';
      final lines = sourceLines(text, <StyledRun>[
        _run(1, 7, color: 1, weight: 700, italic: true),
      ]);
      expect(lines[0].runs.single.range, const Range16(start: 1, end: 2));
      expect(lines[1].runs.single.range, const Range16(start: 3, end: 5));
      expect(lines[2].runs.single.range, const Range16(start: 6, end: 7));
      // The cut keeps the style.
      for (final line in lines) {
        expect(line.runs.single.color, 1);
        expect(line.runs.single.weight, 700);
        expect(line.runs.single.italic, isTrue);
      }
    });

    test('takes runs in the order of their start', () {
      final lines = sourceLines('abcdef', <StyledRun>[
        _run(4, 6, color: 2),
        _run(0, 2, color: 1),
      ]);
      expect(lines.single.runs.map((run) => run.color), <int?>[1, 2]);
    });

    test('leaves out what an earlier run covers, and what is not there', () {
      final lines = sourceLines('abcdef', <StyledRun>[
        _run(0, 4, color: 1),
        _run(2, 3, color: 2),
        _run(3, 8, color: 3),
        _run(5, 5, color: 4),
        _run(9, 12, color: 5),
      ]);
      expect(
        lines.single.runs.map((run) => (run.range.start, run.range.end)),
        <(int, int)>[(0, 4), (4, 6)],
      );
      expect(lines.single.runs.last.color, 3);
    });

    Glados2<String, List<StyledRun>>(
      any.lined,
      any.anyRuns,
      kansoExploreConfig(),
    ).test('the lines are the text between the breaks', (text, runs) {
      final lines = sourceLines(text, runs);
      final parts = text.split('\n');
      expect(lines, hasLength(parts.length));
      for (var i = 0; i < parts.length; i++) {
        final expected = parts[i].endsWith('\r')
            ? parts[i].substring(0, parts[i].length - 1)
            : parts[i];
        expect(_slice(text, lines[i].start, lines[i].end), expected);
      }
    });

    Glados2<String, List<StyledRun>>(
      any.lined,
      any.anyRuns,
      kansoExploreConfig(),
    ).test('each run is inside its line, in order, and not empty', (
      text,
      runs,
    ) {
      for (final line in sourceLines(text, runs)) {
        var last = line.start;
        for (final run in line.runs) {
          expect(run.range.start, greaterThanOrEqualTo(last));
          expect(run.range.end, greaterThan(run.range.start));
          expect(run.range.end, lessThanOrEqualTo(line.end));
          last = run.range.end;
        }
      }
    });

    Glados2<String, List<StyledRun>>(
      any.lined,
      any.anyRuns,
      kansoExploreConfig(),
    ).test('the runs cover what the runs given cover of the text', (
      text,
      runs,
    ) {
      final wanted = List<bool>.filled(text.length, false);
      for (final run in runs) {
        for (var i = run.range.start; i < run.range.end; i++) {
          if (i < text.length) {
            wanted[i] = true;
          }
        }
      }
      final got = List<bool>.filled(text.length, false);
      final lines = sourceLines(text, runs);
      for (final line in lines) {
        for (final run in line.runs) {
          for (var i = run.range.start; i < run.range.end; i++) {
            got[i] = true;
          }
        }
      }
      // A break is not text of any line, and a carriage return before one
      // is left out with it.
      for (var i = 0; i < text.length; i++) {
        final inLine = lines.any((line) => i >= line.start && i < line.end);
        expect(got[i], wanted[i] && inLine, reason: 'at $i of "$text"');
      }
    });
  });
}
