// Kanso — crash_grammar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The patterns of docs/CRASH-REPORTS.md, written once. The scrubber lets a
/// value through only when it matches; the tests check every frame of every
/// report against [isFrame]; the repository checks a report again before it is
/// sent.
///
/// Nothing here is a "cleaner": a value that does not match is dropped or
/// replaced, never edited.
abstract final class CrashGrammar {
  /// The most frames a report holds.
  static const int maxFrames = 128;

  /// The longest a frame is, in characters.
  static const int maxFrameLength = 300;

  /// The most a report's text may be, in bytes.
  static const int maxBodyBytes = 32 * 1024;

  /// The highest patch number.
  static const int maxPatch = 100000;

  // Fragments, unanchored, that the whole-value patterns are made of.
  static const String _dartSymbol = r'[A-Za-z_$<][A-Za-z0-9_$.<>]{0,199}';
  static const String _dartUri =
      r'(?:package:[a-z0-9_]{1,64}/[A-Za-z0-9_/.-]{1,200}\.dart'
      '|dart:[a-z0-9_/.-]{1,100})';
  static const String _number = '[0-9]{1,7}';
  static const String _hex = '[0-9a-f]{1,16}';
  static const String _binary = '[A-Za-z0-9_.+-]{1,64}';
  static const String _nativeSymbol = r'[A-Za-z0-9_$.]{1,256}';
  static const String _buildId = '[0-9a-fA-F-]{8,64}';
  static const String _jvmClass = r'[A-Za-z_$][A-Za-z0-9_$.]{0,199}';
  static const String _jvmMethod = r'[A-Za-z_$<][A-Za-z0-9_$<>]{0,99}';
  static const String _jvmLine = '-?[0-9]{1,7}';

  // Fields.
  static final RegExp type = RegExp(r'^[A-Za-z_$][A-Za-z0-9_$.]{0,127}$');
  static final RegExp appVersion = RegExp(r'^[0-9]{1,4}(\.[0-9]{1,4}){2}$');
  static final RegExp appBuild = RegExp(r'^[0-9]{1,10}$');
  static final RegExp osVersion = RegExp(r'^[0-9A-Za-z._-]{1,32}$');
  static final RegExp device = RegExp(r'^[A-Za-z0-9 ,._()-]{1,64}$');
  static final RegExp day = RegExp(r'^[0-9]{4}-[0-9]{2}-[0-9]{2}$');

  // Parts of frames.
  static final RegExp dartSymbol = RegExp('^$_dartSymbol\$');
  static final RegExp dartUri = RegExp('^$_dartUri\$');
  static final RegExp number = RegExp('^$_number\$');
  static final RegExp hex = RegExp('^$_hex\$');
  static final RegExp binary = RegExp('^$_binary\$');
  static final RegExp nativeSymbol = RegExp('^$_nativeSymbol\$');
  static final RegExp buildId = RegExp('^$_buildId\$');
  static final RegExp jvmClass = RegExp('^$_jvmClass\$');
  static final RegExp jvmMethod = RegExp('^$_jvmMethod\$');
  static final RegExp jvmLine = RegExp('^$_jvmLine\$');

  // Whole frames.
  static final RegExp dartFrame = RegExp(
    '^dart (?:$_dartSymbol|\\?) '
    '(?:$_dartUri:$_number(?::$_number)?|\\?)\$',
  );
  static final RegExp dartPcFrame = RegExp('^dart-pc \\+0x$_hex\$');
  static final RegExp nativeFrame = RegExp(
    '^native (?:$_binary|\\?) \\+0x$_hex'
    '(?: $_nativeSymbol\\+0x$_hex)?'
    '(?: $_buildId)?\$',
  );
  static final RegExp jvmFrame = RegExp(
    '^jvm $_jvmClass\\.$_jvmMethod:$_jvmLine\$',
  );

  /// Whether [text] is one of the four frame forms and no longer than
  /// [maxFrameLength].
  static bool isFrame(String text) {
    return text.length <= maxFrameLength &&
        (dartFrame.hasMatch(text) ||
            dartPcFrame.hasMatch(text) ||
            nativeFrame.hasMatch(text) ||
            jvmFrame.hasMatch(text));
  }
}
