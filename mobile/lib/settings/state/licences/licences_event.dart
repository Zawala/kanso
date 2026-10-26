// Kanso — licences_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'licences_bloc.dart';

@freezed
sealed class LicencesEvent with _$LicencesEvent {
  /// The page was opened, or the person asked to try again: read the
  /// licences.
  const factory LicencesEvent.started() = LicencesStartedEvent;

  /// The person opened or closed the licences of the package [name].
  const factory LicencesEvent.packageToggled(String name) =
      LicencesPackageToggledEvent;
}
