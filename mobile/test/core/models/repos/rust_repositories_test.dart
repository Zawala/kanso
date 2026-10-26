// Kanso — rust_repositories_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/locks.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_library_repository.dart';
import 'package:kanso/core/models/repos/impl/rust_session_repository.dart';
import 'package:mocktail/mocktail.dart';

class _MockLibrary extends Mock implements KansoLibrary {}

class _MockLocks extends Mock implements KansoLocks {}

const _query = NoteQuery(
  trash: false,
  limit: 0,
  skip: 0,
  sort: SortKey.updated,
  descending: true,
);

void main() {
  group('RustLibraryRepository', () {
    late _MockLibrary library;
    late RustLibraryRepository repository;

    setUp(() {
      library = _MockLibrary();
      repository = RustLibraryRepository(library);
    });

    test('returns the notes the library lists', () async {
      when(() => library.notes(query: _query)).thenAnswer(
        (_) async => const <NoteRow>[],
      );
      expect(
        await repository.notes(_query),
        const Right<ApplicationError, List<NoteRow>>(<NoteRow>[]),
      );
    });

    test('a note the library does not have is a not-found error', () async {
      when(() => library.note(id: 'n_1')).thenAnswer(
        (_) => Future<NoteView>.error(const KansoError.notFound()),
      );
      final result = await repository.note('n_1');
      expect(result.swap().toOption().toNullable(), isA<NotFoundError>());
    });

    test('trashing returns unit', () async {
      when(() => library.trash(id: 'n_1')).thenAnswer((_) async {});
      expect(
        await repository.trash('n_1'),
        const Right<ApplicationError, Unit>(unit),
      );
    });

    test('a conflict carries the revision the library is at', () async {
      when(
        () => library.save(id: 'n_1', baseRev: '1-a', body: 'x'),
      ).thenAnswer(
        (_) =>
            Future<Saved>.error(const KansoError.conflict(currentRev: '2-b')),
      );
      final result = await repository.save(
        id: 'n_1',
        baseRev: '1-a',
        body: 'x',
      );
      expect(
        (result.swap().toOption().toNullable()! as ConflictError).currentRev,
        '2-b',
      );
    });

    test("the changes are the library's stream", () async {
      when(() => library.changes()).thenAnswer(
        (_) => Stream<List<String>>.value(const <String>['n_1']),
      );
      expect(await repository.changes().first, <String>['n_1']);
    });
  });

  group('RustSessionRepository', () {
    test('a wrong password is false, not an error', () async {
      final locks = _MockLocks();
      when(() => locks.unlock(password: 'nope')).thenAnswer((_) async => false);
      expect(
        await RustSessionRepository(locks).unlock('nope'),
        const Right<ApplicationError, bool>(false),
      );
    });

    test('a panic that got past the library is an internal error', () async {
      final locks = _MockLocks();
      when(locks.isUnlocked).thenAnswer(
        (_) => Future<bool>.error(StateError('x')),
      );
      final result = await RustSessionRepository(locks).isUnlocked();
      expect(result.swap().toOption().toNullable(), isA<UnexpectedError>());
    });
  });
}
