// Kanso — tombstone_parser_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, Glados3, IntAnys, any;
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/protobuf_reader.dart';
import 'package:kanso/crash/utils/tombstone_parser.dart';

import '../../helpers/crash_generators.dart';
import '../../helpers/property.dart';
import '../../helpers/tombstone_encoder.dart';

/// What a report says of [frame], worked out from the page and not from the
/// scrubber: the oracle of the round trip.
String _expected(TombstoneFrame frame) {
  final file = frame.file;
  final base = file == null
      ? '?'
      : file.substring(file.lastIndexOf(RegExp('[/!]')) + 1);
  final buffer = StringBuffer(
    'native $base +0x${frame.relPc.toRadixString(16)}',
  );
  if (frame.function != null) {
    buffer.write(
      ' ${frame.function}+0x${frame.functionOffset.toRadixString(16)}',
    );
  }
  if (frame.buildId != null) {
    buffer.write(' ${frame.buildId}');
  }
  return buffer.toString();
}

const List<TombstoneFrame> _crashed = <TombstoneFrame>[
  TombstoneFrame(
    relPc: 0x4e7a4,
    function: 'abort',
    functionOffset: 164,
    file: 'libc.so',
    buildId: '0123456789abcdef0123456789abcdef01234567',
  ),
  TombstoneFrame(
    relPc: 0x1f2e30,
    function: '_ZN7flutter5Shell6CreateE',
    functionOffset: 52,
    file: 'base.apk!libflutter.so',
  ),
  TombstoneFrame(relPc: 0x9000, file: 'libapp.so'),
];

const String _longFrame1 =
    'native libc.so +0x4e7a4 abort+0xa4 '
    '0123456789abcdef0123456789abcdef01234567';

