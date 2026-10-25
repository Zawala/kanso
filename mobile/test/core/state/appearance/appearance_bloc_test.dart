// Kanso — appearance_bloc_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/appearance_mode.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';

void main() {
  group('AppearanceBloc', () {
    test('starts following the system', () {
      expect(AppearanceBloc().state.mode, AppearanceMode.system);
    });

    blocTest<AppearanceBloc, AppearanceState>(
      'started announces the default appearance',
      build: AppearanceBloc.new,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[const AppearanceState()],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'changed emits the chosen mode',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.changed(AppearanceMode.dark))
        ..add(const AppearanceEvent.changed(AppearanceMode.light)),
      expect: () => <AppearanceState>[
        const AppearanceState(mode: AppearanceMode.dark),
        const AppearanceState(mode: AppearanceMode.light),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'choosing the same mode twice emits once',
      build: AppearanceBloc.new,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.changed(AppearanceMode.dark))
        ..add(const AppearanceEvent.changed(AppearanceMode.dark)),
      expect: () => <AppearanceState>[
        const AppearanceState(mode: AppearanceMode.dark),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started after a change goes back to the system',
      build: AppearanceBloc.new,
      seed: () => const AppearanceState(mode: AppearanceMode.dark),
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[const AppearanceState()],
    );
  });
}
