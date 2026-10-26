// Kanso — sync_status_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'sync_status_bloc.dart';

@freezed
abstract class SyncStatusState with _$SyncStatusState {
  const factory SyncStatusState({
    /// The status of the account and its sync; null until it is first read.
    SyncStatus? status,

    /// Whether a sync the person asked for is running.
    @Default(false) bool syncing,

    /// Why the last sync the person asked for failed.
    ApplicationError? error,
  }) = _SyncStatusState;

  const SyncStatusState._();

  /// Whether an account is signed in on this phone.
  bool get signedIn {
    final phase = status?.phase;
    return phase != null && phase != SyncPhase.signedOut;
  }

  /// Whether a sync is running, asked for or not.
  bool get busy => syncing || status?.phase == SyncPhase.syncing;
}
