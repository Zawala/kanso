// Kanso — licences_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/licence_package.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/settings/state/licences/licences_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _packages = <LicencePackage>[
  LicencePackage(name: 'alpha', licences: <String>['Alpha terms.']),
  LicencePackage(name: 'beta', licences: <String>['One.', 'Two.']),
];

void main() {
  late MockLicencesRepository licences;

  setUp(() {
    licences = MockLicencesRepository();
    when(() => licences.licences()).thenAnswer(
      (_) async =>
          const Right<ApplicationError, List<LicencePackage>>(_packages),
    );
  });

  LicencesBloc build() => LicencesBloc(licences);

  group('LicencesBloc', () {
    test('starts loading, with none open', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.packages, isEmpty);
      expect(state.expanded, isNull);
    });

    blocTest<LicencesBloc, LicencesState>(
      'started reads the packages',
      build: build,
      act: (bloc) => bloc.add(const LicencesEvent.started()),
      expect: () => <LicencesState>[
        const LicencesState(),
        const LicencesState(status: LoadStatus.ready, packages: _packages),
      ],
    );

    blocTest<LicencesBloc, LicencesState>(
      'started reads nothing when the licences cannot be read, then again '
      'when asked to try again',
      setUp: () {
        var calls = 0;
        when(() => licences.licences()).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, List<LicencePackage>>(
                  ApplicationError.unexpected(message: 'Unreadable.'),
                )
              : const Right<ApplicationError, List<LicencePackage>>(
                  _packages,
                );
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LicencesEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const LicencesEvent.started());
      },
      expect: () => <LicencesState>[
        const LicencesState(),
        const LicencesState(status: LoadStatus.failed),
        const LicencesState(),
        const LicencesState(status: LoadStatus.ready, packages: _packages),
      ],
    );

    blocTest<LicencesBloc, LicencesState>(
      'a package opens when toggled, closes when toggled again, and another '
      'package takes its place',
      build: build,
      seed: () =>
          const LicencesState(status: LoadStatus.ready, packages: _packages),
      act: (bloc) => bloc
        ..add(const LicencesEvent.packageToggled('alpha'))
        ..add(const LicencesEvent.packageToggled('beta'))
        ..add(const LicencesEvent.packageToggled('beta')),
      expect: () => <LicencesState>[
        const LicencesState(
          status: LoadStatus.ready,
          packages: _packages,
          expanded: 'alpha',
        ),
        const LicencesState(
          status: LoadStatus.ready,
          packages: _packages,
          expanded: 'beta',
        ),
        const LicencesState(status: LoadStatus.ready, packages: _packages),
      ],
    );
  });
}
