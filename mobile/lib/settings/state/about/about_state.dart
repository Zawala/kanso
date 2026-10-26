// Kanso — about_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'about_bloc.dart';

@freezed
abstract class AboutState with _$AboutState {
  const factory AboutState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The version of the app; null until it is read.
    AppVersion? version,
  }) = _AboutState;
}
