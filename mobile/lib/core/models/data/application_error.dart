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
}
