// Kanso — application_error.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'application_error.freezed.dart';

/// Why an operation failed, in terms the app can act on.
///
/// [message] is for diagnostics; screens show their own localised copy.
@freezed
sealed class ApplicationError with _$ApplicationError {
  /// Something the app did not expect to go wrong.
  const factory ApplicationError.unexpected({required String message}) =
      UnexpectedError;

  /// A note, notebook or other item that does not exist (any more).
  const factory ApplicationError.notFound({required String message}) =
      NotFoundError;

  /// The item is locked and needs unlocking first.
  const factory ApplicationError.locked({required String message}) =
      LockedError;

  /// Reading or writing the device's storage failed.
  const factory ApplicationError.storage({required String message}) =
      StorageError;

  /// The note changed since the revision the caller saw; [currentRev] is its
  /// revision now (empty when the library can't say).
  const factory ApplicationError.conflict({
    required String message,
    @Default('') String currentRev,
  }) = ConflictError;

  /// The request was wrong: a name too long, a password too short, a query
  /// that does not parse.
  const factory ApplicationError.invalid({required String message}) =
      InvalidError;

  /// The server could not be reached, or answered with a problem that a later
  /// try may clear.
  const factory ApplicationError.network({required String message}) =
      NetworkError;

  /// No account is signed in, or the server ended this device's session.
  const factory ApplicationError.signedOut({required String message}) =
      SignedOutError;

  /// The server or the phone's secret store refused: a wrong password, a
  /// turned-off account, a locked Keychain.
  const factory ApplicationError.auth({required String message}) = AuthError;

  /// A bug: a call in the library failed in a way it should not have. The
  /// library still works.
  const factory ApplicationError.internal({required String message}) =
      InternalError;
}
