// Kanso — theme_picker_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../models/data/theme_preview.dart';
import '../../utils/theme_preview_from_builtin.dart';

part 'theme_picker_bloc.freezed.dart';
part 'theme_picker_event.dart';
part 'theme_picker_state.dart';

/// The themes the picker offers: the built-in ones in the desktop's order,
/// each with the preview drawn from its tokens.
///
/// Which are in use, and picking one, are the `AppearanceBloc`'s.
@injectable
class ThemePickerBloc extends Bloc<ThemePickerEvent, ThemePickerState> {
  ThemePickerBloc(this._renderer) : super(const ThemePickerState()) {
    on<ThemePickerStartedEvent>(_onStarted, transformer: restartable());
  }

  final RendererRepository _renderer;

  Future<void> _onStarted(
    ThemePickerStartedEvent event,
    Emitter<ThemePickerState> emit,
  ) async {
    emit(const ThemePickerState());
    final result = await _renderer.builtinThemes();
    emit(
      result.fold(
        (error) => const ThemePickerState(status: LoadStatus.failed),
        (themes) => ThemePickerState(
          status: LoadStatus.ready,
          themes: <ThemePreview>[...themes.map(themePreviewFromBuiltin)],
        ),
      ),
    );
  }
}
