// Kanso — dart_stack_parser.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'crash_grammar.dart';
import 'crash_scrubber.dart';

/// Frames from the text of a Dart `StackTrace`.
///
/// Two forms are read: the symbolic one, `#0      Foo.bar
/// (package:kanso/x.dart:12:3)`, and the non-symbolic one of an AOT build made
/// with `--split-debug-info`, `#00 abs … virt …
/// _kDartIsolateSnapshotInstructions+0x6a87`.
/// Header lines (`*** *** ***`, `pid:`, `build_id`, `isolate_dso_base`…),
/// `<asynchronous suspension>` and anything else are ignored.
abstract final class DartStackParser {
  /// The longest line looked at; a longer one is not a frame.
  static const int _maxLine = 2000;

  /// The most text looked at.
  static const int _maxText = 512 * 1024;

  static final RegExp _symbolic = RegExp(r'^\s*#[0-9]+\s+(.*?)\s+\((.*)\)\s*$');
  static final RegExp _absolute = RegExp(
    r'^\s*#[0-9]+\s+abs\s+[0-9a-fA-F]+\s+(?:virt\s+[0-9a-fA-F]+\s+)?(\S+)',
  );
  static final RegExp _isolateOffset = RegExp(
    r'^_kDartIsolateSnapshotInstructions\+0x([0-9a-fA-F]{1,16})$',
  );

  /// The scrubbed frames of [trace], innermost first, at most
  /// [CrashGrammar.maxFrames]. Never throws.
  static List<String> parse(String trace) {
    final frames = <String>[];
    try {
      final text = trace.length > _maxText
          ? trace.substring(0, _maxText)
          : trace;
      for (final line in text.split('\n')) {
        if (frames.length >= CrashGrammar.maxFrames) {
          break;
        }
        if (line.length > _maxLine) {
          continue;
        }
        final frame = _frame(line);
        if (frame != null) {
          frames.add(frame);
        }
      }
    } on Object {
      // Whatever was read stays; the rest is not a trace.
    }
    return frames;
  }

  static String? _frame(String line) {
    final absolute = _absolute.firstMatch(line);
    if (absolute != null) {
      final offset = _isolateOffset.firstMatch(absolute.group(1)!);
      return offset == null
          ? null
          : CrashScrubber.dartPcFrame(int.parse(offset.group(1)!, radix: 16));
    }
    final symbolic = _symbolic.firstMatch(line);
    if (symbolic == null) {
      return null;
    }
    return CrashScrubber.dartFrame(
      symbol: symbolic.group(1)!,
      location: symbolic.group(2)!,
    );
  }
}
