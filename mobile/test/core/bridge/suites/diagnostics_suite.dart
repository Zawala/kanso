// Kanso — diagnostics_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/diagnostics.dart';
import 'package:kanso/core/bridge/api/types.dart';

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
      expect(await library.notes(query: everyNote), hasLength(2));
    });

    test('the secret store keeps a value and gives it back', () async {
      // Rust writes, reads and removes an item in the Keychain, the Keystore
      // or (on a computer) memory.
      await secretsProbe();
    });
  });
}
