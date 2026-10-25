// Kanso — appearance_state.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'appearance_bloc.dart';

@freezed
abstract class AppearanceState with _$AppearanceState {
  const factory AppearanceState({
    @Default(AppearanceMode.system) AppearanceMode mode,
  }) = _AppearanceState;
}
