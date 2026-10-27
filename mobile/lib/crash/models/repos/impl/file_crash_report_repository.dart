// Kanso — file_crash_report_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:path/path.dart' as p;

import '../../../../core/models/data/application_error.dart';
import '../../../configs/crash_configs.dart';
import '../../../utils/anr_parser.dart';
import '../../../utils/crash_report_assembler.dart';
import '../../../utils/crash_report_text.dart';
import '../../../utils/crash_scrubber.dart';
import '../../../utils/dart_stack_parser.dart';
import '../../../utils/jvm_exception_parser.dart';
import '../../../utils/metrickit_parser.dart';
import '../../../utils/tombstone_parser.dart';
import '../../data/crash_folder.dart';
import '../../data/crash_kind.dart';
import '../../data/crash_report.dart';
import '../../data/crash_send_outcome.dart';
import '../../data/crash_trace.dart';
import '../../data/pending_crash_report.dart';
import '../abs/crash_context_repository.dart';
import '../abs/crash_report_repository.dart';

/// [CrashReportRepository] over the inbox folder: `raw/` (what the native
/// code dropped) and `pending/` (scrubbed reports, one JSON file each).
///
/// A raw file is parsed, turned into reports and deleted straight after,
/// whatever came of it. Everything that touches the files waits its turn, so
/// a burst of errors writes one report at a time.
class FileCrashReportRepository implements CrashReportRepository {
  FileCrashReportRepository({
    required this._folder,
    required this._context,
    required this._endpoint,
    HttpClient Function()? httpClient,
  }) : _httpClient = httpClient ?? HttpClient.new;

  final CrashFolder _folder;
  final CrashContextRepository _context;
  final Uri _endpoint;
  final HttpClient Function() _httpClient;

  Future<void> _queue = Future<void>.value();
  int _sequence = 0;

  /// Keys of what this run has already kept, so an error that fires every
  /// frame costs nothing after the first.
  final Set<String> _seen = <String>{};

  static final RegExp _idPattern = RegExp(r'^[0-9]{1,20}-[0-9]{1,6}$');

  @override
  Future<void> recordDartError(Object error, StackTrace stack) async {
    try {
      // Only the type's name and the trace's frames: `error` is never
      // turned into text.
      final trace = CrashTrace(
        type: CrashScrubber.type(error.runtimeType.toString()),
        frames: DartStackParser.parse(stack.toString()),
      );
      final key = _key(CrashKind.dart, trace.type, trace.frames);
      if (trace.frames.isEmpty || _seen.contains(key)) {
        return;
      }
      await _serialised(() => _store(CrashKind.dart, trace, clock.now()));
    } on Object {
      // Reporting a crash must never be a crash.
    }
  }

  @override
  Future<Either<ApplicationError, List<PendingCrashReport>>> pending() {
    return _serialised(() async {
      try {
        await _ingest();
        return Right<ApplicationError, List<PendingCrashReport>>(
          await _readPending(),
        );
      } on FileSystemException catch (error) {
        return Left<ApplicationError, List<PendingCrashReport>>(
          ApplicationError.storage(message: error.message),
        );
      }
    });
  }

  @override
  Future<Either<ApplicationError, Unit>> discard(String id) {
    return _serialised(() async {
      try {
        await _delete(_pendingFile(id));
        return const Right<ApplicationError, Unit>(unit);
      } on FileSystemException catch (error) {
        return Left<ApplicationError, Unit>(
          ApplicationError.storage(message: error.message),
        );
      }
    });
  }

  @override
  Future<CrashSendOutcome> send(String id) {
    return _serialised(() async {
      final report = await _readOne(_pendingFile(id));
      if (report == null) {
        return CrashSendOutcome.rejected;
      }
      final outcome = await _post(report.body);
      if (outcome == CrashSendOutcome.sent) {
        try {
          await _delete(_pendingFile(id));
        } on FileSystemException {
          // Sent; a file that stays is sent again at worst.
        }
      }
      return outcome;
    });
  }

  Future<CrashSendOutcome> _post(String body) async {
    final client = _httpClient()
      ..connectionTimeout = CrashConfigs.sendTimeout
      // No user agent, no cookie, no identifier: the body is all there is.
      ..userAgent = null;
    try {
      final bytes = utf8.encode(body);
      final request = await client
          .postUrl(_endpoint)
          .timeout(CrashConfigs.sendTimeout);
      request.headers.contentType = ContentType('application', 'json');
      request
        ..contentLength = bytes.length
        ..add(bytes);
      final response = await request.close().timeout(CrashConfigs.sendTimeout);
      await response.drain<void>();
      return switch (response.statusCode) {
        204 => CrashSendOutcome.sent,
        400 || 413 => CrashSendOutcome.rejected,
        429 => CrashSendOutcome.tryLater,
        _ => CrashSendOutcome.unreachable,
      };
    } on Object {
      return CrashSendOutcome.unreachable;
    } finally {
      client.close(force: true);
    }
  }

  Future<T> _serialised<T>(Future<T> Function() action) {
    final result = _queue.then((_) => action());
    _queue = result.then<void>((_) {}, onError: (Object _) {});
    return result;
  }

  String _key(CrashKind kind, String type, List<String> frames) =>
      '${kind.wire}\n$type\n${frames.join('\n')}';

