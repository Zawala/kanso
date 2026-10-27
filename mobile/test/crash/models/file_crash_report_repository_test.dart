// Kanso — file_crash_report_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/configs/crash_configs.dart';
import 'package:kanso/crash/models/data/crash_folder.dart';
import 'package:kanso/crash/models/data/crash_kind.dart';
import 'package:kanso/crash/models/data/crash_send_outcome.dart';
import 'package:kanso/crash/models/data/pending_crash_report.dart';
import 'package:kanso/crash/models/repos/impl/file_crash_report_repository.dart';
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/crash_report_assembler.dart';
import 'package:path/path.dart' as p;

import '../../helpers/crash_fixtures.dart';
import '../../helpers/crash_samples.dart';
import '../../helpers/tombstone_encoder.dart';

/// Lets a test talk to a real server on this machine: the test binding would
/// otherwise answer every request with a 400.
class _RealHttp extends HttpOverrides {}

HttpClient _realClient() =>
    HttpOverrides.runWithHttpOverrides(HttpClient.new, _RealHttp());

void main() {
  late Directory root;
  late CrashFolder folder;
  late HttpServer server;
  late List<(String, String, String?)> requests;
  var status = 204;

  Uri endpoint() => Uri.parse(
    'http://${server.address.host}:${server.port}/v1/crash-reports',
  );

  FileCrashReportRepository build({Uri? to}) => FileCrashReportRepository(
    folder: folder,
    context: FixtureCrashContextRepository(),
    endpoint: to ?? endpoint(),
    httpClient: _realClient,
  );

  Future<void> drop(String name, List<int> bytes) async {
    final raw = Directory(folder.rawPath)..createSync(recursive: true);
    await File(p.join(raw.path, name)).writeAsBytes(bytes);
  }

  Future<List<PendingCrashReport>> pending(
    FileCrashReportRepository repository,
  ) async => (await repository.pending()).getOrElse(
    () => throw StateError('no reports'),
  );

  setUp(() async {
    root = await Directory.systemTemp.createTemp('kanso_crash_test_');
    folder = CrashFolder(p.join(root.path, 'crash-reports'));
    status = 204;
    requests = <(String, String, String?)>[];
    server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    // ignore: cascade_invocations, the server is assigned above.
    server.listen((request) async {
      final body = await utf8.decoder.bind(request).join();
      requests.add((
        request.method,
        body,
        request.headers.contentType?.toString(),
      ));
      await (request.response..statusCode = status).close();
    });
  });

  tearDown(() async {
    await server.close(force: true);
    await root.delete(recursive: true);
  });

  group('Dart errors', () {
    test('keep the type and the frames, never the message', () async {
      final repository = build();
      final error = StateError('Could not save "My diary"');
      await repository.recordDartError(
        error,
        StackTrace.fromString(
          '#0      Saver.save (package:kanso/a.dart:12:3)\n'
          '#1      main.<anonymous closure> (file:///Users/me/My diary/m.dart:1:1)',
        ),
      );
      final reports = await pending(repository);
      expect(reports, hasLength(1));
      final report = reports.single.report;
      expect(report.kind, CrashKind.dart);
      expect(report.type, 'StateError');
      expect(report.frames, <String>[
        'dart Saver.save package:kanso/a.dart:12:3',
        'dart main.<closure> ?',
      ]);
      expect(report.app.patch, 2);
      expect(reports.single.body, isNot(contains('diary')));
      expect(CrashReportAssembler.isValid(report), isTrue);
    });

    test('are kept exactly once', () async {
      final repository = build();
      final trace = StackTrace.fromString('#0 a (package:kanso/a.dart:1:1)');
      for (var i = 0; i < 5; i++) {
        await repository.recordDartError(StateError('x$i'), trace);
      }
      expect(await pending(repository), hasLength(1));
    });

    test('a trace with no frame is not a report', () async {
      final repository = build();
      await repository.recordDartError(StateError('x'), StackTrace.empty);
      expect(await pending(repository), isEmpty);
    });

    test('the inbox keeps at most ten, the newest', () async {
      final repository = build();
      for (var i = 0; i < 14; i++) {
        await repository.recordDartError(
          StateError('x'),
          StackTrace.fromString('#0 f$i (package:kanso/a.dart:$i:1)'),
        );
      }
      final reports = await pending(repository);
      expect(reports, hasLength(CrashConfigs.maxPending));
      expect(reports.last.report.frames.single, contains('f13'));
      expect(reports.first.report.frames.single, contains('f4 '));
    });

    test('never throws, even when the folder can not be written', () async {
      final blocked = File(p.join(root.path, 'blocked'))..writeAsStringSync('');
      final repository = FileCrashReportRepository(
        folder: CrashFolder(p.join(blocked.path, 'x')),
        context: FixtureCrashContextRepository(),
        endpoint: endpoint(),
      );
      await repository.recordDartError(
        StateError('x'),
        StackTrace.fromString('#0 a (package:kanso/a.dart:1:1)'),
      );
    });
  });

  group('raw crashes from the native code', () {
    test('a tombstone becomes a report and the raw file is deleted', () async {
      await drop(
        '1792000000000.tombstone.pb',
        TombstoneEncoder.encode(
          tid: 5,
          threads: <int, List<TombstoneFrame>>{
            5: const <TombstoneFrame>[
              TombstoneFrame(relPc: 0x10, function: 'abort', file: 'libc.so'),
            ],
          },
          noise: const TombstoneNoise(abortMessage: 'My diary'),
        ),
      );
      final repository = build();
      final reports = await pending(repository);
      expect(reports.single.report.kind, CrashKind.androidNative);
      expect(reports.single.report.type, 'SIGSEGV');
      expect(reports.single.report.day, '2026-10-14');
      expect(reports.single.body, isNot(contains('diary')));
      expect(Directory(folder.rawPath).listSync(), isEmpty);
    });

    test('an ANR, an exception and a MetricKit payload too', () async {
      await drop('1792000000001.anr.txt', utf8.encode(anrTrace()));
      await drop(
        '1792000000002.jvm.json',
        utf8.encode(
          jsonEncode(<String, dynamic>{
            'type': 'java.lang.IllegalStateException',
            'frames': <Map<String, dynamic>>[
              <String, dynamic>{'cls': 'a.B', 'method': 'c', 'line': 3},
            ],
            'day': '2026-10-24',
          }),
        ),
      );
      await drop(
        'A1B2.metrickit.json',
        utf8.encode(
          metricKitPayload(
            crashedOutermostFirst: const <MetricKitFrame>[
              MetricKitFrame(
                binaryName: 'Runner',
                offset: 4096,
                binaryUuid: '0A7FC3D2-1111-2222-3333-444455556666',
              ),
            ],
            appVersion: '0.8.0',
          ),
        ),
      );
      final reports = await pending(build());
      expect(
        reports.map((r) => r.report.kind).toSet(),
        <CrashKind>{
          CrashKind.androidAnr,
          CrashKind.androidJvm,
          CrashKind.iosNative,
        },
      );
      final jvm = reports.firstWhere(
        (r) => r.report.kind == CrashKind.androidJvm,
      );
      expect(jvm.report.day, '2026-10-24');
      // An older build's crash is not given today's patch.
      final ios = reports.firstWhere(
        (r) => r.report.kind == CrashKind.iosNative,
      );
      expect(ios.report.app.version, '0.8.0');
      expect(ios.report.app.patch, isNull);
      expect(Directory(folder.rawPath).listSync(), isEmpty);
    });

    test('garbage is deleted without a report or a throw', () async {
      await drop('1792000000003.tombstone.pb', List<int>.filled(300, 0xff));
      await drop('1792000000004.anr.txt', utf8.encode('nothing here'));
      await drop('1792000000005.jvm.json', utf8.encode('{'));
      await drop('A.metrickit.json', utf8.encode('[]'));
      await drop('whatever.txt', utf8.encode('My diary'));
      expect(await pending(build()), isEmpty);
      expect(Directory(folder.rawPath).listSync(), isEmpty);
    });

    test('a file still being written is left alone', () async {
      await drop('1792000000006.jvm.json.tmp', utf8.encode('{'));
      await pending(build());
      expect(Directory(folder.rawPath).listSync(), hasLength(1));
    });

    test('the same crash twice is one report', () async {
      final bytes = TombstoneEncoder.encode(
        tid: 5,
        threads: <int, List<TombstoneFrame>>{
          5: const <TombstoneFrame>[TombstoneFrame(relPc: 0x10, file: 'a.so')],
        },
      );
      await drop('1792000000007.tombstone.pb', bytes);
      await drop('1792000000008.tombstone.pb', bytes);
      expect(await pending(build()), hasLength(1));
    });
  });

  group('pending reports', () {
    test(
      'a file that is not a report is deleted; extra keys never go out',
      () async {
        final repository = build();
        await repository.recordDartError(
          StateError('x'),
          StackTrace.fromString('#0 a (package:kanso/a.dart:1:1)'),
        );
        final dir = Directory(folder.pendingPath);
        final kept = dir.listSync().whereType<File>().single;
        File(p.join(dir.path, '1-1.json')).writeAsStringSync('{"v":1}');
        File(p.join(dir.path, '2-1.json')).writeAsStringSync('nope');
        File(p.join(dir.path, '3-1.json')).writeAsStringSync(
          jsonEncode(<String, dynamic>{
            ...jsonDecode(kept.readAsStringSync()) as Map<String, dynamic>,
            'extra': 'My diary',
          }),
        );
        final reports = await pending(repository);
        // The two broken files are gone; the one with an extra key is read as
        // the report it holds, and what is shown and sent has no extra key.
        expect(reports, hasLength(2));
        for (final report in reports) {
          expect(report.body, isNot(contains('diary')));
          expect(report.body, isNot(contains('extra')));
        }
        expect(dir.listSync().whereType<File>(), hasLength(2));
      },
    );

    test('a report with a frame outside the grammar is deleted', () async {
      final repository = build();
      final bad = sampleCrashReport(
        frames: <String>['dart a file:///My diary'],
      );
      Directory(folder.pendingPath).createSync(recursive: true);
      File(p.join(folder.pendingPath, '5-1.json')).writeAsStringSync(
        jsonEncode(bad.toJson()),
      );
      expect(await pending(repository), isEmpty);
    });

    test('discard deletes one, and only a real id', () async {
      final repository = build();
      await repository.recordDartError(
        StateError('x'),
        StackTrace.fromString('#0 a (package:kanso/a.dart:1:1)'),
      );
      final id = (await pending(repository)).single.id;
      expect((await repository.discard(id)).isRight(), isTrue);
      expect(await pending(repository), isEmpty);
      expect(
        () => repository.discard('../../library'),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('the id orders the reports by time', () async {
      final repository = build();
      var time = DateTime.utc(2026, 10, 26, 10);
      for (var i = 0; i < 3; i++) {
        await withClock(Clock.fixed(time), () async {
          await repository.recordDartError(
            StateError('x'),
            StackTrace.fromString('#0 f$i (package:kanso/a.dart:1:1)'),
          );
        });
        time = time.add(const Duration(minutes: 1));
      }
      final reports = await pending(repository);
      expect(
        reports.map((r) => r.report.frames.single).toList(),
        <String>[
          for (var i = 0; i < 3; i++) 'dart f$i package:kanso/a.dart:1:1',
        ],
      );
      expect(reports.first.report.day, '2026-10-26');
    });
  });

  group('sending', () {
    Future<String> keepOne(FileCrashReportRepository repository) async {
      await repository.recordDartError(
        StateError('x'),
        StackTrace.fromString('#0 a (package:kanso/a.dart:1:1)'),
      );
      return (await pending(repository)).single.id;
    }

    test('204: sent exactly as shown, and the report is deleted', () async {
      final repository = build();
      final id = await keepOne(repository);
      final shown = (await pending(repository)).single.body;
      expect(await repository.send(id), CrashSendOutcome.sent);
      expect(requests, hasLength(1));
      expect(requests.single.$1, 'POST');
      expect(requests.single.$2, shown);
      expect(requests.single.$3, 'application/json');
      expect(await pending(repository), isEmpty);
    });

    test('400 and 413: kept, and said to be rejected', () async {
      final repository = build();
      final id = await keepOne(repository);
      for (final code in <int>[400, 413]) {
        status = code;
        expect(await repository.send(id), CrashSendOutcome.rejected);
        expect(await pending(repository), hasLength(1));
      }
    });

    test('429: kept, to try later', () async {
      final repository = build();
      final id = await keepOne(repository);
      status = 429;
      expect(await repository.send(id), CrashSendOutcome.tryLater);
      expect(await pending(repository), hasLength(1));
    });

    test('any other answer or no server: kept, unreachable', () async {
      final repository = build();
      final id = await keepOne(repository);
      status = 500;
      expect(await repository.send(id), CrashSendOutcome.unreachable);
      final closed = build(
        to: Uri.parse('http://127.0.0.1:1/v1/crash-reports'),
      );
      expect(await closed.send(id), CrashSendOutcome.unreachable);
      expect(await pending(repository), hasLength(1));
    });

    test('sends nothing for a report that is not there', () async {
      final repository = build();
      expect(await repository.send('1-1'), CrashSendOutcome.rejected);
      expect(requests, isEmpty);
    });

    test('nothing is sent until send is called', () async {
      final repository = build();
      await keepOne(repository);
      await pending(repository);
      expect(requests, isEmpty);
    });

    test('the body is at most 32 KiB', () async {
      final repository = build();
      final frames = <String>[
        for (var i = 0; i < 400; i++)
          'native libflutter.so +0x$i sym${'a' * 240}+0x1',
      ];
      expect(frames.every(CrashGrammar.isFrame), isTrue);
      Directory(folder.rawPath).createSync(recursive: true);
      await drop(
        '1792000000009.tombstone.pb',
        TombstoneEncoder.encode(
          tid: 1,
          threads: <int, List<TombstoneFrame>>{
            1: <TombstoneFrame>[
              for (var i = 0; i < 400; i++)
                TombstoneFrame(
                  relPc: i,
                  function: 'sym${'a' * 240}',
                  file: 'libflutter.so',
                ),
            ],
          },
        ),
      );
      final body = (await pending(repository)).single.body;
      expect(body.length, lessThanOrEqualTo(CrashGrammar.maxBodyBytes));
    });
  });
}
