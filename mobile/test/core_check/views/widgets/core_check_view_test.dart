// Kanso — core_check_view_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core_check/models/data/core_check_status.dart';
import 'package:kanso/core_check/state/core_check/core_check_bloc.dart';
import 'package:kanso/core_check/views/widgets/core_check_view.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/note_rows.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/seeded_core_check_bloc.dart';

class _MockCoreCheckBloc extends MockBloc<CoreCheckEvent, CoreCheckState>
    implements CoreCheckBloc {}

final _ready = CoreCheckState(
  status: CoreCheckStatus.ready,
  version: '0.8.4',
  tlsReady: true,
  noteCount: 12,
  latest: <NoteRow>[
    noteRow('n_1', title: 'Core check 12'),
    noteRow('n_2', title: 'Tax return', locked: true),
    noteRow('n_3', title: 'Reading list'),
  ],
  lastChange: const <String>['n_1'],
);

Widget _view(CoreCheckState state) {
  final bloc = SeededCoreCheckBloc(state);
  addTearDown(bloc.close);
  return BlocProvider<CoreCheckBloc>.value(
    value: bloc,
    child: const CoreCheckView(),
  );
}

void main() {
  group('CoreCheckView', () {
    testWidgets('phone, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _view(_ready),
        name: 'phone_core_check_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('phone, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _view(_ready),
        name: 'phone_core_check_dark',
        size: phoneSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('tablet, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _view(_ready),
        name: 'tablet_core_check_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('tablet, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _view(_ready),
        name: 'tablet_core_check_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });

    testWidgets('phone, failed', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _view(
          const CoreCheckState(
            status: CoreCheckStatus.failed,
            error: ApplicationError.storage(message: 'disk'),
          ),
        ),
        name: 'phone_core_check_failed_light',
        size: phoneSize,
        brightness: Brightness.light,
      );
    });
  });

  group('CoreCheckView facts', () {
    testWidgets('shows what the library said', (tester) async {
      await pumpKanso(tester, _view(_ready));
      expect(find.text('0.8.4'), findsOneWidget);
      expect(find.text('Ready'), findsOneWidget);
      expect(find.text('12'), findsOneWidget);
      expect(find.text('1 note changed'), findsOneWidget);
      // A locked note is listed by its title, with the lock's icon.
      expect(find.text('Tax return'), findsOneWidget);
    });

    testWidgets('the button asks for a new note', (tester) async {
      final bloc = _MockCoreCheckBloc();
      whenListen(
        bloc,
        const Stream<CoreCheckState>.empty(),
        initialState: _ready,
      );
      await pumpKanso(
        tester,
        BlocProvider<CoreCheckBloc>.value(
          value: bloc,
          child: const CoreCheckView(),
        ),
      );
      await tester.tap(find.text('New note'));
      verify(() => bloc.add(const CoreCheckEvent.noteCreated())).called(1);
    });

    testWidgets('the error card offers to try again', (tester) async {
      final bloc = _MockCoreCheckBloc();
      whenListen(
        bloc,
        const Stream<CoreCheckState>.empty(),
        initialState: const CoreCheckState(status: CoreCheckStatus.failed),
      );
      await pumpKanso(
        tester,
        BlocProvider<CoreCheckBloc>.value(
          value: bloc,
          child: const CoreCheckView(),
        ),
      );
      await tester.tap(find.text('Try again'));
      verify(() => bloc.add(const CoreCheckEvent.started())).called(1);
    });

    testWidgets('while loading it says so', (tester) async {
      await pumpKanso(tester, _view(const CoreCheckState()));
      expect(find.text('Opening the library…'), findsOneWidget);
    });
  });
}
