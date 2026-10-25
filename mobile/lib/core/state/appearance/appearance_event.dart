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

  /// The person picked [font] for writing in the editor.
  const factory AppearanceEvent.editorFontChanged(KansoFontFamily font) =
      AppearanceEditorFontChangedEvent;

  /// The person picked [font] for reading a note.
  const factory AppearanceEvent.previewFontChanged(KansoFontFamily font) =
      AppearancePreviewFontChangedEvent;

  /// The person picked [font] for code; a family that is not monospaced
  /// is ignored.
  const factory AppearanceEvent.codeFontChanged(KansoFontFamily font) =
      AppearanceCodeFontChangedEvent;

  /// The person picked [size] for the text of notes.
  const factory AppearanceEvent.textSizeChanged(KansoTextSize size) =
      AppearanceTextSizeChangedEvent;
}
