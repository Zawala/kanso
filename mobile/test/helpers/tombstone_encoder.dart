// Kanso — tombstone_encoder.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:typed_data';

/// A frame of a tombstone's backtrace, as a test writes it.
class TombstoneFrame {
  const TombstoneFrame({
    required this.relPc,
    this.function,
    this.functionOffset = 0,
    this.file,
    this.buildId,
  });

  final int relPc;
  final String? function;
  final int functionOffset;
  final String? file;
  final String? buildId;
}

/// What a tombstone holds besides the frames that a report may keep: text
/// that must never reach a report, in every field the format has for it.
class TombstoneNoise {
  const TombstoneNoise({
    this.abortMessage = '',
    this.logLine = '',
    this.commandLine = '',
    this.threadName = '',
    this.registerName = '',
    this.pathPrefix = '/data/app/~~x==/app.kanso.kanso-y==/lib/arm64',
    this.selinuxLabel = '',
    this.openFilePath = '',
  });

  final String abortMessage;
  final String logLine;
  final String commandLine;
  final String threadName;
  final String registerName;

  /// The directories in front of every frame's file name.
  final String pathPrefix;
  final String selinuxLabel;
  final String openFilePath;
}

/// Writes a tombstone in the protobuf wire format of AOSP's
/// `tombstone.proto`, for tests. It is a second, independent implementation
/// of the format, so the reader is checked against something it didn't make.
class TombstoneEncoder {
  TombstoneEncoder._();

  /// A tombstone of [threads] (thread id to its backtrace), crashed in [tid].
  static Uint8List encode({
    required int tid,
    required Map<int, List<TombstoneFrame>> threads,
    String signalName = 'SIGSEGV',
    int signalNumber = 11,
    TombstoneNoise noise = const TombstoneNoise(),
  }) {
    final out = _Writer()
      ..string(1, 'arm64')
      ..string(2, 'google/pixel/fingerprint')
      ..varint(5, 4242)
      ..varint(6, tid)
      ..string(8, noise.selinuxLabel)
      ..string(9, noise.commandLine)
      ..message(10, (w) {
        w
          ..varint(1, signalNumber)
          ..string(2, signalName)
          ..varint(3, 1)
          ..string(4, 'SEGV_MAPERR');
      })
      ..string(14, noise.abortMessage);
    for (final entry in threads.entries) {
      out.message(16, (entry16) {
        entry16
          ..varint(1, entry.key)
          ..message(2, (thread) {
            thread
              ..varint(1, entry.key)
              ..string(2, noise.threadName)
              ..message(3, (register) {
                register
                  ..string(1, noise.registerName)
                  ..varint(2, 0x7ffd1234)
                  ..fixed64(3, 99);
              })
              ..message(5, (dump) {
                dump
                  ..string(1, noise.registerName)
                  ..bytes(3, utf8.encode(noise.abortMessage));
              });
            for (final frame in entry.value) {
              thread.message(4, (f) {
                f
                  ..varint(1, frame.relPc)
                  ..varint(2, frame.relPc + 0x7000000000)
                  ..varint(3, 0x7ffd0000)
                  ..string(4, frame.function ?? '')
                  ..varint(5, frame.functionOffset)
                  ..string(
                    6,
                    frame.file == null
                        ? ''
                        : '${noise.pathPrefix}/${frame.file}',
                  )
                  ..varint(7, 4096)
                  ..string(8, frame.buildId ?? '');
              });
            }
          });
      });
    }
    out
      ..message(17, (mapping) {
        mapping
          ..varint(1, 0x7000000000)
          ..string(7, '${noise.pathPrefix}/${noise.openFilePath}');
      })
      ..message(18, (log) {
        log
          ..string(1, 'main')
          ..message(3, (line) {
            line
              ..varint(1, 100)
              ..string(4, noise.logLine);
          });
      })
      ..message(19, (fd) {
        fd
          ..varint(1, 3)
          ..string(2, noise.openFilePath);
      })
      ..fixed32(99, 7);
    return out.toBytes();
  }
}

class _Writer {
  final BytesBuilder _bytes = BytesBuilder();

  void _varint(int value) {
    var rest = value;
    while (true) {
      final low = rest & 0x7f;
      rest = rest >>> 7;
      if (rest == 0) {
        _bytes.addByte(low);
        return;
      }
      _bytes.addByte(low | 0x80);
    }
  }

  void _tag(int field, int wire) => _varint((field << 3) | wire);

  void varint(int field, int value) {
    _tag(field, 0);
    _varint(value);
  }

  void fixed64(int field, int value) {
    _tag(field, 1);
    _bytes.add(
      (ByteData(8)..setUint64(0, value, Endian.little)).buffer.asUint8List(),
    );
  }

  void fixed32(int field, int value) {
    _tag(field, 5);
    _bytes.add(
      (ByteData(4)..setUint32(0, value, Endian.little)).buffer.asUint8List(),
    );
  }

  void bytes(int field, List<int> value) {
    _tag(field, 2);
    _varint(value.length);
    _bytes.add(value);
  }

  void string(int field, String value) => bytes(field, utf8.encode(value));

  void message(int field, void Function(_Writer writer) build) {
    final inner = _Writer();
    build(inner);
    bytes(field, inner.toBytes());
  }

  Uint8List toBytes() => _bytes.toBytes();
}
