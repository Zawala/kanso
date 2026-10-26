// Kanso — core_check_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as glados show any;
import 'package:glados/glados.dart' show Glados, IntAnys;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/diagnostics_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core_check/models/data/core_check_status.dart';
import 'package:kanso/core_check/state/core_check/core_check_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/note_rows.dart';
import '../../../helpers/property.dart';

class _MockLibrary extends Mock implements LibraryRepository {}

class _MockDiagnostics extends Mock implements DiagnosticsRepository {}

class _FakeNewNote extends Fake implements NewNote {}

class _FakeListQuery extends Fake implements ListQuery {}

const _error = ApplicationError.storage(message: 'disk');

const _shape = ListShape(
  sort: SortKey.updated,
  byDate: true,
  date: DateField.updated,
  groupPinned: true,
);

void main() {
  late _MockLibrary library;
  late _MockDiagnostics diagnostics;
  late StreamController<List<String>> changes;

  setUpAll(() {
    registerFallbackValue(_FakeNewNote());
    registerFallbackValue(_FakeListQuery());
  });

  setUp(() {
    library = _MockLibrary();
    diagnostics = _MockDiagnostics();
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    when(library.changes).thenAnswer((_) => changes.stream);
    when(
      () => diagnostics.version(),
    ).thenAnswer((_) async => const Right<ApplicationError, String>('0.8.4'));
    when(
      () => diagnostics.tlsReady(),
    ).thenAnswer((_) async => const Right<ApplicationError, bool>(true));
  });

  void notesAre(List<NoteRow> rows) {
    when(() => library.list(any())).thenAnswer(
      (_) async => Right<ApplicationError, NoteList>(
        NoteList(rows: rows, shape: _shape),
      ),
    );
  }

  CoreCheckBloc build() => CoreCheckBloc(library, diagnostics);

  group('CoreCheckBloc', () {
    test('starts loading', () {
      expect(build().state.status, CoreCheckStatus.loading);
    });

    blocTest<CoreCheckBloc, CoreCheckState>(
      'started reads the version, certificates and notes',
      setUp: () => notesAre(<NoteRow>[noteRow('n_1'), noteRow('n_2')]),
      build: build,
      act: (bloc) => bloc.add(const CoreCheckEvent.started()),
      expect: () => <CoreCheckState>[
        const CoreCheckState(),
        CoreCheckState(
          status: CoreCheckStatus.ready,
          version: '0.8.4',
          tlsReady: true,
          noteCount: 2,
          latest: <NoteRow>[noteRow('n_1'), noteRow('n_2')],
        ),
      ],
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'started fails when the library cannot be read',
      setUp: () =>
          when(
            () => library.list(any()),
          ).thenAnswer(
            (_) async => const Left<ApplicationError, NoteList>(_error),
          ),
      build: build,
      act: (bloc) => bloc.add(const CoreCheckEvent.started()),
      expect: () => <CoreCheckState>[
        const CoreCheckState(),
        const CoreCheckState(status: CoreCheckStatus.failed, error: _error),
      ],
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'started again after a failure tries again',
      setUp: () {
        var calls = 0;
        when(() => library.list(any())).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, NoteList>(_error)
              : const Right<ApplicationError, NoteList>(
                  NoteList(rows: <NoteRow>[], shape: _shape),
                );
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const CoreCheckEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CoreCheckEvent.started());
      },
      expect: () => <CoreCheckState>[
        const CoreCheckState(),
        const CoreCheckState(status: CoreCheckStatus.failed, error: _error),
        const CoreCheckState(),
        const CoreCheckState(
          status: CoreCheckStatus.ready,
          version: '0.8.4',
          tlsReady: true,
        ),
      ],
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'a change the library reports refreshes the count and the last change',
      setUp: () => notesAre(<NoteRow>[noteRow('n_1')]),
      build: build,
      act: (bloc) async {
        bloc.add(const CoreCheckEvent.started());
        await Future<void>.delayed(Duration.zero);
        notesAre(<NoteRow>[noteRow('n_2'), noteRow('n_1')]);
        changes.add(<String>['n_2']);
      },
      wait: const Duration(milliseconds: 20),
      skip: 2,
      expect: () => <CoreCheckState>[
        CoreCheckState(
          status: CoreCheckStatus.ready,
          version: '0.8.4',
          tlsReady: true,
          noteCount: 2,
          latest: <NoteRow>[noteRow('n_2'), noteRow('n_1')],
          lastChange: const <String>['n_2'],
        ),
      ],
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'noteCreated makes a note named for the next number',
      setUp: () {
        notesAre(<NoteRow>[noteRow('n_1')]);
        when(() => library.create(any())).thenAnswer(
          (_) async => const Right<ApplicationError, Saved>(
            Saved(id: 'n_2', rev: '1-a'),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const CoreCheckEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CoreCheckEvent.noteCreated());
      },
      skip: 2,
      verify: (_) {
        final made =
            verify(() => library.create(captureAny())).captured.single
                as NewNote;
        expect(made.title, 'Core check 2');
      },
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'noteCreated shows the error when the note could not be made',
      setUp: () {
        notesAre(const <NoteRow>[]);
        when(() => library.create(any())).thenAnswer(
          (_) async => const Left<ApplicationError, Saved>(_error),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const CoreCheckEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CoreCheckEvent.noteCreated());
      },
      skip: 2,
      expect: () => <Matcher>[
        isA<CoreCheckState>().having((s) => s.error, 'error', _error),
      ],
    );

    blocTest<CoreCheckBloc, CoreCheckState>(
      'a locked note is listed by its title',
      setUp: () => notesAre(<NoteRow>[
        noteRow('n_1', title: 'Tax return', locked: true),
      ]),
      build: build,
      act: (bloc) => bloc.add(const CoreCheckEvent.started()),
      verify: (bloc) {
        expect(bloc.state.latest.single.title, 'Tax return');
        expect(bloc.state.latest.single.snippet, isEmpty);
        expect(bloc.state.latest.single.locked, isTrue);
      },
    );

    Glados(glados.any.intInRange(0, 12), kansoExploreConfig()).test(
      'the count is the notes the library lists and the latest are its first '
      'three',
      (count) async {
        final rows = <NoteRow>[
          for (var i = 0; i < count; i++) noteRow('n_$i'),
        ];
        notesAre(rows);
        final bloc = build()..add(const CoreCheckEvent.started());
        await bloc.stream.firstWhere(
          (state) => state.status == CoreCheckStatus.ready,
        );
        expect(bloc.state.noteCount, count);
        expect(bloc.state.latest, rows.take(3).toList());
        await bloc.close();
      },
    );
  });
}
