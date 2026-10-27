// Kanso — crash_report_assembler_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, any;
import 'package:kanso/crash/models/data/crash_app.dart';
import 'package:kanso/crash/models/data/crash_context.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/crash_report.dart';
import 'package:kanso/crash/models/data/crash_trace.dart';
import 'package:kanso/crash/utils/anr_parser.dart';
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/crash_report_assembler.dart';
import 'package:kanso/crash/utils/crash_report_text.dart';
import 'package:kanso/crash/utils/dart_stack_parser.dart';
import 'package:kanso/crash/utils/jvm_exception_parser.dart';
import 'package:kanso/crash/utils/metrickit_parser.dart';
import 'package:kanso/crash/utils/tombstone_parser.dart';

import '../../helpers/crash_fixtures.dart';
import '../../helpers/crash_generators.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/property.dart';
import '../../helpers/tombstone_encoder.dart';

const List<MetricKitFrame> _ios = <MetricKitFrame>[
  MetricKitFrame(
    binaryName: 'Runner',
    offset: 0x2a4c8,
    binaryUuid: '0A7FC3D2-1111-2222-3333-444455556666',
  ),
];

/// One raw crash of each source, with [a] in every place a message, a log, a
/// register, a thread name or a path may go, and [b] where a path leads to a
/// file. The report of each, as the JSON text that would be sent.
Map<String, String?> _reports(String a, String b) {
  String? report(CrashKind kind, CrashTrace? trace) => trace == null
      ? null
      : crashReportText(
          CrashReportAssembler.assemble(
            kind: kind,
            trace: trace,
            context: sampleCrashContext,
            fallbackDay: '2026-10-26',
          )!,
        );
  final oneLine = a.replaceAll('\n', ' ');
  final dartText =
      'Unhandled exception: $a\n'
      '#0      Foo.bar (package:kanso/x.dart:3:4)\n'
      '#1      baz (file://$b/n.dart:1:1)\n'
      '<asynchronous suspension>';
  final dartFrames = DartStackParser.parse(dartText);
  return <String, String?>{
    'dart': report(
      CrashKind.dart,
      CrashTrace(type: 'StateError', frames: dartFrames),
    ),
    'ios': report(
      CrashKind.iosNative,
      MetricKitParser.parse(
        metricKitPayload(crashedOutermostFirst: _ios, noise: a),
      ).single,
    ),
    'tombstone': report(
      CrashKind.androidNative,
      TombstoneParser.parse(
        TombstoneEncoder.encode(
          tid: 2,
          threads: <int, List<TombstoneFrame>>{
            2: const <TombstoneFrame>[
              TombstoneFrame(relPc: 64, function: 'abort', file: 'libc.so'),
            ],
          },
          noise: TombstoneNoise(
            abortMessage: a,
            logLine: a,
            commandLine: a,
            threadName: a,
            registerName: a,
            pathPrefix: '/$b',
          ),
        ),
      ),
    ),
    'anr': report(
      CrashKind.androidAnr,
      AnrParser.parse(anrTrace(noise: oneLine, libPath: '/$b')),
    ),
    'jvm': report(
      CrashKind.androidJvm,
      JvmExceptionParser.parse(
        jsonEncode(<String, dynamic>{
          'type': 'java.lang.IllegalStateException',
          'message': a,
          'frames': <Map<String, dynamic>>[
            <String, dynamic>{
              'cls': 'a.B',
              'method': 'c',
              'line': 3,
              'file': a,
            },
          ],
          'day': '2026-10-26',
        }),
      ),
    ),
  };
}

const String _longFrame1 =
    'dart _NoteEditorState._save '
    'package:kanso/note/views/widgets/note_editor.dart:120:7';

