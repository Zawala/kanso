// Kanso — application_error_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';

void main() {
  group('ApplicationError', () {
    test('errors with the same kind and message are equal', () {
      expect(
        const ApplicationError.storage(message: 'disk full'),
        const ApplicationError.storage(message: 'disk full'),
      );
    });

    test('errors of different kinds are not equal', () {
      expect(
        const ApplicationError.notFound(message: 'gone'),
        isNot(const ApplicationError.locked(message: 'gone')),
      );
    });

    test('a switch covers every kind', () {
      String describe(ApplicationError error) => switch (error) {
        UnexpectedError() => 'unexpected',
        NotFoundError() => 'notFound',
        LockedError() => 'locked',
        StorageError() => 'storage',
        ConflictError() => 'conflict',
        InvalidError() => 'invalid',
        NetworkError() => 'network',
        SignedOutError() => 'signedOut',
        AuthError() => 'auth',
        InternalError() => 'internal',
      };
      expect(
        describe(const ApplicationError.locked(message: 'x')),
        'locked',
      );
    });
  });
}
