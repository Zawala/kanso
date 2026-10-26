// Kanso — diagnostics_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/diagnostics.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'package:path/path.dart' as p;

import 'open_library.dart';

/// The library's diagnostics, through the bridge: every function of
/// `api/diagnostics.dart`.
void diagnosticsSuite(OpenLibrary open) {
  group('diagnostics', () {
    test('the library has a version and can check certificates', () async {
      expect(await version(), matches(RegExp(r'^\d+\.\d+\.\d+')));
      expect(await tlsReady(), isTrue);
    });

    test('a panic comes back as an error and the library goes on', () async {
      final library = await open();
      final id = await makeNote(library, 'Before', 'text');
      await expectLater(
        panicProbe(message: 'on purpose'),
        throwsA(
          isA<KansoError_Internal>().having(
            (error) => error.message,
            'message',
            'on purpose',
          ),
        ),
      );
      expect((await library.note(id: id)).title, 'Before');
      expect(await makeNote(library, 'After', 'text'), isNotEmpty);
      expect((await library.list(query: everyNote)).rows, hasLength(2));
    });

    test('a library folder is kept out of backups on an Apple phone', () async {
      final folder = Directory.systemTemp.createTempSync('kanso_backup_');
      addTearDown(() => folder.deleteSync(recursive: true));
      // Nothing is open in it yet, so it is not excluded; once a library is
      // open there, it is. Android keeps the app out of backups in its
      // manifest and reports nothing.
      final before = await backupExcluded(folder: folder.path);
      expect(before, anyOf(isNull, isFalse));
      await KansoLibrary.open(path: p.join(folder.path, 'library.db'));
      final after = await backupExcluded(folder: folder.path);
      if (Platform.isIOS || Platform.isMacOS) {
        expect(after, isTrue);
      } else {
        expect(after, isNull);
      }
      expect(
        await backupExcluded(folder: p.join(folder.path, 'nowhere')),
        isNull,
      );
    });

    test('the secret store keeps a value and gives it back', () async {
      // Rust writes, reads and removes an item in the Keychain, the Keystore
      // or (on a computer) memory.
      await secretsProbe();
    });
  });
}
