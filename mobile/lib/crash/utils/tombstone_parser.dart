// Kanso — tombstone_parser.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import '../models/data/crash_trace.dart';
import 'crash_grammar.dart';
import 'crash_scrubber.dart';
import 'protobuf_reader.dart';

/// A native crash from an Android tombstone (the protobuf of AOSP's
/// `tombstone.proto`, what `ApplicationExitInfo.getTraceInputStream` returns
/// for `REASON_CRASH_NATIVE`).
///
/// Read, by field number: `Tombstone.tid` (6), `signal_info` (10) with its
/// `number` (1) and `name` (2), `threads` (16, a map of thread id to
/// `Thread`), `Thread.id` (1), `current_backtrace` (4), and each
/// `BacktraceFrame`'s `rel_pc` (1), `function_name` (4), `function_offset`
/// (5), `file_name` (6) and `build_id` (8). Every other field, the abort
/// message (14), the logs (18), the registers, the memory dumps, the command
/// line, the open files among them, is skipped by its wire type and never
/// looked at.
abstract final class TombstoneParser {
  static const int _tombstoneTid = 6;
  static const int _tombstoneSignal = 10;
  static const int _tombstoneThreads = 16;
  static const int _threadId = 1;
  static const int _threadBacktrace = 4;
  static const int _frameRelPc = 1;
  static const int _frameFunctionName = 4;
  static const int _frameFunctionOffset = 5;
  static const int _frameFileName = 6;
  static const int _frameBuildId = 8;

  /// The most threads and the most backtrace frames looked at.
  static const int _maxThreads = 4096;
  static const int _maxLooked = 4096;

  static const Map<int, String> _signals = <int, String>{
    1: 'SIGHUP',
    2: 'SIGINT',
    3: 'SIGQUIT',
    4: 'SIGILL',
    5: 'SIGTRAP',
    6: 'SIGABRT',
    7: 'SIGBUS',
    8: 'SIGFPE',
    9: 'SIGKILL',
    10: 'SIGUSR1',
    11: 'SIGSEGV',
    12: 'SIGUSR2',
    13: 'SIGPIPE',
    14: 'SIGALRM',
    15: 'SIGTERM',
    31: 'SIGSYS',
  };

  /// The crashing thread of [bytes], or null if there is none. Never throws.
  static CrashTrace? parse(Uint8List bytes) {
    try {
      return _parse(bytes);
    } on Object {
      return null;
    }
  }

  static CrashTrace? _parse(Uint8List bytes) {
    final reader = ProtobufReader(bytes);
    int? tid;
    String? signalName;
    int? signalNumber;
    final threads = <(int, Uint8List)>[];
    while (!reader.atEnd) {
      final (field, wire) = reader.readTag();
      if (field == _tombstoneTid && wire == ProtobufReader.varintType) {
        tid = reader.readVarint();
      } else if (field == _tombstoneSignal &&
          wire == ProtobufReader.delimitedType) {
        final signal = reader.readBytes();
        final (name, number) = _signal(signal);
        signalName = name ?? signalName;
        signalNumber = number ?? signalNumber;
      } else if (field == _tombstoneThreads &&
          wire == ProtobufReader.delimitedType) {
        final entry = reader.readBytes();
        if (threads.length < _maxThreads) {
          threads.add(_mapEntry(entry));
        }
      } else {
        reader.skip(wire);
      }
    }
    if (tid == null) {
      return null;
    }
    Uint8List? crashed;
    for (final (key, value) in threads) {
      if ((_idOf(value) ?? key) == tid) {
        crashed = value;
        break;
      }
    }
    if (crashed == null) {
      return null;
    }
    final frames = _backtrace(crashed);
    if (frames.isEmpty) {
      return null;
    }
    final named = signalName == null
        ? null
        : CrashGrammar.type.hasMatch(signalName)
        ? signalName
        : null;
    return CrashTrace(
      type: CrashScrubber.type(named ?? _signals[signalNumber]),
      frames: frames,
    );
  }

  /// `Signal.name` (2) and `Signal.number` (1).
  static (String?, int?) _signal(Uint8List bytes) {
    final reader = ProtobufReader(bytes);
    String? name;
    int? number;
    while (!reader.atEnd) {
      final (field, wire) = reader.readTag();
      if (field == 1 && wire == ProtobufReader.varintType) {
        number = reader.readVarint();
      } else if (field == 2 && wire == ProtobufReader.delimitedType) {
        name = reader.readString();
      } else {
        reader.skip(wire);
      }
    }
    return (name, number);
  }

  /// A map entry: key (1) and value (2).
  static (int, Uint8List) _mapEntry(Uint8List bytes) {
    final reader = ProtobufReader(bytes);
    var key = -1;
    var value = Uint8List(0);
    while (!reader.atEnd) {
      final (field, wire) = reader.readTag();
      if (field == 1 && wire == ProtobufReader.varintType) {
        key = reader.readVarint();
      } else if (field == 2 && wire == ProtobufReader.delimitedType) {
        value = reader.readBytes();
      } else {
        reader.skip(wire);
      }
    }
    return (key, value);
  }

  static int? _idOf(Uint8List thread) {
    try {
      final reader = ProtobufReader(thread);
      while (!reader.atEnd) {
        final (field, wire) = reader.readTag();
        if (field == _threadId && wire == ProtobufReader.varintType) {
          return reader.readVarint();
        }
        reader.skip(wire);
      }
    } on Object {
      return null;
    }
    return null;
  }

  static List<String> _backtrace(Uint8List thread) {
    final frames = <String>[];
    final reader = ProtobufReader(thread);
    var looked = 0;
    while (!reader.atEnd && frames.length < CrashGrammar.maxFrames) {
      final (field, wire) = reader.readTag();
      if (field == _threadBacktrace && wire == ProtobufReader.delimitedType) {
        final frame = reader.readBytes();
        if (++looked > _maxLooked) {
          break;
        }
        final text = _frame(frame);
        if (text != null) {
          frames.add(text);
        }
      } else {
        reader.skip(wire);
      }
    }
    return frames;
  }

  static String? _frame(Uint8List bytes) {
    final reader = ProtobufReader(bytes);
    int? relPc;
    String? function;
    int? functionOffset;
    String? file;
    String? buildId;
    while (!reader.atEnd) {
      final (field, wire) = reader.readTag();
      final isNumber = wire == ProtobufReader.varintType;
      final isText = wire == ProtobufReader.delimitedType;
      if (field == _frameRelPc && isNumber) {
        relPc = reader.readVarint();
      } else if (field == _frameFunctionName && isText) {
        function = reader.readString();
      } else if (field == _frameFunctionOffset && isNumber) {
        functionOffset = reader.readVarint();
      } else if (field == _frameFileName && isText) {
        file = reader.readString();
      } else if (field == _frameBuildId && isText) {
        buildId = reader.readString();
      } else {
        reader.skip(wire);
      }
    }
    if (relPc == null) {
      return null;
    }
    return CrashScrubber.nativeFrame(
      binary: file == null ? null : CrashScrubber.baseName(file),
      offset: relPc,
      symbol: function,
      symbolOffset: functionOffset ?? 0,
      buildId: buildId,
    );
  }
}
