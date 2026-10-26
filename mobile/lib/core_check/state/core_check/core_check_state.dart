// Kanso — core_check_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'core_check_bloc.dart';

@freezed
abstract class CoreCheckState with _$CoreCheckState {
  const factory CoreCheckState({
    @Default(CoreCheckStatus.loading) CoreCheckStatus status,

    /// The Rust library's version.
    @Default('') String version,

    /// Whether the library can check a server's certificate.
    @Default(false) bool tlsReady,

    /// The notes outside the trash.
    @Default(0) int noteCount,

    /// The newest few notes: their titles show even when locked.
    @Default(<NoteRow>[]) List<NoteRow> latest,

    /// The IDs in the last batch of changes the library reported.
    @Default(<String>[]) List<String> lastChange,

    /// Why the status is failed, or why the last action did not work.
    ApplicationError? error,
  }) = _CoreCheckState;
}
