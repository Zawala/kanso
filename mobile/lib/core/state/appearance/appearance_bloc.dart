// Kanso — appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/appearance_mode.dart';
import '../../models/data/kanso_font_family.dart';
import '../../models/data/kanso_fonts.dart';
import '../../models/data/kanso_text_size.dart';

part 'appearance_bloc.freezed.dart';
part 'appearance_event.dart';
part 'appearance_state.dart';

/// Holds how the app looks: light, dark or the device's setting, the fonts
/// of notes and the size of their text.
///
/// The choice is kept in memory only; saving it comes with the settings
/// storage.
@injectable
class AppearanceBloc extends Bloc<AppearanceEvent, AppearanceState> {
  AppearanceBloc() : super(const AppearanceState()) {
    on<AppearanceStartedEvent>((event, emit) {
      emit(const AppearanceState());
    });
    on<AppearanceChangedEvent>((event, emit) {
      emit(state.copyWith(mode: event.mode));
    });
    on<AppearanceEditorFontChangedEvent>((event, emit) {
      emit(
        state.copyWith(fonts: state.fonts.copyWith(editorFont: event.font)),
      );
    });
    on<AppearancePreviewFontChangedEvent>((event, emit) {
      emit(
        state.copyWith(fonts: state.fonts.copyWith(previewFont: event.font)),
      );
    });
    on<AppearanceCodeFontChangedEvent>((event, emit) {
      // Code is always monospaced: any other family is ignored.
      if (!event.font.monospaced) {
        return;
      }
      emit(state.copyWith(fonts: state.fonts.copyWith(codeFont: event.font)));
    });
    on<AppearanceTextSizeChangedEvent>((event, emit) {
      emit(state.copyWith(textSize: event.size));
    });
  }
}
