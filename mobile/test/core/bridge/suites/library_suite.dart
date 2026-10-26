// Kanso — library_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The notes of a library, through the bridge: every call of `KansoLibrary`.
/// It runs on this computer (`test/core/bridge`) and on a simulator or an
/// emulator (`integration_test`).
void librarySuite(OpenLibrary open) {
  group('KansoLibrary', () {
    test('a new library holds no notes', () async {
      final library = await open();
      expect(await library.notes(query: everyNote), isEmpty);
      expect(await library.tags(), isEmpty);
    });

    test('creates, reads, saves and trashes a note', () async {
      final library = await open();
      final saved = await library.create(
        note: const NewNote(title: 'First', body: 'hello **world**'),
      );
      expect(saved.id, isNotEmpty);

      final rows = await library.notes(query: everyNote);
      expect(rows, hasLength(1));
      expect(rows.single.id, saved.id);
      expect(rows.single.title, 'First');
      expect(rows.single.snippet, contains('hello'));
      expect(rows.single.locked, isFalse);

      final note = await library.note(id: saved.id);
      expect(note.body, 'hello **world**');
      expect(note.lock, isA<LockState_Plain>());
      expect(note.rev, saved.rev);

      final changed = await library.save(
        id: saved.id,
        baseRev: note.rev,
        body: 'changed text',
      );
      expect(changed.rev, isNot(saved.rev));
      expect((await library.note(id: saved.id)).body, 'changed text');

      await library.trash(id: saved.id);
      expect(await library.notes(query: everyNote), isEmpty);
      final trashed = await library.notes(query: trashedNotes);
      expect(trashed.map((row) => row.id), <String>[saved.id]);
    });

    test(
      'a stale save is a conflict and an unknown note is not found',
      () async {
        final library = await open();
        final saved = await library.create(
          note: const NewNote(title: 'Plan', body: 'one'),
        );
        await library.save(id: saved.id, baseRev: saved.rev, body: 'two');
        await expectLater(
          library.save(id: saved.id, baseRev: saved.rev, body: 'three'),
          throwsA(isA<KansoError_Conflict>()),
        );
        await expectLater(
          library.note(id: 'note:nope'),
          throwsA(isA<KansoError_NotFound>()),
        );
      },
    );

    test('lists notebooks with their counts', () async {
      final library = await open();
      await makeNote(library, 'One', 'text');
      await makeNote(library, 'Two', 'text');
      final notebooks = await library.notebooks();
      expect(notebooks, isNotEmpty);
      expect(notebooks.fold<int>(0, (sum, book) => sum + book.count), 2);
      final inBook = await library.notes(
        query: NoteQuery(
          notebookId: notebooks.first.id,
          trash: false,
          limit: 0,
          skip: 0,
          sort: SortKey.title,
          descending: false,
        ),
      );
      expect(inBook.map((row) => row.title), <String>['One', 'Two']);
    });

    test('search finds a note and says where the words matched', () async {
      final library = await open();
      await makeNote(library, 'Grocery list', 'buy tulips on Tuesday');
      await makeNote(library, 'Other', 'nothing here');
      final hits = await library.search(text: 'tulips', limit: 10);
      expect(hits, hasLength(1));
      expect(hits.single.row.title, 'Grocery list');
      expect(hits.single.snippetRanges, isNotEmpty);
      final range = hits.single.snippetRanges.first;
      expect(
        hits.single.row.snippet.substring(range.start, range.end),
        'tulips',
      );
      expect(await library.search(text: 'zebra', limit: 10), isEmpty);
    });

    test('the properties of a note are its front matter', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'P',
        '---\nrating: 4.5\ndone: true\nowner: Ada\n---\nbody',
      );
      final props = await library.properties(id: id);
      expect(props.map((p) => p.key), <String>['rating', 'done', 'owner']);
      expect(props.map((p) => p.kind), <PropertyKind>[
        PropertyKind.number,
        PropertyKind.bool,
        PropertyKind.text,
      ]);
      expect(props.last.text, 'Ada');
      final none = await makeNote(library, 'None', 'no front matter');
      expect(await library.properties(id: none), isEmpty);
    });

    test('a query view is a table, and a bad one is refused', () async {
      final library = await open();
      final a = await makeNote(
        library,
        'Alpha',
        '---\nstatus: open\nrank: 2\n---\n',
      );
      final b = await makeNote(
        library,
        'Beta',
        '---\nstatus: open\nrank: 1\n---\n',
      );
      final table = await library.queryView(
        source:
            'from: status:none has:rank\ncolumns: rank, status\n'
            'sort: rank asc',
      );
      expect(table.columns, <String>['Title', 'rank', 'status']);
      expect(table.rows.map((row) => row.id), <String>[b, a]);
      expect(table.rows.first.cells, <String?>['1', 'open']);
      final without = await library.queryView(
        source: 'from: status:none has:rank',
        selfId: a,
      );
      expect(without.rows.map((row) => row.id), <String>[b]);
      await expectLater(
        library.queryView(source: 'nonsense'),
        throwsA(isA<KansoError_Invalid>()),
      );
    });

    test('the changes stream reports what is written', () async {
      final library = await open();
      final written = <String>{};
      final heard = <String>{};
      final reported = Completer<void>();
      final subscription = library.changes().listen((ids) {
        heard.addAll(ids);
        if (heard.any(written.contains) && !reported.isCompleted) {
          reported.complete();
        }
      });
      // Cancelling waits for the library's next event: don't wait for it.
      addTearDown(() => unawaited(subscription.cancel()));
      // The stream is set up on a thread of the library's, so a note written
      // at once may come before it: write until one is reported. (The ID of
      // a new notebook may come first; it is not a note written here.)
      for (var i = 0; i < 40 && !reported.isCompleted; i++) {
        written.add(await makeNote(library, 'Change $i', 'text'));
        await Future.any(<Future<void>>[
          reported.future,
          Future<void>.delayed(const Duration(milliseconds: 250)),
        ]);
      }
      await reported.future.timeout(const Duration(seconds: 5));
      expect(heard.any(written.contains), isTrue);
    });

    test('the parts of a library come from it', () async {
      final library = await open();
      expect(library.locks(), isA<Object>());
      expect(library.syncing(), isA<Object>());
      expect(library.renderer(), isA<Object>());
    });
  });
}
