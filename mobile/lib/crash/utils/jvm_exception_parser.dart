// Kanso — jvm_exception_parser.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import '../models/data/crash_trace.dart';
import 'crash_grammar.dart';
import 'crash_scrubber.dart';

/// An uncaught exception, as the Kotlin handler writes it:
/// `{"type":"java.lang.IllegalStateException","frames":[{"cls":"…",
/// "method":"…","line":12}],"day":"2026-10-26"}`. It has no message and no
/// file name, and the parser reads no other key.
abstract final class JvmExceptionParser {
  static const int _maxText = 1024 * 1024;

  /// The crash in [json], or null if it has no usable frame. Never throws.
  static CrashTrace? parse(String json) {
    try {
      if (json.length > _maxText) {
        return null;
      }
      final root = jsonDecode(json);
      if (root is! Map<String, dynamic>) {
        return null;
      }
      final list = root['frames'];
      if (list is! List) {
        return null;
      }
      final frames = <String>[];
      for (final item in list) {
        if (frames.length >= CrashGrammar.maxFrames) {
          break;
        }
        if (item is! Map<String, dynamic>) {
          continue;
        }
        final cls = item['cls'];
        final method = item['method'];
        final line = item['line'];
        if (cls is! String || method is! String) {
          continue;
        }
        final frame = CrashScrubber.jvmFrame(
          cls: cls,
          method: method,
          line: line is int ? line : -1,
        );
        if (frame != null) {
          frames.add(frame);
        }
      }
      if (frames.isEmpty) {
        return null;
      }
      final type = root['type'];
      final day = root['day'];
      return CrashTrace(
        type: CrashScrubber.type(type is String ? type : null),
        frames: frames,
        day: CrashScrubber.day(day is String ? day : null),
      );
    } on Object {
      return null;
    }
  }
}
