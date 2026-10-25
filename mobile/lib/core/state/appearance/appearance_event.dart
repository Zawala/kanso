// Kanso — appearance_event.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'appearance_bloc.dart';

@freezed
sealed class AppearanceEvent with _$AppearanceEvent {
  /// The app started: show the default appearance.
  const factory AppearanceEvent.started() = AppearanceStartedEvent;

  /// The person picked [mode].
  const factory AppearanceEvent.changed(AppearanceMode mode) =
      AppearanceChangedEvent;
}