void main() {
  group('TombstoneParser examples', () {
    test('reads the crashing thread, not the others', () {
      final bytes = TombstoneEncoder.encode(
        tid: 4243,
        threads: <int, List<TombstoneFrame>>{
          4242: const <TombstoneFrame>[
            TombstoneFrame(relPc: 0xdead, file: 'libother.so'),
          ],
          4243: _crashed,
        },
        noise: const TombstoneNoise(
          abortMessage: 'Could not save "My diary"',
          logLine: 'user typed: dear diary',
          commandLine: 'app.kanso.kanso --note=secret',
          threadName: 'Dear diary',
          registerName: 'x0 dear diary',
          selinuxLabel: 'u:r:untrusted_app:s0',
          openFilePath: '/data/notes/My diary.md',
        ),
      );
      final trace = TombstoneParser.parse(bytes)!;
      expect(trace.type, 'SIGSEGV');
      expect(trace.frames, <String>[
        _longFrame1,
        'native libflutter.so +0x1f2e30 _ZN7flutter5Shell6CreateE+0x34',
        'native libapp.so +0x9000',
      ]);
      expect(trace.toString(), isNot(contains('diary')));
      expect(trace.toString(), isNot(contains('secret')));
    });

    test('the signal falls back to its number, then to unknown', () {
      Uint8List make(String name, int number) => TombstoneEncoder.encode(
        tid: 1,
        threads: <int, List<TombstoneFrame>>{1: _crashed},
        signalName: name,
        signalNumber: number,
      );
      expect(TombstoneParser.parse(make('', 6))!.type, 'SIGABRT');
      expect(TombstoneParser.parse(make('my note', 11))!.type, 'SIGSEGV');
      expect(TombstoneParser.parse(make('', 77))!.type, 'unknown');
    });

    test('no thread with the tid, no trace', () {
      final bytes = TombstoneEncoder.encode(
        tid: 9,
        threads: <int, List<TombstoneFrame>>{1: _crashed},
      );
      expect(TombstoneParser.parse(bytes), isNull);
    });

    test('empty, truncated and garbage input give nothing', () {
      expect(TombstoneParser.parse(Uint8List(0)), isNull);
      final bytes = TombstoneEncoder.encode(
        tid: 1,
        threads: <int, List<TombstoneFrame>>{1: _crashed},
      );
      expect(
        TombstoneParser.parse(
          Uint8List.sublistView(bytes, 0, bytes.length - 3),
        ),
        isNull,
      );
      expect(
        TombstoneParser.parse(Uint8List.fromList(List<int>.filled(50, 0xff))),
        isNull,
      );
    });
  });

  group('ProtobufReader', () {
    test('reads varints, including a full 64-bit one', () {
      final reader = ProtobufReader(
        Uint8List.fromList(<int>[
          0x96, 0x01, // 150
          0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, // -1
        ]),
      );
      expect(reader.readVarint(), 150);
      expect(reader.readVarint(), -1);
      expect(reader.atEnd, isTrue);
    });

    test('refuses a varint that never ends and a length past the end', () {
      expect(
        () => ProtobufReader(
          Uint8List.fromList(List<int>.filled(12, 0x80)),
        ).readVarint(),
        throwsFormatException,
      );
      expect(
        () => ProtobufReader(Uint8List.fromList(<int>[5, 1])).readBytes(),
        throwsFormatException,
      );
    });

    test('skips every wire type it knows and refuses groups', () {
      final reader =
          ProtobufReader(
              Uint8List.fromList(<int>[
                0x01, // varint
                1, 2, 3, 4, 5, 6, 7, 8, // fixed64
                2, 0xaa, 0xbb, // delimited
                1, 2, 3, 4, // fixed32
              ]),
            )
            ..skip(0)
            ..skip(1)
            ..skip(2)
            ..skip(5);
      expect(reader.atEnd, isTrue);
      expect(() => reader.skip(3), throwsFormatException);
    });
  });

  group('TombstoneParser properties', () {
    Glados3<String, String, String>(
      any.noise,
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test('changing any text the format has leaves the report', (a, b, c) {
      TombstoneNoise noise(String x, String y) => TombstoneNoise(
        abortMessage: x,
        logLine: y,
        commandLine: x + y,
        threadName: y,
        registerName: x,
        pathPrefix: '/${x.replaceAll('\u0000', '')}/$y',
        selinuxLabel: y,
        openFilePath: x,
      );
      Uint8List make(TombstoneNoise n) => TombstoneEncoder.encode(
        tid: 7,
        threads: <int, List<TombstoneFrame>>{1: _crashed, 7: _crashed},
        noise: n,
      );
      final one = TombstoneParser.parse(make(noise(a, b)));
      final other = TombstoneParser.parse(make(noise(b, c)));
      expect(one, isNotNull);
      expect(one, other);
    });

    Glados(any.noteText, kansoExploreConfig()).test(
      'a note in any field, or as a file name, function or build id, never '
      'reaches a report',
      (secret) {
        final bytes = TombstoneEncoder.encode(
          tid: 7,
          threads: <int, List<TombstoneFrame>>{
            7: <TombstoneFrame>[
              TombstoneFrame(
                relPc: 16,
                function: secret,
                file: secret,
                buildId: secret,
              ),
              const TombstoneFrame(relPc: 32, file: 'libc.so'),
            ],
          },
          signalName: secret,
          noise: TombstoneNoise(
            abortMessage: secret,
            logLine: secret,
            commandLine: secret,
            threadName: secret,
            registerName: secret,
            pathPrefix: '/$secret',
            selinuxLabel: secret,
            openFilePath: secret,
          ),
        );
        final trace = TombstoneParser.parse(bytes)!;
        expect(trace.toString(), isNot(contains(leakMarker)));
        expect(trace.type, 'SIGSEGV');
        expect(trace.frames, <String>[
          'native ? +0x10',
          'native libc.so +0x20',
        ]);
      },
    );

    Glados2<List<TombstoneFrame>, int>(
      any.validBacktrace,
      any.intInRange(1, 100000),
      kansoExploreConfig(),
    ).test('round trip: what is encoded is what is decoded', (frames, tid) {
      final bytes = TombstoneEncoder.encode(
        tid: tid,
        threads: <int, List<TombstoneFrame>>{
          tid + 1: const <TombstoneFrame>[TombstoneFrame(relPc: 1)],
          tid: frames,
        },
      );
      final trace = TombstoneParser.parse(bytes)!;
      expect(trace.frames, frames.map(_expected).toList());
    });

    Glados(any.wildTombstoneFrame, kansoExploreConfig()).test(
      'every frame is in the grammar, whatever the names say',
      (frame) {
        final trace = TombstoneParser.parse(
          TombstoneEncoder.encode(
            tid: 1,
            threads: <int, List<TombstoneFrame>>{
              1: <TombstoneFrame>[frame],
            },
          ),
        );
        for (final text in trace?.frames ?? <String>[]) {
          expect(CrashGrammar.isFrame(text), isTrue, reason: text);
        }
      },
    );

    Glados(any.randomBytes, kansoExploreConfig()).test(
      'fuzz: random bytes never throw and never make a bad frame',
      (bytes) {
        expect(() => TombstoneParser.parse(bytes), returnsNormally);
        for (final frame
            in TombstoneParser.parse(bytes)?.frames ?? <String>[]) {
          expect(CrashGrammar.isFrame(frame), isTrue);
        }
      },
    );

    Glados2<List<TombstoneFrame>, int>(
      any.validBacktrace,
      any.mutationSeed,
      kansoExploreConfig(),
    ).test('fuzz: a tombstone with bytes changed or cut never throws', (
      frames,
      seed,
    ) {
      final bytes = TombstoneEncoder.encode(
        tid: 3,
        threads: <int, List<TombstoneFrame>>{3: frames},
      );
      final broken = mutate(bytes, seed);
      expect(() => TombstoneParser.parse(broken), returnsNormally);
      for (final frame in TombstoneParser.parse(broken)?.frames ?? <String>[]) {
        expect(CrashGrammar.isFrame(frame), isTrue);
      }
    });
  });
}
