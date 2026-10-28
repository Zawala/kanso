// Kanso — preview_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'preview_bloc.dart';

@freezed
sealed class PreviewEvent with _$PreviewEvent {
  /// Draws the note [noteId] in the built-in theme [themeId].
  const factory PreviewEvent.started(
    String noteId,
    String themeId, {
    String? font,
    String? mono,
    @Default(previewBaseTextSize) double size,
  }) = PreviewStartedEvent;

  /// The theme changed: the page is drawn again in [themeId].
  const factory PreviewEvent.themeChanged(String themeId) =
      PreviewThemeChangedEvent;

  /// Changes note fonts or text size and redraws the page.
  const factory PreviewEvent.styleChanged({
    required double size,
    String? font,
    String? mono,
  }) = PreviewStyleChangedEvent;

  /// Something in the library changed: the note, or a note one of its query
  /// views or links shows.
  const factory PreviewEvent.changesReceived() = PreviewChangesReceivedEvent;

  /// The address [url] was tapped in the page.
  const factory PreviewEvent.linkTapped(String url) = PreviewLinkTappedEvent;

  /// The screen did [PreviewState.action].
  const factory PreviewEvent.actionHandled() = PreviewActionHandledEvent;

  /// The notice of a tap that did nothing was dismissed.
  const factory PreviewEvent.noticeDismissed() = PreviewNoticeDismissedEvent;
}
