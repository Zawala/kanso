// Kanso — locks_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The lock session, through the bridge: every call of `KansoLocks`.
void locksSuite(OpenLibrary open) {
  group('KansoLocks', () {
    test(
      'a locked note closes to its title, and opens with the password',
      () async {
        final library = await open();
        final locks = library.locks();
        final id = await makeNote(library, 'Plan', 'the secret plan');

        // Nothing is locked yet, so there is no password to try.
        await expectLater(
          locks.unlock(password: lockPassword),
          throwsA(isA<KansoError_Invalid>()),
        );
        expect(await locks.isUnlocked(), isFalse);

        final events = <LockEvent>[];
        final subscription = locks.lockEvents().listen(events.add);
        // Cancelling waits for the library's next event: don't wait for it.
        addTearDown(() => unawaited(subscription.cancel()));
        await Future<void>.delayed(const Duration(milliseconds: 200));

        // Locking the first note sets the password, and leaves it open.
        final saved = await locks.lockNote(id: id, password: lockPassword);
        expect(saved.id, id);
        expect(await locks.isUnlocked(), isTrue);
        expect(await locks.isNoteUnlocked(id: id), isTrue);
        final open_ = await library.note(id: id);
        expect(open_.lock, isA<LockState_Open>());
        expect(open_.body, 'the secret plan');

        await locks.lock();
        expect(await locks.isUnlocked(), isFalse);
        final closed = await library.note(id: id);
        expect(closed.lock, isA<LockState_Closed>());
        expect(closed.body, isNull);
        // The title shows; the text never does.
        expect(closed.title, 'Plan');
        final row = (await library.notes(query: everyNote)).single;
        expect(row.locked, isTrue);
        expect(row.title, 'Plan');
        expect(row.snippet, isEmpty);
        expect(await library.properties(id: id), isEmpty);
        expect(await library.search(text: 'secret', limit: 10), isEmpty);
        await expectLater(
          library.save(id: id, baseRev: closed.rev, body: 'overwritten'),
          throwsA(isA<KansoError_Locked>()),
        );
        await expectLater(
          library.renderer().previewHtml(id: id, palette: _palette),
          throwsA(isA<KansoError_Locked>()),
        );

        expect(await locks.unlock(password: 'not the password'), isFalse);
        expect(await locks.unlock(password: lockPassword), isTrue);
        expect((await library.note(id: id)).body, 'the secret plan');
        await expectLater(
          locks.isNoteUnlocked(id: 'note:nope'),
          throwsA(isA<KansoError_NotFound>()),
        );

        await Future<void>.delayed(const Duration(milliseconds: 200));
        expect(events, <LockEvent>[
          const LockEvent.unlocked(),
          const LockEvent.locked(reason: LockReason.manual),
          const LockEvent.unlocked(),
        ]);
      },
      timeout: const Timeout(Duration(minutes: 3)),
    );
  });
}

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFFFFFF,
  text: 0x222222,
  muted: 0x888888,
  accent: 0x3366AA,
  border: 0xDDDDDD,
  surface: 0xF4F4F4,
  quote: 0x666666,
  taskDone: 0x888888,
  inlineCodeBackground: 0xEEEEEE,
);
