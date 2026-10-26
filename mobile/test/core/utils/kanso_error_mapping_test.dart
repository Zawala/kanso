// Kanso — kanso_error_mapping_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/kanso_error_mapping.dart';

import '../../helpers/property.dart';

void main() {
  group('applicationErrorOf', () {
    test('each kind of the library keeps its meaning', () {
      expect(
        applicationErrorOf(const KansoError.notFound()),
        isA<NotFoundError>(),
      );
      expect(
        applicationErrorOf(const KansoError.locked()),
        isA<LockedError>(),
      );
      expect(
        applicationErrorOf(const KansoError.signedOut()),
        isA<SignedOutError>(),
      );
      expect(
        applicationErrorOf(const KansoError.conflict(currentRev: '2-ab')),
        const ApplicationError.conflict(
          message: 'The note changed meanwhile.',
          currentRev: '2-ab',
        ),
      );
    });

    test('a panic the bridge caught is an internal error', () {
      expect(
        applicationErrorOf(PanicException('boom')),
        const ApplicationError.internal(message: 'boom'),
      );
    });

    test('anything else is unexpected', () {
      expect(applicationErrorOf(StateError('x')), isA<UnexpectedError>());
    });

    Glados(any.kansoError, kansoExploreConfig()).test(
      'every error of the library maps to its own kind, keeping its message',
      (error) {
        final mapped = applicationErrorOf(error);
        switch (error) {
          case KansoError_NotFound():
            expect(mapped, isA<NotFoundError>());
          case KansoError_Locked():
            expect(mapped, isA<LockedError>());
          case KansoError_SignedOut():
            expect(mapped, isA<SignedOutError>());
          case KansoError_Conflict(:final currentRev):
            expect((mapped as ConflictError).currentRev, currentRev);
          case KansoError_Invalid(:final message):
            expect((mapped as InvalidError).message, message);
          case KansoError_Storage(:final message):
            expect((mapped as StorageError).message, message);
          case KansoError_Network(:final message):
            expect((mapped as NetworkError).message, message);
          case KansoError_Auth(:final message):
            expect((mapped as AuthError).message, message);
          case KansoError_Internal(:final message):
            expect((mapped as InternalError).message, message);
        }
      },
    );
  });

  group('attempt', () {
    test('returns what the call returned', () async {
      expect(
        await attempt(() async => 3),
        const Right<ApplicationError, int>(3),
      );
    });

    test('returns the error the call failed with', () async {
      final result = await attempt<int>(
        () => Future<int>.error(const KansoError.locked()),
      );
      expect(
        result,
        const Left<ApplicationError, int>(
          ApplicationError.locked(message: 'The note is locked.'),
        ),
      );
    });

    Glados(any.kansoError, kansoExploreConfig()).test(
      'an error thrown by the call is returned, never rethrown',
      (error) async {
        final result = await attempt<int>(() => Future<int>.error(error));
        expect(result.isLeft(), isTrue);
        expect(result.swap().getOrElse(() => throw StateError('x')), isNotNull);
      },
    );
  });
}
