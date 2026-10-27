// Kanso — crash_generators.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';
import 'dart:typed_data';

import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        CombinableAny,
        Generator,
        GeneratorUtils,
        IntAnys,
        ListAnys,
        StringAnys;

import 'tombstone_encoder.dart';

/// The start of every generated note text: if it shows up in a report, text
/// leaked, whatever else the report holds.
const String leakMarker = 'Zk9';

/// Characters that no part of the report grammar allows, not even the
/// device's, which allows spaces and commas.
const List<String> _outsideTheGrammar = <String>[
  'é',
  '漢',
  '!',
  '@',
  '#',
  '%',
  r'\',
  '*',
  '"',
  ';',
  '=',
  '&',
];

/// More characters a note has: spaces and commas too.
const List<String> _inNotes = <String>[..._outsideTheGrammar, ' ', ','];

/// What a message, a log line or a path might hold, newlines and quotes
/// included.
const List<String> _noiseCharacters = <String>[
  'a',
  'Z',
  '7',
  ' ',
  'é',
  '漢',
  '"',
  r'\',
  '\n',
  '#',
  '(',
  ')',
  ':',
  '/',
  '<',
  '>',
  r'$',
  '.',
  '-',
  '+',
  '?',
  '{',
  '}',
  '[',
  ']',
  "'",
  '0x',
  'at ',
  'dart ',
];

extension CrashAny on Any {
  /// A note's text: starts with [leakMarker] and holds at least one
  /// character outside `[A-Za-z0-9_$.]` that the grammar doesn't allow either.
  Generator<String> get noteText => combine4(
    letterOrDigits,
    choose(_outsideTheGrammar),
    letterOrDigits,
    choose(_inNotes),
    (String a, String c, String b, String d) => '$leakMarker$a$c$b$d',
  );

  /// Any text, with newlines, quotes, slashes and the shapes of frames in it.
  Generator<String> get noise => listWithLengthInRange<String>(
    0,
    24,
    choose(_noiseCharacters),
  ).map((parts) => parts.join());

  /// [noise] on one line.
  Generator<String> get noiseLine =>
      noise.map((text) => text.replaceAll('\n', ' '));

  /// Random bytes.
  Generator<Uint8List> get randomBytes => listWithLengthInRange<int>(
    0,
    400,
    intInRange(0, 256),
  ).map(Uint8List.fromList);

  /// A valid native frame, as a tombstone holds it.
  Generator<TombstoneFrame> get validTombstoneFrame => combine5(
    intInRange(0, 1 << 40),
    choose(<String?>[
      null,
      'abort',
      '_ZN3art6Thread4DumpE',
      'pthread_kill',
      'Java_app_kanso_Native_init',
    ]),
    intInRange(0, 5000),
    choose(<String?>[
      null,
      'libc.so',
      'libflutter.so',
      'base.apk!libapp.so',
      'libkanso_mobile_bridge.so',
    ]),
    choose(<String?>[
      null,
      '0123456789abcdef0123456789abcdef01234567',
      'ABCDEF01-2345-6789-ABCD-EF0123456789',
    ]),
    (int pc, String? fn, int offset, String? file, String? build) =>
        TombstoneFrame(
          relPc: pc,
          function: fn,
          functionOffset: offset,
          file: file,
          buildId: build,
        ),
  );

  /// A native frame whose every text part is [noise].
  Generator<TombstoneFrame> get wildTombstoneFrame => combine5(
    intInRange(0, 1 << 62),
    noise,
    intInRange(0, 1 << 30),
    noise,
    noise,
    (int pc, String fn, int offset, String file, String build) =>
        TombstoneFrame(
          relPc: pc,
          function: fn,
          functionOffset: offset,
          file: file,
          buildId: build,
        ),
  );

  /// A thread's backtrace of one to six valid frames.
  Generator<List<TombstoneFrame>> get validBacktrace =>
      listWithLengthInRange<TombstoneFrame>(1, 7, validTombstoneFrame);

  /// A seed for mutating bytes.
  Generator<int> get mutationSeed => intInRange(0, 1 << 30);
}

/// [bytes] with a few bytes changed and maybe cut short, from [seed].
Uint8List mutate(Uint8List bytes, int seed) {
  final random = Random(seed);
  final copy = Uint8List.fromList(bytes);
  if (copy.isEmpty) {
    return copy;
  }
  for (var i = 0; i < 1 + random.nextInt(4); i++) {
    copy[random.nextInt(copy.length)] = random.nextInt(256);
  }
  return random.nextBool()
      ? Uint8List.sublistView(copy, 0, random.nextInt(copy.length + 1))
      : copy;
}