void main() {
  group('the report as it is sent', () {
    Glados2<String, String>(
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test(
      'non-interference: changing any message, log, register, path prefix or '
      'thread name leaves the JSON byte-identical, for every source',
      (a, b) {
        final base = _reports('first message', 'first/path');
        final changed = _reports(a, b.replaceAll('\n', '_'));
        expect(changed.keys, base.keys);
        for (final source in base.keys) {
          expect(base[source], isNotNull, reason: source);
          expect(
            utf8.encode(changed[source]!),
            utf8.encode(base[source]!),
            reason: source,
          );
        }
      },
    );

    Glados2<String, String>(
      any.noteText,
      any.noteText,
      kansoExploreConfig(),
    ).test(
      'no leak: a note placed in any of those fields, or as a file path or '
      'URI, is nowhere in the JSON of any source',
      (secret, path) {
        final reports = _reports(secret, path.replaceAll('\n', '_'));
        for (final entry in reports.entries) {
          final text = entry.value!;
          expect(text, isNot(contains(secret)), reason: entry.key);
          expect(text, isNot(contains(path)), reason: entry.key);
          expect(text, isNot(contains(leakMarker)), reason: entry.key);
        }
      },
    );

    Glados2<String, String>(
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test('the grammar: every report has only the documented keys and '
        'every frame matches', (a, b) {
      for (final text in _reports(a, b.replaceAll('\n', '_')).values) {
        final json = jsonDecode(text!) as Map<String, dynamic>;
        expect(json.keys.toList(), <String>[
          'v',
          'kind',
          'type',
          'app',
          'os',
          'device',
          'day',
          'frames',
        ]);
        expect((json['app'] as Map<String, dynamic>).keys.toList(), <String>[
          'version',
          'build',
          'patch',
        ]);
        expect((json['os'] as Map<String, dynamic>).keys.toList(), <String>[
          'name',
          'version',
        ]);
        final frames = (json['frames'] as List<dynamic>).cast<String>();
        expect(frames, isNotEmpty);
        expect(frames.length, lessThanOrEqualTo(CrashGrammar.maxFrames));
        for (final frame in frames) {
          expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
        }
        expect(text.length, lessThanOrEqualTo(CrashGrammar.maxBodyBytes));
        expect(
          CrashReportAssembler.isValid(CrashReport.fromJson(json)),
          isTrue,
        );
        expect(CrashGrammar.type.hasMatch(json['type'] as String), isTrue);
        expect(CrashGrammar.day.hasMatch(json['day'] as String), isTrue);
      }
    });

    Glados(any.noteText, kansoExploreConfig()).test(
      'no leak through the app, the OS or the device of a context either',
      (secret) {
        final report = CrashReportAssembler.assemble(
          kind: CrashKind.dart,
          trace: CrashTrace(
            type: secret,
            frames: <String>['dart a ?', 'dart $secret ?'],
            day: secret,
            appVersion: secret,
            appBuild: secret,
            osVersion: secret,
            device: secret,
          ),
          context: CrashContext(
            app: CrashApp(version: secret, build: secret, patch: 99999999),
            os: sampleCrashContext.os.copyWith(version: secret),
            device: secret,
          ),
          fallbackDay: secret,
        )!;
        final text = crashReportText(report);
        expect(text, isNot(contains(leakMarker)));
        expect(report.frames, <String>['dart a ?']);
        expect(report.type, 'unknown');
        expect(report.device, 'unknown');
        expect(report.os.version, 'unknown');
        expect(report.app.patch, isNull);
        expect(CrashReportAssembler.isValid(report), isTrue);
      },
    );
  });

  group('CrashReportAssembler examples', () {
    test('the example of docs/CRASH-REPORTS.md', () {
      final report = CrashReportAssembler.assemble(
        kind: CrashKind.dart,
        trace: CrashTrace(
          type: 'StateError',
          frames: sampleCrashReport().frames,
        ),
        context: sampleCrashContext,
        fallbackDay: '2026-10-26',
      )!;
      expect(report.toJson(), <String, dynamic>{
        'v': 1,
        'kind': 'dart',
        'type': 'StateError',
        'app': <String, dynamic>{'version': '0.9.0', 'build': '57', 'patch': 2},
        'os': <String, dynamic>{'name': 'ios', 'version': '18.1'},
        'device': 'iPhone17,1',
        'day': '2026-10-26',
        'frames': <String>[
          _longFrame1,
          'dart <closure> dart:async/zone.dart:1525:47',
        ],
      });
      expect(CrashReportAssembler.isValid(report), isTrue);
    });

    test('no frames, no report', () {
      expect(
        CrashReportAssembler.assemble(
          kind: CrashKind.dart,
          trace: const CrashTrace(type: 'X', frames: <String>['nope']),
          context: sampleCrashContext,
          fallbackDay: '2026-10-26',
        ),
        isNull,
      );
    });

    test('over 128 frames the outermost go; over 32 KiB so do more', () {
      final long = <String>[
        for (var i = 0; i < 300; i++)
          'native libflutter.so +0x$i ${'S' * 240}+0x1',
      ];
      final report = CrashReportAssembler.assemble(
        kind: CrashKind.androidNative,
        trace: CrashTrace(type: 'SIGSEGV', frames: long),
        context: sampleCrashContext,
        fallbackDay: '2026-10-26',
      )!;
      expect(report.frames.first, long.first);
      expect(report.frames.length, lessThan(128));
      expect(
        crashReportText(report).length,
        lessThanOrEqualTo(CrashGrammar.maxBodyBytes),
      );
    });
  });
}
