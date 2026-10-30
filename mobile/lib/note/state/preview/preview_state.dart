// Kanso — preview_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'preview_bloc.dart';

@freezed
abstract class PreviewState with _$PreviewState {
  const factory PreviewState({
    /// The note drawn.
    @Default('') String noteId,

    /// The themes it is drawn in.
    @Default((
      ui: 'kanso-light',
      syntax: 'kanso-light',
      preview: 'kanso-light',
      dark: false,
    ))
    PreviewThemes themes,

    /// The note's body size in logical pixels.
    @Default(previewBaseTextSize) double size,

    /// The note's blocks, once they are drawn; the last ones stay while they
    /// are drawn again.
    List<PreviewBlock>? blocks,

    /// The revision of the note the blocks were drawn from: a tick is made
    /// against it.
    @Default('') String rev,

    /// Why the note could not be drawn, when it couldn't.
    ApplicationError? error,

    /// What the screen is to do for the last tap, until it says it has.
    PreviewAction? action,

    /// Why the last tap did nothing, until it is dismissed.
    ApplicationError? notice,
  }) = _PreviewState;
}
