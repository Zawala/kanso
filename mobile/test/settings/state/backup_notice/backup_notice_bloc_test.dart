// Kanso — backup_notice_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:mocktail/mocktail.dart';

class _MockPreferences extends Mock implements PreferencesRepository {}

void main() {
  late _MockPreferences preferences;
  late Preferences saved;

  setUpAll(() {
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    preferences = _MockPreferences();
    saved = const Preferences();
    when(
      preferences.load,
    ).thenAnswer((_) async => Right<ApplicationError, Preferences>(saved));
    when(() => preferences.update(any())).thenAnswer((call) async {
      final change =
          call.positionalArguments.first as Preferences Function(Preferences);
      saved = change(saved);
      return Right<ApplicationError, Preferences>(saved);
    });
  });

  BackupNoticeBloc build() => BackupNoticeBloc(preferences);

  group('BackupNoticeBloc', () {
    test('starts hidden: it has not read the preferences yet', () {
      expect(build().state.visible, isFalse);
    });

    blocTest<BackupNoticeBloc, BackupNoticeState>(
      'shows the notice on the first launch',
      build: build,
      act: (bloc) => bloc.add(const BackupNoticeEvent.started()),
      expect: () => <BackupNoticeState>[
        const BackupNoticeState(visible: true),
      ],
    );

    blocTest<BackupNoticeBloc, BackupNoticeState>(
      'does not show it again once seen (the first reading says so)',
      setUp: () => saved = const Preferences(backupNoticeSeen: true),
      build: build,
      act: (bloc) => bloc.add(const BackupNoticeEvent.started()),
      expect: () => <BackupNoticeState>[const BackupNoticeState()],
    );

    blocTest<BackupNoticeBloc, BackupNoticeState>(
      'shows it when the preferences cannot be read',
      setUp: () {
        when(preferences.load).thenAnswer(
          (_) async => const Left<ApplicationError, Preferences>(
            ApplicationError.storage(message: 'Unreadable.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const BackupNoticeEvent.started()),
      expect: () => <BackupNoticeState>[
        const BackupNoticeState(visible: true),
      ],
    );

    blocTest<BackupNoticeBloc, BackupNoticeState>(
      'OK hides the notice and remembers that it was seen',
      build: build,
      act: (bloc) async {
        bloc.add(const BackupNoticeEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const BackupNoticeEvent.dismissed());
      },
      expect: () => <BackupNoticeState>[
        const BackupNoticeState(visible: true),
        const BackupNoticeState(),
      ],
      verify: (_) {
        expect(saved.backupNoticeSeen, isTrue);
      },
    );

    blocTest<BackupNoticeBloc, BackupNoticeState>(
      'OK hides the notice even when it cannot be remembered',
      setUp: () {
        when(() => preferences.update(any())).thenAnswer(
          (_) async => const Left<ApplicationError, Preferences>(
            ApplicationError.storage(message: 'Disk full.'),
          ),
        );
      },
      build: build,
      seed: () => const BackupNoticeState(visible: true),
      act: (bloc) => bloc.add(const BackupNoticeEvent.dismissed()),
      expect: () => <BackupNoticeState>[const BackupNoticeState()],
    );
  });
}
