// Kanso — anr_parser_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, any;
import 'package:kanso/crash/utils/anr_parser.dart';
import 'package:kanso/crash/utils/crash_grammar.dart';

import '../../helpers/crash_fixtures.dart';
import '../../helpers/crash_generators.dart';
import '../../helpers/property.dart';

const String _longFrame1 =
    'native libc.so +0x4e7a4 syscall+0x1c '
    '0123456789abcdef0123456789abcdef01234567';

void main() {
  group('AnrParser examples', () {
    test('reads the main thread only, in the order of the trace', () {
      final trace = AnrParser.parse(anrTrace())!;
      expect(trace.type, 'ANR');
      expect(trace.frames, <String>[
        _longFrame1,
        // The C++ name has colons: the symbol is left out.
        'native libart.so +0x123456 abcdef0123456789abcdef01',
        'native libflutter.so +0x1000',
        'native libapp.so +0x2000',
        'jvm java.lang.Object.wait:-2',
        'jvm com.foo.Bar.baz:12',
        'jvm android.os.Looper.loop:288',
        'jvm com.foo.Main.main:-1',
      ]);
    });

    test('nothing from the other threads or the header', () {
      final text = AnrParser.parse(
        anrTrace(noise: 'My secret diary'),
      )!.toString();
      expect(text, isNot(contains('evil')));
      expect(text, isNot(contains('diary')));
    });

    test('no main thread, no trace', () {
      expect(
        AnrParser.parse('"Signal Catcher" daemon prio=10 tid=5\n'),
        isNull,
      );
      expect(AnrParser.parse(''), isNull);
    });
  });

  group('AnrParser properties', () {
    Glados2<String, String>(
      any.noiseLine,
      any.noiseLine,
      kansoExploreConfig(),
    ).test('changing the free text leaves the report as it was', (a, b) {
      expect(
        AnrParser.parse(anrTrace(noise: a, libPath: '/x/$a')),
        AnrParser.parse(anrTrace(noise: b, libPath: '/y/$b')),
      );
    });

    Glados(any.noteText, kansoExploreConfig()).test(
      'a note in the free text or a path never reaches a frame',
      (secret) {
        final trace = AnrParser.parse(
          anrTrace(noise: secret, libPath: '/$secret/dir'),
        )!;
        expect(trace.toString(), isNot(contains(leakMarker)));
      },
    );

    Glados2<String, String>(
      any.noiseLine,
      any.noiseLine,
      kansoExploreConfig(),
    ).test('every frame is in the grammar, whatever the lines say', (a, b) {
      final text =
          '"main" prio=5 tid=1 Native\n'
          '  native: #00 pc 0000ff  $a ($b) (BuildId: abcdef01)\n'
          '  native: #01 pc 0000ff  $a\n'
          '  at $a.$b($a)\n'
          '  at $a($b)\n';
      final trace = AnrParser.parse(text);
      for (final frame in trace?.frames ?? <String>[]) {
        expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
      }
    });

    Glados(any.noise, kansoExploreConfig()).test('never throws on any text', (
      text,
    ) {
      expect(() => AnrParser.parse(text), returnsNormally);
      expect(() => AnrParser.parse('"main" prio=5\n$text'), returnsNormally);
    });
  });
}
