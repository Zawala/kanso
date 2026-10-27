// Kanso — crash_scrubber.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'crash_grammar.dart';

/// Turns raw pieces into the whitelisted pieces of a report.
///
/// Every method returns a value that matches [CrashGrammar] or null/`?`; none
/// passes text through that the grammar does not allow. Pure Dart: no Flutter,
/// no IO.
abstract final class CrashScrubber {
  /// The word for a value that is not known.
  static const String unknown = 'unknown';

  /// `<anonymous closure>`, as the VM writes it, and its replacement.
  static const String _anonymousClosure = '<anonymous closure>';
  static const String _closure = '<closure>';

  /// A type name: generic arguments cut at the first `<`, else `unknown`.
  static String type(String? raw) {
    if (raw == null) {
      return unknown;
    }
    final cut = raw.indexOf('<');
    final name = cut < 0 ? raw : raw.substring(0, cut);
    return CrashGrammar.type.hasMatch(name) ? name : unknown;
  }

  /// [raw] if it is an app version, else null.
  static String? appVersion(String? raw) =>
      raw != null && CrashGrammar.appVersion.hasMatch(raw) ? raw : null;

  /// [raw] if it is a build number, else null.
  static String? appBuild(String? raw) =>
      raw != null && CrashGrammar.appBuild.hasMatch(raw) ? raw : null;

  /// [raw] if it is a patch number in range, else null.
  static int? patch(int? raw) =>
      raw != null && raw >= 0 && raw <= CrashGrammar.maxPatch ? raw : null;

  /// [raw] if it is an OS version, else null.
  static String? osVersion(String? raw) =>
      raw != null && CrashGrammar.osVersion.hasMatch(raw) ? raw : null;

  /// The first dotted number in [raw] (`iPhone OS 18.1 (22B83)` gives
  /// `18.1`) if it is an OS version, else null.
  static String? osVersionIn(String? raw) {
    if (raw == null || raw.length > 256) {
      return null;
    }
    final found = RegExp(r'[0-9]+(?:\.[0-9]+){0,3}').firstMatch(raw);
    return found == null ? null : osVersion(found.group(0));
  }

  /// [raw] if it is a device identifier, else null.
  static String? device(String? raw) =>
      raw != null && CrashGrammar.device.hasMatch(raw) ? raw : null;

  /// [raw] if it is a day (`2026-10-26`) that exists, else null.
  static String? day(String? raw) {
    if (raw == null || !CrashGrammar.day.hasMatch(raw)) {
      return null;
    }
    final year = int.parse(raw.substring(0, 4));
    final month = int.parse(raw.substring(5, 7));
    final dayOfMonth = int.parse(raw.substring(8, 10));
    if (month < 1 || month > 12 || dayOfMonth < 1 || dayOfMonth > 31) {
      return null;
    }
    return year >= 2000 ? raw : null;
  }

  /// The UTC day of [time], `2026-10-26`.
  static String dayOf(DateTime time) {
    final utc = time.toUtc();
    String two(int n) => n.toString().padLeft(2, '0');
    return '${utc.year.toString().padLeft(4, '0')}-${two(utc.month)}-'
        '${two(utc.day)}';
  }

  /// [value] in lower-case hexadecimal as an unsigned 64-bit number.
  static String hex(int value) =>
      BigInt.from(value).toUnsigned(64).toRadixString(16);

  /// `dart <symbol> <uri>:<line>[:<column>]`, with `?` for a symbol or a
  /// location that the grammar does not allow.
  ///
  /// [location] is the text after the symbol, for example
  /// `package:kanso/a.dart:12:3`; its last one or two `:number` parts are the
  /// line and column.
  static String dartFrame({required String symbol, required String location}) {
    final name = _dartSymbol(symbol);
    final place = _dartLocation(location);
    final frame = 'dart $name $place';
    return frame.length <= CrashGrammar.maxFrameLength ? frame : 'dart $name ?';
  }

  static String _dartSymbol(String raw) {
    if (raw.length > 400) {
      return '?';
    }
    final name = raw.replaceAll(_anonymousClosure, _closure);
    return CrashGrammar.dartSymbol.hasMatch(name) ? name : '?';
  }

  static String _dartLocation(String raw) {
    final text = raw.trim();
    if (text.length > 400) {
      return '?';
    }
    final withColumn = RegExp(r'^(.*):([0-9]+):([0-9]+)$').firstMatch(text);
    final withLine = RegExp(r'^(.*):([0-9]+)$').firstMatch(text);
    final String uri;
    final String line;
    final String? column;
    if (withColumn != null) {
      uri = withColumn.group(1)!;
      line = withColumn.group(2)!;
      column = withColumn.group(3);
    } else if (withLine != null) {
      uri = withLine.group(1)!;
      line = withLine.group(2)!;
      column = null;
    } else {
      return '?';
    }
    if (!CrashGrammar.dartUri.hasMatch(uri) ||
        !CrashGrammar.number.hasMatch(line) ||
        (column != null && !CrashGrammar.number.hasMatch(column))) {
      return '?';
    }
    return column == null ? '$uri:$line' : '$uri:$line:$column';
  }

  /// `dart-pc +0x<hex>`, or null if [offset] is negative.
  static String? dartPcFrame(int offset) {
    if (offset < 0) {
      return null;
    }
    final frame = 'dart-pc +0x${hex(offset)}';
    return CrashGrammar.dartPcFrame.hasMatch(frame) ? frame : null;
  }

  /// `native <binary> +0x<hex>[ <symbol>+0x<hex>][ <build-id>]`.
  ///
  /// [binary] is a base name (see [baseName]); anything else is `?`. A
  /// [symbol] or [buildId] the grammar does not allow is left out.
  static String? nativeFrame({
    required String? binary,
    required int offset,
    String? symbol,
    int? symbolOffset,
    String? buildId,
  }) {
    if (offset < 0) {
      return null;
    }
    final name = binary != null && CrashGrammar.binary.hasMatch(binary)
        ? binary
        : '?';
    final buffer = StringBuffer('native $name +0x${hex(offset)}');
    if (symbol != null &&
        CrashGrammar.nativeSymbol.hasMatch(symbol) &&
        symbolOffset != null &&
        symbolOffset >= 0) {
      buffer.write(' $symbol+0x${hex(symbolOffset)}');
    }
    if (buildId != null && CrashGrammar.buildId.hasMatch(buildId)) {
      buffer.write(' $buildId');
    }
    final frame = buffer.toString();
    return CrashGrammar.isFrame(frame) ? frame : null;
  }

  /// `jvm <class>.<method>:<line>`, or null if the class or the method is
  /// not allowed. A line out of range is `-1`.
  static String? jvmFrame({
    required String cls,
    required String method,
    required int line,
  }) {
    if (!CrashGrammar.jvmClass.hasMatch(cls) ||
        !CrashGrammar.jvmMethod.hasMatch(method)) {
      return null;
    }
    final number = line.abs() <= 9999999 ? line : -1;
    final frame = 'jvm $cls.$method:$number';
    return CrashGrammar.isFrame(frame) ? frame : null;
  }

  /// What follows the last `/` or `!` of [path]: the base name of a binary.
  /// Everything before it (the directories, the APK) is never looked at.
  static String baseName(String path) {
    final cut = path.lastIndexOf('/') > path.lastIndexOf('!')
        ? path.lastIndexOf('/')
        : path.lastIndexOf('!');
    return cut < 0 ? path : path.substring(cut + 1);
  }
}
