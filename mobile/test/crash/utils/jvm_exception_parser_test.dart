// Kanso — jvm_exception_parser_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, Glados3, IntAnys, any;
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/jvm_exception_parser.dart';

import '../../helpers/crash_generators.dart';
import '../../helpers/property.dart';

String _json({
  Object? type = 'java.lang.IllegalStateException',
  List<Map<String, dynamic>>? frames,
  String noise = 'secret',
}) => jsonEncode(<String, dynamic>{
  'type': type,
  'message': 'Could not save $noise',
  'cause': <String, dynamic>{'message': noise},
  'thread': noise,
  'frames':
      frames ??
      <Map<String, dynamic>>[
        <String, dynamic>{
          'cls': 'app.kanso.kanso.Saver',
          'method': 'save',
          'line': 12,
          'file': '$noise.kt',
        },
        <String, dynamic>{
          'cls': r'app.kanso.kanso.Saver$Inner',
          'method': '<init>',
          'line': -1,
        },
      ],
  'day': '2026-10-26',
});

void main() {
  group('JvmExceptionParser examples', () {
    test('reads the type, the frames and the day', () {
      final trace = JvmExceptionParser.parse(_json())!;
      expect(trace.type, 'java.lang.IllegalStateException');
      expect(trace.frames, <String>[
        'jvm app.kanso.kanso.Saver.save:12',
        r'jvm app.kanso.kanso.Saver$Inner.<init>:-1',
      ]);
      expect(trace.day, '2026-10-26');
    });

    test('a frame with a bad class or method is dropped', () {
      final trace = JvmExceptionParser.parse(
        _json(
          frames: <Map<String, dynamic>>[
            <String, dynamic>{'cls': 'my note', 'method': 'a', 'line': 1},
            <String, dynamic>{'cls': 'a.B', 'method': 'two words', 'line': 1},
            <String, dynamic>{'cls': 'a.B', 'method': 'ok', 'line': 'x'},
            <String, dynamic>{'cls': 1, 'method': 'ok', 'line': 1},
          ],
        ),
      )!;
      expect(trace.frames, <String>['jvm a.B.ok:-1']);
    });

    test('a type that is not a name is unknown', () {
      expect(JvmExceptionParser.parse(_json(type: 'my note'))!.type, 'unknown');
      expect(JvmExceptionParser.parse(_json(type: 3))!.type, 'unknown');
    });

    test('no frame, no trace', () {
      expect(JvmExceptionParser.parse(_json(frames: [])), isNull);
      expect(JvmExceptionParser.parse('{'), isNull);
      expect(JvmExceptionParser.parse('[]'), isNull);
    });
  });

  group('JvmExceptionParser properties', () {
    Glados2<String, String>(
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test('changing the message, thread or file leaves the report', (a, b) {
      expect(
        JvmExceptionParser.parse(_json(noise: a)),
        JvmExceptionParser.parse(_json(noise: b)),
      );
    });

    Glados(any.noteText, kansoExploreConfig()).test(
      'a note anywhere, even as a class or method, never reaches a report',
      (secret) {
        final trace = JvmExceptionParser.parse(
          _json(
            type: secret,
            noise: secret,
            frames: <Map<String, dynamic>>[
              <String, dynamic>{'cls': secret, 'method': 'a', 'line': 1},
              <String, dynamic>{'cls': 'a.B', 'method': secret, 'line': 1},
              <String, dynamic>{'cls': 'a.B', 'method': 'ok', 'line': 2},
            ],
          ),
        )!;
        expect(trace.toString(), isNot(contains(leakMarker)));
        expect(trace.frames, <String>['jvm a.B.ok:2']);
      },
    );

    Glados3<String, String, int>(
      any.noise,
      any.noise,
      any.int,
      kansoExploreConfig(),
    ).test('every frame is in the grammar', (cls, method, line) {
      final trace = JvmExceptionParser.parse(
        _json(
          type: cls,
          frames: <Map<String, dynamic>>[
            <String, dynamic>{'cls': cls, 'method': method, 'line': line},
            <String, dynamic>{'cls': 'a.B', 'method': 'ok', 'line': line},
          ],
        ),
      );
      for (final frame in trace?.frames ?? <String>[]) {
        expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
      }
    });

    Glados(any.noise, kansoExploreConfig()).test('never throws on any text', (
      text,
    ) {
      expect(() => JvmExceptionParser.parse(text), returnsNormally);
    });
  });
}
