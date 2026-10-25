// Kanso — appearance_bloc.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/appearance_mode.dart';

part 'appearance_bloc.freezed.dart';
part 'appearance_event.dart';
part 'appearance_state.dart';

/// Holds whether the app is light, dark or follows the device.
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
  }
}
