// Kanso — preview_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'preview_bloc.dart';

@freezed
sealed class PreviewEvent with _$PreviewEvent {
  /// Draws the note [noteId] in [themes], its text [size] points.
  const factory PreviewEvent.started(
    String noteId,
    PreviewThemes themes, {
    @Default(previewBaseTextSize) double size,
  }) = PreviewStartedEvent;

  /// The themes in force changed, or the light or dark slot did: the note is
  /// drawn again in [themes].
  const factory PreviewEvent.themeChanged(PreviewThemes themes) =
      PreviewThemeChangedEvent;

  /// The note's text size changed: the note is drawn again, its math and
  /// drawings at [size].
  const factory PreviewEvent.styleChanged({required double size}) =
      PreviewStyleChangedEvent;

  /// Something in the library changed: the note, or a note one of its query
  /// views or links shows.
  const factory PreviewEvent.changesReceived() = PreviewChangesReceivedEvent;

  /// The address [url] was tapped in the preview.
  const factory PreviewEvent.linkTapped(String url) = PreviewLinkTappedEvent;

  /// The screen did [PreviewState.action].
  const factory PreviewEvent.actionHandled() = PreviewActionHandledEvent;

  /// The notice of a tap that did nothing was dismissed.
  const factory PreviewEvent.noticeDismissed() = PreviewNoticeDismissedEvent;
}
