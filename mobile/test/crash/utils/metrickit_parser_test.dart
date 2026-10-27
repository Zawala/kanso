// Kanso — metrickit_parser_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, Glados3, IntAnys, any;
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/metrickit_parser.dart';

import '../../helpers/crash_fixtures.dart';
import '../../helpers/crash_generators.dart';
import '../../helpers/property.dart';

const List<MetricKitFrame> _stack = <MetricKitFrame>[
  MetricKitFrame(
    binaryName: 'dyld',
    offset: 0x1000,
    binaryUuid: 'AAAAAAAA-0000-0000-0000-000000000001',
  ),
  MetricKitFrame(
    binaryName: 'Runner',
    offset: 0x2a4c8,
    binaryUuid: '0A7FC3D2-1111-2222-3333-444455556666',
  ),
  MetricKitFrame(
    binaryName: 'Flutter',
    offset: 0xabcdef,
    binaryUuid: 'BBBBBBBB-0000-0000-0000-000000000002',
  ),
];

void main() {
  group('MetricKitParser examples', () {
    test('reads the crashed thread, innermost frame first', () {
      final traces = MetricKitParser.parse(
        metricKitPayload(crashedOutermostFirst: _stack),
      );
      expect(traces, hasLength(1));
      final trace = traces.single;
      expect(trace.frames, <String>[
        'native Flutter +0xabcdef BBBBBBBB-0000-0000-0000-000000000002',
        'native Runner +0x2a4c8 0A7FC3D2-1111-2222-3333-444455556666',
        'native dyld +0x1000 AAAAAAAA-0000-0000-0000-000000000001',
      ]);
      expect(trace.type, 'EXC_BAD_ACCESS');
      expect(trace.day, '2026-10-26');
      expect(trace.appVersion, '0.9.0');
      expect(trace.appBuild, '57');
      expect(trace.osVersion, '18.1');
      expect(trace.device, 'iPhone17,1');
    });

    test('an abort says its signal, not EXC_CRASH', () {
      final trace = MetricKitParser.parse(
        metricKitPayload(
          crashedOutermostFirst: _stack,
          exceptionType: 10,
          signal: 6,
        ),
      ).single;
      expect(trace.type, 'SIGABRT');
    });

    test('numbers may be strings, and an unknown code is unknown', () {
      expect(
        MetricKitParser.parse(
          metricKitPayload(
            crashedOutermostFirst: _stack,
            exceptionType: '2',
            signal: '4',
          ),
        ).single.type,
        'EXC_BAD_INSTRUCTION',
      );
      expect(
        MetricKitParser.parse(
          metricKitPayload(
            crashedOutermostFirst: _stack,
            exceptionType: 99,
            signal: 99,
          ),
        ).single.type,
        'unknown',
      );
    });

    test('a single MXCrashDiagnostic is read like a payload', () {
      final payload =
          jsonDecode(metricKitPayload(crashedOutermostFirst: _stack))
              as Map<String, dynamic>;
      final diagnostic =
          (payload['crashDiagnostics'] as List<dynamic>).single
              as Map<String, dynamic>;
      final traces = MetricKitParser.parse(jsonEncode(diagnostic));
      expect(traces, hasLength(1));
      expect(traces.single.frames, hasLength(3));
    });

    test('a binary name that is not a base name is ?', () {
      final trace = MetricKitParser.parse(
        metricKitPayload(
          crashedOutermostFirst: _stack,
          binaryName: (frame) =>
              '/private/var/Notes for Mum/${frame.binaryName}',
        ),
      ).single;
      expect(
        trace.frames,
        everyElement(startsWith('native ? +0x')),
      );
    });

    test('a payload with no crash diagnostics, and bad JSON, give nothing', () {
      expect(MetricKitParser.parse('{"hangDiagnostics":[]}'), isEmpty);
      expect(MetricKitParser.parse('[1,2'), isEmpty);
      expect(MetricKitParser.parse(''), isEmpty);
    });
  });

  group('MetricKitParser properties', () {
    Glados2<String, String>(
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test('changing any message leaves the report as it was', (a, b) {
      final one = MetricKitParser.parse(
        metricKitPayload(crashedOutermostFirst: _stack, noise: a),
      );
      final other = MetricKitParser.parse(
        metricKitPayload(crashedOutermostFirst: _stack, noise: b),
      );
      expect(
        jsonEncode(one.map((t) => t.toString()).toList()),
        jsonEncode(other.map((t) => t.toString()).toList()),
      );
    });

    Glados(any.noteText, kansoExploreConfig()).test(
      'a note in any text field never reaches a frame or the type',
      (secret) {
        final traces = MetricKitParser.parse(
          metricKitPayload(
            crashedOutermostFirst: _stack,
            noise: secret,
            binaryName: (frame) => '$secret/${frame.binaryName}',
          ),
        );
        expect(traces.toString(), isNot(contains(leakMarker)));
      },
    );

    Glados3<String, String, int>(
      any.noise,
      any.noise,
      any.positiveInt,
      kansoExploreConfig(),
    ).test('every frame is in the grammar, whatever the names say', (
      name,
      uuid,
      offset,
    ) {
      final traces = MetricKitParser.parse(
        metricKitPayload(
          crashedOutermostFirst: <MetricKitFrame>[
            MetricKitFrame(binaryName: name, offset: offset, binaryUuid: uuid),
          ],
        ),
      );
      for (final trace in traces) {
        expect(trace.frames, isNotEmpty);
        for (final frame in trace.frames) {
          expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
        }
      }
    });

    Glados(any.noise, kansoExploreConfig()).test('never throws on any text', (
      text,
    ) {
      expect(() => MetricKitParser.parse(text), returnsNormally);
      expect(
        () => MetricKitParser.parse('{"crashDiagnostics":$text}'),
        returnsNormally,
      );
    });
  });
}
