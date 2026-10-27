// Kanso — crash_grammar_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/crash/utils/crash_grammar.dart';

const String _longFrame1 =
    'dart _NoteEditorState._save '
    'package:kanso/note/views/widgets/note_editor.dart:120:7';

void main() {
  group('the frames of docs/CRASH-REPORTS.md', () {
    test('the documented examples are frames', () {
      for (final frame in <String>[
        _longFrame1,
        'dart <closure> dart:async/zone.dart:1525:47',
        'dart foo package:kanso/a.dart:12',
        'dart ? ?',
        'dart Foo.bar ?',
        'dart-pc +0x6a87',
        'native libkanso_mobile.so +0x1a2b3c',
        'native Runner +0x1a2b3c 0A7FC3D2-1111-2222-3333-444455556666',
        'native libc.so +0x4e7a4 syscall+0x1c',
        'native libc.so +0x4e7a4 syscall+0x1c abcdef0123456789',
        'native ? +0x0',
        'jvm com.foo.Bar.baz:12',
        'jvm java.lang.Object.wait:-2',
        r'jvm com.foo.Bar$Inner.<init>:-1',
      ]) {
        expect(CrashGrammar.isFrame(frame), isTrue, reason: frame);
      }
    });

    test('anything else is not', () {
      for (final frame in <String>[
        '',
        'dart',
        'dart foo',
        'dart foo file:///Users/me/Notes/a.dart:1:2',
        'dart foo package:kanso/a.dart',
        'dart foo package:kanso/a.dart:1:2:3',
        'dart foo package:Kanso/a.dart:1:2',
        'dart my note package:kanso/a.dart:1:2',
        'dart-pc 0x6a87',
        'dart-pc +0xZZ',
        'dart-pc +0x12345678901234567',
        'native lib so +0x1',
        'native libc.so +1',
        'native libc.so +0x1 sym bol+0x1',
        'native libc.so +0x1 short',
        'jvm Foo:12',
        'jvm com.foo.Bar.baz',
        'jvm com.foo.Bar.baz qux:12',
        'jvm 1com.foo.Bar.baz:12',
        'unknown foo',
        'dart foo package:kanso/a.dart:1:2\n',
        ' dart foo package:kanso/a.dart:1:2',
      ]) {
        expect(CrashGrammar.isFrame(frame), isFalse, reason: frame);
      }
    });

    test('a frame is at most 300 characters', () {
      final longUri = 'package:kanso/${'a' * 200}.dart:1:1';
      final symbol = 'S' * 199;
      expect('dart $symbol $longUri'.length, greaterThan(300));
      expect(CrashGrammar.isFrame('dart $symbol $longUri'), isFalse);
      expect(CrashGrammar.isFrame('dart $symbol ?'), isTrue);
    });
  });

  group('the fields of docs/CRASH-REPORTS.md', () {
    test('type', () {
      expect(CrashGrammar.type.hasMatch('StateError'), isTrue);
      expect(
        CrashGrammar.type.hasMatch('java.lang.IllegalStateException'),
        isTrue,
      );
      expect(CrashGrammar.type.hasMatch(r'_$Foo'), isTrue);
      expect(CrashGrammar.type.hasMatch('1Foo'), isFalse);
      expect(CrashGrammar.type.hasMatch('Foo Bar'), isFalse);
      expect(CrashGrammar.type.hasMatch('a' * 129), isFalse);
      expect(CrashGrammar.type.hasMatch('a' * 128), isTrue);
    });

    test('app, os, device and day', () {
      expect(CrashGrammar.appVersion.hasMatch('0.9.0'), isTrue);
      expect(CrashGrammar.appVersion.hasMatch('0.9'), isFalse);
      expect(CrashGrammar.appVersion.hasMatch('0.9.0-beta'), isFalse);
      expect(CrashGrammar.appBuild.hasMatch('57'), isTrue);
      expect(CrashGrammar.appBuild.hasMatch('57a'), isFalse);
      expect(CrashGrammar.osVersion.hasMatch('18.1'), isTrue);
      expect(CrashGrammar.osVersion.hasMatch('18 1'), isFalse);
      expect(CrashGrammar.device.hasMatch('iPhone17,1'), isTrue);
      expect(CrashGrammar.device.hasMatch('samsung SM-S918B'), isTrue);
      expect(CrashGrammar.device.hasMatch("Ngoni's phone"), isFalse);
      expect(CrashGrammar.day.hasMatch('2026-10-26'), isTrue);
      expect(CrashGrammar.day.hasMatch('2026-10-26T10:00'), isFalse);
    });
  });
}
