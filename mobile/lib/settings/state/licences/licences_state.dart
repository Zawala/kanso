// Kanso — licences_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'licences_bloc.dart';

@freezed
abstract class LicencesState with _$LicencesState {
  const factory LicencesState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<LicencePackage>[]) List<LicencePackage> packages,

    /// The package whose licences are shown; null when none are.
    String? expanded,
  }) = _LicencesState;
}
