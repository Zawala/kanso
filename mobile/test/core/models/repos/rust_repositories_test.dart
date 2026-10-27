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

const _query = ListQuery(
  place: ListPlace.all(),
  pinnedOnly: false,
  sort: SortKey.updated,
  descending: true,
);

const _shape = ListShape(
  sort: SortKey.updated,
  byDate: true,
  date: DateField.updated,
  groupPinned: true,
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
      const empty = NoteList(rows: <NoteRow>[], shape: _shape);
      when(() => library.list(query: _query)).thenAnswer((_) async => empty);
      expect(
        await repository.list(_query),
        const Right<ApplicationError, NoteList>(empty),
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

    test(
      'a change returns how many notes were written, and its ticket',
      () async {
        const change = NoteChange.move(notebookId: 'b_2');
        when(
          () => library.change(ids: <String>['n_1', 'n_2'], change: change),
        ).thenAnswer(
          (_) async => const Changed(
            count: 2,
            undo: 5,
            message: 'Moved 2 notes to Work.',
          ),
        );
        expect(
          await repository.change(<String>['n_1', 'n_2'], change),
          const Right<ApplicationError, Changed>(
            Changed(count: 2, undo: 5, message: 'Moved 2 notes to Work.'),
          ),
        );
      },
    );

    test('a change into a locked notebook is a locked error', () async {
      when(
        () => library.change(
          ids: <String>['n_1'],
          change: const NoteChange.togglePin(),
        ),
      ).thenAnswer((_) => Future<Changed>.error(const KansoError.locked()));
      final result = await repository.change(
        <String>['n_1'],
        const NoteChange.togglePin(),
      );
      expect(result.swap().toOption().toNullable(), isA<LockedError>());
    });

    test(
      'a search looks in all by default and in the scope it is given',
      () async {
        when(
          () => library.search(
            text: 'tulips',
            scope: const SearchScope.all(),
            limit: 50,
          ),
        ).thenAnswer((_) async => const <Hit>[]);
        when(
          () => library.search(
            text: 'tulips',
            scope: const SearchScope.notebook(id: 'b_1'),
            limit: 5,
          ),
        ).thenAnswer((_) async => const <Hit>[]);
        expect(
          await repository.search('tulips'),
          const Right<ApplicationError, List<Hit>>(<Hit>[]),
        );
        expect(
          await repository.search(
            'tulips',
            scope: const SearchScope.notebook(id: 'b_1'),
            limit: 5,
          ),
          const Right<ApplicationError, List<Hit>>(<Hit>[]),
        );
      },
    );

    test('a notebook is made inside its parent', () async {
      const made = NotebookRow(
        id: 'b_2',
        name: 'Plans',
        parentId: 'b_1',
        count: 0,
        locked: false,
        depth: 1,
      );
      when(
        () => library.createNotebook(name: 'Plans', parentId: 'b_1'),
      ).thenAnswer((_) async => made);
      expect(
        await repository.createNotebook('Plans', parentId: 'b_1'),
        const Right<ApplicationError, NotebookRow>(made),
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
