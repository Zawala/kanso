// Kanso — dart_stack_parser_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, any;
import 'package:kanso/crash/utils/crash_grammar.dart';
import 'package:kanso/crash/utils/dart_stack_parser.dart';

import '../../helpers/crash_generators.dart';
import '../../helpers/property.dart';

const String _symbolic = '''
#0      _NoteEditorState._save (package:kanso/note/views/widgets/note_editor.dart:120:7)
#1      _NoteEditorState.build.<anonymous closure> (package:kanso/note/views/widgets/note_editor.dart:88:21)
#2      _rootRunUnary (dart:async/zone.dart:1525:47)
<asynchronous suspension>
#3      new Foo.bar (package:kanso/a.dart:3:4)
#4      main.<anonymous closure> (file:///Users/ngoni/Notes%20for%20Mum/main.dart:9:3)
#5      Foo.bar (org-dartlang-sdk:///lib/core/errors.dart:10:2)
#6      bar (package:kanso/nope.txt:1:2)
''';

const String _aot = '''
*** *** *** *** *** *** *** *** *** *** *** *** *** *** *** ***
pid: 4242, tid: 4243, name 1.ui
os: ios arch: arm64 comp: no sim: no
build_id: 'a1b2c3d4e5f60718293a4b5c6d7e8f90'
isolate_dso_base: 1049f4000, vm_dso_base: 1049f4000
isolate_instructions: 1049fa300, vm_instructions: 1049f6000
    #00 abs 00000001049fb001 virt 0000000000007001 _kDartIsolateSnapshotInstructions+0x6d01
<asynchronous suspension>
    #01 abs 00000001049fb101 virt 0000000000007101 _kDartVmSnapshotInstructions+0x1234
    #02 abs 00000001049fb201 virt 0000000000007201 _kDartIsolateSnapshotInstructions+0xABCDEF
''';

const String _longFrame1 =
    'dart _NoteEditorState._save '
    'package:kanso/note/views/widgets/note_editor.dart:120:7';

const String _longFrame2 =
    'dart _NoteEditorState.build.<closure> '
    'package:kanso/note/views/widgets/note_editor.dart:88:21';

void main() {
  group('DartStackParser examples', () {
    test('symbolic frames', () {
      expect(DartStackParser.parse(_symbolic), <String>[
        _longFrame1,
        _longFrame2,
        'dart _rootRunUnary dart:async/zone.dart:1525:47',
        // A symbol with a space is not let through.
        'dart ? package:kanso/a.dart:3:4',
        // A file path and an SDK path keep only the symbol.
        'dart main.<closure> ?',
        'dart Foo.bar ?',
        // Not a Dart file.
        'dart bar ?',
      ]);
    });

    test('the non-symbolic form keeps the isolate offsets only', () {
      expect(DartStackParser.parse(_aot), <String>[
        'dart-pc +0x6d01',
        'dart-pc +0xabcdef',
      ]);
    });

    test('the header lines and suspensions are not frames', () {
      expect(
        DartStackParser.parse(
          'pid: 1\nbuild_id: abc\n<asynchronous suspension>',
        ),
        isEmpty,
      );
    });

    test('at most 128 frames, the innermost', () {
      final trace = <String>[
        for (var i = 0; i < 300; i++)
          '#$i      f$i (package:kanso/a.dart:$i:1)',
      ].join('\n');
      final frames = DartStackParser.parse(trace);
      expect(frames, hasLength(128));
      expect(frames.first, 'dart f0 package:kanso/a.dart:0:1');
    });

    test('a very long line is not a frame', () {
      expect(
        DartStackParser.parse('#0 ${'a' * 5000} (package:kanso/a.dart:1:1)'),
        isEmpty,
      );
    });

    test('a symbol over 200 characters becomes ?', () {
      final frames = DartStackParser.parse(
        '#0 ${'a' * 201} (package:kanso/a.dart:1:1)',
      );
      expect(frames, <String>['dart ? package:kanso/a.dart:1:1']);
    });
  });

  group('DartStackParser properties', () {
    Glados2<String, String>(
      any.noise,
      any.noise,
      kansoExploreConfig(),
    ).test('whatever the lines say, every frame is in the grammar', (a, b) {
      final trace = '#0 $a ($b)\n    #1 abs $a virt $b $a';
      for (final frame in DartStackParser.parse(trace)) {
        expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
      }
    });

    Glados(any.noise, kansoExploreConfig()).test('never throws on any text', (
      text,
    ) {
      expect(() => DartStackParser.parse(text), returnsNormally);
    });

    Glados2<String, String>(
      any.noteText,
      any.noteText,
      kansoExploreConfig(),
    ).test('a note in a path or a message never reaches a frame', (a, b) {
      final trace =
          'Unhandled: $a\n'
          '#0      foo (file:///Users/$a/$b.dart:1:2)\n'
          '#1      bar (package:kanso/x.dart:3:4)\n'
          '    #2 abs 00 virt 00 $a';
      final frames = DartStackParser.parse(trace);
      expect(frames.join('\n'), isNot(contains(leakMarker)));
      expect(frames.first, 'dart foo ?');
    });
  });
}