  File _pendingFile(String id) {
    if (!_idPattern.hasMatch(id)) {
      throw ArgumentError.value(id, 'id', 'not a report id');
    }
    return File(p.join(_folder.pendingPath, '$id.json'));
  }

  Future<void> _delete(File file) async {
    if (file.existsSync()) {
      await file.delete();
    }
  }

  /// Keeps the report of [trace] unless the inbox has it already.
  Future<void> _store(CrashKind kind, CrashTrace trace, DateTime when) async {
    final report = CrashReportAssembler.assemble(
      kind: kind,
      trace: trace,
      context: await _context.read(),
      fallbackDay: CrashScrubber.dayOf(when),
    );
    if (report == null) {
      return;
    }
    final key = _key(report.kind, report.type, report.frames);
    _seen.add(key);
    final kept = await _readPending();
    if (kept.any(
      (r) => _key(r.report.kind, r.report.type, r.report.frames) == key,
    )) {
      return;
    }
    // The oldest make room.
    for (var i = 0; kept.length - i >= CrashConfigs.maxPending; i++) {
      await _delete(_pendingFile(kept[i].id));
    }
    await Directory(_folder.pendingPath).create(recursive: true);
    final id =
        '${clock.now().toUtc().millisecondsSinceEpoch}-'
        '${(_sequence++ % 1000000).toString().padLeft(2, '0')}';
    final file = _pendingFile(id);
    final temporary = File('${file.path}.tmp');
    await temporary.writeAsString(crashReportText(report), flush: true);
    await temporary.rename(file.path);
  }

  Future<List<PendingCrashReport>> _readPending() async {
    final directory = Directory(_folder.pendingPath);
    if (!directory.existsSync()) {
      return <PendingCrashReport>[];
    }
    final files = <File>[
      for (final entity in directory.listSync())
        if (entity is File && entity.path.endsWith('.json')) entity,
    ]..sort((a, b) => p.basename(a.path).compareTo(p.basename(b.path)));
    final reports = <PendingCrashReport>[];
    for (final file in files) {
      final report = await _readOne(file);
      if (report == null) {
        // Not a report of ours: it can't be sent, so it goes.
        await _delete(file);
      } else {
        reports.add(report);
      }
    }
    return reports;
  }

  /// The report in [file], or null if it is not a report inside the grammar.
  Future<PendingCrashReport?> _readOne(File file) async {
    try {
      if (!file.existsSync()) {
        return null;
      }
      final json = jsonDecode(await file.readAsString());
      if (json is! Map<String, dynamic>) {
        return null;
      }
      final report = CrashReport.fromJson(json);
      if (!CrashReportAssembler.isValid(report)) {
        return null;
      }
      return PendingCrashReport(
        id: p.basenameWithoutExtension(file.path),
        report: report,
        body: crashReportText(report),
      );
    } on Object {
      return null;
    }
  }

  /// Turns the native code's drops into reports and deletes them.
  Future<void> _ingest() async {
    final directory = Directory(_folder.rawPath);
    if (!directory.existsSync()) {
      return;
    }
    final files = <File>[
      for (final entity in directory.listSync())
        if (entity is File) entity,
    ]..sort((a, b) => p.basename(a.path).compareTo(p.basename(b.path)));
    for (final file in files) {
      // The native code writes `name.tmp` and renames it; a `.tmp` is still
      // being written.
      if (file.path.endsWith('.tmp')) {
        continue;
      }
      try {
        await _ingestOne(file);
      } on Object {
        // An unreadable drop is a lost report, not a failed launch.
      } finally {
        try {
          await _delete(file);
        } on Object {
          // Nothing to do about it.
        }
      }
    }
  }

  Future<void> _ingestOne(File file) async {
    final name = p.basename(file.path);
    final CrashKind kind;
    if (name.endsWith('.metrickit.json')) {
      kind = CrashKind.iosNative;
    } else if (name.endsWith('.tombstone.pb')) {
      kind = CrashKind.androidNative;
    } else if (name.endsWith('.anr.txt')) {
      kind = CrashKind.androidAnr;
    } else if (name.endsWith('.jvm.json')) {
      kind = CrashKind.androidJvm;
    } else {
      return;
    }
    final stat = file.statSync();
    if (stat.size > CrashConfigs.maxRawBytes) {
      return;
    }
    final stamp = RegExp(r'^([0-9]{13})\.').firstMatch(name);
    final when = stamp == null
        ? stat.modified
        : DateTime.fromMillisecondsSinceEpoch(
            int.parse(stamp.group(1)!),
            isUtc: true,
          );
    final traces = switch (kind) {
      CrashKind.iosNative => MetricKitParser.parse(
        utf8.decode(await file.readAsBytes(), allowMalformed: true),
      ),
      CrashKind.androidNative => <CrashTrace>[
        ?TombstoneParser.parse(await file.readAsBytes()),
      ],
      CrashKind.androidAnr => <CrashTrace>[
        ?AnrParser.parse(
          utf8.decode(await file.readAsBytes(), allowMalformed: true),
        ),
      ],
      CrashKind.androidJvm => <CrashTrace>[
        ?JvmExceptionParser.parse(
          utf8.decode(await file.readAsBytes(), allowMalformed: true),
        ),
      ],
      CrashKind.dart => const <CrashTrace>[],
    };
    for (final trace in traces) {
      await _store(kind, trace, when);
    }
  }
}
